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

fn unused() {
    vec![1, 2, 3, 4, 5].resize(0, 5);
}
