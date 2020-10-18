//! The Tap check involves running a shell command, like `cmd`, and then
//! interpreting its output as TAP.
//!
//! # Check example
//!
//! ```toml
//! [[tap]]
//! shell = './my_tests'
//! ```


use std::fmt;
use std::rc::Rc;

use log::*;
use once_cell::sync::Lazy;
use regex::Regex;

use spec_exec::Executor;

use crate::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError};

use super::{Invocation, ShellCommand, Environment, RunShell};


/// The **tap check** runs a shell command and interprets its output as TAP.
#[derive(PartialEq, Debug)]
pub struct TapCheck {
    invocation: Invocation,
}


// ---- the check description ----

impl fmt::Display for TapCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { invocation } = &self;

        write!(f, "TAP tests for command ‘{}’", invocation)
    }
}


// ---- reading from TOML ----

impl Check for TapCheck {
    const TYPE: &'static str = "tap";
}

impl TapCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["shell", "environment"])?;

        let shell = ShellCommand::read(table)?;
        let environment = Environment::read(table)?;
        let invocation = Invocation { shell, environment };
        Ok(Self { invocation })
    }
}


// ---- running the check ----

impl<S: RunShell> RunCheck<S> for TapCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, shell: &mut S) {
        shell.prime(&self.invocation);
    }

    fn check(&self, executor: &mut Executor, shell: &S) -> Vec<CheckResult<Pass, Fail>> {
        info!("Running check");

        let ran_command = match shell.run_command(executor, &self.invocation) {
            Ok(c)  => c,
            Err(e) => {
                warn!("Error running command: {}", e);
                return vec![ CheckResult::Failed(Fail::CommandFailed) ];
            }
        };

        let mut results = Vec::new();
        let mut expected_count: Option<TestNumber> = None;
        let mut test_count: TestNumber = 0;

        for line in ran_command.stdout_lines() {
            if let Some(caps) = COUNT_LINE.captures(&line) {
                if results.is_empty() {
                    expected_count = Some(caps[2].parse().unwrap());
                }
                else {
                    panic!("The count line came late! What gives?");
                }
            }
            else if let Some(caps) = RESULT_LINE.captures(&line) {
                test_count += 1;

                let number = caps[2].parse().unwrap();
                let description = caps.get(3).map(|e| String::from(e.as_str()));

                if caps.get(1).is_some() {
                    results.push(CheckResult::Failed(Fail::TestFailed(number, description)));
                }
                else {
                    results.push(CheckResult::Passed(Pass::TestPassed(number, description)));
                }
            }
            else {
                results.push(CheckResult::Failed(Fail::UnparseableLine(line)));
            }
        }

        if let Some(expected) = expected_count {
            if test_count == expected {
                results.push(CheckResult::Passed(Pass::CorrectNumber(expected)));
            }
            else {
                results.push(CheckResult::Failed(Fail::IncorrectNumber { expected, got: test_count }));
            }
        }

        results
    }
}

/// Regular expression for the count line of a TAP file.
static COUNT_LINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r##"(?x) ^
        (\d+) \.\. (\d+)
    $ "##).unwrap()
});

/// Regular expression for the count line of a TAP file.
static RESULT_LINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r##"(?x) ^
        (?: (not) \s+)?
        ok \s+
        (\d+)
        (?:
          \s*
          -
          \s*
          (.+)
        )?
    $ "##).unwrap()
});


/// A TAP test’s number.
pub type TestNumber = u32;

/// The successful result of a TAP check.
#[derive(PartialEq, Debug)]
pub enum Pass {

    /// A TAP test passed, with its number and description string.
    TestPassed(TestNumber, Option<String>),

    /// The correct number of tests were run.
    CorrectNumber(TestNumber),
}

/// The failure result of running a TAP check.
#[derive(PartialEq, Debug)]
pub enum Fail {

    /// The command failed to be run, or exited with a non-zero status
    /// code. In this case, we don’t check the output for TAP at all.
    CommandFailed,

    /// A TAP test failed, with its number and description string.
    TestFailed(TestNumber, Option<String>),

    /// The incorrect number of tests were run at the end.
    IncorrectNumber { expected: TestNumber, got: TestNumber },

    /// One of the output lines didn’t make any gosh darn sense.
    UnparseableLine(Rc<str>),
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TestPassed(num, None) => {
                write!(f, "TAP test #{} passed", num)
            }
            Self::TestPassed(num, Some(desc)) => {
                write!(f, "TAP test #{} passed ({})", num, desc)
            }
            Self::CorrectNumber(expected) => {
                write!(f, "Correct number ({}) of tests run", expected)
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandFailed => {
                write!(f, "The command failed to be run")
            }
            Self::TestFailed(num, None) => {
                write!(f, "TAP test #{} failed", num)
            }
            Self::TestFailed(num, Some(desc)) => {
                write!(f, "TAP test #{} failed ({})", num, desc)
            }
            Self::IncorrectNumber { expected, got } => {
                write!(f, "Incorrect number of tests run (expected {}, got {})", expected, got)
            }
            Self::UnparseableLine(line) => {
                write!(f, "Unparseable TAP line {:?}", line)
            }
        }
    }
}
