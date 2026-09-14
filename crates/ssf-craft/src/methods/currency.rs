//! The basic currency orbs. The whole crafting loop lives here.
//!
//! Every `build` is the same three steps: check legality and return early,
//! construct the outcome distribution, push one `Action`. The `miss`
//! accumulator keeps RULE 03 true by construction - start at 1.0, subtract
//! each named outcome, and the remainder is "nothing useful happened".

use ssf_data::Slot;

use crate::constants::{Confidence, PREFIX_SHARE, RARE_AFFIX_COUNTS};
use crate::context::CraftContext;
use crate::mechanics::Action;
use crate::methods::{Method, MethodInfo};
use crate::options::Family;
use crate::state::{LOCK_PREFIX, LOCK_SUFFIX, MAGIC, NORMAL, RARE, State};

// --- shared helpers ---------------------------------------------------------

/// Probability a target lands at least once in `draws` rolls on its side.
fn p_lands(chance: f64, draws: u8) -> f64 {
    1.0 - (1.0 - chance).powi(i32::from(draws))
}

/// The distribution over which targets survive a full reroll.
///
/// Enumerates every subset of targets. That is 2^n, and n is at most
/// `MAX_TARGETS`, so it is bounded - and in practice nobody aims at eight.
///
/// The approximation: targets are treated as independent. They are not, since
/// they compete for the same affix slots, so a multi-target reroll looks
/// slightly better here than it is. This is the same trade the state
/// abstraction makes, and it is written down for the same reason.
fn reroll_outcomes(
    ctx: &CraftContext,
    landed_on: State,
    prefixes: u8,
    suffixes: u8,
) -> Vec<(f64, State)> {
    let n = ctx.targets.len();

    // Per-target chance of appearing, in target-index order.
    let mut chances = vec![0.0; n];
    for slot in [Slot::Prefix, Slot::Suffix] {
        let pool = ctx.pool_for(landed_on, slot);
        let draws = if slot == Slot::Prefix {
            prefixes
        } else {
            suffixes
        };
        for (i, id) in ctx.wanted_in(landed_on, slot) {
            chances[i] = p_lands(pool.chance_of(id), draws);
        }
    }

    let mut outcomes = Vec::with_capacity(1 << n);

    for mask in 0u32..(1 << n) {
        let mut p = 1.0;
        for (i, &chance) in chances.iter().enumerate() {
            p *= if mask & (1 << i) != 0 {
                chance
            } else {
                1.0 - chance
            };
        }
        if p <= 0.0 {
            continue;
        }

        let mut s = landed_on;
        let mut hit_p = 0u8;
        let mut hit_s = 0u8;

        for i in 0..n {
            if mask & (1 << i) != 0 {
                s = s.with_target(i);
                match ctx.targets.get(i).map(|t| t.slot) {
                    Some(Slot::Prefix) => hit_p += 1,
                    Some(Slot::Suffix) => hit_s += 1,
                    None => {}
                }
            }
        }

        // Whatever the reroll produced that you did not want is junk.
        s = s
            .with_junk_prefixes(prefixes.saturating_sub(hit_p))
            .with_junk_suffixes(suffixes.saturating_sub(hit_s));

        outcomes.push((p, s));
    }

    outcomes
}

/// A reroll to Rare, weighted over how many affixes it rolls.
fn rare_reroll(ctx: &CraftContext, cleared: State) -> Vec<(f64, State)> {
    let mut outcomes = Vec::new();

    for &(count, weight) in RARE_AFFIX_COUNTS {
        // Split the affixes across the two sides.
        let prefixes = (f64::from(count) * PREFIX_SHARE).round() as u8;
        let suffixes = count - prefixes;

        for (p, s) in reroll_outcomes(ctx, cleared, prefixes.min(3), suffixes.min(3)) {
            outcomes.push((p * weight, s));
        }
    }

    outcomes
}

/// One roll onto `landed_on`, split across both sides. Returns the named
/// outcomes and whatever probability is left over.
fn single_roll(ctx: &CraftContext, from: State, landed_on: State) -> (Vec<(f64, State)>, f64) {
    let mut outcomes = Vec::new();
    let mut miss = 1.0;

    for slot in [Slot::Prefix, Slot::Suffix] {
        let share = if slot == Slot::Prefix {
            PREFIX_SHARE
        } else {
            1.0 - PREFIX_SHARE
        };
        let pool = ctx.pool_for(from, slot);

        for (i, id) in ctx.wanted_in(from, slot) {
            let p = pool.chance_of(id) * share;
            if p > 0.0 {
                outcomes.push((p, landed_on.with_target(i)));
                miss -= p;
            }
        }
    }

    (outcomes, miss)
}

// --- Orb of Transmutation ---------------------------------------------------

struct Transmutation;

impl Method for Transmutation {
    fn info(&self) -> MethodInfo {
        MethodInfo {
            key: "transmutation",
            label: "Orb of Transmutation",
            family: Family::Currency,
            summary: "Upgrade a Normal item to Magic",
            currencies: &["transmutation"],
            confidence: Confidence::Exact,
            note: "",
        }
    }

    fn build(&self, ctx: &CraftContext, state: State, out: &mut Vec<Action>) {
        if state.rarity() != NORMAL || state.is_corrupted() {
            return;
        }

        let magic = state.with_rarity(MAGIC);
        let (mut outcomes, miss) = single_roll(ctx, state, magic);
        outcomes.push((miss, magic.with_junk_prefixes(1)));

        out.push(Action::new(
            "Orb of Transmutation",
            ctx.cost_of("transmutation"),
            "transmutation",
            outcomes,
        ));
    }
}

// --- Orb of Augmentation ----------------------------------------------------

struct Augmentation;

impl Method for Augmentation {
    fn info(&self) -> MethodInfo {
        MethodInfo {
            key: "augmentation",
            label: "Orb of Augmentation",
            family: Family::Currency,
            summary: "Add one modifier to a Magic item",
            currencies: &["augmentation"],
            confidence: Confidence::Exact,
            note: "",
        }
    }

    fn build(&self, ctx: &CraftContext, state: State, out: &mut Vec<Action>) {
        if state.rarity() != MAGIC || state.is_corrupted() {
            return;
        }
        // A Magic item holds one per side. It needs a free one somewhere.
        if ctx.free_affixes(state, Slot::Prefix) == 0 && ctx.free_affixes(state, Slot::Suffix) == 0
        {
            return;
        }

        let (mut outcomes, miss) = single_roll(ctx, state, state);
        outcomes.push((miss, state.with_junk_suffixes(state.junk_suffixes() + 1)));

        out.push(Action::new(
            "Orb of Augmentation",
            ctx.cost_of("augmentation"),
            "augmentation",
            outcomes,
        ));
    }
}

// --- Orb of Alteration ------------------------------------------------------

struct Alteration;

impl Method for Alteration {
    fn info(&self) -> MethodInfo {
        MethodInfo {
            key: "alteration",
            label: "Orb of Alteration",
            family: Family::Currency,
            summary: "Reroll the modifiers on a Magic item",
            currencies: &["alteration"],
            confidence: Confidence::Exact,
            note: "",
        }
    }

    fn build(&self, ctx: &CraftContext, state: State, out: &mut Vec<Action>) {
        if state.rarity() != MAGIC || state.is_corrupted() {
            return;
        }
        // Fractured modifiers survive a reroll, so that is a different move.
        if state.fractured() != 0 {
            return;
        }

        let cleared = State::default().with_rarity(MAGIC);
        let (mut outcomes, miss) = single_roll(ctx, cleared, cleared);

        // Most of the time you land back where you started: a self-loop.
        // M5 folds this algebraically rather than iterating it.
        outcomes.push((miss, state));

        out.push(Action::new(
            "Orb of Alteration",
            ctx.cost_of("alteration"),
            "alteration",
            outcomes,
        ));
    }
}

// --- Regal Orb --------------------------------------------------------------

struct Regal;

impl Method for Regal {
    fn info(&self) -> MethodInfo {
        MethodInfo {
            key: "regal",
            label: "Regal Orb",
            family: Family::Currency,
            summary: "Upgrade a Magic item to Rare, adding one modifier",
            currencies: &["regal"],
            confidence: Confidence::Exact,
            note: "",
        }
    }

    fn build(&self, ctx: &CraftContext, state: State, out: &mut Vec<Action>) {
        if state.rarity() != MAGIC || state.is_corrupted() {
            return;
        }

        let rare = state.with_rarity(RARE);
        let (mut outcomes, miss) = single_roll(ctx, state, rare);
        outcomes.push((miss, rare.with_junk_prefixes(state.junk_prefixes() + 1)));

        out.push(Action::new(
            "Regal Orb",
            ctx.cost_of("regal"),
            "regal",
            outcomes,
        ));
    }
}

// --- Orb of Alchemy ---------------------------------------------------------

struct Alchemy;

impl Method for Alchemy {
    fn info(&self) -> MethodInfo {
        MethodInfo {
            key: "alchemy",
            label: "Orb of Alchemy",
            family: Family::Currency,
            summary: "Upgrade a Normal item straight to Rare",
            currencies: &["alchemy"],
            confidence: Confidence::Reconstructed,
            note: "affix-count distribution is reconstructed",
        }
    }

    fn build(&self, ctx: &CraftContext, state: State, out: &mut Vec<Action>) {
        if state.rarity() != NORMAL || state.is_corrupted() {
            return;
        }

        let cleared = State::default().with_rarity(RARE);
        out.push(Action::new(
            "Orb of Alchemy",
            ctx.cost_of("alchemy"),
            "alchemy",
            rare_reroll(ctx, cleared),
        ));
    }
}

// --- Chaos Orb --------------------------------------------------------------

struct Chaos;

impl Method for Chaos {
    fn info(&self) -> MethodInfo {
        MethodInfo {
            key: "chaos",
            label: "Chaos Orb",
            family: Family::Currency,
            summary: "Reroll a Rare item entirely",
            currencies: &["chaos"],
            confidence: Confidence::Reconstructed,
            note: "affix-count distribution is reconstructed",
        }
    }

    fn build(&self, ctx: &CraftContext, state: State, out: &mut Vec<Action>) {
        if state.rarity() != RARE || state.is_corrupted() {
            return;
        }
        if state.fractured() != 0 || state.has_flag(LOCK_PREFIX) || state.has_flag(LOCK_SUFFIX) {
            return;
        }

        let cleared = State::default().with_rarity(RARE);
        out.push(Action::new(
            "Chaos Orb",
            ctx.cost_of("chaos"),
            "chaos",
            rare_reroll(ctx, cleared),
        ));
    }
}

// --- Exalted Orb ------------------------------------------------------------

struct Exalted;

impl Method for Exalted {
    fn info(&self) -> MethodInfo {
        MethodInfo {
            key: "exalted",
            label: "Exalted Orb",
            family: Family::Currency,
            summary: "Add one modifier to a Rare item",
            currencies: &["exalted"],
            confidence: Confidence::Exact,
            note: "",
        }
    }

    fn build(&self, ctx: &CraftContext, state: State, out: &mut Vec<Action>) {
        if state.rarity() != RARE || state.is_corrupted() {
            return;
        }

        let free_p = ctx.free_affixes(state, Slot::Prefix);
        let free_s = ctx.free_affixes(state, Slot::Suffix);
        if free_p == 0 && free_s == 0 {
            return; // the check `free_affixes` exists for
        }

        let (mut outcomes, miss) = single_roll(ctx, state, state);

        // The junk lands on whichever side had room.
        let junked = if free_p > 0 {
            state.with_junk_prefixes(state.junk_prefixes() + 1)
        } else {
            state.with_junk_suffixes(state.junk_suffixes() + 1)
        };
        outcomes.push((miss, junked));

        out.push(Action::new(
            "Exalted Orb",
            ctx.cost_of("exalted"),
            "exalted",
            outcomes,
        ));
    }
}

// --- Orb of Annulment -------------------------------------------------------

struct Annulment;

impl Method for Annulment {
    fn info(&self) -> MethodInfo {
        MethodInfo {
            key: "annulment",
            label: "Orb of Annulment",
            family: Family::Currency,
            summary: "Remove one modifier at random",
            currencies: &["annulment"],
            confidence: Confidence::Exact,
            note: "",
        }
    }

    fn build(&self, ctx: &CraftContext, state: State, out: &mut Vec<Action>) {
        if state.rarity() == NORMAL || state.is_corrupted() {
            return;
        }

        let junk_p = state.junk_prefixes();
        let junk_s = state.junk_suffixes();
        let landed: Vec<usize> = (0..ctx.targets.len())
            .filter(|&i| state.has_target(i))
            .collect();

        let total = u32::from(junk_p) + u32::from(junk_s) + landed.len() as u32;
        if total == 0 {
            return; // nothing to remove
        }
        let each = 1.0 / f64::from(total);

        let mut outcomes = Vec::new();

        // It can take one of your targets. That is the risk, and the solver
        // needs to see it: these outcomes are *worse* than where you were.
        for i in landed {
            outcomes.push((each, state.without_target(i)));
        }
        if junk_p > 0 {
            outcomes.push((
                each * f64::from(junk_p),
                state.with_junk_prefixes(junk_p - 1),
            ));
        }
        if junk_s > 0 {
            outcomes.push((
                each * f64::from(junk_s),
                state.with_junk_suffixes(junk_s - 1),
            ));
        }

        out.push(Action::new(
            "Orb of Annulment",
            ctx.cost_of("annulment"),
            "annulment",
            outcomes,
        ));
    }
}

// --- Orb of Scouring --------------------------------------------------------

struct Scouring;

impl Method for Scouring {
    fn info(&self) -> MethodInfo {
        MethodInfo {
            key: "scouring",
            label: "Orb of Scouring",
            family: Family::Currency,
            summary: "Strip an item back to Normal",
            currencies: &["scouring"],
            confidence: Confidence::Exact,
            note: "",
        }
    }

    fn build(&self, ctx: &CraftContext, state: State, out: &mut Vec<Action>) {
        if state.rarity() == NORMAL || state.is_corrupted() {
            return;
        }

        // Fractured modifiers survive, and keep the item Magic.
        let fractured = state.fractured();
        let mut stripped = State::default()
            .with_rarity(if fractured == 0 { NORMAL } else { MAGIC })
            .with_fractured(fractured);

        for i in 0..ctx.targets.len() {
            if fractured & (1 << i) != 0 {
                stripped = stripped.with_target(i);
            }
        }

        out.push(Action::new(
            "Orb of Scouring",
            ctx.cost_of("scouring"),
            "scouring",
            vec![(1.0, stripped)],
        ));
    }
}

// --- the registry -----------------------------------------------------------

pub fn register(methods: &mut Vec<Box<dyn Method>>) {
    methods.push(Box::new(Transmutation));
    methods.push(Box::new(Augmentation));
    methods.push(Box::new(Alteration));
    methods.push(Box::new(Regal));
    methods.push(Box::new(Alchemy));
    methods.push(Box::new(Chaos));
    methods.push(Box::new(Exalted));
    methods.push(Box::new(Annulment));
    methods.push(Box::new(Scouring));
}
