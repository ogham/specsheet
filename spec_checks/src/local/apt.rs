//! The Apt remote check involves running Apt and searching the list of
//! installed packages it provides.
//!
//! ```toml
//! [[apt]]
//! package = 'httpd'
//! state = 'installed'
//! ```


use std::fmt;
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf};


/// A check against the Apt packages database.
#[derive(PartialEq, Debug)]
pub struct AptCheck {

    /// The name of the Apt package being checked.
    package_name: PackageName,

    /// The condition to test it with.
    condition: Condition,
}

#[derive(PartialEq, Debug)]
struct PackageName(String);

#[derive(PartialEq, Debug)]
enum Condition {

    /// Check that this package is present in the list.
    Installed(PackageVersion),

    /// Check that this package is _not_ present in the list.
    Missing,
}

#[derive(PartialEq, Debug)]
enum PackageVersion {

    Any,

    Specific(String),
}


// ---- the check description ----

impl fmt::Display for AptCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { package_name, condition } = &self;

        match condition {
            Condition::Installed(PackageVersion::Specific(version)) => {
                write!(f, "Package ‘{}’ version ‘{}’ is installed", package_name.0, version)
            }
            Condition::Installed(PackageVersion::Any) => {
                write!(f, "Package ‘{}’ is installed", package_name.0)
            }
            Condition::Missing => {
                write!(f, "Package ‘{}’ is not installed", package_name.0)
            }
        }
    }
}


// ---- reading from TOML ----

impl Check for AptCheck {
    const TYPE: &'static str = "apt";
}

impl AptCheck {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["package", "state", "version"])?;

        let package_name = PackageName::read(table)?;
        let condition = Condition::read(table)?;
        Ok(Self { package_name, condition })
    }
}

impl PackageName {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let name_value = table.get_or_read_error("package")?;
        let package_name = name_value.string_or_error("package")?;

        if package_name.is_empty() {
            Err(ReadError::invalid("package", name_value.clone(), "it must not be empty"))
        }
        else if package_name.contains('/') {
            Err(ReadError::invalid("package", name_value.clone(), "it must not contain a ‘/’ character"))
        }
        else {
            Ok(Self(package_name))
        }
    }
}

impl Condition {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let version = PackageVersion::read(table)?;

        let state_value = match table.get("state") {
            Some(s) => s,
            None    => return Ok(Self::Installed(version)),
        };

        match &state_value.string_or_error2("state", OneOf(&["installed", "missing"]))?[..] {
            "installed" => {
                Ok(Self::Installed(version))
            }
            "missing" => {
                if table.get("version").is_some() {
                    Err(ReadError::conflict2("version", "state", state_value.clone()))
                }
                else {
                    Ok(Self::Missing)
                }
            }
            _ => {
                Err(ReadError::invalid("state", state_value.clone(), OneOf(&["installed", "missing"])))
            }
        }
    }
}

impl PackageVersion {
    pub fn read(table: &TomlValue) -> Result<Self, ReadError> {
        if let Some(version_value) = table.get("version") {
            let version_string = version_value.string_or_error("version")?;

            if version_string.is_empty() {
                return Err(ReadError::invalid("version", version_value.clone(), "it must not be empty"));
            }

            Ok(Self::Specific(version_string))
        }
        else {
            Ok(Self::Any)
        }
    }
}


// ---- running the check ----

/// The interface to the local Apt package database used by [`AptCheck`].
pub trait RunApt {

    /// Prime the command for running.
    fn prime(&mut self) { }

    /// Running the command if it hasn’t been run already, consult the
    /// database and find the installed version of the package with the
    /// given name, if any.
    fn find_package(&self, executor: &mut Executor, package_name: &str) -> Result<Option<String>, Rc<ExecError>>;
}

impl<A: RunApt> RunCheck<A> for AptCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, apt: &mut A) {
        apt.prime();
    }

    fn check(&self, executor: &mut Executor, apt: &A) -> Vec<CheckResult<Pass, Fail>> {
        use self::Condition::*;
        info!("Running check");

        let package = match apt.find_package(executor, &self.package_name.0) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        match (&self.condition, package.as_ref()) {
            (Installed(PackageVersion::Specific(expected_version)), Some(got_version)) => {
                if expected_version == got_version {
                    vec![ CheckResult::Passed(Pass::IsInstalled),
                          CheckResult::Passed(Pass::HasCorrectVersion { got_version: got_version.clone() }) ]
                }
                else {
                    vec![ CheckResult::Passed(Pass::IsInstalled),
                          CheckResult::Failed(Fail::WrongVersion { got_version: got_version.clone() }) ]
                }
            }
            (Installed(PackageVersion::Specific(_expected_version)), None) => {
                vec![ CheckResult::Failed(Fail::IsMissing) ]
            }
            (Installed(PackageVersion::Any), Some(_got_version)) => {
                vec![ CheckResult::Passed(Pass::IsInstalled) ]
            }
            (Installed(PackageVersion::Any), None) => {
                vec![ CheckResult::Failed(Fail::IsMissing) ]
            }
            (Missing, Some(_got_version)) => {
                vec![ CheckResult::Failed(Fail::IsPresent) ]
            }
            (Missing, None) => {
                vec![ CheckResult::Passed(Pass::IsMissing) ]
            }
        }
    }
}

/// The successful result of an Apt check.
#[derive(PartialEq, Debug)]
pub enum Pass {

    /// The package is installed.
    IsInstalled,

    /// The package is not installed.
    IsMissing,

    /// The version of the installed package is correct.
    HasCorrectVersion {
        got_version: String,
    },
}

/// The failure result of running an Apt check.
#[derive(PartialEq, Debug)]
pub enum Fail {

    /// The package was meant to be installed, but it was missing.
    IsMissing,

    /// The package was meant to be _not_ installed, but it was installed.
    IsPresent,

    /// The package was installed, but with the wrong version number.
    WrongVersion {
        got_version: String,
    },
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
            Self::HasCorrectVersion { got_version } => {
                write!(f, "version ‘{}’ is installed", got_version)
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
            Self::IsPresent => {
                write!(f, "it is installed")
            }
            Self::WrongVersion { got_version } => {
                write!(f, "version ‘{}’ is installed", got_version)
            }
        }
    }
}
