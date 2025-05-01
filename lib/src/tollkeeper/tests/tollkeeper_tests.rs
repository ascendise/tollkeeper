use super::*;
use crate::tollkeeper::*;
use test_case::test_case;

#[test]
pub fn creating_new_toolkeeper_with_no_gates_should_fail() {
    // Arrange
    // Act
    let result = TollkeeperImpl::new(vec![]);
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
    let suspect = Suspect::new("1.2.3.4", "FriendlyCrawler", Destination::new("localhost"));
    let description: Box<dyn Description> = Box::new(StubDescription::new(matches_description));
    let order = Order::new(
        vec![description],
        access_policy,
        Box::new(StubDeclaration::new()),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![order]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let mut request = SpyRequest::new();
    let result = sut.guarded_access::<SpyRequest>(&suspect, &Option::None, &mut request, |req| {
        req.access();
    });
    // Assert
    assert_eq!(
        Option::None,
        result,
        "Requires toll even though access should be granted!"
    );
    assert!(
        request.accessed(),
        "Destination was not accessed despite allowed!"
    );
}

#[test_case(AccessPolicy::Blacklist, true ; "accessing a gate with a matching blacklist order description")]
#[test_case(AccessPolicy::Whitelist, false ; "accessing a gate with a whitelist order and not matching description")]
pub fn should_require_toll_if_matching_toll_requirement(
    access_policy: AccessPolicy,
    matches_description: bool,
) {
    // Arrange
    let suspect = Suspect::new("1.2.3.4", "BadCrawler", Destination::new("localhost"));
    let description: Box<dyn Description> = Box::new(StubDescription::new(matches_description));
    let order = Order::new(
        vec![description],
        access_policy,
        Box::new(StubDeclaration::new()),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![order]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let mut request = SpyRequest::new();
    let result = sut.guarded_access::<SpyRequest>(&suspect, &Option::None, &mut request, |req| {
        req.access();
    });
    // Assert
    assert!(
        result.is_some(),
        "Required no toll despite suspect not matching whitelist order description",
    );
    assert!(
        !request.accessed(),
        "Destination was accessed despite no toll was paid!"
    );
}

#[test]
pub fn passing_gate_with_first_matching_order_requiring_toll_should_return_toll() {
    // Arrange
    let malicious_suspect =
        Suspect::new("1.2.3.4", "FriendlyCrawler", Destination::new("localhost"));
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
        Destination::new("localhost"),
        vec![first_order, matching_order, last_order],
    )
    .unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let mut request = SpyRequest::new();
    let result =
        sut.guarded_access::<SpyRequest>(&malicious_suspect, &Option::None, &mut request, |req| {
            req.access();
        });
    // Assert
    assert!(
        result.is_some(),
        "Required no toll despite first matching order being a blacklist"
    );
    assert!(
        !request.accessed(),
        "Destination was not accessed despite allowed!"
    );
}

#[test]
pub fn passing_gate_with_first_matching_order_allowing_access_should_allow_access() {
    // Arrange
    let benign_suspect = Suspect::new("1.2.3.4", "FriendlyCrawler", Destination::new("localhost"));
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
        Destination::new("localhost"),
        vec![first_order, matching_order, last_order],
    )
    .unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let mut request = SpyRequest::new();
    let result =
        sut.guarded_access::<SpyRequest>(&benign_suspect, &Option::None, &mut request, |req| {
            req.access();
        });
    // Assert
    assert_eq!(
        Option::None,
        result,
        "Required a toll despite first matching order being a whitelist"
    );
    assert!(
        request.accessed(),
        "Destination was not accessed despite allowed!"
    );
}

#[test]
pub fn passing_gate_with_valid_visa_should_allow_access() {
    // Arrange
    let suspect = Suspect::new("1.2.3.4", "Bot", Destination::new("localhost"));
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new()),
    );
    let order_id = require_payment_order.id.clone();
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let gate_id = gate.id.clone();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let visa = Visa::new(OrderIdentifier::new(gate_id, order_id), suspect.clone());
    let mut request = SpyRequest::new();
    let result =
        sut.guarded_access::<SpyRequest>(&suspect, &Option::Some(visa), &mut request, |req| {
            req.access();
        });
    // Assert
    assert_eq!(Option::None, result, "Required a toll despite having visa!");
    assert!(request.accessed(), "No access despite having visa");
}

#[test]
pub fn passing_gate_with_visa_for_unknown_order_should_return_new_toll() {
    // Arrange
    let suspect = Suspect::new("1.2.3.4", "Bot", Destination::new("localhost"));
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new()),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let gate_id = gate.id.clone();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let visa = Visa::new(
        OrderIdentifier::new(gate_id, "not_an_order_id"),
        suspect.clone(),
    );
    let mut request = SpyRequest::new();
    let result =
        sut.guarded_access::<SpyRequest>(&suspect, &Option::Some(visa), &mut request, |req| {
            req.access();
        });
    // Assert
    assert!(
        result.is_some(),
        "Did not return new toll despite suspect being a different one!"
    );
    assert!(!request.accessed(), "Was accessed despite not having visa!");
}

#[test]
pub fn passing_gate_with_visa_for_different_suspect_should_return_new_toll_for_current_suspect() {
    // Arrange
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new("localhost"));
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new()),
    );
    let order_id = require_payment_order.id.clone();
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let gate_id = gate.id.clone();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let visa = Visa::new(
        OrderIdentifier::new(gate_id, order_id),
        Suspect::new("4.3.2.1", "Alice", Destination::new("localhost")),
    );
    let mut request = SpyRequest::new();
    let result =
        sut.guarded_access::<SpyRequest>(&suspect, &Option::Some(visa), &mut request, |req| {
            req.access();
        });
    // Assert
    assert!(result.is_some());
    assert!(result.unwrap().recipient == suspect);
    assert!(!request.accessed(), "Was accessed despite not having visa!");
}

#[test]
pub fn buying_visa_for_valid_order_with_valid_payment_should_return_visa() {
    // Arrange
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new("localhost"));
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new_payment_stub()),
    );
    let order_id = require_payment_order.id.clone();
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let gate_id = gate.id.clone();
    let toll = Toll::new(
        suspect.clone(),
        OrderIdentifier::new(gate_id, order_id),
        HashMap::new(),
    );
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let payment = Payment::new(toll, "legal tender");
    let result = sut.buy_visa(&suspect, &payment);
    assert!(
        result.is_ok_and(|r| r.is_ok()),
        "Failed to buy visa with valid payment"
    );
}

#[test]
pub fn buying_visa_for_valid_order_with_invalid_payment_should_return_visa() {
    // Arrange
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new("localhost"));
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new()),
    );
    let order_id = require_payment_order.id.clone();
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let gate_id = gate.id.clone();
    let toll = Toll::new(
        suspect.clone(),
        OrderIdentifier::new(gate_id, order_id),
        HashMap::new(),
    );
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let payment = Payment::new(toll, "legal tender");
    let result = sut.buy_visa(&suspect, &payment);
    assert!(
        result.is_ok_and(|r| r.is_err()),
        "Was able to buy visa without valid payment"
    );
}

#[test]
pub fn buying_visa_for_different_suspect_should_return_new_toll_for_current_suspect() {
    // Arrange
    let suspect_alice = Suspect::new("1.2.3.4", "Alice", Destination::new("localhost"));
    let suspect_bob = Suspect::new("90.1.2.6", "Bob", Destination::new("localhost"));
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new_payment_stub()),
    );
    let order_id = require_payment_order.id.clone();
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let gate_id = gate.id.clone();
    let bobs_toll = Toll::new(
        suspect_bob.clone(),
        OrderIdentifier::new(gate_id, order_id),
        HashMap::new(),
    );
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let payment = Payment::new(bobs_toll, "legal tender");
    let result = sut.buy_visa(&suspect_alice, &payment); // Alice pays with bobs toll!
    let err = match result.unwrap() {
        Result::Ok(_) => panic!("Returned visa despite different suspect paying!"),
        Result::Err(e) => e,
    };
    let err = match err {
        PaymentDeniedError::MismatchedSuspect(e) => e,
        PaymentDeniedError::InvalidPayment(_) => {
            panic!("Unexpected failure do to unexpected payment")
        }
    };
    assert_eq!(err.new_toll().recipient, suspect_alice);
}

#[test]
pub fn buying_visa_for_unknown_gate_should_return_error() {
    // Arrange
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new("localhost"));
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new_payment_stub()),
    );
    let order_id = require_payment_order.id.clone();
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let toll = Toll::new(
        suspect.clone(),
        OrderIdentifier::new("gate?", order_id),
        HashMap::new(),
    );
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let payment = Payment::new(toll, "legal tender");
    let result = sut.buy_visa(&suspect, &payment);
    let expected: Result<Result<Visa, PaymentDeniedError>, GatewayError> =
        Result::Err(MissingGateError::new("gate?").into());
    assert_eq!(expected, result);
}

#[test]
pub fn buying_visa_for_unknown_order_should_return_error() {
    // Arrange
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new("localhost"));
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new_payment_stub()),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let gate_id = gate.id.clone();
    let toll = Toll::new(
        suspect.clone(),
        OrderIdentifier::new(&gate_id, "order?"),
        HashMap::new(),
    );
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let payment = Payment::new(toll, "legal tender");
    let result = sut.buy_visa(&suspect, &payment);
    let expected: Result<Result<Visa, PaymentDeniedError>, GatewayError> =
        Result::Err(MissingOrderError::new(gate_id, "order?").into());
    assert_eq!(expected, result);
}
