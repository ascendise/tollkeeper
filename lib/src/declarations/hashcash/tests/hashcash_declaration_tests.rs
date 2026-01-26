use crate::declarations::*;
use crate::{
    declarations::{
        hashcash::{DoubleSpentDatabaseImpl, HashcashDeclaration},
        Declaration, Payment,
    },
    descriptions::Destination,
    util::FakeDateTimeProvider,
};
use chrono::TimeZone;
use pretty_assertions::assert_eq;
use test_case::test_case;

use std::collections::HashSet;

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
        Destination::new("example.com", 8888, "/hello"),
    );
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = sut.declare(suspect.clone(), order_id.clone());
    let mut expected_challenge = Challenge::new();
    expected_challenge.insert("ver".into(), "1".into());
    expected_challenge.insert("bits".into(), "4".into());
    expected_challenge.insert("width".into(), "12".into());
    expected_challenge.insert("resource".into(), "example.com(8888)/hello".into());
    expected_challenge.insert("ext".into(), "suspect.ip=1.2.3.4".into());
    // Assert
    assert_eq!(&suspect, toll.recipient());
    assert_eq!(&order_id, toll.order_id());
    assert_eq!(&expected_challenge, toll.challenge());
}

#[test]
pub fn pay_with_valid_payment_should_return_visa() {
    // Arrange
    let today = chrono::Utc
        .with_ymd_and_hms(2025, 5, 6, 20, 24, 6)
        .unwrap()
        .to_utc();
    let sut = setup_with_date(today); //Expiry duration set to 1 Day
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new("example.com", 8888, "/hello"),
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
    let expected_expiry_date = today + chrono::Duration::days(1);
    assert_eq!(&suspect, visa.suspect());
    assert_eq!(&order_id, visa.order_id());
    assert_eq!(&expected_expiry_date, visa.expires());
    assert!(sut.double_spent_db.stamps().contains(stamp));
}

#[test]
pub fn pay_with_invalid_stamp_should_return_error() {
    // Arrange
    let sut = setup();
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new("example.com", 8888, "/hello"),
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

#[test_case("1:3:260120153000:example(8888)/:suspect.ip=1.2.3.4:xaF/u7xxK4q/PR8s:000000000000000000000000000H" ; "too low difficulty")]
#[test_case("1:4:260120153000:example(8888)/:suspect.ip=0.0.0.0:vLORtrZJj3brGev6:000000000000000000000000000s" ; "wrong extension")]
#[test_case("1:4:260120153000:example(8888)/:hello=world:KNCJk/cUp3L/Qf2/:00000000000000000000000000000000003" ; "different extension")]
#[test_case("1:4:260120153000:example(8888)/::yJpIYAvg9PBnLNaz:0000000000000000000000000000000000000000000002" ; "missing extension")]
#[test_case("1:4:260120153000:example(1234)/:suspect.ip=1.2.3.4:zMVAqKtfiZt/z83K:0000000000000000000000000005" ; "wrong destination/resource")]
pub fn paying_with_a_stamp_not_matching_challenge_should_return_error(invalid_stamp: &str) {
    // Arrange
    let today = chrono::Utc
        .with_ymd_and_hms(2026, 1, 20, 16, 30, 0)
        .unwrap()
        .to_utc();
    let expiry = chrono::Duration::days(1);
    let double_spent_db = DoubleSpentDatabaseImpl::new();
    let sut = HashcashDeclaration::new(
        4,
        expiry,
        Box::new(FakeDateTimeProvider(today)),
        Box::new(double_spent_db),
    );
    let suspect = Suspect::new("1.2.3.4", "Bot", Destination::new("example", 8888, "/"));
    // Act
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = sut.declare(suspect.clone(), order_id.clone());
    let payment = Payment::new(toll, invalid_stamp);
    let error = sut
        .pay(payment.clone(), &suspect)
        .expect_err("Expected InvalidPaymentError, got Visa");
    // Assert
    assert_eq!(&payment, error.payment());
}

#[test]
pub fn pay_with_expired_stamp_should_return_error() {
    // Arrange
    let today = chrono::Utc
        .with_ymd_and_hms(2025, 5, 8, 20, 24, 6)
        .unwrap()
        .to_utc();
    let sut = setup_with_date(today);
    // Act
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new("example.com", 8888, "/hello"),
    );
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = sut.declare(suspect.clone(), order_id.clone());
    let payment = Payment::new(toll, "1:4:250506202406:example.com(8888)/hello:suspect.ip=1.2.3.4:VM81iAlX9M94FSXy:0000000000000000002"); //minted two days earlier
    let error = sut
        .pay(payment.clone(), &suspect)
        .expect_err("Expected InvalidPaymentError, got Visa");
    // Assert
    assert_eq!(&payment, error.payment());
}

#[test]
pub fn pay_with_stamp_from_the_future_should_return_error() {
    // Arrange
    let today = chrono::Utc
        .with_ymd_and_hms(2025, 5, 4, 20, 24, 6)
        .unwrap()
        .to_utc();
    let sut = setup_with_date(today);
    // Act
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new("example.com", 8888, "/hello"),
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

#[test_case("1:4:000102130004:example.com(8888)/hello:suspect.ip=1.2.3.4:pPpGomTDbOIdN3Z4:000000000000000000H" ; "Desync into future")]
#[test_case("1:4:000101130000:example.com(8888)/hello:suspect.ip=1.2.3.4:bAgDUTm7uB1uIVHG:000000000000000000y" ; "Desync into past")]
pub fn pay_with_expired_stamp_should_allow_grace_period_for_desyncs(stamp: &str) {
    // Arrange
    let today = chrono::Utc
        .with_ymd_and_hms(2000, 1, 2, 12, 59, 59)
        .unwrap()
        .to_utc();
    let sut = setup_with_date(today);
    // Act
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new("example.com", 8888, "/hello"),
    );
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = sut.declare(suspect.clone(), order_id.clone());
    let payment = Payment::new(toll, stamp);
    let result = sut.pay(payment.clone(), &suspect);
    // Assert
    assert!(
        result.is_ok(),
        "Stamp got rejected despite time desync being inside grace period"
    );
}

#[test]
pub fn pay_with_duplicate_stamp_should_return_error() {
    // Arrange
    let stamp = String::from("1:4:250506202406:example.com(8888)/hello:suspect.ip=1.2.3.4:kuwuD8w8/fkWCM+K:0000000000000000006");
    let mut stamps = HashSet::<String>::new();
    stamps.insert(stamp.clone());
    let sut = setup_with_init_db(stamps);
    // Act
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new("example.com", 8888, "/hello"),
    );
    let order_id = OrderIdentifier::new("gate", "order");
    let toll = sut.declare(suspect.clone(), order_id.clone());
    let payment = Payment::new(toll, stamp); //Reusing stamp already present in Double-Spent
                                             //database
    let error = sut
        .pay(payment.clone(), &suspect)
        .expect_err("Expected InvalidPaymentError, got Visa");
    // Assert
    assert_eq!(&payment, error.payment());
}
