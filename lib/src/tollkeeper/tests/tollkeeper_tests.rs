use super::*;
use crate::tollkeeper::*;

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

#[test]
pub fn passing_gate_with_a_blacklist_order_but_no_matching_description_should_allow_access() {
    // Arrange
    let no_match_description: Box<dyn Description> = Box::new(StubDescription::new(false));
    let toll = Toll::new(ChallengeAlgorithm::SHA1, "abcd", 4);
    let order = Order::new(
        vec![no_match_description],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(toll)),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![order]).unwrap();
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
        "Requires toll even though access should be granted!"
    );
    assert!(
        request.accessed(),
        "Destination was not accessed despite allowed!"
    );
}

#[test]
pub fn passing_gate_with_a_whitelist_order_but_no_matching_description_should_request_toll() {
    // Arrange
    let no_match_description: Box<dyn Description> = Box::new(StubDescription::new(false));
    let toll = Toll::new(ChallengeAlgorithm::SHA1, "abcd", 4);
    let order = Order::new(
        vec![no_match_description],
        AccessPolicy::Whitelist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![order]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let malicious_suspect = Suspect::new("1.2.3.4", "BadCrawler", Destination::new("localhost"));
    let mut request = SpyRequest::new();
    let result =
        sut.guarded_access::<SpyRequest>(&malicious_suspect, &Option::None, &mut request, |req| {
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
pub fn passing_gate_with_a_blacklist_order_and_matching_description_should_request_toll() {
    // Arrange
    let match_description: Box<dyn Description> = Box::new(StubDescription::new(true));
    let toll = Toll::new(ChallengeAlgorithm::SHA1, "abcd", 4);
    let order = Order::new(
        vec![match_description],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![order]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let malicious_suspect = Suspect::new("1.2.3.4", "BadCrawler", Destination::new("localhost"));
    let mut request = SpyRequest::new();
    let result =
        sut.guarded_access::<SpyRequest>(&malicious_suspect, &Option::None, &mut request, |req| {
            req.access();
        });
    // Assert
    assert_eq!(
        Option::Some(toll),
        result,
        "Did not require a toll despite matching description on blacklist order!"
    );
    assert!(
        !request.accessed(),
        "Destination was accessed despite triggering trap!"
    );
}

#[test]
pub fn passing_gate_with_a_whitelist_order_and_matching_description_should_allow_access() {
    // Arrange
    let match_description: Box<dyn Description> = Box::new(StubDescription::new(true));
    let toll = Toll::new(ChallengeAlgorithm::SHA1, "abcd", 4);
    let order = Order::new(
        vec![match_description],
        AccessPolicy::Whitelist,
        Box::new(StubDeclaration::new(toll)),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![order]).unwrap();
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
        "Required a toll despite suspect not matching whitelist order"
    );
    assert!(
        request.accessed(),
        "Destination was not accessed despite allowed!"
    );
}

#[test]
pub fn passing_gate_with_first_matching_order_requiring_toll_should_return_toll() {
    // Arrange
    let toll = Toll::new(ChallengeAlgorithm::SHA1, "abcd", 4);
    let order1 = Order::new(
        vec![Box::new(StubDescription::new(false))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let order2 = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let order3 = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Whitelist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![order1, order2, order3]).unwrap();
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
    let order1 = Order::new(
        vec![Box::new(StubDescription::new(false))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let order2 = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Whitelist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let order3 = Order::new(
        vec![Box::new(StubDescription::new(true))],
        AccessPolicy::Blacklist,
        Box::new(StubDeclaration::new(toll.clone())),
    );
    let gate = Gate::new(Destination::new("localhost"), vec![order1, order2, order3]).unwrap();
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
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bot", Destination::new("localhost"));
    let visa = Visa::new(order_id, suspect.clone());
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
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let suspect = Suspect::new("1.2.3.4", "Bot", Destination::new("localhost"));
    let visa = Visa::new("not_an_order_id", suspect.clone());
    let mut request = SpyRequest::new();
    let result =
        sut.guarded_access::<SpyRequest>(&suspect, &Option::Some(visa), &mut request, |req| {
            req.access();
        });
    // Assert
    assert_eq!(Option::Some(new_toll), result);
    assert!(!request.accessed(), "Was accessed despite not having visa!");
}
