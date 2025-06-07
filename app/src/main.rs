use std::{
    io::{self, Read},
    net,
};

use http::{request::*, response::*, server::*, Headers};

#[allow(dead_code)]
mod http;

fn main() -> Result<(), io::Error> {
    let listener = net::TcpListener::bind("127.0.0.1:9000")?;
    let echo_handler = Box::new(EchoHandler {});
    let panic_handler = Box::new(PanicHandler {});
    let endpoints = vec![
        Endpoint::new(Method::Post, "/echo", echo_handler),
        Endpoint::new(Method::Post, "/panic", panic_handler),
    ];
    let mut server = Server::create_http_endpoints(listener, endpoints);
    let (_, receiver) = cancellation_token::create_cancellation_token();
    server.start_listening(receiver).unwrap();
    Ok(())
}

struct EchoHandler;
impl HttpServe for EchoHandler {
    fn serve(&self, request: &mut Request) -> Response {
        let headers = Headers::new(indexmap::IndexMap::new());
        let headers = ResponseHeaders::new(headers);
        let body = match &mut request.body() {
            Some(s) => {
                let mut body = String::new();
                match s.read_to_string(&mut body) {
                    Ok(_) => {}
                    Err(e) => println!("{e}"),
                };
                body
            }
            None => String::new(),
        };
        Response::with_reason_phrase(StatusCode::OK, "OK", headers, body.into_bytes())
    }
}

struct PanicHandler;
impl HttpServe for PanicHandler {
    fn serve(&self, _: &mut Request) -> Response {
        panic!("Called wrong handler!")
    }
}
