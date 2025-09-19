use http::server::*;
use proxy::{ProxyServe, ProxyServiceImpl};
use std::{io, net, thread};

mod config;
mod data_formats;
#[allow(dead_code)]
mod http;
mod payment;
mod proxy;

fn main() -> Result<(), io::Error> {
    let base_url = url::Url::parse("http://localhost:9100/").unwrap();
    thread::scope(|s| {
        let proxy_config = config::ServerConfig::new(base_url);
        let api_config = proxy_config.clone();
        s.spawn(move || {
            println!("Starting Proxy Socket");
            let (mut proxy_server, proxy_server_cancellation) =
                create_proxy_server(proxy_config).unwrap();
            proxy_server
                .start_listening(proxy_server_cancellation)
                .unwrap();
        });
        s.spawn(move || {
            println!("Starting Api Socket");
            let (mut proxy_server, proxy_server_cancellation) =
                create_api_server(api_config).unwrap();
            proxy_server
                .start_listening(proxy_server_cancellation)
                .unwrap();
        });
    });
    Ok(())
}

fn create_proxy_server(
    server_config: config::ServerConfig,
) -> Result<(Server, cancellation_token::CancelReceiver), io::Error> {
    let listener = net::TcpListener::bind("127.0.0.1:9000")?;
    let tollkeeper = create_tollkeeper(true);
    let proxy_service = ProxyServiceImpl::new(tollkeeper);
    let proxy_handler = ProxyServe::new(server_config, Box::new(proxy_service));
    let server = Server::new(listener, Box::new(proxy_handler));
    let (_, receiver) = cancellation_token::create_cancellation_token();
    Ok((server, receiver))
}

fn create_tollkeeper(requires_challenge: bool) -> tollkeeper::Tollkeeper {
    let destination = tollkeeper::descriptions::Destination::new("wtfismyip.com", 80, "/json");
    let date_provider = tollkeeper::util::DateTimeProviderImpl {};
    let double_spent_db = tollkeeper::declarations::hashcash::DoubleSpentDatabaseImpl::new();
    let hashcash_declaration = tollkeeper::declarations::hashcash::HashcashDeclaration::new(
        4,
        chrono::TimeDelta::hours(1),
        Box::new(date_provider),
        Box::new(double_spent_db),
    );
    let description = StubDescription {
        is_match: requires_challenge,
    };
    let order = tollkeeper::Order::with_id(
        "order",
        vec![Box::new(description)],
        tollkeeper::AccessPolicy::Blacklist,
        Box::new(hashcash_declaration),
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

fn create_api_server(
    server_config: config::ServerConfig,
) -> Result<(Server, cancellation_token::CancelReceiver), io::Error> {
    let listener = net::TcpListener::bind("127.0.0.1:9100")?;
    let tollkeeper = create_tollkeeper(true);
    let payment_service = payment::PaymentServiceImpl::new(tollkeeper);
    let payment_endpoint =
        payment::create_pay_toll_endpoint("/api/pay", server_config, Box::new(payment_service));
    let server = Server::create_http_endpoints(listener, vec![payment_endpoint]);
    let (_, receiver) = cancellation_token::create_cancellation_token();
    Ok((server, receiver))
}

struct StubDescription {
    is_match: bool,
}
impl tollkeeper::Description for StubDescription {
    fn matches(&self, _: &tollkeeper::descriptions::Suspect) -> bool {
        self.is_match
    }
}
