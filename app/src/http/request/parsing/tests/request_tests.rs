use indexmap::IndexMap;
use std::io::{BufReader, Read, Write};
use std::net;
use test_case::test_case;

use crate::http::request::{parsing::*, Request};
use crate::http::request::{Headers, RequestHeaders};

fn setup_listener() -> net::TcpListener {
    net::TcpListener::bind("127.0.0.1:0").expect("Failed to setup test socket")
}

fn write_bytes_to_target(listener: &net::TcpListener, bytes: &[u8]) -> BufReader<net::TcpStream> {
    let address = listener.local_addr().unwrap();
    let mut stream = net::TcpStream::connect(address).expect("Failed to write bytes for test");
    let mut incoming = listener.incoming();
    stream.write(bytes).expect("Failed to write to stream");
    let stream = incoming
        .next()
        .unwrap()
        .expect("Could not open server stream");
    BufReader::new(stream)
}

#[test]
pub fn parse_should_read_minimal_http_request() {
    // Arrange
    let raw_request = concat!("GET / HTTP/1.1\r\n", "Host:localhost\r\n\r\n");
    let raw_request = raw_request.as_bytes();
    let listener = setup_listener();
    // Act
    let stream = write_bytes_to_target(&listener, raw_request);
    let request = Request::parse(stream).expect("Failed to parse perfectly valid request");
    // Assert
    assert_eq!(Method::Get, *request.method());
    assert_eq!("/", request.request_target());
    assert_eq!("HTTP/1.1", request.http_version());
    let mut headers = IndexMap::<String, String>::new();
    headers.insert("Host".into(), "localhost".into());
    let expected_headers = Headers::new(headers);
    let expected_headers = RequestHeaders::new(expected_headers).unwrap();
    assert_eq!(expected_headers, *request.headers());
}

#[test]
pub fn parse_should_read_http_request_with_body() {
    // Arrange
    let raw_request = concat!(
        "POST / HTTP/1.1\r\n",
        "Host: localhost\r\n",
        "Content-Type: text/raw; charset=utf8\r\n",
        "Content-Length: 15\r\n",
        "\r\n",
        "Hello, World!\r\n"
    );
    let raw_request = raw_request.as_bytes();
    let listener = setup_listener();
    // Act
    let incoming_stream = write_bytes_to_target(&listener, raw_request);
    let mut request =
        Request::parse(incoming_stream).expect("Failed to parse perfectly valid request");
    // Assert
    assert_eq!(&Method::Post, request.method());
    assert_eq!("/", request.request_target());
    assert_eq!("HTTP/1.1", request.http_version());
    let mut expected_headers = IndexMap::<String, String>::new();
    expected_headers.insert("Host".into(), "localhost".into());
    expected_headers.insert("Content-Type".into(), "text/raw; charset=utf8".into());
    expected_headers.insert("Content-Length".into(), "15".into());
    let expected_headers = Headers::new(expected_headers);
    let expected_headers = RequestHeaders::new(expected_headers).unwrap();
    assert_eq!(&expected_headers, request.headers());
    let mut content = String::new();
    match request.body() {
        Some(b) => b
            .read_to_string(&mut content)
            .expect("Something bad happened while trying to read body"),
        None => panic!("No body found"),
    };
    let expected_content = "Hello, World!\r\n";
    assert_eq!(expected_content, content);
}

#[test_case(String::from("Hello") ; "Hello")]
#[test_case(String::from("GET/HTTP/1.1\r\n") ; "no whitespace")]
#[test_case(String::from("GET/HTTP /1.1\r\n") ; "only some whitespace")]
#[test_case(String::from("GET\t/\tHTTP/1.1\r\n") ; "tab instead of whitespace")]
#[test_case(String::from("GET   /   HTTP/1.1\r\n") ; "too much whitespace")]
#[test_case(String::from("GET   /   HTTP/1.1\r") ; "no line feed")]
#[test_case(String::from("GET   /   HTTP/1.1\n") ; "no carriage return")]
#[test_case(String::from("GET   /   HTTP/1.1") ; "no new line")]
#[test_case(String::from("    GET / HTTP/1.1\r\n") ; "leading whitespace")]
#[test_case(String::from("GET / HTTP/1.1     \r\n") ; "trailing whitespace")]
#[test_case(String::from(" / HTTP/1.1\r\n") ; "Missing method")]
#[test_case(String::from("GET HTTP/1.1\r\n") ; "Missing request target")]
#[test_case(String::from("GET /\r\n") ; "Missing HTTP version")]
pub fn parse_should_reject_status_line_with_invalid_format(request_line: String) {
    // Arrange
    let raw_request = request_line + "Host:localhost\r\n\r\n";
    let raw_request = raw_request.as_bytes();
    let listener = setup_listener();
    // Act
    let stream = write_bytes_to_target(&listener, raw_request);
    let result = Request::parse(stream);
    // Assert
    let error = match result {
        Ok(r) => panic!(
            "Invalid request line got accepted!: '{} {} {}'",
            r.method(),
            r.request_target(),
            r.http_version()
        ),
        Err(e) => e,
    };
    let expected = ParseError::RequestLine;
    assert_eq!(expected, error);
}

#[test_case(String::from("X-Hello:Do you know where my mommy is?\r\n") ; "no Host header")]
#[test_case(String::from("Host:localhost\r\nX-Whitespace :text\r\n") ; "forbidden whitespace (SPACE) between field name and colon")]
#[test_case(String::from("Host:localhost\r\nX-Whitespace\t:text\r\n") ; "forbidden whitespace (TAB) between field name and colon")]
#[test_case(String::from("Host:localhost\r") ; "no line feed")]
#[test_case(String::from("Host:localhost\n") ; "no carriage return")]
pub fn parse_should_reject_headers_with_invalid_format(headers: String) {
    // Arrange
    let raw_request = format!("GET / HTTP/1.1\r\n{headers}\r\n");
    let raw_request = raw_request.as_bytes();
    let listener = setup_listener();
    // Act
    let stream = write_bytes_to_target(&listener, raw_request);
    let result = Request::parse(stream);
    // Assert
    let error = match result {
        Ok(r) => panic!("Invalid headers got accepted!: '{}'", r.headers()),
        Err(e) => e,
    };
    let expected = ParseError::Header;
    assert_eq!(expected, error);
}

#[test]
pub fn parse_should_treat_headers_case_insensitive() {
    // Arrange
    let raw_request = concat!(
        "GET / HTTP/1.1\r\n",
        "Host:localhost\r\n",
        "USER-AGENT:value\r\n",
        "\r\n",
    );
    let raw_request = raw_request.as_bytes();
    let listener = setup_listener();
    // Act
    let stream = write_bytes_to_target(&listener, raw_request);
    let request = Request::parse(stream).expect("Could not parse valid request");
    // Assert
    let ua_header = request
        .headers()
        .user_agent()
        .expect("User-Agent was not found (because of case-sensitivity?)");
    assert_eq!("value", ua_header);
}
