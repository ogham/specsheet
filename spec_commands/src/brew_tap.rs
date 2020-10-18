//! The `brew tap` command
//!
//! # Sample output
//!
//! The output of the command is a list of taps, with one tap per line.
//!
//! ```text
//! $ brew tap
//! dteoh/sqa
//! homebrew/cask
//! homebrew/cask-drivers
//! homebrew/core
//! railwaycat/emacsmacport
//! ```

use std::rc::Rc;

use log::*;

use spec_checks::homebrew_tap::RunBrewTap;
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **brew tap command** that runs the `brew` binary with the `tap`
/// argument.
#[derive(Debug)]
pub struct BrewTapCommand {
    exec: Option<Exec<BrewTapOutput>>,
}

impl BrewTapCommand {

    /// Creates a new command to run `brew tap`.
    pub fn create(global_options: &impl GlobalOptions) -> Self {
        let exec = global_options.command("brew-tap.output");
        Self { exec }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.exec.into_iter().flat_map(Exec::into_command)
    }
}

impl RunBrewTap for BrewTapCommand {
    fn prime(&mut self) {
        if self.exec.is_none() {
            debug!("Priming brew tap command");
            self.exec = Some(Exec::actual(brew_list_taps_cmd()));
        }
    }

    fn find_tap(&self, executor: &mut Executor, tap_name: &str) -> Result<bool, Rc<ExecError>> {
        debug!("Finding brew tap -> {:?}", tap_name);
        let output = self.exec.as_ref().unwrap().run(executor)?;
        Ok(output.find_tap(tap_name))
    }
}

fn brew_list_taps_cmd() -> Command {
    let mut cmd = Command::new("brew");
    cmd.arg("tap");
    cmd
}


/// The **brew tap output** encapsulates the output lines of an
/// invoked `BrewTapCommand`.
#[derive(Debug)]
pub struct BrewTapOutput {
    lines: Vec<Rc<str>>,
}

impl CommandOutput for BrewTapOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        exit_reason.should_be(0)?;
        Ok(Self { lines })
    }
}

impl BrewTapOutput {

    /// Searches through the output lines for a tap with the given name.
    fn find_tap(&self, tap_name: &str) -> bool {
        self.lines.iter().any(|line| **line == *tap_name)
    }
}
