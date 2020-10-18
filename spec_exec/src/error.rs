use std::fmt;
use std::io::Error as IoError;

use crate::executor::ExitReason as ER;


/// Something that can go wrong while running a program. To help with
/// debugging, there are three different IO error variants, corresponding to
/// the three places such an error could be raised.
#[derive(Debug)]
pub enum ExecError {

    /// There was an IO error while spawning the program.
    Spawn(IoError),

    /// There was an IO error getting the program’s output.
    Stdout(IoError),

    /// There was an IO error exiting the program.
    Wait(IoError),

    /// The process didn’t exit for the reason we expected. This may mean it
    /// exited with a status other than 0, or that it was killed by a signal.
    StatusMismatch(ER),
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Spawn(ref ioe)                  => write!(f, "Spawning failed: {}", ioe),
            Self::Stdout(ref ioe)                 => write!(f, "Read failed: {}", ioe),
            Self::Wait(ref ioe)                   => write!(f, "Wait failed: {}", ioe),
            Self::StatusMismatch(ER::Status(s))   => write!(f, "Process exited with status code ‘{}’", s),
            Self::StatusMismatch(ER::Signal(s))   => write!(f, "Process was killed with signal ‘{}’", s),
            Self::StatusMismatch(ER::Unknown)     => write!(f, "Process exited for an unknown reason"),
            Self::StatusMismatch(ER::Overridden)  => unreachable!(),
        }
    }
}
