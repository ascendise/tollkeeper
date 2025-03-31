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
