//! Turning a pool into the probabilities a solver can use.

use ssf_data::ModId;

use crate::pool::Pool;

/// A pool plus the question being asked of it.
#[derive(Debug, Clone)]
pub struct DrawProfile<'a> {
    pool: &'a Pool,
}

impl<'a> DrawProfile<'a> {
    #[must_use]
    pub fn new(pool: &'a Pool) -> Self {
        Self { pool }
    }

    /// Probability that one roll lands on any modifier in `wanted`.
    #[must_use]
    pub fn p_any(&self, wanted: &[ModId]) -> f64 {
        wanted.iter().map(|&id| self.pool.chance_of(id)).sum()
    }

    /// Probability that one roll lands on exactly `id`.
    #[must_use]
    pub fn p_one(&self, id: ModId) -> f64 {
        self.pool.chance_of(id)
    }

    /// Expected rolls to hit `id` once. Infinite if it cannot roll.
    #[must_use]
    pub fn one_in(&self, id: ModId) -> f64 {
        self.pool.one_in(id)
    }
}
