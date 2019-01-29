//! This modules contains all error type definitions for this crate
//!
//! See the description of the individual error types for more details.

use failure::Fail;
#[cfg(feature = "jsonl")]
use failure::{Backtrace, Context};
use std::{
    fmt::{self, Display},
    path::{Path, PathBuf},
};

/// `ErrorKind` variant for [`MtJsonlError`]s.
///
/// The error kind specifies the error in more detail. Please see the individual variants for details.
///
/// [`MtJsonlError`]: ./MtJsonlError.t.html
#[cfg(feature = "jsonl")]
#[allow(variant_size_differences)]
#[derive(Debug, Fail)]
pub enum MtJsonlErrorKind {
    /// Indicates some error while processing the file.
    /// Not all lines in the file were processed.
    #[fail(display = "Reading the file has failed and not all entries could be read.")]
    NotCompleted,

    /// Some error occured while opening or reading the file.
    /// Created in the reader thread based on a [`std::io::Error`].
    ///
    /// [`std::io::Error`]: https://doc.rust-lang.org/std/io/struct.Error.html
    #[fail(display = "IO Error while processing the file '{:?}'", file)]
    IoError {
        /// Custom message describing the error in more detail.
        msg: String,
        /// File which causes the IO Errors.
        file: PathBuf,
    },

    /// Some error occured while parsing a JSON value
    /// Created in the parsing thread based on a [`serde_json::Error`][serde_json]
    ///
    /// [serde_json]: https://docs.rs/serde_json/
    #[fail(display = "Could not parse a JSON value")]
    ParsingError,
}

/// Error value for elements returned by [`MtJsonl`].
///
/// Look at [`MtJsonlErrorKind`] for details.
///
/// [`MtJsonl`]: ../fs/MtJsonl.t.html
/// [`MtJsonlErrorKind`]: ./MtJsonlErrorKind.t.html
#[cfg(feature = "jsonl")]
#[derive(Debug)]
pub struct MtJsonlError {
    inner: Context<MtJsonlErrorKind>,
}

#[cfg(feature = "jsonl")]
impl Fail for MtJsonlError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

#[cfg(feature = "jsonl")]
impl Display for MtJsonlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

#[cfg(feature = "jsonl")]
impl MtJsonlError {
    /// Return the error kind for this error.
    pub fn kind(&self) -> &MtJsonlErrorKind {
        self.inner.get_context()
    }
}

#[cfg(feature = "jsonl")]
impl From<MtJsonlErrorKind> for MtJsonlError {
    fn from(kind: MtJsonlErrorKind) -> MtJsonlError {
        MtJsonlError {
            inner: Context::new(kind),
        }
    }
}

#[cfg(feature = "jsonl")]
impl From<Context<MtJsonlErrorKind>> for MtJsonlError {
    fn from(inner: Context<MtJsonlErrorKind>) -> MtJsonlError {
        MtJsonlError { inner }
    }
}

/// Error to indicate that a path does not point to a valid file.
#[derive(Fail, Debug)]
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

impl Display for NotAFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("'{}' is not a file.", self.path.display()))
    }
}
