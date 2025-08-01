use crate::http::{
    self, request,
    response::StatusCode,
    server::{tests::HelloHandler, *},
};

fn setup(endpoints: Vec<Endpoint>) -> HttpEndpointsServe {
    HttpEndpointsServe::new(endpoints)
}

const fn client_addr() -> net::SocketAddr {
    let v4_addr = net::Ipv4Addr::new(127, 0, 0, 1);
    let v4_addr = net::SocketAddrV4::new(v4_addr, 5501);
    net::SocketAddr::V4(v4_addr)
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
    let request = Request::new(Method::Get, "/hello", headers).unwrap();
    let response = sut.serve_http(&client_addr(), request);
    // Assert
    assert!(response.is_ok());
    let mut response = response.unwrap();
    assert_eq!(StatusCode::OK, response.status_code());
    assert_eq!(Some(8), response.headers().content_length());
    let mut body = String::new();
    response.body().unwrap().read_to_string(&mut body).unwrap();
    assert_eq!("Hello!\r\n", body);
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
    let request = Request::new(Method::Get, "/hello", headers).unwrap();
    let mut response = sut.serve_http(&client_addr(), request).unwrap();
    // Assert
    let mut body = String::new();
    response.body().unwrap().read_to_string(&mut body).unwrap();
    assert_eq!("Hello!\r\n", body);
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
    let request = Request::new(Method::Get, "/this-path-is-unknown", headers).unwrap();
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
    let request = Request::new(Method::Head, "/hello", headers).unwrap();
    let response = sut.serve_http(&client_addr(), request).unwrap();
    // Assert
    assert_eq!(StatusCode::MethodNotAllowed, response.status_code());
}
