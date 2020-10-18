use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use log::*;
use regex::Regex;


/// The **side process** gets run in the background as the checks are run. It
/// contains a string of shell that gets executed.
#[derive(PartialEq, Debug)]
pub struct SideProcess {
    pub shell: String,
    pub wait: StartupWait,
    pub signal: KillSignal,
}

/// What we should do to wait for the external process to start up.
#[derive(PartialEq, Debug, Clone)]
pub enum StartupWait {

    /// Start running checks immediately after the process starts.
    Immediate,

    /// Wait the given amount of time before starting to run checks.
    Delay(Duration),

    /// Continuously attempt to connect to the given TCP network port, and
    /// start running checks after a successful connection.
    Port(u16),

    /// Continuously check for the existence of a given file, and start
    /// running checks once it appears.
    File(PathBuf),

    /// Examine the output of the process, and start running checks once it
    /// outputs a line that matches the given regex.
    OutputLine(String),
}

/// What signal we should send to the external process to get it to stop.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum KillSignal {

    /// Send SIGINT, which sends an interrupt.
    Int,

    /// Send SIGTERM, which lets it exit gracefully.
    Term,

    /// Send SIGKILL, which kills it outright.
    Kill,
}


impl Default for StartupWait {
    fn default() -> Self {
        Self::Immediate
    }
}

impl Default for KillSignal {
    fn default() -> Self {
        // TODO: Some sort of ability to send TERM, then wait 10 seconds, then
        // only send KILL if they are still around
        Self::Kill
    }
}


impl StartupWait {

    /// Do the actual waiting.
    fn wait(&self) {
        match self {
            Self::Immediate => {
                info!("Running immediately");
            }
            Self::Delay(duration) => {
                info!("Delay -> {:?}", duration);
                thread::sleep(*duration);
            }
            Self::Port(port) => {
                use std::net::{SocketAddr, TcpStream, Ipv4Addr};

                info!("Waiting for port -> {:?}", port);
                let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), *port);

                loop {
                    thread::sleep(Duration::from_millis(100));
                    match TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
                        Ok(stream) => {
                            debug!("Received response -> {:?}", stream.peer_addr());
                            break;
                        }
                        Err(e) => {
                            debug!("Connection failed -> {:?}", e);
                            continue;
                        }
                    }
                }
            }
            Self::File(path) => {
                info!("Waiting for file -> {:?}", path);

                loop {
                    thread::sleep(Duration::from_millis(100));
                    if path.exists() {
                        debug!("File exists");
                        break;
                    }
                    else {
                        debug!("File does not exist yet");
                        continue;
                    }
                }
            }
            Self::OutputLine(regex) => {
                let _ = Regex::new(regex);
                unimplemented!("Just not done yet")
            }
        }
    }
}


impl SideProcess {

    /// Execute the process and return its handle.
    pub fn start(&self) -> u32 {
        use std::io::{BufRead, BufReader};

        debug!("Spawning side process -> {:?}", self.shell);

        let (tx, rx) = mpsc::channel();

        let builder = thread::Builder::new().name("side process thread".into());
        let shell = self.shell.clone();
        let wait = self.wait.clone();
        builder.spawn(move || {
            let cmd = Command::new("bash")
                .arg("-c")
                .arg(&shell)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to execute child");


            wait.wait();
            tx.send(cmd.id()).expect("Sending tx");

            let reader = BufReader::new(cmd.stdout.expect("Process stdout"));
            for line in reader.lines() {
                let line = line.expect("Line IO error");
                debug!("Child line -> {:?}", line);
            }
        }).expect("spawn");

        rx.recv().expect("Receiving rx")
    }

    /// Given a handle that was started earlier, kill it.
    pub fn stop(&self, child_pid: u32) -> io::Result<()> {
        debug!("Stopping side process with ID -> {}", child_pid);

        // This needs unsafe because it’s a libc function. Killing processes
        // does exist in std, but only for SIGKILL.
        let ret_val = unsafe {
            libc::kill(child_pid as i32, self.signal.number())
        };

        // According to the man page, `kill` returns 0 on success.
        if ret_val == 0 {
            debug!("Process dead.");
            Ok(())
        }
        else {
            warn!("Stopping side process failed");
            Err(io::Error::last_os_error())
        }
    }
}

impl KillSignal {
    fn number(self) -> libc::c_int {
        match self {
            Self::Int  => libc::SIGINT,
            Self::Kill => libc::SIGKILL,
            Self::Term => libc::SIGTERM,
        }
    }
}
