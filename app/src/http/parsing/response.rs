use crate::http::response::{self, Response, StatusCode};
use crate::http::{self, Body, Parse};
use std::io::{self};
use std::io::{BufRead, BufReader};

use super::ParseError;

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
        let mut status_line = Vec::new();
        stream
            .read_until(b'\r', &mut status_line)
            .or(Err(ParseError::StatusLine))?;
        status_line.pop(); // Remove trailing CR
        let status_line = String::from_utf8(status_line).or(Err(ParseError::StatusLine))?;
        let status_line: Vec<&str> = status_line.splitn(3, ' ').collect();
        let status_code = status_line.get(1).ok_or(ParseError::StatusLine)?;
        let status_code = StatusCode::from(status_code).ok_or(ParseError::StatusLine)?;
        let reason_phrase = status_line.get(2).unwrap_or(&"");
        stream.consume(1); //Consume trailing LF
        let status_line = if reason_phrase.is_empty() || reason_phrase.contains(char::is_whitespace)
        {
            StatusLine::new(status_code, Option::None)
        } else {
            StatusLine::new(status_code, Option::Some(reason_phrase.trim().into()))
        };
        Ok(status_line)
    }
}

impl<T: io::Read> Parse<&mut io::BufReader<T>> for response::Headers {
    type Err = ParseError;

    fn parse(stream: &mut io::BufReader<T>) -> Result<Self, Self::Err> {
        let headers = http::Headers::parse(stream)?;
        Ok(response::Headers::new(headers))
    }
}
