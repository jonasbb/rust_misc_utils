#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
#![warn(
    clippy::semicolon_if_nothing_returned,
    rustdoc::missing_crate_level_docs
)]
#![doc(html_root_url = "https://docs.rs/misc_utils/4.1.0")]

//! This crate contains miscellaneous utility functions
//!
//! Currently this crate contains functions to
//!
//! * interact with the filesystem in `fs`
//! * process line-separated JSON data

#[cfg(feature = "async-fs")]
pub mod async_fs;
pub mod error;
pub mod fs;
mod minmax;
pub mod path;

pub use crate::minmax::{Max, Min};
