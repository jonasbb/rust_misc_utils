use bzip2;
use bzip2::bufread::BzDecoder;
use bzip2::write::BzEncoder;
use error::*;
use flate2;
use flate2::bufread::MultiGzDecoder;
use flate2::write::GzEncoder;
use std::borrow::Borrow;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, SeekFrom};
use std::path::Path;
use xz2::bufread::XzDecoder;
use xz2::stream::{Check, MtStreamBuilder};
use xz2::write::XzEncoder;

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
pub fn file_open_read<P>(file: P) -> Result<Box<Read>>
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
            open_options: open_options,
            threads: 1,
        }
    }
}

impl PartialEq for WriteOptions {
    fn eq(&self, other: &Self) -> bool {
        self.buffer_capacity == other.buffer_capacity &&
            self.compression_level == other.compression_level &&
            self.filetype == other.filetype && self.threads == other.threads
    }
}

impl Eq for WriteOptions {}

/// Specify the output filetype.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum FileType {
    /// Create a `bz2` compressed archive.
    Bz2,
    /// Create a `gz` compressed archive.
    Gz,
    /// Create a plaintext file.<br />
    /// This is the default variant.
    PlainText,
    /// Create a `xz` compressed archive.
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

impl Into<bzip2::Compression> for Compression {
    fn into(self) -> bzip2::Compression {
        use bzip2::Compression::*;

        match self {
            Compression::Fastest => Fastest,
            Compression::Numeric(n) if n <= 3 => Fastest,
            Compression::Default => Default,
            Compression::Numeric(n) if 4 <= n && n <= 6 => Default,
            Compression::Best => Best,
            Compression::Numeric(n) if 7 <= n && n <= 9 => Best,

            // catchall for all value > 9
            Compression::Numeric(_) => Best,
        }
    }
}

impl Into<flate2::Compression> for Compression {
    fn into(self) -> flate2::Compression {
        use flate2::Compression::*;

        match self {
            Compression::Numeric(0) => None,
            Compression::Fastest => Fast,
            Compression::Numeric(n) if 1 <= n && n <= 3 => Fast,
            Compression::Default => Default,
            Compression::Numeric(n) if 4 <= n && n <= 6 => Default,
            Compression::Best => Best,
            Compression::Numeric(n) if 7 <= n && n <= 9 => Best,

            // catchall for all value > 9
            Compression::Numeric(_) => Best,
        }
    }
}

/// Implementation detail to convert a [`Compression`] into a `u32` in the range `0-9` (inclusive).
///
/// [`Compression`]: ./enum.Compression.html
struct XzCompression(u32);
impl Into<XzCompression> for Compression {
    fn into(self) -> XzCompression {
        match self {
            Compression::Fastest => XzCompression(0),
            Compression::Default => XzCompression(6),
            Compression::Best => XzCompression(9),
            Compression::Numeric(n) => XzCompression(clamp(n as u32, 0, 9)),
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
///
/// [`BufReader`]: https://doc.rust-lang.org/std/io/struct.BufReader.html
/// [`WriteOptions`]: ./struct.WriteOptions.html
pub fn file_open_write<P>(file: P, mut options: WriteOptions) -> Result<Box<Write>>
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
        .chain_err(|| format!("Could not open file {:?}", file))?;
    let bufwrite = if let Some(size) = options.buffer_capacity {
        BufWriter::with_capacity(size, f)
    } else {
        BufWriter::new(f)
    };

    match options.filetype {
        Bz2 => {
            let level = options.compression_level.into();
            Ok(Box::new(BzEncoder::new(bufwrite, level)))
        }
        Gz => {
            let level = options.compression_level.into();
            Ok(Box::new(GzEncoder::new(bufwrite, level)))
        }
        PlainText => Ok(Box::new(bufwrite)),
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
                    .chain_err(|| "Failed to initialize the xz multithreaded stream")?;
                Ok(Box::new(XzEncoder::new_stream(bufwrite, stream)))
            }
        }
    }
}
