use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use log::*;
use shellexpand::tilde as expand_tilde;
pub use toml::Value as TomlValue;


/// Extra methods added to TOML values.
pub trait ValueExtras {

    /// Get a value from this table, or return an error.
    fn get_or_read_error(&self, parameter_name: &'static str) -> Result<&TomlValue, ReadError>;

    /// Make sure the value is a table, or return an error.
    fn ensure_table(&self, parameter_name: &'static str) -> Result<(), ReadError>;

    /// Get the contents from this string, or return an error.
    fn number_or_error(&self, parameter_name: &'static str) -> Result<i64, ReadError>;

    /// Get the contents from this string, or return an error.
    fn boolean_or_error(&self, parameter_name: &'static str) -> Result<bool, ReadError>;

    /// Get the contents from this string, or return an error.
    fn string_or_error(&self, parameter_name: &'static str) -> Result<String, ReadError>;

    /// Get the contents from this string, or return an error.
    fn string_or_error2(&self, parameter_name: &'static str, display: impl fmt::Display + Sized + 'static) -> Result<String, ReadError>;

    /// Get a string array from this table, or return an error.
    fn string_array_or_read_error(&self, parameter_name: &'static str) -> Result<Vec<String>, ReadError>;

    /// Get a string-to-string map from this table, or return an error.
    fn string_map_or_read_error(&self, parameter_name: &'static str) -> Result<BTreeMap<String, String>, ReadError>;

    /// Returns an error if this table has any keys in it other than these.
    fn ensure_only_keys(&self, parameter_names: &'static [&'static str]) -> Result<(), ReadError>;
}

impl ValueExtras for TomlValue {
    fn get_or_read_error(&self, parameter_name: &'static str) -> Result<&TomlValue, ReadError> {
        match self.get(parameter_name) {
            Some(val) => Ok(val),
            None      => Err(ReadError::MissingParameter { parameter_name }),
        }
    }

    fn ensure_table(&self, parameter_name: &'static str) -> Result<(), ReadError> {
        match self.as_table() {
            Some(_) => {
                Ok(())
            }
            None => {
                Err(ReadError::invalid(parameter_name, self.clone(), "it must be a table"))
            }
        }
    }

    fn number_or_error(&self, parameter_name: &'static str) -> Result<i64, ReadError> {
        match self.as_integer() {
            Some(s) => {
                Ok(s)
            }
            None => {
                Err(ReadError::invalid(parameter_name, self.clone(), "it must be an integer"))
            }
        }
    }

    fn boolean_or_error(&self, parameter_name: &'static str) -> Result<bool, ReadError> {
        match self.as_bool() {
            Some(s) => {
                Ok(s)
            }
            None => {
                Err(ReadError::invalid(parameter_name, self.clone(), "it must be a boolean"))
            }
        }
    }

    fn string_or_error(&self, parameter_name: &'static str) -> Result<String, ReadError> {
        match self.as_str() {
            Some(s) => {
                Ok(s.into())
            }
            None => {
                Err(ReadError::invalid(parameter_name, self.clone(), "it must be a string"))
            }
        }
    }

    fn string_or_error2(&self, parameter_name: &'static str, display: impl fmt::Display + Sized + 'static) -> Result<String, ReadError> {
        match self.as_str() {
            Some(s) => {
                Ok(s.into())
            }
            None => {
                Err(ReadError::invalid(parameter_name, self.clone(), display))
            }
        }
    }

    fn string_array_or_read_error(&self, parameter_name: &'static str) -> Result<Vec<String>, ReadError> {
        let mut vec = Vec::new();

        let array = match self.as_array() {
            Some(a) => a,
            None    => return Err(ReadError::invalid(parameter_name, self.clone(), "it must be an array of strings")),
        };

        for el in array {
            if let Some(s) = el.as_str() {
                vec.push(s.into());
            }
            else {
                return Err(ReadError::invalid(parameter_name, self.clone(), "it must be an array of strings"));
            }
        }

        Ok(vec)
    }

    fn string_map_or_read_error(&self, parameter_name: &'static str) -> Result<BTreeMap<String, String>, ReadError> {
        let mut map = BTreeMap::new();

        let table = match self.as_table() {
            Some(a) => a,
            None    => return Err(ReadError::invalid(parameter_name, self.clone(), "it must be a map of strings to strings")),
        };

        for (k, v) in table {
            map.insert(k.clone(), v.string_or_error(parameter_name)?);
        }

        Ok(map)
    }

    fn ensure_only_keys(&self, keys: &[&str]) -> Result<(), ReadError> {
        if let Some(t) = self.as_table() {
            if let Some(invalid_param) = t.keys().find(|key| ! keys.iter().any(|k| k == key)) {
                Err(ReadError::UnknownParameter { parameter_name: invalid_param.into() })
            }
            else {
                Ok(())
            }
        }
        else {
            panic!("Not a table: {:#?}", self)
        }
    }
}


/// A general error that can occur while reading a check from a TOML value.
pub enum ReadError {

    /// A table required a key that was missing.
    MissingParameter {
        parameter_name: &'static str,
    },

    /// A table contained a key we don't know about.
    UnknownParameter {
        parameter_name: String,
    },

    /// A key was set to a value that was not valid for the key.
    InvalidValue {
        parameter_name: &'static str,
        given_value: TomlValue,
        ordinance: Box<dyn fmt::Display>,
    },

    /// One argument has no effect because another in specified.
    Conflict {
        parameter_name: &'static str,
        other_parameter_name: &'static str,
        specific_value: Option<TomlValue>,
    },

    /// One parameter is aliased to another, but both are present.
    AliasClash {
        parameter_name: &'static str,
        other_parameter_name: &'static str,
    },
}

impl ReadError {
    pub fn invalid(parameter_name: &'static str, given_value: TomlValue, ordinance: impl fmt::Display + Sized + 'static) -> Self {
        Self::InvalidValue { parameter_name, given_value, ordinance: Box::new(ordinance) }
    }

    pub fn conflict(parameter_name: &'static str, other_parameter_name: &'static str) -> Self {
        Self::Conflict { parameter_name, other_parameter_name, specific_value: None }
    }

    pub fn conflict2(parameter_name: &'static str, other_parameter_name: &'static str, specific: TomlValue) -> Self {
        Self::Conflict { parameter_name, other_parameter_name, specific_value: Some(specific) }
    }
}

impl fmt::Debug for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingParameter { parameter_name } => {
                write!(f, "Parameter ‘{}’ is missing", parameter_name)
            }
            Self::UnknownParameter { parameter_name } => {
                write!(f, "Parameter ‘{}’ is unknown", parameter_name)
            }
            Self::InvalidValue { parameter_name, given_value, ordinance} => {
                write!(f, "Parameter ‘{}’ value ‘{}’ is invalid ({})", parameter_name, given_value, (ordinance))
            }
            Self::Conflict { parameter_name, other_parameter_name, specific_value: None } => {
                write!(f, "Parameter ‘{}’ is inappropriate when parameter ‘{}’ is given", parameter_name, other_parameter_name)
            }
            Self::Conflict { parameter_name, other_parameter_name, specific_value: Some(ov) } => {
                write!(f, "Parameter ‘{}’ is inappropriate when parameter ‘{}’ is ‘{}’", parameter_name, other_parameter_name, ov)
            }
            Self::AliasClash { parameter_name, other_parameter_name } => {
                write!(f, "Parameters ‘{}’ and ‘{}’ are both given (they are aliases)", parameter_name, other_parameter_name)
            }
        }
    }
}



#[derive(PartialEq, Debug, Copy, Clone)]
pub struct OneOf(pub &'static [&'static str]);

impl fmt::Display for OneOf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() == 2 {
            write!(f, "it must be ‘{}’ or ‘{}’", self.0[0], self.0[1])
        }
        else if self.0.len() == 3 {
            write!(f, "it must be ‘{}’ or ‘{}’ or ‘{}’", self.0[0], self.0[1], self.0[2])
        }
        else {
            panic!("OneOf")
        }
    }
}



#[derive(PartialEq, Debug)]
pub enum Rewrite {

    Path(PathBuf, PathBuf),

    Interface(String, String),

    Url(String, String),
}

#[derive(PartialEq, Debug, Default)]
pub struct Rewrites {
    rules: Vec<Rewrite>,
    expand_home: bool,
}

impl Rewrites {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expand_tildes(&mut self) {
        self.expand_home = true;
    }

    pub fn add(&mut self, rule: Rewrite) {
        self.rules.push(rule);
    }

    pub fn path(&self, path: String) -> PathBuf {
        let pb = if self.expand_home { PathBuf::from(expand_tilde(&path).as_ref()) }
                                else { PathBuf::from(path) };

        for rule in &self.rules {
            if let Rewrite::Path(from, to) = rule {
                if let Ok(rest) = pb.strip_prefix(from) {
                    let mut new_path = to.clone();
                    new_path.push(rest);

                    trace!("Rewriting path {:?} -> {:?}", pb, new_path);
                    return new_path;
                }
            }
        }

        pb
    }

    pub fn interface(&self, string: String) -> String {
        for rule in &self.rules {
            if let Rewrite::Interface(from, to) = rule {
                if from == &string {
                    trace!("Rewriting interface {:?} -> {:?}", string, to);
                    return to.to_string();
                }
            }
        }

        string
    }

    pub fn url(&self, url: String) -> String {
        for rule in &self.rules {
            if let Rewrite::Url(from, to) = rule {
                if url.starts_with(from) {
                    let new_url = to.clone() + &url[from.len() ..].to_string();
                    trace!("Rewriting URL {:?} -> {:?}", url, new_url);
                    return new_url;
                }
            }
        }

        url
    }
}
