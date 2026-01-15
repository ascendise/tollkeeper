use super::*;
use crate::http;
pub mod body_reader;

pub struct Request {
    method: Method,
    request_target: String,
    absolute_target: url::Url,
    headers: Headers,
    body: Body,
}
impl Request {
    pub fn new(
        method: Method,
        request_target: impl Into<String>,
        headers: Headers,
        body: Body,
    ) -> Result<Self, BadRequestError> {
        Self::create(method, request_target.into(), headers, body)
    }

    fn create(
        method: Method,
        request_target: String,
        headers: Headers,
        body: Body,
    ) -> Result<Self, BadRequestError> {
        let absolute_target = Self::resolve_absolute_target(&request_target, &headers)?;
        let request = Self {
            method,
            request_target,
            absolute_target,
            headers,
            body,
        };
        Ok(request)
    }

    fn resolve_absolute_target(
        request_target: &str,
        headers: &Headers,
    ) -> Result<url::Url, BadRequestError> {
        let protocol = String::from("http://");
        let host_url = protocol.clone() + headers.host();
        let mut host_url =
            url::Url::parse(&host_url).map_err(BadRequestError::FailedTargetParse)?;
        let absolute_target = if Self::is_relative(request_target) {
            host_url.set_path(request_target);
            Ok(host_url)
        } else {
            let target_url = protocol + request_target;
            let absolute_target =
                url::Url::parse(&target_url).map_err(BadRequestError::FailedTargetParse)?;
            if absolute_target.host() != host_url.host() {
                Err(BadRequestError::MismatchedTargetHost)
            } else {
                Ok(absolute_target)
            }
        }?;
        Ok(absolute_target)
    }

    fn is_relative(request_target: &str) -> bool {
        request_target.starts_with("/")
    }

    /// HTTP Protocol version
    pub fn http_version(&self) -> &str {
        "HTTP/1.1"
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

    pub fn body(&mut self) -> &mut Body {
        &mut self.body
    }

    pub fn matches_path(&self, path: &str) -> bool {
        self.absolute_target().path() == path
    }

    pub fn matches_method(&self, method: &Method) -> bool {
        self.method() == method
    }

    /// Turns [Request] into an HTTP representation
    pub fn as_bytes(&mut self) -> Vec<u8> {
        let method = self.method();
        let request_target = self.request_target();
        let http_version = self.http_version();
        let headers = self.headers();
        let http_message = format!(
            "{} {} {}\r\n{}\r\n",
            method, request_target, http_version, headers
        );
        let mut raw_data = Vec::from(http_message.as_bytes());
        if let Body::Buffer(body) = self.body() {
            raw_data.extend(body.data());
        }
        raw_data
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Headers {
    headers: http::Headers,
}
impl Headers {
    pub fn new(headers: http::Headers) -> Result<Self, BadRequestError> {
        if headers.get("host").is_none() {
            Err(BadRequestError::NoHostHeader)
        } else {
            Ok(Self { headers })
        }
    }

    pub fn accept(&self) -> Option<&str> {
        self.headers.get("accept")
    }

    pub fn accept_charset(&self) -> Option<&str> {
        self.headers.get("accept-charset")
    }

    pub fn accept_encoding(&self) -> Option<&str> {
        self.headers.get("accept-encoding")
    }

    pub fn accept_language(&self) -> Option<&str> {
        self.headers.get("accept-language")
    }

    pub fn authorization(&self) -> Option<&str> {
        self.headers.get("authorization")
    }

    pub fn expect(&self) -> Option<&str> {
        self.headers.get("expect")
    }

    pub fn from(&self) -> Option<&str> {
        self.headers.get("from")
    }

    pub fn host(&self) -> &str {
        self.headers.get("host").unwrap()
    }

    pub fn if_match(&self) -> Option<&str> {
        self.headers.get("if-match")
    }

    pub fn if_modified_since(&self) -> Option<&str> {
        self.headers.get("if-modified-since")
    }

    pub fn if_none_match(&self) -> Option<&str> {
        self.headers.get("if-none-match")
    }

    pub fn if_range(&self) -> Option<&str> {
        self.headers.get("if-range")
    }

    pub fn if_unmodified_since(&self) -> Option<&str> {
        self.headers.get("if-unmodified-since")
    }

    pub fn max_forwards(&self) -> Option<&str> {
        self.headers.get("max-forwards")
    }

    pub fn proxy_authorization(&self) -> Option<&str> {
        self.headers.get("proxy-authorization")
    }

    pub fn range(&self) -> Option<&str> {
        self.headers.get("range")
    }

    pub fn referrer(&self) -> Option<&str> {
        self.headers.get("referrer")
    }

    pub fn te(&self) -> Option<&str> {
        self.headers.get("te")
    }

    pub fn user_agent(&self) -> Option<&str> {
        self.headers.get("user-agent")
    }

    pub fn content_length(&self) -> Option<usize> {
        let content_length = self.headers.get("content-length")?;
        usize::from_str(content_length).ok()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type")
    }

    pub fn cookie(&self, key: &str) -> Option<&str> {
        let cookies = self.headers.get("Cookie")?;
        let cookies = cookies
            .split(";")
            .map(|s| s.trim().split_once("=").unwrap_or(("", "")))
            .collect::<Vec<(&str, &str)>>();
        for (cookie_key, cookie_value) in cookies {
            if cookie_key == key {
                return Some(cookie_value);
            }
        }
        None
    }

    pub fn extension(&self, name: &str) -> Option<&str> {
        self.headers.get(name)
    }
}
impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.headers.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BadRequestError {
    NoHostHeader,
    MismatchedTargetHost,
    FailedTargetParse(url::ParseError),
}
