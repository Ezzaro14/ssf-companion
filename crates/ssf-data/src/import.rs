//! Turning the dump into the index.

use std::collections::BTreeMap;

use rustc_hash::FxHashMap;

use crate::ids::{BaseId, GroupId, Interner, ModId, TagId};
use crate::index::{BaseRecord, CRAFTABLE_DOMAINS, GameData, ModRecord, Slot, TagSet};
use crate::raw::{RawBase, RawMod};

/// Build the index from the two parsed dump files.
///
/// The inputs are `BTreeMap`, not `HashMap`, and that is load-bearing: a
/// record's position becomes its id, so iteration order decides the ids. A
/// `BTreeMap` iterates in sorted key order every time, which is what makes two
/// builds of the same dump produce byte-identical snapshots.
#[must_use]
pub fn build(bases: BTreeMap<String, RawBase>, mods: BTreeMap<String, RawMod>) -> GameData {
    let mut tags = Interner::default();
    let mut groups = Interner::default();
    let mut records: Vec<ModRecord> = Vec::new();

    for (internal_id, raw) in mods {
        if !CRAFTABLE_DOMAINS.contains(&raw.domain.as_str()) {
            continue;
        }
        let Some(slot) = Slot::from_generation_type(&raw.generation_type) else {
            continue;
        };

        let spawn_weights = raw
            .spawn_weights
            .iter()
            .map(|w| (TagId(tags.intern(&w.tag) as u16), w.weight as i32))
            .collect();

        records.push(ModRecord {
            id: ModId(records.len() as u32),
            internal_id,
            slot,
            group: GroupId(groups.intern(raw.groups.first().map_or("", |g| g))),
            required_level: raw.required_level as u32,
            spawn_weights,
            text: raw.text,
            is_essence_only: raw.is_essence_only,
        });
    }

    let mut base_records: Vec<BaseRecord> = Vec::new();
    let mut by_name: FxHashMap<String, BaseId> = FxHashMap::default();

    for (_metadata_path, raw) in bases {
        if raw.release_state != "released" {
            continue;
        }

        let mut tag_set = TagSet::default();
        for tag in &raw.tags {
            tag_set.insert(TagId(tags.intern(tag) as u16));
        }

        let id = BaseId(base_records.len() as u32);
        by_name.insert(raw.name.clone(), id);
        base_records.push(BaseRecord {
            id,
            name: raw.name,
            item_class: raw.item_class,
            drop_level: raw.drop_level as u32,
            tags: tag_set,
        });
    }
    GameData {
        mods: records,
        bases: base_records,
        tags,
        groups,
        by_name,
    }
}
