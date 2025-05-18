use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{BufRead, BufReader, Cursor, Read},
    str::FromStr,
};

use crate::http::Method;

use super::{Headers, Request};

pub trait Parse<T>: Sized {
    type Err;
    fn parse(stream: Cursor<T>) -> Result<Self, Self::Err>;
}
impl Parse<&[u8]> for Request {
    type Err = RequestParseError;
    fn parse(stream: Cursor<&[u8]>) -> Result<Request, RequestParseError> {
        todo!();
    }
}

#[derive(Debug)]
pub enum RequestParseError {
    StatusLineParseFail(String),
    HeaderParseFail(String),
}
impl Error for RequestParseError {}
impl Display for RequestParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestParseError::StatusLineParseFail(v) => write!(f, "Invalid status line: '{v}'"),
            RequestParseError::HeaderParseFail(v) => write!(f, "Invalid header line: '{v}'"),
        }
    }
}

struct StatusLine {
    method: Method,
    request_target: String,
    http_version: String,
}
impl FromStr for StatusLine {
    type Err = RequestParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let status_line = s.split(' ').collect::<Vec<&str>>();
        if status_line.len() != 3 {
            Err(RequestParseError::StatusLineParseFail(s.into()))
        } else {
            let status_line = StatusLine {
                method: Method::from_str(status_line[0]).expect("Failed to parse method"),
                request_target: status_line[1].into(),
                http_version: status_line[2].into(),
            };
            Ok(status_line)
        }
    }
}
