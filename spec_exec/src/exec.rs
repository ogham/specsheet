//! The Exec type and its methods.


use std::fmt;
use std::rc::Rc;
use std::sync::Mutex;
pub use std::process::Command;

use log::*;

use crate::error::ExecError;
use crate::executor::{Executor, RanCommand, ExitReason};


/// An **Exec** is the main thing that Specsheet deals with. It
/// encapsulates a runnable process as something that can be executed
/// to produce output in the form of some type `T`.
#[derive(Debug)]
pub struct Exec<T>(Inner<T>);

/// The inner, non-public internal mechanisms of Execs.
///
/// There are two kinds of Exec: one kind that’s been overridden to
/// produce an existing piece of output, and the default kind that
/// runs processes normally.
#[derive(Debug)]
enum Inner<T> {

    /// This exec has been overridden by user input, producing the
    /// given output thing.
    Predetermined {

        /// The argument this was read from.
        name: &'static str,

        /// A value created from the user’s input.
        object: Rc<T>,
    },

    /// This exec runs a process and uses its exit status and output
    /// streams. The state of the process is stored behind a Mutex.
    Invocation(Mutex<State<T>>),
}

/// The internal state of a process-running Exec.
#[derive(Debug)]
enum State<T> {

    /// This Exec is currently being run, and as proof, we have the Command it
    /// was launched by.
    Primed(Command),

    /// Temporary state while the command is being run.
    Running,

    /// This Exec has already run and succeeded, producing the output value
    /// created from its lines (as long as it’s not raw)
    Completed(Rc<RanCommand>, Option<Rc<T>>),

    /// This Exec has already run and failed.
    Attempted(Rc<ExecError>),
}

/// Common trait for all the output types.
pub trait CommandOutput: Sized {

    /// Determine whether a process succeeded by examining its exit reason and
    /// standard output lines. Usually, an exit status of 0 signifies success,
    /// and the output format is up to the command.
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError>;
}


impl<T> Exec<T> {

    /// Create a new Exec with a command to run.
    pub fn actual(command: Command) -> Self {
        Self(Inner::Invocation(Mutex::new(State::Primed(command))))
    }

    /// Create a new Exec that’s already been executed, with some pre-existing
    /// output and the command-line argument it came from.
    pub fn predetermined(name: &'static str, object: T) -> Self {
        Self(Inner::Predetermined { name, object: Rc::new(object) })
    }
}

impl<T: CommandOutput> Exec<T> {

    /// Run the loaded command, producing an output value shared with the
    /// executor as well as its exit status, or an error if it fails. Non-zero
    /// exit statuses are not counted as failures.
    pub fn run(&self, executor: &mut Executor) -> Result<Rc<T>, Rc<ExecError>> {
        use std::mem;

        // An overridden Exec has been “run” with some output already.
        let mutex = match self {
            Self(Inner::Predetermined { object, .. })  => return Ok(Rc::clone(object)),
            Self(Inner::Invocation(mutex))             => mutex,
        };

        // Lock the mutex until the command has been run
        let mut state = mutex.lock().unwrap();
        match &*state {
            State::Primed(_)                => {/* continue further */},
            State::Running                  => unreachable!("State still running"),
            State::Completed(_rc, Some(t))  => return Ok(Rc::clone(t)),
            State::Completed(_rc, None)     => unreachable!("No output value"),
            State::Attempted(err)           => return Err(Rc::clone(err)),
        }

        // We need to temporarily set the state to Running in order to
        // move the Primed state out
        let old_state = mem::replace(&mut *state, State::Running);

        // Extract the variables we skipped over earlier
        let cmd = match old_state {
            State::Primed(cmd)  => cmd,
            _                   => unreachable!(),
        };

        // Then just set the state based on how running it goes
        match executor.run_and_store(cmd) {
            Ok(ran_command) => {
                let er = ran_command.exit_reason;
                match T::interpret_command_output(ran_command.stdout_lines(), er) {
                    Ok(t) => {
                        let rc_t = Rc::new(t);
                        *state = State::Completed(ran_command, Some(Rc::clone(&rc_t)));
                        Ok(rc_t)
                    }
                    Err(e) => {
                        let rc = Rc::new(e);
                        *state = State::Attempted(Rc::clone(&rc));
                        // todo: put the failure reason in Attempted somewhere
                        Err(rc)
                    }
                }
            }
            Err(e) => {
                let rc = Rc::new(e);
                *state = State::Attempted(Rc::clone(&rc));
                Err(rc)
            }
        }
    }
}

impl Exec<RanCommand> {

    /// Runs a command, like `run`, but does not try to interpret the
    /// output, instead returning the raw `RanCommand`.
    pub fn run_raw(&self, executor: &mut Executor) -> Result<Rc<RanCommand>, Rc<ExecError>> {
        use std::mem;

        // An overridden Exec has been “run” with some output already.
        let mutex = match self {
            Self(Inner::Predetermined { .. })  => unreachable!(),
            Self(Inner::Invocation(mutex))     => mutex,
        };

        // Lock the mutex until the command has been run
        let mut state = mutex.lock().unwrap();
        match &*state {
            State::Primed(_)         => {/* continue further */},
            State::Running           => unreachable!("State still running"),
            State::Completed(rc, _)  => return Ok(Rc::clone(rc)),
            State::Attempted(err)    => return Err(Rc::clone(err)),
        }

        // We need to temporarily set the state to Running in order to
        // move the Primed state out
        let old_state = mem::replace(&mut *state, State::Running);

        // Extract the variables we skipped over earlier
        let cmd = match old_state {
            State::Primed(cmd)          => cmd,
            State::Completed(rc, None)  => return Ok(Rc::clone(&rc)),
            _                           => unreachable!(),
        };

        // Then just set the state based on how running it goes
        match executor.run_and_store(cmd) {
            Ok(ran_command) => {
                let rc_t = Rc::clone(&ran_command);
                *state = State::Completed(ran_command, None);
                Ok(rc_t)
            }
            Err(e) => {
                let rc = Rc::new(e);
                *state = State::Attempted(Rc::clone(&rc));
                Err(rc)
            }
        }
    }
}

impl<T: fmt::Debug> Exec<T> {

    /// Return the inner Command, if any, that has been loaded into
    /// this Exec. This is used when listing commands to the user.
    pub fn into_command(self) -> Option<Command> {
        debug!("Extracting command -> {:?}", self);

        if let Self(Inner::Invocation(mutex)) = self {
            let state = mutex.into_inner().unwrap();
            if let State::Primed(command) = state {
                Some(command)
            }
            else {
                warn!("Command not primed -> {:?}", state);
                None
            }
        }
        else {
            None
        }
    }
}
