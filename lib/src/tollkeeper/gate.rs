use super::*;

/// Specifies a condition on which a gate should trigger
///
/// E.g. A [Gate] for allowing/blocking User-Agents could search for a pattern in
/// the User-Agent header and trigger if it matches.
pub trait Gate {
    /// True if [Suspect] meets conditions set for gate
    fn matches_description(&self, suspect: &dyn Suspect) -> bool;
}
