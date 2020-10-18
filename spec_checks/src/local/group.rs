//! Group checks
//!
//! # Check example
//!
//! ```toml
//! [[group]]
//! group = 'consul'
//! ```
//!
//! # Commands
//!
//! No commands are run for group checks; Specsheet queries the groups
//! database itself.


use std::fmt;

use log::*;
use users::Group;

use spec_analysis::DataPoint;

use crate::check::{Check, BuiltInCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// A check against the local groups database.
#[derive(PartialEq, Debug)]
pub struct GroupCheck {
    group_name: GroupName,
    condition: Condition,
}

/// The name of the group being checked.
#[derive(PartialEq, Debug)]
struct GroupName(String);

/// The condition we are checking.
#[derive(PartialEq, Debug)]
enum Condition {

    /// The named group should be present.
    Exists,

    /// The named group should not be present.
    Missing,
}


// ---- the check description ----

impl fmt::Display for GroupCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { group_name, condition } = &self;

        match condition {
            Condition::Exists => {
                write!(f, "Group ‘{}’ exists", group_name.0)
            }
            Condition::Missing => {
                write!(f, "Group ‘{}’ does not exist", group_name.0)
            }
        }
    }
}


// ---- reading from TOML ----

impl Check for GroupCheck {
    const TYPE: &'static str = "group";
}

impl GroupCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["group", "state"])?;

        let group_name = GroupName::read(table)?;
        let condition = Condition::read(table)?;
        Ok(Self { group_name, condition })
    }
}

impl GroupName {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let name = table.get_or_read_error("group")?
                        .string_or_error("group")?;

        if name.is_empty() {
            Err(ReadError::invalid("group", name.into(), "it must not be empty"))
        }
        else {
            Ok(Self(name))
        }
    }
}

impl Condition {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let state_value = match table.get("state") {
            Some(s) => s,
            None    => return Ok(Self::Exists),
        };

        match &state_value.string_or_error2("state", OneOf(&["present", "missing"]))?[..] {
            "exists" | "present" => {
                Ok(Self::Exists)
            }
            "absent" | "missing" => {
                Ok(Self::Missing)
            }
            _ => {
                Err(ReadError::invalid("state", state_value.clone(), OneOf(&["present", "missing"])))
            }
        }
    }
}


// ---- analysis properties ----

impl GroupCheck {
    pub fn properties<'a>(&'a self) -> Vec<DataPoint<'a>> {
        let mut points = Vec::new();
        points.push(DataPoint::InvolvesGroup(&*self.group_name.0));
        points
    }
}


// ---- running the check ----

// The interface to the groups list in the passwd database used by
// [`GroupCheck`].
pub trait LookupGroup {

    /// Primes the command for running.
    #[allow(unused)]
    fn prime(&mut self, group_name: &str) { }

    /// Running the command if it hasn’t been run already, consults the
    /// groups list and returns a group depending on whether or not one
    /// is present.
    fn lookup_group(&self, group_name: &str) -> Option<Group>;
}

impl<P: LookupGroup> BuiltInCheck<P> for GroupCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, passwd: &mut P) {
        passwd.prime(&self.group_name.0);
    }

    fn check(&self, passwd: &P) -> Vec<CheckResult<Pass, Fail>> {
        info!("Running check");

        let group = passwd.lookup_group(&self.group_name.0);

        match (&self.condition, &group) {
            (Condition::Exists, Some(_)) => {
                vec![ CheckResult::Passed(Pass::GroupExists) ]
            }
            (Condition::Exists, None) => {
                vec![ CheckResult::Failed(Fail::GroupIsMissing) ]
            }
            (Condition::Missing, Some(_)) => {
                vec![ CheckResult::Failed(Fail::GroupExists) ]
            }
            (Condition::Missing, None) => {
                vec![ CheckResult::Passed(Pass::GroupIsMissing) ]
            }
        }
    }
}

/// The successful result of a group check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The group exists.
    GroupExists,

    /// The group does not exist.
    GroupIsMissing,
}

/// The failure result of running a group check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Fail {

    /// The group was meant to exist, but it’s missing.
    GroupIsMissing,

    /// The group was meant to be missing, but it exists.
    GroupExists,
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----
impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GroupExists => {
                write!(f, "it exists")
            }
            Self::GroupIsMissing => {
                write!(f, "it is missing")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GroupIsMissing => {
                write!(f, "it exists")
            }
            Self::GroupExists => {
                write!(f, "it is missing")
            }
        }
    }
}
