use super::*;

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

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn orders(&self) -> &Vec<Order> {
        &self.orders
    }

    pub fn pass(&self, suspect: &dyn Suspect) -> Option<Challenge> {
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

    pub fn investigate(&self, suspect: &dyn Suspect) -> Option<Challenge> {
        let is_match = self.is_match(suspect);
        let require_challenge = (is_match && self.status == GateStatus::Blacklist)
            || (!is_match && self.status == GateStatus::Whitelist);
        if require_challenge {
            Option::Some(Challenge::new("challenge"))
        } else {
            Option::None
        }
    }

    fn is_match(&self, suspect: &dyn Suspect) -> bool {
        self.descriptions.iter().any(|d| d.matches(suspect))
    }
}
