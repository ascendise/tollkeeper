use super::*;

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
