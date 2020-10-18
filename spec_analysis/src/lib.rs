//! Analysing the properties of completed checks to look for any similarities
//! or correlations.

#![warn(deprecated_in_future)]
#![warn(future_incompatible)]
#![warn(missing_copy_implementations)]
#![warn(missing_docs)]
#![warn(nonstandard_style)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rustdoc)]
#![allow(single_use_lifetimes)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused)]

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::must_use_attribute)]
#![allow(clippy::non_ascii_literal)]

#![deny(unsafe_code)]

mod property;
pub use self::property::DataPoint;

mod table;
pub use self::table::AnalysisTable;
