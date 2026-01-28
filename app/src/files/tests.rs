use pretty_assertions::assert_eq;
use std::{
    collections::VecDeque,
    io::{self, BufReader},
    net::SocketAddr,
    path::PathBuf,
    str::FromStr,
};

use indexmap::IndexMap;

use crate::{
    files::{FileReader, FileServe},
    http::{
        request::{self, Method},
        response::{self, StatusCode},
        server::HttpServe,
        Body, Headers, Request,
    },
};

#[test]
pub fn file_serve_should_return_requested_file() {
    // Arrange
    let content: VecDeque<u8> = String::from("Hello, World!").into_bytes().into();
    let file_reader = FakeFileReader::new(indexmap::indexmap![
        "/assets/file.txt".into() => content.clone()
    ]);
    let sut = FileServe::new(PathBuf::from("/assets/file.txt"), Box::new(file_reader));
    // Act
    let headers =
        request::Headers::new(Headers::new(vec![("Host".into(), "localhost".into())])).unwrap();
    let request = Request::new(Method::Get, "/assets/file.txt", headers, Body::None).unwrap();
    let mut response = sut
        .serve_http(&addr(), request)
        .expect("valid request failed");
    // Assert
    assert_eq!(StatusCode::OK, response.status_code());
    let expected_headers = Headers::new(vec![("Content-Length".into(), content.len().to_string())]);
    let expected_headers = response::Headers::new(expected_headers);
    assert_eq!(&expected_headers, response.headers());
    let body = match response.body() {
        Body::Buffer(buffer_body) => buffer_body,
        Body::Stream(_) => panic!(),
        Body::None => panic!("File was sent without body!"),
    };
    assert_eq!(&content, body.data());
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
