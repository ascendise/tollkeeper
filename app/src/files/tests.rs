use pretty_assertions::assert_eq;
use std::{
    collections::VecDeque,
    io::{self, BufReader, Read},
    net::SocketAddr,
    path::PathBuf,
    str::FromStr,
};

use indexmap::IndexMap;

use crate::{
    files::{FileReader, FileServe},
    http::{
        self,
        request::{self, Method},
        response::{self, StatusCode},
        server::HttpServe,
        Body, Headers, Request,
    },
};
use test_case::test_case;

#[test_case("file.txt", "text/plain")]
#[test_case("file.js", "text/javascript")]
#[test_case("file.html", "text/html")]
#[test_case("file.css", "text/css")]
#[test_case("file.senta", "text/plain" ; "unknown file type")]
pub fn file_serve_should_return_requested_file(file_name: &str, expected_content_type: &str) {
    // Arrange
    let content: VecDeque<u8> = String::from("Hello, World!").into_bytes().into();
    let server_file = format!("/assets/{file_name}");
    let file_reader = FakeFileReader::new(indexmap::indexmap![
        server_file.clone() => content.clone()
    ]);
    let sut = FileServe::new(PathBuf::from(server_file.clone()), Box::new(file_reader));
    // Act
    let headers =
        request::Headers::new(Headers::new(vec![("Host".into(), "localhost".into())])).unwrap();
    let request = Request::new(Method::Get, server_file, headers, Body::None).unwrap();
    let mut response = sut
        .serve_http(&addr(), request)
        .expect("valid request failed");
    // Assert
    assert_eq!(StatusCode::OK, response.status_code());
    let expected_headers = Headers::new(vec![
        ("Transfer-Encoding".into(), "chunked".into()),
        ("Content-Type".into(), expected_content_type.into()),
    ]);
    let expected_headers =
        response::Headers::with_cors(expected_headers, Some(&[http::Method::Get]));
    assert_eq!(&expected_headers, response.headers());
    let body = match response.body() {
        Body::Buffer(_) => panic!("Expected chunked response!"),
        Body::Stream(b) => b,
        Body::None => panic!("File was sent without body!"),
    };
    let mut actual_body = String::new();
    body.read_to_string(&mut actual_body).unwrap();
    let expected_body = "d\r\nHello, World!\r\n0\r\n\r\n";
    assert_eq!(expected_body, actual_body);
}

#[test]
pub fn file_serve_should_return_404_if_file_does_not_exist() {
    // Arrange
    let content: VecDeque<u8> = String::from("Hello, World!").into_bytes().into();
    let file_reader = FakeFileReader::new(indexmap::indexmap![
        "/assets/file.txt".into() => content.clone()
    ]);
    let sut = FileServe::new(PathBuf::from("/assets/file.txt"), Box::new(file_reader));
    // Act
    let headers =
        request::Headers::new(Headers::new(vec![("Host".into(), "localhost".into())])).unwrap();
    let request = Request::new(Method::Get, "/etc/passwd", headers, Body::None).unwrap();
    let mut response = sut
        .serve_http(&addr(), request)
        .expect("valid request failed");
    // Assert
    assert_eq!(StatusCode::NotFound, response.status_code());
    let expected_headers = Headers::empty();
    let expected_headers = response::Headers::new(expected_headers);
    assert_eq!(&expected_headers, response.headers());
    assert!(!response.body().has_body());
}

fn addr() -> SocketAddr {
    SocketAddr::from_str("192.168.1.2:1234").unwrap()
}

struct FakeFileReader {
    files: IndexMap<String, VecDeque<u8>>,
}
impl FakeFileReader {
    fn new(files: IndexMap<String, VecDeque<u8>>) -> Self {
        Self { files }
    }
}
impl FileReader for FakeFileReader {
    fn read(&self, path: &std::path::Path) -> std::io::Result<Box<dyn std::io::Read>> {
        let no_file_found = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let file = self
            .files
            .get(path.to_str().unwrap())
            .ok_or(no_file_found)?
            .clone();
        let reader = BufReader::new(file);
        Ok(Box::new(reader) as Box<dyn std::io::Read>)
    }
}
