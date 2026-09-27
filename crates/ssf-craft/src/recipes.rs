//! The crafting bench: which modifiers it can add, and what each costs.

use ssf_data::Slot;

use crate::constants::Confidence;

#[derive(Debug, Clone)]
pub struct Recipe {
    /// Text as the bench shows it.
    pub label: &'static str,
    pub slot: Slot,
    /// Currency slug and quantity.
    pub cost: &'static [(&'static str, f64)],
    pub confidence: Confidence,
}

pub const RECIPES: &[Recipe] = &[
    Recipe {
        label: "+(25-29) to maximum Life",
        slot: Slot::Prefix,
        cost: &[("augmentation", 4.0)],
        confidence: Confidence::Reconstructed,
    },
    Recipe {
        label: "+(15-17)% to Chaos Resistance",
        slot: Slot::Suffix,
        cost: &[("chaos", 2.0)],
        confidence: Confidence::Reconstructed,
    },
];

#[must_use]
pub fn find(label: &str) -> Option<&'static Recipe> {
    RECIPES.iter().find(|r| r.label == label)
}
