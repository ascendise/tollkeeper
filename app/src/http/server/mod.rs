pub mod cancellation_token;
#[cfg(test)]
mod tests;

use cancellation_token::CancelReceiver;

use crate::http::Body;

use super::{
    parsing,
    request::{Method, Request},
    response::Response,
    Parse,
};
use std::net;
use std::{
    error::Error,
    fmt::Display,
    io::{self, Write},
    panic, thread,
};

pub struct Server {
    listener: net::TcpListener,
    handler: Box<dyn TcpServe + Send + Sync>,
}
impl Server {
    /// Create low level TCP [Server]
    pub fn new(listener: net::TcpListener, handler: Box<dyn TcpServe + Send + Sync>) -> Self {
        Self { listener, handler }
    }

    /// Blocks execution and starts listening for connections.
    /// Connections get handled in independent threads
    pub fn start_listening(&mut self, cancel_receiver: CancelReceiver) {
        thread::scope(|s| {
            while !cancel_receiver.is_shutting_down() {
                let stream = match self.listener.accept() {
                    Ok(result) => result.0,
                    Err(_) => continue,
                };
                let handler = &self.handler;
                s.spawn(move || {
                    let res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        handler.serve_tcp(stream);
                    })); // Keep server alive when a request crashes handler
                    match res {
                        Ok(_) => tracing::debug!("Request handled exceptionless!"),
                        Err(e) => tracing::error!(panic = ?e, "Request failed"),
                    }
                });
            }
        });
    }
}

/// Serve implementation that handles HTTP [requests](Request) and returns HTTP
/// [responses](Response)
pub struct HttpEndpointsServe {
    real_ip_header: Option<String>,
    endpoints: Vec<Endpoint>,
}
impl HttpEndpointsServe {
    pub fn new(endpoints: Vec<Endpoint>, real_ip_header: Option<String>) -> Self {
        Self {
            endpoints,
            real_ip_header,
        }
    }
}
impl HttpServe for HttpEndpointsServe {
    fn serve_http(
        &self,
        client_addr: &net::SocketAddr,
        request: Request,
    ) -> Result<Response, InternalServerError> {
        let mut endpoints = self
            .endpoints
            .iter()
            .filter(|e| request.matches_path(&e.path))
            .peekable();
        let client_addr = match &self.real_ip_header {
            Some(h) => &request
                .headers()
                .read_real_ip(h)
                .expect("No IP Header found"),
            None => client_addr,
        };
        if endpoints.peek().is_some() {
            match endpoints.find(|e| request.matches_method(&e.method)) {
                Some(e) => e.serve(client_addr, request),
                None => Ok(Response::method_not_allowed()),
            }
        } else {
            Ok(Response::not_found())
        }
    }
}

pub struct Endpoint {
    method: Method,
    path: String,
    handler: Box<dyn HttpServe + Sync + Send>,
}
impl Endpoint {
    pub fn new(
        method: Method,
        path: impl Into<String>,
        handler: Box<dyn HttpServe + Sync + Send>,
    ) -> Self {
        Self {
            method,
            path: path.into(),
            handler,
        }
    }

    pub fn serve(
        &self,
        client_addr: &net::SocketAddr,
        request: Request,
    ) -> Result<Response, InternalServerError> {
        self.handler.serve_http(client_addr, request)
    }
}

pub trait TcpServe {
    fn serve_tcp(&self, stream: net::TcpStream);
}
pub trait HttpServe {
    fn serve_http(
        &self,
        client_addr: &net::SocketAddr,
        request: Request,
    ) -> Result<Response, InternalServerError>;
}
impl<T: HttpServe> TcpServe for T {
    fn serve_tcp(&self, stream: net::TcpStream) {
        match handle_incoming_request(self, stream.try_clone().unwrap()) {
            Ok(_) => (),
            Err(_) => send_response(&stream, Response::bad_request()),
        }
    }
}
fn handle_incoming_request(
    http_serve: &impl HttpServe,
    stream: net::TcpStream,
) -> Result<(), parsing::ParseError> {
    let mut write_stream = stream.try_clone().unwrap();
    let reader = io::BufReader::new(stream);
    let request = Request::parse(reader)?;
    tracing::debug!("Incoming Request:\r\n{request}");
    let mut response = match http_serve.serve_http(&write_stream.peer_addr().unwrap(), request) {
        Ok(res) => res,
        Err(_) => Response::internal_server_error(),
    };
    tracing::debug!("Outgoing Response:\r\n{response}",);
    let response_raw = response.as_bytes();
    write_stream.write_all(&response_raw).unwrap();
    if let Body::Stream(body) = response.body() {
        let mut body = body;
        io::copy(&mut body, &mut write_stream).unwrap();
    }
    Ok(())
}

fn send_response(mut stream: &net::TcpStream, mut response: Response) {
    let response = &response.as_bytes();
    stream.write_all(response).unwrap();
}

/// Return this error in case an unrecoverable error happened
#[derive(Debug, PartialEq, Eq)]
pub struct InternalServerError;
impl InternalServerError {
    pub fn new() -> Self {
        Self {}
    }
}
impl Error for InternalServerError {}
impl Display for InternalServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unrecoverable error happened while processing request",)
    }
}
