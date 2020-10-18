//! The Homebrew Tap check involves searching through the list of installed
//! taps to see if one is present.
//!
//! # Check example
//!
//! ```toml
//! [[homebrew_tap]]
//! cask = "homebrew/cask-versions"
//! state = "present"
//! ```
//!
//! # Commands
//!
//! This check works by running `brew tap`.


use std::fmt;
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// The Homebrew Tap check checks the list of taps.
#[derive(PartialEq, Debug)]
pub struct HomebrewTapCheck {
    tap_name: TapName,
    condition: Condition,
}

/// The name of the tap we are checking.
#[derive(PartialEq, Debug)]
struct TapName(String);

/// The condition we are expecting it to be in.
#[derive(PartialEq, Debug)]
enum Condition {

    /// We expect it to be installed.
    Present,

    /// We expect it to be _not_ installed.
    Missing,
}


// ---- the check description ----

impl fmt::Display for HomebrewTapCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { tap_name, condition } = &self;

        match condition {
            Condition::Present => {
                write!(f, "Tap ‘{}’ is present", tap_name.0)
            }
            Condition::Missing => {
                write!(f, "Tap ‘{}’ is not present", tap_name.0)
            }
        }
    }
}


// ---- reading from TOML ----

impl Check for HomebrewTapCheck {
    const TYPE: &'static str = "homebrew_tap";
}

impl HomebrewTapCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["tap", "state"])?;

        let tap_name = TapName::read(table)?;
        let condition = Condition::read(table)?;
        Ok(Self { tap_name, condition })
    }
}

impl TapName {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let name_value = table.get_or_read_error("tap")?;
        let name = name_value.string_or_error("tap")?;

        if name.is_empty() || ! name.contains('/') {
            Err(ReadError::invalid("tap", name_value.clone(), "it must not be empty"))
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
            None    => return Ok(Self::Present),
        };

        match &state_value.string_or_error2("state", OneOf(&["present", "missing"]))?[..] {
            "present" => {
                Ok(Self::Present)
            }
            "missing" => {
                Ok(Self::Missing)
            }
            _ => {
                Err(ReadError::invalid("state", state_value.clone(), OneOf(&["present", "missing"])))
            }
        }
    }
}


// ---- running the check ----

/// The interface to the list of Homebrew taps used by [`HomebrewTapCheck`].
pub trait RunBrewTap {

    /// Primes the command for running.
    fn prime(&mut self) { }

    /// Running the command if it hasn’t been run already, consults the
    /// database and returns whether a tap with the given name is
    /// present.
    fn find_tap(&self, executor: &mut Executor, tap_name: &str) -> Result<bool, Rc<ExecError>>;
}

impl<BT: RunBrewTap> RunCheck<BT> for HomebrewTapCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, brew_tap: &mut BT) {
        brew_tap.prime();
    }

    fn check(&self, executor: &mut Executor, brew_tap: &BT) -> Vec<CheckResult<Pass, Fail>> {
        use self::Condition::*;
        info!("Running check");

        let tap = match brew_tap.find_tap(executor, &self.tap_name.0) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        match (&self.condition, tap) {
            (Present, true) => {
                vec![ CheckResult::Passed(Pass::IsPresent) ]
            }
            (Present, false) => {
                vec![ CheckResult::Failed(Fail::IsMissing) ]
            }
            (Missing, true) => {
                vec![ CheckResult::Failed(Fail::IsPresent) ]
            }
            (Missing, false) => {
                vec![ CheckResult::Passed(Pass::IsMissing) ]
            }
        }
    }
}

/// The successful result of a Homebrew tap check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The tap is present.
    IsPresent,

    /// The tap is missing.
    IsMissing,
}

/// The failure result of running a Homebrew tap check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Fail {

    /// The tap was meant to be installed, but it’s missing.
    IsMissing,

    /// The tap was meant to be missing, but it’s installed.
    IsPresent,
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IsPresent => {
                write!(f, "it is present")
            }
            Self::IsMissing => {
                write!(f, "it is not present")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IsMissing => {
                write!(f, "it is not present")
            }
            Self::IsPresent => {
                write!(f, "it is present")
            }
        }
    }
}
