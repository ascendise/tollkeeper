use std::{io::Read, net, str::FromStr};

mod request;
pub use request::*;

#[derive(Debug, PartialEq, Eq)]
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
impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let method = match s {
            "OPTIONS" => Method::OPTIONS,
            "GET" => Method::GET,
            "HEAD" => Method::HEAD,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "TRACE" => Method::TRACE,
            "CONNECT" => Method::CONNECT,
            _ => Method::EXTENSION(s.into()),
        };
        Ok(method)
    }
}

pub struct BodyStream {
    tcp_stream: net::TcpStream,
}
impl Read for BodyStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.tcp_stream.read(buf)
    }
}
