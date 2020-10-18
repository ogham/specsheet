//! The `ping` command
//!
//! # Sample output
//!
//! The output includes the ping result and a summary. Specsheet ignores the
//! summary and concentrates on the ping result.
//!
//! ```text
//! $ ping 1.1.1.1 -c 1
//! PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.
//! 64 bytes from 1.1.1.1: icmp_seq=1 ttl=61 time=1.48 ms
//!
//! --- 1.1.1.1 ping statistics ---
//! 1 packets transmitted, 1 received, 0% packet loss, time 0ms
//! rtt min/avg/max/mdev = 1.478/1.478/1.478/0.000 ms
//! ```

use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::Duration;

use log::*;

use spec_checks::ping::RunPing;
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **ping command** that runs the `ping` binary.
#[derive(Debug)]
pub struct PingCommand {
    results: BTreeMap<String, Exec<PingOutput>>,
    timeout: Option<Duration>,
}

impl PingCommand {

    /// Creates a new command to run `ping`.
    pub fn create(global_options: &impl GlobalOptions) -> Self {
        let results = BTreeMap::new();
        let timeout = global_options.duration("ping.timeout");
        PingCommand { results, timeout }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.results.into_iter().flat_map(|e| e.1.into_command())
    }
}

impl RunPing for PingCommand {
    fn prime(&mut self, target: &str) {
        if ! self.results.contains_key(target) {
            debug!("Priming ping command with {:?}", target);
            let exec = Exec::actual(ping_target_cmd(target));
            self.results.insert(target.to_owned(), exec);
        }
    }

    fn is_target_up(&self, executor: &mut Executor, target: &str) -> Result<bool, Rc<ExecError>> {
        debug!("Pinging target -> {:?}", target);
        let output = self.results[target].run(executor)?;
        Ok(output.received_response())
    }
}

fn ping_target_cmd(target: &str) -> Command {
    let mut cmd = Command::new("ping");
    cmd.arg(target).arg("-c").arg("1");
    cmd
}


/// The **ping output** encapsulates the output lines of an
/// invoked `PingCommand`.
#[derive(Debug)]
pub struct PingOutput {
    lines: Vec<Rc<str>>,
}

impl CommandOutput for PingOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        // Ping returns 0 on success, but returns 1 on Linux and 2 on macOS
        // and BSD upon failure
        if exit_reason.is(0) || exit_reason.is(1) || exit_reason.is(2) {
            Ok(Self { lines })
        }
        else {
            Err(ExecError::StatusMismatch(exit_reason))
        }
    }
}

impl PingOutput {

    /// Checks the output lines for whether we received a ping response.
    fn received_response(&self) -> bool {
        self.lines.iter().any(|e| e.contains("1 packets transmitted, 1 received, 0% packet loss"))
    }
}
