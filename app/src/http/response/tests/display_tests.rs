use crate::http::response::{self, Response, StatusCode};
use crate::http::{self, StreamBody};
use std::collections::VecDeque;

#[test]
pub fn from_str_should_create_http_format_response() {
    // Arrange
    let mut headers = http::Headers::empty();
    headers.insert("Server", "Tollkeeper");
    headers.insert("Content-Length", "14");
    let headers = response::Headers::new(headers);
    let body = b"Hello, World\r\n";
    let body = StreamBody::<VecDeque<u8>>::new(body.to_vec().into());
    let sut = Response::new(
        StatusCode::OK,
        Some("No-Error".into()),
        headers,
        Some(Box::new(body)),
    );
    // Act
    let raw_data: Vec<u8> = sut.into_bytes();
    let response_str = String::from_utf8(raw_data).expect("Failed to parse raw data");
    // Assert
    let expected =
        "HTTP/1.1 200 No-Error\r\nServer: Tollkeeper\r\nContent-Length: 14\r\n\r\nHello, World\r\n";
    assert_eq!(expected, response_str);
}
