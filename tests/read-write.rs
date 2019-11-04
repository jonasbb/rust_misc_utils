use misc_utils::fs::{self, file_open_read, file_open_write, WriteOptions};
#[cfg(any(feature = "file-gz", feature = "file-xz", feature = "file-bz2"))]
use misc_utils::fs::{Compression, FileType};
use pretty_assertions::assert_eq;
use std::{fs::File, io::prelude::*, path::Path};
use tempfile::{Builder, NamedTempFile};

const LOREM_IPSUM: &str = r#"Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod
tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At
vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren,
no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit
amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut
labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam
et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata
sanctus est Lorem ipsum dolor sit amet."#;

fn assert_file_eq(file1: &Path, file2: &Path) {
    let mut content1 = Vec::new();
    let mut content2 = Vec::new();
    let mut file1 = File::open(file1).unwrap();
    let mut file2 = File::open(file2).unwrap();
    file1.read_to_end(&mut content1).unwrap();
    file2.read_to_end(&mut content2).unwrap();
    assert_eq!(content1, content2);
}

fn do_read_test(file: &Path, expected: &str) {
    let mut reader = file_open_read(Path::new(file)).unwrap();
    let mut content = String::new();
    reader.read_to_string(&mut content).unwrap();
    assert_eq!(content, expected);
}

fn do_write_test(file: &Path, options: WriteOptions) {
    let tmpfile = NamedTempFile::new().unwrap();
    let mut writer = file_open_write(tmpfile.path(), options).unwrap();
    writer.write_all(LOREM_IPSUM.as_bytes()).unwrap();
    // flush all data
    writer.flush().unwrap();
    // finish archive creation
    drop(writer);

    assert_file_eq(tmpfile.path(), file);
}

fn do_read_test_fs_bytes(file: &Path, expected: &str) {
    let content = fs::read(file).unwrap();
    assert_eq!(content, expected.as_bytes());
}

fn do_read_test_fs_string(file: &Path, expected: &str) {
    let content = fs::read_to_string(file).unwrap();
    assert_eq!(content, expected);
}

fn do_write_test_fs(file: &Path, suffix: &str) {
    let tmpfile = Builder::new().suffix(suffix).tempfile().unwrap();
    fs::write(tmpfile.path(), LOREM_IPSUM).unwrap();

    assert_file_eq(tmpfile.path(), file);
}

#[test]
fn test_read_empty_file() {
    do_read_test(Path::new("./tests/data/empty.txt"), "");
}

#[test]
fn test_read_plaintext() {
    do_read_test(Path::new("./tests/data/lorem.txt"), LOREM_IPSUM);
}

#[cfg_attr(not(feature = "file-bz2"), ignore)]
#[test]
fn test_read_bz2() {
    do_read_test(Path::new("./tests/data/lorem.txt.bz2"), LOREM_IPSUM);
}

#[cfg_attr(not(feature = "file-gz"), ignore)]
#[test]
fn test_read_gz() {
    do_read_test(Path::new("./tests/data/lorem.txt.gz"), LOREM_IPSUM);
}

#[cfg_attr(not(feature = "file-xz"), ignore)]
#[test]
fn test_read_xz() {
    do_read_test(Path::new("./tests/data/lorem.txt.xz"), LOREM_IPSUM);
}

#[test]
fn test_write_plaintext() {
    let options = WriteOptions::default();
    do_write_test(Path::new("./tests/data/lorem.txt"), options);
}

#[cfg(feature = "file-bz2")]
#[test]
fn test_write_bzip2() {
    let options = WriteOptions::default()
        .set_filetype(FileType::Bz2)
        .set_compression_level(Compression::Best);
    do_write_test(Path::new("./tests/data/lorem.txt.bz2"), options);
}

#[cfg(feature = "file-gz")]
#[test]
fn test_write_gzip() {
    let options = WriteOptions::default()
        .set_filetype(FileType::Gz)
        .set_compression_level(Compression::Best);
    do_write_test(Path::new("./tests/data/lorem.txt.gz"), options);
}

#[cfg(feature = "file-xz")]
#[test]
fn test_write_xz() {
    let options = WriteOptions::default()
        .set_filetype(FileType::Xz)
        .set_compression_level(Compression::Best);
    do_write_test(Path::new("./tests/data/lorem.txt.xz"), options);
}

#[test]
fn test_read_empty_file_fs_bytes() {
    do_read_test_fs_bytes(Path::new("./tests/data/empty.txt"), "");
}

#[test]
fn test_read_plaintext_fs_bytes() {
    do_read_test_fs_bytes(Path::new("./tests/data/lorem.txt"), LOREM_IPSUM);
}

#[cfg_attr(not(feature = "file-bz2"), ignore)]
#[test]
fn test_read_bz2_fs_bytes() {
    do_read_test_fs_bytes(Path::new("./tests/data/lorem.txt.bz2"), LOREM_IPSUM);
}

#[cfg_attr(not(feature = "file-gz"), ignore)]
#[test]
fn test_read_gz_fs_bytes() {
    do_read_test_fs_bytes(Path::new("./tests/data/lorem.txt.gz"), LOREM_IPSUM);
}

#[cfg_attr(not(feature = "file-xz"), ignore)]
#[test]
fn test_read_xz_fs_bytes() {
    do_read_test_fs_bytes(Path::new("./tests/data/lorem.txt.xz"), LOREM_IPSUM);
}

#[test]
fn test_read_empty_file_fs_string() {
    do_read_test_fs_string(Path::new("./tests/data/empty.txt"), "");
}

#[test]
fn test_read_plaintext_fs_string() {
    do_read_test_fs_string(Path::new("./tests/data/lorem.txt"), LOREM_IPSUM);
}

#[cfg_attr(not(feature = "file-bz2"), ignore)]
#[test]
fn test_read_bz2_fs_string() {
    do_read_test_fs_string(Path::new("./tests/data/lorem.txt.bz2"), LOREM_IPSUM);
}

#[cfg_attr(not(feature = "file-gz"), ignore)]
#[test]
fn test_read_gz_fs_string() {
    do_read_test_fs_string(Path::new("./tests/data/lorem.txt.gz"), LOREM_IPSUM);
}

#[cfg_attr(not(feature = "file-xz"), ignore)]
#[test]
fn test_read_xz_fs_string() {
    do_read_test_fs_string(Path::new("./tests/data/lorem.txt.xz"), LOREM_IPSUM);
}

#[test]
fn test_write_plaintext_fs() {
    do_write_test_fs(Path::new("./tests/data/lorem.txt"), ".txt");
}

#[cfg_attr(not(feature = "file-bz2"), ignore)]
#[test]
fn test_write_bzip2_fs() {
    do_write_test_fs(Path::new("./tests/data/lorem.txt.default.bz2"), ".bz2");
}

#[cfg_attr(not(feature = "file-gz"), ignore)]
#[test]
fn test_write_gzip_fs() {
    do_write_test_fs(Path::new("./tests/data/lorem.txt.default.gz"), ".gz");
}

#[cfg_attr(not(feature = "file-xz"), ignore)]
#[test]
fn test_write_xz_fs() {
    do_write_test_fs(Path::new("./tests/data/lorem.txt.default.xz"), ".xz");
}
