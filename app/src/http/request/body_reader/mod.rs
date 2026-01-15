use std::{error::Error, fmt::Display, io::Read};

use crate::http;
#[cfg(test)]
mod tests;

pub trait ReadJson {
    fn read_json(&mut self) -> Result<serde_json::Value, ReadJsonError>;
}

impl ReadJson for http::Request {
    fn read_json(&mut self) -> Result<serde_json::Value, ReadJsonError> {
        let content_type = self
            .headers
            .content_type()
            .ok_or(ReadJsonError::MismatchedContentType("".into()))?;
        if content_type != "application/json" {
            let err = ReadJsonError::MismatchedContentType(content_type.into());
            return Err(err);
        }
        let content_length = self.headers().content_length().unwrap_or(0);
        let mut json = vec![0; content_length];
        if let http::Body::Buffer(buffer) = self.body_mut() {
            buffer
                .read_exact(&mut json)
                .or(Err(ReadJsonError::IoError))?;
            let json: serde_json::Value =
                serde_json::from_slice(json.as_slice()).or(Err(ReadJsonError::NonJsonData))?;
            Ok(json)
        } else {
            Err(ReadJsonError::NonJsonData)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReadJsonError {
    MismatchedContentType(String),
    NonJsonData,
    IoError,
}
impl Error for ReadJsonError {}
impl Display for ReadJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadJsonError::MismatchedContentType(content_type) => write!(
                f,
                "Expected 'application/json' content-type but got '{content_type}'"
            ),
            ReadJsonError::NonJsonData => write!(f, "Data is not valid JSON!"),
            ReadJsonError::IoError => write!(f, "Failure reading request stream!"),
        }
    }
}
