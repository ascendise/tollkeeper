use super::mocks::*;
use crate::tollkeeper::*;

#[test]
pub fn accessing_blacklisted_destination_without_matching_description_should_allow_access() {
    // Arrange
    let hosts = vec![Destination::new("localhost", GateStatus::Blacklist, vec![])];
    let sut = TollkeeperImpl::new(hosts);
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
        "Returned a challenge even tho access should be granted!"
    );
    assert!(
        benign_suspect.is_accessed(),
        "Destination was not accessed despite allowed!"
    );
}

#[test]
pub fn accessing_whitelisted_destination_without_matching_description_should_return_challenge() {
    // Arrange
    let hosts = vec![Destination::new("localhost", GateStatus::Whitelist, vec![])];
    let sut = TollkeeperImpl::new(hosts);
    // Act
    let mut malicious_suspect = SpySuspect::new("1.2.3.4", "BadCrawler", "localhost", "/");
    let result = sut.guarded_access::<SpySuspect>(&mut malicious_suspect, |req| {
        req.access();
    });
    let malicious_suspect = malicious_suspect;
    // Assert
    assert_eq!(
        Option::Some(Challenge::new("challenge")),
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
    let gates: Vec<Box<dyn Gate>> = vec![Box::new(StubGate::new(true))];
    let hosts = vec![Destination::new("localhost", GateStatus::Blacklist, gates)];
    let sut = TollkeeperImpl::new(hosts);
    // Act
    let mut malicious_suspect = SpySuspect::new("1.2.3.4", "BadCrawler", "localhost", "/");
    let result = sut.guarded_access::<SpySuspect>(&mut malicious_suspect, |req| {
        req.access();
    });
    let malicious_suspect = malicious_suspect;
    // Assert
    assert_eq!(
        Option::Some(Challenge::new("challenge")),
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
    let gates: Vec<Box<dyn Gate>> = vec![Box::new(StubGate::new(true))];
    let hosts = vec![Destination::new("localhost", GateStatus::Whitelist, gates)];
    let sut = TollkeeperImpl::new(hosts);
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
