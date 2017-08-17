#![allow(unknown_lints)]

use std::path::PathBuf;

error_chain! {
    foreign_links {
        Io(::std::io::Error)
        #[doc = "A wrapper for [`std::io::Error`]"]
        #[doc = "[`std::io::Error`]: https://doc.rust-lang.org/std/io/struct.Error.html"];
    }

    errors {
        #[doc = "Indicates that a [`Path`] must point to a file but does not."]
        #[doc = "[`Path`]: https://doc.rust-lang.org/std/path/struct.Path.html"]
        PathNotAFile(path: PathBuf) {
            description("path does not point to a file"),
            display("the path '{:?}' does not point to a file", path),
        }
    }
}
