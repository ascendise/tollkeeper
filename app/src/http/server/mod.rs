pub mod cancellation_token;
#[cfg(test)]
mod tests;

use cancellation_token::CancelReceiver;

use super::{
    request::{self, Method, Parse, Request},
    response::Response,
};
use std::{
    error::Error,
    fmt::Display,
    io::{self, Write},
    panic,
    sync::Mutex,
    thread,
};
use std::{net, sync::Arc};

pub struct Server {
    listener: net::TcpListener,
    handler: Arc<Mutex<Box<dyn TcpServe + Send + Sync>>>,
}
impl Server {
    /// Creates a new HTTP [Server] with multiple [endpoints](Endpoint)
    pub fn create_http_endpoints(listener: net::TcpListener, endpoints: Vec<Endpoint>) -> Self {
        let handler = HttpEndpointsServe::new(Arc::new(Mutex::new(endpoints)));
        Self {
            listener,
            handler: Arc::new(Mutex::new(Box::new(handler))),
        }
    }

    /// Blocks execution and starts listening for connections.
    /// Connections get handled in independant threads
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
                let handler = self.handler.clone();
                s.spawn(move || {
                    handler.lock().unwrap().serve(stream);
                });
            }
        });
        Ok(())
    }
}

/// Serve implementation that handles HTTP [requests](Request) and returns HTTP
/// [responses](Response)
pub struct HttpEndpointsServe {
    endpoints: Arc<Mutex<Vec<Endpoint>>>,
}
impl TcpServe for HttpEndpointsServe {
    fn serve(&self, stream: net::TcpStream) {
        let endpoints = self.endpoints.clone();
        let result = panic::catch_unwind(|| {
            match Self::handle_incoming_request(endpoints, stream.try_clone().unwrap()) {
                Ok(_) => (),
                Err(_) => Self::send_request(&stream, Response::bad_request()),
            }
        });
        match result {
            Ok(_) => (),
            Err(_) => Self::send_request(&stream, Response::internal_server_error()),
        }
    }
}
impl HttpEndpointsServe {
    pub fn new(endpoints: Arc<Mutex<Vec<Endpoint>>>) -> Self {
        Self { endpoints }
    }

    fn handle_incoming_request(
        endpoints: Arc<Mutex<Vec<Endpoint>>>,
        stream: net::TcpStream,
    ) -> Result<(), request::ParseError> {
        let mut write_stream = stream.try_clone().unwrap();
        let reader = io::BufReader::new(stream);
        let mut request = Request::parse(reader)?;
        let mut endpoints = endpoints.lock().unwrap();
        let mut endpoints = endpoints
            .iter_mut()
            .filter(|e| request.matches_path(&e.path))
            .peekable();
        let response = if endpoints.peek().is_some() {
            match endpoints.find(|e| request.matches_method(&e.method)) {
                Some(e) => e.serve(&mut request),
                None => Response::method_not_allowed(),
            }
        } else {
            Response::not_found()
        };
        write_stream.write_all(&response.into_bytes()).unwrap();
        Ok(())
    }

    fn send_request(mut stream: &net::TcpStream, response: Response) {
        stream.write_all(&response.into_bytes()).unwrap();
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

    pub fn serve(&mut self, request: &mut Request) -> Response {
        self.handler.serve(request)
    }
}

pub trait HttpServe {
    fn serve(&self, request: &mut Request) -> Response;
}

pub trait TcpServe {
    fn serve(&self, stream: net::TcpStream);
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
