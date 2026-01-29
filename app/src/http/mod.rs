use indexmap::IndexMap;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};
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
    headers: IndexMap<String, Vec<Header>>,
}
impl Headers {
    pub fn new(headers: Vec<(String, String)>) -> Self {
        let headers = Self::map_headers_case_insensitive(headers);
        Self { headers }
    }

    fn map_headers_case_insensitive(
        headers: Vec<(String, String)>,
    ) -> IndexMap<String, Vec<Header>> {
        let mut new_headers: IndexMap<String, Vec<Header>> = indexmap::indexmap! {};
        for (key, value) in headers {
            let header = Header {
                original_key: key.clone(),
                value,
            };
            let key = key.to_ascii_lowercase();
            if !new_headers.contains_key(&key) {
                new_headers.insert(key.clone(), vec![]);
            }
            let bucket = new_headers.get_mut(&key).unwrap();
            bucket.push(header);
        }
        new_headers
    }

    pub fn empty() -> Self {
        Self {
            headers: IndexMap::new(),
        }
    }

    /// Gets the first header found with the same key
    pub fn get(&self, key: &str) -> Option<&str> {
        let key = key.to_ascii_lowercase();
        match self.headers.get(&key) {
            Some(v) => Some(&v.first().unwrap().value),
            None => None,
        }
    }

    /// Returns all headers with the given key.
    ///
    /// As most headers are unique, this only applies to exceptions like `Set-Cookie`
    pub fn get_all(&self, key: &str) -> Option<Vec<&str>> {
        let key = key.to_ascii_lowercase();
        match self.headers.get(&key) {
            Some(v) => {
                let values = v.iter().map(|v| v.value.as_ref()).collect();
                Some(values)
            }
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
        if !self.headers.contains_key(key) {
            self.headers.insert(key.clone(), vec![]);
        }
        let bucket = self.headers.get_mut(key).unwrap();
        bucket.push(header);
    }
}
impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for header_bucket in &self.headers {
            for header in header_bucket.1 {
                write!(f, "{}: {}\r\n", header.original_key, header.value)?
            }
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
                let stream = ChunkedTcpStream::new(Box::new(BufReader::new(stream)));
                let body = StreamBody::new(Box::new(stream));
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
            Body::None => false,
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
    stream: Box<dyn ChunkedStream>,
    current_chunk: Option<BufReader<VecDeque<u8>>>,
    bytes_read: usize,
    next_chunk: usize,
}
impl StreamBody {
    pub fn new(stream: Box<dyn ChunkedStream>) -> Self {
        Self {
            stream,
            current_chunk: None,
            bytes_read: 0,
            next_chunk: 0,
        }
    }
}
impl Read for StreamBody {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.current_chunk.is_none() {
            let chunk = match self.stream.next_chunk() {
                Some(c) => c,
                None => return Ok(0),
            };
            let chunk = chunk.into_bytes();
            let chunk_len = chunk.len();
            self.current_chunk = Some(BufReader::new(chunk.into()));
            self.next_chunk += chunk_len;
        }
        let reader = self.current_chunk.as_mut().unwrap();
        let res = reader.read(buf)?;
        self.bytes_read += res;
        if self.bytes_read >= self.next_chunk {
            self.current_chunk = None;
        }
        Ok(res)
    }
}
#[derive(Debug, PartialEq, Eq)]
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
            content: vec![],
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

    pub fn into_bytes(mut self) -> Vec<u8> {
        let mut data = Vec::new();
        data.append(&mut format!("{:x}\r\n", self.size).into_bytes());
        data.append(&mut self.content);
        data.append(&mut vec![b'\r', b'\n']);
        data
    }
}
pub trait ChunkedStream {
    fn next_chunk(&mut self) -> Option<Chunk>;
}

pub struct ChunkedTcpStream {
    stream: Box<io::BufReader<dyn io::Read>>,
}
impl ChunkedTcpStream {
    pub fn new(stream: Box<io::BufReader<dyn io::Read>>) -> Self {
        Self { stream }
    }

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
        if chunk_size.trim().is_empty() {
            chunk_size.clear();
            let _ = self.stream.by_ref().read_line(&mut chunk_size).ok()?;
        }
        let chunk_size = chunk_size.trim();
        if chunk_size.is_empty() {
            None
        } else {
            let chunk_size = usize::from_str_radix(chunk_size, 16).unwrap();
            Some(chunk_size)
        }
    }

    fn read_chunk_content(&mut self, chunk_size: usize) -> Option<Vec<u8>> {
        let mut content = vec![0; chunk_size];
        self.stream.read_exact(&mut content).ok()?;
        self.stream.consume(2); // Remove CRLF from stream
        Some(content)
    }
}
impl ChunkedStream for ChunkedTcpStream {
    fn next_chunk(&mut self) -> Option<Chunk> {
        self.read_chunk()
    }
}
