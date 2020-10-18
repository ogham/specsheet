//! The Homebrew check involves searching through the list of installed
//! formulas to see if one is installed.
//!
//! # Check example
//!
//! ```toml
//! [[homebrew]]
//! cask = "exa"
//! state = "installed"
//! ```
//!
//! # Commands
//!
//! This check works by running `brew`.


use std::fmt;
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// A check against the Homebrew formulas database.
#[derive(PartialEq, Debug)]
pub struct HomebrewCheck {

    /// The name of the Homebrew formula being checked.
    formula_name: FormulaName,

    /// The condition to test it with.
    condition: Condition,
}

/// The name of the formula we are checking.
#[derive(PartialEq, Debug)]
struct FormulaName(String);

/// The condition we are expecting it to be in.
#[derive(PartialEq, Debug)]
enum Condition {

    /// Check that this formula is present in the list.
    Installed,

    /// Check that this formula is _not_ present in the list.
    Missing,
}


// ---- check stuff ----

impl fmt::Display for HomebrewCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { formula_name, condition } = &self;

        match condition {
            Condition::Installed => {
                write!(f, "Formula ‘{}’ is installed", formula_name.0)
            }
            Condition::Missing => {
                write!(f, "Formula ‘{}’ is not installed", formula_name.0)
            }
        }
    }
}


// ---- reading from TOML ----

impl Check for HomebrewCheck {
    const TYPE: &'static str = "homebrew";
}

impl HomebrewCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["formula", "state"])?;

        let formula_name = FormulaName::read(table)?;
        let condition = Condition::read(table)?;
        Ok(Self { formula_name, condition })
    }
}

impl FormulaName {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let name_value = table.get_or_read_error("formula")?;
        let formula_name = name_value.string_or_error("formula")?;

        if formula_name.is_empty() {
            Err(ReadError::invalid("formula", name_value.clone(), "it must not be empty"))
        }
        else if formula_name.contains('/') {
            Err(ReadError::invalid("formula", name_value.clone(), "it must not contain a ‘/’ character"))
        }
        else if formula_name.contains(char::is_whitespace) {
            Err(ReadError::invalid("formula", name_value.clone(), "it must not contain whitespace"))
        }
        else {
            Ok(Self(formula_name))
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
            "installed" | "present" => {
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

/// The interface to the local Homebrew database used by [`HomebrewCheck`].
pub trait RunBrew {

    /// Primes the command for running.
    fn prime(&mut self) { }

    /// Running the database if it hasn’t been run already, consults the
    /// list of packages and returns whether the formula with the given
    /// name is installed.
    fn find_formula(&self, executor: &mut Executor, formula_name: &str) -> Result<bool, Rc<ExecError>>;
}

impl<BC: RunBrew> RunCheck<BC> for HomebrewCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, brew: &mut BC) {
        debug!("Priming brew command");
        brew.prime();
    }

    fn check(&self, executor: &mut Executor, apt: &BC) -> Vec<CheckResult<Pass, Fail>> {
        use self::Condition::*;
        info!("Running check");

        let formula = match apt.find_formula(executor, &self.formula_name.0) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        match (&self.condition, formula) {
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


/// The successful result of an Homebrew check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The formula is installed.
    IsInstalled,

    /// The formula is not installed.
    IsMissing,
}

/// The failure result of running an Homebrew check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Fail {

    /// The formula was meant to be installed, but it was missing.
    IsMissing,

    /// The formula was meant to be _not_ installed, but it was installed.
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
