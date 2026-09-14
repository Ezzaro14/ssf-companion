//! Numbers that did not come from the game data, and how much to trust them.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// Straight from the dump. The game's own number.
    Exact,
    /// Derived from community testing or documented mechanics.
    Reconstructed,
    /// A considered guess.
    Estimated,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Exact => "exact",
            Self::Reconstructed => "reconstructed",
            Self::Estimated => "estimated",
        };
        f.write_str(s)
    }
}

/// Share of a roll that lands on the prefix side.

pub const PREFIX_SHARE: f64 = 0.5;
pub const PREFIX_SHARE_CONFIDENCE: Confidence = Confidence::Reconstructed;

/// How many affixes a fresh Rare rolls, and how often (based on community testing).
pub const RARE_AFFIX_COUNTS: &[(u8, f64)] = &[(4, 0.50), (5, 0.33), (6, 0.17)];
pub const RARE_AFFIX_CONFIDENCE: Confidence = Confidence::Reconstructed;