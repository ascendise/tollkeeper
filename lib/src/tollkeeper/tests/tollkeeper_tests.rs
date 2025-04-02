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
    let gate = Gate::new("localhost", vec![order]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let mut benign_suspect = SpySuspect::new("1.2.3.4", "FriendlyCrawler", "localhost", "/");
    let result = sut.guarded_access::<SpySuspect>(&mut benign_suspect, |req| {
        req.access();
    });
    let benign_suspect = benign_suspect;
    // Assert
    assert_eq!(
        Option::None,
        result,
        "Requires toll even though access should be granted!"
    );
    assert!(
        benign_suspect.is_accessed(),
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
    let gate = Gate::new("localhost", vec![order]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let mut malicious_suspect = SpySuspect::new("1.2.3.4", "BadCrawler", "localhost", "/");
    let result = sut.guarded_access::<SpySuspect>(&mut malicious_suspect, |req| {
        req.access();
    });
    let malicious_suspect = malicious_suspect;
    // Assert
    assert_eq!(
        Option::Some(toll),
        result,
        "Required no toll despite suspect not matching whitelist order description"
    );
    assert!(
        !malicious_suspect.is_accessed(),
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
    let gate = Gate::new("localhost", vec![order]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let mut malicious_suspect = SpySuspect::new("1.2.3.4", "BadCrawler", "localhost", "/");
    let result = sut.guarded_access::<SpySuspect>(&mut malicious_suspect, |req| {
        req.access();
    });
    let malicious_suspect = malicious_suspect;
    // Assert
    assert_eq!(
        Option::Some(toll),
        result,
        "Did not require a toll despite matching description on blacklist order!"
    );
    assert!(
        !malicious_suspect.is_accessed(),
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
    let gate = Gate::new("localhost", vec![order]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let mut benign_suspect = SpySuspect::new("1.2.3.4", "FriendlyCrawler", "localhost", "/");
    let result = sut.guarded_access::<SpySuspect>(&mut benign_suspect, |req| {
        req.access();
    });
    let benign_suspect = benign_suspect;
    // Assert
    assert_eq!(
        Option::None,
        result,
        "Required a toll despite suspect not matching whitelist order"
    );
    assert!(
        benign_suspect.is_accessed(),
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
    let gate = Gate::new("localhost", vec![order1, order2, order3]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let mut malicious_suspect = SpySuspect::new("1.2.3.4", "FriendlyCrawler", "localhost", "/");
    let result = sut.guarded_access::<SpySuspect>(&mut malicious_suspect, |req| {
        req.access();
    });
    let malicious_suspect = malicious_suspect;
    // Assert
    assert_eq!(
        Option::Some(toll),
        result,
        "Required no toll despite first matching order being a blacklist"
    );
    assert!(
        !malicious_suspect.is_accessed(),
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
    let gate = Gate::new("localhost", vec![order1, order2, order3]).unwrap();
    let sut = TollkeeperImpl::new(vec![gate]).unwrap();
    // Act
    let mut benign_suspect = SpySuspect::new("1.2.3.4", "FriendlyCrawler", "localhost", "/");
    let result = sut.guarded_access::<SpySuspect>(&mut benign_suspect, |req| {
        req.access();
    });
    let benign_suspect = benign_suspect;
    // Assert
    assert_eq!(
        Option::None,
        result,
        "Required a toll despite first matching order being a whitelist"
    );
    assert!(
        benign_suspect.is_accessed(),
        "Destination was not accessed despite allowed!"
    );
}
