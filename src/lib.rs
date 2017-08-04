extern crate bzip2;
#[macro_use]
extern crate error_chain;
extern crate flate2;
#[macro_use]
extern crate log;
extern crate xz2;

pub mod error;
pub mod fs;

pub use error::{Error, ErrorKind};
