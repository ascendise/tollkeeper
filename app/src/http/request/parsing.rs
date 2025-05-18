use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{BufRead, Cursor},
    str::FromStr,
};

use crate::http::Method;

use super::{Headers, Request};

pub trait Parse: Sized {
    type Err;
    fn parse(stream: &mut Cursor<&[u8]>) -> Result<Self, Self::Err>;
    fn get_string_until(stream: &mut Cursor<&[u8]>, byte: u8) -> Result<String, ()> {
        let mut buffer = Vec::new();
        if let Err(_) = stream.read_until(byte, &mut buffer) {
            return Err(());
        };
        buffer.pop(); //Remove whitespace from read
        match String::from_utf8(buffer) {
            Ok(s) => Ok(s),
            Err(_) => Err(()),
        }
    }
}
impl Parse for Request {
    type Err = RequestParseError;
    fn parse(stream: &mut Cursor<&[u8]>) -> Result<Request, RequestParseError> {
        let status_line = StatusLine::parse(stream).unwrap();
        let headers = Headers::new(HashMap::new());
        let request = Request::new(
            status_line.method,
            status_line.request_target,
            status_line.http_version,
            headers,
        );
        Ok(request)
    }
}

#[derive(Debug)]
pub enum RequestParseError {
    StatusLineParseFail,
    HeaderParseFail(String),
}
impl Error for RequestParseError {}
impl Display for RequestParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestParseError::StatusLineParseFail => write!(f, "Invalid status line"),
            RequestParseError::HeaderParseFail(v) => write!(f, "Invalid header line: '{v}'"),
        }
    }
}

struct StatusLine {
    method: Method,
    request_target: String,
    http_version: String,
}
impl Parse for StatusLine {
    type Err = RequestParseError;

    fn parse(stream: &mut Cursor<&[u8]>) -> Result<Self, Self::Err> {
        let result = |result: Result<_, _>| match result {
            Ok(v) => Ok(v),
            Err(_) => Err(RequestParseError::StatusLineParseFail),
        };
        let method = result(Self::get_string_until(stream, b' '))?;
        let request_target = result(Self::get_string_until(stream, b' '))?;
        let http_version = result(Self::get_string_until(stream, b' '))?;
        let status_line = StatusLine {
            method: Method::from_str(&method).unwrap(),
            request_target,
            http_version,
        };
        Ok(status_line)
    }
}
