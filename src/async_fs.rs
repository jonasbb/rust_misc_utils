//! This module contains functions related to async filesystem operations.
//!
//! The module mirrors the functions from [`tokio::fs`].
//! All types from [`tokio::fs`] are re-exported here.
//! Some functions are overwritten and have different error types.

use crate::error::Error;
use std::path::Path;
#[doc(inline)]
pub use tokio::fs::*;

/// Read the entire contents of a file into a bytes vector.
///
/// This function supports opening compressed files transparently.
///
/// The API mirrors the function in [`tokio::fs::read`] but is implemented via [`crate::fs::read`].
pub async fn read(path: impl AsRef<Path>) -> Result<Vec<u8>, Error> {
    let path = path.as_ref().to_owned();
    tokio::task::spawn_blocking(move || crate::fs::read(path)).await?
}

/// Read the entire contents of a file into a string.
///
/// This function supports opening compressed files transparently.
///
/// The API mirrors the function in [`tokio::fs::read_to_string`] but is implemented via [`crate::fs::read_to_string`].
pub async fn read_to_string(path: impl AsRef<Path>) -> Result<String, Error> {
    let path = path.as_ref().to_owned();
    tokio::task::spawn_blocking(move || crate::fs::read_to_string(path)).await?
}

/// Write a slice as the entire contents of a file.
///
/// The functions chooses the filetype based on the extension.
/// If a recognized extension is used the file will be compressed otherwise the file will be written as plaintext.
/// If compression is used, it will use the default (6) compression ratio.
/// The method will truncate the file before writing, such that `contents` will be the only content of the file.
///
/// The API mirrors the function in [`tokio::fs::write`] but is implemented via [`crate::fs::write`].
pub async fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<(), Error> {
    // TODO: Should probably use scoped tasks, if they become available
    // https://github.com/tokio-rs/tokio/issues/3162
    let path = path.as_ref().to_owned();
    let contents = contents.as_ref().to_owned();
    tokio::task::spawn_blocking(move || crate::fs::write(path, contents)).await?
}

/// Append the content to the file.
///
/// This function only works for plaintext and gzip files.
///
/// The function is similar to [`write()`] but will append the content to the end of the file instead of truncating.
pub async fn append(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<(), Error> {
    // TODO: Should probably use scoped tasks, if they become available
    // https://github.com/tokio-rs/tokio/issues/3162
    let path = path.as_ref().to_owned();
    let contents = contents.as_ref().to_owned();
    tokio::task::spawn_blocking(move || crate::fs::append(path, contents)).await?
}
