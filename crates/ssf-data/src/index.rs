//! The indexed game data

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::ids::{BaseId, DomainId, GroupId, Interner, ModId, TagId};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Slot {
    Prefix,
    Suffix,
}

impl Slot {
    // Returns the slot from a generation type string, or None if the string is not a valid generation type.
    pub fn from_generation_type(kind: &str) -> Option<Self> {
        match kind {
            "prefix" => Some(Self::Prefix),
            "suffix" => Some(Self::Suffix),
            // if not suffix or prefix, discard.
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagSet {
    tags: Vec<TagId>,
}
// A set of tags, represented as a vector of TagIds. The vector is kept sorted and unique.
impl TagSet {
    pub fn insert(&mut self, tag: TagId) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    // Returns true if the tag is in the set, false otherwise.
    pub fn contains(&self, tag: TagId) -> bool {
        self.tags.contains(&tag)
    }
    // Returns an iterator over the tags in the set.
    pub fn iter(&self) -> impl Iterator<Item = TagId> + '_ {
        self.tags.iter().copied()
    }
}
// A mod record is a record of a mod, including its id, internal id, slot, group, required level, spawn weights, and text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModRecord {
    pub id: ModId,
    // The name of the mod, e.g. "Strength1"
    pub internal_id: String,
    pub slot: Slot,
    pub group: GroupId,
    pub required_level: u32,
    pub spawn_weights: Vec<(TagId, i32)>,
    pub text: String,
}
// A mod record has a list of spawn weights, which are pairs of (tag, weight). The weight is used to determine the probability of the mod spawning on an item with the given tag. The first matching tag is used, and the weight is returned. If no tags match, the weight is 0.
#[derive(Debug, Clone, Serialize, Deserialize)]
impl ModRecord {
    pub fn weight_for(&self, tags: &TagSet) -> i32 {
        for &(tag, weight) in &self.spawn_weights {
            if tags.contains(tag) {
                return weight;      // stop here — even when it is 0
            }
        }
        0
    }
}
// A base record is a record of a base item, including its id, name, item class, drop level, and tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseRecord {
    pub id: BaseId,
    pub name: String,
    pub item_class: String,
    pub drop_level: u32,
    pub tags: TagSet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    pub mods: Vec<ModRecord>,
    pub bases: Vec<BaseRecord>,
    pub tags: Interner,
    pub groups: Interner,
    pub by_name: FxHashMap<String, BaseId>,
}
// A GameData struct is a collection of mods, bases, tags, and groups. It is used to look up mods and bases by id or name, and to get the weight of a mod for a given set of tags.
impl GameData {
    pub fn modifier(&self, id: ModId) -> &ModRecord {
        &self.mods[id.index()]
    }

    pub fn base(&self, id: BaseId) -> &BaseRecord {
        &self.bases[id.index()]
    }

    pub fn base_by_name(&self, name: &str) -> Option<&BaseRecord> {
        self.by_name.get(name).map(|&id| self.base(id))
    }

    pub fn tag_id(&self, name: &str) -> Option<TagId> {
        self.tags.get(name).map(|n| TagId(n as u16))
    }
}