#[cfg(test)]
mod tests;

/// Gaurds actions against spam by requiring a PoW [challenge](Toll) to be solved before proceeding.
pub trait Tollkeeper {
    /// Checks if [Suspect] [matches description](Description::matches) and has to [pay a toll](Toll) before proceeding with it's
    /// action.
    ///
    /// Returns [Option::None] and calls ```on_access``` if [Suspect] is permitted or [Toll]
    /// to be paid before being able to try again.
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

    /// Defines which [suspects](Suspect) to look out for and how to proceed with them. Priority is
    /// based on order, meaning the first [Order] that explicitly [grants](AccessPolicy::Whitelist) or [denies](AccessPolicy::Blacklist) access will be
    /// executed.
    pub fn orders(&self) -> &Vec<Order> {
        &self.orders
    }

    /// Examine [Suspect] and check if it has to pay a [Toll]
    pub fn pass(&self, suspect: &dyn Suspect) -> Option<Toll> {
        for order in &self.orders {
            let exam = order.examine(suspect);
            if exam.access_granted {
                return Option::None;
            }
            if exam.toll.is_some() {
                return exam.toll;
            }
        }
        Option::None
    }
}

/// Defines if [gates](Gate) suspects are allowed or denied on matching [Description]
/// [AccessPolicy::Whitelist]
#[derive(Debug, PartialEq, Eq)]
pub enum AccessPolicy {
    Whitelist,
    Blacklist,
}

/// Defines operational standards for a [Gate]
pub struct Order {
    descriptions: Vec<Box<dyn Description>>,
    access_policy: AccessPolicy,
    toll_declaration: Box<dyn Declaration>,
}

impl Order {
    pub fn new(
        descriptions: Vec<Box<dyn Description>>,
        access_policy: AccessPolicy,
        toll_declaration: Box<dyn Declaration>,
    ) -> Self {
        Self {
            descriptions,
            access_policy,
            toll_declaration,
        }
    }

    fn examine(&self, suspect: &dyn Suspect) -> Examination {
        let matches_description = self.is_match(suspect);
        let require_toll = (matches_description && self.access_policy == AccessPolicy::Blacklist)
            || (!matches_description && self.access_policy == AccessPolicy::Whitelist);
        let toll = if require_toll {
            Option::Some(self.toll_declaration.declare())
        } else {
            Option::None
        };
        let access_granted = toll.is_none() && matches_description;
        Examination::new(toll, access_granted)
    }

    fn is_match(&self, suspect: &dyn Suspect) -> bool {
        self.descriptions.iter().any(|d| d.matches(suspect))
    }
}

/// Defines what kind of [Suspect] the [Tollkeeper] is looking out for
pub trait Description {
    fn matches(&self, suspect: &dyn Suspect) -> bool;
}

/// Information about the source trying to access the resource
pub trait Suspect {
    fn client_ip(&self) -> &str;
    fn user_agent(&self) -> &str;
    fn target_host(&self) -> &str;
    fn target_path(&self) -> &str;
}

struct Examination {
    toll: Option<Toll>,
    access_granted: bool,
}

impl Examination {
    fn new(toll: Option<Toll>, access_granted: bool) -> Self {
        Self {
            toll,
            access_granted,
        }
    }
}

/// Factory for creating [Toll] challenges
pub trait Declaration {
    fn declare(&self) -> Toll;
}

/// A Proof-of-Work challenge to be solved before being granted access
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Toll {
    challenge: ChallengeAlgorithm,
    seed: String,
    difficulty: u8,
}

impl Toll {
    pub fn new(challenge: ChallengeAlgorithm, seed: String, difficulty: u8) -> Self {
        Self {
            challenge,
            seed,
            difficulty,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ChallengeAlgorithm {
    SHA1,
    SHA256,
    SHA3,
}
