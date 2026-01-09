use pretty_assertions::assert_eq;
use std::{collections::VecDeque, io::Read};

use crate::http::{
    response::{Response, StatusCode},
    Body, Parse,
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
            assert_eq!(StatusCode::OK, r.status_code());
            assert_eq!(Some("OK"), r.reason_phrase());
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
        Ok(r) => assert_eq!("Hello", r.headers().extension("Server").unwrap()),
    }
}

#[test]
pub fn parse_should_include_body() {
    // Arrange
    let raw_response = String::from("HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nHello\r\n");
    let raw_response: VecDeque<u8> = raw_response.into_bytes().into();
    // Act
    let mut response = Response::parse(raw_response).expect("expected response, got error");
    // Assert
    match response.body() {
        Body::Buffer(buffer_body) => {
            let mut body = String::new();
            buffer_body.read_to_string(&mut body).unwrap();
            assert_eq!("Hello\r\n", body);
        }
        Body::Stream(_) => panic!("Got fixed content as stream"),
        Body::None => panic!("Expected body but none was sent"),
    }
}

#[test]
pub fn parse_should_include_chunked_body() {
    // Arrange
    let expected_body = vec![
        "3\r\nHel\r\n",
        "6\r\nlo, Wo\r\n",
        "4\r\nrld!\r\n",
        "0\r\n\r\n",
    ];
    let body = expected_body.concat().to_string();
    let raw_response = format!("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{body}");
    let raw_response: VecDeque<u8> = raw_response.into_bytes().into();
    // Act
    let mut response = Response::parse(raw_response).expect("Expected Response, got error");
    // Assert
    match response.body() {
        Body::Buffer(_) => {
            panic!("Got streamed content as fixed sized content");
        }
        Body::Stream(body) => {
            let mut actual_body: Vec<String> = Vec::new();
            while let Some(chunk) = body.read_chunk() {
                let chunk = String::from_utf8(chunk.content().into()).unwrap();
                actual_body.push(chunk);
            }
            assert_eq!(expected_body, actual_body);
        }
        Body::None => panic!("Expected body but none was sent"),
    }
}
