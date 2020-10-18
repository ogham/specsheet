//! The Homebrew Cask check involves searching through the list of installed
//! casks to see if one is installed.
//!
//! # Check example
//!
//! ```toml
//! [[homebrew_cask]]
//! cask = "micro-snitch"
//! state = "present"
//! ```
//!
//! # Commands
//!
//! This check works by running `brew cask`.


use std::fmt;
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// The Homebrew Cask check checks the list of installed casks.
#[derive(PartialEq, Debug)]
pub struct HomebrewCaskCheck {
    cask_name: CaskName,
    condition: Condition,
}

/// The name of the cask we are checking.
#[derive(PartialEq, Debug)]
struct CaskName(String);

/// The condition we are expecting it to be in.
#[derive(PartialEq, Debug)]
enum Condition {

    /// We expect the cask to be installed.
    Installed,

    /// We expected the cask to _not_ be installed.
    Missing,
}



// ---- the check description ----

impl fmt::Display for HomebrewCaskCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { cask_name, condition } = &self;

        match condition {
            Condition::Installed => {
                write!(f, "Cask ‘{}’ is installed", cask_name.0)
            }
            Condition::Missing => {
                write!(f, "Cask ‘{}’ is not installed", cask_name.0)
            }
        }
    }
}


// ---- reading from TOML ----

impl Check for HomebrewCaskCheck {
    const TYPE: &'static str = "homebrew_cask";
}

impl HomebrewCaskCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["cask", "state"])?;

        let cask_name = CaskName::read(table)?;
        let condition = Condition::read(table)?;
        Ok(Self { cask_name, condition })
    }
}

impl CaskName {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let name = table.get_or_read_error("cask")?
                        .string_or_error("cask")?;

        if name.is_empty() {
            Err(ReadError::invalid("cask", name.into(), "it must not be empty"))
        }
        else if name.contains('/') {
            Err(ReadError::invalid("cask", name.into(), "it must not contain a ‘/’ character"))
        }
        else {
            Ok(Self(name))
        }
    }
}

impl Condition {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let state = match table.get("state") {
            Some(s) => s,
            None    => return Ok(Self::Installed),
        };

        match &state.string_or_error2("state", OneOf(&["installed", "missing"]))?[..] {
            "installed" => {
                Ok(Self::Installed)
            }
            "missing" => {
                Ok(Self::Missing)
            }
            _ => {
                Err(ReadError::invalid("state", state.clone(), OneOf(&["installed", "missing"])))
            }
        }
    }
}


// ---- running the check ----

/// The interface to the local Homebrew Cask list.
pub trait RunBrewCask {

    /// Prime the command for running.
    fn prime(&mut self) { }

    /// Running the command if it hasn't been run already, consults the
    /// list and returns whether a cask with the given name is present.
    fn find_cask(&self, executor: &mut Executor, cask_name: &str) -> Result<bool, Rc<ExecError>>;
}

impl<BC: RunBrewCask> RunCheck<BC> for HomebrewCaskCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, brew_cask: &mut BC) {
         brew_cask.prime();
    }

    fn check(&self, executor: &mut Executor, brew_cask: &BC) -> Vec<CheckResult<Pass, Fail>> {
        use self::Condition::*;
        info!("Running check");

        let cask = match brew_cask.find_cask(executor, &self.cask_name.0) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        match (&self.condition, cask) {
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

/// The successful result of a Cask check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The cask is installed.
    IsInstalled,

    /// The cask is missing.
    IsMissing,
}

/// The failure result of running a Cask check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Fail {

    /// The cask was meant to be installed, but it’s missing.
    IsMissing,

    /// The cask was meant to be missing, but it’s installed.
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
