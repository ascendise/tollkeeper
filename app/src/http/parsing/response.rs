use crate::http::response::{self, Response, StatusCode};
use crate::http::{self, Body, Parse};
use std::io::{self};
use std::io::{BufRead, BufReader};

use super::{util, ParseError};

impl<T: io::Read + 'static> Parse<T> for Response {
    type Err = ParseError;

    fn parse(stream: T) -> Result<Self, Self::Err> {
        let mut stream = BufReader::new(stream);
        let status_line = StatusLine::parse(&mut stream)?;
        let headers = response::Headers::parse(&mut stream)?;
        let response = if has_body(&headers) {
            stream.consume(2); //Consume additional newline for body
            let body = Body::from_stream(Box::new(stream), headers.content_length());
            Response::new(
                status_line.status_code,
                status_line.reason_phrase,
                headers,
                body,
            )
        } else {
            Response::new(
                status_line.status_code,
                status_line.reason_phrase,
                headers,
                Body::None,
            )
        };
        Ok(response)
    }
}

fn has_body(headers: &response::Headers) -> bool {
    headers.content_length().is_some() || headers.transfer_encoding().unwrap_or("") == "chunked"
}

struct StatusLine {
    status_code: StatusCode,
    reason_phrase: Option<String>,
}
impl StatusLine {
    pub fn new(status_code: StatusCode, reason_phrase: Option<String>) -> Self {
        Self {
            status_code,
            reason_phrase,
        }
    }
}
impl<T: io::Read> Parse<&mut io::BufReader<T>> for StatusLine {
    type Err = ParseError;

    fn parse(stream: &mut BufReader<T>) -> Result<Self, Self::Err> {
        let _ = util::get_string_until(stream, b' ', ParseError::StatusLine)?; //HTTP Version ->
                                                                               //TODO: Err if HTTP version is not HTTP/1.1
        let status_code = util::get_string_until(stream, b' ', ParseError::StatusLine)?;
        let status_code = StatusCode::from(&status_code).ok_or(ParseError::StatusLine)?;
        let reason_phrase = util::get_string_until(stream, b'\r', ParseError::StatusLine)?;
        stream.consume(1);
        if !reason_phrase.len() > 0 && !reason_phrase.contains(char::is_whitespace) {
            let status_line =
                StatusLine::new(status_code, Option::Some(reason_phrase.trim().into()));
            Ok(status_line)
        } else {
            let status_line = StatusLine::new(status_code, Option::None);
            Ok(status_line)
        }
    }
}

impl<T: io::Read> Parse<&mut io::BufReader<T>> for response::Headers {
    type Err = ParseError;

    fn parse(stream: &mut io::BufReader<T>) -> Result<Self, Self::Err> {
        let headers = http::Headers::parse(stream)?;
        Ok(response::Headers::new(headers))
    }
}
