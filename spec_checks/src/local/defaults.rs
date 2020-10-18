//! The Defaults remote check involves checking values against the macOS
//! preferences database.
//!
//! # Check example
//!
//! ```toml
//! [[defaults]]
//! domain = "com.apple.Finder"
//! key = "ShowExternalHardDrivesOnDesktop"
//! value = "1"
//! ```
//!
//! # Commands
//!
//! This check works by running `defaults` once per domain+key combination
//! that needs to be checked.


use std::fmt;
use std::path::PathBuf;
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf, Rewrites};


/// The **defaults check** checks the macOS preferences databases.
#[derive(PartialEq, Debug)]
pub struct DefaultsCheck {
    location: DefaultsLocation,
    condition: Condition,
}

/// The absolute path to a defaults key to query.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash)]
pub struct DefaultsLocation {

    /// Which database is being accessed.
    pub place: DefaultsPlace,

    /// The key to access the value at.
    pub key: String,
}

/// Which database is being accessed.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash)]
pub enum DefaultsPlace {

    /// The key exists in a domain in the global database.
    Domain(String),

    /// The key exists in a local database inside the given file.
    File(PathBuf),
}


/// The condition we are checking about the value.
#[derive(PartialEq, Debug)]
enum Condition {

    /// It should exist, with the given value.
    Present(String),

    /// It should be missing.
    Missing,
}


// ---- the check description ----

impl fmt::Display for DefaultsCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { location, condition } = &self;

        match condition {
            Condition::Present(value) => {
                write!(f, "Defaults value ‘{}/{}’ is ‘{}’", location.place, location.key, value)?;
            }
            Condition::Missing => {
                write!(f, "Defaults value ‘{}/{}’ is absent", location.place, location.key)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for DefaultsPlace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(domain) => write!(f, "{}", domain),
            Self::File(path)     => write!(f, "{}", path.display()),
        }
    }
}


// ---- reading from TOML ----

impl Check for DefaultsCheck {
    const TYPE: &'static str = "defaults";
}

impl DefaultsCheck {
    pub fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["domain", "key", "state", "value", "file"])?;

        let location = DefaultsLocation::read(table, rewrites)?;
        let condition = Condition::read(table)?;
        Ok(Self { location, condition })
    }
}

impl DefaultsLocation {
    pub fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        let key_value = table.get_or_read_error("key")?;
        let key = key_value.string_or_error("key")?;

        let domain = table.get("domain").map(|e| e.string_or_error("domain")).transpose()?;
        if domain.as_ref().map_or(false, String::is_empty) {
            return Err(ReadError::invalid("domain", table.get("domain").unwrap().clone(), "it must not be empty"));
        }

        let file = table.get("file").map(|e| e.string_or_error("file")).transpose()?;
        if file.as_ref().map_or(false, String::is_empty) {
            return Err(ReadError::invalid("file", table.get("file").unwrap().clone(), "it must not be empty"));
        }

        if key.is_empty() {
            return Err(ReadError::invalid("key", key_value.clone(), "it must not be empty"));
        }

        match (domain, file) {
            (Some(domain), None) => {
                let place = DefaultsPlace::Domain(domain);
                Ok(Self { place, key })
            }
            (None, Some(file)) => {
                let place = DefaultsPlace::File(rewrites.path(file));
                Ok(Self { place, key })
            }
            (None, None) => {
                // Recommend ‘domain’ because it’s the more common one
                Err(ReadError::MissingParameter { parameter_name: "domain" })
            }
            (Some(_), Some(_)) => {
                Err(ReadError::conflict("domain", "file"))
            }
        }
    }
}

impl Condition {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {

        let value = table.get("value").map(|v| {
            if let Some(integer) = v.as_integer() {
                Ok(integer.to_string())
            }
            else if let Some(string) = v.as_str() {
                Ok(string.to_string())
            }
            else {
                Err(ReadError::invalid("value", table.get("value").unwrap().clone(), "it must be a string or a number"))
            }
        }).transpose()?;

        if let Some(state_value) = table.get("state") {
            match &state_value.string_or_error2("state", OneOf(&["present", "absent"]))?[..] {
                "present" => {
                    // continue
                }
                "absent" => {
                    if value.is_some() {
                        return Err(ReadError::conflict2("value", "state", state_value.clone()));
                    }
                    else {
                        return Ok(Condition::Missing);
                    }
                }
                _ => {
                    return Err(ReadError::invalid("state", state_value.clone(), OneOf(&["present", "absent"])));
                }
            }
        }

        if let Some(value) = value {
            Ok(Condition::Present(value))
        }
        else {
            Err(ReadError::MissingParameter { parameter_name: "value" })
        }
    }
}


// ---- running the check ----

/// The interface to the local defaults database used by [`DefaultsCheck`].
pub trait RunDefaults {

    /// Prime the command for running, to access the given defaults location.
    #[allow(unused)]
    fn prime(&mut self, location: &DefaultsLocation) { }

    /// Running the command if it hasn't been run already, examines the
    /// output and returns it as a string.
    fn get_value(&self, executor: &mut Executor, location: &DefaultsLocation) -> Result<Option<Rc<str>>, Rc<ExecError>>;
}

impl<D: RunDefaults> RunCheck<D> for DefaultsCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, defaults: &mut D) {
        defaults.prime(&self.location);
    }

    fn check(&self, executor: &mut Executor, defaults: &D) -> Vec<CheckResult<Pass, Fail>> {
        use self::Condition::*;
        info!("Running check");

        let value = match defaults.get_value(executor, &self.location) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        match (&self.condition, value.as_ref()) {
            (Present(expected_value), Some(got_value)) => {
                if expected_value == &**got_value {
                    vec![ CheckResult::Passed(Pass::ValueMatches) ]
                }
                else {
                    vec![ CheckResult::Failed(Fail::ValueMismatch { got_value: got_value.to_string() }) ]
                }
            }
            (Present(_expected_value), None) => {
                vec![ CheckResult::Failed(Fail::IsMissing) ]
            }
            (Missing, Some(_got_value)) => {
                vec![ CheckResult::Failed(Fail::IsPresent) ]
            }
            (Missing, None) => {
                vec![ CheckResult::Passed(Pass::IsMissing) ]
            }
        }
    }
}

/// The successful result of a Defaults check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The value matches the expected value.
    ValueMatches,

    /// The value is missing.
    IsMissing,
}

/// The failure result of running a Defaults check.
#[derive(PartialEq, Debug)]
pub enum Fail {

    /// The actual value did not match the expected value.
    ValueMismatch {
        got_value: String,
    },

    /// A value was meant to exist, but it's missing.
    IsMissing,

    /// A value was meant to be missing, but one exists.
    IsPresent,

    /// The input file does not actually exist.
    MissingFile,
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----
impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValueMatches => {
                write!(f, "the value matches")
            }
            Self::IsMissing => {
                write!(f, "value is missing")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValueMismatch { got_value } => {
                write!(f, "values do not match; got ‘{}’", got_value)
            }
            Self::IsMissing => {
                write!(f, "value is missing")
            }
            Self::IsPresent => {
                write!(f, "a value is present")
            }
            Self::MissingFile => {
                write!(f, "plist file does not exist!")
            }
        }
    }
}
