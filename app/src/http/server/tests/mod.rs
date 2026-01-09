use std::{collections::VecDeque, net};

use crate::http::{
    self,
    response::{self, StatusCode},
    server::{HttpServe, InternalServerError},
    BufferBody, Request, Response, StreamBody,
};

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
        let body = StreamBody::new(Box::new(data));
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
