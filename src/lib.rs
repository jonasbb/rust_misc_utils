#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
#![warn(missing_docs)]

extern crate bzip2;
#[macro_use]
extern crate failure;
extern crate flate2;
#[macro_use]
extern crate log;
#[cfg(feature = "jsonl")]
extern crate serde;
#[cfg(feature = "jsonl")]
extern crate serde_json;
extern crate xz2;

pub mod error;
pub mod fs;
