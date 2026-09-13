//! What a mechanic produces, and the invariant every one of them must hold.

use crate::state::State;

/// One move: what it costs, and where it might leave you.
#[derive(Debug, Clone)]
pub struct Action {
    pub label: String,
    /// In whatever unit the objective is minimising.
    pub cost: f64,
    /// Probability and resulting state. Must sum to 1.
    pub outcomes: Vec<(f64, State)>,
    /// Which mechanic offered this, for the explanation.
    pub method: &'static str,
}

impl Action {
    /// Build an action, collapsing duplicate outcomes and checking Rule 03.
    ///
    /// Two mechanics in three will produce the same resulting state twice - a
    /// roll that misses in two different ways lands in the same place. Summing
    /// those is not an optimisation: leaving them separate inflates the state
    /// count and makes the solver do the same work twice.
    #[must_use]
    pub fn new(
        label: impl Into<String>,
        cost: f64,
        method: &'static str,
        outcomes: Vec<(f64, State)>,
    ) -> Self {
        let outcomes = collapse(outcomes);
        debug_assert!(
            is_distribution(&outcomes),
            "outcomes do not sum to 1: {outcomes:?}"
        );
        Self {
            label: label.into(),
            cost,
            outcomes,
            method,
        }
    }

    /// Probability of staying in the same state after this action.
    #[must_use]
    pub fn p_stay(&self, from: State) -> f64 {
        self.outcomes
            .iter()
            .filter(|&&(_, s)| s == from)
            .map(|&(p, _)| p)
            .sum()
    }
}

/// Sum the probabilities of duplicate resulting states.
#[must_use]
pub fn collapse(outcomes: Vec<(f64, State)>) -> Vec<(f64, State)> {
    let mut out: Vec<(f64, State)> = Vec::with_capacity(outcomes.len());
    for (p, s) in outcomes {
        if p <= 0.0 {
            continue; // if can't happen, don't include it in the pool
        }
        if let Some(slot) = out.iter_mut().find(|(_, existing)| *existing == s) {
            slot.0 += p;
        } else {
            out.push((p, s));
        }
    }
    out
}

/// RULE 03: sums to one, nothing negative, not empty.
#[must_use]
pub fn is_distribution(outcomes: &[(f64, State)]) -> bool {
    if outcomes.is_empty() {
        return false;
    }
    if outcomes.iter().any(|&(p, _)| p < 0.0 || !p.is_finite()) {
        return false;
    }
    let sum: f64 = outcomes.iter().map(|&(p, _)| p).sum();
    (sum - 1.0).abs() < 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_states_are_summed() {
        let s = State::default();
        let collapsed = collapse(vec![(0.25, s), (0.75, s)]);
        assert_eq!(collapsed.len(), 1);
        assert!((collapsed[0].0 - 1.0).abs() < 1e-12);
    }

    #[test]
    fn impossible_outcomes_are_dropped() {
        let a = State::default();
        let b = State::default().with_rarity(1);
        let collapsed = collapse(vec![(1.0, a), (0.0, b)]);
        assert_eq!(collapsed.len(), 1);
    }

    #[test]
    fn an_empty_pool_is_not_a_distribution() {
        assert!(!is_distribution(&[]));
    }
}
