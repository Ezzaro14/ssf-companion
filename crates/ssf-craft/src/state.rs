//! What the solver remembers about the item in hand.
//!
//! The search space is infinite if an item is a list of modifiers: every junk
//! suffix makes a different item. It becomes finite once you notice the solver
//! only needs the things that change which action is correct.
//!
//! All of it packs into a `u64`, so a state is `Copy`, hashes in one round and
//! compares in one instruction.

use std::fmt;

/// How many modifiers a plan can aim at. The mask has to fit in the state.
pub const MAX_TARGETS: usize = 8;

pub const NORMAL: u8 = 0;
pub const MAGIC: u8 = 1;
pub const RARE: u8 = 2;

// --- the layout ------------------------------------------------------------

const RARITY_BITS: u32 = 2;
const HIT_BITS: u32 = MAX_TARGETS as u32;
const BENCH_BITS: u32 = MAX_TARGETS as u32;
const FRACTURED_BITS: u32 = MAX_TARGETS as u32;
const JUNK_BITS: u32 = 2;
const FLAG_BITS: u32 = 9;
const IMPRINT_BITS: u32 = 11;

const RARITY_SHIFT: u32 = 0;
const HIT_SHIFT: u32 = RARITY_SHIFT + RARITY_BITS;
const BENCH_SHIFT: u32 = HIT_SHIFT + HIT_BITS;
const FRACTURED_SHIFT: u32 = BENCH_SHIFT + BENCH_BITS;
const JUNK_P_SHIFT: u32 = FRACTURED_SHIFT + FRACTURED_BITS;
const JUNK_S_SHIFT: u32 = JUNK_P_SHIFT + JUNK_BITS;
const FLAGS_SHIFT: u32 = JUNK_S_SHIFT + JUNK_BITS;
const IMPRINT_SHIFT: u32 = FLAGS_SHIFT + FLAG_BITS;

const TOTAL_BITS: u32 = IMPRINT_SHIFT + IMPRINT_BITS;
const _: () = assert!(TOTAL_BITS <= 64);

const fn mask(bits: u32) -> u64 {
    (1u64 << bits) - 1
}

const RARITY_MASK: u64 = mask(RARITY_BITS);
const HIT_MASK: u64 = mask(HIT_BITS);
const BENCH_MASK: u64 = mask(BENCH_BITS);
const FRACTURED_MASK: u64 = mask(FRACTURED_BITS);
const JUNK_MASK: u64 = mask(JUNK_BITS);
const FLAGS_MASK: u64 = mask(FLAG_BITS);
const IMPRINT_MASK: u64 = mask(IMPRINT_BITS);

// --- flags -----------------------------------------------------------------
// Persistent facts that are not modifiers in their own right.

pub const MULTIMOD: u16 = 1 << 0;
pub const LOCK_PREFIX: u16 = 1 << 1;
pub const LOCK_SUFFIX: u16 = 1 << 2;
pub const CORRUPTED: u16 = 1 << 3;
pub const FRACTURED_JUNK_P: u16 = 1 << 4;
pub const FRACTURED_JUNK_S: u16 = 1 << 5;
pub const EXARCH_FILLED: u16 = 1 << 6;
pub const EATER_FILLED: u16 = 1 << 7;
pub const VEILED: u16 = 1 << 8;

pub const FLAG_NAMES: [(u16, &str); 9] = [
    (MULTIMOD, "multimod"),
    (LOCK_PREFIX, "prefixes locked"),
    (LOCK_SUFFIX, "suffixes locked"),
    (CORRUPTED, "corrupted"),
    (FRACTURED_JUNK_P, "a junk prefix is fractured"),
    (FRACTURED_JUNK_S, "a junk suffix is fractured"),
    (EXARCH_FILLED, "Exarch implicit"),
    (EATER_FILLED, "Eater implicit"),
    (VEILED, "veiled modifier to unveil"),
];

// --- the state -------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct State(u64);

impl State {
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    // -- rarity

    #[must_use]
    pub const fn rarity(self) -> u8 {
        ((self.0 >> RARITY_SHIFT) & RARITY_MASK) as u8
    }

    #[must_use]
    pub const fn with_rarity(self, r: u8) -> Self {
        Self(
            (self.0 & !(RARITY_MASK << RARITY_SHIFT))
                | ((r as u64 & RARITY_MASK) << RARITY_SHIFT),
        )
    }

    // -- targets on the item

    #[must_use]
    pub const fn hits(self) -> u8 {
        ((self.0 >> HIT_SHIFT) & HIT_MASK) as u8
    }

    #[must_use]
    pub const fn has_target(self, i: usize) -> bool {
        self.0 & (1 << (HIT_SHIFT + i as u32)) != 0
    }

    #[must_use]
    pub const fn with_target(self, i: usize) -> Self {
        Self(self.0 | (1 << (HIT_SHIFT + i as u32)))
    }

    #[must_use]
    pub const fn without_target(self, i: usize) -> Self {
        Self(self.0 & !(1 << (HIT_SHIFT + i as u32)))
    }

    // -- which targets came from the bench, and which are fractured

    #[must_use]
    pub const fn bench(self) -> u8 {
        ((self.0 >> BENCH_SHIFT) & BENCH_MASK) as u8
    }

    #[must_use]
    pub const fn with_bench(self, mask_bits: u8) -> Self {
        Self(
            (self.0 & !(BENCH_MASK << BENCH_SHIFT))
                | ((mask_bits as u64 & BENCH_MASK) << BENCH_SHIFT),
        )
    }

    #[must_use]
    pub const fn fractured(self) -> u8 {
        ((self.0 >> FRACTURED_SHIFT) & FRACTURED_MASK) as u8
    }

    #[must_use]
    pub const fn with_fractured(self, mask_bits: u8) -> Self {
        Self(
            (self.0 & !(FRACTURED_MASK << FRACTURED_SHIFT))
                | ((mask_bits as u64 & FRACTURED_MASK) << FRACTURED_SHIFT),
        )
    }

    // -- junk counts

    #[must_use]
    pub const fn junk_prefixes(self) -> u8 {
        ((self.0 >> JUNK_P_SHIFT) & JUNK_MASK) as u8
    }

    #[must_use]
    pub const fn with_junk_prefixes(self, n: u8) -> Self {
        Self((self.0 & !(JUNK_MASK << JUNK_P_SHIFT)) | ((n as u64 & JUNK_MASK) << JUNK_P_SHIFT))
    }

    #[must_use]
    pub const fn junk_suffixes(self) -> u8 {
        ((self.0 >> JUNK_S_SHIFT) & JUNK_MASK) as u8
    }

    #[must_use]
    pub const fn with_junk_suffixes(self, n: u8) -> Self {
        Self((self.0 & !(JUNK_MASK << JUNK_S_SHIFT)) | ((n as u64 & JUNK_MASK) << JUNK_S_SHIFT))
    }

    // -- flags

    #[must_use]
    pub const fn flags(self) -> u16 {
        ((self.0 >> FLAGS_SHIFT) & FLAGS_MASK) as u16
    }

    #[must_use]
    pub const fn has_flag(self, f: u16) -> bool {
        self.flags() & f != 0
    }

    #[must_use]
    pub const fn set_flag(self, f: u16) -> Self {
        Self(self.0 | ((f as u64 & FLAGS_MASK) << FLAGS_SHIFT))
    }

    #[must_use]
    pub const fn clear_flag(self, f: u16) -> Self {
        Self(self.0 & !((f as u64 & FLAGS_MASK) << FLAGS_SHIFT))
    }

    // -- imprint (a beast copy of a magic item)

    #[must_use]
    pub const fn imprint(self) -> u16 {
        ((self.0 >> IMPRINT_SHIFT) & IMPRINT_MASK) as u16
    }

    #[must_use]
    pub const fn with_imprint(self, v: u16) -> Self {
        Self(
            (self.0 & !(IMPRINT_MASK << IMPRINT_SHIFT))
                | ((v as u64 & IMPRINT_MASK) << IMPRINT_SHIFT),
        )
    }
}

// --- free functions --------------------------------------------------------

/// Affixes per side, by rarity.
#[must_use]
pub const fn capacity(rarity: u8) -> u8 {
    match rarity {
        NORMAL => 0,
        MAGIC => 1,
        _ => 3,
    }
}

#[must_use]
pub const fn rarity_name(rarity: u8) -> &'static str {
    match rarity {
        NORMAL => "Normal",
        MAGIC => "Magic",
        _ => "Rare",
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", rarity_name(self.rarity()))?;

        for i in 0..MAX_TARGETS {
            if self.has_target(i) {
                write!(f, ", target {i}")?;
            }
        }

        let free_p = capacity(self.rarity()).saturating_sub(self.junk_prefixes());
        let free_s = capacity(self.rarity()).saturating_sub(self.junk_suffixes());
        write!(f, ", {free_p}p/{free_s}s free")?;

        for (bit, name) in FLAG_NAMES {
            if self.has_flag(bit) {
                write!(f, ", {name}")?;
            }
        }
        Ok(())
    }
}