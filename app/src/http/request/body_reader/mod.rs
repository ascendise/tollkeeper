use std::{error::Error, fmt::Display};

use crate::http;
#[cfg(test)]
pub mod tests;

pub trait ReadJson {
    fn read_json(&mut self) -> Result<serde_json::Value, ReadJsonError>;
}

impl ReadJson for http::Request {
    fn read_json(&mut self) -> Result<serde_json::Value, ReadJsonError> {
        //let content_length = self.headers().content_length();
        let content_type = self
            .headers
            .content_type()
            .ok_or(ReadJsonError::MismatchedContentType("".into()))?;
        if content_type != "application/json" {
            let err = ReadJsonError::MismatchedContentType(content_type.into());
            return Err(err);
        }
        let body = self.body().as_mut().ok_or(ReadJsonError::Unknown)?;
        let mut json = String::new();
        body.read_to_string(&mut json)
            .or(Err(ReadJsonError::Unknown))?;
        let json = serde_json::from_str(&json).or(Err(ReadJsonError::FailedParsing))?;
        Ok(json)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReadJsonError {
    MismatchedContentType(String),
    FailedParsing,
    Unknown,
}
impl Error for ReadJsonError {}
impl Display for ReadJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error during json reading") //TODO: Better error messages
    }
}
