#[cfg(test)]
mod tests;

/// Guards actions against spam by requiring a PoW [challenge](Toll) to be solved before proceeding.
pub trait Tollkeeper {
    /// Checks if [Suspect] [matches description](Description::matches) and has to [pay a toll](Toll) before proceeding with it's
    /// action.
    ///
    /// Returns [Option::None] and calls ```on_access``` if [Suspect] is permitted or [Toll]
    /// to be paid before being able to try again.
    fn guarded_access<T>(
        &self,
        suspect: &Suspect,
        request: &mut T,
        on_access: impl Fn(&mut T),
    ) -> Option<Toll>;
}
///
/// Default implementation of the [Tollkeeper].
pub struct TollkeeperImpl {
    gates: Vec<Gate>,
}

impl TollkeeperImpl {
    pub fn new(gates: Vec<Gate>) -> Result<Self, ConfigError> {
        if gates.is_empty() {
            Result::Err(ConfigError::new(
                String::from("gates"),
                String::from("No gates defined. Tollkeeper has nothing to protect!"),
            ))
        } else {
            Result::Ok(Self { gates })
        }
    }

    fn find_gate(&self, suspect: &Suspect) -> Option<&Gate> {
        self.gates
            .iter()
            .find(|g| g.destination == suspect.destination)
            .or(Option::None)
    }
}

/// Sends [Suspect] through matching [Gate] and  requests a [Toll] if necessary
impl Tollkeeper for TollkeeperImpl {
    fn guarded_access<T>(
        &self,
        suspect: &Suspect,
        request: &mut T,
        on_access: impl Fn(&mut T),
    ) -> Option<Toll> {
        let gate = match self.find_gate(suspect) {
            Option::Some(g) => g,
            Option::None => return Option::None,
        };
        let result = gate.pass(suspect);
        match result {
            Option::Some(g) => Option::Some(g),
            Option::None => {
                on_access(request);
                Option::None
            }
        }
    }
}

/// Defines the target machine and which [suspects](Suspect) are allowed or not
pub struct Gate {
    destination: Destination,
    orders: Vec<Order>,
}

impl Gate {
    pub fn new(destination: Destination, orders: Vec<Order>) -> Result<Self, ConfigError> {
        if orders.is_empty() {
            Result::Err(ConfigError::new(
                "orders",
                "You need to define at least one order for the gate!",
            ))
        } else {
            Result::Ok(Self {
                destination,
                orders,
            })
        }
    }

    /// Target machine destination
    pub fn destination(&self) -> &Destination {
        &self.destination
    }

    /// Defines which [suspects](Suspect) to look out for and how to proceed with them. Priority is
    /// based on order, meaning the first [Order] that explicitly [grants](AccessPolicy::Whitelist) or [denies](AccessPolicy::Blacklist) access will be
    /// executed.
    pub fn orders(&self) -> &Vec<Order> {
        &self.orders
    }

    /// Examine [Suspect] and check if it has to pay a [Toll]
    fn pass(&self, suspect: &Suspect) -> Option<Toll> {
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

/// Defines conditional process for a [Gate]
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

    fn examine(&self, suspect: &Suspect) -> Examination {
        let matches_description = self.is_match(suspect);
        let require_toll = (matches_description && self.access_policy == AccessPolicy::Blacklist)
            || (!matches_description && self.access_policy == AccessPolicy::Whitelist);
        let toll = if require_toll && !self.has_paid(suspect) {
            Option::Some(self.toll_declaration.declare())
        } else {
            Option::None
        };
        let access_granted = toll.is_none() && matches_description;
        Examination::new(toll, access_granted)
    }

    fn is_match(&self, suspect: &Suspect) -> bool {
        self.descriptions.iter().any(|d| d.matches(suspect))
    }

    fn has_paid(&self, suspect: &Suspect) -> bool {
        match suspect.payment() {
            Option::Some(p) => self.toll_declaration.pay(&p),
            Option::None => false,
        }
    }
}

/// Examines [Suspect] for a defined condition like matching IP/User-Agent/...
pub trait Description {
    fn matches(&self, suspect: &Suspect) -> bool;
}

/// Information about the source trying to access the resource
pub struct Suspect {
    client_ip: String,
    user_agent: String,
    destination: Destination,
    payment: Option<Payment>,
}

impl Suspect {
    pub fn new(
        client_ip: impl Into<String>,
        user_agent: impl Into<String>,
        destination: Destination,
    ) -> Self {
        Self {
            client_ip: client_ip.into(),
            user_agent: user_agent.into(),
            destination,
            payment: Option::None,
        }
    }
    pub fn with_payment(
        client_ip: impl Into<String>,
        user_agent: impl Into<String>,
        destination: Destination,
        payment: Payment,
    ) -> Self {
        Self {
            client_ip: client_ip.into(),
            user_agent: user_agent.into(),
            destination,
            payment: Option::Some(payment),
        }
    }

    pub fn client_ip(&self) -> &str {
        &self.client_ip
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub fn destination(&self) -> &Destination {
        &self.destination
    }

    pub fn payment(&self) -> &Option<Payment> {
        &self.payment
    }
}

#[derive(PartialEq, Eq)]
pub struct Destination {
    base_url: String,
    port: u16,
    path: String,
}

impl Destination {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            port: 80,
            path: String::from("/"),
        }
    }

    pub fn new_with_details(
        base_url: impl Into<String>,
        port: u16,
        path: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            port,
            path: path.into(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn port(&self) -> u16 {
        self.port
    }
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

/// Creates and verifies [tolls](Toll)
pub trait Declaration {
    fn declare(&self) -> Toll;
    fn pay(&self, payment: &Payment) -> bool;
}

/// A Proof-of-Work challenge to be solved before being granted access
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Toll {
    challenge: ChallengeAlgorithm,
    seed: String,
    difficulty: u8,
}

impl Toll {
    pub fn new(challenge: ChallengeAlgorithm, seed: impl Into<String>, difficulty: u8) -> Self {
        Self {
            challenge,
            seed: seed.into(),
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

/// Solution for solved [challenge](Toll)
pub struct Payment {
    toll: Toll,
    value: String,
}

impl Payment {
    /// Creates a payment containing the [challenge][Toll] to be solved and the calculated hash
    pub fn new(toll: Toll, value: impl Into<String>) -> Self {
        Self {
            toll,
            value: value.into(),
        }
    }

    pub fn toll(&self) -> &Toll {
        &self.toll
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Return this error when there are problems during creation of the [Tollkeeper] or
/// it's subentities caused by wrong init arguments
#[derive(Debug, Eq, Clone)]
pub struct ConfigError {
    key: String,
    description: String,
}

impl ConfigError {
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
        }
    }

    /// Property that caused the error
    pub fn key(&self) -> &str {
        &self.key
    }

    /// User-friendly message describing what is wrong with the configuration
    /// Not part of equality comparison
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl PartialEq for ConfigError {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }

    fn ne(&self, other: &Self) -> bool {
        self.key != other.key
    }
}
