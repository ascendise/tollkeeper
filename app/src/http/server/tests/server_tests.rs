use crate::http::server::{
    tests::{ChunkedHandler, HelloHandler, PanicHandler},
    *,
};
use pretty_assertions::assert_eq;
use std::{
    io::{Read, Write},
    net::{self},
    thread,
};

fn setup(handler: Box<dyn TcpServe + Send + Sync + 'static>) -> (Server, net::SocketAddr) {
    let listener = net::TcpListener::bind("127.0.0.1:0").expect("Failed to open test socket");
    listener.set_nonblocking(true).unwrap(); //Allows shutting down server as it changes polling
                                             //from blocking (never getting shutdown signal) to
                                             //busy loop
    let local_addr = listener
        .local_addr()
        .expect("Failed to retrieve address of test socket");
    (Server::new(listener, handler), local_addr)
}

fn send_request(addr: net::SocketAddr, request: &[u8]) -> (String, net::SocketAddr) {
    let mut connection = net::TcpStream::connect(addr).expect("Failed to connect to test socket");
    connection
        .write_all(request)
        .expect("Failed to send test request");
    let mut response = String::new();
    connection
        .read_to_string(&mut response)
        .expect("Failed to read response");
    (response, connection.local_addr().unwrap())
}

#[test]
pub fn server_should_handle_request() {
    // Arrange
    let handler = Box::new(HelloHandler {
        body: b"Hello!\r\n".into(),
    });
    let (mut sut, addr) = setup(handler);
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
    let (response, _) = send_request(addr, request);
    // Assert
    let expected_response = "HTTP/1.1 200 OK\r\nContent-Length: 8\r\n\r\nHello!\r\n";
    assert_eq!(expected_response, response);
    sender.send_shutdown().unwrap();
    server_thread.join().unwrap();
}

#[test]
pub fn server_should_return_414_for_too_long_uri() {
    // Arrange
    let handler = Box::new(HelloHandler {
        body: b"Hello!\r\n".into(),
    });
    let (mut sut, addr) = setup(handler);
    let (sender, receiver) = cancellation_token::create_cancellation_token();
    // Act
    let server_thread = thread::spawn(move || sut.start_listening(receiver));
    let too_long_uri: String = vec!['a'; Request::MAX_REQUEST_LINE_SIZE + 1]
        .iter()
        .collect();
    let request = format!(
        "POST {too_long_uri} HTTP/1.1\r\nHost: localhost\r\nContent-Length: 13\r\n\r\nHey Server!\r\n"
    );
    let (response, _) = send_request(addr, request.as_bytes());
    // Assert
    let expected_response = "HTTP/1.1 414 URI Too Long\r\n\r\n";
    assert_eq!(expected_response, response);
    sender.send_shutdown().unwrap();
    server_thread.join().unwrap();
}

#[test]
pub fn server_should_return_413_for_too_big_of_a_body() {
    // Arrange
    let handler = Box::new(HelloHandler {
        body: b"Hello!\r\n".into(),
    });
    let (mut sut, addr) = setup(handler);
    let (sender, receiver) = cancellation_token::create_cancellation_token();
    // Act
    let server_thread = thread::spawn(move || sut.start_listening(receiver));
    let very_big_body: String = vec!['a'; Request::MAX_BODY_SIZE + 1].iter().collect();
    let request = format!(
        "POST /hello HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
        very_big_body.len(),
        very_big_body
    );
    let (response, _) = send_request(addr, request.as_bytes());
    // Assert
    let expected_response = "HTTP/1.1 413 Content Too Large\r\n\r\n";
    assert_eq!(expected_response, response);
    sender.send_shutdown().unwrap();
    server_thread.join().unwrap();
}

#[test]
pub fn server_should_return_chunked_response() {
    // Arrange
    let chunked_body = "6\r\nHello!\r\n5\r\nChunk\r\n0\r\n\r\n";
    let handler = Box::new(ChunkedHandler {
        chunked_body: chunked_body.into(),
    });
    let (mut sut, addr) = setup(handler);
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
    let (response, _) = send_request(addr, request);
    // Assert
    let expected_response =
        format!("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{chunked_body}");
    assert_eq!(expected_response, response);
    sender.send_shutdown().unwrap();
    server_thread.join().unwrap();
}

#[test]
pub fn server_should_continue_running_when_serve_panics() {
    // Arrange
    let panic_handler = Box::new(PanicHandler);
    let (mut sut, addr) = setup(panic_handler);
    let (sender, receiver) = cancellation_token::create_cancellation_token();
    // Act
    let server_thread = thread::spawn(move || sut.start_listening(receiver));
    let request = String::from("GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
    let request = request.as_bytes();
    let _ = send_request(addr, request);
    // Assert
    sender.send_shutdown().unwrap();
    match server_thread.join() {
        Ok(_) => {}
        Err(_) => panic!("Server died after encountering panic in handler!"),
    }
}
