//! The `defaults` command
//!
//! # Sample output
//!
//! The output is either just the value, or an error message saying the value
//! does not exist.
//!
//! ```text
//! $ defaults read com.apple.dock mru-spaces
//! 0
//! ```
//!
//! ```text
//! $ defaults read ~/Library/Containers/com.apple.Safari/Data/Library/Preferences/com.apple.Safari ShowIconsInTabs
//! 1
//! ```
//!
//! ```text
//! $ defaults read com.apple.dock wibble
//! 2019-10-05 08:03:56.575 defaults[6872:1732538]
//! The domain/default pair of (com.apple.dock, wibble) does not exist
//! ```
//!
//! The process returns 1 if the value is not present in the database.

use std::collections::BTreeMap;
use std::rc::Rc;

use log::*;

use spec_checks::defaults::{RunDefaults, DefaultsLocation};
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **defaults command** that runs the `defaults` binary.
#[derive(Debug, Default)]
pub struct DefaultsCommand {
    results: BTreeMap<DefaultsLocation, Exec<DefaultsOutput>>,
}

impl DefaultsCommand {

    /// Creates a new command to run `defaults`.
    pub fn create(_global_options: &impl GlobalOptions) -> Self {
        Self::default()
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.results.into_iter().map(|(_, e)| e.into_command().unwrap())
    }
}

impl RunDefaults for DefaultsCommand {
    fn prime(&mut self, location: &DefaultsLocation) {
        if ! self.results.contains_key(location) {
            debug!("Priming defaults command with {:?}", location);
            let exec = Exec::actual(defaults_lookup_cmd(location));
            self.results.insert(location.clone(), exec);
        }
    }

    fn get_value(&self, executor: &mut Executor, location: &DefaultsLocation) -> Result<Option<Rc<str>>, Rc<ExecError>> {
        debug!("Finding defaults value -> {:?}", location);
        let output = self.results[location].run(executor)?;

        if output.missing {
            Ok(None)
        }
        else {
            Ok(Some(output.get_value()))
        }
    }
}

fn defaults_lookup_cmd(location: &DefaultsLocation) -> Command {
    let mut cmd = Command::new("defaults");
    cmd.arg("read").arg(&location.place.to_string()).arg(&location.key);
    cmd
}


/// The **defaults output** encapsulates the output lines of an
/// invoked `DefaultsCommand`.
#[derive(Debug)]
pub struct DefaultsOutput {
    lines: Vec<Rc<str>>,
    missing: bool,
}

impl CommandOutput for DefaultsOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        if exit_reason.is(0) {
            let missing = false;
            Ok(Self { lines, missing })
        }
        else if exit_reason.is(1) {
            let missing = true;
            Ok(Self { lines, missing })
        }
        else {
            Err(ExecError::StatusMismatch(exit_reason))
        }
    }
}

impl DefaultsOutput {

    /// Returns a clone of the value read from the defaults, which should be
    /// on the first and only line.
    fn get_value(&self) -> Rc<str> {
        Rc::clone(&self.lines.first().unwrap())
    }
}
