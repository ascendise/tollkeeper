use super::*;

/// Defines what kind of [Suspect] the [Tollkeeper] is looking out for  
pub trait Description {
    fn matches(&self, suspect: &dyn Suspect) -> bool;
}
