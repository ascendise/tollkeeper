use super::*;
use crate::{signatures::InMemorySecretKeyProvider, util::FakeDateTimeProvider, *};
use chrono::TimeZone;
use pretty_assertions::assert_eq;
use test_case::test_case;

fn setup(current_time: Option<chrono::DateTime<chrono::Utc>>) -> (Tollkeeper, OrderIdentifier) {
    let secret_key: Vec<u8> = b"Secret key".into();
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new()),
    );
    let order_id = require_payment_order.id.clone();
    let gate = Gate::new(
        Destination::new_base("localhost"),
        vec![require_payment_order],
    )
    .unwrap();
    let order_id = OrderIdentifier::new(gate.id.clone(), order_id);
    let secret_key_provider = InMemorySecretKeyProvider::new(secret_key);
    let secret_key_provider = Box::new(secret_key_provider);
    let current_time = current_time.unwrap_or(chrono::Utc::now());
    let date_time_provider = Box::new(FakeDateTimeProvider(current_time));
    let tollkeeper = Tollkeeper::new(vec![gate], secret_key_provider, date_time_provider).unwrap();
    (tollkeeper, order_id)
}

fn setup_with_gates(
    gates: Vec<Gate>,
    current_time: Option<chrono::DateTime<chrono::Utc>>,
) -> Tollkeeper {
    let secret_key: Vec<u8> = b"Secret key".into();
    let secret_key_provider = InMemorySecretKeyProvider::new(secret_key);
    let secret_key_provider = Box::new(secret_key_provider);
    let current_time = current_time.unwrap_or(chrono::Utc::now());
    let date_time_provider = Box::new(FakeDateTimeProvider(current_time));
    Tollkeeper::new(gates, secret_key_provider, date_time_provider).unwrap()
}

fn setup_with_payment(
    current_time: Option<chrono::DateTime<chrono::Utc>>,
) -> (Tollkeeper, OrderIdentifier) {
    let secret_key: Vec<u8> = b"Secret key".into();
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new_payment_stub()),
    );
    let order_id = require_payment_order.id.clone();
    let gate = Gate::new(
        Destination::new_base("localhost"),
        vec![require_payment_order],
    )
    .unwrap();
    let order_id = OrderIdentifier::new(gate.id.clone(), order_id);
    let secret_key_provider = InMemorySecretKeyProvider::new(secret_key);
    let secret_key_provider = Box::new(secret_key_provider);
    let current_time = current_time.unwrap_or(chrono::Utc::now());
    let date_time_provider = Box::new(FakeDateTimeProvider(current_time));
    let tollkeeper = Tollkeeper::new(vec![gate], secret_key_provider, date_time_provider).unwrap();
    (tollkeeper, order_id)
}

fn assert_is_allowed(access_result: &Result<(), AccessError>) {
    match access_result {
        Ok(_) => (),
        Err(e) => match e {
            AccessError::AccessDeniedError(_) => {
                panic!("Expected access allowed but got a toll!")
            }
            AccessError::DestinationNotFound(destination) => {
                panic!("Expected access allowed but could not find destination!: {destination}")
            }
        },
    }
}

fn assert_is_denied(access_result: &Result<(), AccessError>) -> &Signed<Toll> {
    match access_result {
        Ok(_) => panic!("Expected a toll but was allowed access!"),
        Err(e) => match e {
            AccessError::AccessDeniedError(e) => e,
            AccessError::DestinationNotFound(destination) => {
                panic!("Expected access allowed but could not find destination!: {destination}")
            }
        },
    }
}

#[test]
pub fn creating_new_toolkeeper_with_no_gates_should_fail() {
    // Arrange
    let secret_key_provider = InMemorySecretKeyProvider::new("Secret key".into());
    let secret_key_provider = Box::new(secret_key_provider);
    let date_time_provider = Box::new(FakeDateTimeProvider(chrono::Utc::now()));
    // Act
    let result = Tollkeeper::new(vec![], secret_key_provider, date_time_provider);
    // Assert
    assert!(
        result.is_err(),
        "Expected creation of Toolkeeper without gates to return an error",
    );
}

#[test_case(AccessPolicy::Blacklist, false ; "accessing gate with a blacklist order and not matching description")]
#[test_case(AccessPolicy::Whitelist, true ; "accessing gate with a matching whitelist order description")]
pub fn should_require_no_toll_if_not_matching_toll_requirements(
    access_policy: AccessPolicy,
    matches_description: bool,
) {
    // Arrange
    let suspect = Suspect::new(
        "1.2.3.4",
        "FriendlyCrawler",
        Destination::new_base("localhost"),
    );
    let description: Box<dyn Description + Send + Sync> =
        Box::new(StubDescription::new(matches_description));
    let order = Order::new(
        vec![description],
        access_policy,
        Box::new(StubDeclaration::new()),
    );
    let gate = Gate::new(Destination::new_base("localhost"), vec![order]).unwrap();
    let sut = setup_with_gates(vec![gate], None);
    // Act
    let access_result = sut.check_access(&suspect, None);
    // Assert
    assert_is_allowed(&access_result);
}

#[test_case(AccessPolicy::Blacklist, true ; "accessing a gate with a matching blacklist order description")]
#[test_case(AccessPolicy::Whitelist, false ; "accessing a gate with a whitelist order and not matching description")]
pub fn should_require_toll_if_matching_toll_requirement(
    access_policy: AccessPolicy,
    matches_description: bool,
) {
    // Arrange
    let suspect = Suspect::new("1.2.3.4", "BadCrawler", Destination::new_base("localhost"));
    let description: Box<dyn Description + Send + Sync> =
        Box::new(StubDescription::new(matches_description));
    let order = Order::new(
        vec![description],
        access_policy,
        Box::new(StubDeclaration::new()),
    );
    let gate = Gate::new(Destination::new_base("localhost"), vec![order]).unwrap();
    let sut = setup_with_gates(vec![gate], None);
    // Act
    let access_result = sut.check_access(&suspect, None);
    // Assert
    assert_is_denied(&access_result);
}

#[test]
pub fn passing_gate_with_first_matching_order_requiring_toll_should_return_toll() {
    // Arrange
    let malicious_suspect = Suspect::new(
        "1.2.3.4",
        "FriendlyCrawler",
        Destination::new_base("localhost"),
    );
    let first_order = Order::new(
        vec![Box::new(StubDescription::new(false))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new()),
    );
    let matching_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new()),
    );
    let last_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Whitelist,
        Box::new(StubDeclaration::new()),
    );
    let gate = Gate::new(
        Destination::new_base("localhost"),
        vec![first_order, matching_order, last_order],
    )
    .unwrap();
    let sut = setup_with_gates(vec![gate], None);
    // Act
    let access_result = sut.check_access(&malicious_suspect, None);
    // Assert
    let toll = assert_is_denied(&access_result);
    assert!(
        toll.verify(b"Secret key").is_ok(),
        "Returned toll has wrong signature!"
    );
}

#[test]
pub fn passing_gate_with_first_matching_order_allowing_access_should_allow_access() {
    // Arrange
    let benign_suspect = Suspect::new(
        "1.2.3.4",
        "FriendlyCrawler",
        Destination::new_base("localhost"),
    );
    let first_order = Order::new(
        vec![Box::new(StubDescription::new(false))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new()),
    );
    let matching_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Whitelist,
        Box::new(StubDeclaration::new()),
    );
    let last_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new()),
    );
    let gate = Gate::new(
        Destination::new_base("localhost"),
        vec![first_order, matching_order, last_order],
    )
    .unwrap();
    let sut = setup_with_gates(vec![gate], None);
    // Act
    let access_result = sut.check_access(&benign_suspect, None);
    // Assert
    assert_is_allowed(&access_result);
}

#[test]
pub fn passing_gate_with_valid_visa_should_allow_access() {
    // Arrange
    let (sut, order_id) = setup(None);
    // Act
    let destination = Destination::new_base("localhost");
    let suspect = Suspect::new("1.2.3.4", "Bot", destination);
    let visa = Visa::new(order_id, suspect.clone(), expires_from_now(1));
    let visa = Signed::sign(visa, b"Secret key");
    let access_result = sut.check_access(&suspect, Some(visa));
    // Assert
    assert_is_allowed(&access_result);
}

fn expires_from_now(add_days: u64) -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
        .checked_add_days(chrono::Days::new(add_days))
        .unwrap()
}

#[test]
pub fn passing_gate_with_valid_visa_for_resource_should_not_challenge_again_on_subresources() {
    // Arrange
    let (sut, order_id) = setup(None);
    // Act
    let visa_destination = Destination::new_base("localhost");
    let visa_suspect = Suspect::new("1.2.3.4", "Bot", visa_destination);
    let visa = Visa::new(order_id, visa_suspect.clone(), expires_from_now(1));
    let visa = Signed::sign(visa, b"Secret key");
    let suspect = Suspect::new(
        "1.2.3.4",
        "Bot",
        Destination::new("localhost", 80, "/child"),
    );
    let access_result = sut.check_access(&suspect, Some(visa));
    // Assert
    assert_is_allowed(&access_result);
}

#[test]
pub fn passing_gate_with_valid_visa_acquired_from_subresource_access_should_allow_access_to_all_resources_in_gate(
) {
    // Arrange
    let (sut, order_id) = setup(None);
    // Act
    let visa_destination = Destination::new("localhost", 80, "/child");
    let visa_suspect = Suspect::new("1.2.3.4", "Bot", visa_destination);
    let visa = Visa::new(order_id, visa_suspect.clone(), expires_from_now(1));
    let visa = Signed::sign(visa, b"Secret key");
    let suspect = Suspect::new("1.2.3.4", "Bot", Destination::new("localhost", 80, "/"));
    let access_result = sut.check_access(&suspect, Some(visa));
    // Assert
    assert_is_allowed(&access_result);
}

#[test]
pub fn passing_gate_should_include_child_resources_in_access_control() {
    // Arrange
    let (sut, _) = setup(None);
    // Act
    let destination = Destination::new("localhost", 80, "/child");
    let suspect = Suspect::new("1.2.3.4", "Bot", destination);
    let access_result = sut.check_access(&suspect, None);
    // Assert
    let access_result = access_result.unwrap_err();
    match access_result {
        AccessError::AccessDeniedError(_) => (),
        AccessError::DestinationNotFound(_) => panic!("Gate was not found!"),
    }
}

#[test_case(Destination::new("localhost", 80, "/otherroot/page/"))]
#[test_case(Destination::new("localhost", 80, "/wwroot/stuff/page/"))]
pub fn passing_gate_should_return_destination_not_found_for_mismatched_paths(
    destination: Destination,
) {
    // Arrange
    let order = Order::new(
        vec![Box::new(StubDescription::new(false))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new()),
    );
    let gate = Gate::new(
        Destination::new("localhost", 80, "/wwroot/page/"),
        vec![order],
    )
    .unwrap();
    let sut = setup_with_gates(vec![gate], None);
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bot", destination);
    let access_result = sut.check_access(&suspect, None);
    // Assert
    let access_result = access_result.expect_err("Tollkeeper allowed access to unguarded resource");
    match access_result {
        AccessError::AccessDeniedError(_) => {
            panic!("Unguarded resources should not be controlled by tollkeeper!")
        }
        AccessError::DestinationNotFound(_) => (),
    }
}

#[test]
pub fn passing_gate_with_expired_visa_should_return_new_toll() {
    // Arrange
    let current_time = chrono::Utc.with_ymd_and_hms(2025, 12, 1, 13, 0, 0).unwrap();
    let (sut, order_id) = setup(Some(current_time));
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bot", Destination::new_base("localhost"));
    let visa_expiry = current_time - chrono::Duration::seconds(1);
    let visa = Visa::new(
        OrderIdentifier::new(order_id.gate_id(), order_id.order_id()),
        suspect.clone(),
        visa_expiry,
    );
    let visa = Signed::sign(visa, b"Secret key");
    let access_result = sut.check_access(&suspect, Some(visa));
    // Assert
    assert_is_denied(&access_result);
}

#[test]
pub fn passing_gate_with_visa_for_unknown_order_should_return_new_toll() {
    // Arrange
    let (sut, order_id) = setup(None);
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bot", Destination::new_base("localhost"));
    let visa = Visa::new(
        OrderIdentifier::new(order_id.gate_id(), "not_an_order_id"),
        suspect.clone(),
        expires_from_now(1),
    );
    let visa = Signed::sign(visa, b"Secret key");
    let access_result = sut.check_access(&suspect, Some(visa));
    // Assert
    assert_is_denied(&access_result);
}

#[test]
pub fn passing_gate_with_visa_for_different_suspect_should_return_new_toll_for_current_suspect() {
    // Arrange
    let (sut, order_id) = setup(None);
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new_base("localhost"));
    let visa = Visa::new(
        order_id,
        Suspect::new("4.3.2.1", "Alice", Destination::new_base("localhost")),
        expires_from_now(1),
    );
    let visa = Signed::sign(visa, b"Secret key");
    let access_result = sut.check_access(&suspect, Some(visa));
    // Assert
    let toll = assert_is_denied(&access_result);
    let (_, toll) = toll.deconstruct();
    assert!(toll.recipient() == &suspect);
}

#[test]
pub fn passing_gate_with_visa_with_invalid_signature_should_reject_with_new_toll() {
    // Arrange
    let (sut, order_id) = setup(None);
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bot", Destination::new_base("localhost"));
    let visa = Visa::new(order_id.clone(), suspect.clone(), expires_from_now(1));
    let signed_visa = Signed::sign(visa, b"Secret key");
    let signature = signed_visa.signature();
    let forged_suspect = Suspect::new("11.22.33.44", "Bot", Destination::new_base("localhost"));
    let forged_visa = Visa::new(order_id, forged_suspect.clone(), expires_from_now(1));
    let forged_visa = Signed::new(forged_visa, signature.raw().to_vec());
    let access_result = sut.check_access(&forged_suspect, Some(forged_visa));
    // Assert
    let _ = assert_is_denied(&access_result);
}

#[test]
pub fn paying_toll_for_valid_order_with_valid_payment_should_return_visa() {
    // Arrange
    let (sut, order_id) = setup_with_payment(None);
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new_base("localhost"));
    let toll = Toll::new(suspect.clone(), order_id, Challenge::new());
    let toll = Signed::sign(toll, b"Secret key");
    let payment = SignedPayment::new(toll, "legal tender");
    let result = sut.pay_toll(&suspect, payment);
    // Assert
    let visa = match result {
        Ok(v) => v,
        Err(e) => panic!("Expected Visa got error: {e}"),
    };
    assert!(
        visa.verify(b"Secret key").is_ok(),
        "Got visa with invalid signature!"
    );
}

#[test]
pub fn paying_toll_for_valid_order_with_invalid_payment_should_return_error() {
    // Arrange
    let (sut, order_id) = setup(None);
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new_base("localhost"));
    let toll = Toll::new(suspect.clone(), order_id, Challenge::new());
    let toll = Signed::sign(toll, b"Secret key");
    let payment = SignedPayment::new(toll, "legal tender");
    let result = sut.pay_toll(&suspect, payment);
    // Assert
    assert!(result.is_err(), "Expected error, got visa!");
    match result.unwrap_err() {
        PaymentDeniedError::InvalidPayment(e) => {
            let toll = e.new_toll();
            assert!(
                toll.verify(b"Secret key").is_ok(),
                "Returned toll got invalid signature!"
            );
        }
        PaymentDeniedError::MismatchedSuspect(_) => {
            panic!("Expected invalid payment error, got error for mismatched suspect")
        }
        PaymentDeniedError::InvalidSignature => {
            panic!("Expected invalid payment error, got error for invalid signature")
        }
        PaymentDeniedError::GatewayError(_) => {
            panic!("Expected invalid payment error, got gateway error")
        }
    }
}

#[test]
pub fn paying_toll_for_different_suspect_should_return_new_toll_for_current_suspect() {
    // Arrange
    let (sut, order_id) = setup_with_payment(None);
    // Act
    let suspect_alice = Suspect::new("1.2.3.4", "Alice", Destination::new_base("localhost"));
    let suspect_bob = Suspect::new("90.1.2.6", "Bob", Destination::new_base("localhost"));
    let bobs_toll = Toll::new(suspect_bob.clone(), order_id, Challenge::new());
    let bobs_toll = Signed::sign(bobs_toll, b"Secret key");
    let payment = SignedPayment::new(bobs_toll, "legal tender");
    let result = sut.pay_toll(&suspect_alice, payment); // Alice pays with bobs toll!

    // Assert
    let err = match result {
        Result::Ok(_) => panic!("Returned visa despite different suspect paying!"),
        Result::Err(e) => e,
    };
    let err = match err {
        PaymentDeniedError::MismatchedSuspect(e) => e,
        _ => panic!("Unexpected failure: {err}"),
    };
    let toll = err.new_toll().verify(b"Secret key").unwrap();
    assert_eq!(toll.recipient(), &suspect_alice);
    assert_eq!(err.actual(), &suspect_alice);
    assert_eq!(err.expected(), &suspect_bob);
}

#[test]
pub fn paying_toll_for_unknown_gate_should_return_error() {
    // Arrange
    let (sut, order_id) = setup_with_payment(None);
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new_base("localhost"));
    let toll = Toll::new(
        suspect.clone(),
        OrderIdentifier::new("gate?", order_id.order_id()),
        Challenge::new(),
    );
    let toll = Signed::sign(toll, b"Secret key");
    let payment = SignedPayment::new(toll, "legal tender");
    let result = sut.pay_toll(&suspect, payment);
    // Assert
    let expected: Result<Signed<Visa>, PaymentDeniedError> =
        Err(GatewayError::MissingGate(MissingGateError::new("gate?")).into());
    assert_eq!(expected, result);
}

#[test]
pub fn paying_toll_for_unknown_order_should_return_error() {
    // Arrange
    let (sut, order_id) = setup_with_payment(None);
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new_base("localhost"));
    let toll = Toll::new(
        suspect.clone(),
        OrderIdentifier::new(order_id.gate_id(), "order?"),
        Challenge::new(),
    );
    let toll = Signed::sign(toll, b"Secret key");
    let payment = SignedPayment::new(toll, "legal tender");
    let result = sut.pay_toll(&suspect, payment);
    // Assert
    let expected: Result<Signed<Visa>, PaymentDeniedError> = Err(GatewayError::MissingOrder(
        MissingOrderError::new(order_id.gate_id(), "order?"),
    )
    .into());
    assert_eq!(expected, result);
}

#[test]
pub fn paying_toll_with_forged_toll_should_return_error_without_new_toll() {
    // Arrange
    let (sut, order_id) = setup_with_payment(None);
    // Act
    let real_suspect = Suspect::new("1.2.3.4", "Bob", Destination::new_base("localhost"));
    let real_toll = Toll::new(
        real_suspect.clone(),
        OrderIdentifier::new(order_id.gate_id(), "order?"),
        Challenge::new(),
    );
    let real_toll = Signed::sign(real_toll, b"Secret key");
    let forged_suspect = Suspect::new("11.22.33.44", "Alice", Destination::new_base("localhost"));
    let forged_toll = Toll::new(
        forged_suspect.clone(),
        OrderIdentifier::new(order_id.gate_id(), order_id.order_id()),
        Challenge::new(),
    );
    let signature = real_toll.signature().raw().to_vec();
    let forged_toll = Signed::new(forged_toll, signature);
    let payment = SignedPayment::new(forged_toll, "legal tender");
    let result = sut.pay_toll(&forged_suspect, payment);
    // Assert
    let expected: Result<Signed<Visa>, PaymentDeniedError> =
        Err(PaymentDeniedError::InvalidSignature);
    assert_eq!(expected, result);
}
