use crate::http::Headers;
use pretty_assertions::assert_eq;
use test_case::test_case;

pub fn to_string_should_return_all_headers() {
    // Arrange
    let sut = Headers::new(vec![
        ("Hello".into(), "World".into()),
        ("Cookie".into(), "Foo".into()),
        ("Cookie".into(), "Bar".into()),
    ]);
    // Act
    let headers_str = sut.to_string();
    // Assert
    let expected_headers = "Hello: World\r\nCookie: Foo\r\nCookie: Bar\r\n";
    assert_eq!(expected_headers, headers_str);
}

#[test_case("User-Agent" ; "normal case")]
#[test_case("user-agent" ; "all lowercase")]
#[test_case("user-Agent" ; "first char first word lowercase")]
#[test_case("User-agent" ; "first char second word lowercase")]
#[test_case("USER-AGENT" ; "ALL CAPS")]
pub fn get_by_key_should_be_case_insensitive(key: &str) {
    // Arrange
    let sut = Headers::new(vec![("User-Agent".into(), "bob".into())]);
    // Act
    let result = sut.get(key);
    // Assert
    assert!(result.is_some(), "Header not found!");
    assert_eq!("bob", result.unwrap());
}
