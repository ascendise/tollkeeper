use crate::http::{self, request::Headers};
use pretty_assertions::assert_eq;

#[test]
pub fn accept_encoding_header_should_return_encodings_sorted_by_highest_preference() {
    // Arrange
    let mut headers = http::Headers::empty();
    headers.insert("Host", "localhost");
    headers.insert("Accept-Encoding", "gzip;q=0.5, zstd;q=1.0, *;q=0");
    let sut = Headers::new(headers).unwrap();
    // Act
    let encodings = sut
        .accept_encoding()
        .expect("Accept-Encoding header missing!");
    // Assert
    let expected_encodings = vec!["zstd", "gzip", "*"];
    assert_eq!(expected_encodings, encodings);
}
