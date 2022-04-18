//! This modules contains all error type definitions for this crate
//!
//! See the description of the individual error types for more details.

use std::{io, path::PathBuf};

/// Error type for misc_utils crate.
///
/// Please see the individual variants for details.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The path to open is not a file
    #[error("{} is not a file", path.display())]
    NotAFileError {
        /// Path
        path: PathBuf,
    },
    /// Wrapper around [io::Error]
    #[error("{msg} while operating on file {}", file.display())]
    FileIo {
        /// File which caused the error
        file: PathBuf,
        /// Message describing what went wrong
        msg: &'static str,
        /// Underlying source [io::Error]
        #[source]
        source: io::Error,
    },
    /// Errors when a known compression technique is used but the crate feature is not enabled
    #[error("File {} is detected to be type `{technique}`, but the file-{technique} feature is not enabled.", file.display())]
    CompressionNotEnabled {
        /// File which is used for reading or writing
        file: PathBuf,
        /// Name of the compression technique
        technique: &'static str,
    },
    #[cfg(feature = "file-xz")]
    /// Error when creating a XZ reader
    ///
    /// This variant only exists if the `file-xz` feature is enabled.
    #[error("Failed to initialize the xz multithreaded stream for file {}", file.display())]
    XzError {
        /// File which is opened for reading
        file: PathBuf,
        /// Original cause of the error
        #[source]
        source: xz2::stream::Error,
    },
    /// Error when joining an async task
    ///
    /// This variant only exists if the `async-fs` feature is enabled.
    #[cfg(feature = "async-fs")]
    #[error("Failed to join Tokio task")]
    JoinError {
        /// Original cause of the error
        #[from]
        source: tokio::task::JoinError,
    },
}

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
    #[error(transparent)]
    IoError {
        /// Source Error
        #[from]
        source: Error,
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
