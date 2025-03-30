#[cfg(test)]
mod tests;

/// Default implementation of the [Tollkeeper]. Uses a list of destination machines, each with
/// their own gates to [Challenge] access for [Destination] endpoints.
pub struct TollkeeperImpl {
    destinations: Vec<Destination>,
}

impl TollkeeperImpl {
    pub fn new(hosts: Vec<Destination>) -> Self {
        Self {
            destinations: hosts,
        }
    }
}

impl Tollkeeper for TollkeeperImpl {
    fn guarded_access<TSuspect: Suspect>(
        &self,
        suspect: &mut TSuspect,
        on_access: impl Fn(&mut TSuspect),
    ) -> Option<Challenge> {
        let destination = match self
            .destinations
            .iter()
            .find(|h| suspect.target_host().contains(&h.base_url))
        {
            Option::Some(h) => h,
            Option::None => return Option::None,
        };
        let matches_description = destination
            .gates()
            .iter()
            .any(|t| t.matches_description(suspect));
        if (matches_description && destination.gate_status == GateStatus::Blacklist)
            || (!matches_description && destination.gate_status == GateStatus::Whitelist)
        {
            return Option::Some(Challenge::new("challenge"));
        }
        on_access(suspect);
        Option::None
    }
}

/// Gaurds actions against spam by requiring a PoW [Challenge] to be solved before proceeding.
pub trait Tollkeeper {
    /// Checks if [Suspect] [matches description](Gate::matches_description) and has to be [challenged](Challenge) before proceeding with it's
    /// action.
    ///
    /// Returns [Option::None] and calls ```on_access``` if suspect is permitted or [Challenge]
    /// to be solved before being able to try again.
    fn guarded_access<TSuspect: Suspect>(
        &self,
        suspect: &mut TSuspect,
        on_access: impl Fn(&mut TSuspect),
    ) -> Option<Challenge>;
}

/// Target machine guarded by [TollkeeperImpl].
pub struct Destination {
    base_url: String,
    gate_status: GateStatus,
    gates: Vec<Box<dyn Gate>>,
}

impl Destination {
    pub fn new(
        base_url: impl Into<String>,
        gate_status: GateStatus,
        gates: Vec<Box<dyn Gate>>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            gate_status,
            gates,
        }
    }

    /// Location of the host to be protected. E.g. https://172.0.0.1:3000
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Define if (gates)[Gate] are supposed to [allow](GateStatus::Whitelist) or [challenge](GateStatus::Blacklist) if [Suspect] matches the gate
    /// condition
    pub fn gate_status(&self) -> &GateStatus {
        &self.gate_status
    }

    /// Defines the [gates](Gate) that guard a [Destination]
    pub fn gates(&self) -> &Vec<Box<dyn Gate>> {
        &self.gates
    }
}

/// Defines if [gates](Gate) act as a defense [GateStatus::Blacklist] or as a gateway
/// [GateStatus::Whitelist]
#[derive(Debug, PartialEq, Eq)]
pub enum GateStatus {
    Whitelist,
    Blacklist,
}

/// Specifies a condition on which a gate should trigger
///
/// E.g. A [Gate] for allowing/blocking User-Agents could search for a pattern in
/// the User-Agent header and trigger if it matches.
pub trait Gate {
    /// True if [Suspect] meets conditions set for gate
    fn matches_description(&self, suspect: &dyn Suspect) -> bool;
}

/// Information about the source trying to access the resource, read by [gates](Gate) to match
/// descriptions
pub trait Suspect {
    fn client_ip(&self) -> &str;
    fn user_agent(&self) -> &str;
    fn target_host(&self) -> &str;
    fn target_path(&self) -> &str;
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
