use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{self, BufRead, Cursor, Read, Seek, SeekFrom},
    str::FromStr,
};

use crate::http::{BodyStream, Method};

use super::{Headers, Request};

pub trait Parse: Sized {
    type Err;
    fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Self, Self::Err>;
}
impl Parse for Request {
    type Err = ParseError;
    fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Request, ParseError> {
        let request_line = RequestLine::parse(cursor)?;
        let headers = Headers::parse(cursor)?;
        let request = match headers.content_length() {
            Some(v) => {
                let content_length = v.parse().expect("Failed to parse content length");
                let mut body = vec![0u8; content_length];
                cursor
                    .seek(SeekFrom::Current(2))
                    .expect("Failed skipping newline for reading body");
                cursor.read_exact(&mut body).or(Err(ParseError::Body))?;
                println!("{}", body.len());
                let body_cursor = Cursor::new(body);
                Request::with_body(
                    request_line.method,
                    request_line.request_target,
                    request_line.http_version,
                    headers,
                    BodyStream::new(body_cursor),
                )
            }
            None => Request::new(
                request_line.method,
                request_line.request_target,
                request_line.http_version,
                headers,
            ),
        };
        Ok(request)
    }
}

fn get_string_until(
    stream: &mut Cursor<&[u8]>,
    byte: u8,
    on_error: ParseError,
) -> Result<String, ParseError> {
    let mut buffer = Vec::new();
    stream
        .read_until(byte, &mut buffer)
        .or_else(|e| Err(handle_io_error(e, on_error.clone())))?;
    buffer.pop(); //Remove whitespace from read
    String::from_utf8(buffer).or(Err(on_error))
}

fn handle_io_error(err: io::Error, new_err: ParseError) -> ParseError {
    match err.kind() {
        io::ErrorKind::UnexpectedEof => new_err,
        _ => panic!("Unexpected IO error! : '{}'", err),
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseError {
    RequestLine,
    Header,
    Body,
}
impl Error for ParseError {}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::RequestLine => write!(f, "Invalid request line"),
            ParseError::Header => write!(f, "Invalid header line"),
            ParseError::Body => write!(f, "Failed to read body"),
        }
    }
}

struct RequestLine {
    method: Method,
    request_target: String,
    http_version: String,
}
impl RequestLine {
    fn new(
        method: Method,
        request_target: String,
        http_version: String,
    ) -> Result<Self, ParseError> {
        let request_line = Self {
            method,
            request_target: Self::check_field(request_target)?,
            http_version: Self::check_field(http_version)?,
        };
        Ok(request_line)
    }
    fn check_field(str: String) -> Result<String, ParseError> {
        let bytes = str.as_bytes();
        if bytes.len() == 0 {
            return Err(ParseError::RequestLine);
        }
        if bytes[0] == b' ' || bytes[bytes.len() - 1] == b' ' {
            Err(ParseError::RequestLine)
        } else {
            Ok(str)
        }
    }
}
impl Parse for RequestLine {
    type Err = ParseError;

    fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Self, Self::Err> {
        let result = |result: Result<_, _>| match result {
            Ok(v) => Ok(v),
            Err(_) => Err(ParseError::RequestLine),
        };
        let method = result(get_string_until(cursor, b' ', ParseError::RequestLine))?;
        let request_target = result(get_string_until(cursor, b' ', ParseError::RequestLine))?;
        let http_version = result(get_string_until(cursor, b'\r', ParseError::RequestLine))?;
        let mut newline = [0; 1];
        cursor
            .read_exact(&mut newline)
            .or_else(|e| Err(handle_io_error(e, ParseError::RequestLine)))?;
        let status_line = RequestLine::new(
            Method::from_str(&method).or(Err(ParseError::RequestLine))?,
            request_target,
            http_version,
        )?;
        Ok(status_line)
    }
}

impl Parse for Headers {
    type Err = ParseError;

    fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Self, Self::Err> {
        let mut headers = HashMap::new();
        while !is_end_of_headers(cursor)? {
            let key = get_string_until(cursor, b':', ParseError::Header)?;
            if key.contains(' ') {
                return Err(ParseError::Header);
            }
            let value = get_string_until(cursor, b'\r', ParseError::Header)?;
            let mut newline = [0; 1];
            cursor
                .read_exact(&mut newline)
                .or_else(|e| Err(handle_io_error(e, ParseError::Header)))?;
            headers.insert(key, value);
        }
        Ok(Headers::new(headers)?)
    }
}

fn is_end_of_headers(cursor: &mut Cursor<&[u8]>) -> Result<bool, ParseError> {
    let skipped = cursor
        .skip_until(b'\n')
        .or_else(|e| Err(handle_io_error(e, ParseError::Header)))? as i64;
    cursor.seek(SeekFrom::Current(skipped * -1)).unwrap();
    Ok(skipped == 2)
}

impl From<super::BadRequestError> for ParseError {
    fn from(err: super::BadRequestError) -> Self {
        match err {
            super::BadRequestError::NoHostHeader => ParseError::Header,
        }
    }
}
