//! `RePoE` ingest, the game data index, and the binary snapshot the app loads.

pub mod ids;
pub mod import;
pub mod index;
pub mod raw;
pub mod snapshot;

pub use ids::{BaseId, GroupId, ModId, StatId, TagId};
pub use index::{BaseRecord, GameData, ModRecord, Slot, TagSet};

use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("reading {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("parsing {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_json::Error,
    },
}

/// Read `base_items.json` and `mods.json` and build the index.
pub fn import_from_dump(base_items: &Path, mods: &Path) -> Result<GameData, DataError> {
    let base_text = std::fs::read_to_string(base_items).map_err(|source| DataError::Read {
        path: base_items.display().to_string(),
        source,
    })?;
    let bases: BTreeMap<String, raw::RawBase> =
        serde_json::from_str(&base_text).map_err(|source| DataError::Parse {
            path: base_items.display().to_string(),
            source,
        })?;

    let mod_text = std::fs::read_to_string(mods).map_err(|source| DataError::Read {
        path: mods.display().to_string(),
        source,
    })?;
    let mod_map: BTreeMap<String, raw::RawMod> =
        serde_json::from_str(&mod_text).map_err(|source| DataError::Parse {
            path: mods.display().to_string(),
            source,
        })?;

    Ok(import::build(bases, mod_map))
}
