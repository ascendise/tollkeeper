use std::net;

use tollkeeper::signatures::Base64;

use crate::config;
use crate::http::request::Method;
use crate::http::response::{self, StatusCode};
use crate::http::server::HttpServe;
use crate::http::{self, request, Headers, Request, StreamBody};
use crate::proxy::{Challenge, OrderId, ProxyServe};
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
    let server_config =
        config::ServerConfig::new(url::Url::parse("http://guard.tollkeeper.ch/").unwrap());
    ProxyServe::new(server_config, Box::new(stub_proxy_service))
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
            challenge: Challenge::new(Vec::new()),
            signature: Base64::encode(b"do-not-modify"),
        };
        Err(PaymentRequiredError(Box::new(toll)))
    }
    let create_error = Box::new(create_error);
    let stub_proxy_service = StubProxyService::new(create_error);
    let stub_proxy_service = Box::new(stub_proxy_service);
    let server_config =
        config::ServerConfig::new(url::Url::parse("http://guard.tollkeeper.ch/").unwrap());
    ProxyServe::new(server_config, stub_proxy_service)
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
    let mut response = response.unwrap();
    assert_eq!(StatusCode::PaymentRequired, response.status_code());

    assert_eq!(
        Some("application/hal+json"),
        response.headers().content_type()
    );
    assert!(response.headers().content_length().is_some());
    let expected_toll = serde_json::json!({
        "toll": {
            "recipient": {
                "client_ip": "192.1.2.3",
                "user_agent": "Bot",
                "destination": "127.0.0.1",
            },
            "order_id": "12#13",
            "challenge": {},
            "signature": Base64::encode(b"do-not-modify"),
        },
        "_links": {
            "pay": "http://guard.tollkeeper.ch/api/pay/"
        }
    });
    let mut actual_toll = String::new();
    response
        .body()
        .unwrap()
        .read_to_string(&mut actual_toll)
        .unwrap();
    let actual_toll: serde_json::Value = serde_json::from_str(&actual_toll).unwrap();
    assert_eq!(expected_toll, actual_toll);
}
