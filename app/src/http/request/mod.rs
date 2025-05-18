mod parsing;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use super::*;

pub struct Request {
    method: Method,
    uri: String,
    http_version: String,
    host: String,
    headers: Headers,
    body: Option<BodyStream>,
}
impl Request {
    pub fn new(
        method: Method,
        uri: impl Into<String>,
        http_version: impl Into<String>,
        host: impl Into<String>,
        headers: Headers,
    ) -> Self {
        Self {
            method,
            uri: uri.into(),
            http_version: http_version.into(),
            host: host.into(),
            headers,
            body: None,
        }
    }

    pub fn with_body(
        method: Method,
        uri: impl Into<String>,
        http_version: impl Into<String>,
        host: impl Into<String>,
        headers: Headers,
        body: BodyStream,
    ) -> Self {
        Self {
            method,
            uri: uri.into(),
            http_version: http_version.into(),
            host: host.into(),
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
        &self.uri
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Target host for request
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Request headers
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn body(&mut self) -> &Option<BodyStream> {
        &self.body
    }
}

pub struct Headers {
    headers: HashMap<String, String>,
}
impl Headers {
    pub fn new(headers: HashMap<String, String>) -> Self {
        Self { headers }
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

    pub fn host(&self) -> Option<&String> {
        self.headers.get("Host")
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

    pub fn extension(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }
}
