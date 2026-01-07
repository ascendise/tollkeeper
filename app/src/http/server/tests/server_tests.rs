use crate::http::server::{
    tests::{HelloHandler, PanicHandler},
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
    let local_addr = listener
        .local_addr()
        .expect("Failed to retrieve address of test socket");
    (Server::new(listener, handler), local_addr)
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
    let response = send_request(addr, request);
    // Assert
    let expected_response = "HTTP/1.1 200 OK\r\nContent-Length: 8\r\n\r\nHello!\r\n";
    assert_eq!(expected_response, response);
    sender.send_shutdown().unwrap();
    server_thread.join().unwrap().unwrap();
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
