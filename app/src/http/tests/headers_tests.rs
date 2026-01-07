use crate::http::Headers;
use indexmap::IndexMap;
use pretty_assertions::assert_eq;
use test_case::test_case;

#[test_case("User-Agent" ; "normal case")]
#[test_case("user-agent" ; "all lowercase")]
#[test_case("user-Agent" ; "first char first word lowercase")]
#[test_case("User-agent" ; "first char second word lowercase")]
#[test_case("USER-AGENT" ; "ALL CAPS")]
pub fn get_by_key_should_be_case_insensitive(key: &str) {
    // Assert
    let mut headers = IndexMap::<String, String>::new();
    headers.insert("User-Agent".into(), "bob".into());
    let sut = Headers::new(headers);
    // Act
    let result = sut.get(key);
    // Assert
    assert!(result.is_some(), "Header not found!");
    assert_eq!("bob", result.unwrap());
}
