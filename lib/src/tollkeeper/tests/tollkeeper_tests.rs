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
    let description: Box<dyn Description> = Box::new(StubDescription::new(matches_description));
    let toll = Toll::new(ChallengeAlgorithm::SHA1, "abcd", 4);
    let order = Order::new(
        vec![description],
        access_policy,
        Box::new(StubDeclaration::new(toll)),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![order]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let suspect = Suspect::new("1.2.3.4", "FriendlyCrawler", Destination::new("localhost"));
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
    let description: Box<dyn Description> = Box::new(StubDescription::new(matches_description));
    let toll = Toll::new(ChallengeAlgorithm::SHA1, "abcd", 4);
    let order = Order::new(
        vec![description],
        access_policy,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![order]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let suspect = Suspect::new("1.2.3.4", "BadCrawler", Destination::new("localhost"));
    let mut request = SpyRequest::new();
    let result = sut.guarded_access::<SpyRequest>(&suspect, &Option::None, &mut request, |req| {
        req.access();
    });
    // Assert
    assert_eq!(
        Option::Some(toll),
        result,
        "Required no toll despite suspect not matching whitelist order description"
    );
    assert!(
        !request.accessed(),
        "Destination was accessed despite no toll was paid!"
    );
}

#[test]
pub fn passing_gate_with_first_matching_order_requiring_toll_should_return_toll() {
    // Arrange
    let toll = Toll::new(ChallengeAlgorithm::SHA1, "abcd", 4);
    let first_order = Order::new(
        vec![Box::new(StubDescription::new(false))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let matching_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let last_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Whitelist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let gate = Gate::new(
        Destination::new("localhost"),
        vec![first_order, matching_order, last_order],
    )
    .unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let malicious_suspect =
        Suspect::new("1.2.3.4", "FriendlyCrawler", Destination::new("localhost"));
    let mut request = SpyRequest::new();
    let result =
        sut.guarded_access::<SpyRequest>(&malicious_suspect, &Option::None, &mut request, |req| {
            req.access();
        });
    // Assert
    assert_eq!(
        Option::Some(toll),
        result,
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
    let toll = Toll::new(ChallengeAlgorithm::SHA1, "abcd", 4);
    let first_order = Order::new(
        vec![Box::new(StubDescription::new(false))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let matching_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Whitelist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let last_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let gate = Gate::new(
        Destination::new("localhost"),
        vec![first_order, matching_order, last_order],
    )
    .unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let benign_suspect = Suspect::new("1.2.3.4", "FriendlyCrawler", Destination::new("localhost"));
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
    let toll = Toll::new(ChallengeAlgorithm::SHA1, "abcd", 4);
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let order_id = require_payment_order.id.clone();
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let gate_id = gate.id.clone();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bot", Destination::new("localhost"));
    let visa = Visa::new(&gate_id, &order_id, suspect.clone());
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
    let new_toll = Toll::new(ChallengeAlgorithm::SHA3, "gofuckyourself", 99);
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(new_toll.clone())),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let gate_id = gate.id.clone();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bot", Destination::new("localhost"));
    let visa = Visa::new(&gate_id, "not_an_order_id", suspect.clone());
    let mut request = SpyRequest::new();
    let result =
        sut.guarded_access::<SpyRequest>(&suspect, &Option::Some(visa), &mut request, |req| {
            req.access();
        });
    // Assert
    assert_eq!(Option::Some(new_toll), result);
    assert!(!request.accessed(), "Was accessed despite not having visa!");
}

#[test_case(true, |gate_id, order_id, suspect, _| Result::Ok(Visa::new(gate_id, order_id, suspect)) ; "with valid payment should return visa")]
#[test_case(false, |_, _, _, toll| Result::Err(toll) ; "with invalid payment should return toll")]
pub fn buying_visa_for_valid_order(
    accept_payment: bool,
    expected_result: impl Fn(&str, &str, Suspect, Toll) -> Result<Visa, Toll>,
) {
    // Arrange
    let new_toll = Toll::new(ChallengeAlgorithm::SHA3, "gofuckyourself", 99);
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new_payment_stub(
            new_toll.clone(),
            accept_payment,
        )),
    );
    let order_id = require_payment_order.id.clone();
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let gate_id = gate.id.clone();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new("localhost"));
    let payment = Payment::new(&gate_id, &order_id, "legal tender");
    let result = sut.buy_visa(&suspect, &payment);
    assert_eq!(
        Result::Ok(expected_result(&gate_id, &order_id, suspect, new_toll)),
        result
    );
}

#[test]
pub fn buying_visa_for_unknown_gate_should_return_error() {
    // Arrange
    let new_toll = Toll::new(ChallengeAlgorithm::SHA3, "gofuckyourself", 99);
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new_payment_stub(new_toll.clone(), true)),
    );
    let order_id = require_payment_order.id.clone();
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new("localhost"));
    let payment = Payment::new("wrong gate number", &order_id, "legal tender");
    let result = sut.buy_visa(&suspect, &payment);
    let expected: Result<Result<Visa, Toll>, GatewayError> =
        Result::Err(MissingGateError::new("wrong gate number").into());
    assert_eq!(expected, result);
}

#[test]
pub fn buying_visa_for_unknown_order_should_return_error() {
    // Arrange
    let new_toll = Toll::new(ChallengeAlgorithm::SHA3, "gofuckyourself", 99);
    let require_payment_order = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new_payment_stub(new_toll.clone(), true)),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![require_payment_order]).unwrap();
    let gate_id = gate.id.clone();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bob", Destination::new("localhost"));
    let payment = Payment::new(&gate_id, "wrong order number", "legal tender");
    let result = sut.buy_visa(&suspect, &payment);
    let expected: Result<Result<Visa, Toll>, GatewayError> =
        Result::Err(MissingOrderError::new(gate_id, "wrong order number").into());
    assert_eq!(expected, result);
}
