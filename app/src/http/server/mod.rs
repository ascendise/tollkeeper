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
    /// Creates a new HTTP [Server] with multiple [endpoints](Endpoint)
    pub fn create_http_endpoints(listener: net::TcpListener, endpoints: Vec<Endpoint>) -> Self {
        let handler = HttpEndpointsServe::new(endpoints);
        Self::new(listener, Box::new(handler))
    }

    /// Create low level TCP [Server]
    pub fn new(listener: net::TcpListener, handler: Box<dyn TcpServe + Send + Sync>) -> Self {
        Self { listener, handler }
    }

    /// Blocks execution and starts listening for connections.
    /// Connections get handled in independent threads
    pub fn start_listening(&mut self, cancel_receiver: CancelReceiver) -> Result<(), StartupError> {
        match self.listener.set_nonblocking(true) {
            Ok(_) => Ok(()),
            Err(e) => Err(StartupError::new(e.to_string())),
        }?;
        thread::scope(|s| {
            while !cancel_receiver.is_shutting_down() {
                let stream = match self.listener.accept() {
                    Ok(result) => result.0,
                    Err(_) => continue,
                };
                s.spawn(|| {
                    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        self.handler.serve_tcp(stream);
                    })); // Keep server alive when a request crashes handler
                });
            }
        });
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StartupError {
    msg: String,
}
impl Error for StartupError {}
impl Display for StartupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to start up server: '{}'", self.msg)
    }
}
impl StartupError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

/// Serve implementation that handles HTTP [requests](Request) and returns HTTP
/// [responses](Response)
pub struct HttpEndpointsServe {
    endpoints: Vec<Endpoint>,
}
impl HttpEndpointsServe {
    pub fn new(endpoints: Vec<Endpoint>) -> Self {
        Self { endpoints }
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
        match self::handle_incoming_request(self, stream.try_clone().unwrap()) {
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
    let mut response = match http_serve.serve_http(&write_stream.peer_addr().unwrap(), request) {
        Ok(res) => res,
        Err(_) => Response::internal_server_error(),
    };
    let response_raw = response.as_bytes();
    println!(
        "Sending Response: \r\n{}",
        String::from_utf8(response_raw.clone()).unwrap()
    );
    write_stream.write_all(&response_raw).unwrap();
    if let Body::Stream(body) = response.body() {
        println!("(Chunked Body)");
        while let Some(chunk) = body.read_chunk() {
            write_stream.write_all(chunk.content()).unwrap();
            println!("{}", String::from_utf8(chunk.content().into()).unwrap());
        }
    }
    Ok(())
}
fn send_response(mut stream: &net::TcpStream, mut response: Response) {
    let response = &response.as_bytes();
    stream.write_all(response).unwrap();
}

/// Return this error in case an unrecoverable error happened
#[derive(Debug, PartialEq, Eq)]
pub struct InternalServerError {}
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
