use std::{error::Error, fmt::Display, io::Read};

use crate::http;
#[cfg(test)]
mod tests;

pub trait ReadJson {
    fn read_json<T>(&mut self) -> Result<T, ReadJsonError>
    where
        T: for<'de> serde::Deserialize<'de>;
}

impl ReadJson for http::Request {
    fn read_json<T>(&mut self) -> Result<T, ReadJsonError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
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
            match serde_json::from_value(json) {
                Ok(d) => Ok(d),
                Err(e) => Err(ReadJsonError::InvalidJsonData(e.to_string())),
            }
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
    InvalidJsonData(String),
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
            ReadJsonError::InvalidJsonData(e) => write!(f, "Invalid JSON data: {e}"),
        }
    }
}
