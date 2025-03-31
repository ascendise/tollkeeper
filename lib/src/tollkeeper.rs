#[cfg(test)]
mod tests;

mod challenge;
mod destination;
mod gate;
mod gate_status;
mod suspect;

pub use self::challenge::*;
pub use self::destination::*;
pub use self::gate::*;
pub use self::gate_status::*;
pub use self::suspect::*;

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

/// Default implementation of the [Tollkeeper]. Uses a list of destination machines, each with
/// their own gates to [Challenge] access for [Destination] endpoints.
pub struct TollkeeperImpl {
    destinations: Vec<Destination>,
}

impl TollkeeperImpl {
    pub fn new(destinations: Vec<Destination>) -> Self {
        Self { destinations }
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
            .find(|d| suspect.target_host().contains(&d.base_url()))
        {
            Option::Some(h) => h,
            Option::None => return Option::None,
        };
        let matches_description = destination
            .gates()
            .iter()
            .any(|t| t.matches_description(suspect));
        if (matches_description && *destination.gate_status() == GateStatus::Blacklist)
            || (!matches_description && *destination.gate_status() == GateStatus::Whitelist)
        {
            return Option::Some(Challenge::new("challenge"));
        }
        on_access(suspect);
        Option::None
    }
}
