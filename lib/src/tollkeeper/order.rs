use super::*;

/// Defines operational standards for a [Gate]
pub struct Order {
    descriptions: Vec<Box<dyn Description>>,
    status: GateStatus,
}

impl Order {
    pub fn new(descriptions: Vec<Box<dyn Description>>, status: GateStatus) -> Self {
        Self {
            descriptions,
            status,
        }
    }

    pub fn investigate(&self, suspect: &dyn Suspect) -> Option<Toll> {
        let is_match = self.is_match(suspect);
        let require_challenge = (is_match && self.status == GateStatus::Blacklist)
            || (!is_match && self.status == GateStatus::Whitelist);
        if require_challenge {
            Option::Some(Toll::new("challenge"))
        } else {
            Option::None
        }
    }

    fn is_match(&self, suspect: &dyn Suspect) -> bool {
        self.descriptions.iter().any(|d| d.matches(suspect))
    }
}
