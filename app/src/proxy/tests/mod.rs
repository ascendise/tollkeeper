use std::net;

use crate::http;

use super::{PaymentRequiredError, ProxyService};

mod header_tests;
mod json_tests;
mod proxy_serve_tests;
mod proxy_service_tests;

pub struct StubProxyService {
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
