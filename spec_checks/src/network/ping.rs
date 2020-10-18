//! The Ping check involves making an ICMP request, and seeing if we receive a
//! response.
//!
//! # Check example
//!
//! ```toml
//! [[ping]]
//! target = "192.168.0.1"
//! ```
//!
//! # Commands
//!
//! This check works by running `ping`.


use std::fmt;
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// The Ping check makes an ICMP request and awaits a response.
#[derive(PartialEq, Debug)]
pub struct PingCheck {
    target: Target,
    condition: Condition,
}

/// The network address of the machine we are pinging.
#[derive(PartialEq, Debug)]
struct Target(String);

/// Whether we expect a response.
#[derive(PartialEq, Debug)]
enum Condition {
    ReceivedResponse,
    NoResponse,
}


// ---- the check description ----

impl fmt::Display for PingCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { target, condition } = &self;

        match condition {
            Condition::ReceivedResponse => {
                write!(f, "Pinging ‘{}’ should receive a response", target.0)
            }
            Condition::NoResponse => {
                write!(f, "Pinging ‘{}’ should time out", target.0)
            }
        }
    }
}


// ---- reading from TOML ----

impl Check for PingCheck {
    const TYPE: &'static str = "ping";
}

impl PingCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["target", "state"])?;

        let target = Target::read(table)?;
        let condition = Condition::read(table)?;
        Ok(Self { target, condition })
    }
}

impl Target {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let target = table.get_or_read_error("target")?
                          .string_or_error("target")?;

        if target.is_empty() {
            return Err(ReadError::invalid("target", target.into(), "it must not be empty"));
        }

        Ok(Self(target))
    }
}

impl Condition {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let state_value = match table.get("state") {
            Some(s) => s,
            None    => return Ok(Self::ReceivedResponse),
        };

        match &state_value.string_or_error2("state", OneOf(&["responds", "no-response"]))?[..] {
            "no-response" => {
                Ok(Self::NoResponse)
            }
            "responds" => {
                Ok(Self::ReceivedResponse)
            }
            _ => {
                Err(ReadError::invalid("state", state_value.clone(), OneOf(&["responds", "no-response"])))
            }
        }
    }
}


// ---- running the check ----

/// The interface to pinging servers used by [`PingCheck`].
pub trait RunPing {

    /// Primes the command for a particular target.
    #[allow(unused)]
    fn prime(&mut self, target: &str) { }

    /// Running the command if it hasn’t been run already for this
    /// target, examine the output and return whether a response was
    /// received.
    fn is_target_up(&self, executor: &mut Executor, target: &str) -> Result<bool, Rc<ExecError>>;
}

impl<P: RunPing> RunCheck<P> for PingCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, ping: &mut P) {
        ping.prime(&self.target.0);
    }

    fn check(&self, executor: &mut Executor, ping: &P) -> Vec<CheckResult<Pass, Fail>> {
        info!("Running check");

        let package = match ping.is_target_up(executor, &self.target.0) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        match (&self.condition, package) {
            (Condition::ReceivedResponse, true) => {
                vec![ CheckResult::Passed(Pass::ReceivedResponse) ]
            }
            (Condition::ReceivedResponse, false) => {
                vec![ CheckResult::Failed(Fail::NoResponse) ]
            }
            (Condition::NoResponse, true) => {
                vec![ CheckResult::Failed(Fail::ReceivedResponse) ]
            }
            (Condition::NoResponse, false) => {
                vec![ CheckResult::Passed(Pass::NoResponse) ]
            }
        }
    }
}

/// The successful result of a Ping check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// We received a response from ping.
    ReceivedResponse,

    /// We did not receive a response.
    NoResponse,
}

/// The failure result of running a Ping check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Fail {

    /// We expected to receive a response, but we did.
    NoResponse,

    /// We expected to receive no response, but we did receive one.
    ReceivedResponse,
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReceivedResponse => {
                write!(f, "Received response")
            }
            Self::NoResponse => {
                write!(f, "No response")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoResponse => {
                write!(f, "No response")
            }
            Self::ReceivedResponse => {
                write!(f, "Received response")
            }
        }
    }
}
