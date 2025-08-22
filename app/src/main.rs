use http::server::*;
use proxy::{ProxyServe, ProxyServiceImpl};
use std::{collections::HashMap, io, net};

mod config;
mod data_formats;
#[allow(dead_code)]
mod http;
mod proxy;

fn main() -> Result<(), io::Error> {
    let listener = net::TcpListener::bind("127.0.0.1:9000")?;
    let tollkeeper = create_tollkeeper(true);
    let proxy_service = ProxyServiceImpl::new(tollkeeper);
    let base_url = url::Url::parse("http://localhost:9100/").unwrap();
    let server_config = config::ServerConfig::new(base_url);
    let proxy_handler = ProxyServe::new(server_config, Box::new(proxy_service));
    let mut server = Server::new(listener, Box::new(proxy_handler));
    let (_, receiver) = cancellation_token::create_cancellation_token();
    server.start_listening(receiver).unwrap();
    Ok(())
}

fn create_tollkeeper(requires_challenge: bool) -> tollkeeper::Tollkeeper {
    let destination = tollkeeper::descriptions::Destination::new("wtfismyip.com", 80, "/json");
    let description = StubDescription {
        is_match: requires_challenge,
    };
    let order = tollkeeper::Order::with_id(
        "order",
        vec![Box::new(description)],
        tollkeeper::AccessPolicy::Blacklist,
        Box::new(StubTollDeclaration),
    );
    println!("Order: {}", order.id());
    let orders = vec![order];
    let gate = tollkeeper::Gate::with_id("gate", destination, orders).unwrap();
    println!("Gate: {}", gate.id());
    let gates = vec![gate];
    let secret_key_provider =
        tollkeeper::signatures::InMemorySecretKeyProvider::new(b"Secret key".into());
    let secret_key_provider = Box::new(secret_key_provider);
    tollkeeper::Tollkeeper::new(gates, secret_key_provider).unwrap()
}

struct StubDescription {
    is_match: bool,
}
impl tollkeeper::Description for StubDescription {
    fn matches(&self, _: &tollkeeper::descriptions::Suspect) -> bool {
        self.is_match
    }
}

struct StubTollDeclaration;
impl tollkeeper::Declaration for StubTollDeclaration {
    fn declare(
        &self,
        suspect: tollkeeper::descriptions::Suspect,
        order_id: tollkeeper::declarations::OrderIdentifier,
    ) -> tollkeeper::declarations::Toll {
        tollkeeper::declarations::Toll::new(suspect, order_id, HashMap::new())
    }

    fn pay(
        &mut self,
        _: tollkeeper::declarations::Payment,
        _: &tollkeeper::descriptions::Suspect,
    ) -> Result<tollkeeper::declarations::Visa, tollkeeper::declarations::PaymentError> {
        todo!()
    }
}
