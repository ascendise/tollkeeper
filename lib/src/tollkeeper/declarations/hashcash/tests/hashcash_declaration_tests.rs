#[cfg(test)]
use crate::tollkeeper::declarations::{Destination, OrderIdentifier, Suspect};
use crate::tollkeeper::{
    declarations::{
        hashcash::{DoubleSpentDatabaseImpl, HashcashDeclaration},
        Declaration, Payment,
    },
    util::FakeDateTimeProvider,
};
use chrono::TimeZone;

use std::collections::{HashMap, HashSet};

fn setup() -> HashcashDeclaration {
    let today = chrono::Utc
        .with_ymd_and_hms(2025, 5, 6, 20, 24, 6)
        .unwrap()
        .to_utc();
    let expiry = chrono::Duration::days(1);
    let double_spent_db = DoubleSpentDatabaseImpl::new();
    HashcashDeclaration::new(
        4,
        expiry,
        Box::new(FakeDateTimeProvider(today)),
        Box::new(double_spent_db),
    )
}
fn setup_with_date(date: chrono::DateTime<chrono::Utc>) -> HashcashDeclaration {
    let expiry = chrono::Duration::days(1);
    let double_spent_db = DoubleSpentDatabaseImpl::new();
    HashcashDeclaration::new(
        4,
        expiry,
        Box::new(FakeDateTimeProvider(date)),
        Box::new(double_spent_db),
    )
}
fn setup_with_init_db(stamps: HashSet<String>) -> HashcashDeclaration {
    let today = chrono::Utc
        .with_ymd_and_hms(2025, 5, 6, 20, 24, 6)
        .unwrap()
        .to_utc();
    let expiry = chrono::Duration::days(1);
    let double_spent_db = DoubleSpentDatabaseImpl::init(stamps);
    HashcashDeclaration::new(
        4,
        expiry,
        Box::new(FakeDateTimeProvider(today)),
        Box::new(double_spent_db),
    )
}

#[test]
pub fn declare_should_return_new_toll_for_suspect() {
    // Arrange
    let sut = setup();
    // Act
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new_with_details("example.com", 8888, "/hello"),
    );
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = sut.declare(suspect.clone(), order_id.clone());
    let mut expected_challenge = HashMap::<String, String>::new();
    expected_challenge.insert("ver".into(), "1".into());
    expected_challenge.insert("bits".into(), "4".into());
    expected_challenge.insert("resource".into(), "example.com(8888)/hello".into());
    expected_challenge.insert("ext".into(), "suspect.ip=1.2.3.4".into());
    // Assert
    assert_eq!(toll.recipient(), &suspect);
    assert_eq!(toll.order_id(), &order_id);
    assert_eq!(toll.challenge(), &expected_challenge);
}

#[test]
pub fn pay_with_valid_payment_should_return_visa() {
    // Arrange
    let mut sut = setup();
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new_with_details("example.com", 8888, "/hello"),
    );
    // Act
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = sut.declare(suspect.clone(), order_id.clone());
    let stamp = "1:4:250506202406:example.com(8888)/hello:suspect.ip=1.2.3.4:VM81iAlX9M94FSXy:0000000000000000002";
    let payment = Payment::new(toll, stamp);
    let visa = sut
        .pay(payment, &suspect)
        .expect("Expected Visa, got InvalidPaymentError");
    // Assert
    assert_eq!(visa.suspect(), &suspect);
    assert_eq!(visa.order_id(), &order_id);
    assert!(sut.double_spent_db.stamps().contains(stamp));
}

#[test]
pub fn pay_with_invalid_stamp_should_return_error() {
    // Arrange
    let mut sut = setup();
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new_with_details("example.com", 8888, "/hello"),
    );
    // Act
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = sut.declare(suspect.clone(), order_id.clone());
    let payment = Payment::new(
        toll,
        "1:4:250506202406:example.com(8888)/hello:suspect.ip=1.2.3.4:notitchief:0",
    );
    let error = sut
        .pay(payment.clone(), &suspect)
        .expect_err("Expected InvalidPaymentError, got Visa");
    // Assert
    assert_eq!(error.payment(), &payment);
}

#[test]
pub fn pay_with_expired_stamp_should_return_error() {
    // Arrange
    let today = chrono::Utc
        .with_ymd_and_hms(2025, 5, 8, 20, 24, 6)
        .unwrap()
        .to_utc();
    let mut sut = setup_with_date(today);
    // Act
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new_with_details("example.com", 8888, "/hello"),
    );
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = sut.declare(suspect.clone(), order_id.clone());
    let payment = Payment::new(toll, "1:4:250506202406:example.com(8888)/hello:suspect.ip=1.2.3.4:VM81iAlX9M94FSXy:0000000000000000002"); //minted two days earlier
    let error = sut
        .pay(payment.clone(), &suspect)
        .expect_err("Expected InvalidPaymentError, got Visa");
    // Assert
    assert_eq!(error.payment(), &payment);
}

#[test]
pub fn pay_with_stamp_from_the_future_should_return_error() {
    // Arrange
    let today = chrono::Utc
        .with_ymd_and_hms(2025, 5, 4, 20, 24, 6)
        .unwrap()
        .to_utc();
    let mut sut = setup_with_date(today);
    // Act
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new_with_details("example.com", 8888, "/hello"),
    );
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = sut.declare(suspect.clone(), order_id.clone());
    let payment = Payment::new(toll, "1:4:250506202406:example.com(8888)/hello:suspect.ip=1.2.3.4:VM81iAlX9M94FSXy:0000000000000000002"); //minted two days in the future!
    let error = sut
        .pay(payment.clone(), &suspect)
        .expect_err("Expected InvalidPaymentError, got Visa");
    // Assert
    assert_eq!(error.payment(), &payment);
}

#[test]
pub fn pay_with_duplicate_stamp_should_return_error() {
    // Arrange
    let stamp = String::from("1:4:250506202406:example.com(8888)/hello:suspect.ip=1.2.3.4:kuwuD8w8/fkWCM+K:0000000000000000006");
    let mut stamps = HashSet::<String>::new();
    stamps.insert(stamp.clone());
    let mut sut = setup_with_init_db(stamps);
    // Act
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new_with_details("example.com", 8888, "/hello"),
    );
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = sut.declare(suspect.clone(), order_id.clone());
    let payment = Payment::new(toll, stamp); //Reusing stamp already present in Double-Spent
                                             //database
    let error = sut
        .pay(payment.clone(), &suspect)
        .expect_err("Expected InvalidPaymentError, got Visa");
    // Assert
    assert_eq!(error.payment(), &payment);
}
