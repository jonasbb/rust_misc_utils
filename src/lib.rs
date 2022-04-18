#![warn(
    clippy::semicolon_if_nothing_returned,
    missing_copy_implementations,
    missing_crate_level_docs,
    missing_debug_implementations,
    missing_docs,
    rust_2018_compatibility,
    rust_2021_compatibility,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
#![warn(rust_2018_idioms)]
#![doc(html_root_url = "https://docs.rs/misc_utils/4.0.1")]

//! This crate contains miscellaneous utility functions
//!
//! Currently this crate contains functions to
//!
//! * interact with the filesystem in `fs`
//! * process line-separated JSON data

pub mod error;
pub mod fs;
mod minmax;
pub mod path;

pub use crate::minmax::{Max, Min};

pub fn check() {
    vec!(1, 2, 3, 4, 5).resize(0, 5)
}
