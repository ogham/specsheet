//! The `brew cask` command
//!
//! # Sample output
//!
//! The output of the command is a list of installed Cask packages, with one per line.
//!
//! When running in a terminal, the command will try to format the table
//! nicely using the terminal width. Specsheet runs it without a terminal,
//! though, and it will fall back to putting them on each line.
//!
//! ```text
//! $ brew cask list | cat
//! 1password
//! aerial
//! arq
//! atom
//! audio-hijack
//! ```

use std::rc::Rc;

use log::*;

use spec_checks::homebrew_cask::RunBrewCask;
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **brew cask command** that runs the `brew` binary with the `cask`
/// argument.
#[derive(Debug)]
pub struct BrewCaskCommand {
    exec: Option<Exec<BrewCaskOutput>>,
}

impl BrewCaskCommand {

    /// Creates a new command to run `brew cask`.
    pub fn create(global_options: &impl GlobalOptions) -> Self {
        let exec = global_options.command("brew-cask.output");
        Self { exec }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.exec.into_iter().flat_map(Exec::into_command)
    }
}

impl RunBrewCask for BrewCaskCommand {
    fn prime(&mut self) {
        if self.exec.is_none() {
            debug!("Priming brew cask command");
            self.exec = Some(Exec::actual(brew_list_casks_cmd()));
        }
    }

    fn find_cask(&self, executor: &mut Executor, cask_name: &str) -> Result<bool, Rc<ExecError>> {
        debug!("Finding brew cask -> {:?}", cask_name);
        let output = self.exec.as_ref().unwrap().run(executor)?;
        Ok(output.find_cask(cask_name))
    }
}

fn brew_list_casks_cmd() -> Command {
    let mut cmd = Command::new("brew");
    cmd.arg("list").arg("--casks");
    cmd
}


/// The **brew cask output** encapsulates the output lines of an
/// invoked `BrewCaskCommand`.
#[derive(Debug)]
pub struct BrewCaskOutput {
    lines: Vec<Rc<str>>,
}

impl CommandOutput for BrewCaskOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        exit_reason.should_be(0)?;
        Ok(Self { lines })
    }
}

impl BrewCaskOutput {

    /// Searches through the lines of output for a cask with the given name.
    fn find_cask(&self, cask_name: &str) -> bool {
        self.lines.iter().any(|line| **line == *cask_name)
    }
}
