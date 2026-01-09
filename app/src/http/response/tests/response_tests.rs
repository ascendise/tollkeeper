use std::collections::VecDeque;

use crate::http::response::{self, Response, StatusCode};
use crate::http::{self};
use pretty_assertions::assert_eq;

#[test]
pub fn into_bytes_should_create_http_format_response_with_fixed_size_body() {
    // Arrange
    let mut headers = http::Headers::empty();
    headers.insert("Server", "Tollkeeper");
    headers.insert("Content-Length", "14");
    let headers = response::Headers::new(headers);
    let body = http::Body::from_string("Hello, World\r\n".into());
    let mut sut = Response::new(StatusCode::OK, Some("No-Error".into()), headers, body);
    // Act
    let raw_data: Vec<u8> = sut.as_bytes();
    let response_str = String::from_utf8(raw_data).expect("Failed to parse raw data");
    // Assert
    let expected =
        "HTTP/1.1 200 No-Error\r\nServer: Tollkeeper\r\nContent-Length: 14\r\n\r\nHello, World\r\n";
    assert_eq!(expected, response_str);
}

#[test]
pub fn into_bytes_should_skip_parsing_body_when_is_chunked() {
    // Arrange
    let mut headers = http::Headers::empty();
    headers.insert("Server", "Tollkeeper");
    headers.insert("Transfer-Encoding", "chunked");
    let headers = response::Headers::new(headers);
    let chunked_body = String::from("5\r\nHello\r\n4\r\n, Wo\r\n3\r\nrld\r\n0\r\n\r\n");
    let chunked_body: VecDeque<u8> = chunked_body.into_bytes().into();
    let body = http::Body::from_stream(Box::new(chunked_body), None);
    let mut sut = Response::new(StatusCode::OK, Some("No-Error".into()), headers, body);
    // Act
    let raw_data: Vec<u8> = sut.as_bytes();
    let response_str = String::from_utf8(raw_data).expect("Failed to parse raw data");
    // Assert
    let expected =
        "HTTP/1.1 200 No-Error\r\nServer: Tollkeeper\r\nTransfer-Encoding: chunked\r\n\r\n";
    assert_eq!(expected, response_str);
}
