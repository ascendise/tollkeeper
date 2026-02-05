use pretty_assertions::assert_eq;

use crate::http::request::body_reader::{ReadJson, ReadJsonError};
use crate::http::request::{Headers, Method};
use crate::http::{self};

fn setup(json: String) -> http::Request {
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/json");
    headers.insert("Content-Length", json.len().to_string());
    setup_with_headers(json, headers)
}

fn setup_no_body() -> http::Request {
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/json");
    headers.insert("Content-Length", "0");
    headers.insert("Host", "localhost:80");
    let request = http::Request::new(
        Method::Post,
        "/",
        Headers::new(headers).unwrap(),
        http::Body::None,
    );
    request.unwrap()
}

fn setup_with_headers(json: String, mut headers: http::Headers) -> http::Request {
    let body = http::Body::from_string(json);
    headers.insert("Host", "localhost:80");
    let request = http::Request::new(Method::Post, "/", Headers::new(headers).unwrap(), body);
    request.unwrap()
}

#[test]
pub fn read_json_from_body_should_return_json_for_valid_body() {
    // Arrange
    let json = serde_json::json!({
        "key": "value"
    });
    let raw_json = json.to_string();
    let mut request = setup(raw_json);
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
    let raw_json = json.to_string();
    let mut headers = http::Headers::empty();
    headers.insert("Content-Length", raw_json.len().to_string());
    let mut request = setup_with_headers(raw_json, http::Headers::empty());
    // Act
    let result: Result<serde_json::Value, ReadJsonError> = request.read_json();
    // Assert
    assert_eq!(result, Err(ReadJsonError::MismatchedContentType("".into())));
}

#[test]
pub fn read_json_from_body_should_return_error_if_mismatched_content_type_header() {
    // Arrange
    let json = serde_json::json!({
        "key": "value"
    });
    let raw_json = json.to_string();
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/xml");
    headers.insert("Content-Length", raw_json.len().to_string());
    let mut request = setup_with_headers(raw_json, headers);
    // Act
    let result: Result<serde_json::Value, ReadJsonError> = request.read_json();
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
    let raw_json = no_json.to_string();
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/json");
    headers.insert("Content-Length", raw_json.len().to_string());
    let mut request = setup_with_headers(raw_json, headers);
    // Act
    let result: Result<serde_json::Value, ReadJsonError> = request.read_json();
    // Assert
    assert_eq!(result, Err(ReadJsonError::NonJsonData));
}

#[test]
pub fn read_json_from_body_should_read_only_defined_content_length() {
    // Arrange
    let json = serde_json::json!({
        "key": "value"
    });
    let raw_json = json.to_string();
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/json");
    let content_length = raw_json.len() / 2; // We half the content length to test if this causes
                                             // a ReadJsonError::NonJsonData error
    headers.insert("Content-Length", content_length.to_string());
    let mut request = setup_with_headers(raw_json, headers);
    // Act
    let result: Result<serde_json::Value, ReadJsonError> = request.read_json();
    // Assert
    assert_eq!(result, Err(ReadJsonError::NonJsonData));
}

#[test]
pub fn read_json_from_body_should_treat_no_content_length_as_no_body() {
    // Arrange
    let json = serde_json::json!({
        "key": "value"
    });
    let raw_json = json.to_string();
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/json");
    //// No Content Length
    let mut request = setup_with_headers(raw_json, headers);
    // Act
    let result: Result<serde_json::Value, ReadJsonError> = request.read_json();
    // Assert
    assert_eq!(result, Err(ReadJsonError::NonJsonData));
}

#[test]
pub fn read_json_from_body_should_treat_empty_body_as_non_json_data() {
    // Arrange
    let mut request = setup_no_body();
    // Act
    let result: Result<serde_json::Value, ReadJsonError> = request.read_json();
    // Assert
    assert_eq!(result, Err(ReadJsonError::NonJsonData));
}

#[test]
pub fn read_json_from_body_should_return_io_error_when_missing_data() {
    // Arrange
    let json = serde_json::json!({
        "key": "value"
    });
    let raw_json = json.to_string();
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/json");
    let content_length = raw_json.len() * 2; // We double the content length, which
                                             // means there is more data coming
    headers.insert("Content-Length", content_length.to_string());
    let mut request = setup_with_headers(raw_json, headers);
    // Act
    let result: Result<serde_json::Value, ReadJsonError> = request.read_json();
    // Assert
    assert_eq!(result, Err(ReadJsonError::IoError));
}
