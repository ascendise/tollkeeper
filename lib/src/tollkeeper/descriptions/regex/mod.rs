#[cfg(test)]
mod tests;

use ::regex::{Error, Regex};

use super::*;

/// Checks property on [Suspect] using regex
/// Also allows on negative lookaheads on the whole regex
pub struct RegexDescription {
    key: String,
    regex: Regex,
    negative_lookahead: bool,
}

impl RegexDescription {
    /// Create a description that matches the specific regex.
    /// E.g. to find specific ips
    pub fn new(key: impl Into<String>, regex: &str) -> Result<Self, Error> {
        let description = Self {
            key: key.into(),
            regex: Regex::new(regex)?,
            negative_lookahead: false,
        };
        Ok(description)
    }

    /// Create a description that matches the specific regex but uses negative lookahead (not
    /// supported by rust regex engine).
    /// E.g. to match everything expect a specific IP
    pub fn negative_lookahead(key: impl Into<String>, regex: &str) -> Result<Self, Error> {
        let description = Self {
            key: key.into(),
            regex: Regex::new(regex)?,
            negative_lookahead: true,
        };
        Ok(description)
    }
}

impl Description for RegexDescription {
    fn matches(&self, suspect: &Suspect) -> bool {
        let map: HashMap<String, String> = suspect.into();
        match map.get(&self.key) {
            Some(v) => self.regex.is_match(v) || self.negative_lookahead,
            None => false,
        }
    }
}
