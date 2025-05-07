use chrono::TimeZone;

#[cfg(test)]
use crate::tollkeeper::declarations::hashcash::*;

#[test]
pub fn to_string_should_return_hashcash_v1_format_string() {
    // Arrange
    let date = chrono::Utc
        .with_ymd_and_hms(2025, 5, 7, 22, 24, 06)
        .unwrap()
        .to_utc();
    let date = Timestamp(date);
    let ext: Vec<(String, String)> = vec![
        ("key".into(), "value".into()),
        ("rust".into(), "good".into()),
        ("hotel?".into(), "trivago!".into()),
    ];
    let ext = Extension(ext);
    let stamp = Stamp::new(1, 3, date, "test", ext, "veryrandomstring", "123");
    // Act
    let stamp_str = stamp.to_string();
    // Assert
    let expected = "1:3:250507222406:test:key=value;rust=good;hotel?=trivago!:veryrandomstring:123";
    assert_eq!(expected, stamp_str);
}
