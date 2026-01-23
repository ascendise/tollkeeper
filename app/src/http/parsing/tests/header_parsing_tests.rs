use pretty_assertions::assert_eq;
use std::io::BufReader;

use crate::http::{Headers, Parse};

#[test]
fn parse_should_return_headers_from_stream() {
    // Arrange
    let raw_request = "Hello: World\r\nCookie: Foo\r\nCookie: Bar\r\n\r\n";
    let mut raw_headers = BufReader::new(raw_request.as_bytes());
    // Act
    let headers =
        Headers::parse(&mut raw_headers).expect("Failed to parse perfectly valid headers");
    // Assert
    let expected_headers = vec![
        ("Hello".into(), "World".into()),
        ("Cookie".into(), "Foo".into()),
        ("Cookie".into(), "Bar".into()),
    ];
    let expected_headers = Headers::new(expected_headers);
    assert_eq!(expected_headers, headers);
}
