use std::io::BufReader;
use std::{io, thread};
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

fn setup_proxy(listener: net::TcpListener, response: Vec<u8>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for conn in listener.incoming() {
            let mut conn = conn.unwrap();
            conn.write_all(&response).unwrap();
            return;
        }
    })
}

#[test]
pub fn serve_should_return_response_of_target() {
    // Arrange
    let (sut, server_listener) = setup();
    let server_addr = server_listener
        .local_addr()
        .expect("Failed to retrieve address of test socket");
    let proxy_listener = net::TcpListener::bind("127.0.0.1:0").unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    let proxy = setup_proxy(proxy_listener, b"Hello, my friend!".into());
    let mut client_conn =
        net::TcpStream::connect(server_addr).expect("Failed to connect to test socket");
    // Act
    let request = format!(
        "GET / HTTP/1.1\r\nHost:127.0.0.1:{}\r\n\r\nHello, Server!\r\n",
        proxy_addr.port()
    );
    client_conn
        .write_all(request.as_bytes())
        .expect("Failed to send test request");
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
    let expected_response = "Hello, my friend!";
    assert_eq!(expected_response, response);
}
