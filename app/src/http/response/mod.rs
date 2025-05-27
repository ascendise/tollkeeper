#[cfg(test)]
mod tests;

use std::fmt::Display;

use super::Headers;

pub struct Response {
    http_version: String,
    status_code: StatusCode,
    reason_phrase: Option<String>,
    headers: ResponseHeaders,
    body: Vec<u8>,
}
impl Response {
    pub fn new(
        http_version: impl Into<String>,
        status_code: StatusCode,
        reason_phrase: Option<String>,
        headers: ResponseHeaders,
        body: Vec<u8>,
    ) -> Self {
        Self {
            http_version: http_version.into(),
            status_code,
            reason_phrase,
            headers,
            body,
        }
    }

    pub fn http_version(&self) -> &str {
        &self.http_version
    }

    pub fn status_code(&self) -> StatusCode {
        self.status_code
    }

    pub fn reason_phrase(&self) -> Option<&String> {
        self.reason_phrase.as_ref()
    }

    pub fn headers(&self) -> &ResponseHeaders {
        &self.headers
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Turns [Response] into an HTTP representation
    /// Consumes [self] to avoid having two copies of the body
    pub fn into_bytes(self) -> Vec<u8> {
        let http_version = &self.http_version;
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
        let body = self.body;
        let mut raw_data = Vec::from(http_message.as_bytes());
        raw_data.extend(body);
        raw_data
    }
}

#[derive(Copy, Clone)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct ResponseHeaders(Headers);
impl ResponseHeaders {
    pub fn extension(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}
impl Display for ResponseHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
