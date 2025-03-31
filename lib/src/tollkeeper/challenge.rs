/// A Proof-of-Work challenge to be solved before being granted access
#[derive(Debug, Eq, PartialEq)]
pub struct Challenge {
    name: String,
}

impl Challenge {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
