use std::{fmt::Display, io::Read, str::FromStr};

mod request;

#[derive(Debug, PartialEq, Eq)]
pub enum Method {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect,
    Extension(String),
}
impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let method = match self {
            Method::Options => "OPTIONS",
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Trace => "TRACE",
            Method::Connect => "CONNECT",
            Method::Extension(v) => v,
        };
        write!(f, "{method}")
    }
}
impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(());
        }
        let method = match s {
            "OPTIONS" => Method::Options,
            "GET" => Method::Get,
            "HEAD" => Method::Head,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            "TRACE" => Method::Trace,
            "CONNECT" => Method::Connect,
            _ => Method::Extension(s.into()),
        };
        Ok(method)
    }
}

pub struct BodyStream {
    cursor: std::io::Cursor<Vec<u8>>,
}

impl BodyStream {
    pub fn new(cursor: std::io::Cursor<Vec<u8>>) -> Self {
        Self { cursor }
    }
}
impl Read for BodyStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(buf)
    }
}
