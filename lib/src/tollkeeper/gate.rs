use super::*;

/// Defines the target machine and which [suspects](Suspect) are allowed or not
pub struct Gate {
    destination: String,
    orders: Vec<Order>,
}

impl Gate {
    pub fn new(destination: String, orders: Vec<Order>) -> Self {
        Self {
            destination,
            orders,
        }
    }

    /// Base URL of the target host machine. E.g. <https://example.com:3000>
    pub fn destination(&self) -> &str {
        &self.destination
    }

    /// Defines which [suspects](Suspect) to look out for and how to proceed with them
    pub fn orders(&self) -> &Vec<Order> {
        &self.orders
    }

    /// Examine [Suspect] and check if he has to pay a toll
    pub fn pass(&self, suspect: &dyn Suspect) -> Option<Toll> {
        for order in &self.orders {
            let exam = order.investigate(suspect);
            match exam {
                Option::Some(c) => return Option::Some(c),
                Option::None => continue,
            };
        }
        Option::None
    }
}

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
