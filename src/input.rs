use std::fs::File;
use std::fmt;
use std::io::{self, Read};
use std::iter;
use std::path::PathBuf;

use derive_more::{From, Display};
use log::*;
use serde::Serialize;

use spec_checks::load::{parse_toml, CheckDocument, TomlError};


/// Where the input TOML comes from. This produces an iterator that yields
/// [`InputSource`] values.
#[derive(PartialEq, Debug)]
pub enum Inputs {

    /// The command-line options say to read from standard input.
    Stdin,

    /// The command-line options say to read from the files at the given
    /// paths. These must be files, not directories.
    Files(Vec<PathBuf>),
}

impl IntoIterator for Inputs {
    type Item = InputSource;

    type IntoIter = Box<dyn Iterator<Item=Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Stdin        => Box::new(iter::once(InputSource::Stdin)),
            Self::Files(files) => Box::new(files.into_iter().map(InputSource::File)),
        }
    }
}

/// The type iterated by an [`Inputs`] iterator.
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "source", content = "path")]
pub enum InputSource {

    /// Read from standard input.
    Stdin,

    /// Read from the file at this path.
    File(PathBuf),
}

impl fmt::Display for InputSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdin       => write!(f, "<stdin>"),
            Self::File(path)  => write!(f, "{}", path.display()),
        }
    }
}

impl InputSource {

    pub fn is_stdin(&self) -> bool {
        matches!(self, Self::Stdin)
    }

    pub fn load(&self) -> Result<CheckDocument, LoadError> {
        let contents = self.read_to_string()?;
        let document = parse_toml(&contents)?;
        Ok(document)
    }

    /// Reads the entirety of the relevant input stream, returing an IO error
    /// if it fails that gets shown to the user.
    fn read_to_string(&self) -> io::Result<String> {
        match self {
            Self::Stdin => {
                info!("Reading checks from standard input");
                let stdin = io::stdin();
                let mut handle = stdin.lock();

                let mut contents = String::new();
                handle.read_to_string(&mut contents)?;
                trace!("Successfully read stdin");
                Ok(contents)
            }

            Self::File(path) => {
                info!("Reading checks from file {:?}", path);
                let mut file = File::open(path)?;

                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                trace!("Successfully read file contents");
                Ok(contents)
            }
        }
    }
}


/// Something that can go wrong while reading a file into a list of checks.
#[derive(From, Display)]
pub enum LoadError {

    /// An I/O error prevented a check document from being opened or read.
    Io(io::Error),

    /// A check document file was able to be read, but the TOML it contains
    /// has invalid syntax.
    Toml(TomlError),
}
