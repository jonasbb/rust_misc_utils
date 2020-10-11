//! This module contains functions related to filesystem operations.
//!
//! ## [`file_open_read`] / [`file_open_read_with_capacity`]
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
//! ## [`file_write`]
//!
//! Similar to [`file_open_read`] this functions is a convenience wrapper but for writing
//! compressed files. The function always requires an argument as the filetype has to be specified.
//! The function detects the correct filetype from the file extension.
//! The detected choice can be overwritten using [`WriteBuilder::filetype`].
//!
//! There are two modes the file can be opened, in either the [`truncate`] or the [`append`] mode.
//!
//! ```no_run
//! # use misc_utils::fs::file_write;
//! #
//! # fn main() -> Result<(), anyhow::Error> {
//! let mut writer = file_write("./text.txt").truncate()?;
//! writer.write_all("Hello World".as_bytes())?;
//! # Ok(())
//! # }
//! ```
//!
//! ```no_run
//! # use misc_utils::fs::file_write;
//! #
//! # fn main() -> Result<(), anyhow::Error> {
//! let mut writer = file_write("./text.txt").append()?;
//! writer.write_all("Hello World".as_bytes())?;
//! # Ok(())
//! # }
//! ```
//!
//! ## [`parse_jsonl_multi_threaded`]
//!
//! Create multiple thread reading and parsing a [JSONL] file.
//!
//! This function is especially useful if the file is compressed with a high compression (such as
//! xz2) and the parsing overhead is non-negligible. The inter-thread communication is batched to
//! reduce overhead.
//!
//! [`append`]: WriteBuilder::append
//! [`truncate`]: WriteBuilder::truncate
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
    ffi::OsStr,
    fs::OpenOptions,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
#[cfg(feature = "jsonl")]
use std::{io::BufRead, sync::mpsc, thread};
#[cfg(feature = "file-xz")]
use xz2::{
    bufread::XzDecoder,
    stream::{Check, MtStreamBuilder},
    write::XzEncoder,
};

// This is unused when --all-features is given
#[allow(unused_imports)]
use anyhow::bail;

/// Create reader for uncompressed or compressed files transparently.
///
/// This function opens the given `file` and tries to determine the filetype by reading the magic
/// bytes from the start of the file. If a known archive type, like xz, gz, or bz2, is found this
/// function will transparent create a reader which decompresses the data on the fly.
///
/// File I/O will always be buffered using a [`BufReader`].
/// You can use [`file_open_read_with_capacity`] to specify the buffer size.
///
/// [`BufReader`]: https://doc.rust-lang.org/std/io/struct.BufReader.html
pub fn file_open_read<P>(file: P) -> Result<Box<dyn Read>, Error>
where
    P: AsRef<Path>,
{
    do_file_open_read(file.as_ref(), None)
}

/// Create reader for uncompressed or compressed files transparently.
///
/// This function opens the given `file` and tries to determine the filetype by reading the magic
/// bytes from the start of the file. If a known archive type, like xz, gz, or bz2, is found this
/// function will transparent create a reader which decompresses the data on the fly.
///
/// File I/O will always be buffered using a [`BufReader`].
/// The `buffer_capacity` argument specifies the capacity of the [`BufReader`] in bytes.
///
/// [`BufReader`]: https://doc.rust-lang.org/std/io/struct.BufReader.html
pub fn file_open_read_with_capacity<P>(
    file: P,
    buffer_capacity: usize,
) -> Result<Box<dyn Read>, Error>
where
    P: AsRef<Path>,
{
    do_file_open_read(file.as_ref(), Some(buffer_capacity))
}

fn do_file_open_read(file: &Path, buffer_capacity: Option<usize>) -> Result<Box<dyn Read>, Error> {
    if !file.is_file() {
        return Err(NotAFileError::new(file).into());
    }

    let f = OpenOptions::new()
        .create(false)
        .read(true)
        .write(false)
        .open(file)
        .context(format!("Could not open file {}", file.display()))?;
    let mut bufread = if let Some(size) = buffer_capacity {
        BufReader::with_capacity(size, f)
    } else {
        BufReader::new(f)
    };

    // read magic bytes
    let mut buffer = [0; 6];
    if bufread.read_exact(&mut buffer).is_err() {
        // reset buffer into a valid state
        // this will trigger the plaintext case below
        buffer = [0; 6];
    };
    // reset the read position
    bufread
        .seek(SeekFrom::Start(0))
        .context("Failed to seek to start of file.")?;

    if buffer[..6] == [0xfd, b'7', b'z', b'X', b'Z', 0x00] {
        debug!("File {} is detected to have type `xz`", file.display());
        #[cfg(feature = "file-xz")]
        return Ok(Box::new(XzDecoder::new(bufread)));
        #[cfg(not(feature = "file-xz"))]
        bail!(
            "File {} is detected to have type `xz`, but the file-xz feature is not enabled.",
            file.display()
        );
    }
    if buffer[..2] == [0x1f, 0x8b] {
        debug!("File {} is detected to have type `gz`", file.display());
        #[cfg(feature = "file-gz")]
        return Ok(Box::new(MultiGzDecoder::new(bufread)));
        #[cfg(not(feature = "file-gz"))]
        bail!(
            "File {} is detected to have type `gz`, but the file-gz feature is not enabled.",
            file.display()
        );
    }
    if buffer[..3] == [b'B', b'Z', b'h'] {
        debug!("File {} is detected to have type `bz2`", file.display());
        #[cfg(feature = "file-bz2")]
        return Ok(Box::new(BzDecoder::new(bufread)));
        #[cfg(not(feature = "file-bz2"))]
        bail!(
            "File {} is detected to have type `bz2`, but the file-bz2 feature is not enabled.",
            file.display()
        );
    }

    debug!("Open file {} as plaintext", file.display());
    Ok(Box::new(bufread))
}

/// Specify the output filetype.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum FileType {
    /// Create a `bz2` compressed archive.
    #[cfg(feature = "file-bz2")]
    Bz2,
    /// Create a `gz` compressed archive.
    #[cfg(feature = "file-gz")]
    Gz,
    /// Create a plaintext file (default).
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
    /// Fine-grained control over the compression for the `xz` algorithm. Allowed values are `0-9`.
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

/// Builder to control how the writeable file will be opened.
#[derive(Debug)]
pub struct WriteBuilder {
    /// Controls the buffer size of the [`BufWriter`].
    buffer_capacity: Option<usize>,
    /// Compression level of the file.
    ///
    /// Ignored for [`FileType::PlainText`].
    compression_level: Compression,
    /// FileType of the new file.
    ///
    /// The filetype is guessed from the file extensions using [`guess_file_type`].
    filetype: Option<FileType>,
    /// Path where the file will be written.
    path: PathBuf,
    /// Controls how the file will be opened.
    open_options: OpenOptions,
    /// Number of threads used during compression.
    ///
    /// Ignored for [`FileType::PlainText`].
    threads: u8,
}

impl WriteBuilder {
    /// Create a new [`WriteBuilder`] for a given path.
    ///
    /// See the individual methods for the available configuration options.
    pub fn new(path: PathBuf) -> Self {
        let mut open_options = OpenOptions::new();
        open_options.read(false).write(true);

        WriteBuilder {
            path,
            filetype: None,
            open_options,

            buffer_capacity: Default::default(),
            compression_level: Default::default(),
            threads: 1,
        }
    }

    /// Open the file in *append* mode.
    pub fn append(&mut self) -> Result<Box<dyn Write>, Error> {
        self.open_options.append(true);
        self.open()
    }

    /// Open the file in *truncate* mode.
    ///
    pub fn truncate(&mut self) -> Result<Box<dyn Write>, Error> {
        self.open_options.truncate(true);
        self.open()
    }

    fn open(&mut self) -> Result<Box<dyn Write>, Error> {
        use self::FileType::*;

        if self.filetype.is_none() {
            self.filetype = Some(guess_file_type(&self.path)?);
        }

        let file = self
            .open_options
            .open(&self.path)
            .context(format!("Could not open file {}", self.path.display()))?;
        let bufwrite = if let Some(size) = self.buffer_capacity {
            BufWriter::with_capacity(size, file)
        } else {
            BufWriter::new(file)
        };

        match self
            .filetype
            .expect("FileType is set based on extension if it was None")
        {
            #[cfg(feature = "file-bz2")]
            Bz2 => {
                let level = self.compression_level.into();
                Ok(Box::new(BzEncoder::new(bufwrite, level)))
            }
            #[cfg(feature = "file-gz")]
            Gz => {
                let level = self.compression_level.into();
                Ok(Box::new(GzEncoder::new(bufwrite, level)))
            }
            PlainText => Ok(Box::new(bufwrite)),
            #[cfg(feature = "file-xz")]
            Xz => {
                let level: XzCompression = self.compression_level.into();
                let threads = clamp(self.threads, 1, u8::max_value());
                if threads == 1 {
                    Ok(Box::new(XzEncoder::new(bufwrite, level.0)))
                } else {
                    let stream = MtStreamBuilder::new()
                        .preset(level.0)
                        .threads(u32::from(threads))
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

    /// Sets the capacity of the [`BufWriter`] to `capacity` in Bytes.
    pub fn buffer_capacity(&mut self, buffer_capacity: usize) -> &mut Self {
        self.buffer_capacity = Some(buffer_capacity);
        self
    }

    /// Sets the option to create a new file, or open it if it already exists.
    ///
    /// This function is analogue to [`std::fs::OpenOptions::create`].
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.open_options.create(create);
        self
    }

    /// Sets the option to create a new file, failing if it already exists.
    ///
    /// This function is analogue to [`std::fs::OpenOptions::create_new`].
    ///
    /// No file is allowed to exist at the target location, also no (dangling) symlink. In this way, if the call succeeds, the file returned is guaranteed to be new.
    ///
    /// This option is useful because it is atomic. Otherwise between checking whether a file exists and creating a new one, the file may have been created by another process (a TOCTOU race condition / attack).
    ///
    /// If .create_new(true) is set, [`create()`] and [`truncate()`] are ignored.
    ///
    /// [`create()`]: Self::create
    /// [`truncate()`]: Self::truncate
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.open_options.create_new(create_new);
        self
    }

    /// Sets the compression level for archives.
    ///
    /// This configures the compression level used. This option has no effect for [`FileType::PlainText`].
    /// See [`Compression`] for a description of the possible values.
    pub fn compression_level(&mut self, compression_level: Compression) -> &mut Self {
        self.compression_level = compression_level;
        self
    }

    /// Sets the output filetype.
    ///
    /// This can be used to overwrite the automatically detected filetype.
    pub fn filetype(&mut self, filetype: FileType) -> &mut Self {
        self.filetype = Some(filetype);
        self
    }

    /// Specify the maximal number of threads used for compression.
    ///
    /// This gives a hint to the encoder that threading is wanted. This feature is currently only used with `xz`.
    /// The writer will use this value as a maximal number.
    ///
    /// Setting this value to `0` has the same effect as setting it to `1`.
    pub fn threads(&mut self, threads: u8) -> &mut Self {
        self.threads = if threads == 0 { 1 } else { threads };
        self
    }
}

/// Create writers for plaintext or compressed files.
///
/// This function can open a file with different compressors enabled.
/// The options to open and write the file can be controlled with the [`WriteBuilder`].
/// See the documentation on that type for more details.
/// The filetype will be guessed from the extension.
/// The guessing can be disabled by explicitly setting a filetype using [`WriteBuilder::filetype`].
///
/// File I/O will always be buffered using a [`BufReader`].
///
/// Flushing the writer will not write all the data to file.
/// Archives require some finalizer which is only written if the writer is being dropped.
pub fn file_write<P>(path: P) -> WriteBuilder
where
    P: AsRef<Path>,
{
    WriteBuilder::new(path.as_ref().to_path_buf())
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
    let mut reader = file_open_read(path.as_ref())?;
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
    let mut reader = file_open_read(path.as_ref())?;
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

    let mut writer = file_write(path).truncate()?;
    writer.write_all(contents.as_ref())?;
    writer.flush()?;
    drop(writer);
    Ok(())
}

/// Append the content to the file.
///
/// This function only works for plaintext and gzip files.
// Required for no-default-features
#[allow(clippy::match_single_binding)]
pub fn append<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), Error> {
    let path = path.as_ref();
    let mut writer = file_write(path).append()?;
    writer.write_all(contents.as_ref())?;
    writer.flush()?;
    drop(writer);
    Ok(())
}

/// Guess the [`FileType`] from the path extension
///
/// The function will error if a compressed extension is recognized but the corresponding `file-*` feature is not enabled.
/// The function falls back to [`FileType::PlainText`] if the extension is not recognized.
fn guess_file_type(path: &Path) -> Result<FileType, Error> {
    Ok(match path.extension().and_then(OsStr::to_str) {
        Some("xz") => {
            #[cfg(feature = "file-xz")]
            {
                FileType::Xz
            }
            #[cfg(not(feature = "file-xz"))]
            {
                bail!(
                    "Writing to a file {} with detected type `xz`, but the file-xz feature is not enabled.",
                    path.display()
                )
            }
        }

        Some("gzip") | Some("gz") => {
            #[cfg(feature = "file-gz")]
            {
                FileType::Gz
            }
            #[cfg(not(feature = "file-gz"))]
            {
                bail!(
                    "Writing to a file {} with detected type `gz`, but the file-gz feature is not enabled.",
                    path.display()
                )
            }
        }

        Some("bzip") | Some("bz2") => {
            #[cfg(feature = "file-bz2")]
            {
                FileType::Bz2
            }
            #[cfg(not(feature = "file-bz2"))]
            {
                bail!(
                    "Writing to a file {} with detected type `bz2`, but the file-bz2 feature is not enabled.",
                    path.display()
                )
            }
        }

        _ => FileType::PlainText,
    })
}
