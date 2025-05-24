#[cfg(test)]
mod tests;

mod parsing;

use std::io;
use std::{
    collections::HashMap,
    net::{self},
};

use super::*;

type Body = io::BufReader<net::TcpStream>;

pub struct Request {
    method: Method,
    request_target: String,
    absolute_target: url::Url,
    http_version: String,
    headers: Headers,
    body: Option<Body>,
}
#[allow(dead_code)] // I need the getters, just not now
impl Request {
    fn create(
        method: Method,
        request_target: String,
        http_version: String,
        headers: Headers,
        body: Option<Body>,
    ) -> Result<Self, BadRequestError> {
        let absolute_target = match url::Url::parse(&request_target) {
            Ok(v) => v,
            Err(_) => {
                let protocol = String::from("http://");
                let base_url = protocol + headers.host();
                let mut url =
                    url::Url::parse(&base_url).map_err(BadRequestError::FailedTargetParse)?;
                url.set_path(&request_target);
                url
            }
        };
        let request = Self {
            method,
            request_target,
            absolute_target,
            http_version,
            headers,
            body,
        };
        Ok(request)
    }

    pub fn new(
        method: Method,
        request_target: impl Into<String>,
        http_version: impl Into<String>,
        headers: Headers,
    ) -> Result<Self, BadRequestError> {
        Self::create(
            method,
            request_target.into(),
            http_version.into(),
            headers,
            None,
        )
    }

    pub fn with_body(
        method: Method,
        request_target: impl Into<String>,
        http_version: impl Into<String>,
        headers: Headers,
        body: Body,
    ) -> Result<Self, BadRequestError> {
        Self::create(
            method,
            request_target.into(),
            http_version.into(),
            headers,
            Some(body),
        )
    }

    /// HTTP Protocol version
    pub fn http_version(&self) -> &str {
        &self.http_version
    }

    /// Location of the resource. Can be relative or absolute
    pub fn request_target(&self) -> &str {
        &self.request_target
    }

    pub fn absolute_target(&self) -> &url::Url {
        &self.absolute_target
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
    headers: HashMap<String, Header>,
}
#[allow(dead_code)] // Dont piss me off about the getters for the headers..
impl Headers {
    pub fn new(headers: HashMap<String, String>) -> Result<Self, BadRequestError> {
        if headers.contains_key("Host") {
            let headers = Self::map_headers_case_insensitive(headers);
            Ok(Self { headers })
        } else {
            Err(BadRequestError::NoHostHeader)
        }
    }

    fn map_headers_case_insensitive(headers: HashMap<String, String>) -> HashMap<String, Header> {
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

    pub fn accept(&self) -> Option<&String> {
        self.read_header("accept")
    }

    fn read_header(&self, key: &str) -> Option<&String> {
        match self.headers.get(key) {
            Some(v) => Some(&v.value),
            None => None,
        }
    }

    pub fn accept_charset(&self) -> Option<&String> {
        self.read_header("accept-charset")
    }

    pub fn accept_encoding(&self) -> Option<&String> {
        self.read_header("accept-encoding")
    }

    pub fn accept_language(&self) -> Option<&String> {
        self.read_header("accept-language")
    }

    pub fn authorization(&self) -> Option<&String> {
        self.read_header("authorization")
    }

    pub fn expect(&self) -> Option<&String> {
        self.read_header("expect")
    }

    pub fn from(&self) -> Option<&String> {
        self.read_header("from")
    }

    pub fn host(&self) -> &String {
        self.read_header("host").unwrap()
    }

    pub fn if_match(&self) -> Option<&String> {
        self.read_header("if-match")
    }

    pub fn if_modified_since(&self) -> Option<&String> {
        self.read_header("if-modified-since")
    }

    pub fn if_none_match(&self) -> Option<&String> {
        self.read_header("if-none-match")
    }

    pub fn if_range(&self) -> Option<&String> {
        self.read_header("if-range")
    }

    pub fn if_unmodified_since(&self) -> Option<&String> {
        self.read_header("if-unmodified-since")
    }

    pub fn max_forwards(&self) -> Option<&String> {
        self.read_header("max-forwards")
    }

    pub fn proxy_authorization(&self) -> Option<&String> {
        self.read_header("proxy-authorization")
    }

    pub fn range(&self) -> Option<&String> {
        self.read_header("range")
    }

    pub fn referrer(&self) -> Option<&String> {
        self.read_header("referrer")
    }

    pub fn te(&self) -> Option<&String> {
        self.read_header("te")
    }

    pub fn user_agent(&self) -> Option<&String> {
        self.read_header("user-agent")
    }

    pub fn content_length(&self) -> Option<&String> {
        self.read_header("content-length")
    }

    pub fn extension(&self, name: &str) -> Option<&String> {
        self.read_header(name)
    }
}
impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for header in &self.headers {
            write!(f, "{}:{}\r\n", header.0, header.1.value)?
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BadRequestError {
    NoHostHeader,
    FailedTargetParse(url::ParseError),
}

#[derive(Debug, PartialEq, Eq)]
struct Header {
    original_key: String,
    value: String,
}
