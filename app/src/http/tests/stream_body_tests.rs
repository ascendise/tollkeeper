use pretty_assertions::assert_eq;
use std::{
    collections::VecDeque,
    io::{self, BufReader},
};

use crate::http::{Chunk, ChunkedTcpStream, StreamBody};

#[test]
pub fn read_should_return_unexpected_eof_if_chunk_exceeds_size_limit() {
    // Arrange
    let oversized_chunk_data: String = vec!['a'; Chunk::MAX_CHUNK_SIZE + 1].iter().collect();
    let stream = format!(
        "5\r\nHello\r\n{:x}\r\n{}\r\n0\r\n\r\n",
        oversized_chunk_data.len(),
        oversized_chunk_data
    );
    let stream: VecDeque<u8> = stream.into_bytes().into();
    let stream = BufReader::new(stream);
    let stream = ChunkedTcpStream::new(Box::new(stream));
    let mut sut = StreamBody::new(Box::new(stream));
    // Act
    let mut buf = Vec::new();
    let res = io::copy(&mut sut, &mut buf).expect_err("wrote too big chunk to target");
    // Assert
    assert_eq!(io::ErrorKind::QuotaExceeded, res.kind())
}
