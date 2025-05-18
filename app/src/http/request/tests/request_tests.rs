use std::collections::HashMap;
use std::io::BufReader;

use crate::http;
use crate::http::request::{parsing::Parse, Request};

pub fn parse_should_read_minimal_http_request() {
    // Arrange
    let raw_request = String::from(
        r"GET / HTTP/1.1
Host: localhost

",
    );
    let raw_request = BufReader::new(raw_request.as_bytes());
    // Act
    let request = Request::parse(raw_request);
    // Assert
    let expected_request = Request::new(
        http::Method::GET,
        "/",
        "1.1",
        "localhost",
        http::Headers::new(HashMap::new()),
    );
}
