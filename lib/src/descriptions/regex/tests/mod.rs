use test_case::test_case;

use super::RegexDescription;
use crate::descriptions::*;

fn test_suspect() -> Suspect {
    let destination = Destination::new("https://example.com", 1443, "/api");
    Suspect::new("1.2.3.4", "Netscape 9.1", destination)
}

#[test_case("client_ip", r"^1\.2\.3\.4$" ; "check client_ip")]
#[test_case("user_agent", r"^Netscape 9.1$" ; "check user_agent")]
#[test_case("destination", r"^https://example.com:1443/api$" ; "check destination")]
pub fn matches_should_search_for_specified_key_in_suspect(key: &str, regex: &str) {
    //Arrange
    let suspect = test_suspect();
    let sut = RegexDescription::new(key, regex, false).unwrap();
    //Act
    let is_match = sut.matches(&suspect);
    //Assert
    assert!(is_match);
}

#[test]
pub fn matches_should_return_false_if_no_match() {
    //Arrange
    let suspect = test_suspect();
    let sut = RegexDescription::new("user_agent", "NoThisDoesNotMatch", false).unwrap();
    //Act
    let is_match = sut.matches(&suspect);
    //Assert
    assert!(!is_match);
}

#[test]
pub fn matches_should_check_regex_with_negative_lookahead() {
    //Arrange
    let suspect = test_suspect();
    let ip_regex = r"^192\.1\.2\.3"; // Only matches one ip
    let sut = RegexDescription::new("client_ip", ip_regex, true).unwrap(); //Now matches
                                                                           //all ips expect ours
                                                                           //Act
    let is_match = sut.matches(&suspect);
    //Assert
    assert!(is_match);
}

#[test]
pub fn matches_should_return_false_if_matches_on_negative_lookahead() {
    //Arrange
    let suspect = test_suspect();
    let sut = RegexDescription::new("user_agent", "Netscape 9.1", true).unwrap();
    //Act
    let is_match = sut.matches(&suspect);
    //Assert
    assert!(!is_match);
}
