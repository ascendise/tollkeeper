use std::collections::HashMap;
use std::io;

use crate::http::request::Headers;
use crate::http::request::{parsing::Parse, Request};
use crate::http::Method;

#[test]
pub fn parse_should_read_minimal_http_request() {
    // Arrange
    let raw_request = String::from("GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    let raw_request = raw_request.as_bytes();
    // Act
    let request = Request::parse(&mut io::Cursor::new(raw_request))
        .expect("Failed to parse perfectly valid request");
    // Assert
    assert_eq!(Method::Get, *request.method());
    assert_eq!("/", request.uri());
    assert_eq!("HTTP/1.1", request.http_version());
    let mut headers = HashMap::<String, String>::new();
    headers.insert("Host".into(), "localhost".into());
    let expected_headers = Headers::new(headers);
    assert_eq!(expected_headers, *request.headers());
}
