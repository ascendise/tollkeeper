use crate::http::{Headers, Parse};
use std::io::{self, BufRead, Read};

use super::{util, ParseError};

impl<T: io::Read> Parse<&mut io::BufReader<T>> for Headers {
    type Err = ParseError;

    fn parse(reader: &mut io::BufReader<T>) -> Result<Self, Self::Err> {
        let mut headers = vec![];
        while !is_end_of_headers(reader)? {
            let key = util::get_string_until(reader, b':', ParseError::Header)?;
            if contains_whitespace(&key) {
                return Err(ParseError::Header);
            }
            let mut value = util::get_string_until(reader, b'\r', ParseError::Header)?;
            value = value.trim().into();
            let mut newline = [0; 1];
            reader
                .read_exact(&mut newline)
                .map_err(|e| util::handle_io_error(e, ParseError::Header))?;
            headers.push((key, value));
        }
        let headers = Headers::new(headers);
        Ok(headers)
    }
}

fn contains_whitespace(value: &str) -> bool {
    value.chars().any(|c| c.is_whitespace())
}

fn is_end_of_headers<T: io::Read>(reader: &mut io::BufReader<T>) -> Result<bool, ParseError> {
    let unread_bytes = reader.fill_buf().or(Err(ParseError::Header))?;
    if unread_bytes.len() < 2 {
        Err(ParseError::Header)
    } else {
        Ok(unread_bytes[..2] == *b"\r\n")
    }
}
