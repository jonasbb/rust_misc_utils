#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]

//! This crate contains miscellaneous utility functions
//!
//! Currently this crate contains functions to
//!
//! * interact with the filesystem in `fs`
//! * process line-separated JSON data

extern crate bzip2;
extern crate failure;
extern crate flate2;
extern crate log;
extern crate num_traits;
#[cfg(feature = "jsonl")]
extern crate serde;
#[cfg(feature = "jsonl")]
extern crate serde_json;
extern crate xz2;

pub mod error;
pub mod fs;
mod minmax;

pub use minmax::{Max, Min};
