//! The ufw check involves searching through the list of ufw firewall rules.
//!
//! # Check example
//!
//! ```toml
//! [[ufw]]
//! port = 443
//! protocol = 'tcp'
//! allow = 'Anywhere'
//! ipv6 = true
//! ```
//!
//! # Commands
//!
//! This check works by running the `ufw` command.


use std::fmt;
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::common::PortNumber;
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// A check against the list of ufw firewall rules.
#[derive(PartialEq, Debug)]
pub struct UfwCheck {
    portspec: Portspec,
    protocol: Protocol,
    ipv6: bool,
    condition: Condition,
}

/// Which ports are being checked.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Portspec {
    One(u16),
    Range(u16, u16),
}

/// The network protocol of a rule being checked.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Protocol {
    TCP,
    UDP,
}

/// Whether we expect the rule to exist or not.
#[derive(PartialEq, Debug)]
enum Condition {

    /// We expect a rule with the ports and protocol to exist with the given
    /// `Allow` field.
    Exists {
        allow: String,
    },

    /// We expect a rule with the ports and protocol to _not_ exist.
    Missing,
}


// ---- the check description ----

impl fmt::Display for UfwCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { portspec, protocol, ipv6, condition } = &self;

        write!(f, "Rule for {:?}", protocol)?;

        match portspec {
            Portspec::One(one)         => write!(f, " port ‘{}’", one)?,
            Portspec::Range(from, to)  => write!(f, " ports ‘{}–{}’", from, to)?,
        }

        if *ipv6 {
            write!(f, " (IPv6)")?;
        }

        match condition {
            Condition::Exists { allow } => {
                write!(f, " exists with allow ‘{}’", allow)?;
            }
            Condition::Missing => {
                write!(f, " does not exist")?;
            }
        }

        Ok(())
    }
}


// ---- reading from TOML ----

impl Check for UfwCheck {
    const TYPE: &'static str = "ufw";
}

impl UfwCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["port", "protocol", "ipv6", "state", "allow"])?;

        let portspec = Portspec::read(table)?;
        let protocol = Protocol::read(table)?;
        let ipv6 = table.get("ipv6").map(|e| e.boolean_or_error("ipv6")).transpose()?.unwrap_or_default();
        let condition = Condition::read(table)?;
        Ok(Self { portspec, protocol, ipv6, condition })
    }
}

impl Portspec {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let inner = PortNumber::read(table)?;
        Ok(Self::One(inner.0))
    }
}

impl Condition {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let state_value = table.get("state");
        if state_value.is_none() {
            let allow_value = table.get_or_read_error("allow")?;
            let allow = allow_value.string_or_error("allow")?;
            if allow.is_empty() {
                return Err(ReadError::invalid("allow", allow_value.clone(), "it must not be empty"));
            }

            return Ok(Self::Exists { allow });
        }

        match &state_value.unwrap().string_or_error2("state", OneOf(&["present", "missing"]))?[..] {
            "present" => {
                let allow = table.get_or_read_error("allow")?.string_or_error("allow")?;
                Ok(Self::Exists { allow })
            }
            "missing" => {
                if table.get("allow").is_some() {
                    return Err(ReadError::conflict2("allow", "state", state_value.unwrap().clone()));
                }

                Ok(Self::Missing)
            }
            _ => {
                Err(ReadError::invalid("state", state_value.unwrap().clone(), OneOf(&["present", "missing"])))
            }
        }
    }
}

impl Protocol {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let protocol_value = table.get_or_read_error("protocol")?;

        match &protocol_value.string_or_error2("protocol", OneOf(&["tcp", "udp"]))?[..] {
            "tcp" => Ok(Self::TCP),
            "udp" => Ok(Self::UDP),
            _     => Err(ReadError::invalid("protocol", protocol_value.clone(), OneOf(&["tcp", "udp"]))),
        }
    }
}


// ---- running the check ----

/// The interface to the local UFW rules list used by [`UfwCheck`].
pub trait RunUfw {

    /// Primes the command for running.
    fn prime(&mut self) { }

    /// Running the command if it hasn’t been run already, consults the
    /// rules list and returns whether a rule with the given parameters
    /// is present.
    fn find_rule(&self, executor: &mut Executor, portspec: Portspec, protocol: Protocol) -> Result<Option<Rule>, Rc<ExecError>>;
}

#[derive(PartialEq, Debug)]
pub struct Rule {

    /// The network interface for this rule, if specified.
    pub iface: Option<String>,

    /// The `Allow` field for this rule.
    pub allow: String,

    /// Whether this rule has the IPv6 flag.
    pub ipv6: bool,
}

impl<U: RunUfw> RunCheck<U> for UfwCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, ufw: &mut U) {
        ufw.prime();
    }

    fn check(&self, executor: &mut Executor, ufw: &U) -> Vec<CheckResult<Pass, Fail>> {
        info!("Running check");

        let rule = match ufw.find_rule(executor, self.portspec, self.protocol) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        match (&self.condition, rule.as_ref()) {
            (Condition::Exists { allow }, Some(rule)) => {

                if *allow == rule.allow {
                    vec![ CheckResult::Passed(Pass::RuleExists),
                          CheckResult::Passed(Pass::AllowMatches), ]
                }
                else {
                    vec![ CheckResult::Passed(Pass::RuleExists),
                          CheckResult::Failed(Fail::AllowMismatch(rule.allow.clone())), ]
                }
            }
            (Condition::Exists { .. }, None) => {
                vec![ CheckResult::Failed(Fail::RuleMissing) ]
            }
            (Condition::Missing, Some(_)) => {
                vec![ CheckResult::Failed(Fail::RuleExists) ]
            }
            (Condition::Missing, None) => {
                vec![ CheckResult::Passed(Pass::RuleMissing) ]
            }
        }
    }
}

/// The successful result of a UFW check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The rule exists.
    RuleExists,

    /// The `Allow` field of the rule contains the expected value.
    AllowMatches,

    /// No rule matching the criteria exists.
    RuleMissing,
}

/// The failure result of running a UFW check.
#[derive(PartialEq, Debug)]
pub enum Fail {

    /// We expected a rule to exist, but no rule matches the criteria.
    RuleMissing,

    /// The `Allow` field of the existing rule contains a different value.
    AllowMismatch(String),

    /// We expected no rule to exist, but there is an existing rule that
    /// matches the criteria.
    RuleExists,
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuleExists => {
                write!(f, "rule exists")
            }
            Self::AllowMatches => {
                write!(f, "Allow matches")
            }
            Self::RuleMissing => {
                write!(f, "rule missing")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuleMissing => {
                write!(f, "rule missing")
            }
            Self::AllowMismatch(actual_allow) => {
                write!(f, "Allow is ‘{}’", actual_allow)
            }
            Self::RuleExists => {
                write!(f, "rule exists")
            }
        }
    }
}
