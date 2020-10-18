//! The `gem` command
//!
//! # Sample output
//!
//! The output of the command is a list of installed gems and their versions.
//!
//! ```text
//! $ gem list
//!
//! *** LOCAL GEMS ***
//!
//! activesupport (5.2.3)
//! addressable (2.6.0)
//! ansi (1.5.0)
//! ast (2.4.0)
//! aws-eventstream (1.0.3)
//! ```

use std::rc::Rc;

use log::*;

use spec_checks::gem::RunGem;
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **gem command** that runs the `gem` binary.
#[derive(Debug)]
pub struct GemCommand {
    exec: Option<Exec<GemListOutput>>,
}

impl GemCommand {

    /// Creates a new command to run `gem`.
    pub fn create(global_options: &impl GlobalOptions) -> Self {
        let exec = global_options.command("gem.output");
        Self { exec }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.exec.into_iter().flat_map(Exec::into_command)
    }
}

impl RunGem for GemCommand {
    fn prime(&mut self) {
        if self.exec.is_none() {
            debug!("Priming gem command");
            self.exec = Some(Exec::actual(gem_list_cmd()));
        }
    }

    fn find_gem(&self, executor: &mut Executor, gem_name: &str) -> Result<bool, Rc<ExecError>> {
        debug!("Finding gem -> {:?}", gem_name);
        let output = self.exec.as_ref().unwrap().run(executor)?;
        Ok(output.find_gem(gem_name))
    }
}

fn gem_list_cmd() -> Command {
    let mut cmd = Command::new("gem");
    cmd.arg("list");
    cmd
}


/// The **gem output** encapsulates the output lines of an
/// invoked `GemCommand`.
#[derive(Debug)]
pub struct GemListOutput {
    lines: Vec<Rc<str>>,
}

impl CommandOutput for GemListOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        exit_reason.should_be(0)?;
        Ok(Self { lines })
    }
}

impl GemListOutput {

    /// Searches through the output lines for a gem with the given name.
    fn find_gem(&self, gem_name: &str) -> bool {
        self.lines.iter().any(|line| line.starts_with(gem_name))
    }
}
