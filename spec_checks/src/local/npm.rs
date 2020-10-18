//! The NPM check involves searching through the list of installed packages to see
//! if one is installed.
//!
//! # Check example
//!
//! ```toml
//! [[npm]]
//! package = "typescript"
//! ```
//!
//! # Commands
//!
//! This check works by running `npm`.


use std::fmt;
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// A check against the installed npm packages list.
#[derive(PartialEq, Debug)]
pub struct NpmCheck {
    package_name: PackageName,
    condition: Condition,
}

/// The name of the package we are checking.
#[derive(PartialEq, Debug)]
struct PackageName(String);

/// The condition we are expecting it to be in.
#[derive(PartialEq, Debug)]
enum Condition {

    /// We expect it to be installed.
    Installed,

    /// We expect it to be missing.
    Missing,
}


// ---- the check description ----

impl fmt::Display for NpmCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { package_name, condition } = &self;

        match condition {
            Condition::Installed => {
                write!(f, "Package ‘{}’ is installed", package_name.0)
            }
            Condition::Missing => {
                write!(f, "Package ‘{}’ is not installed", package_name.0)
            }
        }
    }
}


// ---- reading from TOML ----

impl Check for NpmCheck {
    const TYPE: &'static str = "npm";
}

impl NpmCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["package", "state", "version"])?;

        let package_name = PackageName::read(table)?;
        let condition = Condition::read(table)?;
        Ok(Self { package_name, condition })
    }
}

impl PackageName {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let name_value = table.get_or_read_error("package")?;
        let name = name_value.string_or_error("package")?;

        if name.is_empty() || name.contains('/') {
            Err(ReadError::invalid("package", name_value.clone(), "it must not be empty"))
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
            "installed" | "present" => {
                Ok(Self::Installed)
            }
            "uninstalled" | "missing" => {
                Ok(Self::Missing)
            }
            _ => {
                Err(ReadError::invalid("state", state.clone(), OneOf(&["installed", "missing"])))
            }
        }
    }
}


// ---- running the check ----

/// The interface to the local npm package database used by [`NpmCheck`].
pub trait RunNpm {

    /// Prime the command for running.
    fn prime(&mut self) { }

    /// Running the command if it hasn’t been run already, consul the
    /// database and return whether a package with the given name is
    /// installed.
    fn find_package(&self, executor: &mut Executor, package_name: &str) -> Result<bool, Rc<ExecError>>;
}

impl<N: RunNpm> RunCheck<N> for NpmCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, npm: &mut N) {
        npm.prime();
    }

    fn check(&self, executor: &mut Executor, npm: &N) -> Vec<CheckResult<Pass, Fail>> {
        use self::Condition::*;
        info!("Running check");

        let package = match npm.find_package(executor, &self.package_name.0) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        match (&self.condition, package) {
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

/// The successful result of an npm check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The package is installed.
    IsInstalled,

    /// The package is missing.
    IsMissing,
}

/// The failure result of running an npm check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Fail {

    /// The package is missing, but was meant to be installed.
    IsMissing,

    /// The package is installed, but was meant to be missing.
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
