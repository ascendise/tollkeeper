/// Default implementation of the [Tollkeeper]. Uses a list of defined forward hosts and filter rules
/// to [Challenge] access for [Host] endpoints.
pub struct TollkeeperImpl {
    hosts: Vec<Host>,
}

impl Tollkeeper for TollkeeperImpl {
    fn access<TRequest>(req: Request, on_access: &dyn Fn(TRequest)) -> Option<Challenge> {
        Option::None
    }
}

/// Gaurds actions against spam by requiring a PoW [Challenge] to be solved before proceeding.
pub trait Tollkeeper {
    fn access<TRequest>(req: Request, on_access: &dyn Fn(TRequest)) -> Option<Challenge>;
}

/// Target machines guarded by [TollkeeperImpl].
pub struct Host {
    /// Location of the [Host] to be protected. E.g. https://172.0.0.1:3000
    base_url: String,
    /// Defines if a tripped [Trap] [challenges](Operation::Challenge) or [allows](Operation::Allow) access
    ///
    /// E.g. to challenge all access except for API endpoints, you define [Host::on_trap] to
    /// be [Operation::Allow] and set a [Trap] for requests to /api/*.
    on_trap: Operation,
    /// Defines the [traps](Trap) that guard a [Host]
    traps: Vec<Box<dyn Trap>>,
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

/// A Proof-of-Work challenge to be solved before being granted access
pub struct Challenge {
    name: String,
}
