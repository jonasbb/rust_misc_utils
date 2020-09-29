//! This modules contains all error type definitions for this crate
//!
//! See the description of the individual error types for more details.

use std::path::{Path, PathBuf};

/// Error value for elements returned by [`MtJsonl`](crate::fs::MtJsonl).
///
/// Please see the individual variants for details.
#[cfg(feature = "jsonl")]
#[allow(variant_size_differences)]
#[derive(Debug, thiserror::Error)]
pub enum MtJsonlError {
    /// Indicates some error while processing the file.
    /// Not all lines in the file were processed.
    #[error("Reading the file has failed and not all entries could be read.")]
    NotCompleted,

    /// Some error occured while opening or reading the file.
    /// Created in the reader thread based on a [`std::io::Error`].
    #[error("IO Error while processing the file '{:?}'", file)]
    IoError {
        /// Custom message describing the error in more detail.
        msg: String,
        /// File which causes the IO Errors.
        file: PathBuf,
        /// Underlying IO Error
        #[source]
        source: anyhow::Error,
    },

    /// Some error occured while parsing a JSON value
    /// Created in the parsing thread based on a [`serde_json::Error`]
    #[error("Could not parse a JSON value")]
    ParsingError {
        /// Error message of the parsing library
        #[from]
        #[source]
        source: serde_json::Error,
    },
}

/// Error to indicate that a path does not point to a valid file.
#[derive(thiserror::Error, Debug)]
#[error("'{}' is not a file", path.display())]
pub struct NotAFileError {
    /// The path which produced the error.
    pub path: PathBuf,
}

impl NotAFileError {
    /// Create a new error for the given path.
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}
