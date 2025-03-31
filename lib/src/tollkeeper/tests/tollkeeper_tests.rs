use super::mocks::*;
use crate::tollkeeper::*;

#[test]
pub fn accessing_blacklisted_destination_without_matching_description_should_allow_access() {
    // Arrange
    let no_match_description: Box<dyn Description> = Box::new(StubDescription::new(false));
    let order = Order::new(vec![no_match_description], GateStatus::Blacklist);
    let gate = Gate::new(String::from("localhost"), vec![order]);
    let sut = TollkeeperImpl::new(vec![gate]);
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
        "Returned a challenge even though access should be granted!"
    );
    assert!(
        benign_suspect.is_accessed(),
        "Destination was not accessed despite allowed!"
    );
}

#[test]
pub fn accessing_whitelisted_destination_without_matching_description_should_return_challenge() {
    // Arrange
    let no_match_description: Box<dyn Description> = Box::new(StubDescription::new(false));
    let order = Order::new(vec![no_match_description], GateStatus::Whitelist);
    let gate = Gate::new(String::from("localhost"), vec![order]);
    let sut = TollkeeperImpl::new(vec![gate]);
    // Act
    let mut malicious_suspect = SpySuspect::new("1.2.3.4", "BadCrawler", "localhost", "/");
    let result = sut.guarded_access::<SpySuspect>(&mut malicious_suspect, |req| {
        req.access();
    });
    let malicious_suspect = malicious_suspect;
    // Assert
    assert_eq!(
        Option::Some(Toll::new("challenge")),
        result,
        "Returned no challenge despite default set to allow and no gates triggered!"
    );
    assert!(
        !malicious_suspect.is_accessed(),
        "Destination was accessed despite allowed!"
    );
}

#[test]
pub fn accessing_blacklisted_destination_with_matching_description_should_return_challenge() {
    // Arrange
    let match_description: Box<dyn Description> = Box::new(StubDescription::new(true));
    let order = Order::new(vec![match_description], GateStatus::Blacklist);
    let gate = Gate::new(String::from("localhost"), vec![order]);
    let sut = TollkeeperImpl::new(vec![gate]);
    // Act
    let mut malicious_suspect = SpySuspect::new("1.2.3.4", "BadCrawler", "localhost", "/");
    let result = sut.guarded_access::<SpySuspect>(&mut malicious_suspect, |req| {
        req.access();
    });
    let malicious_suspect = malicious_suspect;
    // Assert
    assert_eq!(
        Option::Some(Toll::new("challenge")),
        result,
        "Did not return a challenge despite triggering trap!"
    );
    assert!(
        !malicious_suspect.is_accessed(),
        "Destination was accessed despite triggering trap!"
    );
}

#[test]
pub fn accessing_whitelisted_destination_with_matching_description_should_allow_access() {
    // Arrange
    let match_description: Box<dyn Description> = Box::new(StubDescription::new(true));
    let order = Order::new(vec![match_description], GateStatus::Whitelist);
    let gate = Gate::new(String::from("localhost"), vec![order]);
    let sut = TollkeeperImpl::new(vec![gate]);
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
        "Returned a challenge despite default set to allow gates triggered!"
    );
    assert!(
        benign_suspect.is_accessed(),
        "Destination was not accessed despite allowed!"
    );
}
