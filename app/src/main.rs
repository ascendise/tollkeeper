use http::server::*;
use proxy::{ProxyServe, ProxyServiceImpl};
use std::{
    fs, io, net,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    thread,
};
use tollkeeper::Tollkeeper;
use tracing::{event, span, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    files::{FileReaderImpl, FileServe},
    http::request::Method,
    templates::{handlebars::HandlebarTemplateRenderer, FileTemplateStore},
};

mod config;
mod data_formats;
mod files;
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
        let url_resolver = Box::new(config.create_url_resolver());
        let proxy_tollkeeper = tollkeeper.clone();
        let proxy_config = config.api.clone();
        let server_config = config.server();
        let proxy_port = server_config.proxy_port();
        s.spawn(move || {
            let _span = span!(Level::INFO, "[Proxy]").entered();
            event!(Level::INFO, "Startup on Port {proxy_port}");
            let (mut proxy_server, proxy_server_cancellation) =
                create_proxy_server(proxy_port, proxy_config, proxy_tollkeeper, url_resolver)
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
    let env = std::env::var("RUST_ENV").unwrap_or("".into());
    let config_path = if env.is_empty() {
        String::from("app/config.toml")
    } else {
        format!("app/config.{}.toml", env)
    };
    let config_path = std::env::current_dir().unwrap().join(config_path);
    event!(Level::INFO, "Read config from {}", config_path.display());
    let config = fs::read_to_string(config_path.clone())
        .unwrap_or_else(|_| panic!("Cannot find config file at {}", config_path.display()));
    config::Config::from_toml(&config).unwrap()
}

fn create_proxy_server(
    port: usize,
    server_config: config::Api,
    tollkeeper: Arc<Tollkeeper>,
    url_resolver: Box<dyn proxy::UrlResolver + Send + Sync>,
) -> Result<(Server, cancellation_token::CancelReceiver), io::Error> {
    let listener = net::TcpListener::bind(format!("0.0.0.0:{port}"))?;

    let proxy_service = ProxyServiceImpl::new(tollkeeper, url_resolver);
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
    let mut api_endpoints = vec![];
    let mut payment_endpoints = payment::create_pay_toll_endpoint(
        "/api/pay/",
        server_config.clone(),
        Box::new(payment_service),
    );
    api_endpoints.append(&mut payment_endpoints);
    api_endpoints.append(&mut create_file_endpoints("app/assets", "app/"));
    let http_endpoints = HttpEndpointsServe::new(api_endpoints, server_config.real_ip_header);
    let server = Server::new(listener, Box::new(http_endpoints));
    let (_, receiver) = cancellation_token::create_cancellation_token();
    Ok((server, receiver))
}

fn create_file_endpoints(fs_dir: &str, path_prefix: &str) -> Vec<Endpoint> {
    let mut endpoints = vec![];
    let path = PathBuf::from_str(fs_dir).unwrap();
    let files = find_files(&path);
    for file in files {
        let endpoint = create_file_endpoint(file, path_prefix);
        endpoints.push(endpoint);
    }
    endpoints
}

fn find_files(path: &Path) -> Vec<PathBuf> {
    let mut all_entries = vec![];
    let entries = fs::read_dir(path).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            let mut subentries = find_files(&entry.path());
            all_entries.append(&mut subentries);
        }
        all_entries.push(entry.path());
    }
    all_entries
}

fn create_file_endpoint(file: PathBuf, path_prefix: &str) -> Endpoint {
    let mut api_path = PathBuf::from_str("/").unwrap();
    let api_file = file.strip_prefix(path_prefix).unwrap();
    api_path.push(api_file);
    tracing::info!(
        "Serving file {} from {}",
        api_path.display(),
        file.display()
    );
    let mut file_serve = FileServe::new(api_path.clone(), Box::new(FileReaderImpl));
    file_serve.set_fs_path(file);
    Endpoint::new(
        Method::Get,
        api_path.to_string_lossy(),
        Box::new(file_serve),
    )
}

struct StubDescription {
    is_match: bool,
}
impl tollkeeper::Description for StubDescription {
    fn matches(&self, _: &tollkeeper::descriptions::Suspect) -> bool {
        self.is_match
    }
}
