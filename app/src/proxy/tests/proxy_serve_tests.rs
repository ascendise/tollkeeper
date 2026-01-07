use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::net;

use tollkeeper::signatures::Base64;

use crate::config;
use crate::http::request::Method;
use crate::http::response::{self, StatusCode};
use crate::http::server::HttpServe;
use crate::http::{self, request, Headers, Request, StreamBody};
use crate::proxy::{Challenge, OrderId, ProxyServe};
use crate::proxy::{PaymentRequiredError, Recipient, Toll};
use crate::templates::{handlebars::HandlebarTemplateRenderer, InMemoryTemplateStore};
use test_case::test_case;

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
    let template_store = InMemoryTemplateStore::new(HashMap::new());
    let template_renderer = HandlebarTemplateRenderer::new(Box::new(template_store));
    ProxyServe::new(
        server_config,
        Box::new(stub_proxy_service),
        Box::new(template_renderer),
    )
}

fn setup_with_failing_stub(templates: Option<HashMap<String, String>>) -> ProxyServe {
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
    let server_config =
        config::ServerConfig::new(url::Url::parse("http://guard.tollkeeper.ch/").unwrap());
    let templates = templates.unwrap_or_default();
    let template_store = InMemoryTemplateStore::new(templates);
    let template_renderer = HandlebarTemplateRenderer::new(Box::new(template_store));
    ProxyServe::new(
        server_config,
        Box::new(stub_proxy_service),
        Box::new(template_renderer),
    )
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
    let sut = setup_with_failing_stub(None);
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

#[test_case("text/html" ; "simple text/html header")]
#[test_case(
    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8" ;
    "firefox 132+"
)]
#[test_case("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8" ; "Safari 18+")]
#[test_case("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7" ; "Chrome 131+")]
#[test_case("text/html, application/xhtml+xml, image/jxr, */*" ; "Edge")]
#[test_case("text/html, application/xml;q=0.9, application/xhtml+xml, image/png, image/webp, image/jpeg, image/gif, image/x-xbitmap, */*;q=0.1" ; "Opera")]
pub fn serve_should_return_challenge_html_page_if_request_accepts_html(accept_header: &str) {
    // Arrange
    let mut stub_templates = HashMap::new();
    stub_templates.insert("challenge.html".into(), "<div>Stub</div>".into());
    let sut = setup_with_failing_stub(Some(stub_templates));
    // Act
    let mut headers = Headers::empty();
    headers.insert("Host", "127.0.0.1:65000");
    headers.insert("Content-Length", "16");
    headers.insert("Accept", accept_header);
    let headers = request::Headers::new(headers).unwrap();
    let body = StreamBody::new("Hello, Server!\r\n".as_bytes());
    let request = Request::with_body(Method::Get, "/", headers, Box::new(body)).unwrap();
    let response = sut.serve_http(&client_addr(), request);
    // Assert
    assert!(response.is_ok());
    let mut response = response.unwrap();
    assert_eq!(StatusCode::PaymentRequired, response.status_code());
    assert_eq!(Some("text/html"), response.headers().content_type());
    assert!(response.headers().content_length().is_some());
    let expected_body = "<div>Stub</div>";
    let mut actual_body = String::new();
    response
        .body()
        .unwrap()
        .read_to_string(&mut actual_body)
        .unwrap();
    assert_eq!(expected_body, actual_body);
}

#[test]
pub fn serve_should_return_internal_server_error_on_render_failure() {
    // Arrange
    let mut stub_templates = HashMap::new();
    stub_templates.insert(
        "challenge.html".into(),
        "<div>{{unclosed-placeholder</div>".into(), //invalid template
    );
    let sut = setup_with_failing_stub(Some(stub_templates));
    // Act
    let mut headers = Headers::empty();
    headers.insert("Host", "127.0.0.1:65000");
    headers.insert("Content-Length", "16");
    headers.insert("Accept", "text/html");
    let headers = request::Headers::new(headers).unwrap();
    let body = StreamBody::new("Hello, Server!\r\n".as_bytes());
    let request = Request::with_body(Method::Get, "/", headers, Box::new(body)).unwrap();
    let response = sut.serve_http(&client_addr(), request);
    // Assert
    assert!(
        response.is_err(),
        "invalid template did not return Internal Server Error"
    );
}
