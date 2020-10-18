//! The check types for Specsheet.
//!
//! This crate handles:
//!
//! - Information about each check, such as their names, what parameters they accept, and their data types.
//! - Reading checks from TOML.
//! - Running checks by examining the system or executing a command.
//! - Writing check names and results out as sentences.

#![warn(deprecated_in_future)]
#![warn(future_incompatible)]
#![warn(missing_copy_implementations)]
#![allow(missing_docs)]
#![warn(nonstandard_style)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rustdoc)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused)]

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::pub_enum_variant_names)]
#![allow(clippy::wildcard_imports)]
#![warn(clippy::clone_on_ref_ptr)]

#![deny(unsafe_code)]


// the base types

mod check;
pub use self::check::{Check, BuiltInCheck, RunCheck, CheckResult, PassResult, FailResult};


// check types

mod local;
pub use self::local::*;

mod command;
pub use self::command::*;

mod network;
pub use self::network::*;


// helpers

pub mod common;
pub mod contents;
pub mod load;
pub mod read;
