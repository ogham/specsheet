//! Filesystem checks
//!
//! # Check example
//!
//! ```toml
//! [[fs]]
//! path = '/opt/consul/etc/consul.hcl'
//! kind = 'file'
//! ```
//!
//! # Commands
//!
//! No commands are run by filesystem checks; Specsheet queries the filesystem
//! itself.

use std::convert::TryInto;
use std::ffi::OsString;
use std::fs::{FileType, Metadata};
use std::io::Error as IoError;
use std::fmt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use log::*;

use spec_analysis::DataPoint;

use crate::check::{Check, BuiltInCheck, CheckResult, PassResult, FailResult};
use crate::contents::{self, ContentsMatcher};
use crate::read::{TomlValue, ValueExtras, ReadError, OneOf, Rewrites};


/// A check against the local filesystem.
#[derive(PartialEq, Debug)]
pub struct FilesystemCheck {
    input_path: PathBuf,
    condition: Condition,
    follow: bool,
}

#[derive(PartialEq, Debug)]
enum Condition {

    /// A file with the given path should exist, with these extra checks.
    Exists(MetadataChecks),

    /// No file with the given path should exist.
    Missing,
}

#[derive(PartialEq, Debug)]
struct MetadataChecks {
    kind: Option<FileKindCheck>,
    permissions: Option<ModeCheck>,
    owner: Option<OwnerCheck>,
    group: Option<GroupCheck>,
}

#[derive(PartialEq, Debug)]
enum ModeCheck {
    Executable,
    Octal(String),
}

#[derive(PartialEq, Debug)]
enum FileKindCheck {

    /// The file entry at this path should be a regular file.
    File {
        explicit_check: bool,
        contents: Option<ContentsMatcher>,
    },

    /// The file entry at this path should be a directory.
    Directory,

    /// The file entry at this path should be a symbolic link.
    Link {

        /// If specified, the target of the link.
        target: Option<PathBuf>,
    },
}

#[derive(PartialEq, Debug)]
enum OwnerCheck {
    ByName(String),
    ByID(u32),
}

#[derive(PartialEq, Debug)]
enum GroupCheck {
    ByName(String),
    ByID(u32),
}


// ---- the check description ----

impl Check for FilesystemCheck {
    const TYPE: &'static str = "fs";
}

impl fmt::Display for FilesystemCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { input_path, condition, follow } = &self;

        write!(f, "File ‘{}’", input_path.display())?;

        if let Condition::Exists(checks) = &condition {
            match &checks.kind {
                Some(FileKindCheck::File { explicit_check, contents }) => {

                    if *explicit_check {
                        write!(f, " is a regular file")?;

                        if contents.is_some() {
                            write!(f, " that")?;
                        }
                    }

                    // The language here is _slightly_ more natural than the English
                    // written by `ContentsMatcher::describe`.
                    match contents {
                        Some(ContentsMatcher::LineRegex(regex, true))      => write!(f, " matches regex ‘/{}/’", regex)?,
                        Some(ContentsMatcher::LineRegex(regex, false))     => write!(f, " does not match regex ‘/{}/’", regex)?,
                        Some(ContentsMatcher::StringMatch(string, true))   => write!(f, " contains string ‘{}’", string)?,
                        Some(ContentsMatcher::StringMatch(string, false))  => write!(f, " does not contain string ‘{}’", string)?,
                        Some(ContentsMatcher::FileMatch(path))             => write!(f, " has the contents of file ‘{}’", path.display())?,
                        Some(ContentsMatcher::ShouldBeEmpty)               => write!(f, " is empty")?,
                        Some(ContentsMatcher::ShouldBeNonEmpty)            => write!(f, " is not empty")?,
                        None                                               => {/* nothing to match */},
                    }
                }
                Some(FileKindCheck::Directory)                 => write!(f, " is a directory")?,
                Some(FileKindCheck::Link { target: None })     => write!(f, " is a symbolic link")?,
                Some(FileKindCheck::Link { target: Some(t) })  => write!(f, " is a symbolic link to ‘{}’", t.display())?,
                None                                           => {/* do nothing */},
            }

            if let Some(owner) = &checks.owner {
                if checks.kind.is_some() { write!(f, " and")?; }
                if checks.kind.is_none() { write!(f, " has")?; }

                match owner {
                    OwnerCheck::ByID(uid)  =>  write!(f, " owner ID ‘{}’", uid)?,
                    OwnerCheck::ByName(un) =>  write!(f, " owner ‘{}’", un)?,
                }
            }

            if let Some(group) = &checks.group {
                if checks.owner.is_some() { write!(f, " and")?; }
                if checks.owner.is_none() { write!(f, " has")?; }

                match group {
                    GroupCheck::ByID(gid)  =>  write!(f, " group ID ‘{}’", gid)?,
                    GroupCheck::ByName(gn) =>  write!(f, " group ‘{}’", gn)?,
                }
            }

            if let Some(permissions) = &checks.permissions {
                if checks.kind.is_some() || checks.group.is_some() || checks.owner.is_some() { write!(f, " and")?; }

                match permissions {
                    ModeCheck::Executable  =>  write!(f, " is executable")?,
                    ModeCheck::Octal(mode) =>  write!(f, " has permissions ‘{}’", mode)?,
                }
            }

            if ! (checks.kind.is_some() || checks.group.is_some() || checks.owner.is_some() || checks.permissions.is_some()) {
                write!(f, " exists")?;
            }

            if *follow {
                write!(f, " (following symlinks)")?;
            }

            Ok(())
        }
        else {
            write!(f, " does not exist")
        }
    }
}


// ---- reading from TOML ----

impl FilesystemCheck {
    pub fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        table.ensure_only_keys(&["path", "kind", "state", "permissions", "mode",
                                 "owner", "group", "link_target", "contents", "follow"])?;

        let input_value = table.get_or_read_error("path")?;
        let input_path = input_value.string_or_error("path")?;

        if input_path.is_empty() {
            return Err(ReadError::invalid("path", input_value.clone(), "it must not be empty"));
        }

        let condition = Condition::read(table, rewrites)?;
        let follow = table.get("follow").map(|b| b.boolean_or_error("follow")).transpose()?.unwrap_or_default();

        Ok(Self { input_path: rewrites.path(input_path), condition, follow })
    }
}

impl Condition {
    fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        let metadata = MetadataChecks::read(table, rewrites)?;

        let state_value = match table.get("state") {
            Some(s) => s,
            None    => return Ok(Self::Exists(metadata)),
        };

        match &state_value.string_or_error2("state", OneOf(&["present", "missing"]))?[..] {
            "present" => {
                Ok(Self::Exists(metadata))
            }
            "absent" | "missing" => {
                if table.get("kind").is_some() {
                    return Err(ReadError::conflict2("kind", "state", state_value.clone()));
                }
                if table.get("link_target").is_some() {
                    return Err(ReadError::conflict2("link_target", "state", state_value.clone()));
                }
                if table.get("contents").is_some() {
                    return Err(ReadError::conflict2("contents", "state", state_value.clone()));
                }
                Ok(Self::Missing)
            }
            _ => {
                Err(ReadError::invalid("state", state_value.clone(), OneOf(&["present", "missing"])))
            }
        }
    }
}

impl MetadataChecks {
    fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Self, ReadError> {
        Ok(Self {
            kind:        FileKindCheck::read(table, rewrites)?,
            permissions: ModeCheck::read(table)?,
            owner:       OwnerCheck::read(table)?,
            group:       GroupCheck::read(table)?,
        })
    }
}

impl OwnerCheck {
    fn read(table: &TomlValue) -> Result<Option<Self>, ReadError> {
        if let Some(owner_value) = table.get("owner") {
            if let Some(owner) = owner_value.as_str() {
                if owner.is_empty() {
                    Err(ReadError::invalid("owner", owner_value.clone(), "it must not be empty"))
                }
                else {
                    Ok(Some(Self::ByName(owner.into())))
                }
            }
            else if let Some(int) = owner_value.as_integer() {
                Ok(Some(Self::ByID(int.try_into().expect("number out of range"))))
            }
            else {
                Err(ReadError::invalid("owner", owner_value.clone(), "it must be a string or a number"))
            }
        }
        else {
            Ok(None)
        }
    }
}

impl GroupCheck {
    fn read(table: &TomlValue) -> Result<Option<Self>, ReadError> {
        if let Some(group_value) = table.get("group") {
            if let Some(group) = group_value.as_str() {
                if group.is_empty() {
                    Err(ReadError::invalid("group", group_value.clone(), "it must not be empty"))
                }
                else {
                    Ok(Some(Self::ByName(group.into())))
                }
            }
            else if let Some(int) = group_value.as_integer() {
                Ok(Some(Self::ByID(int.try_into().expect("number out of range"))))
            }
            else {
                Err(ReadError::invalid("group", group_value.clone(), "it must be a string or a number"))
            }
        }
        else {
            Ok(None)
        }
    }
}

impl ModeCheck {
    fn read(table: &TomlValue) -> Result<Option<Self>, ReadError> {
        use regex::Regex;

        let permissions = table.get("permissions");
        let mode = table.get("mode");

        if permissions.is_some() && mode.is_some() {
            return Err(ReadError::AliasClash { parameter_name: "permissions", other_parameter_name: "mode" });
        }

        if let Some(mode_value) = permissions.as_ref().or_else(|| mode.as_ref()) {
            let parameter_name = if permissions.is_some() { "permissions" } else { "mode" };

            let mode = mode_value.string_or_error(parameter_name)?;

            if mode == "+x" {
                return Ok(Some(Self::Executable));
            }

            let regex = Regex::new(r##"(?x)
                ([0-7]{3,4})
            "##).unwrap();

            if let Some(matches) = regex.captures(&mode) {
                if let Some(octal) = matches.get(1) {
                    return Ok(Some(Self::Octal(octal.as_str().into())));
                }
            }

            Err(ReadError::invalid(parameter_name, (*mode_value).clone(), "it must be a permissions string"))
        }
        else {
            Ok(None)
        }
    }
}

impl FileKindCheck {
    fn read(table: &TomlValue, rewrites: &Rewrites) -> Result<Option<Self>, ReadError> {
        if let Some(kind_value) = table.get("kind") {
            let kind = kind_value.string_or_error2("kind", OneOf(&["file", "directory", "symlink"]))?;

            match &*kind {
                "file" => {
                    if table.get("link_target").is_some() {
                        return Err(ReadError::conflict2("link_target", "kind", kind_value.clone()));
                    }
                    let contents = table.get("contents").map(|e| ContentsMatcher::read("contents", e)).transpose()?;
                    Ok(Some(Self::File { explicit_check: true, contents }))
                }
                "dir" | "directory" => {
                    if table.get("contents").is_some() {
                        return Err(ReadError::conflict2("contents", "kind", kind_value.clone()));
                    }
                    if table.get("link_target").is_some() {
                        return Err(ReadError::conflict2("link_target", "kind", kind_value.clone()));
                    }
                    Ok(Some(Self::Directory))
                }
                "link" | "symlink" => {
                    let target = table.get("link_target").map(|e| e.string_or_error("link_target")).transpose()?;
                    if target.as_ref().map_or(false, String::is_empty) {
                        return Err(ReadError::invalid("link_target", table.get("link_target").unwrap().clone(), "it must not be empty"));
                    }
                    if table.get("contents").is_some() {
                        return Err(ReadError::conflict2("contents", "kind", kind_value.clone()));
                    }

                    Ok(Some(Self::Link { target: target.map(|e| rewrites.path(e)) }))
                }
                _  => {
                    Err(ReadError::invalid("kind", kind_value.clone(), OneOf(&["file", "directory", "symlink"])))
                }
            }
        }
        else if let Some(link_target) = table.get("link_target") {
            let target = link_target.string_or_error("link_target")?;
            if target.is_empty() {
                return Err(ReadError::invalid("link_target", link_target.clone(), "it must not be empty"));
            }

            if table.get("contents").is_some() {
                return Err(ReadError::conflict("contents", "link_target"));
            }

            Ok(Some(Self::Link { target: Some(rewrites.path(target)) }))
        }
        else if let Some(re) = table.get("contents") {
            let contents = Some(re).map(|re| ContentsMatcher::read("contents", re)).transpose()?;
            Ok(Some(Self::File { explicit_check: false, contents }))
        }
        else {
            Ok(None)
        }
    }
}


// ---- analysis properties ----

impl FilesystemCheck {
    pub fn properties<'a>(&'a self) -> Vec<DataPoint<'a>> {
        let mut points = Vec::new();

        points.push(DataPoint::InvolvesPath(&self.input_path));

        if let Condition::Exists(metadata_checks) = &self.condition {
            if let Some(OwnerCheck::ByName(owner)) = &metadata_checks.owner {
                points.push(DataPoint::InvolvesUser(&owner));
            }

            if let Some(GroupCheck::ByName(group)) = &metadata_checks.group {
                points.push(DataPoint::InvolvesGroup(&group));
            }
        }

        points
    }
}


// ---- running the check ----

pub trait LookupFile {

    fn prime(&mut self, path: &Path, follow: bool);

    fn does_file_exist(&self, path: &Path) -> bool;

    fn lookup_file(&self, path: &Path, follow: bool) -> Metadata;

    fn read_file_contents(&self, path: &Path) -> Vec<u8>;

    fn lookup_link_target(&self, path: &Path) -> Result<PathBuf, IoError>;
}

impl<F: LookupFile> BuiltInCheck<F> for FilesystemCheck {
    type PASS = Pass;
    type FAIL = Fail;

    fn load(&self, fs: &mut F) {
        fs.prime(&self.input_path, self.follow)
    }

    fn check(&self, fs: &F) -> Vec<CheckResult<Pass, Fail>> {
        info!("Running check");

        let checks = match (&self.condition, fs.does_file_exist(&self.input_path)) {
            (Condition::Exists(cs), true) => cs,
            (Condition::Exists(_), false) => return vec![ CheckResult::Failed(Fail::FileIsMissing) ],
            (Condition::Missing,    true) => return vec![ CheckResult::Failed(Fail::FileExists) ],
            (Condition::Missing,   false) => return vec![ CheckResult::Passed(Pass::FileIsMissing) ],
        };

        let mut results = vec![ CheckResult::Passed(Pass::FileExists) ];

        let metadata = fs.lookup_file(&self.input_path, self.follow);

        if let Some(kind_checks) = &checks.kind {
            self.check_kind(&metadata, kind_checks, &mut results, fs);
        }

        if let Some(owner_checks) = &checks.owner {
            self.check_owner(metadata.uid(), owner_checks, &mut results);
        }

        if let Some(group_checks) = &checks.group {
            self.check_group(metadata.gid(), group_checks, &mut results);
        }

        results
    }
}

impl FilesystemCheck {
    fn check_owner(&self, actual_uid: u32, check: &OwnerCheck, results: &mut Vec<CheckResult<Pass, Fail>>) {
        let actual_user = users::get_user_by_uid(actual_uid);

        match check {
            OwnerCheck::ByName(expected_name) => {
                if let Some(actual_user) = actual_user {
                    if actual_user.name().to_string_lossy() == *expected_name {
                        results.push(CheckResult::Passed(Pass::FileHasOwner));
                    }
                    else {
                        results.push(CheckResult::Failed(Fail::FileHasDifferentOwner(actual_uid, Some(actual_user.name().to_os_string()))));

                        // On top of this, the user we searched for might not exist either
                        if users::get_user_by_name(expected_name).is_none() {
                            results.push(CheckResult::Failed(Fail::UserDoesNotExist(expected_name.clone())));
                        }
                    }
                }
                else {
                    results.push(CheckResult::Failed(Fail::FileHasDifferentOwner(actual_uid, None)));

                    // As above
                    if users::get_user_by_name(expected_name).is_none() {
                        results.push(CheckResult::Failed(Fail::UserDoesNotExist(expected_name.clone())));
                    }
                }
            }
            OwnerCheck::ByID(expected_uid) => {
                if actual_uid == *expected_uid {
                    results.push(CheckResult::Passed(Pass::FileHasOwner));
                }
                else {
                    results.push(CheckResult::Failed(Fail::FileHasDifferentOwner(actual_uid, actual_user.map(|e| e.name().to_os_string()))));
                }
            }
        }
    }

    fn check_group(&self, actual_gid: u32, check: &GroupCheck, results: &mut Vec<CheckResult<Pass, Fail>>) {
        let actual_group = users::get_group_by_gid(actual_gid);

        match check {
            GroupCheck::ByName(expected_name) => {
                if let Some(actual_group) = actual_group {
                    if actual_group.name().to_string_lossy() == *expected_name {
                        results.push(CheckResult::Passed(Pass::FileHasGroup));
                    }
                    else {
                        results.push(CheckResult::Failed(Fail::FileHasDifferentGroup(actual_gid, Some(actual_group.name().to_os_string()))));

                        // On top of this, the user we searched for might not exist either
                        if users::get_user_by_name(expected_name).is_none() {
                            results.push(CheckResult::Failed(Fail::GroupDoesNotExist(expected_name.clone())));
                        }
                    }
                }
                else {
                    results.push(CheckResult::Failed(Fail::FileHasDifferentGroup(actual_gid, None)));

                    // On top of this, the user we searched for might not exist either
                    if users::get_user_by_name(expected_name).is_none() {
                        results.push(CheckResult::Failed(Fail::UserDoesNotExist(expected_name.clone())));
                    }
                }
            }
            GroupCheck::ByID(expected_gid) => {
                if actual_gid == *expected_gid {
                    results.push(CheckResult::Passed(Pass::FileHasGroup));
                }
                else {
                    results.push(CheckResult::Failed(Fail::FileHasDifferentGroup(actual_gid, actual_group.map(|e| e.name().to_os_string()))));
                }
            }
        }
    }

    fn check_kind(&self, metadata: &Metadata, check: &FileKindCheck, results: &mut Vec<CheckResult<Pass, Fail>>, fs: &impl LookupFile) {
        match &check {
            FileKindCheck::File { contents, explicit_check: _ } => {
                if metadata.is_file() {
                    results.push(CheckResult::Passed(Pass::FileIsRegularFile));

                    if let Some(contents) = contents {
                        let read_contents = fs.read_file_contents(&self.input_path);
                        match contents.check(&read_contents) {
                            CheckResult::Passed(pass) => {
                                results.push(CheckResult::Passed(Pass::ContentsPass(pass)));
                            }
                            CheckResult::Failed(fail) => {
                                results.push(CheckResult::Failed(Fail::ContentsFail(fail)));
                            }
                            CheckResult::CommandError(_) => {
                                unreachable!();
                            }
                        }
                    }
                }
                else {
                    let kind = ActualFileKind::from(metadata.file_type());
                    results.push(CheckResult::Failed(Fail::FileIsWrongKind(kind)));
                }
            }
            FileKindCheck::Directory => {
                if metadata.is_dir() {
                    results.push(CheckResult::Passed(Pass::FileIsDirectory));
                }
                else {
                    let kind = ActualFileKind::from(metadata.file_type());
                    results.push(CheckResult::Failed(Fail::FileIsWrongKind(kind)));
                }
            }
            FileKindCheck::Link { target } => {
                if metadata.file_type().is_symlink() {
                    results.push(CheckResult::Passed(Pass::FileIsLink));

                    if let Some(expected_target) = target {
                        let got_target = match fs.lookup_link_target(&self.input_path) {
                            Ok(ta) => ta,
                            Err(e) => {
                                results.push(CheckResult::Failed(Fail::IoErrorReadingLink(e)));
                                return;
                            }
                        };

                        if &got_target == expected_target {
                            results.push(CheckResult::Passed(Pass::FileIsCorrectLink));
                        }
                        else {
                            results.push(CheckResult::Failed(Fail::FileLinksSomewhereElse(got_target)));
                        }
                    }
                }
                else {
                    let kind = ActualFileKind::from(metadata.file_type());
                    results.push(CheckResult::Failed(Fail::FileIsWrongKind(kind)));
                }
            }
        }
    }
}

/// The successful result of a filesystem check.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Pass {

    /// The file exists.
    FileExists,

    /// The file does not exist.
    FileIsMissing,

    /// The file is of the regular file type.
    FileIsRegularFile,

    ContentsPass(contents::Pass),

    /// The file is a directory.
    FileIsDirectory,

    /// The file is a symlink.
    FileIsLink,

    /// The file is a symlink to where it was expected to go.
    FileIsCorrectLink,

    /// The file has the expected permissions.
    FileHasPermissions,

    /// The file has the expected owner.
    FileHasOwner,

    /// The file has the expected group.
    FileHasGroup,
}

/// The failure result of running a filesystem check.
#[derive(Debug)]
pub enum Fail {

    /// The file was meant to exist, but it is missing.
    FileIsMissing,

    /// The file was meant to be missing, but it exists.
    FileExists,

    /// The file was meant to be a certain kind, but it’s actually this kind.
    FileIsWrongKind(ActualFileKind),

    /// The file was meant to link somewhere, but instead it links to this path.
    FileLinksSomewhereElse(PathBuf),

    /// There was an I/O error following this file’s link, so we can’t
    /// tell where it goes.
    IoErrorReadingLink(IoError),

    ContentsFail(contents::Fail),

    /// The file was meant to have certain permissions, but it has different ones.
    FileHasDifferentPermissions,

    /// The file was meant to have a certain owner, but it actually
    /// has this user ID, which may map to the given name.
    FileHasDifferentOwner(u32, Option<OsString>),

    /// The owner the user asked for does not actually exist.
    UserDoesNotExist(String),

    /// The file was meant to be in a certain group, but it actually
    /// has this group, ID, which may map to the given name.
    FileHasDifferentGroup(u32, Option<OsString>),

    /// The group the user asked for does not actually exist.
    GroupDoesNotExist(String),
}

/// One of the file kinds used when printing results.
#[derive(Debug, Copy, Clone)]
pub enum ActualFileKind {
    File,
    Directory,
    Link,
    Other,
}

impl PassResult for Pass {}

impl FailResult for Fail {
    fn command_output(&self) -> Option<(String, &String)> {
        match self {
            Self::ContentsFail(fail)  => fail.command_output("File contents:"),
            _                         => None,
        }
    }

    fn diff_output(&self) -> Option<(String, &String, &String)> {
        match self {
            Self::ContentsFail(fail)  => fail.diff_output(),
            _                         => None,
        }
    }
}

// ---- check result descriptions ----

impl fmt::Display for Pass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileExists => {
                write!(f, "it exists")
            }
            Self::FileIsMissing => {
                write!(f, "it is missing")
            }

            Self::FileIsRegularFile => {
                write!(f, "it is a regular file")
            }
            Self::ContentsPass(contents_pass) => {
                write!(f, "its contents {}", contents_pass)
            }
            Self::FileIsDirectory => {
                write!(f, "it is a directory")
            }
            Self::FileIsLink => {
                write!(f, "it is a link")
            }
            Self::FileIsCorrectLink => {
                write!(f, "it links to the right place")
            }

            Self::FileHasPermissions => {
                write!(f, "it has the right permissions")
            }
            Self::FileHasOwner => {
                write!(f, "it has the right owner")
            }
            Self::FileHasGroup => {
                write!(f, "it has the right group")
            }
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileIsMissing => {
                write!(f, "it is missing")
            }
            Self::FileExists => {
                write!(f, "a file exists")
            }

            Self::FileIsWrongKind(ft) => {
                write!(f, "it is a {}", ft)
            }
            Self::FileLinksSomewhereElse(got_target) => {
                write!(f, "it actually links to {}", got_target.display())
            }
            Self::IoErrorReadingLink(ioe) => {
                write!(f, "error reading link: {}", ioe)
            }

            Self::ContentsFail(contents_fail) => {
                write!(f, "its contents {}", contents_fail)
            }

            Self::FileHasDifferentPermissions => {
                write!(f, "it has the wrong permissions")
            }

            Self::UserDoesNotExist(user) => {
                write!(f, "user ‘{}’ does not exist", user)
            }
            Self::FileHasDifferentOwner(actual_uid, None) => {
                write!(f, "it is actually owned by an unknown user ({})", actual_uid)
            }
            Self::FileHasDifferentOwner(actual_uid, Some(actual_owner)) => {
                write!(f, "it is actually owned by ‘{}’ ({})", actual_owner.to_string_lossy(), actual_uid)
            }
            Self::GroupDoesNotExist(group) => {
                write!(f, "group {} does not exist", group)
            }
            Self::FileHasDifferentGroup(actual_gid, None) => {
                write!(f, "it actually has an unknown group ({})", actual_gid)
            }
            Self::FileHasDifferentGroup(actual_gid, Some(actual_owner)) => {
                write!(f, "it actually has group ‘{}’ ({})", actual_owner.to_string_lossy(), actual_gid)
            }
        }
    }
}

impl fmt::Display for ActualFileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File       => write!(f, "regular file"),
            Self::Directory  => write!(f, "directory"),
            Self::Link       => write!(f, "link"),
            Self::Other      => write!(f, "other"),
        }
    }
}

impl From<FileType> for ActualFileKind {
    fn from(ft: FileType) -> Self {
        if ft.is_dir() {
            Self::Directory
        }
        else if ft.is_symlink() {
            Self::Link
        }
        else if ft.is_file() {
            Self::File
        }
        else {
            Self::Other
        }
    }
}
