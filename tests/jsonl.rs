#![cfg(feature = "jsonl")]

extern crate misc_utils;
#[macro_use]
extern crate pretty_assertions;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use misc_utils::fs::parse_jsonl_multi_threaded;

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

#[test]
fn test_read_compressed_jsonl() {
    let mut iter = parse_jsonl_multi_threaded::<_, (u64, u64)>("./tests/data/jsonl.xz", 1024);
    match iter.next().unwrap() {
        Ok(d) => assert_eq!(d, (1, 2)),
        _ => panic!("First value must be Data"),
    }
    match iter.next().unwrap() {
        Ok(d) => assert_eq!(d, (987, 666)),
        _ => panic!("Second value must be Data"),
    }
    match iter.next().unwrap() {
        Ok(d) => assert_eq!(d, (0, 0)),
        _ => panic!("Third value must be Data"),
    }
    // assert finished completely
    assert!(iter.next().is_none());
}

#[test]
fn test_read_complex_type() {
    let mut iter = parse_jsonl_multi_threaded::<_, Deserializeable>(
        "./tests/data/jsonl-complex-type.txt",
        1024,
    );
    match iter.next().unwrap() {
        Ok(d) => assert_eq!(
            d,
            Deserializeable {
                int: 6782,
                value: Value::Something,
            }
        ),
        _ => panic!("First value must be Data"),
    }
    match iter.next().unwrap() {
        Ok(d) => assert_eq!(
            d,
            Deserializeable {
                int: 986_273,
                value: Value::Else,
            }
        ),
        _ => panic!("Second value must be Data"),
    }
    // assert finished completely
    assert!(iter.next().is_none())
}

#[test]
fn test_read_broken_json() {
    let mut iter = parse_jsonl_multi_threaded::<_, Deserializeable>(
        "./tests/data/jsonl-broken-json.txt",
        1024,
    );
    match iter.next().unwrap() {
        Ok(d) => assert_eq!(
            d,
            Deserializeable {
                int: 6782,
                value: Value::Something,
            }
        ),
        _ => panic!("First value must be Data"),
    }
    if iter.next().unwrap().is_ok() {
        panic!("Second value must be ParsingError")
    }
    // assert finished completely
    assert!(iter.next().is_none())
}
