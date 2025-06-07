use std::{
    io::{Read, Write},
    net::{self},
    thread,
};

use crate::http::{
    response::{ResponseHeaders, StatusCode},
    server::*,
    Headers,
};

fn setup(endpoints: Vec<Endpoint>) -> (Server, net::SocketAddr) {
    let listener = net::TcpListener::bind("127.0.0.1:0").expect("Failed to open test socket");
    let local_addr = listener
        .local_addr()
        .expect("Failed to retrieve address of test socket");
    (Server::new(listener, endpoints), local_addr)
}

fn send_request(addr: net::SocketAddr, request: &[u8]) -> String {
    let mut connection = net::TcpStream::connect(addr).expect("Failed to connect to test socket");
    connection
        .write_all(request)
        .expect("Failed to send test request");
    let mut response = String::new();
    connection
        .read_to_string(&mut response)
        .expect("Failed to read response");
    response
}

#[test]
pub fn server_should_handle_request_through_defined_endpoint() {
    // Arrange
    let handler = Box::new(HelloHandler {
        body: b"Hello!\r\n".into(),
    });
    let endpoints = vec![Endpoint::new(Method::Post, "/hello", handler)];
    let (mut sut, addr) = setup(endpoints);
    let (sender, receiver) = cancellation_token::create_cancellation_token();
    // Act
    let server_thread = thread::spawn(move || sut.start_listening(receiver));
    let request = concat!(
        "POST /hello HTTP/1.1\r\n",
        "Host: localhost\r\n",
        "Content-Length: 13\r\n",
        "\r\n",
        "Hey Server!\r\n"
    )
    .as_bytes();
    let response = send_request(addr, request);
    // Assert
    let expected_response = "HTTP/1.1 200 OK\r\n\r\nHello!\r\n";
    assert_eq!(expected_response, response);
    sender.send_shutdown().unwrap();
    server_thread.join().unwrap().unwrap();
}

#[test]
pub fn server_should_handle_request_on_specific_path() {
    // Arrange
    let hello_handler = Box::new(HelloHandler {
        body: "Hello!\r\n".into(),
    });
    let good_morning_handler = Box::new(HelloHandler {
        body: "Good Morning!\r\n".into(),
    });
    let endpoints = vec![
        Endpoint::new(Method::Post, "/hello", hello_handler),
        Endpoint::new(Method::Post, "/good-morning", good_morning_handler),
    ];
    let (mut sut, addr) = setup(endpoints);
    let (sender, receiver) = cancellation_token::create_cancellation_token();
    // Act
    let server_thread = thread::spawn(move || sut.start_listening(receiver));
    let request = concat!(
        "POST /good-morning HTTP/1.1\r\n",
        "Host: localhost\r\n",
        "Content-Length: 13\r\n",
        "\r\n",
        "Hey Server!\r\n"
    )
    .as_bytes();
    let response = send_request(addr, request);
    // Assert
    let expected_response = "HTTP/1.1 200 OK\r\n\r\nGood Morning!\r\n";
    assert_eq!(expected_response, response);
    sender.send_shutdown().unwrap();
    server_thread.join().unwrap().unwrap();
}

#[test]
pub fn server_should_return_not_found_when_no_matching_endpoint_path_is_found() {
    // Arrange
    let hello_handler = Box::new(HelloHandler {
        body: "Hello!\r\n".into(),
    });
    let endpoints = vec![Endpoint::new(Method::Post, "/hello", hello_handler)];
    let (mut sut, addr) = setup(endpoints);
    let (sender, receiver) = cancellation_token::create_cancellation_token();
    // Act
    let server_thread = thread::spawn(move || sut.start_listening(receiver));
    let request = concat!(
        "POST /this-path-is-unknown HTTP/1.1\r\n",
        "Host: localhost\r\n",
        "Content-Length: 13\r\n",
        "\r\n",
        "Hey Server!\r\n"
    )
    .as_bytes();
    let response = send_request(addr, request);
    // Assert
    let expected_response = "HTTP/1.1 404 Not Found\r\n\r\n";
    assert_eq!(expected_response, response);
    sender.send_shutdown().unwrap();
    server_thread.join().unwrap().unwrap();
}

#[test]
pub fn server_should_return_method_not_allowed_when_matching_endpoint_is_wrong_path() {
    // Arrange
    let hello_handler = Box::new(HelloHandler {
        body: "Hello!\r\n".into(),
    });
    let endpoints = vec![Endpoint::new(Method::Post, "/hello", hello_handler)];
    let (mut sut, addr) = setup(endpoints);
    let (sender, receiver) = cancellation_token::create_cancellation_token();
    // Act
    let server_thread = thread::spawn(move || sut.start_listening(receiver));
    let request = concat!(
        "PUT /hello HTTP/1.1\r\n",
        "Host: localhost\r\n",
        "Content-Length: 13\r\n",
        "\r\n",
        "Hey Server!\r\n"
    )
    .as_bytes();
    let response = send_request(addr, request);
    // Assert
    let expected_response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
    assert_eq!(expected_response, response);
    sender.send_shutdown().unwrap();
    server_thread.join().unwrap().unwrap();
}

#[test]
pub fn server_should_return_an_internal_server_error_and_not_crash() {
    // Arrange
    let panic_handler = Box::new(PanicHandler);
    let endpoints = vec![Endpoint::new(Method::Get, "/panic", panic_handler)];
    let (mut sut, addr) = setup(endpoints);
    let (sender, receiver) = cancellation_token::create_cancellation_token();
    // Act
    let server_thread = thread::spawn(move || sut.start_listening(receiver));
    let request = concat!("GET /panic HTTP/1.1\r\n", "Host: localhost\r\n\r\n");
    let request = request.as_bytes();
    let response = send_request(addr, request);
    // Assert
    let expected_response = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
    sender.send_shutdown().unwrap();
    server_thread.join().unwrap().unwrap();
    assert_eq!(expected_response, response);
}

#[test]
pub fn server_should_return_a_bad_request_on_parsing_error() {
    // Arrange
    let panic_handler = Box::new(HelloHandler {
        body: b"Hello".into(),
    });
    let endpoints = vec![Endpoint::new(Method::Get, "/hello", panic_handler)];
    let (mut sut, addr) = setup(endpoints);
    let (sender, receiver) = cancellation_token::create_cancellation_token();
    // Act
    let server_thread = thread::spawn(move || sut.start_listening(receiver));
    let request =
        String::from("In the grim dark future of the year 40000, there is only war.\r\n\r\n");
    let request = request.as_bytes();
    let response = send_request(addr, request);
    // Assert
    let expected_response = "HTTP/1.1 400 Bad Request\r\n\r\n";
    sender.send_shutdown().unwrap();
    server_thread.join().unwrap().unwrap();
    assert_eq!(expected_response, response);
}

struct HelloHandler {
    body: Vec<u8>,
}
impl HttpServe for HelloHandler {
    fn serve(&self, _: &mut Request) -> Response {
        let mut headers = indexmap::IndexMap::<String, String>::new();
        headers.insert("Content-Length".into(), "8".into());
        let headers = Headers::new(indexmap::IndexMap::new());
        let headers = ResponseHeaders::new(headers);
        Response::with_reason_phrase(StatusCode::OK, "OK", headers, self.body.clone())
    }
}

struct PanicHandler;
impl HttpServe for PanicHandler {
    fn serve(&self, _: &mut Request) -> Response {
        panic!("AAAAAAAAAAAAAA");
    }
}
