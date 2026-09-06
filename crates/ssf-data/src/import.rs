//! Turning the dump into index.

use std::collections::BTreeMap;

use crate::ids::*;
use crate::index::*;
use crate::raw::{RawBase, RawMod};

pub const CRAFTABLE_DOMAINS: [&str; 3] = ["item", "flask", "abyss_jewel"];

pub fn build(bases: BTreeMap<String, RawBase>, mods: BTreeMap<String, RawMod>) -> GameData {
    let mut tags = Interner::default();
    let mut groups = Interner::default();
    let mut records: Vec<ModRecord> = Vec::new();

    for (internal_id, raw) in mods {
        if !CRAFTABLE_DOMAINS.contains(&raw.domain.as_str()) {
            continue; // 1
        }
        let Some(slot) = Slot::from_generation_type(&raw.generation_type) else {
            continue; // 2
        };

        let spawn_weights = raw // 3
            .spawn_weights
            .iter()
            .map(|w| (TagId(tags.intern(&w.tag) as u16), w.weight as i32))
            .collect();

        records.push(ModRecord {
            // 4
            id: ModId(records.len() as u32),
            internal_id,
            slot,
            group: GroupId(groups.intern(raw.groups.first().map_or("", |g| g))),
            required_level: raw.required_level as u32,
            spawn_weights,
            text: raw.text,
        });
    }
    todo!()
}
