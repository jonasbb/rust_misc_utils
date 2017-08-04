use bzip2::bufread::BzDecoder;
use error::*;
use flate2::bufread::MultiGzDecoder;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::path::Path;
use xz2::bufread::XzDecoder;

/// Configure behaviour of the [`file_open_read_with_options`] function
///
/// [`file_open_read_with_options`]: ./fn.file_open_read_with_options.html
#[derive(Clone, Debug)]
pub struct ReadOptions {
    buffer_capacity: Option<usize>,
    open_options: OpenOptions,
}

impl ReadOptions {
    /// Create a new `ReadOptions` with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the capacity of the [`BufReader`] to `capacity` in Bytes.
    ///
    /// [`BufReader`]: https://doc.rust-lang.org/std/io/struct.BufReader.html
    pub fn set_buffer_capacity(mut self, capacity: usize) -> Self {
        self.buffer_capacity = Some(capacity);
        self
    }

    /// Specify a set of [`OpenOptions`] to use
    ///
    /// The option `read` will always be overwritten to `true` and `write` will always be set to
    /// `false`.
    ///
    /// [`OpenOptions`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html
    pub fn set_open_options(mut self, open_options: OpenOptions) -> Self {
        self.open_options = open_options;
        self
    }
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            buffer_capacity: None,
            open_options: OpenOptions::new(),
        }
    }
}

/// Create reader for uncompressed or compressed files transparently
///
/// See [`file_open_read_with_options`] for the full documentation.
///
/// [`file_open_read_with_options`]: ./fn.file_open_read_with_options.html
pub fn file_open_read<P>(file: P) -> Result<Box<Read>>
where
    P: AsRef<Path>,
{
    file_open_read_with_option_do(file.as_ref(), ReadOptions::default())
}

/// Create reader for uncompressed or compressed files transparently
///
/// This function opens the given `file` and tries to determine the filetype by reading the magic
/// bytes from the start of the file. If a known archive type, like xz, gz, or bz2, is found this
/// function will transparent create a reader which decompresses the data on the fly.
///
/// File I/O will always be buffered using a [`BufReader`].
///
/// The behaviour of this function can be configured using [`ReadOptions`]. See the documentation
/// on the struct for details.
///
/// [`BufReader`]: https://doc.rust-lang.org/std/io/struct.BufReader.html
/// [`ReadOptions`]: ./struct.ReadOptions.html
pub fn file_open_read_with_options<P>(file: P, options: ReadOptions) -> Result<Box<Read>>
where
    P: AsRef<Path>,
{
    file_open_read_with_option_do(file.as_ref(), options)
}

fn file_open_read_with_option_do(file: &Path, mut options: ReadOptions) -> Result<Box<Read>> {
    if !file.is_file() {
        return Err(ErrorKind::PathNotAFile(file.to_path_buf()).into());
    }

    let f = options
        .open_options
        .read(true)
        .write(false)
        .open(file)
        .chain_err(|| format!("Could not open file {:?}", file))?;
    let mut bufread = if let Some(size) = options.buffer_capacity {
        BufReader::with_capacity(size, f)
    } else {
        BufReader::new(f)
    };

    // read magic bytes
    let mut buffer = [0; 6];
    bufread.read_exact(&mut buffer).chain_err(|| {
        format!("Failed to read file header of {:?}", file)
    })?;
    // reset the read position
    bufread.seek(SeekFrom::Start(0)).chain_err(
        || "Failed to seek to start of file.",
    )?;

    // check if file if XZ compressed
    if buffer[..6] == [0xfd, '7' as u8, 'z' as u8, 'X' as u8, 'Z' as u8, 0x00] {
        debug!("File {:?} is detected to have type `xz`", file);
        Ok(Box::new(XzDecoder::new(bufread)))
    } else if buffer[..2] == [0x1f, 0x8b] {
        debug!("File {:?} is detected to have type `gz`", file);
        Ok(Box::new(MultiGzDecoder::new(bufread).chain_err(
            || "Failed to create the gz reader",
        )?))
    } else if buffer[..3] == ['B' as u8, 'Z' as u8, 'h' as u8] {
        debug!("File {:?} is detected to have type `bz2`", file);
        Ok(Box::new(BzDecoder::new(bufread)))
    } else {
        debug!("Open file {:?} as plaintext", file);
        Ok(Box::new(bufread))
    }
}
