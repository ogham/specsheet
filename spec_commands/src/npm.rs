//! The `npm` command.
//!
//! # Sample output
//!
//! ```text
//! $ npm list -g --depth=0
//! /usr/lib
//! ├── babel-cli@6.26.0
//! ├── babel-preset-env@1.7.0
//! ├── npm@6.7.0
//! ├── sass-lint@1.12.1
//! └── typescript@3.4.3
//! ```

use std::rc::Rc;

use log::*;

use spec_checks::npm::RunNpm;
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **npm command** that runs the `npm` binary.
#[derive(Debug)]
pub struct NpmCommand {
    exec: Option<Exec<NpmListOutput>>,
}

impl NpmCommand {

    /// Creates a new command to run `npm`.
    pub fn create(global_options: &impl GlobalOptions) -> Self {
        let exec = global_options.command("npm.output");
        Self { exec }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.exec.into_iter().flat_map(Exec::into_command)
    }
}

impl RunNpm for NpmCommand {
    fn prime(&mut self) {
        if self.exec.is_none() {
            debug!("Priming npm command");
            self.exec = Some(Exec::actual(npm_list_cmd()));
        }
    }

    fn find_package(&self, executor: &mut Executor, package_name: &str) -> Result<bool, Rc<ExecError>> {
        debug!("Finding npm package -> {:?}", package_name);
        let output = self.exec.as_ref().unwrap().run(executor)?;
        Ok(output.find_package(package_name))
    }
}

fn npm_list_cmd() -> Command {
    let mut cmd = Command::new("npm");
    cmd.arg("list").arg("-g").arg("--depth=0");
    cmd
}


/// The **npm output** encapsulates the output lines of an
/// invoked `NpmCommand`.
#[derive(Debug)]
pub struct NpmListOutput {
    lines: Vec<Rc<str>>,
}

impl CommandOutput for NpmListOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        exit_reason.should_be(0)?;
        Ok(Self { lines })
    }
}

impl NpmListOutput {

    /// Searches the output lines for a package with the given name.
    fn find_package(&self, package_name: &str) -> bool {
        self.lines.iter().any(|line| line.contains(package_name))
    }
}
