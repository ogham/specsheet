use std::fmt;
use std::fs::read;
use std::io;
use std::path::PathBuf;

//use log::*;
use regex::{Error as RegexError, bytes::Regex};

use crate::CheckResult;
use crate::read::{TomlValue, ValueExtras, ReadError};


/// A **contents matcher** asserts properties of a string, which has
/// been obtained from somewhere (such as a file’s contents, a command’s
/// output, or a web page’s body).
#[derive(PartialEq, Debug)]
pub enum ContentsMatcher {

    /// The output should contain a line matching the given regex.
    LineRegex(String, bool),

    /// The output should contain the given string.
    StringMatch(String, bool),

    /// The output should match the file at the given path.
    FileMatch(PathBuf),

    /// The output should be empty.
    ShouldBeEmpty,

    /// The output should be non-empty.
    ShouldBeNonEmpty,
}

impl ContentsMatcher {

    /// Writes a short description of this matcher to the formatter.
    /// This is used when writing out check descriptions.
    pub fn describe(&self, f: &mut fmt::Formatter<'_>, noun: &'static str) -> fmt::Result {
        match self {
            Self::LineRegex(regex, true)      => write!(f, " {} matching regex ‘/{}/’", noun, regex),
            Self::LineRegex(regex, false)     => write!(f, " {} not matching regex ‘/{}/’", noun, regex),
            Self::StringMatch(string, true)   => write!(f, " {} containing ‘{}’", noun, string),
            Self::StringMatch(string, false)  => write!(f, " {} not containing ‘{}’", noun, string),
            Self::FileMatch(path)             => write!(f, " {} matching file ‘{}’", noun, path.display()),
            Self::ShouldBeEmpty               => write!(f, " empty {}", noun),
            Self::ShouldBeNonEmpty            => write!(f, " non-empty {}", noun),
        }
    }
}


// ---- reading ----

impl ContentsMatcher {
    pub fn read(parameter_name: &'static str, table: &TomlValue) -> Result<Self, ReadError> {
        table.ensure_table(parameter_name)?;
        table.ensure_only_keys(&["regex", "string", "file", "empty", "matches"])?;

        let matches = table.get("matches")
                           .map(|m| m.boolean_or_error("matches")).transpose()?
                           .unwrap_or(true);

        if let Some(regex_value) = table.get("regex") {
            let regex = regex_value.string_or_error("regex")?;
            if regex.is_empty() {
                return Err(ReadError::invalid(parameter_name, regex_value.clone(), ContentsReadError::EmptyRegex));
            }
            else {
                return Ok(Self::LineRegex(regex, matches));
            }
        }

        if let Some(string_value) = table.get("string") {
            let string = string_value.string_or_error("string")?;
            if string.is_empty() {
                return Err(ReadError::invalid(parameter_name, string_value.clone(), ContentsReadError::EmptyString));
            }
            else {
                return Ok(Self::StringMatch(string, matches));
            }
        }

        if let Some(file_value) = table.get("file") {
            let path = file_value.string_or_error("file")?;
            if table.get("matches").is_some() {
                panic!("Can't use matches with file");
            }
            return Ok(Self::FileMatch(PathBuf::from(path)));
        }

        if let Some(empty_value) = table.get("empty") {
            if table.get("matches").is_some() {
                panic!("Can't use matches with empty");
            }

            if empty_value == &TomlValue::Boolean(true) {
                return Ok(Self::ShouldBeEmpty);
            }
            else if empty_value == &TomlValue::Boolean(false) {
                return Ok(Self::ShouldBeNonEmpty);
            }
            else {
                panic!("booleans??");
            }
        }

        Err(ReadError::invalid(parameter_name, table.clone(), ContentsReadError::NoConditions))
    }
}

/// Something that can go wrong while reading a `ContentsMatcher`.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ContentsReadError {

    /// No conditions were actually specified.
    NoConditions,

    /// The input regex was empty.
    EmptyRegex,

    /// The input string to match on was empty.
    EmptyString,
}

impl fmt::Display for ContentsReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoConditions => {
                write!(f, "No conditions")
            }
            Self::EmptyRegex => {
                write!(f, "Empty regex")
            }
            Self::EmptyString => {
                write!(f, "Empty string")
            }
        }
    }
}


// ---- running the check ----

impl ContentsMatcher {
    pub fn check(&self, contents: &[u8]) -> CheckResult<Pass, Fail> {

        // regex check
        if let Self::LineRegex(regex_str, matches) = &self {
            let mut re = regex_str.clone();
            re.insert_str(0, "(?m)");
            match Regex::new(&re) {
                Ok(re) => {
                    if *matches {
                        if re.is_match(contents) {
                            return CheckResult::Passed(Pass::OutputMatchesRegex);
                        }
                        else {
                            let output_string = String::from_utf8_lossy(contents).into();
                            return CheckResult::Failed(Fail::OutputRegexMismatch(output_string));
                        }
                    }
                    else {
                        if re.is_match(contents) {
                            let output_string = String::from_utf8_lossy(contents).into();
                            return CheckResult::Failed(Fail::OutputMatchesRegex(output_string));
                        }
                        else {
                            return CheckResult::Passed(Pass::OutputRegexMismatch);
                        }
                    }
                }
                Err(e) => {
                    return CheckResult::Failed(Fail::InvalidRegex(e));
                }
            }
        }

        // string check
        if let Self::StringMatch(search_string, matches) = &self {
            let result = bytes_contains(contents, search_string.as_bytes());

            if *matches {
                if result {
                    return CheckResult::Passed(Pass::OutputMatchesString);
                }
                else {
                    let output_string = String::from_utf8_lossy(contents).into();
                    return CheckResult::Failed(Fail::OutputStringMismatch(output_string));
                }
            }
            else {
                if result {
                    let output_string = String::from_utf8_lossy(contents).into();
                    return CheckResult::Failed(Fail::OutputMatchesString(output_string));
                }
                else {
                    return CheckResult::Passed(Pass::OutputStringMismatch);
                }
            }
        }

        // file check
        if let Self::FileMatch(contents_file) = &self {
            match read(contents_file) {
                Ok(read_contents) => {
                    if read_contents == contents {
                        return CheckResult::Passed(Pass::OutputMatchesFile);
                    }
                    else {
                        let expected_string = String::from_utf8_lossy(&read_contents).into();
                        let output_string = String::from_utf8_lossy(contents).into();
                        return CheckResult::Failed(Fail::OutputFileMismatch(expected_string, output_string));
                    }
                }
                Err(e) => {
                    return CheckResult::Failed(Fail::IoReadingOutputFile(contents_file.clone(), e));
                }
            }
        }

        // blank check
        if let Self::ShouldBeEmpty = &self {
            if contents.is_empty() {
                return CheckResult::Passed(Pass::OutputEmpty);
            }
            else {
                let output_string = String::from_utf8_lossy(contents).into();
                return CheckResult::Failed(Fail::OutputNotEmpty(output_string));
            }
        }
        else if let Self::ShouldBeNonEmpty = &self {
            if ! contents.is_empty() {
                return CheckResult::Passed(Pass::OutputNonEmpty);
            }
            else {
                return CheckResult::Failed(Fail::OutputEmpty);
            }
        }

        unreachable!()
    }
}

fn bytes_contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len())
            .any(|e| e == needle)
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The contents matched the input regular expression.
    OutputMatchesRegex,

    /// The contents did not match the input regular expression.
    OutputRegexMismatch,

    /// The contents contains the input string.
    OutputMatchesString,

    /// The contents do not contain the input string.
    OutputStringMismatch,

    /// The contents is the same as a file.
    OutputMatchesFile,

    /// The contents was empty.
    OutputEmpty,

    /// The contents was non-empty.
    OutputNonEmpty,
}

#[derive(Debug)]
pub enum Fail {

    InvalidRegex(RegexError),  // this can’t be a read error because it’s not cloneable or something

    /// The contents did _not_ match the input regular expression, when
    /// it was supposed to.
    OutputRegexMismatch(String),

    /// The contents _did_ match the input regular expression, when it
    /// was supposed to.
    OutputMatchesRegex(String),

    /// The contents did _not_ contain an input string, when it was
    /// supposed to.
    OutputStringMismatch(String),

    /// The contents _did_ contain an input string, when it was not
    /// supposed to.
    OutputMatchesString(String),

    /// The contents differs from a file.
    OutputFileMismatch(String, String),

    /// An IO error occurred while reading a file to compare
    /// the contents with.
    IoReadingOutputFile(PathBuf, io::Error),

    /// The contents should have been empty, but isn’t.
    OutputNotEmpty(String),

    /// The contents should have been non-empty, but was empty.
    OutputEmpty,
}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutputMatchesRegex => {
                write!(f, "matches regex")
            }
            Self::OutputRegexMismatch => {
                write!(f, "does not match regex")
            }
            Self::OutputMatchesString => {
                write!(f, "matches string")
            }
            Self::OutputStringMismatch => {
                write!(f, "does not match string")
            }
            Self::OutputMatchesFile => {
                write!(f, "matches file")
            }
            Self::OutputEmpty => {
                write!(f, "is empty")
            }
            Self::OutputNonEmpty => {
                write!(f, "is non-empty")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRegex(regex_error) => {
                write!(f, "invalid regex: ‘{}’", regex_error)
            }
            Self::OutputRegexMismatch(_) => {
                write!(f, "did not match the regex")
            }
            Self::OutputMatchesRegex(_) => {
                write!(f, "matched the regex")
            }
            Self::OutputStringMismatch(_) => {
                write!(f, "did not match the string")
            }
            Self::OutputMatchesString(_) => {
                write!(f, "matched the string")
            }
            Self::OutputFileMismatch(_, _) => {
                write!(f, "did not match the file")
            }
            Self::IoReadingOutputFile(path, ioe) => {
                write!(f, "IO error reading file {}: {}", path.display(), ioe)
            }
            Self::OutputNotEmpty(_) => {
                write!(f, "was not empty")
            }
            Self::OutputEmpty => {
                write!(f, "was empty")
            }
        }
    }
}

// Similar to FailResult
impl Fail {
    pub fn command_output(&self, title: &'static str) -> Option<(String, &String)> {
        match self {
            Self::OutputRegexMismatch(got)  |
            Self::OutputStringMismatch(got) |
            Self::OutputNotEmpty(got)       => Some((title.into(), got)),
            _                                  => None,
        }
    }

    pub fn diff_output(&self) -> Option<(String, &String, &String)> {
        match self {
            Self::OutputFileMismatch(expected, got) => {
                Some(("Difference between expected and got:".into(), expected, got))
            }
            _ => {
                None
            }
        }
    }
}
