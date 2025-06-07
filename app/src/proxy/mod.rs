use std::io;
use std::io::BufReader;
use std::net;

use crate::http::request::Parse;
use crate::http::request::Request;

use super::http::server::*;

#[cfg(test)]
mod tests;

pub struct ProxyServe {
    client: Box<dyn ProxyClient>,
}

impl ProxyServe {
    pub fn new(client: Box<dyn ProxyClient>) -> Self {
        Self { client }
    }
}
impl TcpServe for ProxyServe {
    fn serve(&self, stream: std::net::TcpStream) {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let mut request = Request::parse(reader).unwrap(); //TODO: Return 400 Bad Request
        self.client.send(&mut request);
    }
}

pub struct ProxyClientImpl;
impl ProxyClient for ProxyClientImpl {
    fn send(&self, request: &mut Request) -> io::BufReader<net::TcpStream> {
        todo!("Implement ProxyClient!");
    }
}

pub trait ProxyClient {
    fn send(&self, request: &mut Request) -> io::BufReader<net::TcpStream>;
}
