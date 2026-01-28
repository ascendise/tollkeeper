use pretty_assertions::assert_eq;
use ringmap::RingSet;

use crate::declarations::hashcash::{DoubleSpentDatabase, DoubleSpentDatabaseImpl, StampError};

#[test]
pub fn insert_with_full_db_should_discard_old_stamp() {
    // Arrange
    let stamp_limit = 10;
    let sut = DoubleSpentDatabaseImpl::new(Some(stamp_limit));
    // Act
    for i in 1..=20 {
        sut.insert(i.to_string()).unwrap();
    }
    // Assert
    let expected_stamps = (11..=20)
        .map(|i| i.to_string())
        .collect::<RingSet<String>>();
    assert_eq!(expected_stamps, sut.stamps());
}

#[test]
pub fn insert_with_too_long_stamp_should_be_rejected() {
    // Arrange
    let sut = DoubleSpentDatabaseImpl::new(None);
    // Act
    let stamp = self::gen_str(256); //Limit is 255
    let result = sut.insert(stamp);
    // Assert
    assert_eq!(Err(StampError::StampTooLong), result);
}

fn gen_str(str_len: i32) -> String {
    let mut str = String::new();
    for _ in 0..str_len {
        str.push('a');
    }
    str
}
