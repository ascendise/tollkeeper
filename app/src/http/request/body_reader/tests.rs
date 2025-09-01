use std::collections::VecDeque;

use crate::http::request::body_reader::{ReadJson, ReadJsonError};
use crate::http::request::{Headers, Method};
use crate::http::{self, StreamBody};

fn setup(json: Vec<u8>) -> http::Request {
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/json");
    headers.insert("Content-Length", json.len().to_string());
    setup_with_headers(json, headers)
}

fn setup_with_headers(json: Vec<u8>, mut headers: http::Headers) -> http::Request {
    let json: VecDeque<u8> = json.into();
    let body = StreamBody::new(json);
    let body = Box::new(body);
    headers.insert("Host", "localhost:80");
    let request = http::Request::with_body(Method::Post, "/", Headers::new(headers).unwrap(), body);
    request.unwrap()
}

#[test]
pub fn read_json_from_body_should_return_json_for_valid_body() {
    // Arrange
    let json = serde_json::json!({
        "key": "value"
    });
    let raw_json = json.to_string().into_bytes();
    let mut request = setup(raw_json.clone());
    // Act
    let result = request.read_json();
    // Assert
    assert_eq!(result, Ok(json));
}

#[test]
pub fn read_json_from_body_should_return_error_if_missing_content_type_header() {
    // Arrange
    let json = serde_json::json!({
        "key": "value"
    });
    let raw_json = json.to_string().into_bytes();
    let mut headers = http::Headers::empty();
    headers.insert("Content-Length", raw_json.len().to_string());
    let mut request = setup_with_headers(raw_json.clone(), http::Headers::empty());
    // Act
    let result = request.read_json();
    // Assert
    assert_eq!(result, Err(ReadJsonError::MismatchedContentType("".into())));
}

#[test]
pub fn read_json_from_body_should_return_error_if_mismatched_content_type_header() {
    // Arrange
    let json = serde_json::json!({
        "key": "value"
    });
    let raw_json = json.to_string().into_bytes();
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/xml");
    headers.insert("Content-Length", raw_json.len().to_string());
    let mut request = setup_with_headers(raw_json.clone(), headers);
    // Act
    let result = request.read_json();
    // Assert
    assert_eq!(
        result,
        Err(ReadJsonError::MismatchedContentType(
            "application/xml".into()
        ))
    );
}

#[test]
pub fn read_json_from_body_should_return_error_if_not_able_to_parse_body() {
    // Arrange
    let no_json = r"
    Name,FirstName,Street
    Muster,Max,Streetsway
    ";
    let raw_json = no_json.to_string().into_bytes();
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/json");
    headers.insert("Content-Length", raw_json.len().to_string());
    let mut request = setup_with_headers(raw_json.clone(), headers);
    // Act
    let result = request.read_json();
    // Assert
    assert_eq!(result, Err(ReadJsonError::FailedParsing));
}
