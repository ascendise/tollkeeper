use std::collections::VecDeque;

use crate::http::request::body_utils::ReadJson;
use crate::http::request::{Headers, Method};
use crate::http::{self, StreamBody};

fn setup(json: serde_json::Value) -> http::Request {
    let json = json.to_string().into_bytes();
    let json: VecDeque<u8> = json.into();
    let body = StreamBody::new(json);
    let body = Box::new(body);
    let mut headers = http::Headers::empty();
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
    let mut request = setup(json.clone());
    // Act
    let result = request.read_json();
    // Assert
    assert_eq!(result, Ok(json));
}
