//! Spec commands


#![warn(deprecated_in_future)]
#![warn(future_incompatible)]
#![warn(missing_copy_implementations)]
#![warn(missing_docs)]
#![warn(nonstandard_style)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rustdoc)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused)]

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::map_entry)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::option_option)]
#![allow(clippy::tabs_in_doc_comments)]
#![allow(clippy::wildcard_imports)]
#![warn(clippy::clone_on_ref_ptr)]

#![deny(unsafe_code)]

pub mod apt;
pub mod brew_cask;
pub mod brew_tap;
pub mod brew;
pub mod curl;
pub mod defaults;
pub mod dig;
pub mod files;
pub mod gem;
pub mod hash;
pub mod net;
pub mod npm;
pub mod passwd;
pub mod ping;
pub mod shell;
pub mod systemctl;
pub mod ufw;

use std::collections::BTreeMap;
use std::time::Duration;

use spec_exec::Exec;


/// **Global options** are set by a user per-run. They contain things that can
/// be overridden.
pub trait GlobalOptions {

    /// A simple key-value lookup.
    fn key_value(&self, key_name: &'static str) -> Option<String>;

    /// All options that start with the given key.
    fn key_prefix_values(&self, key_prefix: &'static str) -> BTreeMap<String, String>;

    /// A duration lookup.
    fn duration(&self, key_name: &'static str) -> Option<Duration>;

    /// A command lookup.
    fn command<T: spec_exec::CommandOutput>(&self, key_name: &'static str) -> Option<Exec<T>>;
}
