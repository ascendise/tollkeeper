use http::server::*;
use proxy::ProxyServe;
use std::{io, net};

#[allow(dead_code)]
mod http;
mod proxy;

fn main() -> Result<(), io::Error> {
    let listener = net::TcpListener::bind("127.0.0.1:9000")?;
    let proxy_handler = ProxyServe::new();
    let mut server = Server::new(listener, Box::new(proxy_handler));
    let (_, receiver) = cancellation_token::create_cancellation_token();
    server.start_listening(receiver).unwrap();
    Ok(())
}
