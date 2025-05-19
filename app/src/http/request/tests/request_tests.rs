use std::collections::HashMap;
use std::io::{self, Read};

use crate::http::request::Headers;
use crate::http::request::{parsing::Parse, Request};
use crate::http::Method;

#[test]
pub fn parse_should_read_minimal_http_request() {
    // Arrange
    let raw_request = concat!("GET / HTTP/1.1\r\n", "Host:localhost\r\n\r\n");
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

#[test]
pub fn parse_should_read_http_request_with_body() {
    // Arrange
    let raw_request = concat!(
        "POST / HTTP/1.1\r\n",
        "Host:localhost\r\n",
        "Content-Type:text/raw; charset=utf8\r\n",
        "\r\n",
        "Hello, World!\r\n"
    );
    let raw_request = raw_request.as_bytes();
    // Act
    let mut request = Request::parse(&mut io::Cursor::new(raw_request))
        .expect("Failed to parse perfectly valid request");
    // Assert
    assert_eq!(&Method::Post, request.method());
    assert_eq!("/", request.uri());
    assert_eq!("HTTP/1.1", request.http_version());
    let mut expected_headers = HashMap::<String, String>::new();
    expected_headers.insert("Host".into(), "localhost".into());
    expected_headers.insert("Content-Type".into(), "text/raw; charset=utf8".into());
    let expected_headers = Headers::new(expected_headers);
    assert_eq!(&expected_headers, request.headers());
    let mut content = String::new();
    match request.body() {
        Some(b) => b
            .read_to_string(&mut content)
            .expect("Something bad happened while trying to read body"),
        None => panic!("No body found"),
    };
    let expected_content = "Hello, World!\r\n";
    assert_eq!(expected_content, content);
}
