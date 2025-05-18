use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

use crate::http::Method;

use super::{Headers, Request};

pub trait Parse {
    fn parse(stream: impl Read) -> Result<Request, RequestParseError>;
}
impl Parse for Request {
    fn parse(stream: impl Read) -> Result<Request, RequestParseError> {
        let reader = BufReader::new(stream);
        let lines: Vec<String> = reader
            .lines()
            .map(|r| r.unwrap())
            .take_while(|l| l.is_empty())
            .collect();
        let status_line = StatusLine::from_str(&lines[0]).unwrap();
        let mut headers = HashMap::<String, String>::new();
        for line in lines.iter().skip(1) {
            let header = line.split(':').collect::<Vec<&str>>();
            if header.len() != 2 {
                let error = RequestParseError::HeaderParseFail(line.into());
                return Err(error);
            }
            let field_name = header[0];
            let field_value = header[1].trim();
            headers.insert(field_name.into(), field_value.into());
        }
        let req = Request::new(
            status_line.method,
            status_line.request_target,
            status_line.http_version,
            Headers::new(headers),
        );
        Ok(req)
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
