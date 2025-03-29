/// Default implementation of the [Tollkeeper]. Uses a list of defined forward hosts and filter rules
/// to [Challenge] access for [Host] endpoints.
pub struct TollkeeperImpl {
    hosts: Vec<Host>,
}

impl TollkeeperImpl {
    pub fn new(hosts: Vec<Host>) -> Self {
        Self { hosts }
    }
}

impl Tollkeeper for TollkeeperImpl {
    fn access(self: &Self, req: Request, on_access: impl FnMut(())) -> Option<Challenge> {
        Option::None
    }
}

/// Gaurds actions against spam by requiring a PoW [Challenge] to be solved before proceeding.
pub trait Tollkeeper {
    fn access(self: &Self, req: Request, on_access: impl FnMut(())) -> Option<Challenge>;
}

/// Target machines guarded by [TollkeeperImpl].
pub struct Host {
    base_url: String,
    on_trap: Operation,
    traps: Vec<Box<dyn Trap>>,
}

impl Host {
    pub fn new(base_url: impl Into<String>, on_trap: Operation, traps: Vec<Box<dyn Trap>>) -> Self {
        Self {
            base_url: base_url.into(),
            on_trap,
            traps,
        }
    }

    /// Location of the [Host] to be protected. E.g. https://172.0.0.1:3000
    pub fn base_url(self: &Self) -> &str {
        &self.base_url
    }

    /// Defines if a tripped [Trap] [challenges](Operation::Challenge) or [allows](Operation::Allow) access
    ///
    /// E.g. to challenge all access except for API endpoints, you define [Host::on_trap] to
    /// be [Operation::Allow] and set a [Trap] for requests to /api/*.
    pub fn on_trap(self: &Self) -> &Operation {
        &self.on_trap
    }

    /// Defines the [traps](Trap) that guard a [Host]
    pub fn traps(self: &Self) -> &Vec<Box<dyn Trap>> {
        &self.traps
    }
}

/// Defines if a trap acts as a defense [Operation::Challenge] or as a gateway [Operation::Allow]
pub enum Operation {
    Allow,
    Challenge,
}

/// Condition to be met for the [Tollkeeper] to [capture attacks](Operation::Challenge) or [allow whitelisted](Operation::Allow)
/// [requests](Request).
pub trait Trap {
    /// True if [Request] meets the condition
    fn is_trapped(&self, req: Request) -> bool;
}

/// Information about the incoming [requests](Request), used by [traps](Trap) to trigger
pub struct Request {
    client_ip: String,
    user_agent: String,
    target_host: String,
    target_path: String,
}

impl Request {
    pub fn new(
        client_ip: impl Into<String>,
        user_agent: impl Into<String>,
        target_host: impl Into<String>,
        target_path: impl Into<String>,
    ) -> Self {
        Self {
            client_ip: client_ip.into(),
            user_agent: user_agent.into(),
            target_host: target_host.into(),
            target_path: target_path.into(),
        }
    }
}

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
