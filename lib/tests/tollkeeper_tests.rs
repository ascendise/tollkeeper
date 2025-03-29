use ::tollkeeper::tollkeeper::Host;
use tollkeeper::tollkeeper::{Operation, Request, Tollkeeper, TollkeeperImpl};

#[test]
pub fn accessing_guarded_endpoint_without_tripping_filters_should_allow_access() {
    // Arrange
    let hosts = vec![Host::new("localhost", Operation::Challenge, vec![])];
    let sut = TollkeeperImpl::new(hosts);
    // Act
    let benign_request = Request::new("1.2.3.4", "FriendlyCrawler", "localhost", "/");
    let mut request_granted = false;
    let result = sut.access(benign_request, |()| request_granted = true);
    // Assert
    assert_eq!(
        Option::None,
        result,
        "Returned a challenge even tho access should be granted!"
    );
    assert_eq!(
        true, request_granted,
        "No challenge returned but on_access was not run!"
    )
}

#[test]
pub fn accessing_guarded_endpoint_and_tripping_filters_should_return_challenge() {
    // Arrange
    // Act
    // Assert
}
