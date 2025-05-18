use std::collections::HashMap;
use std::io;

use crate::http;
use crate::http::request::{parsing::Parse, Request};

#[test]
pub fn parse_should_read_minimal_http_request() {
    // Arrange
    let raw_request = String::from("GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    let raw_request = raw_request.as_bytes();
    // Act
    let request = Request::parse(io::Cursor::new(raw_request))
        .expect("Failed to parse perfectly valid request");
    // Assert
    assert_eq!(http::Method::GET, *request.method());
    assert_eq!("/", request.uri());
    assert_eq!("HTTP/1.1", request.http_version());
    let mut headers = HashMap::<String, String>::new();
    headers.insert("Host".into(), "localhost".into());
    let expected_headers = http::Headers::new(headers);
    assert_eq!(expected_headers, *request.headers());
}
