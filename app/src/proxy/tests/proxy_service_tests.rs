use std::{io::Write, net, sync::Arc, thread};

use tollkeeper::{
    declarations::{self},
    descriptions::{self},
    signatures::{Base64, InMemorySecretKeyProvider},
};

use crate::{
    http::{
        self,
        request::{self, Method},
        response::StatusCode,
        Request,
    },
    proxy::{Challenge, OrderId, ProxyService, ProxyServiceImpl, Recipient, Toll},
};

fn setup_and_get_id(
    requires_challenge: bool,
    proxy_addr: net::SocketAddr,
) -> (OrderId, ProxyServiceImpl) {
    let destination =
        descriptions::Destination::new(proxy_addr.ip().to_string(), proxy_addr.port(), "/");
    let description = StubDescription {
        is_match: requires_challenge,
    };
    let orders = vec![tollkeeper::Order::new(
        vec![Box::new(description)],
        tollkeeper::AccessPolicy::Blacklist,
        Box::new(StubTollDeclaration),
    )];
    let order_id = orders[0].id().to_string();
    let gates = vec![tollkeeper::Gate::new(destination, orders).unwrap()];
    let gate_id = gates[0].id().to_string();
    let secret_key_provider = InMemorySecretKeyProvider::new("Secret key".into());
    let secret_key_provider = Box::new(secret_key_provider);
    let tollkeeper = tollkeeper::Tollkeeper::new(gates, secret_key_provider).unwrap();
    let order_id = OrderId { gate_id, order_id };
    (order_id, ProxyServiceImpl::new(Arc::new(tollkeeper)))
}

fn setup(requires_challenge: bool, proxy_addr: net::SocketAddr) -> ProxyServiceImpl {
    let (_, sut) = setup_and_get_id(requires_challenge, proxy_addr);
    sut
}

fn setup_proxy(response: Vec<u8>) -> (thread::JoinHandle<()>, net::SocketAddr) {
    let listener = net::TcpListener::bind("127.0.0.1:0").unwrap();
    let local_addr = listener.local_addr().unwrap();
    let thread = thread::spawn(move || {
        let (mut conn, _) = listener.accept().unwrap();
        conn.write_all(&response).unwrap();
    });
    (thread, local_addr)
}

const fn client_addr() -> net::SocketAddr {
    let v4_addr = net::Ipv4Addr::new(127, 0, 0, 1);
    let v4_addr = net::SocketAddrV4::new(v4_addr, 5501);
    net::SocketAddr::V4(v4_addr)
}

#[test]
pub fn proxy_request_should_send_request_to_target_and_return_response() {
    // Arrange
    let (proxy, proxy_addr) = setup_proxy("HTTP/1.1 200 OK\r\n\r\n".into());
    let sut = setup(false, proxy_addr);
    // Act
    let mut headers = http::Headers::empty();
    headers.insert("Host", format!("127.0.0.1:{}", proxy_addr.port()));
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(Method::Get, "/", headers).unwrap();
    let response = sut
        .proxy_request(&client_addr(), request)
        .expect("Expected response, got denied");
    proxy.join().unwrap();
    // Assert
    assert_eq!(StatusCode::OK, response.status_code());
}

#[test]
pub fn proxy_request_should_return_error_when_payment_is_required() {
    // Arrange
    let mut target_addr = client_addr();
    target_addr.set_port(80);
    let (order_id, sut) = setup_and_get_id(true, target_addr);
    // Act
    let client_addr = client_addr();
    let mut headers = http::Headers::empty();
    headers.insert("Host", "127.0.0.1");
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(Method::Get, "/", headers).unwrap();
    let proxy_result = sut.proxy_request(&client_addr, request);
    // Assert
    assert!(
        proxy_result.is_err(),
        "Expected a PaymentRequiredError, but was proxied successfully!"
    );
    let payment_required_error = proxy_result.err().unwrap();
    let toll = payment_required_error.0;
    let expected_toll = Toll {
        recipient: Recipient {
            client_ip: client_addr.ip().to_string(),
            user_agent: "".into(),
            destination: "127.0.0.1:80/".to_string(),
        },
        order_id,
        challenge: Challenge::empty(),
        signature: toll.signature.clone(),
    };
    assert_eq!(expected_toll, *toll);
}

#[test]
pub fn proxy_request_should_return_404_response_when_trying_to_access_unknown_target() {
    // Arrange
    let mut target_addr = client_addr();
    target_addr.set_port(2200);
    //port
    let sut = setup(false, target_addr);
    let client_addr = client_addr();
    let mut headers = http::Headers::empty();
    let host = "127.0.0.1:3333";
    headers.insert("Host", host);
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(Method::Get, "/", headers).unwrap();
    let proxy_result = sut.proxy_request(&client_addr, request);
    // Assert
    assert!(
        proxy_result.is_ok(),
        "Expected a Response, but got an error!"
    );
    let response = proxy_result.unwrap();
    assert_eq!(StatusCode::NotFound, response.status_code())
}
#[test]
pub fn proxy_request_should_send_request_to_target_if_positive_suspect_has_visa() {
    // Arrange
    let (proxy, proxy_addr) = setup_proxy("HTTP/1.1 200 OK\r\n\r\n".into());
    let (order_id, sut) = setup_and_get_id(true, proxy_addr);
    // Act
    let mut headers = http::Headers::empty();
    let host = format!("127.0.0.1:{}", proxy_addr.port());
    headers.insert("Host", host.clone());
    let visa = serde_json::json!({
        "ip": "127.0.0.1",
        "ua": "Yo Mama",
        "dest": format!("{host}/"),
        "order_id": order_id
    })
    .to_string();
    let signature = tollkeeper::declarations::Visa::new(
        tollkeeper::declarations::OrderIdentifier::new(order_id.gate_id, order_id.order_id),
        tollkeeper::descriptions::Suspect::new(
            "127.0.0.1",
            "Yo Mama",
            tollkeeper::descriptions::Destination::new("127.0.0.1", proxy_addr.port(), "/"),
        ),
    );
    let signature = tollkeeper::signatures::Signed::sign(signature, b"Secret key");
    let visa = Base64::encode(visa.as_bytes());
    let signature = signature.signature().base64();
    let token = format!("{}.{}", visa, signature);
    headers.insert("X-Keeper-Token", token);
    headers.insert("User-Agent", "Yo Mama");
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(Method::Get, "/", headers).unwrap();
    let response = sut
        .proxy_request(&client_addr(), request)
        .expect("Expected response, got denied");
    proxy.join().unwrap();
    // Assert
    assert_eq!(StatusCode::OK, response.status_code());
}

struct StubDescription {
    is_match: bool,
}
impl tollkeeper::Description for StubDescription {
    fn matches(&self, _: &descriptions::Suspect) -> bool {
        self.is_match
    }
}

struct StubTollDeclaration;
impl tollkeeper::Declaration for StubTollDeclaration {
    fn declare(
        &self,
        suspect: descriptions::Suspect,
        order_id: declarations::OrderIdentifier,
    ) -> declarations::Toll {
        tollkeeper::declarations::Toll::new(
            suspect,
            order_id,
            tollkeeper::declarations::Challenge::new(),
        )
    }

    fn pay(
        &self,
        _: declarations::Payment,
        _: &descriptions::Suspect,
    ) -> Result<declarations::Visa, tollkeeper::declarations::PaymentError> {
        todo!()
    }
}
