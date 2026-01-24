use std::{io::Read, str::FromStr};

use crate::http::{
    self, request,
    response::StatusCode,
    server::{
        tests::{HelloHandler, IpSpyHandler},
        *,
    },
};
use pretty_assertions::assert_eq;
use test_case::test_case;

fn setup(endpoints: Vec<Endpoint>) -> HttpEndpointsServe {
    HttpEndpointsServe::new(endpoints, None)
}

const fn client_addr() -> net::SocketAddr {
    let v4_addr = net::Ipv4Addr::new(127, 0, 0, 1);
    let v4_addr = net::SocketAddrV4::new(v4_addr, 5501);
    net::SocketAddr::V4(v4_addr)
}

pub fn assert_body_contains(expected_content: &str, body: &mut http::Body) {
    match body {
        http::Body::Buffer(buffer_body) => {
            let mut body = String::new();
            buffer_body.read_to_string(&mut body).unwrap();
            assert_eq!(expected_content, body);
        }
        http::Body::Stream(_) => panic!("stream body not expected"),
        http::Body::None => panic!("no body"),
    }
}

#[test]
pub fn serve_should_handle_request_through_defined_endpoint() {
    // Arrange
    let handler = Box::new(HelloHandler {
        body: "Hello!\r\n".into(),
    });
    let endpoints = vec![Endpoint::new(Method::Get, "/hello", handler)];
    let sut = setup(endpoints);
    // Act
    let mut headers = http::Headers::empty();
    headers.insert("Host", "localhost");
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(Method::Get, "/hello", headers, http::Body::None).unwrap();
    let response = sut.serve_http(&client_addr(), request);
    // Assert
    assert!(response.is_ok());
    let mut response = response.unwrap();
    assert_eq!(StatusCode::OK, response.status_code());
    assert_eq!(Some(8), response.headers().content_length());
    assert_body_contains("Hello!\r\n", response.body());
}

#[test]
pub fn serve_should_handle_request_on_specific_path() {
    // Arrange
    let hello_handler = Box::new(HelloHandler {
        body: "Hello!\r\n".into(),
    });
    let good_morning_handler = Box::new(HelloHandler {
        body: "Good Morning!\r\n".into(),
    });
    let endpoints = vec![
        Endpoint::new(Method::Get, "/hello", hello_handler),
        Endpoint::new(Method::Get, "/good-morning", good_morning_handler),
    ];
    let sut = setup(endpoints);
    // Act
    let mut headers = http::Headers::empty();
    headers.insert("Host", "localhost");
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(Method::Get, "/hello", headers, http::Body::None).unwrap();
    let mut response = sut.serve_http(&client_addr(), request).unwrap();
    // Assert
    assert_body_contains("Hello!\r\n", response.body());
}

#[test_case("/hello/", "/hello" ; "missing trailing slash")]
#[test_case("/hello", "/hello/" ; "additional trailing slash")]
#[test_case("/hello", "/hello/////" ; "more trailing slashes")]
pub fn serve_should_handle_missing_or_added_trailing_slashes(
    endpoint_path: &str,
    access_path: &str,
) {
    // Arrange
    let hello_handler = Box::new(HelloHandler {
        body: "Hello!\r\n".into(),
    });
    let endpoints = vec![Endpoint::new(Method::Get, endpoint_path, hello_handler)];
    let sut = setup(endpoints);
    // Act
    let mut headers = http::Headers::empty();
    headers.insert("Host", "localhost");
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(Method::Get, access_path, headers, http::Body::None).unwrap();
    let response = sut.serve_http(&client_addr(), request).unwrap();
    // Assert
    assert_eq!(StatusCode::OK, response.status_code(), "did not find path");
}

#[test]
pub fn serve_should_return_not_found_when_no_matching_endpoint_path_is_found() {
    // Arrange
    let hello_handler = Box::new(HelloHandler {
        body: "Hello!\r\n".into(),
    });
    let endpoints = vec![Endpoint::new(Method::Get, "/hello", hello_handler)];
    let sut = setup(endpoints);
    // Act
    let mut headers = http::Headers::empty();
    headers.insert("Host", "localhost");
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(
        Method::Get,
        "/this-path-is-unknown",
        headers,
        http::Body::None,
    )
    .unwrap();
    let response = sut.serve_http(&client_addr(), request).unwrap();
    // Assert
    assert_eq!(StatusCode::NotFound, response.status_code());
}

#[test]
pub fn serve_should_return_method_not_allowed_when_matching_endpoint_is_wrong_path() {
    // Arrange
    let hello_handler = Box::new(HelloHandler {
        body: "Hello!\r\n".into(),
    });
    let endpoints = vec![Endpoint::new(Method::Get, "/hello", hello_handler)];
    let sut = setup(endpoints);
    // Act
    let mut headers = http::Headers::empty();
    headers.insert("Host", "localhost");
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(Method::Head, "/hello", headers, http::Body::None).unwrap();
    let response = sut.serve_http(&client_addr(), request).unwrap();
    // Assert
    assert_eq!(StatusCode::MethodNotAllowed, response.status_code());
}

#[test]
pub fn serve_should_pass_client_address_to_inner_serve_if_no_header_configured() {
    // Arrange
    let ip_spy_handler = IpSpyHandler::default();
    let endpoints = vec![Endpoint::new(
        Method::Get,
        "/",
        Box::new(ip_spy_handler.clone()),
    )];
    let sut = setup(endpoints);
    // Act
    let mut headers = http::Headers::empty();
    headers.insert("Host", "localhost");
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(Method::Get, "/", headers, http::Body::None).unwrap();
    let client_addr = client_addr();
    let _ = sut.serve_http(&client_addr, request).unwrap();
    // Assert
    let expected_ip = &[client_addr];
    ip_spy_handler.assert_ips_equal(expected_ip);
}

#[test_case("X-Real-Ip", "12.34.56.78")]
#[test_case("My-Ip", "87.65.43.21")]
pub fn serve_should_pass_peer_address_from_header_to_inner_serve_if_configured(
    header_name: &str,
    expected_ip: &str,
) {
    // Arrange
    let ip_spy_handler = IpSpyHandler::default();
    let endpoints = vec![Endpoint::new(
        Method::Get,
        "/",
        Box::new(ip_spy_handler.clone()),
    )];
    let sut = HttpEndpointsServe::new(endpoints, Some(header_name.to_string()));
    // Act
    let mut headers = http::Headers::empty();
    headers.insert("Host", "localhost");
    headers.insert(header_name, expected_ip);
    let headers = request::Headers::new(headers).unwrap();
    let request = Request::new(Method::Get, "/", headers, http::Body::None).unwrap();
    let client_addr = client_addr();
    let _ = sut.serve_http(&client_addr, request).unwrap();
    // Assert
    let expected_ip = net::SocketAddr::from_str(&format!("{expected_ip}:0")).unwrap();
    let expected_ip = &[expected_ip];
    ip_spy_handler.assert_ips_equal(expected_ip);
}
