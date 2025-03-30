#[cfg(test)]
mod tests;

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
    fn access<TRequest: Request>(
        self: &Self,
        req: &mut TRequest,
        on_access: impl Fn(&mut TRequest),
    ) -> Option<Challenge> {
        let host = match self
            .hosts
            .iter()
            .find(|h| req.target_host().contains(&h.base_url))
        {
            Option::Some(h) => h,
            Option::None => return Option::None,
        };
        if host.traps().iter().any(|t| t.is_trapped(req)) {
            return Option::Some(Challenge::new("challenge"));
        }
        on_access(req);
        Option::None
    }
}

/// Gaurds actions against spam by requiring a PoW [Challenge] to be solved before proceeding.
pub trait Tollkeeper {
    fn access<TRequest: Request>(
        self: &Self,
        req: &mut TRequest,
        on_access: impl Fn(&mut TRequest),
    ) -> Option<Challenge>;
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
#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    Allow,
    Challenge,
}

/// Condition to be met for the [Tollkeeper] to [capture attacks](Operation::Challenge) or [allow whitelisted](Operation::Allow)
/// [requests](Request).
pub trait Trap {
    /// True if [Request] meets the condition
    fn is_trapped(&self, req: &dyn Request) -> bool;
}

/// Information about the incoming [requests](Request), used by [traps](Trap) to trigger
pub trait Request {
    fn client_ip(self: &Self) -> &str;
    fn user_agent(self: &Self) -> &str;
    fn target_host(self: &Self) -> &str;
    fn target_path(self: &Self) -> &str;
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
