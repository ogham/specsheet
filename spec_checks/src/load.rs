use std::collections::BTreeMap;

use serde::Deserialize;

pub use toml::{Value as TomlValue, de::Error as TomlError};


/// The schema of a check document.
pub type CheckDocument = BTreeMap<String, Vec<CheckEntry>>;

/// The type that is parsed from TOML is not just an arbitrary table, it’s an
/// arbitrary table that could have `name` and `tags` fields! This type holds
/// the fields that are common to all checks. (Also: more fields.)
#[derive(Debug, Deserialize)]
pub struct CheckEntry {

    /// The rest of the check-specific fields, which have not been deciphered
    /// yet by their check type’s ‘read’ function.
    #[serde(flatten)]
    pub inner: TomlValue,

    /// The name of the check, which should override the auto-generated
    /// `fmt::Display` name if provided.
    pub name: Option<String>,

    /// A list of tags, which lets the user control which checks get run.
    pub tags: Option<Tags>,
}

/// Each check can have one or more tags.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Tags {
    One(String),
    Many(Vec<String>),
}

/// Parse the given string (that has been read from standard input or a file)
/// from the TOML representing a check document, or return a parse error.
pub fn parse_toml(check_document: &str) -> Result<CheckDocument, TomlError> {
    toml::from_str(check_document)
}
