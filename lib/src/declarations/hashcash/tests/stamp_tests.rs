use chrono::TimeZone;
use pretty_assertions::assert_eq;

use crate::declarations::hashcash::*;

#[test]
pub fn to_string_should_return_hashcash_v1_format_string() {
    // Arrange
    let date = chrono::Utc
        .with_ymd_and_hms(2025, 5, 7, 22, 24, 6)
        .unwrap()
        .to_utc();
    let date = Timestamp(date);
    let ext = indexmap::indexmap![
        "key".into() => "value".into(),
        "rust".into() => "good".into(),
        "hotel?".into() => "trivago!".into(),
    ];
    let ext = Extension(ext);
    let res = Resource(Destination::new_base("localhost"));
    let stamp = Stamp::new(1, 3, date, res, ext, "veryrandomstring", "123");
    // Act
    let stamp_str = stamp.to_string();
    // Assert
    let expected =
        "1:3:250507222406:localhost(80)/:key=value;rust=good;hotel?=trivago!:veryrandomstring:123";
    assert_eq!(expected, stamp_str);
}

#[test]
pub fn from_string_should_return_stamp_from_syntactically_valid_stamp() {
    // Arrange
    let stamp =
        "1:3:250507222406:localhost(80)/:key=value;rust=good;hotel?=trivago!:veryrandomstring:123";
    // Act
    let stamp = Stamp::from_str(stamp).expect("Failed to parse valid stamp!");
    // Assert
    let date = chrono::Utc
        .with_ymd_and_hms(2025, 5, 7, 22, 24, 6)
        .unwrap()
        .to_utc();
    let date = Timestamp(date);
    let ext = indexmap::indexmap![
        "key".into() => "value".into(),
        "rust".into() => "good".into(),
        "hotel?".into() => "trivago!".into(),
    ];
    let ext = Extension(ext);
    let res = Resource(Destination::new_base("localhost"));
    let expected_stamp = Stamp::new(1, 3, date, res, ext, "veryrandomstring", "123");
    assert_eq!(expected_stamp, stamp);
}

#[test]
pub fn from_string_should_return_stamp_from_stamp_without_extensions() {
    // Arrange
    let stamp = "1:3:250507222406:localhost(80)/::veryrandomstring:123";
    // Act
    let stamp = Stamp::from_str(stamp).expect("Failed to parse valid stamp!");
    // Assert
    let date = chrono::Utc
        .with_ymd_and_hms(2025, 5, 7, 22, 24, 6)
        .unwrap()
        .to_utc();
    let date = Timestamp(date);
    let ext = indexmap::indexmap![];
    let ext = Extension(ext);
    let res = Resource(Destination::new_base("localhost"));
    let expected_stamp = Stamp::new(1, 3, date, res, ext, "veryrandomstring", "123");
    assert_eq!(expected_stamp, stamp);
}

#[test]
pub fn check_hash_should_return_true_if_hash_is_valid() {
    // Arrange
    let date = chrono::Utc
        .with_ymd_and_hms(2025, 5, 7, 20, 24, 6)
        .unwrap()
        .to_utc();
    let date = Timestamp(date);
    let ext = indexmap::indexmap![
        "key".into() => "value".into(),
        "rust".into() => "good".into(),
        "hotel?".into() => "trivago!".into(),
    ];
    let ext = Extension(ext);
    let res = Resource(Destination::new_base("localhost"));
    // 1:3:250507202406:localhost(80)/:key=value;rust=good;hotel?=trivago!:lHM0wrJDfP4CXXml:00000000008
    let stamp = Stamp::new(1, 3, date, res, ext, "lHM0wrJDfP4CXXml", "00000000008");
    // Act
    let is_valid_hash = stamp.is_valid();
    // Assert
    assert!(is_valid_hash);
}

#[test]
pub fn check_hash_should_return_false_if_hash_is_invalid() {
    // Arrange
    let date = chrono::Utc
        .with_ymd_and_hms(2025, 5, 7, 22, 24, 6)
        .unwrap()
        .to_utc();
    let date = Timestamp(date);
    let ext = indexmap::indexmap![
        "key".into() => "value".into(),
        "rust".into() => "good".into(),
        "hotel?".into() => "trivago!".into(),
    ];
    let ext = Extension(ext);
    let res = Resource(Destination::new_base("localhost"));
    let stamp = Stamp::new(1, 3, date, res, ext, "veryrandom", "thisisnotitchief");
    // Act
    let is_valid_hash = stamp.is_valid();
    // Assert
    assert!(!is_valid_hash);
}
