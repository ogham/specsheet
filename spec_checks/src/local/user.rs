//! User checks
//!
//! # Check example
//!
//! ```toml
//! [[user]]
//! user = 'consul'
//! ```
//!
//! # Commands
//!
//! No commands are run for user checks; Specsheet queries the users database
//! itself.


use std::fmt;
use std::path::PathBuf;

use log::*;
use users::User;

use spec_analysis::DataPoint;

use crate::check::{Check, BuiltInCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf, Rewrites};


/// A check against the local users database.
#[derive(PartialEq, Debug)]
pub struct UserCheck {
    user_name: UserName,
    condition: Condition,
}

/// The name of the user being checked.
#[derive(PartialEq, Debug)]
struct UserName(String);

/// The condition we are checking.
#[derive(PartialEq, Debug)]
enum Condition {

    /// This user exists (with the given extra checks).
    Exists(UserDataChecks),

    /// No user was found with the input username.
    Missing,
}

/// Extra checks for a user that exists.
#[derive(PartialEq, Debug)]
struct UserDataChecks {

    /// If given, what this user’s login shell should be.
    login_shell: Option<PathBuf>,

    /// If given a list of names of groups that this user should be in.
    groups: Option<Vec<String>>,
}


// ---- the check description ----

impl fmt::Display for UserCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { user_name, condition } = &self;

        match condition {
            Condition::Exists(checks) => {
                write!(f, "User ‘{}’ exists", user_name.0)?;

                if let Some(ls) = &checks.login_shell {
                    write!(f, " with login shell ‘{}’", ls.display())?;
                }

                if let Some(gs) = &checks.groups {
                    write!(f, " and is a member of groups")?;

                    for (i, g) in gs.iter().enumerate() {
                        if i > 0 { write!(f, " and")?; }
                        write!(f, " ‘{}’", g)?;
                    }
                }

                Ok(())
            }
            Condition::Missing => {
                write!(f, "User ‘{}’ does not exist", user_name.0)
            }
        }
    }
}


// ---- reading from TOML ----

impl Check for UserCheck {
    const TYPE: &'static str = "user";
}

impl UserCheck {
    pub fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["user", "state", "login_shell", "groups"])?;

        let user_name = UserName::read(table)?;
        let condition = Condition::read(table, rewrites)?;
        Ok(Self { user_name, condition })
    }
}

impl UserName {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let name_value = table.get_or_read_error("user")?;
        let name_str = name_value.string_or_error("user")?;

        if name_str.is_empty() {
            Err(ReadError::invalid("user", name_value.clone(), "it must not be empty"))
        }
        else {
            Ok(Self(name_str))
        }
    }
}

impl Condition {
    fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        let state = match table.get("state") {
            Some(s) => s,
            None    => return Ok(Self::Exists(UserDataChecks::read(table, rewrites)?)),
        };

        match &state.string_or_error2("state", OneOf(&["present", "missing"]))?[..] {
            "present" => {
                Ok(Self::Exists(UserDataChecks::read(table, rewrites)?))
            }
            "missing" => {
                Ok(Self::Missing)
            }
            _ => {
                Err(ReadError::invalid("state", state.clone(), OneOf(&["present", "missing"])))
            }
        }
    }
}

impl UserDataChecks {
    fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        let login_shell = table.get("login_shell")
                               .map(|e| {
                                   let s = e.string_or_error("login_shell")?;
                                   if s.is_empty() {
                                       Err(ReadError::invalid("login_shell", e.clone(), "it must not be empty"))
                                   }
                                   else {
                                       Ok(rewrites.path(s))
                                   }
                                })
                               .transpose()?
                               .map(PathBuf::from);

        let groups = table.get("groups")
                          .map(|e| e.string_array_or_read_error("groups"))
                          .transpose()?;

        if groups.as_ref().map_or(false, |gs| gs.iter().any(String::is_empty)) {
            return Err(ReadError::invalid("groups", table.get("groups").unwrap().clone(), "group names must not be empty"));
        }

        Ok(Self { login_shell, groups })
    }
}


// ---- analysis properties ----

impl UserCheck {
    pub fn properties<'a>(&'a self) -> Vec<DataPoint<'a>> {
        let mut points = Vec::new();
        points.push(DataPoint::InvolvesUser(&*self.user_name.0));
        points
    }
}


// ---- running the check ----

/// The interface to the users list in the passwd database used by
/// [`UserCheck`].
pub trait LookupUser {

    /// Primes the command for running.
    #[allow(unused)]
    fn prime(&mut self, username: &str) { }

    /// Running the command if it hasn’t been run already, consults the
    /// users list and returns a user depending on whether or not one is
    /// present.
    fn lookup_user(&self, username: &str) -> Option<User>;
}

impl<P: LookupUser> BuiltInCheck<P> for UserCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, passwd: &mut P) {
        passwd.prime(&self.user_name.0);
    }

    fn check(&self, passwd: &P) -> Vec<CheckResult<Pass, Fail>> {
        use users::os::unix::UserExt;

        info!("Running check");

        let user = passwd.lookup_user(&self.user_name.0);

        match (&self.condition, &user) {
            (Condition::Exists(checks), Some(u)) => {
                let mut results = vec![ CheckResult::Passed(Pass::UserExists) ];

                if let Some(login_shell) = &checks.login_shell {
                    if u.shell() == login_shell {
                        results.push(CheckResult::Passed(Pass::UserHasLoginShell));
                    }
                    else {
                        results.push(CheckResult::Failed(Fail::UserHasDifferentLoginShell));
                    }
                }

                if let Some(input_group_names) = &checks.groups {
                    if let Some(actual_group_names) = u.groups() {
                        for group_name in input_group_names {
                            if actual_group_names.iter().any(|g| g.name() == &**group_name) {
                                results.push(CheckResult::Passed(Pass::UserIsMemberOfGroup(group_name.clone())));
                            }
                            else {
                                results.push(CheckResult::Failed(Fail::UserIsNotMemberOfGroup(group_name.clone())));
                            }
                        }
                    }
                    else {
                        unimplemented!("They asked for group names, but there are no groups");
                    }
                }

                results
            }
            (Condition::Exists(_checks), None) => {
                vec![ CheckResult::Failed(Fail::UserIsMissing) ]
            }
            (Condition::Missing, Some(_)) => {
                vec![ CheckResult::Failed(Fail::UserExists) ]
            }
            (Condition::Missing, None) => {
                vec![ CheckResult::Passed(Pass::UserIsMissing) ]
            }
        }
    }
}

/// The successful result of a user check.
#[derive(PartialEq, Debug)]
pub enum Pass {

    /// The user exists.
    UserExists,

    /// The user does not exist.
    UserIsMissing,

    /// The user is a member of the given group.
    UserIsMemberOfGroup(String),

    /// The user has the correct login shell.
    UserHasLoginShell,
}

/// The failure result of running a user check.
#[derive(PartialEq, Debug)]
pub enum Fail {

    /// The user was meant to exist, but they're missing.
    UserIsMissing,

    /// The user was meant to be missing, but they exist.
    UserExists,

    /// The user was meant to be a member of the given group, but
    /// they’re not.
    UserIsNotMemberOfGroup(String),

    /// The user was meant to have a certain login shell, but they
    /// have a different one.
    UserHasDifferentLoginShell,
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----
impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserExists => {
                write!(f, "user exists")
            }
            Self::UserIsMissing => {
                write!(f, "user is missing")
            }

            Self::UserIsMemberOfGroup(group) => {
                write!(f, "user is member of group ‘{}’", group)
            }
            Self::UserHasLoginShell => {
                write!(f, "user has correct login shell")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserIsMissing => {
                write!(f, "user is missing")
            }
            Self::UserExists => {
                write!(f, "user exists")
            }

            Self::UserIsNotMemberOfGroup(group) => {
                write!(f, "user is not member of group ‘{}’", group)
            }
            Self::UserHasDifferentLoginShell => {
                write!(f, "user has different login shell")
            }
        }
    }
}
