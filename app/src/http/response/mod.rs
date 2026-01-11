#[cfg(test)]
mod tests;

use std::fmt::Display;

use crate::http;

use super::Body;

pub struct Response {
    status_code: StatusCode,
    reason_phrase: Option<String>,
    headers: Headers,
    body: Body,
}
impl Response {
    pub fn new(
        status_code: StatusCode,
        reason_phrase: Option<String>,
        headers: Headers,
        body: Body,
    ) -> Self {
        Self {
            status_code,
            reason_phrase,
            headers,
            body,
        }
    }

    pub fn http_version(&self) -> &str {
        "HTTP/1.1"
    }

    pub fn status_code(&self) -> StatusCode {
        self.status_code
    }

    pub fn reason_phrase(&self) -> Option<&str> {
        self.reason_phrase.as_deref()
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn body(&mut self) -> &mut Body {
        &mut self.body
    }

    /// Turns [Response] into an HTTP representation
    pub fn as_bytes(&mut self) -> Vec<u8> {
        let http_version = self.http_version();
        let status_code: isize = self.status_code as isize;
        let reason_phrase = match &self.reason_phrase {
            Some(v) => v,
            None => "",
        };
        let headers = self.headers();
        let http_message = format!(
            "{} {} {}\r\n{}\r\n",
            http_version, status_code, reason_phrase, headers
        );
        let mut raw_response = Vec::from(http_message.as_bytes());
        if let Body::Buffer(buf) = self.body() {
            raw_response.extend(buf.data());
        }
        raw_response
    }

    pub fn not_found() -> Self {
        Self::new(
            StatusCode::NotFound,
            Some("Not Found".into()),
            Headers::empty(),
            Body::None,
        )
    }

    pub fn method_not_allowed() -> Self {
        Self::new(
            StatusCode::MethodNotAllowed,
            Some("Method Not Allowed".into()),
            Headers::empty(),
            Body::None,
        )
    }

    pub fn internal_server_error() -> Self {
        Self::new(
            StatusCode::InternalServerError,
            Some("Internal Server Error".into()),
            Headers::empty(),
            Body::None,
        )
    }

    pub fn bad_request() -> Self {
        Self::new(
            StatusCode::BadRequest,
            Some("Bad Request".into()),
            Headers::empty(),
            Body::None,
        )
    }

    pub fn payment_required(headers: Headers, body: Body) -> Self {
        Self::new(
            StatusCode::PaymentRequired,
            Some("Payment Required".into()),
            headers,
            body,
        )
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum StatusCode {
    Continue = 100,
    SwitchingProtocols = 101,
    OK = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    ContentTooLarge = 413,
    URITooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    MisdirectedRequest = 421,
    UnprocessableContent = 422,
    UpgradeRequired = 426,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HttpVersionNotSupported = 505,
}
impl StatusCode {
    pub fn from(value: &str) -> Option<Self> {
        let status_code = match value {
            "100" => StatusCode::Continue,
            "101" => StatusCode::SwitchingProtocols,
            "200" => StatusCode::OK,
            "201" => StatusCode::Created,
            "202" => StatusCode::Accepted,
            "203" => StatusCode::NonAuthoritativeInformation,
            "204" => StatusCode::NoContent,
            "205" => StatusCode::ResetContent,
            "206" => StatusCode::PartialContent,
            "300" => StatusCode::MultipleChoices,
            "301" => StatusCode::MovedPermanently,
            "302" => StatusCode::Found,
            "303" => StatusCode::SeeOther,
            "304" => StatusCode::NotModified,
            "305" => StatusCode::UseProxy,
            "307" => StatusCode::TemporaryRedirect,
            "308" => StatusCode::PermanentRedirect,
            "400" => StatusCode::BadRequest,
            "401" => StatusCode::Unauthorized,
            "402" => StatusCode::PaymentRequired,
            "403" => StatusCode::Forbidden,
            "404" => StatusCode::NotFound,
            "405" => StatusCode::MethodNotAllowed,
            "406" => StatusCode::NotAcceptable,
            "407" => StatusCode::ProxyAuthenticationRequired,
            "408" => StatusCode::RequestTimeout,
            "409" => StatusCode::Conflict,
            "410" => StatusCode::Gone,
            "411" => StatusCode::LengthRequired,
            "412" => StatusCode::PreconditionFailed,
            "413" => StatusCode::ContentTooLarge,
            "414" => StatusCode::URITooLong,
            "415" => StatusCode::UnsupportedMediaType,
            "416" => StatusCode::RangeNotSatisfiable,
            "417" => StatusCode::ExpectationFailed,
            "421" => StatusCode::MisdirectedRequest,
            "422" => StatusCode::UnprocessableContent,
            "426" => StatusCode::UpgradeRequired,
            "500" => StatusCode::InternalServerError,
            "501" => StatusCode::NotImplemented,
            "502" => StatusCode::BadGateway,
            "503" => StatusCode::ServiceUnavailable,
            "504" => StatusCode::GatewayTimeout,
            "505" => StatusCode::HttpVersionNotSupported,
            _ => return None,
        };
        Some(status_code)
    }
    pub fn reason_phrase(&self) -> &str {
        match self {
            StatusCode::Continue => "Continue",
            StatusCode::SwitchingProtocols => "Switching Protocols",
            StatusCode::OK => "OK",
            StatusCode::Created => "Created",
            StatusCode::Accepted => "Accepted",
            StatusCode::NonAuthoritativeInformation => "Non Authoritative Information",
            StatusCode::NoContent => "No Content",
            StatusCode::ResetContent => "Reset Content",
            StatusCode::PartialContent => "Partial Content",
            StatusCode::MultipleChoices => "Multiple Choices",
            StatusCode::MovedPermanently => "Moved Permanently",
            StatusCode::Found => "Found",
            StatusCode::SeeOther => "See Other",
            StatusCode::NotModified => "Not Modified",
            StatusCode::UseProxy => "Use Proxy",
            StatusCode::TemporaryRedirect => "Temporary Redirect",
            StatusCode::PermanentRedirect => "Permanent Redirect",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::PaymentRequired => "Payment Required",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::NotAcceptable => "Not Acceptable",
            StatusCode::ProxyAuthenticationRequired => "Proxy Authentication Required",
            StatusCode::RequestTimeout => "Request Timeout",
            StatusCode::Conflict => "Conflict",
            StatusCode::Gone => "Gone",
            StatusCode::LengthRequired => "Length Required",
            StatusCode::PreconditionFailed => "Precondition Failed",
            StatusCode::ContentTooLarge => "Content Too Large",
            StatusCode::URITooLong => "URI Too Long",
            StatusCode::UnsupportedMediaType => "Unsupported Media Type",
            StatusCode::RangeNotSatisfiable => "Range Not Satisfiable",
            StatusCode::ExpectationFailed => "Expectation Failed",
            StatusCode::MisdirectedRequest => "Misdirected Request",
            StatusCode::UnprocessableContent => "Unprocessable Content",
            StatusCode::UpgradeRequired => "Upgrade Required",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::BadGateway => "Bad Gateway",
            StatusCode::ServiceUnavailable => "Service Unavailable",
            StatusCode::GatewayTimeout => "GatewayTimeout",
            StatusCode::HttpVersionNotSupported => "HTTP Version Not Supported",
        }
    }
}
impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Headers(http::Headers);
impl Headers {
    pub fn new(headers: http::Headers) -> Self {
        Self(headers)
    }
    pub fn empty() -> Self {
        Self(http::Headers::empty())
    }
    pub fn content_length(&self) -> Option<usize> {
        self.0.get("Content-Length").map(|v| v.parse().unwrap())
    }
    pub fn content_type(&self) -> Option<&str> {
        self.0.get("Content-Type")
    }
    pub fn transfer_encoding(&self) -> Option<&str> {
        self.0.get("Transfer-Encoding")
    }
    pub fn extension(&self, key: &str) -> Option<&str> {
        self.0.get(key)
    }
}
impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
