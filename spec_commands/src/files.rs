//! Files and directories
//!
//! This does not actually run any external programs yet!
//! It is just a placeholder.

use std::collections::BTreeMap;
use std::fs::{Metadata, read as read_file};
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::*;

use spec_checks::fs::LookupFile;
use spec_exec::Command;

use super::GlobalOptions;


/// The **filesystem non-command** examines the local filesystem and caches
/// the results.
#[derive(Debug)]
pub struct FilesystemNonCommand {
    exists:   BTreeMap<PathBuf,         Mutex<Option<bool>>>,
    files:    BTreeMap<(PathBuf, bool), Mutex<Option<Metadata>>>,
    contents: BTreeMap<PathBuf,         Mutex<Option<Vec<u8>>>>,
    targets:  BTreeMap<PathBuf,         Mutex<Option<PathBuf>>>,
}

impl FilesystemNonCommand {

    /// Creates a new non-command.
    pub fn create(_global_options: &impl GlobalOptions) -> Self {
        Self {
            exists: BTreeMap::new(),
            files: BTreeMap::new(),
            contents: BTreeMap::new(),
            targets: BTreeMap::new(),
        }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        std::iter::empty()
    }
}

impl LookupFile for FilesystemNonCommand {
    fn prime(&mut self, path: &Path, follow: bool) {
        if ! self.files.contains_key(&(path.into(), follow)) {
            debug!("Priming filesystem with path {:?}", path);
            self.exists.insert(path.to_path_buf(), Mutex::new(None));
            self.files.insert((path.to_path_buf(), follow), Mutex::new(None));
            self.contents.insert(path.to_path_buf(), Mutex::new(None));
            self.targets.insert(path.to_path_buf(), Mutex::new(None));
        }
    }

    fn does_file_exist(&self, path: &Path) -> bool {
        let mut slot = self.exists.get(path).unwrap().lock().unwrap();
        let target = slot.get_or_insert_with(|| path.exists());
        *target
    }

    fn lookup_file(&self, path: &Path, follow: bool) -> Metadata {
        let mut slot = self.files.get(&(path.to_path_buf(), follow)).unwrap().lock().unwrap();
        let metadata = slot.get_or_insert_with(|| {
            if follow { path.metadata() }
                 else { path.symlink_metadata() }.unwrap()
        });
        metadata.clone()
    }

    fn read_file_contents(&self, path: &Path) -> Vec<u8> {
        let mut slot = self.contents.get(path).unwrap().lock().unwrap();
        let target = slot.get_or_insert_with(|| read_file(path).unwrap());
        target.clone()
    }

    fn lookup_link_target(&self, path: &Path) -> Result<PathBuf, IoError> {
        let mut slot = self.targets.get(path).unwrap().lock().unwrap();
        let target = slot.get_or_insert_with(|| path.read_link().unwrap());
        Ok(target.clone())
    }
}
