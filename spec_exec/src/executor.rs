//! The executor, which actually runs commands.

use std::process::{Command, Stdio, ExitStatus};
use std::rc::Rc;
use std::os::unix::process::ExitStatusExt;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, Instant, Duration};
use std::thread::spawn as spawn_thread;

use log::*;

use crate::error::ExecError;


/// All commands are run through an **executor**, which not only
/// executes actual Commands, but also records their exit status and
/// outputs so that they can be referred to later.
#[derive(Debug)]
pub struct Executor {
    command_history: CommandHistory,
}

#[derive(Debug)]
struct CommandHistory(Vec<Rc<RanCommand>>);

impl Executor {

    /// Creates a new executor with an empty command history.
    pub fn new() -> Self {
        Executor {
            command_history: CommandHistory(Vec::new()),
        }
    }

    /// Runs the given Command and stores its results in the command history.
    pub fn run_and_store(&mut self, mut command: Command) -> Result<Rc<RanCommand>, ExecError> {
        use std::io::{BufReader, BufRead};

        // Set up the command I/O so we can read its output.
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Spawn the command and get its output pipes
        info!("Spawning command -> {:?}", command);
        let timer = Instant::now();
        let mut child = match command.spawn() {
            Ok(c) => c,
            Err(e) => return Err(ExecError::Spawn(e)),
        };

        let stdout = BufReader::new(child.stdout.take().unwrap());
        let stderr = BufReader::new(child.stderr.take().unwrap());

        // I had loads of trouble reading from stdout and stderr at the
        // same time. Then I had even more trouble reading into a Vec
        // from another thread, so I just loaded up on wrapping types
        // until it compiled. Rust™
        let mut stdout_lines = Vec::new();
        let stderr_lines_tmp = Arc::new(Mutex::new(Vec::new()));

        let tmp2 = Arc::clone(&stderr_lines_tmp);
        let thread = spawn_thread(move || {
            let mut tmp = tmp2.lock().unwrap();
            for line in stderr.lines() {
                let line = line.unwrap();  // this is not the same as the one below!
                tmp.push((SystemTime::now(), line));
            }
        });

        for line in stdout.lines() {
            let rc = Rc::from(line.map_err(ExecError::Stdout)?);
            stdout_lines.push(OutputLine { timestamp: SystemTime::now(), line: rc });
        }

        thread.join().unwrap();

        let stderr_lines = Arc::try_unwrap(stderr_lines_tmp).unwrap().into_inner().unwrap().into_iter().map(|(timestamp, line)| {   // ugh
            let rc = Rc::from(line);
            OutputLine { timestamp, line: rc }
        }).collect::<Vec<_>>();

        // Wait until the process has finished executing, and measure how long
        // it took to run
        let exit = child.wait().map_err(ExecError::Wait)?;
        let runtime = timer.elapsed();
        debug!("Command complete in -> {:?}", runtime);

        // Store the command results in the history
        let rc = self.command_history.store(RanCommand {
            invocation: format!("{:?}", command),
            exit_reason: ExitReason::from(exit),
            stdout_lines, stderr_lines, runtime,
        });

        // Finally, return the shared reference to the result
        Ok(rc)
    }

    /// Returns a list of references to the commands that have been run.
    /// This data is used to populate the result documents.
    pub fn to_commands(&self) -> impl Iterator<Item=&RanCommand> {
        self.command_history.0.iter()
            .map(|rc| Rc::as_ref(rc))
    }
}

impl CommandHistory {

    /// Stores the command we’ve just run in the history, and returns a
    /// reference to it.
    fn store(&mut self, ran_command: RanCommand) -> Rc<RanCommand> {
        let rc = Rc::new(ran_command);
        self.0.push(Rc::clone(&rc));
        rc
    }
}


/// The results of a command that has been executed.
#[derive(Debug)]
pub struct RanCommand {

    /// The shell that it was executed with.
    pub invocation: String,

    // todo: also store the environment and directory

    /// The reason the process exited.
    pub exit_reason: ExitReason,

    /// The process’s lines of standard output, timestamped.
    pub stdout_lines: Vec<OutputLine>,

    /// The process’s lines of standard error, timestamped.
    pub stderr_lines: Vec<OutputLine>,

    /// The amount of time the process took to run.
    pub runtime: Duration,
}

impl RanCommand {

    /// Returns the list of output lines, as untimestamped strings, from
    /// the completed process.
    pub fn stdout_lines(&self) -> Vec<Rc<str>> {
        self.stdout_lines.iter()
            .map(|e| Rc::clone(&e.line))
            .collect()
    }

    /// Returns the bytes of the completed process’s standard output stream,
    /// albeit after UTF-8 encoding and decoding.
    pub fn stdout_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        for line in &self.stdout_lines {
            v.extend(line.line.as_bytes());
            v.extend(b"\n");
        }
        v
    }

    /// Returns the bytes of the completed process’s standard error stream,
    /// albeit after UTF-8 encoding and decoding.
    pub fn stderr_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        for line in &self.stderr_lines {
            v.extend(line.line.as_bytes());
            v.extend(b"\n");
        }
        v
    }
}


/// A line of output text that we have read from a command.
#[derive(Debug, Clone)]
pub struct OutputLine {

    /// The current time at the instant we read the line.
    pub timestamp: SystemTime,

    /// The text that was read.
    pub line: Rc<str>,
}

/// The reason a process exited.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ExitReason {

    /// It exited with the given exit status.
    Status(i32),

    /// It was killed with the given signal.
    Signal(i32),

    /// It exited for a reason that’s neither of those two.
    Unknown,

    /// The process did not run at all; its output was overridden, and we can
    /// assume that an overridden command ‘succeeded’.
    Overridden,
}

impl From<ExitStatus> for ExitReason {

    /// Convert the OS’s representation of an exit status into our
    /// representation of one.
    fn from(es: ExitStatus) -> Self {
        if let Some(code) = es.code() {
            Self::Status(code)
        }
        else if let Some(signal) = es.signal() {
            Self::Signal(signal)
        }
        else {
            Self::Unknown
        }
    }
}

impl ExitReason {

    /// Whether this exit reason is because the process exited with
    /// the given status.
    pub fn is(self, expected_status: u8) -> bool {
        if let Self::Overridden = self {
            true
        }
        else if let Self::Status(got_status) = self {
            i32::from(expected_status) == got_status
        }
        else {
            false
        }
    }

    /// Returns an error if this exit reason is _not_ because the process
    /// exited with the given status. This is used by the checks after running
    /// commands, to make sure they ran successfully.
    pub fn should_be(self, expected_status: u8) -> Result<(), ExecError> {
        if self.is(expected_status) {
            Ok(())
        }
        else {
            Err(ExecError::StatusMismatch(self))
        }
    }
}
