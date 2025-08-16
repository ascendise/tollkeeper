use std::collections::HashMap;
use std::net;

use crate::http::request::Method;
use crate::http::response::{self, StatusCode};
use crate::http::server::HttpServe;
use crate::http::{self, request, Headers, Request, StreamBody};
use crate::proxy::{OrderId, ProxyServe};
use crate::proxy::{PaymentRequiredError, Recipient, Toll};

use super::StubProxyService;

fn setup() -> ProxyServe {
    fn create_response() -> Result<http::Response, PaymentRequiredError> {
        let response = http::Response::new(
            StatusCode::OK,
            Some("OK".into()),
            response::Headers::empty(),
            None,
        );
        Ok(response)
    }
    let create_response = Box::new(create_response);
    let stub_proxy_service = StubProxyService::new(create_response);
    ProxyServe::new(Box::new(stub_proxy_service))
}

fn setup_with_failing_stub() -> ProxyServe {
    fn create_error() -> Result<http::Response, PaymentRequiredError> {
        let toll = Toll {
            recipient: Recipient {
                client_ip: "192.1.2.3".into(),
                user_agent: "Bot".into(),
                destination: "127.0.0.1".into(),
            },
            order_id: OrderId {
                gate_id: "12".into(),
                order_id: "13".into(),
            },
            challenge: HashMap::new(),
            signature: "do-not-modify".into(),
        };
        Err(PaymentRequiredError(Box::new(toll)))
    }
    let create_error = Box::new(create_error);
    let stub_proxy_service = StubProxyService::new(create_error);
    let stub_proxy_service = Box::new(stub_proxy_service);
    ProxyServe::new(stub_proxy_service)
}

const fn client_addr() -> net::SocketAddr {
    let v4_addr = net::Ipv4Addr::new(127, 0, 0, 1);
    let v4_addr = net::SocketAddrV4::new(v4_addr, 5501);
    net::SocketAddr::V4(v4_addr)
}

#[test]
pub fn serve_should_return_response_of_target() {
    // Arrange
    let sut = setup();
    // Act
    let mut headers = Headers::empty();
    headers.insert("Host", "127.0.0.1:65000");
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(Method::Get, "/", headers).unwrap();
    let response = sut.serve_http(&client_addr(), request);
    // Assert
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(StatusCode::OK, response.status_code());
}

#[test]
pub fn serve_should_return_payment_required_if_access_is_denied() {
    // Arrange
    let sut = setup_with_failing_stub();
    // Act
    let mut headers = Headers::empty();
    headers.insert("Host", "127.0.0.1:65000");
    headers.insert("Content-Length", "16");
    let headers = request::Headers::new(headers).unwrap();
    let body = StreamBody::new("Hello, Server!\r\n".as_bytes());
    let request = Request::with_body(Method::Get, "/", headers, Box::new(body)).unwrap();
    let response = sut.serve_http(&client_addr(), request);
    // Assert
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(StatusCode::PaymentRequired, response.status_code());
}
