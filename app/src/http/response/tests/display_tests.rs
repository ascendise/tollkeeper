use crate::http::response::{Response, ResponseHeaders, StatusCode};
use crate::http::{Headers, StreamBody};
use indexmap::IndexMap;
use std::collections::VecDeque;

#[test]
pub fn from_str_should_create_http_format_response() {
    // Arrange
    let mut headers = IndexMap::new();
    headers.insert("Server".into(), "Tollkeeper".into());
    headers.insert("Content-Length".into(), "14".into());
    let headers = Headers::new(headers);
    let headers = ResponseHeaders(headers);
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
