#[cfg(test)]
mod tests;

/// Gaurds actions against spam by requiring a PoW [challenge](Toll) to be solved before proceeding.
pub trait Tollkeeper {
    /// Checks if [Suspect] [matches description](Description::matches) and has to [pay a toll](Toll) before proceeding with it's
    /// action.
    ///
    /// Returns [Option::None] and calls ```on_access``` if suspect is permitted or [Toll]
    /// to be payed before being able to try again.
    fn guarded_access<TSuspect: Suspect>(
        &self,
        suspect: &mut TSuspect,
        on_access: impl Fn(&mut TSuspect),
    ) -> Option<Toll>;
}

/// Default implementation of the [Tollkeeper].
pub struct TollkeeperImpl {
    gates: Vec<Gate>,
}

impl TollkeeperImpl {
    pub fn new(gates: Vec<Gate>) -> Self {
        Self { gates }
    }

    fn find_gate(&self, target_host: &str) -> Option<&Gate> {
        self.gates
            .iter()
            .find(|g| g.destination() == target_host)
            .or(Option::None)
    }
}
/// Sends [Suspect] through matching [Gate] and  requests a [Toll] if necessary
impl Tollkeeper for TollkeeperImpl {
    fn guarded_access<TSuspect: Suspect>(
        &self,
        suspect: &mut TSuspect,
        on_access: impl Fn(&mut TSuspect),
    ) -> Option<Toll> {
        let gate = match self.find_gate(suspect.target_host()) {
            Option::Some(g) => g,
            Option::None => return Option::None,
        };
        let result = gate.pass(suspect);
        match result {
            Option::Some(g) => Option::Some(g),
            Option::None => {
                on_access(suspect);
                Option::None
            }
        }
    }
}

/// Defines what kind of [Suspect] the [Tollkeeper] is looking out for  
pub trait Description {
    fn matches(&self, suspect: &dyn Suspect) -> bool;
}

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

/// Defines if [gates](super::Gate) act as a defense [GateStatus::Blacklist] or as a gateway
/// [GateStatus::Whitelist]
#[derive(Debug, PartialEq, Eq)]
pub enum GateStatus {
    Whitelist,
    Blacklist,
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

/// Information about the source trying to access the resource
pub trait Suspect {
    fn client_ip(&self) -> &str;
    fn user_agent(&self) -> &str;
    fn target_host(&self) -> &str;
    fn target_path(&self) -> &str;
}

/// A Proof-of-Work challenge to be solved before being granted access
#[derive(Debug, Eq, PartialEq)]
pub struct Toll {
    name: String,
}

impl Toll {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
