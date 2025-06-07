use std::io;
use std::{
    io::{Read, Write},
    net,
};

use crate::{
    http::server::TcpServe,
    proxy::{ProxyClient, ProxyServe},
};

fn setup(proxy_response: Vec<u8>) -> (ProxyServe, net::TcpListener) {
    let listener = net::TcpListener::bind("127.0.0.1:0").expect("Failed to open test socket");
    let fake_proxy = StubProxyClient::new(proxy_response);
    let sut = ProxyServe::new(Box::new(fake_proxy));
    (sut, listener)
}

#[test]
pub fn serve_should_return_response_of_target() {
    // Arrange
    let (sut, listener) = setup("Hello, my friend!".into());
    let addr = listener
        .local_addr()
        .expect("Failed to retrieve address of test socket");
    let mut client_conn = net::TcpStream::connect(addr).expect("Failed to connect to test socket");
    // Act
    let request = b"GET / HTTP/1.1\r\nHost:friendly-server.com\r\n\r\nHello, Server!\r\n";
    client_conn
        .write_all(request)
        .expect("Failed to send test request");
    let (server_conn, _) = listener.accept().expect("Failed to retrieve connection");
    sut.serve(server_conn);
    // Assert
    let mut response = String::new();
    client_conn
        .read_to_string(&mut response)
        .expect("Failed to get server response");
    let expected_response = "Hello, my friend!";
    assert_eq!(expected_response, response);
}

pub struct StubProxyClient {
    response: Vec<u8>,
}

impl StubProxyClient {
    pub fn new(response: Vec<u8>) -> Self {
        Self { response }
    }
}
impl ProxyClient for StubProxyClient {
    fn send(&self, _: &mut crate::http::request::Request) -> io::BufReader<net::TcpStream> {
        let listener = net::TcpListener::bind("127.0.0.1:0").expect("Failed to open proxy socket");
        let proxy_addr = listener.local_addr().unwrap();
        let mut client_conn = net::TcpStream::connect(proxy_addr).unwrap();
        client_conn
            .write_all(&self.response)
            .expect("Failed to write proxy response");
        let (server_conn, _) = listener
            .accept()
            .expect("Failed to open proxy connection for response");
        io::BufReader::new(server_conn)
    }
}
