use indexmap::IndexMap;
use std::collections::VecDeque;
use std::io::{BufRead, Read};
use std::{fmt::Display, io, str::FromStr};

#[cfg(test)]
mod tests;

mod parsing;
pub mod request;
pub mod response;
pub mod server;
pub use request::Request;
pub use response::Response;

/// Key-Value collection with case-insensitve access
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Headers {
    headers: IndexMap<String, Header>,
}
impl Headers {
    pub fn new(headers: IndexMap<String, String>) -> Self {
        let headers = Self::map_headers_case_insensitive(headers);
        Self { headers }
    }

    fn map_headers_case_insensitive(headers: IndexMap<String, String>) -> IndexMap<String, Header> {
        headers
            .iter()
            .map(|(k, v)| {
                (
                    k.to_ascii_lowercase(),
                    Header {
                        original_key: k.into(),
                        value: v.into(),
                    },
                )
            })
            .collect()
    }

    pub fn empty() -> Self {
        Self {
            headers: IndexMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        let key = key.to_ascii_lowercase();
        match self.headers.get(&key) {
            Some(v) => Some(&v.value),
            None => None,
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let original_key = key.into();
        let key = &original_key.to_ascii_lowercase();
        let value = value.into();
        let header = Header {
            original_key,
            value,
        };
        self.headers.insert(key.into(), header);
    }
}
impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for header in &self.headers {
            write!(f, "{}: {}\r\n", header.1.original_key, header.1.value)?
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct Header {
    original_key: String,
    value: String,
}

pub trait Parse<T>: Sized {
    type Err;
    fn parse(stream: T) -> Result<Self, Self::Err>;
}

pub enum Body {
    Buffer(BufferBody),
    Stream(StreamBody),
    None,
}
impl Body {
    pub fn from_stream(mut stream: Box<dyn io::Read>, content_length: Option<usize>) -> Self {
        match content_length {
            Some(len) => {
                let mut buffer = vec![0; len];
                stream.read_exact(&mut buffer).unwrap();
                let body = BufferBody::new(buffer.into());
                Body::Buffer(body)
            }
            None => {
                let body = StreamBody::new(stream);
                Body::Stream(body)
            }
        }
    }

    pub fn from_string(data: String) -> Self {
        let data = data.into_bytes().into();
        Self::Buffer(BufferBody::new(data))
    }

    pub fn has_body(&self) -> bool {
        match self {
            Body::Buffer(_) => true,
            Body::Stream(_) => true,
            Body::None => true,
        }
    }
}

/// HTTP Body with fixed length
pub struct BufferBody {
    data: VecDeque<u8>,
}
impl BufferBody {
    pub fn new(data: VecDeque<u8>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &VecDeque<u8> {
        &self.data
    }
}
impl io::Read for BufferBody {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.data.read(buf)
    }
}

/// HTTP Body with data stream
pub struct StreamBody {
    stream: io::BufReader<Box<dyn io::Read>>,
}
impl StreamBody {
    pub fn new(stream: Box<dyn io::Read>) -> Self {
        Self {
            stream: io::BufReader::new(stream),
        }
    }

    /// Reads the next chunk from the stream
    /// EOF will be signalled by a Chunk with zero size ([Chunk::is_eof]) which should also be
    /// written to output stream.
    /// Note: Further reads after EOF will most likely result in blocking
    pub fn read_chunk(&mut self) -> Option<Chunk> {
        let chunk_size = self.read_chunk_size()?;
        if chunk_size == 0 {
            self.stream.consume(2); //Empty Content
            Some(Chunk::eof())
        } else {
            let content = self.read_chunk_content(chunk_size)?;
            let chunk = Chunk::new(chunk_size, content);
            Some(chunk)
        }
    }

    fn read_chunk_size(&mut self) -> Option<usize> {
        let mut chunk_size = String::new();
        let _ = self.stream.by_ref().read_line(&mut chunk_size).ok()?;
        let chunk_size = chunk_size.trim();
        if chunk_size.is_empty() {
            None
        } else {
            let chunk_size = usize::from_str_radix(chunk_size, 16).unwrap();
            Some(chunk_size)
        }
    }

    fn read_chunk_content(&mut self, chunk_size: usize) -> Option<Vec<u8>> {
        let mut content: Vec<u8> = format!("{chunk_size:x}\r\n").into_bytes();
        let mut content_buff = vec![0; chunk_size];
        self.stream.read_exact(&mut content_buff).ok()?;
        content.append(&mut content_buff);
        content.append(&mut vec![b'\r', b'\n']);
        self.stream.consume(2); // Remove CRLF from stream
        Some(content)
    }
}
pub struct Chunk {
    size: usize,
    content: Vec<u8>,
}
impl Chunk {
    pub fn new(size: usize, content: Vec<u8>) -> Self {
        Self { size, content }
    }

    pub fn eof() -> Self {
        Self {
            size: 0,
            content: b"0\r\n\r\n".into(),
        }
    }

    pub fn is_eof(&self) -> bool {
        self.size == 0
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
