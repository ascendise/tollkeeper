use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{BufRead, Cursor, Read},
    str::FromStr,
};

use crate::http::Method;

use super::{Headers, Request};

pub trait Parse: Sized {
    type Err;
    fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Self, Self::Err>;
}
impl Parse for Request {
    type Err = RequestParseError;
    fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Request, RequestParseError> {
        let status_line = StatusLine::parse(cursor).unwrap();
        let headers = parse_headers(cursor).or(Err(RequestParseError::HeaderParseFail))?;
        let headers = Headers::new(headers);
        let request = Request::new(
            status_line.method,
            status_line.request_target,
            status_line.http_version,
            headers,
        );
        Ok(request)
    }
}

fn parse_headers(cursor: &mut Cursor<&[u8]>) -> Result<HashMap<String, String>, ()> {
    let mut headers = HashMap::new();
    let key = get_string_until(cursor, b':')?;
    let value = get_string_until(cursor, b'\r')?;
    let mut newline = [0; 1];
    cursor.read_exact(&mut newline).unwrap();
    headers.insert(key, value);
    Ok(headers)
}
fn get_string_until(stream: &mut Cursor<&[u8]>, byte: u8) -> Result<String, ()> {
    let mut buffer = Vec::new();
    if stream.read_until(byte, &mut buffer).is_err() {
        return Err(());
    };
    buffer.pop(); //Remove whitespace from read
    match String::from_utf8(buffer) {
        Ok(s) => Ok(s),
        Err(_) => Err(()),
    }
}

#[derive(Debug)]
pub enum RequestParseError {
    StatusLineParseFail,
    HeaderParseFail,
}
impl Error for RequestParseError {}
impl Display for RequestParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestParseError::StatusLineParseFail => write!(f, "Invalid status line"),
            RequestParseError::HeaderParseFail => write!(f, "Invalid header line"),
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

    fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Self, Self::Err> {
        let result = |result: Result<_, _>| match result {
            Ok(v) => Ok(v),
            Err(_) => Err(RequestParseError::StatusLineParseFail),
        };
        let method = result(get_string_until(cursor, b' '))?;
        let request_target = result(get_string_until(cursor, b' '))?;
        let http_version = result(get_string_until(cursor, b'\r'))?;
        let mut newline = [0; 1];
        cursor.read_exact(&mut newline).unwrap();
        let status_line = StatusLine {
            method: Method::from_str(&method).unwrap(),
            request_target,
            http_version,
        };
        Ok(status_line)
    }
}
