use indexmap::IndexMap;

use super::*;

#[test]
pub fn from_str_should_create_http_format_response() {
    // Arrange
    let mut headers = IndexMap::new();
    headers.insert("Server".into(), "Tollkeeper".into());
    headers.insert("Header".into(), "Value".into());
    let headers = Headers::new(headers);
    let headers = ResponseHeaders(headers);
    let body = b"Hello, World\r\n";
    let sut = Response::with_reason_phrase(StatusCode::OK, "No-Error", headers, body.to_vec());
    // Act
    let raw_data: Vec<u8> = sut.into_bytes();
    let response_str = String::from_utf8(raw_data).expect("Failed to parse raw data");
    // Assert
    let expected =
        "HTTP/1.1 200 No-Error\r\nServer: Tollkeeper\r\nHeader: Value\r\n\r\nHello, World\r\n";
    assert_eq!(expected, response_str);
}
