//! The Hash check involves searching through the list of installed gems to see
//! if one is installed.
//!
//! # Check example
//!
//! ```toml
//! [[hash]]
//! path = "my-file.dat"
//! algorithm = "sha512sum"
//! hash = "d78abb0542736865f9470..."
//! ```
//!
//! # Commands
//!
//! This check works by running one of the checksum commands, such as `md5sum`
//! or `sha256sum`, depending on the input algorithm.


use std::fmt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use log::*;

use spec_exec::{Executor, ExecError};

use crate::check::{Check, RunCheck, CheckResult, PassResult, FailResult};
use crate::read::{TomlValue, ValueExtras, ReadError, Rewrites};


/// The Hash check computes the hash of a file, and checks that the resulting
/// hash matches an expected hash.
#[derive(PartialEq, Debug)]
pub struct HashCheck {
    input_path: PathBuf,
    algorithm: Algorithm,
    expected_hash: String,
}

/// Which hashing algorithm to use.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
pub enum Algorithm {
    MD5,
    SHA1,
    SHA224,
    SHA256,
    SHA384,
    SHA512,
}


// ---- the check description ----

impl fmt::Display for HashCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { input_path, algorithm, expected_hash } = &self;

        write!(f, "File ‘{}’ has {:?} hash ‘{}’", input_path.display(), algorithm, expected_hash)
    }
}


// ---- reading from TOML ----

impl Check for HashCheck {
    const TYPE: &'static str = "hash";
}

impl HashCheck {
    pub fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["path", "algorithm", "hash"])?;

        let input_value = table.get_or_read_error("path")?;
        let input_path = input_value.string_or_error("path")?;

        if input_path.is_empty() {
            return Err(ReadError::invalid("path", input_value.clone(), "it must not be empty"));
        }

        let algorithm = Algorithm::read(table)?;
        let expected_hash = table.get_or_read_error("hash")?.string_or_error("hash")?;
        Ok(Self { input_path: rewrites.path(input_path), algorithm, expected_hash })
    }
}

impl Algorithm {
    fn read(table: &TomlValue) -> Result<Self, ReadError> {
        let algo_value = table.get_or_read_error("algorithm")?;

        match &algo_value.string_or_error("algorithm")?.to_ascii_lowercase()[..] {
            "md5"    => Ok(Self::MD5),
            "sha1"   => Ok(Self::SHA1),
            "sha224" => Ok(Self::SHA224),
            "sha256" => Ok(Self::SHA256),
            "sha384" => Ok(Self::SHA384),
            "sha512" => Ok(Self::SHA512),
            _        => Err(ReadError::invalid("algorithm", algo_value.clone(), "it must be an algorithm such as ‘MD5’, ‘SHA256’...")),
        }
    }
}


// ---- running the check ----

pub trait RunHash {

    #[allow(unused)]
    fn prime(&mut self, path: &Path, algorithm: Algorithm) { }

    fn hash_file(&self, executor: &mut Executor, path: PathBuf, algorithm: Algorithm) -> Result<String, Rc<ExecError>>;
}

impl<H: RunHash> RunCheck<H> for HashCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, hash: &mut H) {
        hash.prime(&self.input_path, self.algorithm);
    }

    fn check(&self, executor: &mut Executor, hash: &H) -> Vec<CheckResult<Pass, Fail>> {
        info!("Running check");

        let result_hash = match hash.hash_file(executor, self.input_path.clone(), self.algorithm) {
            Ok(p)   => p,
            Err(e)  => return vec![ CheckResult::CommandError(e) ],
        };

        if self.expected_hash == result_hash {
            vec![ CheckResult::Passed(Pass::HashesMatch) ]
        }
        else {
            vec![ CheckResult::Failed(Fail::HashMismatch) ]
        }
    }
}

/// The successful result of a Hash check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The output hash matches the input hash.
    HashesMatch,
}

/// The failure result of running a Hash check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Fail {

    /// The input file being hashed does not exist.
    MissingFile,

    /// The output and input hashes do not match.
    HashMismatch,
}

impl PassResult for Pass {}

impl FailResult for Fail {}


// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HashesMatch => {
                write!(f, "hashes match")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFile => {
                write!(f, "it is not installed")
            }
            Self::HashMismatch => {
                write!(f, "hash mismatch")
            }
        }
    }
}
