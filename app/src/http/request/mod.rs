#[cfg(test)]
mod tests;

mod parsing;

use std::io;
use std::{
    collections::HashMap,
    net::{self, TcpStream},
};

use super::*;

type Body = io::BufReader<net::TcpStream>;

pub struct Request {
    method: Method,
    request_target: String,
    http_version: String,
    headers: Headers,
    body: Option<Body>,
}
impl Request {
    pub fn new(
        method: Method,
        uri: impl Into<String>,
        http_version: impl Into<String>,
        headers: Headers,
    ) -> Self {
        Self {
            method,
            request_target: uri.into(),
            http_version: http_version.into(),
            headers,
            body: None,
        }
    }

    pub fn with_body(
        method: Method,
        uri: impl Into<String>,
        http_version: impl Into<String>,
        headers: Headers,
        body: Body,
    ) -> Self {
        Self {
            method,
            request_target: uri.into(),
            http_version: http_version.into(),
            headers,
            body: Some(body),
        }
    }

    /// HTTP Protocol version
    pub fn http_version(&self) -> &str {
        &self.http_version
    }

    /// Location of the resource. Can be relative or absolute
    pub fn uri(&self) -> &str {
        &self.request_target
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Request headers
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn body(&mut self) -> &mut Option<Body> {
        &mut self.body
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Headers {
    headers: HashMap<String, String>,
}
impl Headers {
    pub fn new(headers: HashMap<String, String>) -> Result<Self, BadRequestError> {
        if headers.contains_key("Host") {
            Ok(Self { headers })
        } else {
            Err(BadRequestError::NoHostHeader)
        }
    }

    pub fn accept(&self) -> Option<&String> {
        self.headers.get("Accept")
    }

    pub fn accept_charset(&self) -> Option<&String> {
        self.headers.get("Accept-Charset")
    }

    pub fn accept_encoding(&self) -> Option<&String> {
        self.headers.get("Accept-Encoding")
    }

    pub fn accept_language(&self) -> Option<&String> {
        self.headers.get("Accept-Language")
    }

    pub fn authorization(&self) -> Option<&String> {
        self.headers.get("Authorization")
    }

    pub fn expect(&self) -> Option<&String> {
        self.headers.get("Expect")
    }

    pub fn from(&self) -> Option<&String> {
        self.headers.get("From")
    }

    pub fn host(&self) -> &String {
        self.headers.get("Host").unwrap()
    }

    pub fn if_match(&self) -> Option<&String> {
        self.headers.get("If-Match")
    }

    pub fn if_modified_since(&self) -> Option<&String> {
        self.headers.get("If-Modified-Since")
    }

    pub fn if_none_match(&self) -> Option<&String> {
        self.headers.get("If-None-Match")
    }

    pub fn if_range(&self) -> Option<&String> {
        self.headers.get("If-Range")
    }

    pub fn if_unmodified_since(&self) -> Option<&String> {
        self.headers.get("If-Unmodified-Since")
    }

    pub fn max_forwards(&self) -> Option<&String> {
        self.headers.get("Max-Forwards")
    }

    pub fn proxy_authorization(&self) -> Option<&String> {
        self.headers.get("Proxy-Authorization")
    }

    pub fn range(&self) -> Option<&String> {
        self.headers.get("Range")
    }

    pub fn referrer(&self) -> Option<&String> {
        self.headers.get("Referrer")
    }

    pub fn te(&self) -> Option<&String> {
        self.headers.get("TE")
    }

    pub fn user_agent(&self) -> Option<&String> {
        self.headers.get("User-Agent")
    }

    pub fn content_length(&self) -> Option<&String> {
        self.headers.get("Content-Length")
    }

    pub fn extension(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }
}
impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for header in &self.headers {
            write!(f, "{}:{}\r\n", header.0, header.1)?
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BadRequestError {
    NoHostHeader,
}
