use std::{
    net,
    sync::{Arc, Mutex},
};

use crate::http;
use pretty_assertions::assert_eq;

use super::{PaymentRequiredError, ProxyService};

mod header_tests;
mod json_tests;
mod proxy_serve_tests;
mod proxy_service_tests;

struct StubProxyService {
    proxy_request_result:
        Box<dyn Fn() -> Result<http::Response, PaymentRequiredError> + Send + Sync + 'static>,
}
impl StubProxyService {
    pub fn new(
        proxy_request_result: Box<
            dyn Fn() -> Result<http::Response, PaymentRequiredError> + Send + Sync + 'static,
        >,
    ) -> Self {
        Self {
            proxy_request_result,
        }
    }
}
impl ProxyService for StubProxyService {
    fn proxy_request(
        &self,
        _: &net::SocketAddr,
        _: http::Request,
    ) -> Result<http::Response, PaymentRequiredError> {
        (self.proxy_request_result)()
    }
}

#[derive(Default, Clone)]
struct SpyProxyService {
    proxy_request_calls: Arc<Mutex<Vec<ProxyRequestCall>>>,
}
impl ProxyService for SpyProxyService {
    fn proxy_request(
        &self,
        client_addr: &net::SocketAddr,
        req: http::Request,
    ) -> Result<http::Response, PaymentRequiredError> {
        let call = ProxyRequestCall {
            client_addr: *client_addr,
            request_headers: req.headers().clone(),
        };
        let mut calls = self.proxy_request_calls.lock().unwrap();
        calls.push(call);
        Ok(http::Response::new(
            http::response::StatusCode::NoContent,
            Some("Stub".to_string()),
            http::response::Headers::new(http::Headers::empty()),
            http::Body::None,
        ))
    }
}
impl SpyProxyService {
    fn assert_all_calls(&self, expected_calls: &Vec<ProxyRequestCall>) {
        let calls = self.proxy_request_calls.lock().unwrap();
        assert_eq!(
            expected_calls,
            calls.as_slice(),
            "Calls on SpyProxyService do not match",
        );
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ProxyRequestCall {
    client_addr: net::SocketAddr,
    request_headers: http::request::Headers,
}
