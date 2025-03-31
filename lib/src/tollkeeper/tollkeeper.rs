use super::*;

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
