#[cfg(test)]
use crate::tollkeeper::declarations::{Destination, OrderIdentifier, Suspect};
use crate::tollkeeper::{
    declarations::hashcash::HashcashDeclaration,
    util::{FakeDateTimeProvider, FakeRandomStringGen},
};
use chrono::TimeZone;

use std::collections::HashMap;

use crate::tollkeeper::declarations::Declaration;

#[test]
pub fn declare_should_return_new_toll_for_suspect() {
    // Arrange
    let fake_date = chrono::Utc
        .with_ymd_and_hms(2025, 5, 6, 22, 24, 06)
        .unwrap()
        .to_utc();
    let declaration = HashcashDeclaration::new(
        4,
        Box::new(FakeDateTimeProvider(fake_date)),
        Box::new(FakeRandomStringGen("abcdefgh".into())),
    );
    // Act
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new_with_details("example.com", 8888, "/hello"),
    );
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = declaration.declare(suspect.clone(), order_id.clone());
    let mut expected_challenge = HashMap::<String, String>::new();
    expected_challenge.insert("ver".into(), "1".into());
    expected_challenge.insert("bits".into(), "4".into());
    expected_challenge.insert("date".into(), "250506222406".into());
    expected_challenge.insert("resource".into(), "example.com:8888/hello".into());
    expected_challenge.insert("ext".into(), "suspect.ip=1.2.3.4".into());
    expected_challenge.insert("rand".into(), "abcdefgh".into());
    expected_challenge.insert("counter".into(), "0".into());
    // Assert
    assert_eq!(toll.recipient(), &suspect);
    assert_eq!(toll.order_id(), &order_id);
    assert_eq!(toll.challenge(), &expected_challenge);
}
