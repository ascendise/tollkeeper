use http::server::*;
use proxy::{ProxyServe, ProxyServiceImpl};
use std::{fs, io, net, sync::Arc, thread};
use tollkeeper::Tollkeeper;
use tracing::{event, span, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    setup_logging();
    thread::scope(|s| {
        let config = read_config();
        let tollkeeper = Arc::new(
            config
                .create_tollkeeper()
                .expect("Failed to create tollkeeper"),
        );
        let proxy_tollkeeper = tollkeeper.clone();
        let proxy_config = config.api.clone();
        let server_config = config.server();
        let proxy_port = server_config.proxy_port();
        s.spawn(move || {
            let _span = span!(Level::INFO, "[Proxy]").entered();
            event!(Level::INFO, "Startup on Port {proxy_port}");
            let (mut proxy_server, proxy_server_cancellation) =
                create_proxy_server(proxy_port, proxy_config, proxy_tollkeeper)
                    .expect("Error during startup (proxy)");
            proxy_server
                .start_listening(proxy_server_cancellation)
                .expect("Error during listening (proxy)");
        });
        let api_tollkeeper = tollkeeper.clone();
        let api_config = config.api.clone();
        let api_port = server_config.api_port();
        s.spawn(move || {
            let _span = span!(Level::INFO, "[API]").entered();
            event!(Level::INFO, "Startup on Port {api_port}");
            let (mut api_server, api_server_cancellation) =
                create_api_server(api_port, api_config, api_tollkeeper)
                    .expect("Error during startup (api)");
            api_server
                .start_listening(api_server_cancellation)
                .expect("Error during listening (proxy)");
        });
    });
    Ok(())
}

fn setup_logging() {
    let format = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(true);
    tracing_subscriber::registry()
        .with(format)
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
}

fn read_config() -> config::Config {
    let config_path = std::env::current_dir().unwrap().join("app/config.toml");
    event!(Level::INFO, "Read config from {}", config_path.display());
    let config = fs::read_to_string(config_path.clone())
        .unwrap_or_else(|_| panic!("Cannot find config file at {}", config_path.display()));
    config::Config::from_toml(&config).unwrap()
}

fn create_proxy_server(
    port: usize,
    server_config: config::Api,
    tollkeeper: Arc<Tollkeeper>,
) -> Result<(Server, cancellation_token::CancelReceiver), io::Error> {
    let listener = net::TcpListener::bind(format!("0.0.0.0:{port}"))?;
    let proxy_service = ProxyServiceImpl::new(tollkeeper);
    let exe_root_dir = std::env::current_dir().unwrap().join("app/templates");
    event!(
        Level::INFO,
        "Using templates located at: '{}'",
        exe_root_dir.display()
    );
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
    port: usize,
    server_config: config::Api,
    tollkeeper: Arc<Tollkeeper>,
) -> Result<(Server, cancellation_token::CancelReceiver), io::Error> {
    let listener = net::TcpListener::bind(format!("0.0.0.0:{port}"))?;
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
