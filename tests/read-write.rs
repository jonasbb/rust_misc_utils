extern crate misc_utils;
#[macro_use]
extern crate pretty_assertions;
extern crate tempfile;

use misc_utils::fs::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tempfile::NamedTempFile;

const LOREM_IPSUM: &str = r#"Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod
tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At
vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren,
no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit
amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut
labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam
et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata
sanctus est Lorem ipsum dolor sit amet."#;

#[test]
fn test_read_empty_file() {
    do_read_test(Path::new("./tests/data/empty.txt"), "");
}

#[test]
fn test_read_plaintext() {
    do_read_test(Path::new("./tests/data/lorem.txt"), LOREM_IPSUM);
}

#[test]
fn test_read_bz2() {
    do_read_test(Path::new("./tests/data/lorem.txt.bz2"), LOREM_IPSUM);
}

#[test]
fn test_read_gz() {
    do_read_test(Path::new("./tests/data/lorem.txt.gz"), LOREM_IPSUM);
}

#[test]
fn test_read_xz() {
    do_read_test(Path::new("./tests/data/lorem.txt.xz"), LOREM_IPSUM);
}

fn do_read_test(file: &Path, expected: &str) {
    let mut reader = file_open_read(Path::new(file)).unwrap();
    let mut content = String::new();
    reader.read_to_string(&mut content).unwrap();
    assert_eq!(content, expected);
}

#[test]
fn test_write_plaintext() {
    let options = WriteOptions::default();
    do_write_test(Path::new("./tests/data/lorem.txt"), options);
}

#[test]
fn test_write_bzip2() {
    let options = WriteOptions::default()
        .set_filetype(FileType::Bz2)
        .set_compression_level(Compression::Best);
    do_write_test(Path::new("./tests/data/lorem.txt.bz2"), options);
}

#[test]
fn test_write_gzip() {
    let options = WriteOptions::default()
        .set_filetype(FileType::Gz)
        .set_compression_level(Compression::Best);
    do_write_test(Path::new("./tests/data/lorem.txt.gz"), options);
}

#[test]
fn test_write_xz() {
    let options = WriteOptions::default()
        .set_filetype(FileType::Xz)
        .set_compression_level(Compression::Best);
    do_write_test(Path::new("./tests/data/lorem.txt.xz"), options);
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

fn assert_file_eq(file1: &Path, file2: &Path) {
    let mut content1 = Vec::new();
    let mut content2 = Vec::new();
    let mut file1 = File::open(file1).unwrap();
    let mut file2 = File::open(file2).unwrap();
    file1.read_to_end(&mut content1).unwrap();
    file2.read_to_end(&mut content2).unwrap();
    assert_eq!(content1, content2);
}
