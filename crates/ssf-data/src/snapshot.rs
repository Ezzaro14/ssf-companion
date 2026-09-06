//! The binary snapshot.

use std::io::{Read, Write};
use std::path::Path;

use crate::index::GameData;

pub const SCHEMA_VERSION: u32 = 1;
const MAGIC: &[u8; 8] = b"SSFIDX01";

#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a snapshot file")]
    NotASnapshot,
    #[error(
        "snapshot schema {found}, this build reads {expected} — rebuild with `cargo xtask build-data`"
    )]
    Schema { found: u32, expected: u32 },
    #[error("decode: {0}")]
    Decode(#[from] bincode::Error),
}
/// Write the snapshot to a file.
pub fn write(data: &GameData, path: &Path) -> Result<u64, SnapshotError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let encoded = bincode::serialize(data)?;
    let compressed = zstd::encode_all(encoded.as_slice(), 10)?;

    let mut file = std::fs::File::create(path)?;
    file.write_all(MAGIC)?;
    file.write_all(&SCHEMA_VERSION.to_le_bytes())?;
    file.write_all(&compressed)?;
    file.flush()?;
    Ok((MAGIC.len() + 4 + compressed.len()) as u64)
}
// Read the snapshot from a file.
pub fn read(path: &Path) -> Result<GameData, SnapshotError> {
    let mut file = std::fs::File::open(path)?;

    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(SnapshotError::NotASnapshot);
    }

    let mut version = [0u8; 4];
    file.read_exact(&mut version)?;
    let found = u32::from_le_bytes(version);
    if found != SCHEMA_VERSION {
        return Err(SnapshotError::Schema {
            found,
            expected: SCHEMA_VERSION,
        });
    }

    let mut compressed = Vec::new();
    file.read_to_end(&mut compressed)?;
    let encoded = zstd::decode_all(compressed.as_slice())?;

    Ok(bincode::deserialize(&encoded)?)
}
