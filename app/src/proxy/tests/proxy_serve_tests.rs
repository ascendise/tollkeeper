use std::thread;
use std::{
    io::{Read, Write},
    net,
};

use crate::{http::server::TcpServe, proxy::ProxyServe};

fn setup() -> (ProxyServe, net::TcpListener) {
    let listener = net::TcpListener::bind("127.0.0.1:0").expect("Failed to open test socket");
    let sut = ProxyServe {};
    (sut, listener)
}

fn setup_proxy(response: Vec<u8>) -> (thread::JoinHandle<()>, net::SocketAddr) {
    let listener = net::TcpListener::bind("127.0.0.1:0").unwrap();
    let local_addr = listener.local_addr().unwrap();
    let thread = thread::spawn(move || {
        for conn in listener.incoming() {
            let mut conn = conn.unwrap();
            conn.write_all(&response).unwrap();
            return;
        }
    });
    (thread, local_addr)
}

fn send_request(server_addr: net::SocketAddr, request: Vec<u8>) -> net::TcpStream {
    let mut client_conn =
        net::TcpStream::connect(server_addr).expect("Failed to connect to test socket");
    client_conn
        .write_all(&request)
        .expect("Failed to send test request");
    client_conn
}

#[test]
pub fn serve_should_return_response_of_target() {
    // Arrange
    let (sut, server_listener) = setup();
    let server_addr = server_listener
        .local_addr()
        .expect("Failed to retrieve address of test socket");
    let (proxy, proxy_addr) = setup_proxy("HTTP/1.1 200 OK\r\n\r\n".into());
    // Act
    let request = format!(
        "GET / HTTP/1.1\r\nHost:127.0.0.1:{}\r\n\r\nHello, Server!\r\n",
        proxy_addr.port()
    );
    let mut client_conn = send_request(server_addr, request.into());
    let (server_conn, _) = server_listener
        .accept()
        .expect("Failed to retrieve connection");
    sut.serve(server_conn);
    proxy.join().unwrap();
    // Assert
    let mut response = String::new();
    client_conn
        .read_to_string(&mut response)
        .expect("Failed to get server response");
    let expected_response = "HTTP/1.1 200 OK\r\n\r\n";
    assert_eq!(expected_response, response);
}

#[test]
pub fn serve_should_return_bad_request_if_not_parseable() {
    // Arrange
    let (sut, server_listener) = setup();
    let server_addr = server_listener
        .local_addr()
        .expect("Failed to retrieve address of test socket");
    // Act
    let request =
        String::from("In the grim dark future of the year 40000, there is only war.\r\n\r\n");
    let mut client_conn = send_request(server_addr, request.into());
    let (server_conn, _) = server_listener
        .accept()
        .expect("Failed to retrieve connection");
    sut.serve(server_conn);
    // Assert
    let mut response = String::new();
    client_conn
        .read_to_string(&mut response)
        .expect("Failed to get server response");
    let expected_response = "HTTP/1.1 400 Bad Request\r\n\r\n";
    assert_eq!(expected_response, response);
}
