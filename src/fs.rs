//! This module contains functions related to filesystem operations.
//!
//! ## [`file_open_read`] / [`file_open_read_with_options`]
//!
//! These functions are convenience wrappers around file I/O. They allow reading of compressed
//! files in a transparent manner.
//!
//! Reading compressed files works for `.bz2`/`.gz`/`.xz` files.
//! The support for for the different file formats is optional.
//! By default `.gz` and `.xz` are enabled.
//! The `file-*` features enable support for the corresponding file extensions.
//!
//! The example shows how to read a file into a string:
//!
//! ```no_run
//! # extern crate misc_utils;
//! # use misc_utils::fs::file_open_read;
//! #
//! # fn main() {
//! let mut reader = file_open_read("./text.txt").unwrap();
//! let mut content = String::new();
//! reader.read_to_string(&mut content).unwrap();
//! # }
//! ```
//!
//! ## [`file_open_write`]
//!
//! Similar to [`file_open_read`] this functions is a convenience wrapper but for writing
//! compressed files. The function always requires an argument as the filetype has to be specified.
//! Using [`Default::default`] is an option and will write a plaintext file.
//!
//!
//! ```no_run
//! # extern crate misc_utils;
//! # use misc_utils::fs::file_open_write;
//! #
//! # fn main() {
//! let mut writer = file_open_write("./text.txt", Default::default()).unwrap();
//! writer.write_all("Hello World".as_bytes()).unwrap();
//! # }
//! ```
//!
//! ## [`parse_jsonl_multi_threaded`]
//!
//! Create multiple thread reading and parsing a [JSONL] file.
//!
//! This function is especially usefull if the file is compressed with a high compression (such as
//! xz2) and the parsing overhead is non-negligible. The inter-thread communication is batched to
//! reduce overhead.
//!
//! [`file_open_read`]: crate::fs::file_open_read
//! [`file_open_read_with_options`]: crate::fs::file_open_read_with_options
//! [`file_open_write`]: crate::fs::file_open_write
//! [`parse_jsonl_multi_threaded`]: crate::fs::parse_jsonl_multi_threaded
//!
//! [JSONL]: http://jsonlines.org/

#[cfg(feature = "jsonl")]
use crate::error::MtJsonlError;
use crate::error::NotAFileError;
use anyhow::{Context, Error};
#[cfg(feature = "file-bz2")]
use bzip2::{self, bufread::BzDecoder, write::BzEncoder};
#[cfg(feature = "file-gz")]
use flate2::{self, bufread::MultiGzDecoder, write::GzEncoder};
use log::debug;
#[cfg(feature = "jsonl")]
use log::{info, warn};
#[cfg(feature = "jsonl")]
use serde::de::DeserializeOwned;
#[cfg(feature = "jsonl")]
use serde_json::Deserializer;
use std::{
    borrow::Borrow,
    ffi::OsStr,
    fs::OpenOptions,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};
#[cfg(feature = "jsonl")]
use std::{io::BufRead, path::PathBuf, sync::mpsc, thread};
#[cfg(feature = "file-xz")]
use xz2::{
    bufread::XzDecoder,
    stream::{Check, MtStreamBuilder},
    write::XzEncoder,
};

/// Configure behaviour of the [`file_open_read_with_options`] function.
///
/// [`file_open_read_with_options`]: ./fn.file_open_read_with_options.html
#[derive(Clone, Debug)]
pub struct ReadOptions {
    buffer_capacity: Option<usize>,
    open_options: OpenOptions,
}

impl ReadOptions {
    /// Create a new `ReadOptions` with default settings.
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

    /// Specify a set of [`OpenOptions`] to use.
    ///
    /// The option `read` will always be overwritten to `true` and `write` will always be set to
    /// `false`.
    ///
    /// [`OpenOptions`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html
    pub fn set_open_options<B>(mut self, open_options: B) -> Self
    where
        B: Borrow<OpenOptions>,
    {
        self.open_options = open_options.borrow().clone();
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

/// Create reader for uncompressed or compressed files transparently.
///
/// See [`file_open_read_with_options`] for the full documentation.
///
/// [`file_open_read_with_options`]: ./fn.file_open_read_with_options.html
pub fn file_open_read<P>(file: P) -> Result<Box<dyn Read>, Error>
where
    P: AsRef<Path>,
{
    file_open_read_with_option_do(file.as_ref(), ReadOptions::default())
}

/// Create reader for uncompressed or compressed files transparently.
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
pub fn file_open_read_with_options<P>(file: P, options: ReadOptions) -> Result<Box<dyn Read>, Error>
where
    P: AsRef<Path>,
{
    file_open_read_with_option_do(file.as_ref(), options)
}

fn file_open_read_with_option_do(
    file: &Path,
    mut options: ReadOptions,
) -> Result<Box<dyn Read>, Error> {
    if !file.is_file() {
        return Err(NotAFileError::new(file).into());
    }

    let f = options
        .open_options
        .read(true)
        .write(false)
        .open(file)
        .context(format!("Could not open file {}", file.display()))?;
    let mut bufread = if let Some(size) = options.buffer_capacity {
        BufReader::with_capacity(size, f)
    } else {
        BufReader::new(f)
    };

    // read magic bytes
    let mut buffer = [0; 6];
    if bufread.read_exact(&mut buffer).is_err() {
        // reset buffer into a valid state
        // this will trigger the plaintext case below

        // The allow is only needed if compiled with --no-default-features
        // because all the if branches below will be removed
        #[allow(unused_assignments)]
        {
            buffer = [0; 6];
        }
    };
    // reset the read position
    bufread
        .seek(SeekFrom::Start(0))
        .context("Failed to seek to start of file.")?;

    #[cfg(feature = "file-xz")]
    {
        // check if file if XZ compressed
        if buffer[..6] == [0xfd, b'7', b'z', b'X', b'Z', 0x00] {
            debug!("File {} is detected to have type `xz`", file.display());
            return Ok(Box::new(XzDecoder::new(bufread)));
        }
    }
    #[cfg(feature = "file-gz")]
    {
        if buffer[..2] == [0x1f, 0x8b] {
            debug!("File {} is detected to have type `gz`", file.display());
            return Ok(Box::new(MultiGzDecoder::new(bufread)));
        }
    }
    #[cfg(feature = "file-bz2")]
    {
        if buffer[..3] == [b'B', b'Z', b'h'] {
            debug!("File {} is detected to have type `bz2`", file.display());
            return Ok(Box::new(BzDecoder::new(bufread)));
        }
    }

    debug!("Open file {} as plaintext", file.display());
    Ok(Box::new(bufread))
}

/// Configure behaviour of the [`file_open_write`] function.
///
/// # Defaults
///
/// ```text
/// WriteOptions {
///     buffer_capacity: None,
///     compression_level: Compression::Default,
///     filetype: FileType::PlainText,
///     open_options: OpenOptions::new(),
///     threads: 1,
/// };
/// ```
///
/// [`file_open_write`]: ./fn.file_open_write.html
#[derive(Clone, Debug)]
pub struct WriteOptions {
    buffer_capacity: Option<usize>,
    compression_level: Compression,
    filetype: FileType,
    open_options: OpenOptions,
    threads: u32,
}

impl WriteOptions {
    /// Create a new `WriteOptions` with default settings.
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

    /// Sets the compression level for archives.
    ///
    /// This configures the compression level used. This option has no effect if the [`FileType`]
    /// is `PlainText`. See [`Compression`] for a description of the possible values.
    ///
    /// [`Compression`]: ./enum.Compression.html
    /// [`FileType`]: ./enum.FileType.html
    pub fn set_compression_level(mut self, compression: Compression) -> Self {
        self.compression_level = compression;
        self
    }

    /// Sets the output filetype.
    ///
    /// This specifies if the file will be plaintext or which archive form will be used. See
    /// [`FileType`] for details on the possible values.
    ///
    /// [`FileType`]: ./enum.FileType.html
    pub fn set_filetype(mut self, ty: FileType) -> Self {
        self.filetype = ty;
        self
    }

    /// Specify a set of [`OpenOptions`] to use.
    ///
    /// The option `read` will always be overwritten to `false` and `write` will always be set to
    /// `true`.
    ///
    /// This allows to specify flags like `append` or `truncate` while writing.
    ///
    /// [`OpenOptions`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html
    pub fn set_open_options<B>(mut self, open_options: B) -> Self
    where
        B: Borrow<OpenOptions>,
    {
        self.open_options = open_options.borrow().clone();
        self
    }

    /// Specify the maximal number of threads used for compression.
    ///
    /// This gives a hint to the encoder that threading is wanted. This feature is currently only
    /// used with `xz`. The writer will the value of `threads` as a maximal number.
    ///
    /// Setting this option to `0` has the same effect as setting it to `1`.
    pub fn set_threads(mut self, mut threads: u32) -> Self {
        if threads == 0 {
            threads = 1;
        }
        self.threads = threads;
        self
    }
}

impl Default for WriteOptions {
    fn default() -> Self {
        let mut open_options = OpenOptions::new();
        open_options.create(true);
        Self {
            buffer_capacity: None,
            compression_level: Compression::default(),
            filetype: FileType::default(),
            open_options,
            threads: 1,
        }
    }
}

impl PartialEq for WriteOptions {
    fn eq(&self, other: &Self) -> bool {
        self.buffer_capacity == other.buffer_capacity
            && self.compression_level == other.compression_level
            && self.filetype == other.filetype
            && self.threads == other.threads
    }
}

impl Eq for WriteOptions {}

/// Specify the output filetype.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum FileType {
    /// Create a `bz2` compressed archive.
    #[cfg(feature = "file-bz2")]
    Bz2,
    /// Create a `gz` compressed archive.
    #[cfg(feature = "file-gz")]
    Gz,
    /// Create a plaintext file.<br />
    /// This is the default variant.
    PlainText,
    /// Create a `xz` compressed archive.
    #[cfg(feature = "file-xz")]
    Xz,
}

impl Default for FileType {
    /// Returns the `PlainText` variant.
    fn default() -> Self {
        FileType::PlainText
    }
}

/// Specify the compression level used.
///
/// There are three presets provided, `Fastest`, `Default`, and `Best`. They correspond to the
/// settings in `bzip2`, `gzip`, and `xz`. `Default` corresponds to the value 6.
///
/// For `bzip2` and `gzip` the `Numeric` values are mapped as follows:
///
/// |      Numeric |              bzip2 |      gzip |
/// | -----------: | -----------------: | --------: |
/// |            0 | `<No compression>` | `Fastest` |
/// |            1 |          `Fastest` | `Fastest` |
/// |            2 |          `Fastest` | `Fastest` |
/// |            3 |          `Fastest` | `Fastest` |
/// |            4 |          `Default` | `Default` |
/// |            5 |          `Default` | `Default` |
/// |            6 |          `Default` | `Default` |
/// |            7 |             `Best` |    `Best` |
/// |            8 |             `Best` |    `Best` |
/// |            9 |             `Best` |    `Best` |
/// | other values |             `Best` |    `Best` |
///
/// For `xz` `Numeric` values in the range `0-9` (inclusive) are valid. The named variants are
/// mapped to `0` for `Fastest`, `6` for `Default`, and `9` for `Best`.
///
/// Be aware that the result in compression ratio and time/memory consumption is highly dependent
/// on the chosen filetype.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Compression {
    /// Provide the fastest compression possible.
    Fastest,
    /// Use a reasonable default, which does not consume too much CPU/memory.
    Default,
    /// Provide the best compression possible.
    Best,
    /// Fine-grained controll over the compression for the `xz` algorithm. Allowed values are `0-9`.
    Numeric(u8),
}

impl Default for Compression {
    /// Returns the `Default` variant.
    fn default() -> Self {
        Compression::Default
    }
}

#[cfg(feature = "file-bz2")]
impl Into<bzip2::Compression> for Compression {
    fn into(self) -> bzip2::Compression {
        use bzip2::Compression as BzipCompression;

        match self {
            Compression::Fastest => BzipCompression::fast(),
            Compression::Default => BzipCompression::default(),
            Compression::Best => BzipCompression::best(),
            Compression::Numeric(n) => BzipCompression::new(clamp(u32::from(n), 0, 9)),
        }
    }
}

#[cfg(feature = "file-gz")]
impl Into<flate2::Compression> for Compression {
    fn into(self) -> flate2::Compression {
        use flate2::Compression as FlateCompression;

        match self {
            Compression::Fastest => FlateCompression::fast(),
            Compression::Default => FlateCompression::default(),
            Compression::Best => FlateCompression::best(),
            Compression::Numeric(n) => FlateCompression::new(clamp(u32::from(n), 0, 9)),
        }
    }
}

/// Implementation detail to convert a [`Compression`] into a `u32` in the range `0-9` (inclusive).
///
/// [`Compression`]: ./enum.Compression.html
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
struct XzCompression(u32);
impl Into<XzCompression> for Compression {
    fn into(self) -> XzCompression {
        match self {
            Compression::Fastest => XzCompression(0),
            Compression::Default => XzCompression(6),
            Compression::Best => XzCompression(9),
            Compression::Numeric(n) => XzCompression(clamp(u32::from(n), 0, 9)),
        }
    }
}

// TODO consider using num for this
// https://docs.rs/num/0.1.40/num/fn.clamp.html
fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

/// Create writers for plaintext or compressed files.
///
/// This function can open a file with different compressors enabled. It hides the complexity of
/// creating the correct writer behind the [`WriteOptions`] builder. See the documentation on the
/// struct for more details.
///
/// File I/O will always be buffered using a [`BufReader`].
///
/// Flushing the writer will not write all the data to file. Archives require some finalizer which
/// is only written if the writer is being dropped.
pub fn file_open_write<P>(file: P, mut options: WriteOptions) -> Result<Box<dyn Write>, Error>
where
    P: AsRef<Path>,
{
    use self::FileType::*;
    let file = file.as_ref();

    let f = options
        .open_options
        .read(false)
        .write(true)
        .open(file)
        .context(format!("Could not open file {}", file.display()))?;
    let bufwrite = if let Some(size) = options.buffer_capacity {
        BufWriter::with_capacity(size, f)
    } else {
        BufWriter::new(f)
    };

    match options.filetype {
        #[cfg(feature = "file-bz2")]
        Bz2 => {
            let level = options.compression_level.into();
            Ok(Box::new(BzEncoder::new(bufwrite, level)))
        }
        #[cfg(feature = "file-gz")]
        Gz => {
            let level = options.compression_level.into();
            Ok(Box::new(GzEncoder::new(bufwrite, level)))
        }
        PlainText => Ok(Box::new(bufwrite)),
        #[cfg(feature = "file-xz")]
        Xz => {
            let level: XzCompression = options.compression_level.into();
            let threads = clamp(options.threads, 1, u32::max_value());
            if threads == 1 {
                Ok(Box::new(XzEncoder::new(bufwrite, level.0)))
            } else {
                let stream = MtStreamBuilder::new()
                    .preset(level.0)
                    .threads(threads)
                    // let LZMA2 choose the best blocksize
                    .block_size(0)
                    // use the same value as the xz command line tool
                    .timeout_ms(300)
                    .check(Check::Crc64)
                    .encoder()
                    .context("Failed to initialize the xz multithreaded stream")?;
                Ok(Box::new(XzEncoder::new_stream(bufwrite, stream)))
            }
        }
    }
}

/// Result type for [`parse_jsonl_multi_threaded`].
///
/// This enum encapsulates certain error conditions which can occur either during file I/O or JSON
/// parsing and the data produced by it. Every user of the [`parse_jsonl_multi_threaded`] **must**
/// verify that the last element of the iteration is the `Complete` variant, to ensure the whole
/// file has been read and all lines could successfully parsed.
#[cfg(feature = "jsonl")]
#[derive(Debug)]
enum ProcessingStatus<T>
where
    T: 'static + Send,
{
    /// Indicates a successful completion of all steps.
    /// Every user **must** verify that this element occurs during iteration.
    Completed,
    /// Wrapper for any user-defined datatype
    Data(T),
    Error(MtJsonlError),
}

/// An iterator over deserialized JSON objects
///
/// This struct is created by the [`parse_jsonl_multi_threaded`] function.
#[cfg(feature = "jsonl")]
#[derive(Debug)]
pub struct MtJsonl<T>
where
    T: 'static + DeserializeOwned + Send,
{
    iter: mpsc::IntoIter<ProcessingStatus<Vec<Result<T, MtJsonlError>>>>,
    tmp_state: ::std::vec::IntoIter<Result<T, MtJsonlError>>,
    did_complete: bool,
    file: PathBuf,
}

#[cfg(feature = "jsonl")]
impl<T> MtJsonl<T>
where
    T: 'static + DeserializeOwned + Send,
{
    fn new<F>(iter: mpsc::IntoIter<ProcessingStatus<Vec<Result<T, MtJsonlError>>>>, file: F) -> Self
    where
        F: AsRef<Path>,
    {
        Self {
            iter,
            tmp_state: vec![].into_iter(),
            did_complete: false,
            file: file.as_ref().to_path_buf(),
        }
    }
}

#[cfg(feature = "jsonl")]
impl<T> Iterator for MtJsonl<T>
where
    T: 'static + DeserializeOwned + Send,
{
    type Item = Result<T, MtJsonlError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(res) = self.tmp_state.next() {
                return Some(match res {
                    Ok(x) => Ok(x),
                    Err(err) => {
                        info!("{:?}", err);
                        Err(err)
                    }
                });
            } else if let Some(state) = self.iter.next() {
                match state {
                    ProcessingStatus::Data(data) => self.tmp_state = data.into_iter(),
                    ProcessingStatus::Completed => self.did_complete = true,
                    // path through error
                    ProcessingStatus::Error(err) => return Some(Err(err)),
                }
                continue;
            }

            // No more data to read from underlying iterators
            return if self.did_complete {
                None
            } else {
                Some(Err(MtJsonlError::NotCompleted))
            };
        }
    }
}

/// Create a multi-threaded [JSONL] parser.
///
/// This returns an iterator over `Result<T>`. If any reading errors of the file or parsing errors
/// happen they will be passed to the caller of the iterator.
///
/// Internally this will spawn two threads. The first thread is responsible for reading from the
/// underlying file. It uses [`file_open_read`] for this task, thus it also supports compressed
/// files transparently. The second thread receives multiple lines as [`String`] and parses them
/// into a `Vec<Result<T>>`. Then they are passed to the caller as a single iterator.
///
/// Since the processing is based on thread the communication overhead should be minimal. For this
/// the `batchsize` can be specifies, which controls how many lines are read before passing them to
/// the second thread and thus how large the vector will be.
///
/// [JSONL]: http://jsonlines.org/
#[cfg(feature = "jsonl")]
pub fn parse_jsonl_multi_threaded<P, T>(path: P, batchsize: u32) -> MtJsonl<T>
where
    P: AsRef<Path>,
    T: 'static + DeserializeOwned + Send,
{
    let path = path.as_ref().to_path_buf();
    let path_ = path.clone();
    const CHAN_BUFSIZE: usize = 2;

    // create channels
    let (lines_sender, lines_receiver) = mpsc::sync_channel(CHAN_BUFSIZE);
    #[allow(clippy::type_complexity)]
    let (struct_sender, struct_receiver): (
        _,
        mpsc::Receiver<ProcessingStatus<Vec<Result<T, MtJsonlError>>>>,
    ) = mpsc::sync_channel(CHAN_BUFSIZE);

    // spawn reader thread of file
    thread::spawn(move || {
        info!(
            "Start background reading thread: {:?}",
            thread::current().id()
        );
        let mut rdr = match file_open_read(&path) {
            Ok(rdr) => BufReader::new(rdr),
            Err(err) => {
                warn!(
                    "Background reading thread cannot open file {} {:?}",
                    path.display(),
                    thread::current().id()
                );
                // cannot communicate channel failures
                let _ = lines_sender.send(ProcessingStatus::Error(MtJsonlError::IoError {
                    msg: "Background reading thread cannot open file".to_string(),
                    file: path.to_path_buf(),
                    source: err,
                }));
                return;
            }
        };
        let mut is_eof = false;
        while !is_eof {
            let mut batch = String::new();
            for _ in 0..batchsize {
                match rdr.read_line(&mut batch) {
                    Ok(0) => {
                        is_eof = true;
                        break;
                    }
                    Ok(_) => {}
                    Err(err) => {
                        warn!(
                            "Background reading thread cannot read line {:?}",
                            thread::current().id()
                        );
                        // cannot communicate channel failures
                        let _ = lines_sender.send(ProcessingStatus::Error(MtJsonlError::IoError {
                            msg: "Background reading thread cannot read line".to_string(),
                            file: path.to_path_buf(),
                            source: err.into(),
                        }));
                        return;
                    }
                }
            }
            // cannot communicate channel failures
            if lines_sender.send(ProcessingStatus::Data(batch)).is_err() {
                // kill on sent error
                return;
            }
            info!(
                "Background reading thread: sent batch {:?}",
                thread::current().id()
            );
        }
        // cannot communicate channel failures
        let _ = lines_sender.send(ProcessingStatus::Completed);
        info!(
            "Background reading thread: successful processed file {:?} {:?}",
            path,
            thread::current().id()
        );
    });

    // spawn JSONL parser
    thread::spawn(move || {
        info!(
            "Start background parsing thread {:?}",
            thread::current().id()
        );
        let mut channel_successful_completed = false;
        lines_receiver.iter().for_each(|batch| {
            match batch {
                ProcessingStatus::Error(e) => {
                    info!(
                        "Background parsing thread: pass through error {:?}",
                        thread::current().id()
                    );
                    // cannot communicate channel failures
                    let _ = struct_sender.send(ProcessingStatus::Error(e));
                }
                // not the success status for future use
                ProcessingStatus::Completed => channel_successful_completed = true,
                ProcessingStatus::Data(batch) => {
                    let batch: Vec<Result<T, MtJsonlError>> = Deserializer::from_str(&*batch)
                        .into_iter()
                        .map(|v| v.map_err(|err| MtJsonlError::ParsingError { source: err }))
                        .collect();

                    info!(
                        "Background parsing thread: batch parsed {:?}",
                        thread::current().id()
                    );
                    // cannot communicate channel failures
                    if struct_sender.send(ProcessingStatus::Data(batch)).is_err() {
                        warn!(
                            "Background parsing thread: sent channel error {:?}",
                            thread::current().id()
                        );
                        // kill on send error
                        return;
                    }
                }
            }
        });
        if channel_successful_completed {
            info!(
                "Background parsing thread: successfully completed {:?}",
                thread::current().id()
            );
            if struct_sender.send(ProcessingStatus::Completed).is_err() {
                warn!(
                    "Background parsing thread: sent channel error {:?}",
                    thread::current().id()
                );
                // kill on send error
                return;
            }
        } else {
            warn!(
                "Background parsing thread: did not receive complete message from underlying reader {:?}",
                thread::current().id()
            );
        }
    });

    MtJsonl::new(struct_receiver.into_iter(), path_)
}

/// Read the entire contents of a file into a bytes vector.
///
/// This function supports opening compressed files transparently.
///
/// The API mirrors the function in [`std::fs::read`] except for the error type.
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::new();
    let mut reader = file_open_read_with_option_do(path.as_ref(), ReadOptions::default())?;
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Read the entire contents of a file into a string.
///
/// This function supports opening compressed files transparently.
///
/// The API mirrors the function in [`std::fs::read_to_string`] except for the error type.
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let mut buffer = String::new();
    let mut reader = file_open_read_with_option_do(path.as_ref(), ReadOptions::default())?;
    reader.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Write a slice as the entire contents of a file.
///
/// The functions chooses the filetype based on the extension.
/// If a recognized extension is used the file will be compressed otherwise the file will be written as plaintext.
/// If compression is used, it will use the default (6) compression ratio.
/// The method will truncate the file before writing, such that `contents` will be the only content of the file.
///
/// The API mirrors the function in [`std::fs::write`] except for the error type.

// Required for no-default-features
#[allow(clippy::match_single_binding)]
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), Error> {
    let path = path.as_ref();
    let mut options = WriteOptions::default();
    options.open_options.truncate(true);
    options = match path.extension().and_then(OsStr::to_str) {
        #[cfg(feature = "file-xz")]
        Some("xz") => options
            .set_filetype(FileType::Xz)
            .set_compression_level(Compression::Default),
        #[cfg(feature = "file-gz")]
        Some("gzip") | Some("gz") => options
            .set_filetype(FileType::Gz)
            .set_compression_level(Compression::Default),
        #[cfg(feature = "file-bz2")]
        Some("bzip") | Some("bz2") => options
            .set_filetype(FileType::Bz2)
            .set_compression_level(Compression::Default),
        _ => options.set_filetype(FileType::PlainText),
    };

    let mut writer = file_open_write(path, options)?;
    writer.write_all(contents.as_ref())?;
    writer.flush()?;
    drop(writer);
    Ok(())
}
