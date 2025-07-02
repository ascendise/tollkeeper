use super::*;

pub struct Request {
    method: Method,
    request_target: String,
    absolute_target: url::Url,
    headers: RequestHeaders,
    body: Option<Box<dyn Body>>,
}
impl Request {
    pub fn new(
        method: Method,
        request_target: impl Into<String>,
        headers: RequestHeaders,
    ) -> Result<Self, BadRequestError> {
        Self::create(method, request_target.into(), headers, None)
    }

    pub fn with_body(
        method: Method,
        request_target: impl Into<String>,
        headers: RequestHeaders,
        body: Box<dyn Body>,
    ) -> Result<Self, BadRequestError> {
        Self::create(method, request_target.into(), headers, Some(body))
    }

    fn create(
        method: Method,
        request_target: String,
        headers: RequestHeaders,
        body: Option<Box<dyn Body>>,
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
        headers: &RequestHeaders,
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
    pub fn headers(&self) -> &RequestHeaders {
        &self.headers
    }

    pub fn body(&mut self) -> &mut Option<Box<dyn Body>> {
        &mut self.body
    }

    pub fn matches_path(&self, path: &str) -> bool {
        self.absolute_target().path() == path
    }

    pub fn matches_method(&self, method: &Method) -> bool {
        self.method() == method
    }

    /// Turns [Request] into an HTTP representation
    /// Consumes [self] to avoid having two copies of the body
    pub fn into_bytes(self) -> Vec<u8> {
        let method = self.method();
        let request_target = self.request_target();
        let http_version = self.http_version();
        let headers = self.headers();
        let http_message = format!(
            "{} {} {}\r\n{}\r\n",
            method, request_target, http_version, headers
        );
        let mut raw_data = Vec::from(http_message.as_bytes());
        if self.body.is_some() {
            let mut body = self.body.unwrap();
            let mut data = String::new();
            body.read_to_string(&mut data).unwrap();
            raw_data.extend(data.as_bytes());
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
    MismatchedTargetHost,
    FailedTargetParse(url::ParseError),
}
