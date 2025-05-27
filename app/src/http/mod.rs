use std::{fmt::Display, str::FromStr};

use indexmap::IndexMap;

#[cfg(test)]
mod tests;

mod request;
mod response;

/// Key-Value collection with case-insensitve access
#[derive(Debug, PartialEq, Eq)]
pub struct Headers {
    headers: IndexMap<String, Header>,
}
impl Headers {
    pub fn new(headers: IndexMap<String, String>) -> Self {
        let headers = Self::map_headers_case_insensitive(headers);
        Self { headers }
    }

    fn map_headers_case_insensitive(headers: IndexMap<String, String>) -> IndexMap<String, Header> {
        headers
            .iter()
            .map(|(k, v)| {
                (
                    k.to_ascii_lowercase(),
                    Header {
                        original_key: k.into(),
                        value: v.into(),
                    },
                )
            })
            .collect()
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        let key = key.to_ascii_lowercase();
        match self.headers.get(&key) {
            Some(v) => Some(&v.value),
            None => None,
        }
    }
}
impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for header in &self.headers {
            write!(f, "{}: {}\r\n", header.1.original_key, header.1.value)?
        }
        Ok(())
    }
}
#[derive(Debug, PartialEq, Eq)]
struct Header {
    original_key: String,
    value: String,
}
