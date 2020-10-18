//! Command checks
//!
//! # Check example
//!
//! ```text
//! [[cmd]]
//! shell = "consul version"
//! status = 0
//! stdout = { string = "Consul v1.5" }
//! ```
//!
//! # Commands
//!
//! These checks only run the commands that they are given.


use std::fmt;

use log::*;

use spec_exec::{Executor, ExitReason};

use crate::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::contents::{self, ContentsMatcher};
use crate::read::{TomlValue, ValueExtras, ReadError};

use super::{Invocation, ShellCommand, Environment, RunShell};


/// A check that involves running a command on the local machine and
/// testing its exit status and output streams.
#[derive(PartialEq, Debug)]
pub struct CommandCheck {
    invocation: Invocation,
    status: ExpectedStatus,
    stdout: Option<ContentsMatcher>,
    stderr: Option<ContentsMatcher>,
}

/// The return code we expect from the process.
#[derive(PartialEq, Debug)]
enum ExpectedStatus {

    /// The process can exit with any code, including signals and
    /// unknown exit reasons.
    Any,

    /// The process must exit with the given code.
    Specific(u8),
}


// ---- the check description ----

impl Check for CommandCheck {
    const TYPE: &'static str = "cmd";
}

impl fmt::Display for CommandCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { invocation, status, stdout, stderr } = &self;

        write!(f, "Command ‘{}’ ", invocation)?;

        match (stdout, stderr, status) {
            (None, None, ExpectedStatus::Any) => {
                write!(f, "executes")
            }
            (None, None, ExpectedStatus::Specific(ec)) => {
                write!(f, "returns ‘{}’", ec)
            }
            (Some(ContentsMatcher::ShouldBeEmpty), Some(ContentsMatcher::ShouldBeEmpty), _) => {
                if let ExpectedStatus::Specific(ec) = status {
                    write!(f, "returns ‘{}’ with", ec)?;
                }
                else {
                    write!(f, "executes with")?;
                }

                write!(f, " empty stdout and stderr")?;
                Ok(())
            }
            (Some(ContentsMatcher::ShouldBeNonEmpty), Some(ContentsMatcher::ShouldBeNonEmpty), _) => {
                if let ExpectedStatus::Specific(ec) = status {
                    write!(f, "returns ‘{}’ with", ec)?;
                }
                else {
                    write!(f, "executes with")?;
                }

                write!(f, " non-empty stdout and stderr")?;
                Ok(())
            }
            _ => {
                if let ExpectedStatus::Specific(ec) = status {
                    write!(f, "returns ‘{}’ with", ec)?;
                }
                else {
                    write!(f, "executes with")?;
                }

                if let Some(contents_matcher) = stdout {
                    contents_matcher.describe(f, "stdout")?;
                }

                if stdout.is_some() && stderr.is_some() {
                    write!(f, " and")?;
                }

                if let Some(contents_matcher) = stderr {
                    contents_matcher.describe(f, "stderr")?;
                }

                Ok(())
            }
        }
    }
}


// ---- reading from TOML ----

impl CommandCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["shell", "environment", "status", "stdout", "stderr"])?;

        let shell = ShellCommand::read(table)?;
        let environment = Environment::read(table)?;
        let invocation = Invocation { shell, environment };

        let status = ExpectedStatus::read(table)?;
        let stdout = table.get("stdout").map(|e| ContentsMatcher::read("stdout", e)).transpose()?;
        let stderr = table.get("stderr").map(|e| ContentsMatcher::read("stderr", e)).transpose()?;
        Ok(Self { invocation, status, stdout, stderr })
    }
}

impl ShellCommand {
    pub(crate) fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let shell_value = table.get_or_read_error("shell")?;
        let shell_str = shell_value.string_or_error("shell")?;

        if shell_str.is_empty() {
            Err(ReadError::invalid("shell", shell_value.clone(), "it must not be empty"))
        }
        else {
            Ok(Self(shell_str))
        }
    }
}

impl Environment {
    pub(crate) fn read(table: &TomlValue) -> Result<Self, ReadError> {
        if let Some(env_table) = table.get("environment") {
            let map = env_table.string_map_or_read_error("environment")?;
            Ok(Self(map))
        }
        else {
            Ok(Self::default())
        }
    }
}

impl ExpectedStatus {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        use std::convert::TryFrom;

        if let Some(status_value) = table.get("status") {
            let number = status_value.number_or_error("status")?;
            match u8::try_from(number) {
                Ok(status) => {
                    Ok(Self::Specific(status))
                }
                Err(e) => {
                    warn!("Number out of range: {}", e);
                    Err(ReadError::invalid("status", status_value.clone(), "it must be between 0 and 255"))
                }
            }
        }
        else {
            Ok(Self::Any)
        }
    }
}


// ---- running the check ----

impl<S: RunShell> RunCheck<S> for CommandCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, shell: &mut S) {
        shell.prime(&self.invocation);
    }

    fn check(&self, executor: &mut Executor, shell: &S) -> Vec<CheckResult<Pass, Fail>> {
        let ran_command = match shell.run_command(executor, &self.invocation) {
            Ok(c)  => c,
            Err(e) => {
                warn!("Error running command: {}", e);
                return vec![ CheckResult::Failed(Fail::No) ];
            }
        };

        let mut results = vec![ CheckResult::Passed(Pass::CommandWasExecuted) ];

        // Status check
        if let ExpectedStatus::Specific(num) = self.status {
            if ran_command.exit_reason.is(num) {
                results.push(CheckResult::Passed(Pass::StatusCodeMatches));
            }
            else {
                results.push(CheckResult::Failed(Fail::ExitReasonMismatch(ran_command.exit_reason)));
            }
        }

        if let Some(stdout_matcher) = &self.stdout {
            match stdout_matcher.check(&ran_command.stdout_bytes()) {
                CheckResult::Passed(pass) => {
                    results.push(CheckResult::Passed(Pass::ContentsPass("stdout", pass)));
                }
                CheckResult::Failed(fail) => {
                    results.push(CheckResult::Failed(Fail::ContentsFail("stdout", fail)));
                }
                CheckResult::CommandError(_) => {
                    unreachable!();
                }
            }
        }

        if let Some(stderr_matcher) = &self.stderr {
            match stderr_matcher.check(&ran_command.stderr_bytes()) {
                CheckResult::Passed(pass) => {
                     results.push(CheckResult::Passed(Pass::ContentsPass("stderr", pass)));
                }
                CheckResult::Failed(fail) => {
                    results.push(CheckResult::Failed(Fail::ContentsFail("stderr", fail)));
                }
                CheckResult::CommandError(_) => {
                    unreachable!();
                }
            }
        }

        results
    }
}

/// The successful result of a command check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The command was able to be executed without incident.
    CommandWasExecuted,

    /// The process’s exit status was the one we expected.
    StatusCodeMatches,

    ContentsPass(&'static str, contents::Pass),
}

/// The failure result of running a command check.
#[derive(Debug)]
pub enum Fail {

    No,

    /// The process’s exit reason was different from the one we expected.
    ExitReasonMismatch(ExitReason),

    /// One of the two contents matchers did not match.
    ContentsFail(&'static str, contents::Fail),
}

impl PassResult for Pass {
}

impl FailResult for Fail {
    fn command_output(&self) -> Option<(String, &String)> {
        match self {
            Self::ContentsFail(_, fail)  => fail.command_output("Command output:"),
            _                            => None,
        }
    }

    fn diff_output(&self) -> Option<(String, &String, &String)> {
        match self {
            Self::ContentsFail(_, fail)  => fail.diff_output(),
            _                            => None,
        }
    }
}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandWasExecuted => {
                write!(f, "command was executed")
            }
            Self::StatusCodeMatches => {
                write!(f, "status code matches")
            }
            Self::ContentsPass(stream, contents_pass) => {
                write!(f, "{} {}", stream, contents_pass)
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::No => {
                write!(f, "No")
            }
            Self::ExitReasonMismatch(ExitReason::Status(num)) => {
                write!(f, "command exited with status code ‘{}’", num)
            }
            Self::ExitReasonMismatch(e) => {
                write!(f, "command exited with reason ‘{:?}’", e)  // todo: englishify these variants
            }
            Self::ContentsFail(stream, contents_fail) => {
                write!(f, "{} {}", stream, contents_fail)
            }
        }
    }
}
