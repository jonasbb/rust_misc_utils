#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    missing_docs,
    rust_2018_compatibility,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
#![warn(
    rust_2018_idioms,
)]

//! This crate contains miscellaneous utility functions
//!
//! Currently this crate contains functions to
//!
//! * interact with the filesystem in `fs`
//! * process line-separated JSON data

pub mod error;
pub mod fs;
mod minmax;

pub use crate::minmax::{Max, Min};
