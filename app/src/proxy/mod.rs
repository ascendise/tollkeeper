use std::io;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net;

use crate::http::request::Parse;
use crate::http::request::Request;

use super::http::server::*;

#[cfg(test)]
mod tests;

pub struct ProxyServe {}

impl ProxyServe {}
impl TcpServe for ProxyServe {
    fn serve(&self, mut stream: std::net::TcpStream) {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let request = Request::parse(reader).unwrap(); //TODO: Return 400 Bad Request
                                                       //TODO: Do stuff
        let target = request.absolute_target();
        let host = target.host_str().unwrap();
        let port = target.port().unwrap();
        let addr = format!("{host}:{port}");
        let mut conn = net::TcpStream::connect(addr).unwrap();
        //TODO:
        // Return error indicating error from other server
        let mut response = String::new();
        conn.read_to_string(&mut response).unwrap();
        stream.write_all(response.as_bytes()).unwrap();
    }
}
