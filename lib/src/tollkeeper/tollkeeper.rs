use super::*;

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
    gates: Vec<Gate>,
}

impl TollkeeperImpl {
    pub fn new(gates: Vec<Gate>) -> Self {
        Self { gates }
    }

    fn find_gate(&self, destination: &str) -> Option<&Gate> {
        self.gates
            .iter()
            .find(|g| g.destination() == destination)
            .or(Option::None)
    }
}

impl Tollkeeper for TollkeeperImpl {
    fn guarded_access<TSuspect: Suspect>(
        &self,
        suspect: &mut TSuspect,
        on_access: impl Fn(&mut TSuspect),
    ) -> Option<Challenge> {
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
