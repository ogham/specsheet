//! A collection of hashing commands.
//!
//! # Sample output
//!
//! ```text
//! $ md5sum wibble
//! d41d8cd98f00b204e9800998ecf8427e  wibble
//! ```


use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use log::*;
use once_cell::sync::Lazy;
use regex::Regex;

use spec_checks::hashes::{RunHash, Algorithm};
use spec_exec::{Command, Exec, Executor, ExecError, CommandOutput, ExitReason};

use super::GlobalOptions;


/// The **hash command** that runs one of many hashing binaries.
#[derive(Debug, Default)]
pub struct HashCommand {
    results: BTreeMap<(PathBuf, Algorithm), Exec<HashOutput>>,
    // this is a BTreeMap but it’s also a hash map. get it?
}


impl HashCommand {

    /// Creates a new command to run the hashing programs.
    pub fn create(_global_options: &impl GlobalOptions) -> Self {
        Self::default()
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        self.results.into_iter().flat_map(|t| t.1.into_command())
    }
}

impl RunHash for HashCommand {
    fn prime(&mut self, path: &Path, algorithm: Algorithm) {
        if ! self.results.contains_key(&(path.to_path_buf(), algorithm)) {  // todo: stop allocating
            debug!("Priming {:?} hash command for {:?}", algorithm, path);
            let exec = Exec::actual(hash_file_cmd(path, algorithm));
            self.results.insert((path.into(), algorithm), exec);
        }
    }

    fn hash_file(&self, executor: &mut Executor, path: PathBuf, algorithm: Algorithm) -> Result<String, Rc<ExecError>> {
        debug!("Calculating {:?} hash for file -> {:?}", algorithm, path);
        let output = self.results[&(path, algorithm)].run(executor)?;
        Ok(output.get_hash())
    }
}

fn hash_file_cmd(path: &Path, algorithm: Algorithm) -> Command {
    let command_name = match algorithm {
        Algorithm::MD5     => "md5sum",
        Algorithm::SHA1    => "sha1sum",
        Algorithm::SHA224  => "sha224sum",
        Algorithm::SHA256  => "sha256sum",
        Algorithm::SHA384  => "sha384sum",
        Algorithm::SHA512  => "sha512sum",
    };

    let mut cmd = Command::new(command_name);
    cmd.arg(path);
    cmd
}


/// The **hash output** encapsulates the output lines of an
/// invoked `HashCommand`.
#[derive(Debug)]
pub struct HashOutput {
    lines: Vec<Rc<str>>,
}

impl CommandOutput for HashOutput {
    fn interpret_command_output(lines: Vec<Rc<str>>, exit_reason: ExitReason) -> Result<Self, ExecError> {
        exit_reason.should_be(0)?;
        Ok(Self { lines })
    }
}

impl HashOutput {

    /// Extracts the hash from the first line of the output.
    fn get_hash(&self) -> String {
        let first_line = &**self.lines.first().unwrap();
        let caps = REGEX.captures(first_line).unwrap();
        caps.get(1).unwrap().as_str().into()
    }
}


/// Regular expression for a hexadecimal hash at the start of a line.
static REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r##"(?x)
        ^
        ([0-9 a-f]{10,})
        \s
    "##).unwrap()
});
