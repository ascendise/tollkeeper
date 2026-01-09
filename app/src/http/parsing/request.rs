use std::{
    io::{self, BufRead, Read},
    str::FromStr,
};

use super::{util, ParseError};
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
        let request = if headers.content_length().is_some() {
            stream.consume(2); //Consume additional newline for body
            let content_length = headers.content_length();
            Request::new(
                request_line.method,
                request_line.request_target,
                headers,
                Body::from_stream(Box::new(stream), content_length),
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
impl<T: Read> Parse<&mut io::BufReader<T>> for RequestLine {
    type Err = ParseError;

    fn parse(reader: &mut io::BufReader<T>) -> Result<Self, Self::Err> {
        let result = |result: Result<_, _>| match result {
            Ok(v) => Ok(v),
            Err(_) => Err(ParseError::RequestLine),
        };
        let method = result(util::get_string_until(
            reader,
            b' ',
            ParseError::RequestLine,
        ))?;
        let request_target = result(util::get_string_until(
            reader,
            b' ',
            ParseError::RequestLine,
        ))?;
        let http_version = result(util::get_string_until(
            reader,
            b'\r',
            ParseError::RequestLine,
        ))?;
        let mut newline = [0; 1];
        reader
            .read_exact(&mut newline)
            .map_err(|e| util::handle_io_error(e, ParseError::RequestLine))?;
        let status_line = RequestLine::new(
            Method::from_str(&method).or(Err(ParseError::RequestLine))?,
            request_target,
            http_version,
        )?;
        Ok(status_line)
    }
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
