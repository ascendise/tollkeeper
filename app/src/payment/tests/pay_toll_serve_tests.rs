use pretty_assertions::assert_eq;
use std::{
    collections::VecDeque,
    io::Read,
    net::{SocketAddr, SocketAddrV4},
    str::FromStr,
};

use serde_json::json;
use test_case::test_case;
use tollkeeper::signatures::Base64;

use crate::{
    config,
    data_formats::AsHalJson,
    http::{self, server::HttpServe},
    payment::{self, PayTollServe, PaymentError, PaymentService},
    proxy::{self, Challenge},
};

fn setup(
    result: Box<
        dyn Fn() -> Result<payment::Visa, Box<payment::PaymentError>> + Send + Sync + 'static,
    >,
) -> PayTollServe {
    let base_api_url = setup_server_url();
    let config = config::Api {
        base_url: base_api_url,
        real_ip_header: None,
    };
    let stub_payment_service = StubPaymentService::new(result);
    PayTollServe::new(config, Box::new(stub_payment_service))
}

fn setup_server_url() -> url::Url {
    url::Url::parse("http://localhost:9000/").unwrap()
}

fn assert_has_content_length(headers: &http::response::Headers) -> usize {
    let content_length = headers.content_length();
    assert!(content_length.is_some(), "No Content-Length sent!");
    content_length.unwrap()
}

fn assert_body_contains_json(expected_content: serde_json::Value, mut response: http::Response) {
    let content_length = assert_has_content_length(response.headers());
    let mut json = vec![0u8; content_length];
    let body = response.body();
    match body {
        http::Body::Buffer(body) => {
            body.read_exact(&mut json).unwrap();
            let json = String::from_utf8(json).unwrap();
            let json: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(expected_content, json);
        }
        http::Body::Stream(_) => panic!("unexpected stream body"),
        http::Body::None => panic!("no body"),
    }
}

fn setup_payment_request(recipient: proxy::Recipient, order_id: proxy::OrderId) -> http::Request {
    let payment = json!({
        "toll": {
            "recipient": recipient,
            "order_id": order_id,
            "challenge": {},
            "signature": Base64::encode(b"very real; much secure;")
        },
        "value": "hello"
    });
    let json = payment.to_string();
    let data: VecDeque<u8> = json.into_bytes().into();
    let content_length = data.len();
    let body = http::Body::from_stream(Box::new(data), Some(content_length));
    let mut headers = http::Headers::empty();
    headers.insert("Host", "localhost");
    headers.insert("Content-Type", "application/json");
    headers.insert("Content-Length", content_length.to_string());
    let headers = http::request::Headers::new(headers).unwrap();
    http::Request::new(
        http::request::Method::Post,
        "/payment-endpoint",
        headers,
        body,
    )
    .unwrap()
}

#[test]
pub fn pay_toll_serve_should_return_visa_as_json() {
    // Arrange
    let create_visa = move || {
        let recipient = proxy::Recipient::new("1.2.3.4", "Bob", "example.com:80/");
        let order_id = proxy::OrderId::new("gate", "order");
        let expected_visa = payment::Visa::new(
            order_id.clone(),
            recipient.clone(),
            Base64::encode(b"real signature ;D"),
        );
        Ok(expected_visa)
    };
    let expected_visa = create_visa().unwrap();
    let sut = setup(Box::new(create_visa));
    let client_ip = SocketAddr::V4(SocketAddrV4::from_str("1.2.3.4:42420").unwrap());
    let request = setup_payment_request(
        expected_visa.recipient().clone(),
        expected_visa.order_id().clone(),
    );
    // Act
    let response = sut.serve_http(&client_ip, request).unwrap();
    // Assert
    assert_eq!(http::response::StatusCode::OK, response.status_code());
    assert_eq!(
        Some("application/hal+json"),
        response.headers().content_type()
    );
    let expected_body = json!({
        "token": "eyJpcCI6IjEuMi4zLjQiLCJ1YSI6IkJvYiIsImRlc3QiOiJleGFtcGxlLmNvbTo4MC8iLCJvcmRlcl9pZCI6ImdhdGUjb3JkZXIifQ==.cmVhbCBzaWduYXR1cmUgO0Q=",
        "header_name": "X-Keeper-Token",
        "_links": {
            "origin_url": "example.com:80/"
        }
    });
    assert_body_contains_json(expected_body, response);
}

#[test_case("<hello>World<hello>" ; "XML")]
#[test_case(r#"{"hello" = "world"}"# ; "malformed json")]
pub fn pay_toll_serve_should_return_400_for_non_json_data(non_json_data: &str) {
    // Arrange
    let payment_service_stub = || panic!("Malformed request got processed!");
    let sut = setup(Box::new(payment_service_stub));
    let client_ip = SocketAddr::V4(SocketAddrV4::from_str("1.2.3.4:42420").unwrap());
    let body = http::Body::from_string(non_json_data.into());
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/json");
    headers.insert("Content-Length", non_json_data.len().to_string());
    headers.insert("Host", "localhost");
    let headers = http::request::Headers::new(headers).unwrap();
    let request =
        http::Request::new(http::request::Method::Post, "/api/pay", headers, body).unwrap();
    // Act
    let response = sut.serve_http(&client_ip, request).unwrap();
    // Assert
    assert_eq!(
        http::response::StatusCode::BadRequest,
        response.status_code()
    );
}

#[test]
pub fn pay_toll_serve_should_return_400_for_invalid_json_data() {
    // Arrange
    let payment_service_stub = || panic!("Malformed request got processed!");
    let sut = setup(Box::new(payment_service_stub));
    let client_ip = SocketAddr::V4(SocketAddrV4::from_str("1.2.3.4:42420").unwrap());
    let invalid_json_data = json!({
        "toll": {},
        "value": {
            "challenge_part_1": "My",
            "challenge_part_2": "Answer"
        }
    });
    let invalid_json_data = invalid_json_data.to_string();
    let content_len = invalid_json_data.len();
    let body = http::Body::from_string(invalid_json_data);
    let mut headers = http::Headers::empty();
    headers.insert("Content-Type", "application/json");
    headers.insert("Content-Length", content_len.to_string());
    headers.insert("Host", "localhost");
    let headers = http::request::Headers::new(headers).unwrap();
    let request =
        http::Request::new(http::request::Method::Post, "/api/pay", headers, body).unwrap();
    // Act
    let response = sut.serve_http(&client_ip, request).unwrap();
    // Assert
    assert_eq!(
        http::response::StatusCode::UnprocessableContent,
        response.status_code()
    );
}

#[test]
pub fn pay_toll_serve_should_return_400_and_new_toll_for_failed_challenge() {
    // Arrange
    let create_challenge_failed = move || {
        let recipient = proxy::Recipient::new("1.2.3.4", "Bob", "example.com:80/");
        let order_id = proxy::OrderId::new("gate", "order");
        let toll = proxy::Toll::new(
            recipient,
            order_id,
            Challenge::empty(),
            Base64::encode(b"signature"),
        );
        let challenge_failed = PaymentError::ChallengeFailed(toll, "hello".into());
        Err(Box::new(challenge_failed))
    };
    let sut = setup(Box::new(create_challenge_failed));
    let client_ip = SocketAddr::V4(SocketAddrV4::from_str("1.2.3.4:42420").unwrap());
    let expected_err = match *create_challenge_failed().err().unwrap() {
        PaymentError::ChallengeFailed(toll, payment) => (toll, payment),
        _ => panic!("Huh?"),
    };
    let request = setup_payment_request(
        expected_err.0.recipient().clone(),
        expected_err.0.order_id().clone(),
    );
    // Act
    let response = sut.serve_http(&client_ip, request).unwrap();
    // Assert
    assert_eq!(
        http::response::StatusCode::BadRequest,
        response.status_code()
    );
    assert_eq!(
        Some("application/hal+json"),
        response.headers().content_type()
    );
    let expected_body = json!({
        "error": "Challenge failed!",
        "message": "'hello' was not the right answer! Try again with new toll",
        "failed_payment": expected_err.1,
        "new_toll": expected_err.0.as_hal_json(&setup_server_url()), //Link for paying toll already included in toll json :D
    });
    assert_body_contains_json(expected_body, response);
}

#[test]
pub fn pay_toll_serve_should_return_400_with_error_information_for_mismatched_recipients() {
    // Arrange
    let create_mismatched_recipient = move || {
        let expected_recipient = proxy::Recipient::new("1.2.3.4", "Bob", "example.com:80/");
        let actual_recipient = proxy::Recipient::new("5.6.7.8", "Alice", "example.com:80/");
        let order_id = proxy::OrderId::new("gate", "order");
        let toll = proxy::Toll::new(
            actual_recipient,
            order_id,
            Challenge::empty(),
            Base64::encode(b"signature"),
        );
        let mismatched_recipient = PaymentError::MismatchedRecipient(expected_recipient, toll);
        Err(Box::new(mismatched_recipient))
    };
    let expected_err = match *create_mismatched_recipient().err().unwrap() {
        PaymentError::MismatchedRecipient(expected_recipient, new_toll) => {
            (expected_recipient, new_toll)
        }
        _ => panic!("Huh?"),
    };
    let sut = setup(Box::new(create_mismatched_recipient));
    let client_ip = SocketAddr::V4(SocketAddrV4::from_str("1.2.3.4:42420").unwrap());
    let request = setup_payment_request(
        expected_err.1.recipient().clone(),
        expected_err.1.order_id().clone(),
    );
    // Act
    let response = sut.serve_http(&client_ip, request).unwrap();
    // Assert
    assert_eq!(
        http::response::StatusCode::BadRequest,
        response.status_code()
    );
    assert_eq!(
        Some("application/hal+json"),
        response.headers().content_type()
    );
    let expected_body = json!({
        "error": "Mismatched Recipient!",
        "message": "Toll was issued for a different recipient. New toll issued for current recipient",
        "expected_recipient": expected_err.0,
        "new_toll": expected_err.1.as_hal_json(&setup_server_url())
    });
    assert_body_contains_json(expected_body, response);
}

#[test]
pub fn pay_toll_serve_should_return_422_with_message_for_invalid_signature() {
    // Arrange
    let create_invalid_signature = move || Err(Box::new(PaymentError::InvalidSignature));
    let sut = setup(Box::new(create_invalid_signature));
    let client_ip = SocketAddr::V4(SocketAddrV4::from_str("1.2.3.4:42420").unwrap());
    let request = setup_payment_request(
        proxy::Recipient::new("1.2.3.4", "Bob", "example.com:80/"),
        proxy::OrderId::new("gate", "order"),
    );
    // Act
    let response = sut.serve_http(&client_ip, request).unwrap();
    // Assert
    assert_eq!(
        http::response::StatusCode::UnprocessableContent,
        response.status_code()
    );
    assert_eq!(
        Some("application/hal+json"),
        response.headers().content_type()
    );
    let expected_body = json!({
        "error": "Invalid Signature!",
        "message": "Issued toll signature is not valid! Content was probably modified or the key rotated",
    });
    assert_body_contains_json(expected_body, response);
}

#[test]
pub fn pay_toll_serve_should_return_409_with_message_for_no_longer_existing_order() {
    // Arrange
    let create_gateway_error = move || Err(Box::new(PaymentError::GatewayError));
    let sut = setup(Box::new(create_gateway_error));
    let client_ip = SocketAddr::V4(SocketAddrV4::from_str("1.2.3.4:42420").unwrap());
    let request = setup_payment_request(
        proxy::Recipient::new("1.2.3.4", "Bob", "example.com:80/"),
        proxy::OrderId::new("gate", "order"),
    );
    // Act
    let response = sut.serve_http(&client_ip, request).unwrap();
    // Assert
    assert_eq!(
        http::response::StatusCode::Conflict, //Most likely to occur because an order was removed while it still had pending tolls
        response.status_code()
    );
    assert_eq!(
        Some("application/hal+json"),
        response.headers().content_type()
    );
    let expected_body = json!({
        "error": "Gateway Error!",
        "message": "Toll no longer matches any order. Retry request"
    });
    assert_body_contains_json(expected_body, response);
}

struct StubPaymentService {
    pay_toll_result:
        Box<dyn Fn() -> Result<payment::Visa, Box<payment::PaymentError>> + Send + Sync + 'static>,
}

impl StubPaymentService {
    fn new(
        pay_toll_result: Box<
            dyn Fn() -> Result<payment::Visa, Box<payment::PaymentError>> + Send + Sync + 'static,
        >,
    ) -> Self {
        Self { pay_toll_result }
    }
}
impl PaymentService for StubPaymentService {
    fn pay_toll(
        &self,
        _: proxy::Recipient,
        _: payment::Payment,
    ) -> Result<payment::Visa, Box<payment::PaymentError>> {
        (*self.pay_toll_result)()
    }
}
