use std::fmt;
use std::path::Path;


/// A property of a check, used during analysis.
pub enum DataPoint<'a> {

    /// The check has something to do with the given path.
    InvolvesPath(&'a Path),

    /// The check has something to do with the user with the given name.
    InvolvesUser(&'a str),

    /// The check has something to do with the group with the given name.
    InvolvesGroup(&'a str),
}

impl<'a> fmt::Display for DataPoint<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvolvesPath(path)    => write!(f, "involving path ‘{}’", path.display()),
            Self::InvolvesUser(user)    => write!(f, "involving user ‘{}’", user),
            Self::InvolvesGroup(group)  => write!(f, "involving group ‘{}’", group),
        }
    }
}
