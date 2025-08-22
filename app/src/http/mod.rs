use indexmap::IndexMap;
use std::io::Read;
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
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
struct Header {
    original_key: String,
    value: String,
}

pub trait Parse<T>: Sized {
    type Err;
    fn parse(stream: T) -> Result<Self, Self::Err>;
}

/// Body of an HTTP Message
pub trait Body {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error>;
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, io::Error>;
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()>;
}

pub struct StreamBody<T: Read> {
    stream: io::BufReader<T>,
}
impl<T: Read> StreamBody<T> {
    pub fn new(stream: T) -> Self {
        let stream = io::BufReader::new(stream);
        Self { stream }
    }
}
impl<T: Read> Body for StreamBody<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.stream.read(buf)
    }
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, io::Error> {
        self.stream.read_to_string(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.stream.read_exact(buf)
    }
}
