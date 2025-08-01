use std::{collections::VecDeque, net};

use crate::http::{
    self,
    response::{self, StatusCode},
    server::{HttpServe, InternalServerError},
    Request, Response, StreamBody,
};

mod http_endpoints_serve_tests;
mod server_tests;

struct HelloHandler {
    body: Vec<u8>,
}
impl HttpServe for HelloHandler {
    fn serve_http(&self, _: &net::SocketAddr, _: Request) -> Result<Response, InternalServerError> {
        let mut headers = indexmap::IndexMap::<String, String>::new();
        headers.insert("Content-Length".into(), self.body.len().to_string());
        let headers = http::Headers::new(headers);
        let headers = response::Headers::new(headers);
        let body = Box::new(StreamBody::<VecDeque<u8>>::new(self.body.clone().into()));
        let response = Response::new(StatusCode::OK, Some("OK".into()), headers, Some(body));
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
