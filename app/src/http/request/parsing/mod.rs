#[cfg(test)]
mod tests;

use std::{
    error::Error,
    fmt::Display,
    io::{self, BufRead, Read},
    net,
    str::FromStr,
};

use indexmap::IndexMap;

use super::{Headers, Method, Request, RequestHeaders};

pub trait Parse<T>: Sized {
    type Err;
    fn parse(stream: T) -> Result<Self, Self::Err>;
}
impl Parse<io::BufReader<net::TcpStream>> for Request {
    type Err = ParseError;
    fn parse(mut stream: io::BufReader<net::TcpStream>) -> Result<Request, ParseError> {
        let request_line = RequestLine::parse(&mut stream)?;
        let headers = RequestHeaders::parse(&mut stream)?;
        let request = if headers.content_length().is_some() {
            stream.consume(2); //Consume additional newline for body
            Request::with_body(
                request_line.method,
                request_line.request_target,
                headers,
                stream,
            )
        } else {
            Request::new(request_line.method, request_line.request_target, headers)
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
        .read_until(byte, &mut buffer)
        .map_err(|e| handle_io_error(e, on_error.clone()))?;
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
        if http_version != "HTTP/1.1" {
            return Err(ParseError::RequestLine);
        }
        let request_line = Self {
            method,
            request_target: Self::check_field_format(request_target)?,
            http_version: Self::check_field_format(http_version)?,
        };
        Ok(request_line)
    }
    fn check_field_format(str: String) -> Result<String, ParseError> {
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
            .read_exact(&mut newline)
            .map_err(|e| handle_io_error(e, ParseError::RequestLine))?;
        let status_line = RequestLine::new(
            Method::from_str(&method).or(Err(ParseError::RequestLine))?,
            request_target,
            http_version,
        )?;
        Ok(status_line)
    }
}

impl Parse<&mut io::BufReader<net::TcpStream>> for RequestHeaders {
    type Err = ParseError;

    fn parse(reader: &mut io::BufReader<net::TcpStream>) -> Result<Self, Self::Err> {
        let mut headers = IndexMap::new();
        while !is_end_of_headers(reader)? {
            let key = get_string_until(reader, b':', ParseError::Header)?;
            if contains_whitespace(&key) {
                return Err(ParseError::Header);
            }
            let mut value = get_string_until(reader, b'\r', ParseError::Header)?;
            value = value.trim().into();
            let mut newline = [0; 1];
            reader
                .read_exact(&mut newline)
                .map_err(|e| handle_io_error(e, ParseError::Header))?;
            headers.insert(key, value);
        }
        let headers = Headers::new(headers);
        Ok(RequestHeaders::new(headers)?)
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
            super::BadRequestError::MismatchedTargetHost => ParseError::Header,
            super::BadRequestError::FailedTargetParse(_) => ParseError::RequestLine,
        }
    }
}
