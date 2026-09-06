//! Which modifiers a base item can roll, and how heavily.

use ssf_data::{BaseRecord, GameData, GroupId, ModId, Slot};

/// The modifiers available for one slot on one base at one item level.
#[derive(Debug, Clone, Default)]
pub struct Pool {
    /// Modifier and its weight. Weights are always > 0 in here.
    pub entries: Vec<(ModId, i32)>,
    /// Sum of the weights, cached because every chance divides by it.
    pub total: i64,
}

impl Pool {
    /// Probability that one roll lands on `id`.
    #[must_use]
    pub fn chance_of(&self, id: ModId) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        let w = self
            .entries
            .iter()
            .find(|&&(m, _)| m == id)
            .map_or(0, |&(_, w)| w);
        f64::from(w) / self.total as f64
    }

    /// "One roll in N". Infinite when the modifier cannot roll at all.
    #[must_use]
    pub fn one_in(&self, id: ModId) -> f64 {
        let p = self.chance_of(id);
        if p <= 0.0 { f64::INFINITY } else { 1.0 / p }
    }

    /// The same pool with one modifier group removed.
    ///
    /// Two modifiers from the same group cannot coexist, so once one is on the
    /// item its whole group leaves the pool for later rolls.
    #[must_use]
    pub fn without_group(&self, data: &GameData, group: GroupId) -> Self {
        let entries: Vec<(ModId, i32)> = self
            .entries
            .iter()
            .filter(|&&(id, _)| data.modifier(id).group != group)
            .copied()
            .collect();
        let total = entries.iter().map(|&(_, w)| i64::from(w)).sum();
        Self { entries, total }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Every modifier that can roll in `slot` on `base` at `ilvl`.
#[must_use]
pub fn build(data: &GameData, base: &BaseRecord, ilvl: u32, slot: Slot) -> Pool {
    let entries: Vec<(ModId, i32)> = data
        .modifiers()
        .filter(|m| m.slot == slot)
        .filter(|m| m.required_level <= ilvl)
        .filter(|m| !m.is_essence_only)
        .filter_map(|m| {
            let w = m.weight_for(&base.tags);
            (w > 0).then_some((m.id, w))
        })
        .collect();

    let total = entries.iter().map(|&(_, w)| i64::from(w)).sum();
    Pool { entries, total }
}
