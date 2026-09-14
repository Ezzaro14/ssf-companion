//! What every mechanic needs to know, built once per solve.
//!
//! Pools are expensive to build and get asked for constantly, so they are
//! memoised here behind a `RefCell`. Everything else is a borrow of data that
//! outlives the solve.

use std::cell::RefCell;
use std::rc::Rc;

use rustc_hash::FxHashMap;
use ssf_data::{BaseRecord, GameData, GroupId, ModId, Slot};

use crate::options::SolveOptions;
use crate::pool::{self, Pool};
use crate::state::State;
use crate::target::TargetSet;

/// What makes two pool requests the same request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PoolKey {
    slot: Slot,
    /// Which target groups are already excluded, as a bitmask of target index.
    excluded: u8,
}

pub struct CraftContext<'a> {
    pub data: &'a GameData,
    pub base: &'a BaseRecord,
    pub ilvl: u32,
    pub targets: &'a TargetSet,
    pub options: &'a SolveOptions,
    cache: RefCell<FxHashMap<PoolKey, Rc<Pool>>>,
}

impl<'a> CraftContext<'a> {
    #[must_use]
    pub fn new(
        data: &'a GameData,
        base: &'a BaseRecord,
        ilvl: u32,
        targets: &'a TargetSet,
        options: &'a SolveOptions,
    ) -> Self {
        Self {
            data,
            base,
            ilvl,
            targets,
            options,
            cache: RefCell::new(FxHashMap::default()),
        }
    }

    /// The state every craft starts from: a white base.
    #[must_use]
    pub fn start(&self) -> State {
        State::default()
    }

    /// Has this state got everything the user asked for?
    #[must_use]
    pub fn is_goal(&self, state: State) -> bool {
        state.hits() == self.targets.goal_mask()
    }

    /// The pool for one slot, given which target groups are already used up.
    ///
    /// Cached: building a pool walks every modifier in the game, and the search
    /// asks for the same handful of pools hundreds of thousands of times.
    pub fn pool_for(&self, state: State, slot: Slot) -> Rc<Pool> {
        let key = PoolKey {
            slot,
            excluded: state.hits(),
        };

        // The borrow ends at the semicolon. Holding it across the insert below
        // would panic at run time - RefCell checks the borrow rules then.
        if let Some(hit) = self.cache.borrow().get(&key) {
            return Rc::clone(hit);
        }

        let mut built = pool::build(self.data, self.base, self.ilvl, slot);

        // A target already on the item takes its whole group out of play.
        for i in 0..self.targets.len() {
            if state.has_target(i)
                && let Some(t) = self.targets.get(i)
            {
                let group = self.data.modifier(t.id).group;
                built = built.without_group(self.data, group);
            }
        }

        let built = Rc::new(built);
        self.cache.borrow_mut().insert(key, Rc::clone(&built));
        built
    }

    /// What one unit of `slug` costs, in the unit being minimised.
    ///
    /// An unavailable currency is infinity, not a large number: the solver must
    /// treat that route as impossible rather than merely expensive.
    ///
    /// TODO: every objective costs 1.0 for now
    #[must_use]
    pub fn cost_of(&self, slug: &str) -> f64 {
        if self.options.is_unavailable(slug) {
            return f64::INFINITY;
        }
        1.0
    }

    /// The targets that could still be rolled in `slot`.
    #[must_use]
    pub fn wanted_in(&self, state: State, slot: Slot) -> Vec<(usize, ModId)> {
        self.targets
            .iter()
            .enumerate()
            .filter(|(i, t)| t.slot == slot && !state.has_target(*i))
            .map(|(i, t)| (i, t.id))
            .collect()
    }

    /// Affixes still free on one side. An Exalted Orb needs one.
    ///
    /// This lives here rather than on `State` because a state does not know
    /// which of its targets are prefixes — only the `TargetSet` does.
    #[must_use]
    pub fn free_affixes(&self, state: State, slot: Slot) -> u8 {
        let junk = match slot {
            Slot::Prefix => state.junk_prefixes(),
            Slot::Suffix => state.junk_suffixes(),
        };
        let landed = self
            .targets
            .iter()
            .enumerate()
            .filter(|(i, t)| t.slot == slot && state.has_target(*i))
            .count() as u8;

        crate::state::capacity(state.rarity()).saturating_sub(junk + landed)
    }

    /// Group of the modifier behind a target index.
    #[must_use]
    pub fn group_of(&self, target: usize) -> Option<GroupId> {
        self.targets
            .get(target)
            .map(|t| self.data.modifier(t.id).group)
    }
}
