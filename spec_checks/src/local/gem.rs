//! The Gem check involves searching through the list of installed gems to see
//! if one is installed.
//!
//! # Check example
//!
//! ```toml
//! [[gem]]
//! gem = "sinatra"
//! ```
//!
//! # Commands
//!
//! This check works by running `gem`.


use std::fmt;
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// The Gem check checks the list of installed gems.
#[derive(PartialEq, Debug)]
pub struct GemCheck {
    gem_name: GemName,
    condition: Condition,
}

/// The name of the gem we are checking.
#[derive(PartialEq, Debug)]
struct GemName(String);

/// The condition we are checking the gem to be in.
#[derive(PartialEq, Debug)]
enum Condition {

    /// The gem should be installed.
    Installed,

    /// The gem should be missing.
    Missing,
}


// ---- the check description ----

impl fmt::Display for GemCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { gem_name, condition } = &self;

        match condition {
            Condition::Installed => {
                write!(f, "Gem ‘{}’ is installed", gem_name.0)
            }
            Condition::Missing => {
                write!(f, "Gem ‘{}’ is not installed", gem_name.0)
            }
        }
    }
}


// ---- reading from TOML ----

impl Check for GemCheck {
    const TYPE: &'static str = "gem";
}

impl GemCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["gem", "state"])?;

        let gem_name = GemName::read(table)?;
        let condition = Condition::read(table)?;
        Ok(Self { gem_name, condition })
    }
}

impl GemName {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let name_value = table.get_or_read_error("gem")?;
        let name = name_value.string_or_error("gem")?;

        if name.is_empty() {
            Err(ReadError::invalid("gem", name_value.clone(), "it must not be empty"))
        }
        else if name.contains('/') {
            Err(ReadError::invalid("gem", name_value.clone(), "it must not contain a ‘/’ character"))
        }
        else if name.contains(char::is_whitespace) {
            Err(ReadError::invalid("gem", name_value.clone(), "it must not contain whitespace"))
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
            None    => return Ok(Self::Installed),
        };

        match &state_value.string_or_error2("state", OneOf(&["installed", "missing"]))?[..] {
            "installed" => {
                Ok(Self::Installed)
            }
            "missing" => {
                Ok(Self::Missing)
            }
            _ => {
                Err(ReadError::invalid("state", state_value.clone(), OneOf(&["installed", "missing"])))
            }
        }
    }
}


// ---- running the check ----

/// The interface to the local Rubygems database used by [`GemCheck`].
pub trait RunGem {

    /// Prime the command for running.
    fn prime(&mut self) { }

    /// Running the command if it hasn’t been run already, consult the
    /// database and return whether it says the given package is
    /// installed.
    fn find_gem(&self, executor: &mut Executor, gem_name: &str) -> Result<bool, Rc<ExecError>>;
}

impl<G: RunGem> RunCheck<G> for GemCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, gem: &mut G) {
        gem.prime();
    }

    fn check(&self, executor: &mut Executor, gem: &G) -> Vec<CheckResult<Pass, Fail>> {
        use self::Condition::*;
        info!("Running check");

        let gem = match gem.find_gem(executor, &self.gem_name.0) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        match (&self.condition, gem) {
            (Installed, true) => {
                vec![ CheckResult::Passed(Pass::IsInstalled) ]
            }
            (Installed, false) => {
                vec![ CheckResult::Failed(Fail::IsMissing) ]
            }
            (Missing, true) => {
                vec![ CheckResult::Failed(Fail::IsInstalled) ]
            }
            (Missing, false) => {
                vec![ CheckResult::Passed(Pass::IsMissing) ]
            }
        }
    }
}

/// The successful result of a Gem check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The gem is installed.
    IsInstalled,

    /// The gem is not installed.
    IsMissing,
}

/// The failure result of running a Gem check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Fail {

    /// The gem was meant to be installed, but it’s missing.
    IsMissing,

    /// The gem was meant to be missing, but it’s installed.
    IsInstalled,
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IsInstalled => {
                write!(f, "it is installed")
            }
            Self::IsMissing => {
                write!(f, "it is not installed")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IsMissing => {
                write!(f, "it is not installed")
            }
            Self::IsInstalled => {
                write!(f, "it is installed")
            }
        }
    }
}
