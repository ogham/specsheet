pub mod cmd;
pub mod tap;

use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

use spec_exec::{Executor, RanCommand, ExecError};


#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct Invocation {
    pub shell: ShellCommand,
    pub environment: Environment,
}

impl fmt::Display for Invocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (env_var, value) in &self.environment.0 {
            write!(f, "{}={} ", env_var, value)?;
        }

        write!(f, "{}", self.shell.0)
    }
}

/// The shell command that gets executed.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct ShellCommand(pub String);

/// Any environment variables to set when running the process.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Default, Clone)]
pub struct Environment(pub BTreeMap<String, String>);

/// The interface to running shell commands.
pub trait RunShell {

    #[allow(unused)]
    fn prime(&mut self, invocation: &Invocation) { }

    /// Runs a short shell command with the given environment variables,
    /// and returns its output.
    fn run_command(&self, executor: &mut Executor, invocation: &Invocation) -> Result<Rc<RanCommand>, Rc<ExecError>>;
}
