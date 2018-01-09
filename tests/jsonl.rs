#![cfg(feature = "jsonl")]

extern crate misc_utils;
#[macro_use]
extern crate pretty_assertions;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use misc_utils::fs::{parse_jsonl_multi_threaded, ProcessingStatus};

#[test]
fn test_read_compressed_jsonl() {
    let mut iter = parse_jsonl_multi_threaded::<_, (u64, u64)>("./tests/data/jsonl.xz", 1024);
    match iter.next().unwrap() {
        ProcessingStatus::Data(d) => assert_eq!(d, (1, 2)),
        _ => panic!("First value must be Data"),
    }
    match iter.next().unwrap() {
        ProcessingStatus::Data(d) => assert_eq!(d, (987, 666)),
        _ => panic!("Second value must be Data"),
    }
    match iter.next().unwrap() {
        ProcessingStatus::Data(d) => assert_eq!(d, (0, 0)),
        _ => panic!("Third value must be Data"),
    }
    match iter.next().unwrap() {
        ProcessingStatus::Completed => {}
        _ => panic!("Last value must be Completed"),
    }
}

#[test]
fn test_read_compressed_jsonl_complex_type() {
    #[derive(Debug, Eq, PartialEq, Deserialize)]
    enum Value {
        Something,
        Else,
    }

    #[derive(Debug, Eq, PartialEq, Deserialize)]
    struct Deserializeable {
        int: u64,
        value: Value,
    }

    let mut iter = parse_jsonl_multi_threaded::<_, Deserializeable>(
        "./tests/data/jsonl-complex-type.txt",
        1024,
    );
    match iter.next().unwrap() {
        ProcessingStatus::Data(d) => assert_eq!(
            d,
            Deserializeable {
                int: 6782,
                value: Value::Something,
            }
        ),
        _ => panic!("First value must be Data"),
    }
    match iter.next().unwrap() {
        ProcessingStatus::Data(d) => assert_eq!(
            d,
            Deserializeable {
                int: 986273,
                value: Value::Else,
            }
        ),
        _ => panic!("Second value must be Data"),
    }
    match iter.next().unwrap() {
        ProcessingStatus::Completed => {}
        _ => panic!("Last value must be Completed"),
    }
}
