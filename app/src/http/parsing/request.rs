use std::{
    io::{self, BufRead, Read},
    str::FromStr,
};

use super::ParseError;
use crate::http::{
    self,
    request::{self, BadRequestError, *},
};
use crate::http::{Body, Parse};

impl<T: Read + 'static> Parse<T> for Request {
    type Err = ParseError;
    fn parse(stream: T) -> Result<Request, ParseError> {
        let mut stream = io::BufReader::new(stream);
        let request_line = RequestLine::parse(&mut stream)?;
        let headers = request::Headers::parse(&mut stream)?;
        stream.consume(2); //Consume trailing CRLF
        let request = if let Some(content_length) = headers.content_length() {
            if content_length > Request::MAX_BODY_SIZE {
                tracing::warn!(
                    "Content-Length exceeds maximum: {}MB > MAX_BODY_SIZE!",
                    content_length / 1024 / 1024
                );
                return Err(ParseError::Body);
            }
            Request::new(
                request_line.method,
                request_line.request_target,
                headers,
                Body::from_stream(Box::new(stream), Some(content_length)),
            )
        } else {
            Request::new(
                request_line.method,
                request_line.request_target,
                headers,
                Body::None,
            )
        }?;
        Ok(request)
    }
}

struct RequestLine {
    method: Method,
    request_target: String,
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
        };
        Ok(request_line)
    }

    fn read_raw_request_line<T: io::Read>(reader: &mut io::BufReader<T>) -> Option<String> {
        let mut request_line = Vec::with_capacity(Request::MAX_REQUEST_LINE_SIZE);
        reader
            .take(Request::MAX_REQUEST_LINE_SIZE as u64)
            .read_until(b'\r', &mut request_line)
            .ok()?;
        reader.consume(1); // Consume newline
        request_line.pop(); //Remove trailing CR from output
        let request_line = String::from_utf8(request_line).ok()?;
        Some(request_line)
    }

    fn read_request_line_part<'a>(
        index: usize,
        request_line: &Vec<&'a str>,
    ) -> Result<&'a str, ParseError> {
        let value = request_line.get(index).ok_or(ParseError::RequestLine)?;
        Ok(value)
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
impl<T: Read> Parse<&mut io::BufReader<T>> for RequestLine {
    type Err = ParseError;

    fn parse(reader: &mut io::BufReader<T>) -> Result<Self, Self::Err> {
        let request_line = Self::read_raw_request_line(reader).ok_or(ParseError::RequestLine)?;
        if has_trailing_whitespace(&request_line) {
            tracing::warn!("request line has trailing whitespace");
            return Err(ParseError::RequestLine);
        }
        let request_line: Vec<&str> = request_line.splitn(3, ' ').collect();
        let method = Self::read_request_line_part(0, &request_line)?;
        let request_target = Self::read_request_line_part(1, &request_line)?;
        let version = Self::read_request_line_part(2, &request_line)?;
        let status_line = RequestLine::new(
            Method::from_str(method).or(Err(ParseError::RequestLine))?,
            request_target.into(),
            version.into(),
        )?;
        Ok(status_line)
    }
}

fn has_trailing_whitespace(request_line: &str) -> bool {
    request_line.trim_end() != request_line
}

impl<T: Read> Parse<&mut io::BufReader<T>> for Headers {
    type Err = ParseError;

    fn parse(reader: &mut io::BufReader<T>) -> Result<Self, Self::Err> {
        let headers = http::Headers::parse(reader);
        Ok(Headers::new(headers?)?)
    }
}

impl From<BadRequestError> for ParseError {
    fn from(err: BadRequestError) -> Self {
        match err {
            BadRequestError::NoHostHeader => ParseError::Header,
            BadRequestError::MismatchedTargetHost => ParseError::Header,
            BadRequestError::FailedTargetParse(_) => ParseError::RequestLine,
        }
    }
}
