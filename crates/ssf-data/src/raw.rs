//! Serde mirrors of the `RePoE` dump.

use serde::{Deserialize, Deserializer};

/// `text` and `name` are sometimes `null` rather than `""` in the dump.
/// Serde's `default` does not cover an explicit null, so it is spelled out.
fn null_is_empty<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Debug, Deserialize)]
pub struct RawWeight {
    pub tag: String,
    #[serde(default)]
    pub weight: i64,
}

#[derive(Debug, Deserialize)]
pub struct RawStat {
    pub id: String,
    #[serde(default)]
    pub min: i64,
    #[serde(default)]
    pub max: i64,
}

#[derive(Debug, Deserialize)]
pub struct RawMod {
    #[serde(default, deserialize_with = "null_is_empty")]
    pub name: String,
    #[serde(default, deserialize_with = "null_is_empty")]
    pub domain: String,
    #[serde(default, deserialize_with = "null_is_empty")]
    pub generation_type: String,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub spawn_weights: Vec<RawWeight>,
    #[serde(default)]
    pub generation_weights: Vec<RawWeight>,
    #[serde(default)]
    pub required_level: i64,
    #[serde(default)]
    pub stats: Vec<RawStat>,
    #[serde(default)]
    pub adds_tags: Vec<String>,
    #[serde(default)]
    pub implicit_tags: Vec<String>,
    #[serde(default)]
    pub is_essence_only: bool,
    #[serde(default, deserialize_with = "null_is_empty")]
    pub text: String,
    #[serde(default, rename = "type", deserialize_with = "null_is_empty")]
    pub mod_type: String,
}

#[derive(Debug, Deserialize)]
pub struct RawBase {
    #[serde(default, deserialize_with = "null_is_empty")]
    pub name: String,
    #[serde(default, deserialize_with = "null_is_empty")]
    pub domain: String,
    #[serde(default, deserialize_with = "null_is_empty")]
    pub item_class: String,
    #[serde(default)]
    pub drop_level: i64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, deserialize_with = "null_is_empty")]
    pub release_state: String,
}
