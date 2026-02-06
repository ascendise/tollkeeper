use std::{error, fmt};

pub mod headers;
pub mod request;
pub mod response;
#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseError {
    RequestLine,
    StatusLine,
    Header,
    Body,
}
impl error::Error for ParseError {}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::RequestLine => write!(f, "Invalid request line"),
            ParseError::StatusLine => write!(f, "Invalid status line"),
            ParseError::Header => write!(f, "Invalid header line"),
            ParseError::Body => write!(f, "Invalid body"),
        }
    }
}
