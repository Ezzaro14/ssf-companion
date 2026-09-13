//! What the user switched on, and what "cheapest" means to them.
//!
//! Every mechanic belongs to a family, and every family has a switch. That is
//! how you answer "what would this craft cost without Harvest?" - and how the
//! registry decides what to offer.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    Currency,
    Essence,
    Bench,
    Meta,
    Harvest,
    Fossil,
    Eldritch,
    Beast,
    Fracture,
    Veiled,
    Recombinator,
    Foresight,
}

impl Family {
    /// Every family, in a fixed order. This order *is* the bit order below.
    pub const ALL: [Self; 12] = [
        Self::Currency,
        Self::Essence,
        Self::Bench,
        Self::Meta,
        Self::Harvest,
        Self::Fossil,
        Self::Eldritch,
        Self::Beast,
        Self::Fracture,
        Self::Veiled,
        Self::Recombinator,
        Self::Foresight,
    ];

    #[must_use]
    pub const fn index(self) -> u32 {
        self as u32
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Currency => "currency",
            Self::Essence => "essence",
            Self::Bench => "bench",
            Self::Meta => "meta",
            Self::Harvest => "harvest",
            Self::Fossil => "fossil",
            Self::Eldritch => "eldritch",
            Self::Beast => "beast",
            Self::Fracture => "fracture",
            Self::Veiled => "veiled",
            Self::Recombinator => "recombinator",
            Self::Foresight => "foresight",
        }
    }

    /// Parse a `--without harvest` argument.
    #[must_use]
    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.name() == s)
    }
}

impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// What the solver is minimising.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Objective {
    /// Hours of farming. The point of the whole project.
    #[default]
    Time,
    /// Raw orb count, ignoring how hard each one is to get.
    Currency,
    /// Number of actions, for when you have the currency and not the patience.
    Chances,
}

/// How hard to search before giving up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Effort {
    Quick,
    #[default]
    Standard,
    Exhaustive,
}

impl Effort {
    #[must_use]
    pub const fn max_states(self) -> usize {
        match self {
            Self::Quick => 40_000,
            Self::Standard => 400_000,
            Self::Exhaustive => 2_000_000,
        }
    }

    #[must_use]
    pub const fn time_limit_secs(self) -> u64 {
        match self {
            Self::Quick => 3,
            Self::Standard => 20,
            Self::Exhaustive => 120,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Influence {
    Shaper,
    Elder,
    Crusader,
    Redeemer,
    Hunter,
    Warlord,
}

/// Everything the caller decides before a solve starts.
#[derive(Debug, Clone)]
pub struct SolveOptions {
    /// One bit per family, indexed by `Family::index`.
    families: u16,
    pub objective: Objective,
    pub effort: Effort,
    pub influence: Option<Influence>,
    /// Currency slugs the player cannot obtain at all.
    pub unavailable: Vec<String>,
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            families: (1 << Family::ALL.len()) - 1, // every family on
            objective: Objective::default(),
            effort: Effort::default(),
            influence: None,
            unavailable: Vec::new(),
        }
    }
}

impl SolveOptions {
    #[must_use]
    pub const fn allows(&self, family: Family) -> bool {
        self.families & (1 << family.index()) != 0
    }

    pub fn set(&mut self, family: Family, on: bool) {
        if on {
            self.families |= 1 << family.index();
        } else {
            self.families &= !(1 << family.index());
        }
    }

    /// Turn everything off except these. For `--only harvest`.
    pub fn only(&mut self, families: &[Family]) {
        self.families = 0;
        for &f in families {
            self.set(f, true);
        }
    }

    #[must_use]
    pub fn is_unavailable(&self, slug: &str) -> bool {
        self.unavailable.iter().any(|s| s == slug)
    }
}
