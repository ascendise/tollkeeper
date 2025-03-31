/// Defines if [gates](super::Gate) act as a defense [GateStatus::Blacklist] or as a gateway
/// [GateStatus::Whitelist]
#[derive(Debug, PartialEq, Eq)]
pub enum GateStatus {
    Whitelist,
    Blacklist,
}
