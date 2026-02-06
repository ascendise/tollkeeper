use pretty_assertions::assert_eq;
use std::io::BufReader;

use crate::http::{parsing::ParseError, Headers, Parse};

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

#[test]
fn parse_should_return_error_if_header_value_exceed_limit() {
    // Arrange
    let expected_max_size = 8192;
    let raw_header = format!(
        "Hello: {}\r\n\r\n",
        String::from_utf8_lossy(&vec![b'a'; expected_max_size + 1])
    );
    let mut raw_header = BufReader::new(raw_header.as_bytes());
    // Act
    let res = Headers::parse(&mut raw_header);
    // Assert
    assert_eq!(Err(ParseError::Header), res);
}

#[test]
fn parse_should_return_error_if_header_key_exceed_limit() {
    // Arrange
    let expected_max_size = 8192;
    let raw_header = format!(
        "{}: hello\r\n\r\n",
        String::from_utf8_lossy(&vec![b'a'; expected_max_size + 1])
    );
    let mut raw_header = BufReader::new(raw_header.as_bytes());
    // Act
    let res = Headers::parse(&mut raw_header);
    // Assert
    assert_eq!(Err(ParseError::Header), res);
}

#[test]
fn parse_should_return_error_if_too_many_headers_are_sent() {
    // Arrange
    let expected_max_number = 128;
    let mut raw_headers = String::new();
    for i in 0..(expected_max_number + 1) {
        let header = format!("Hello{i}:World{i}\r\n");
        raw_headers.push_str(&header);
    }
    raw_headers.push_str("\r\n");
    let mut raw_headers = BufReader::new(raw_headers.as_bytes());
    // Act
    let res = Headers::parse(&mut raw_headers);
    // Assert
    assert_eq!(Err(ParseError::Header), res);
}
