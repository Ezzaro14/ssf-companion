use ssf_data::{ModId, Slot};

use crate::state::MAX_TARGETS;

/// One modifier the plan is aiming at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub id: ModId,
    pub slot: Slot,
    /// What to print: "+(175-189) to maximum Life".
    pub label: String,
    /// 1 is the best tier.
    pub tier: u32,
    /// How many tiers exist on this base, for "T1/13".
    pub tier_count: u32,
}

/// Up to `MAX_TARGETS` targets, in a fixed order.
#[derive(Debug, Clone, Default)]
pub struct TargetSet {
    targets: Vec<Target>,
}

impl TargetSet {
    /// Fails if given more than `MAX_TARGETS`, because the mask would not fit.
    pub fn new(targets: Vec<Target>) -> Result<Self, String> {
        if targets.len() > MAX_TARGETS {
            return Err(format!(
                "{} targets, but the state holds {MAX_TARGETS}",
                targets.len()
            ));
        }
        Ok(Self { targets })
    }

    /// The bit position of `id`, which is its index in this set.
    #[must_use]
    pub fn index_of(&self, id: ModId) -> Option<usize> {
        self.targets.iter().position(|t| t.id == id)
    }

    /// Every target present: the goal test.
    #[must_use]
    pub fn goal_mask(&self) -> u8 {
        ((1u16 << self.targets.len()) - 1) as u8
    }

    #[must_use]
    pub fn get(&self, i: usize) -> Option<&Target> {
        self.targets.get(i)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.targets.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Target> {
        self.targets.iter()
    }

    /// How many of these sit in `slot`. An item has room for three per side.
    #[must_use]
    pub fn count_in(&self, slot: Slot) -> usize {
        self.targets.iter().filter(|t| t.slot == slot).count()
    }
}