use std::collections::VecDeque;

use crate::http::{
    response::{Response, StatusCode},
    Parse,
};

#[test]
pub fn parse_should_return_response_for_valid_message() {
    // Arrange
    let raw_response = String::from("HTTP/1.1 200 OK\r\n\r\n");
    let raw_response: VecDeque<u8> = raw_response.into_bytes().into();
    // Act
    let response = Response::parse(raw_response);
    // Assert
    match response {
        Err(e) => panic!("Expected Response, got error: {e}"),
        Ok(r) => {
            assert_eq!(r.status_code(), StatusCode::OK);
            assert_eq!(r.reason_phrase(), Some("OK"));
        }
    };
}

#[test]
pub fn parse_should_include_headers() {
    // Arrange
    let raw_response = String::from("HTTP/1.1 200 OK\r\nServer: Hello\r\n\r\n");
    let raw_response: VecDeque<u8> = raw_response.into_bytes().into();
    // Act
    let response = Response::parse(raw_response);
    // Assert
    match response {
        Err(e) => panic!("Expected Response, got error: {e}"),
        Ok(r) => assert_eq!(r.headers().extension("Server").unwrap(), "Hello"),
    }
}

#[test]
pub fn parse_should_include_body() {
    // Arrange
    let raw_response = String::from("HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nHello\r\n");
    let raw_response: VecDeque<u8> = raw_response.into_bytes().into();
    // Act
    let response = Response::parse(raw_response);
    // Assert
    match response {
        Err(e) => panic!("Expected Response, got error: {e}"),
        Ok(mut r) => {
            let mut body = String::new();
            r.body().unwrap().read_to_string(&mut body).unwrap();
            assert_eq!(body, "Hello\r\n");
        }
    }
}
