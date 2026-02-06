use crate::http::{Headers, Parse};
use std::io::{self, BufRead, Read};

use super::ParseError;

impl<T: io::Read> Parse<&mut io::BufReader<T>> for Headers {
    type Err = ParseError;

    fn parse(reader: &mut io::BufReader<T>) -> Result<Self, Self::Err> {
        let mut headers = vec![];
        while !is_end_of_headers(reader)? {
            if headers.len() >= Headers::MAX_HEADER_NUMBER {
                tracing::warn!(
                    "Request header number exceeds limit of {}",
                    Headers::MAX_HEADER_NUMBER
                );
                return Err(ParseError::Header);
            }
            let header = read_header(reader).ok_or(ParseError::Header)?;
            let (key, value) = header.split_once(':').ok_or(ParseError::Header)?;
            if contains_whitespace(key) {
                tracing::warn!("unexpected whitespace in header name: '{key}'");
                return Err(ParseError::Header);
            }
            let value = value.trim();
            headers.push((key.to_string(), value.to_string()));
        }
        let headers = Headers::new(headers);
        Ok(headers)
    }
}

fn is_end_of_headers<T: io::Read>(reader: &mut io::BufReader<T>) -> Result<bool, ParseError> {
    let unread_bytes = reader.fill_buf().or(Err(ParseError::Header))?;
    if unread_bytes.len() < 2 {
        Err(ParseError::Header)
    } else {
        Ok(unread_bytes[..2] == *b"\r\n")
    }
}

fn read_header<T: io::Read>(reader: &mut io::BufReader<T>) -> Option<String> {
    let mut header = Vec::with_capacity(Headers::MAX_HEADER_SIZE);
    reader
        .take(Headers::MAX_HEADER_SIZE as u64)
        .read_until(b'\r', &mut header)
        .ok()?;
    reader.consume(1);
    header.pop(); //Remove semicolon
    let header = String::from_utf8(header).ok()?;
    Some(header)
}

fn contains_whitespace(value: &str) -> bool {
    value.chars().any(|c| c.is_whitespace())
}
