//! The `brew` command
//!
//! # Sample output
//!
//! The output of the command is a list of installed Homebrew packages, with
//! one per line.
//!
//! When running in a terminal, the command will try to format the table
//! nicely using the terminal width. Specsheet runs it without a terminal,
//! though, and it will fall back to putting them on each line.
//!
//! ```text
//! $ brew list | cat
//! ansible
//! ansible-lint
//! aom
//! atomicparsley
//! bash
//! ```

use std::rc::Rc;

use log::*;

use spec_checks::homebrew::RunBrew;
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **brew command** that runs the `brew` binary.
#[derive(Debug)]
pub struct BrewCommand {
    exec: Option<Exec<BrewOutput>>,
}

impl BrewCommand {

    /// Creates a new command to run `brew`.
    pub fn create(global_options: &impl GlobalOptions) -> Self {
        let exec = global_options.command("brew.output");
        Self { exec }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.exec.into_iter().flat_map(Exec::into_command)
    }
}

impl RunBrew for BrewCommand {
    fn prime(&mut self) {
        if self.exec.is_none() {
            debug!("Priming brew command");
            self.exec = Some(Exec::actual(brew_list_formulas_cmd()));
        }
    }

    fn find_formula(&self, executor: &mut Executor, formula_name: &str) -> Result<bool, Rc<ExecError>> {
        debug!("Finding brew formula -> {:?}", formula_name);
        let output = self.exec.as_ref().unwrap().run(executor)?;
        Ok(output.find_formula(formula_name))
    }
}

fn brew_list_formulas_cmd() -> Command {
    let mut cmd = Command::new("brew");
    cmd.env("HOMEBREW_NO_AUTO_UPDATE", "1");
    cmd.arg("list");
    cmd.arg("--formulae");
    cmd
}


/// The **brew output** encapsulates the output lines of an
/// invoked `BrewCommand`.
#[derive(Debug)]
pub struct BrewOutput {
    lines: Vec<Rc<str>>,
}

impl CommandOutput for BrewOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        exit_reason.should_be(0)?;
        Ok(Self { lines })
    }
}

impl BrewOutput {

    /// Searches through the output lines for a formula with the given name.
    fn find_formula(&self, formula_name: &str) -> bool {
        self.lines.iter().any(|line| **line == *formula_name)
    }
}
