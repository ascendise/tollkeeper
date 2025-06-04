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
    endpoints: Arc<Mutex<Vec<Endpoint>>>,
}
impl Server {
    pub fn new(listener: net::TcpListener, endpoints: Vec<Endpoint>) -> Self {
        Self {
            listener,
            endpoints: Arc::new(Mutex::new(endpoints)),
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
                let endpoints = self.endpoints.clone();
                s.spawn(move || {
                    let result = panic::catch_unwind(|| {
                        match Self::handle_incoming_request(endpoints, stream.try_clone().unwrap())
                        {
                            Ok(_) => (),
                            Err(_) => Self::send_request(&stream, Response::bad_request()),
                        }
                    });
                    match result {
                        Ok(_) => (),
                        Err(_) => Self::send_request(&stream, Response::internal_server_error()),
                    }
                });
            }
        });
        Ok(())
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
                Some(e) => Self::serve(e, &mut request),
                None => Response::method_not_allowed(),
            }
        } else {
            Response::not_found()
        };
        write_stream.write_all(&response.into_bytes()).unwrap();
        Ok(())
    }

    fn serve(endpoint: &mut Endpoint, request: &mut Request) -> Response {
        endpoint.serve(request)
    }

    fn send_request(mut stream: &net::TcpStream, response: Response) {
        stream.write_all(&response.into_bytes()).unwrap();
    }
}

pub struct Endpoint {
    method: Method,
    path: String,
    handler: Box<dyn Serve + Sync + Send>,
}

impl Endpoint {
    pub fn new(
        method: Method,
        path: impl Into<String>,
        handler: Box<dyn Serve + Sync + Send>,
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

pub trait Serve {
    fn serve(&self, request: &mut Request) -> Response;
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
