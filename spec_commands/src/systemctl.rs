//! The `systemctl` command.
//!
//! # Sample output
//!
//! The output contains the service name, a list of properties, and some log
//! entries.
//!
//! ```text
//! ● postgresql.service - PostgreSQL RDBMS
//!    Loaded: loaded (/lib/systemd/system/postgresql.service; enabled; vendor preset: enabled)
//!    Active: active (exited) since Tue 2018-02-20 14:41:50 GMT; 1 day 23h ago
//!  Main PID: 2497 (code=exited, status=0/SUCCESS)
//!     Tasks: 0
//!    Memory: 0B
//!       CPU: 0
//!    CGroup: /system.slice/postgresql.service
//!
//! Feb 20 14:41:50 school systemd[1]: Starting PostgreSQL RDBMS...
//! Feb 20 14:41:50 school systemd[1]: Started PostgreSQL RDBMS.
//! Feb 20 14:41:55 school systemd[1]: Started PostgreSQL RDBMS.
//! ```
//!
//! ```text
//! ● seashell.service
//!    Loaded: not-found (Reason: No such file or directory)
//!    Active: inactive (dead)
//! ```
//!
//! The program will return 4 in the case when the service being asked for
//! doesn’t actually exist.


use std::collections::BTreeMap;
use std::rc::Rc;

use log::*;

use spec_checks::systemd::{RunSystemctl, ServiceState};
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **systemctl command** that runs the `systemctl` binary.
#[derive(Debug, Default)]
pub struct SystemctlCommand {
    results: BTreeMap<String, Exec<SystemctlOutput>>,
}

impl SystemctlCommand {

    /// Creates a new command to run `systemctl`.
    pub fn create(_global_options: &impl GlobalOptions) -> Self {
        Self::default()
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.results.into_iter().flat_map(|e| e.1.into_command())
    }
}

impl RunSystemctl for SystemctlCommand {
    fn prime(&mut self, service_name: &str) {
        if ! self.results.contains_key(service_name) {
            debug!("Priming systemctl command with {:?}", service_name);
            let exec = Exec::actual(systemctl_status_cmd(service_name));
            self.results.insert(service_name.to_owned(), exec);
        }
    }

    fn service_state(&self, executor: &mut Executor, service_name: &str) -> Result<ServiceState, Rc<ExecError>> {
        debug!("Looking up service state -> {:?}", service_name);
        let output = self.results[service_name].run(executor)?;

        if output.missing {
            Ok(ServiceState::Missing)
        }
        else {
            Ok(output.service_state())
        }
    }
}

fn systemctl_status_cmd(service_name: &str) -> Command {
    let mut cmd = Command::new("systemctl");
    cmd.arg("status").arg(service_name).arg("--no-pager");
    cmd
}


/// The **systemctl output** encapsulates the output lines of an
/// invoked `SystemctlCommand`.
#[derive(Debug)]
pub struct SystemctlOutput {
    lines: Vec<Rc<str>>,
    missing: bool,
}

impl CommandOutput for SystemctlOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        if exit_reason.is(0) {
            let missing = false;
            Ok(Self { lines, missing })
        }
        else if exit_reason.is(4) {
            let missing = true;
            Ok(Self { lines, missing })
        }
        else {
            Err(ExecError::StatusMismatch(exit_reason))
        }
    }
}

impl SystemctlOutput {

    /// Examines the output lines to determine the service state.
    fn service_state(&self) -> ServiceState {
        if self.lines.is_empty() || self.lines.iter().any(|e| e.contains("Loaded: not-found")) {
            ServiceState::Missing
        }
        else if self.lines.iter().any(|e| e.contains("Active: active")) {
            ServiceState::Running
        }
        else {
            ServiceState::Stopped
        }
    }
}
