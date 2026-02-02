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
    pub fn new(
        key: impl Into<String>,
        regex: &str,
        negative_lookahead: bool,
    ) -> Result<Self, Error> {
        let description = Self {
            key: key.into(),
            regex: Regex::new(regex)?,
            negative_lookahead,
        };
        Ok(description)
    }
}

impl Description for RegexDescription {
    fn matches(&self, suspect: &Suspect) -> bool {
        let map: HashMap<String, String> = suspect.into();
        let value = map.get(&self.key).expect("Key does not exist");
        let is_match = self.regex.is_match(value);
        is_match != self.negative_lookahead
    }
}
