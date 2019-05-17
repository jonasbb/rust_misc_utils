//! This module contains functions for file system path manipulation.

use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

/// This traits extends the available methods on [`Path`].
pub trait PathExt {
    /// Iterator over all file extensions of a [`Path`].
    ///
    /// This iterator provides access to all file extensions from starting with the last extension.
    /// File extensions are separated by a `.`-character. This supplements the [`Path::extension`] method,
    /// which only allows you to access the last file extension.
    ///
    /// Accessing multiple extension can be useful, if extensions are chained to provide hints how the
    /// file is structured, e.g., `archive.tar.xz`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use misc_utils::path::PathExt;
    /// # use std::ffi::OsStr;
    /// # use std::path::Path;
    /// #
    /// let p = &Path::new("/home/user/projects/misc_utils/This.File.has.many.extensions");
    /// assert_eq!(
    ///     p.extensions().collect::<Vec<_>>(),
    ///     vec![
    ///         OsStr::new("extensions"),
    ///         OsStr::new("many"),
    ///         OsStr::new("has"),
    ///         OsStr::new("File")
    ///     ]
    /// );
    /// ```
    fn extensions(&'_ self) -> PathExtensions<'_>;
}

impl PathExt for Path {
    fn extensions(&'_ self) -> PathExtensions<'_> {
        PathExtensions(self)
    }
}

/// This traits extends the available methods on [`PathBuf`].
pub trait PathBufExt {
    /// Appends `extension` to [`self.file_name`](Path::file_name).
    ///
    /// Returns false and does nothing if [`self.file_name`](Path::file_name) is [`None`], returns `true` and appends the extension otherwise.
    ///
    /// The API and documentation should fully mirror [`PathBuf::set_extension`].
    fn add_extension<S: AsRef<OsStr>>(&mut self, extension: S) -> bool;
}

impl PathBufExt for PathBuf {
    fn add_extension<S: AsRef<OsStr>>(&mut self, extension: S) -> bool {
        if self.file_name().is_none() {
            return false;
        }

        let mut stem = match self.file_name() {
            Some(stem) => stem.to_os_string(),
            None => OsString::new(),
        };

        if !extension.as_ref().is_empty() {
            stem.push(".");
            stem.push(extension.as_ref());
        }
        self.set_file_name(&stem);

        true
    }
}

/// Iterator over all file extensions of a [`Path`].
///
/// This iterator provides access to all file extensions from starting with the last extension.
/// File extensions are separated by a `.`-character. This supplements the [`Path::extension`] method,
/// which only allows you to access the last file extension.
///
/// Accessing multiple extension can be useful, if extensions are chained to provide hints how the
/// file is structured, e.g., `archive.tar.xz`.
///
/// # Example
///
/// ```rust
/// # use misc_utils::path::PathExt;
/// # use std::ffi::OsStr;
/// # use std::path::Path;
/// #
/// let p = &Path::new("/home/user/projects/misc_utils/This.File.has.many.extensions");
/// assert_eq!(
///     p.extensions().collect::<Vec<_>>(),
///     vec![
///         OsStr::new("extensions"),
///         OsStr::new("many"),
///         OsStr::new("has"),
///         OsStr::new("File")
///     ]
/// );
/// ```
#[derive(Copy, Clone, Debug)]
pub struct PathExtensions<'a>(&'a Path);

impl<'a> Iterator for PathExtensions<'a> {
    type Item = &'a OsStr;

    fn next(&mut self) -> Option<&'a OsStr> {
        let (new_filestem, new_extension) = (self.0.file_stem(), self.0.extension());
        if new_extension.is_none() {
            self.0 = Path::new("");
            None
        } else {
            if let Some(new_filestem) = new_filestem {
                self.0 = Path::new(new_filestem);
            } else {
                self.0 = Path::new("")
            };
            new_extension
        }
    }
}

#[test]
fn test_path_extensions() {
    let p = &Path::new("/home/user/projects/misc_utils/Cargo.toml");
    assert_eq!(p.extensions().collect::<Vec<_>>(), vec![OsStr::new("toml")]);
    let p = &Path::new("/home/user/projects/misc_utils/This.File.has.many.extensions");
    assert_eq!(
        p.extensions().collect::<Vec<_>>(),
        vec![
            OsStr::new("extensions"),
            OsStr::new("many"),
            OsStr::new("has"),
            OsStr::new("File")
        ]
    );
    let p = &Path::new("/home/user/projects/misc_utils/.hidden");
    assert_eq!(p.extensions().collect::<Vec<_>>(), Vec::<&OsStr>::new());
    let p = &Path::new("Just-A.file");
    assert_eq!(p.extensions().collect::<Vec<_>>(), vec![OsStr::new("file")]);
}

#[test]
fn test_pathbuf_extensions() {
    let p = PathBuf::from("/home/user/projects/misc_utils/Cargo.toml");
    assert_eq!(p.extensions().collect::<Vec<_>>(), vec![OsStr::new("toml")]);
    let p = PathBuf::from("/home/user/projects/misc_utils/This.File.has.many.extensions");
    assert_eq!(
        p.extensions().collect::<Vec<_>>(),
        vec![
            OsStr::new("extensions"),
            OsStr::new("many"),
            OsStr::new("has"),
            OsStr::new("File")
        ]
    );
    let p = PathBuf::from("/home/user/projects/misc_utils/.hidden");
    assert_eq!(p.extensions().collect::<Vec<_>>(), Vec::<&OsStr>::new());
    let p = PathBuf::from("Just-A.file");
    assert_eq!(p.extensions().collect::<Vec<_>>(), vec![OsStr::new("file")]);
}

#[test]
fn test_add_extension() {
    let mut pb = PathBuf::from("some.file");
    assert_eq!(pb, Path::new("some.file"));
    assert!(pb.add_extension("a"));
    assert_eq!(pb, Path::new("some.file.a"));
    assert!(pb.add_extension("b"));
    assert_eq!(pb, Path::new("some.file.a.b"));
    assert!(pb.add_extension("c"));
    assert_eq!(pb, Path::new("some.file.a.b.c"));

    let mut pb = PathBuf::from("/");
    assert!(!pb.add_extension("ext"));
}
