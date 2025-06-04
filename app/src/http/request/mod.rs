mod parsing;
pub use parsing::*;

use std::io;
use std::net::{self};

use super::*;

type Body = io::BufReader<net::TcpStream>;

pub struct Request {
    method: Method,
    request_target: String,
    absolute_target: url::Url,
    http_version: String,
    headers: RequestHeaders,
    body: Option<Body>,
}
impl Request {
    fn create(
        method: Method,
        request_target: String,
        http_version: String,
        headers: RequestHeaders,
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
        headers: RequestHeaders,
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
        headers: RequestHeaders,
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
    pub fn headers(&self) -> &RequestHeaders {
        &self.headers
    }

    pub fn body(&mut self) -> &mut Option<Body> {
        &mut self.body
    }

    pub fn matches_path(&self, path: &str) -> bool {
        self.absolute_target().path() == path
    }

    pub fn matches_method(&self, method: &Method) -> bool {
        self.method() == method
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct RequestHeaders {
    headers: Headers,
}
impl RequestHeaders {
    pub fn new(headers: Headers) -> Result<Self, BadRequestError> {
        if headers.get("host").is_none() {
            Err(BadRequestError::NoHostHeader)
        } else {
            Ok(Self { headers })
        }
    }

    pub fn accept(&self) -> Option<&String> {
        self.headers.get("accept")
    }

    pub fn accept_charset(&self) -> Option<&String> {
        self.headers.get("accept-charset")
    }

    pub fn accept_encoding(&self) -> Option<&String> {
        self.headers.get("accept-encoding")
    }

    pub fn accept_language(&self) -> Option<&String> {
        self.headers.get("accept-language")
    }

    pub fn authorization(&self) -> Option<&String> {
        self.headers.get("authorization")
    }

    pub fn expect(&self) -> Option<&String> {
        self.headers.get("expect")
    }

    pub fn from(&self) -> Option<&String> {
        self.headers.get("from")
    }

    pub fn host(&self) -> &String {
        self.headers.get("host").unwrap()
    }

    pub fn if_match(&self) -> Option<&String> {
        self.headers.get("if-match")
    }

    pub fn if_modified_since(&self) -> Option<&String> {
        self.headers.get("if-modified-since")
    }

    pub fn if_none_match(&self) -> Option<&String> {
        self.headers.get("if-none-match")
    }

    pub fn if_range(&self) -> Option<&String> {
        self.headers.get("if-range")
    }

    pub fn if_unmodified_since(&self) -> Option<&String> {
        self.headers.get("if-unmodified-since")
    }

    pub fn max_forwards(&self) -> Option<&String> {
        self.headers.get("max-forwards")
    }

    pub fn proxy_authorization(&self) -> Option<&String> {
        self.headers.get("proxy-authorization")
    }

    pub fn range(&self) -> Option<&String> {
        self.headers.get("range")
    }

    pub fn referrer(&self) -> Option<&String> {
        self.headers.get("referrer")
    }

    pub fn te(&self) -> Option<&String> {
        self.headers.get("te")
    }

    pub fn user_agent(&self) -> Option<&String> {
        self.headers.get("user-agent")
    }

    pub fn content_length(&self) -> Option<&String> {
        self.headers.get("content-length")
    }

    pub fn extension(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }
}
impl Display for RequestHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.headers.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BadRequestError {
    NoHostHeader,
    FailedTargetParse(url::ParseError),
}
