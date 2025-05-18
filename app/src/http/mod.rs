use std::{io::Read, net};

pub mod request;

pub enum Method {
    OPTIONS,
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    TRACE,
    CONNECT,
    EXTENSION(String),
}

pub struct BodyStream {
    tcp_stream: net::TcpStream,
}
impl Read for BodyStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.tcp_stream.read(buf)
    }
}
