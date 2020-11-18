//! Process execution and output processing

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
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::new_without_default)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::wildcard_imports)]
#![warn(clippy::clone_on_ref_ptr)]

#![deny(unsafe_code)]


mod exec;
pub use self::exec::*;

mod executor;
pub use self::executor::*;

mod error;
pub use self::error::*;
