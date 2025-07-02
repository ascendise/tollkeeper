use std::io::Write;
use std::net;

use crate::http::request::Request;
use crate::http::response::Response;
use crate::http::Parse;

use super::http::server::*;

#[cfg(test)]
mod tests;

pub struct ProxyServe {}

impl ProxyServe {
    pub fn new() -> Self {
        Self {}
    }
}
impl TcpServe for ProxyServe {
    fn serve(&self, mut stream: std::net::TcpStream) {
        println!("Incoming request");
        let request = match Request::parse(stream.try_clone().unwrap()) {
            Ok(v) => v,
            Err(_) => {
                send_response(stream, Response::bad_request());
                return;
            }
        };
        println!("Request successfully parsed!");
        let addr = get_host(&request);
        let mut conn = net::TcpStream::connect(&addr).unwrap();
        println!("Opened connection to {addr}");
        conn.write_all(&request.into_bytes()).unwrap();
        println!("Sent request to target");
        let response = Response::parse(conn.try_clone().unwrap()).unwrap();
        println!("Response successfully parsed!");
        let data = response.into_bytes();
        println!("Response turned into bytes!");
        stream.write_all(&data).unwrap();
        println!("Response successfully written to you!");
    }
}

fn send_response(mut stream: net::TcpStream, response: Response) {
    let response = response.into_bytes();
    stream.write_all(&response).unwrap()
}

fn get_host(request: &Request) -> String {
    let target = request.absolute_target();
    let host = target.host_str().unwrap();
    let port = target.port().unwrap_or(80);
    let addr = format!("{host}:{port}");
    addr
}
