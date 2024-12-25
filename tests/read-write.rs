use anyhow::Error;
use misc_utils::byteascii::ByteAscii;
#[cfg(any(feature = "file-gz", feature = "file-xz", feature = "file-bz2"))]
use misc_utils::fs::Compression;
use misc_utils::fs::{self, file_open_read, file_write};
use pretty_assertions::assert_eq;
use std::{fs::File, io::prelude::*, path::Path};
use tempfile::Builder;

const LOREM_IPSUM: &str = r#"Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod
tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At
vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren,
no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit
amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut
labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam
et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata
sanctus est Lorem ipsum dolor sit amet."#;

#[track_caller]
fn assert_file_eq(expected_file: &Path, actual_file: &Path) -> Result<(), Error> {
    let mut expected_content = Vec::new();
    let mut actual_content = Vec::new();
    let mut expected_file = File::open(expected_file)?;
    let mut actual_file = File::open(actual_file)?;
    expected_file.read_to_end(&mut expected_content)?;
    actual_file.read_to_end(&mut actual_content)?;
    assert_eq!(ByteAscii(expected_content), ByteAscii(actual_content));
    Ok(())
}

#[track_caller]
fn do_read_test(expected: &str, actual_file: &Path) -> Result<(), Error> {
    let mut reader = file_open_read(Path::new(actual_file))?;
    let mut actual_content = String::new();
    reader.read_to_string(&mut actual_content)?;
    assert_eq!(expected, actual_content);
    Ok(())
}

#[track_caller]
fn do_write_test(
    expected_file: &Path,
    actual_file: &Path,
    mut writer: Box<dyn Write>,
) -> Result<(), Error> {
    writer.write_all(LOREM_IPSUM.as_bytes())?;
    // flush all data
    writer.flush()?;
    // finish archive creation
    drop(writer);

    assert_file_eq(expected_file, actual_file)
}

#[track_caller]
fn do_read_test_fs_bytes(expected: &str, actual_file: &Path) -> Result<(), Error> {
    let content = fs::read(actual_file)?;
    assert_eq!(content, expected.as_bytes());
    Ok(())
}

#[track_caller]
fn do_read_test_fs_string(expected: &str, actual_file: &Path) -> Result<(), Error> {
    let content = fs::read_to_string(actual_file)?;
    assert_eq!(content, expected);
    Ok(())
}

#[track_caller]
fn do_write_test_fs(file: &Path, suffix: &str) -> Result<(), Error> {
    let tmpfile = Builder::new().suffix(suffix).tempfile()?;
    fs::write(tmpfile.path(), LOREM_IPSUM)?;

    assert_file_eq(tmpfile.path(), file)
}

#[test]
fn test_read_empty_file() -> Result<(), Error> {
    do_read_test("", Path::new("./tests/data/empty.txt"))
}

#[test]
fn test_read_plaintext() -> Result<(), Error> {
    do_read_test(LOREM_IPSUM, Path::new("./tests/data/lorem.txt"))
}

#[cfg_attr(not(feature = "file-bz2"), ignore)]
#[test]
fn test_read_bz2() -> Result<(), Error> {
    do_read_test(LOREM_IPSUM, Path::new("./tests/data/lorem.txt.bz2"))
}

#[cfg_attr(not(feature = "file-gz"), ignore)]
#[test]
fn test_read_gz() -> Result<(), Error> {
    do_read_test(LOREM_IPSUM, Path::new("./tests/data/lorem.txt.gz"))
}

#[cfg_attr(not(feature = "file-xz"), ignore)]
#[test]
fn test_read_xz() -> Result<(), Error> {
    do_read_test(LOREM_IPSUM, Path::new("./tests/data/lorem.txt.xz"))
}

#[test]
fn test_write_plaintext() -> Result<(), Error> {
    let tmpfile = Builder::new().suffix(".txt").tempfile()?;
    let writer = file_write(tmpfile.path()).truncate()?;
    do_write_test(Path::new("./tests/data/lorem.txt"), tmpfile.path(), writer)
}

#[cfg(feature = "file-bz2")]
#[test]
fn test_write_bzip2() -> Result<(), Error> {
    let tmpfile = Builder::new().suffix(".bz2").tempfile()?;
    let writer = file_write(tmpfile.path())
        .compression_level(Compression::Best)
        .truncate()?;
    do_write_test(
        Path::new("./tests/data/lorem.txt.bz2"),
        tmpfile.path(),
        writer,
    )
}

#[cfg(feature = "file-gz")]
#[test]
fn test_write_gzip() -> Result<(), Error> {
    let tmpfile = Builder::new().suffix(".gz").tempfile()?;
    let writer = file_write(tmpfile.path())
        .compression_level(Compression::Best)
        .truncate()?;
    do_write_test(
        Path::new("./tests/data/lorem.txt.gz"),
        tmpfile.path(),
        writer,
    )
}

#[cfg(feature = "file-xz")]
#[test]
fn test_write_xz() -> Result<(), Error> {
    let tmpfile = Builder::new().suffix(".xz").tempfile()?;
    let writer = file_write(tmpfile.path())
        .compression_level(Compression::Best)
        .truncate()?;
    do_write_test(
        Path::new("./tests/data/lorem.txt.xz"),
        tmpfile.path(),
        writer,
    )
}

#[test]
fn test_read_empty_file_fs_bytes() -> Result<(), Error> {
    do_read_test_fs_bytes("", Path::new("./tests/data/empty.txt"))
}

#[test]
fn test_read_plaintext_fs_bytes() -> Result<(), Error> {
    do_read_test_fs_bytes(LOREM_IPSUM, Path::new("./tests/data/lorem.txt"))
}

#[cfg_attr(not(feature = "file-bz2"), ignore)]
#[test]
fn test_read_bz2_fs_bytes() -> Result<(), Error> {
    do_read_test_fs_bytes(LOREM_IPSUM, Path::new("./tests/data/lorem.txt.bz2"))
}

#[cfg_attr(not(feature = "file-gz"), ignore)]
#[test]
fn test_read_gz_fs_bytes() -> Result<(), Error> {
    do_read_test_fs_bytes(LOREM_IPSUM, Path::new("./tests/data/lorem.txt.gz"))
}

#[cfg_attr(not(feature = "file-xz"), ignore)]
#[test]
fn test_read_xz_fs_bytes() -> Result<(), Error> {
    do_read_test_fs_bytes(LOREM_IPSUM, Path::new("./tests/data/lorem.txt.xz"))
}

#[test]
fn test_read_empty_file_fs_string() -> Result<(), Error> {
    do_read_test_fs_string("", Path::new("./tests/data/empty.txt"))
}

#[test]
fn test_read_plaintext_fs_string() -> Result<(), Error> {
    do_read_test_fs_string(LOREM_IPSUM, Path::new("./tests/data/lorem.txt"))
}

#[cfg_attr(not(feature = "file-bz2"), ignore)]
#[test]
fn test_read_bz2_fs_string() -> Result<(), Error> {
    do_read_test_fs_string(LOREM_IPSUM, Path::new("./tests/data/lorem.txt.bz2"))
}

#[cfg_attr(not(feature = "file-gz"), ignore)]
#[test]
fn test_read_gz_fs_string() -> Result<(), Error> {
    do_read_test_fs_string(LOREM_IPSUM, Path::new("./tests/data/lorem.txt.gz"))
}

#[cfg_attr(not(feature = "file-xz"), ignore)]
#[test]
fn test_read_xz_fs_string() -> Result<(), Error> {
    do_read_test_fs_string(LOREM_IPSUM, Path::new("./tests/data/lorem.txt.xz"))
}

#[test]
fn test_write_plaintext_fs() -> Result<(), Error> {
    do_write_test_fs(Path::new("./tests/data/lorem.txt"), ".txt")
}

#[cfg_attr(not(feature = "file-bz2"), ignore)]
#[test]
fn test_write_bzip2_fs() -> Result<(), Error> {
    do_write_test_fs(Path::new("./tests/data/lorem.txt.default.bz2"), ".bz2")
}

#[cfg_attr(not(feature = "file-gz"), ignore)]
#[test]
fn test_write_gzip_fs() -> Result<(), Error> {
    do_write_test_fs(Path::new("./tests/data/lorem.txt.default.gz"), ".gz")
}

#[cfg_attr(not(feature = "file-xz"), ignore)]
#[test]
fn test_write_xz_fs() -> Result<(), Error> {
    do_write_test_fs(Path::new("./tests/data/lorem.txt.default.xz"), ".xz")
}

#[test]
fn test_truncating_write() -> Result<(), Error> {
    let tmpfile = Builder::new().suffix(".txt").tempfile()?;

    let long_text = "Long Text\n".repeat(20);
    let short_text = "short\n".repeat(5);

    // First write a long text to expand the file
    fs::write(tmpfile.path(), &long_text)?;
    do_read_test(&long_text, tmpfile.path())?;

    // Then write something short to see if the file got truncated
    fs::write(tmpfile.path(), &short_text)?;
    do_read_test(&short_text, tmpfile.path())
}

#[test]
fn test_append_file() -> Result<(), Error> {
    let tmpfile = Builder::new().suffix(".txt").tempfile()?;

    fs::append(tmpfile.path(), "Hello")?;
    fs::append(tmpfile.path(), " ")?;
    fs::append(tmpfile.path(), "World\n")?;

    do_read_test("Hello World\n", tmpfile.path())
}

#[cfg_attr(not(feature = "file-gz"), ignore)]
#[test]
fn test_append_file_gz() -> Result<(), Error> {
    let tmpfile = Builder::new().suffix(".gz").tempfile()?;

    fs::append(tmpfile.path(), "Hello")?;
    fs::append(tmpfile.path(), " ")?;
    fs::append(tmpfile.path(), "World\n")?;

    do_read_test("Hello World\n", tmpfile.path())
}

#[cfg_attr(not(unix), ignore)]
#[test]
fn test_read_dev_null() -> Result<(), Error> {
    fs::read_to_string("/dev/null")?;
    Ok(())
}
