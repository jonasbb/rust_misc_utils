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
#![doc(html_root_url = "https://docs.rs/misc_utils/4.2.2")]

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

///  Contains functions to print bytes in a human-readable format.
///
/// The bytes are printes in a mostly ASCII format.
/// Non-ASCII values are escaped using the `\xXX` notation.
///
/// # Examples
///
/// ```rust
/// let bytes = [72, 101, 108, 108, 111, 10, 0, 9, 10, 0xde, 0xad, 0xbe, 0xef];
/// let expected = r"Hello\n\0\t\n\xde\xad\xbe\xef";
/// assert_eq!(expected, misc_utils::byteascii::byteascii(&bytes));
/// ```
pub mod byteascii {
    use std::fmt;

    // Map each byte to its escaped version
    #[rustfmt::skip]
    static BYTESPRINTED: [&str; 256] = [
        "\\0",   "\\x01", "\\x02", "\\x03", "\\x04", "\\x05", "\\x06", "\\x07",
        "\\x08", "\\t",   "\\n",   "\\x0b", "\\x0c", "\\r",   "\\x0e", "\\x0f",
        "\\x10", "\\x11", "\\x12", "\\x13", "\\x14", "\\x15", "\\x16", "\\x17",
        "\\x18", "\\x19", "\\x1a", "\\x1b", "\\x1c", "\\x1d", "\\x1e", "\\x1f",
        " ",     "!",     "\"",    "#",     "$",     "%",     "&",     "'",
        "(",     ")",     "*",     "+",     ",",     "-",     ".",     "/",
        "0",     "1",     "2",     "3",     "4",     "5",     "6",     "7",
        "8",     "9",     ":",     ";",     "<",     "=",     ">",     "?",
        "@",     "A",     "B",     "C",     "D",     "E",     "F",     "G",
        "H",     "I",     "J",     "K",     "L",     "M",     "N",     "O",
        "P",     "Q",     "R",     "S",     "T",     "U",     "V",     "W",
        "X",     "Y",     "Z",     "[",     "\\\\",  "]",     "^",    "_",
        "`",     "a",     "b",     "c",     "d",     "e",     "f",     "g",
        "h",     "i",     "j",     "k",     "l",     "m",     "n",     "o",
        "p",     "q",     "r",     "s",     "t",     "u",     "v",     "w",
        "x",     "y",     "z",     "{",     "|",     "}",     "~",     "\\x7f",
        "\\x80", "\\x81", "\\x82", "\\x83", "\\x84", "\\x85", "\\x86", "\\x87",
        "\\x88", "\\x89", "\\x8a", "\\x8b", "\\x8c", "\\x8d", "\\x8e", "\\x8f",
        "\\x90", "\\x91", "\\x92", "\\x93", "\\x94", "\\x95", "\\x96", "\\x97",
        "\\x98", "\\x99", "\\x9a", "\\x9b", "\\x9c", "\\x9d", "\\x9e", "\\x9f",
        "\\xa0", "\\xa1", "\\xa2", "\\xa3", "\\xa4", "\\xa5", "\\xa6", "\\xa7",
        "\\xa8", "\\xa9", "\\xaa", "\\xab", "\\xac", "\\xad", "\\xae", "\\xaf",
        "\\xb0", "\\xb1", "\\xb2", "\\xb3", "\\xb4", "\\xb5", "\\xb6", "\\xb7",
        "\\xb8", "\\xb9", "\\xba", "\\xbb", "\\xbc", "\\xbd", "\\xbe", "\\xbf",
        "\\xc0", "\\xc1", "\\xc2", "\\xc3", "\\xc4", "\\xc5", "\\xc6", "\\xc7",
        "\\xc8", "\\xc9", "\\xca", "\\xcb", "\\xcc", "\\xcd", "\\xce", "\\xcf",
        "\\xd0", "\\xd1", "\\xd2", "\\xd3", "\\xd4", "\\xd5", "\\xd6", "\\xd7",
        "\\xd8", "\\xd9", "\\xda", "\\xdb", "\\xdc", "\\xdd", "\\xde", "\\xdf",
        "\\xe0", "\\xe1", "\\xe2", "\\xe3", "\\xe4", "\\xe5", "\\xe6", "\\xe7",
        "\\xe8", "\\xe9", "\\xea", "\\xeb", "\\xec", "\\xed", "\\xee", "\\xef",
        "\\xf0", "\\xf1", "\\xf2", "\\xf3", "\\xf4", "\\xf5", "\\xf6", "\\xf7",
        "\\xf8", "\\xf9", "\\xfa", "\\xfb", "\\xfc", "\\xfd", "\\xfe", "\\xff",
    ];

    /// Print a byte sequence as an ASCII string.
    ///
    /// This function prints the bytes under the assumption most of the values are in the ASCII range.
    /// It prints ASCII characters and encodes other bytes as `\xHH` where `HH` is the hexadecimal value.
    pub fn byteascii(bytes: &[u8]) -> String {
        bytes.iter().map(|&b| BYTESPRINTED[b as usize]).collect()
    }

    /// [`Debug`] print a byte sequence as an ASCII string.
    ///
    /// This newtype can be used when creating a [`Debug`] implementation for a type.
    /// It prints the bytes identical to [`byteascii`].
    pub struct ByteAscii<B>(pub B);

    impl<B> fmt::Debug for ByteAscii<B>
    where
        B: AsRef<[u8]>,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for &b in self.0.as_ref() {
                f.write_str(BYTESPRINTED[b as usize])?;
            }
            Ok(())
        }
    }

    #[test]
    fn test_byteascii() {
        let mut all_bytes: [u8; 256] = [0; 256];
        for (i, b) in all_bytes.iter_mut().enumerate() {
            *b = i as u8;
        }
        expect_test::expect![[r##"\0\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0b\x0c\r\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\x7f\x80\x81\x82\x83\x84\x85\x86\x87\x88\x89\x8a\x8b\x8c\x8d\x8e\x8f\x90\x91\x92\x93\x94\x95\x96\x97\x98\x99\x9a\x9b\x9c\x9d\x9e\x9f\xa0\xa1\xa2\xa3\xa4\xa5\xa6\xa7\xa8\xa9\xaa\xab\xac\xad\xae\xaf\xb0\xb1\xb2\xb3\xb4\xb5\xb6\xb7\xb8\xb9\xba\xbb\xbc\xbd\xbe\xbf\xc0\xc1\xc2\xc3\xc4\xc5\xc6\xc7\xc8\xc9\xca\xcb\xcc\xcd\xce\xcf\xd0\xd1\xd2\xd3\xd4\xd5\xd6\xd7\xd8\xd9\xda\xdb\xdc\xdd\xde\xdf\xe0\xe1\xe2\xe3\xe4\xe5\xe6\xe7\xe8\xe9\xea\xeb\xec\xed\xee\xef\xf0\xf1\xf2\xf3\xf4\xf5\xf6\xf7\xf8\xf9\xfa\xfb\xfc\xfd\xfe\xff"##]].assert_eq(&byteascii(&all_bytes));
    }

    #[test]
    fn test_byteascii_newtype() {
        let mut all_bytes: [u8; 256] = [0; 256];
        for (i, b) in all_bytes.iter_mut().enumerate() {
            *b = i as u8;
        }
        expect_test::expect![[r##"
            \0\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0b\x0c\r\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\x7f\x80\x81\x82\x83\x84\x85\x86\x87\x88\x89\x8a\x8b\x8c\x8d\x8e\x8f\x90\x91\x92\x93\x94\x95\x96\x97\x98\x99\x9a\x9b\x9c\x9d\x9e\x9f\xa0\xa1\xa2\xa3\xa4\xa5\xa6\xa7\xa8\xa9\xaa\xab\xac\xad\xae\xaf\xb0\xb1\xb2\xb3\xb4\xb5\xb6\xb7\xb8\xb9\xba\xbb\xbc\xbd\xbe\xbf\xc0\xc1\xc2\xc3\xc4\xc5\xc6\xc7\xc8\xc9\xca\xcb\xcc\xcd\xce\xcf\xd0\xd1\xd2\xd3\xd4\xd5\xd6\xd7\xd8\xd9\xda\xdb\xdc\xdd\xde\xdf\xe0\xe1\xe2\xe3\xe4\xe5\xe6\xe7\xe8\xe9\xea\xeb\xec\xed\xee\xef\xf0\xf1\xf2\xf3\xf4\xf5\xf6\xf7\xf8\xf9\xfa\xfb\xfc\xfd\xfe\xff
        "##]].assert_debug_eq(& ByteAscii(&all_bytes));
    }
}
