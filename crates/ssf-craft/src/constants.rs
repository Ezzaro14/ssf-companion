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
