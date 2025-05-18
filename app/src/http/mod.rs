use std::{io::Read, str::FromStr};

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
impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
impl Read for BodyStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(buf)
    }
}
