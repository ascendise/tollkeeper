use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net;

use crate::http::request::Parse;
use crate::http::request::Request;
use crate::http::response::Response;

use super::http::server::*;

#[cfg(test)]
mod tests;

pub struct ProxyServe {}

impl ProxyServe {}
impl TcpServe for ProxyServe {
    fn serve(&self, mut stream: std::net::TcpStream) {
        let reader = BufReader::new(stream.try_clone().unwrap());
        let request = match Request::parse(reader) {
            Ok(v) => v,
            Err(_) => {
                send_response(stream, Response::bad_request());
                return;
            }
        };
        let addr = get_host(request);
        let mut conn = net::TcpStream::connect(addr).unwrap();
        //TODO:
        // Return error indicating error from other server
        let mut response = String::new();
        conn.read_to_string(&mut response).unwrap();
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn get_host(request: Request) -> String {
    let target = request.absolute_target();
    let host = target.host_str().unwrap();
    let port = target.port().unwrap();
    let addr = format!("{host}:{port}");
    addr
}

fn send_response(mut stream: net::TcpStream, response: Response) {
    let response = response.into_bytes();
    stream.write_all(&response).unwrap()
}
