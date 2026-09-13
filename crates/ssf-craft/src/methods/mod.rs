//! Every crafting mechanic the solver knows about.
//!
//! A method is one mechanic that can produce actions from a state. The solver
//! never sees them; it sees the actions they emit. Adding a mechanic is one new
//! module and one line in `all()`, and nothing about the search changes.

pub mod currency;

use crate::constants::Confidence;
use crate::context::CraftContext;
use crate::mechanics::Action;
use crate::options::{Family, SolveOptions};
use crate::state::State;

/// What the UI needs to describe a method, and the switch that gates it.
#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub key: &'static str,
    pub label: &'static str,
    pub family: Family,
    pub summary: &'static str,
    pub currencies: &'static [&'static str],
    pub confidence: Confidence,
    pub note: &'static str,
}

pub trait Method {
    fn info(&self) -> MethodInfo;
    /// Append every action this mechanic offers from `state`.
    fn build(&self, ctx: &CraftContext, state: State, out: &mut Vec<Action>);
}

/// Every method, in a fixed order.
#[must_use]
pub fn all() -> Vec<Box<dyn Method>> {
    let mut methods: Vec<Box<dyn Method>> = Vec::new();
    currency::register(&mut methods);
    // M9 adds eleven more lines here, and nothing else changes.
    methods
}

/// The methods a set of options allows, in registry order.
#[must_use]
pub fn registry(options: &SolveOptions) -> Vec<Box<dyn Method>> {
    all()
        .into_iter()
        .filter(|m| options.allows(m.info().family))
        .collect()
}

/// Every method, for the assumptions page.
#[must_use]
pub fn catalogue() -> Vec<MethodInfo> {
    all().iter().map(|m| m.info()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_method_has_a_unique_key() {
        let catalogue = catalogue();
        let mut keys: Vec<&str> = catalogue.iter().map(|i| i.key).collect();
        let before = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), before, "two methods share a key");
    }

    #[test]
    fn every_method_obeys_its_own_switch() {
        for info in catalogue() {
            let mut options = SolveOptions::default();
            options.set(info.family, false);
            assert!(
                !registry(&options).iter().any(|m| m.info().key == info.key),
                "{} survived its own family being switched off",
                info.key
            );
        }
    }
}
