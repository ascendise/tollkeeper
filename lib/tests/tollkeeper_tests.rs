use ::tollkeeper::tollkeeper::Host;
use tollkeeper::tollkeeper::{Operation, Request, Tollkeeper, TollkeeperImpl};

#[test]
pub fn accessing_guarded_endpoint_without_tripping_filters_should_return_no_challenge() {
    // Arrange
    let hosts = vec![Host::new("localhost", Operation::Challenge, vec![])];
    let sut = TollkeeperImpl::new(hosts);
    // Act
    let benign_request = SpyRequest::new("1.2.3.4", "FriendlyCrawler", "localhost", "/");
    let result = sut.access::<SpyRequest>(&benign_request, |_| {});
    // Assert
    assert_eq!(
        Option::None,
        result,
        "Returned a challenge even tho access should be granted!"
    );
}

#[test]
pub fn accessing_guarded_endpoint_without_tripping_filters_should_allow_access() {
    // Arrange
    let hosts = vec![Host::new("localhost", Operation::Challenge, vec![])];
    let sut = TollkeeperImpl::new(hosts);
    // Act
    let benign_request = SpyRequest::new("1.2.3.4", "FriendlyCrawler", "localhost", "/");
    _ = sut.access::<SpyRequest>(&benign_request, |_| {
        assert!(true);
    });
    // Assert
    assert!(false, "Request was not processed!")
}

#[test]
pub fn accessing_guarded_endpoint_and_tripping_filters_should_return_challenge() {
    // Arrange
    // Act
    // Assert
}

pub struct SpyRequest {
    client_ip: String,
    user_agent: String,
    target_host: String,
    target_path: String,
}

impl SpyRequest {
    pub fn new(
        client_ip: impl Into<String>,
        user_agent: impl Into<String>,
        target_host: impl Into<String>,
        target_path: impl Into<String>,
    ) -> Self {
        Self {
            client_ip: client_ip.into(),
            user_agent: user_agent.into(),
            target_host: target_host.into(),
            target_path: target_path.into(),
        }
    }
}

impl Request for SpyRequest {
    fn client_ip(self: &Self) -> &str {
        &self.client_ip
    }
    fn user_agent(self: &Self) -> &str {
        &self.user_agent
    }
    fn target_host(self: &Self) -> &str {
        &self.target_host
    }
    fn target_path(self: &Self) -> &str {
        &self.target_path
    }
}
