use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{self, BufRead, Read},
    net,
    str::FromStr,
};

use super::{Headers, Request};
use crate::http::Method;

pub trait Parse<T>: Sized {
    type Err;
    fn parse(stream: T) -> Result<Self, Self::Err>;
}
impl Parse<io::BufReader<net::TcpStream>> for Request {
    type Err = ParseError;
    fn parse(mut stream: io::BufReader<net::TcpStream>) -> Result<Request, ParseError> {
        let request_line = RequestLine::parse(&mut stream)?;
        let headers = Headers::parse(&mut stream)?;
        let peek_body = stream.fill_buf().or(Err(ParseError::Body))?;
        let request = if peek_body == *b"\r\n00" {
            Request::new(
                request_line.method,
                request_line.request_target,
                request_line.http_version,
                headers,
            )
        } else {
            stream.consume(2);
            Request::with_body(
                request_line.method,
                request_line.request_target,
                request_line.http_version,
                headers,
                stream,
            )
        }?;
        Ok(request)
    }
}

fn get_string_until(
    stream: &mut io::BufReader<net::TcpStream>,
    byte: u8,
    on_error: ParseError,
) -> Result<String, ParseError> {
    let mut buffer = Vec::new();
    stream
        .read_until(byte, &mut buffer).map_err(|e| handle_io_error(e, on_error.clone()))?;
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
        if bytes.is_empty() {
            return Err(ParseError::RequestLine);
        }
        if bytes[0] == b' ' || bytes[bytes.len() - 1] == b' ' {
            Err(ParseError::RequestLine)
        } else {
            Ok(str)
        }
    }
}
impl Parse<&mut io::BufReader<net::TcpStream>> for RequestLine {
    type Err = ParseError;

    fn parse(reader: &mut io::BufReader<net::TcpStream>) -> Result<Self, Self::Err> {
        let result = |result: Result<_, _>| match result {
            Ok(v) => Ok(v),
            Err(_) => Err(ParseError::RequestLine),
        };
        let method = result(get_string_until(reader, b' ', ParseError::RequestLine))?;
        let request_target = result(get_string_until(reader, b' ', ParseError::RequestLine))?;
        let http_version = result(get_string_until(reader, b'\r', ParseError::RequestLine))?;
        let mut newline = [0; 1];
        reader
            .read_exact(&mut newline).map_err(|e| handle_io_error(e, ParseError::RequestLine))?;
        let status_line = RequestLine::new(
            Method::from_str(&method).or(Err(ParseError::RequestLine))?,
            request_target,
            http_version,
        )?;
        Ok(status_line)
    }
}

impl Parse<&mut io::BufReader<net::TcpStream>> for Headers {
    type Err = ParseError;

    fn parse(reader: &mut io::BufReader<net::TcpStream>) -> Result<Self, Self::Err> {
        let mut headers = HashMap::new();
        while !is_end_of_headers(reader)? {
            let key = get_string_until(reader, b':', ParseError::Header)?;
            if contains_whitespace(&key) {
                return Err(ParseError::Header);
            }
            let mut value = get_string_until(reader, b'\r', ParseError::Header)?;
            value = value.trim().into();
            let mut newline = [0; 1];
            reader
                .read_exact(&mut newline).map_err(|e| handle_io_error(e, ParseError::Header))?;
            headers.insert(key, value);
        }
        Ok(Headers::new(headers)?)
    }
}

fn contains_whitespace(value: &str) -> bool {
    value.chars().any(|c| c.is_whitespace())
}

fn is_end_of_headers(reader: &mut io::BufReader<net::TcpStream>) -> Result<bool, ParseError> {
    let unread_bytes = reader.fill_buf().or(Err(ParseError::Header))?;
    if unread_bytes.len() < 2 {
        Err(ParseError::Header)
    } else {
        Ok(unread_bytes[..2] == *b"\r\n")
    }
}

impl From<super::BadRequestError> for ParseError {
    fn from(err: super::BadRequestError) -> Self {
        match err {
            super::BadRequestError::NoHostHeader => ParseError::Header,
            super::BadRequestError::FailedTargetParse(_) => ParseError::RequestLine,
        }
    }
}
