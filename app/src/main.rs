use http::server::*;
use proxy::{ProxyServe, ProxyServiceImpl};
use std::{io, net, sync::Arc, thread};
use tollkeeper::Tollkeeper;

use crate::templates::{handlebars::HandlebarTemplateRenderer, FileTemplateStore};

mod config;
mod data_formats;
#[allow(dead_code)]
mod http;
mod payment;
mod proxy;
#[allow(dead_code)]
mod templates;

fn main() -> Result<(), io::Error> {
    let base_url = url::Url::parse("http://localhost:9100/").unwrap();
    thread::scope(|s| {
        let proxy_config = config::ServerConfig::new(base_url);
        let api_config = proxy_config.clone();
        let tollkeeper = Arc::new(create_tollkeeper());
        let proxy_tollkeeper = tollkeeper.clone();
        let api_tollkeeper = tollkeeper.clone();
        s.spawn(move || {
            println!("Starting Proxy Socket");
            let (mut proxy_server, proxy_server_cancellation) =
                create_proxy_server(proxy_config, proxy_tollkeeper).unwrap();
            proxy_server
                .start_listening(proxy_server_cancellation)
                .unwrap();
        });
        s.spawn(move || {
            println!("Starting Api Socket");
            let (mut proxy_server, proxy_server_cancellation) =
                create_api_server(api_config, api_tollkeeper).unwrap();
            proxy_server
                .start_listening(proxy_server_cancellation)
                .unwrap();
        });
    });
    Ok(())
}

fn create_tollkeeper() -> tollkeeper::Tollkeeper {
    let json_api_destination =
        tollkeeper::descriptions::Destination::new("wtfismyip.com", 80, "/json");
    let json_api_gate = create_simple_gate("json_api_gate".into(), json_api_destination);
    let web_page_destination = tollkeeper::descriptions::Destination::new("localhost", 80, "/");
    let web_page_gate = create_simple_gate("web_page_gate".into(), web_page_destination);
    let web_page_gate_ext_destination =
        tollkeeper::descriptions::Destination::new("example.com", 80, "/");
    let web_page_gate_ext =
        create_simple_gate("web_page_gate_ext".into(), web_page_gate_ext_destination);
    let gates = vec![json_api_gate, web_page_gate, web_page_gate_ext];
    let secret_key_provider =
        tollkeeper::signatures::InMemorySecretKeyProvider::new(b"Secret key".into());
    let secret_key_provider = Box::new(secret_key_provider);
    tollkeeper::Tollkeeper::new(gates, secret_key_provider).unwrap()
}

fn create_simple_gate(
    gate_id: String,
    destination: tollkeeper::descriptions::Destination,
) -> tollkeeper::Gate {
    let date_provider = tollkeeper::util::DateTimeProviderImpl {};
    let double_spent_db = tollkeeper::declarations::hashcash::DoubleSpentDatabaseImpl::new();
    let hashcash_declaration = tollkeeper::declarations::hashcash::HashcashDeclaration::new(
        4,
        chrono::TimeDelta::hours(1),
        Box::new(date_provider),
        Box::new(double_spent_db),
    );
    let description = StubDescription { is_match: true };
    let order = tollkeeper::Order::with_id(
        "order",
        vec![Box::new(description)],
        tollkeeper::AccessPolicy::Blacklist,
        Box::new(hashcash_declaration),
    );
    println!("Order: {}", order.id());
    let orders = vec![order];
    let gate = tollkeeper::Gate::with_id(gate_id, destination, orders).unwrap();
    println!("Gate: {}", gate.id());
    gate
}

fn create_proxy_server(
    server_config: config::ServerConfig,
    tollkeeper: Arc<Tollkeeper>,
) -> Result<(Server, cancellation_token::CancelReceiver), io::Error> {
    let listener = net::TcpListener::bind("0.0.0.0:9000")?;
    let proxy_service = ProxyServiceImpl::new(tollkeeper);
    let exe_root_dir = std::env::current_dir().unwrap().join("app/templates");
    println!("Using templates located at: '{}'", exe_root_dir.display());
    let template_store = FileTemplateStore::new(exe_root_dir);
    let template_renderer = HandlebarTemplateRenderer::new(Box::new(template_store));
    let proxy_handler = ProxyServe::new(
        server_config,
        Box::new(proxy_service),
        Box::new(template_renderer),
    );
    let server = Server::new(listener, Box::new(proxy_handler));
    let (_, receiver) = cancellation_token::create_cancellation_token();
    Ok((server, receiver))
}

fn create_api_server(
    server_config: config::ServerConfig,
    tollkeeper: Arc<Tollkeeper>,
) -> Result<(Server, cancellation_token::CancelReceiver), io::Error> {
    let listener = net::TcpListener::bind("0.0.0.0:9100")?;
    let payment_service = payment::PaymentServiceImpl::new(tollkeeper);
    let payment_endpoints =
        payment::create_pay_toll_endpoint("/api/pay/", server_config, Box::new(payment_service));
    let server = Server::create_http_endpoints(listener, payment_endpoints);
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
