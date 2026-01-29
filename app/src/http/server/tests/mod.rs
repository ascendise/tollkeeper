use std::{
    collections::VecDeque,
    io, net,
    sync::{Arc, Mutex},
};

use crate::http::{
    self,
    response::{self, StatusCode},
    server::{HttpServe, InternalServerError},
    BufferBody, ChunkedTcpStream, Request, Response, StreamBody,
};

use pretty_assertions::assert_eq;

mod http_endpoints_serve_tests;
mod server_tests;

struct HelloHandler {
    body: Vec<u8>,
}
impl HttpServe for HelloHandler {
    fn serve_http(&self, _: &net::SocketAddr, _: Request) -> Result<Response, InternalServerError> {
        let mut headers = http::Headers::empty();
        headers.insert("Content-Length", self.body.len().to_string());
        let headers = response::Headers::new(headers);
        let data: VecDeque<u8> = self.body.clone().into();
        let body = BufferBody::new(data);
        let body = http::Body::Buffer(body);
        let response = Response::new(StatusCode::OK, Some("OK".into()), headers, body);
        Ok(response)
    }
}

struct ChunkedHandler {
    chunked_body: Vec<u8>,
}
impl HttpServe for ChunkedHandler {
    fn serve_http(&self, _: &net::SocketAddr, _: Request) -> Result<Response, InternalServerError> {
        let mut headers = http::Headers::empty();
        headers.insert("Transfer-Encoding", "chunked");
        let headers = response::Headers::new(headers);
        let data: VecDeque<u8> = self.chunked_body.clone().into();
        let stream = ChunkedTcpStream::new(Box::new(io::BufReader::new(data)));
        let body = StreamBody::new(Box::new(stream));
        let body = http::Body::Stream(body);
        let response = Response::new(StatusCode::OK, Some("OK".into()), headers, body);
        Ok(response)
    }
}

struct FailingHandler;
impl HttpServe for FailingHandler {
    fn serve_http(&self, _: &net::SocketAddr, _: Request) -> Result<Response, InternalServerError> {
        Err(InternalServerError::new())
    }
}

struct PanicHandler;
impl HttpServe for PanicHandler {
    fn serve_http(&self, _: &net::SocketAddr, _: Request) -> Result<Response, InternalServerError> {
        panic!("I am trying to kill the server")
    }
}

#[derive(Default, Clone)]
struct IpSpyHandler {
    ips: Arc<Mutex<Vec<net::SocketAddr>>>,
}

impl IpSpyHandler {
    fn assert_ips_equal(&self, expected_ips: &[net::SocketAddr]) {
        let ips = self.ips.lock().unwrap();
        assert_eq!(expected_ips, ips.as_slice());
    }
}
impl HttpServe for IpSpyHandler {
    fn serve_http(
        &self,
        client_addr: &net::SocketAddr,
        _: Request,
    ) -> Result<Response, InternalServerError> {
        let headers = response::Headers::empty();
        self.ips.lock().unwrap().push(*client_addr);
        Ok(Response::new(
            StatusCode::NoContent,
            Some("No Content".into()),
            headers,
            http::Body::None,
        ))
    }
}
