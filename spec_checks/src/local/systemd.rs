//! The Systemd check involves searching through the list of systemd units.
//!
//! # Check example
//!
//! ```toml
//! [[systemd]]
//! service = 'consul'
//! ```
//!
//! # Commands
//!
//! This check works by running the `systemctl` command.


use std::fmt;
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// A check against the list of running systemd services.
#[derive(PartialEq, Debug)]
pub struct SystemdCheck {

    /// The name of the service being checked.
    service_name: ServiceName,

    /// The condition to test it with.
    condition: Condition,
}

#[derive(PartialEq, Debug)]
struct ServiceName(String);

#[derive(PartialEq, Debug)]
enum Condition {

    /// Check that a service exists and is running.
    Running,

    /// Check that a service exists and is _not_ running.
    Stopped,

    /// Check that a service does not exist.
    Missing,
}


// ---- the check description ----

impl fmt::Display for SystemdCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { service_name, condition } = &self;

        match condition {
            Condition::Running => {
                write!(f, "Service ‘{}’ is running", service_name.0)
            }
            Condition::Stopped => {
                write!(f, "Service ‘{}’ is stopped", service_name.0)
            }
            Condition::Missing => {
                write!(f, "Service ‘{}’ is missing", service_name.0)
            }
        }
    }
}


// ---- reading from TOML ----

impl Check for SystemdCheck {
    const TYPE: &'static str = "systemd";
}

impl SystemdCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["service", "state"])?;

        let service_name = ServiceName::read(table)?;
        let condition = Condition::read(table)?;
        Ok(Self { service_name, condition })
    }
}

impl ServiceName {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let name_value = table.get_or_read_error("service")?;
        let service_name = name_value.string_or_error("service")?;

        if service_name.is_empty() {
            Err(ReadError::invalid("service", service_name.into(), "it must not be empty"))
        }
        else if service_name.contains('/') {
            Err(ReadError::invalid("service", service_name.into(), "it must not contain a ‘/’ character"))
        }
        else {
            Ok(Self(service_name))
        }
    }
}

impl Condition {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let state_value = match table.get("state") {
            Some(s) => s,
            None    => return Ok(Self::Running),
        };

        match &state_value.string_or_error2("state", OneOf(&["running", "stopped", "missing"]))?[..] {
            "running" => {
                Ok(Self::Running)
            }
            "stopped" => {
                Ok(Self::Stopped)
            }
            "missing" => {
                Ok(Self::Missing)
            }
            _ => {
                Err(ReadError::invalid("state", state_value.clone(), OneOf(&["running", "stopped", "missing"])))
            }
        }
    }
}


// ---- running the check ----

/// The interface to the local systemd state used by [`SystemdCheck`].
pub trait RunSystemctl {

    /// Prime the command for running, to get the state of the service with the given name.
    #[allow(unused)]
    fn prime(&mut self, service_name: &str) { }

    /// Running the command if it hasn’t been run already for the given
    /// service, examine the output to return the service’s state.
    fn service_state(&self, executor: &mut Executor, service_name: &str) -> Result<ServiceState, Rc<ExecError>>;
}

/// One of the states a service could be in, according to systemd.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ServiceState {

    /// The service exists and is running.
    Running,

    /// The service exists but is not running.
    Stopped,

    /// No service with the given name is present in the current systemd
    /// state.
    Missing,
}

impl<S: RunSystemctl> RunCheck<S> for SystemdCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, systemctl: &mut S) {
        systemctl.prime(&self.service_name.0);
    }

    fn check(&self, executor: &mut Executor, systemctl: &S) -> Vec<CheckResult<Pass, Fail>> {
        info!("Running check");

        let service_state = match systemctl.service_state(executor, &self.service_name.0) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        match (&self.condition, service_state) {
            // Successes
            (Condition::Running, ServiceState::Running) => {
                vec![ CheckResult::Passed(Pass::IsRunning) ]
            }
            (Condition::Stopped, ServiceState::Stopped) => {
                vec![ CheckResult::Passed(Pass::IsStopped) ]
            }
            (Condition::Missing, ServiceState::Missing) => {
                vec![ CheckResult::Passed(Pass::IsMissing) ]
            }

            // Fails
            (_, ServiceState::Running) => {
                vec![ CheckResult::Failed(Fail::IsRunning) ]
            }
            (_, ServiceState::Stopped) => {
                vec![ CheckResult::Failed(Fail::IsStopped) ]
            }
            (_, ServiceState::Missing) => {
                vec![ CheckResult::Failed(Fail::IsMissing) ]
            }
        }
    }
}

/// The successful result of a systemd check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The service is running.
    IsRunning,

    /// The service is not running.
    IsStopped,

    /// The service could not be found.
    IsMissing,
}

/// The failure result of running a systemd check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Fail {

    /// The service was meant to be stopped or missing, but it's running.
    IsRunning,

    /// The service was meant to be running or missing, but it's stopped.
    IsStopped,

    /// The service was meant to exist, but it doesn't.
    IsMissing,
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IsRunning => {
                write!(f, "it is running")
            }
            Self::IsStopped => {
                write!(f, "it is stopped")
            }
            Self::IsMissing => {
                write!(f, "it is missing")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IsRunning => {
                write!(f, "it is running")
            }
            Self::IsStopped => {
                write!(f, "it is stopped")
            }
            Self::IsMissing => {
                write!(f, "it is missing")
            }
        }
    }
}
