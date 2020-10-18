//! The shell command
//!
//! This lets checks run arbitrary shell commands in the same way as other
//! commands.

use std::collections::BTreeMap;
use std::process::Command;
use std::rc::Rc;

use log::*;
use shell_words::quote as shellquote;

use spec_checks::{Invocation, RunShell};
use spec_exec::{Exec, Executor, RanCommand, ExecError};

use super::GlobalOptions;


/// The **shell command** uses a shell program, `sh` by default, to invoke a
/// string of shell script.
#[derive(Debug)]
pub struct ShellCommand {
    shell_binary: String,
    aliases: BTreeMap<String, String>,
    results: BTreeMap<Invocation, Exec<RanCommand>>,
}

impl ShellCommand {

    /// Creates a new command to run a shell.
    pub fn create(global_options: &impl GlobalOptions) -> Self {
        let shell_binary = match global_options.key_value("cmd.shell") {
            Some(b)  => b,
            None     => "sh".into(),
        };

        let aliases = global_options.key_prefix_values("cmd.target.");

        let results = BTreeMap::new();

        Self { shell_binary, aliases, results }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.results.into_iter().flat_map(|e| e.1.into_command())
    }
}

impl RunShell for ShellCommand {
    fn prime(&mut self, invocation: &Invocation) {
        if ! self.results.contains_key(invocation) {
            debug!("Priming shell command {:?}", invocation);

            let mut cmd = Command::new(&self.shell_binary);
            cmd.arg("-c");
            cmd.envs(&invocation.environment.0);

            let mut command = String::new();
            for (alias, path) in &self.aliases {
                command.push_str(&format!("{} () {{ {} \"$@\"; }}; ", shellquote(&alias[11..]), shellquote(path)));
            }
            if ! self.aliases.is_empty() {
                command.push_str("typeset -xf inner_function; ");
            }
            command.push_str(&invocation.shell.0);
            cmd.arg(&command);

            let exec = Exec::actual(cmd);
            self.results.insert(invocation.clone(), exec);
        }
    }

    fn run_command(&self, executor: &mut Executor, invocation: &Invocation) -> Result<Rc<RanCommand>, Rc<ExecError>> {
        debug!("Actually running command -> {:?}", invocation);

        self.results[invocation].run_raw(executor)
    }
}
