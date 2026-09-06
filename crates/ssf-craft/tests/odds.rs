use std::path::Path;

use ssf_craft::pool;
use ssf_data::{GameData, Slot};

fn snapshot() -> Option<GameData> {
    let path = Path::new("../../assets/index.bin");
    if !path.exists() {
        eprintln!("no snapshot; skipping. run `cargo xtask build-data`");
        return None;
    }
    ssf_data::snapshot::read(path).ok()
}

#[test]
fn every_pool_is_a_probability_distribution() {
    let Some(data) = snapshot() else { return };

    for name in ["Vaal Regalia", "Iron Ring", "Leather Belt"] {
        let Some(base) = data.base_by_name(name) else {
            panic!("base `{name}` not found - has the dump changed?");
        };

        for slot in [Slot::Prefix, Slot::Suffix] {
            let pool = pool::build(&data, base, 86, slot);
            assert!(!pool.is_empty(), "{name} {slot:?}: empty pool");

            let sum: f64 = pool.entries.iter().map(|&(id, _)| pool.chance_of(id)).sum();
            assert!(
                (sum - 1.0).abs() < 1e-9,
                "{name} {slot:?}: probabilities sum to {sum}, not 1"
            );

            for &(_, w) in &pool.entries {
                assert!(
                    w > 0,
                    "{name} {slot:?}: a zero-weight modifier reached the pool"
                );
            }
        }
    }
}

#[test]
fn item_level_gates_the_pool() {
    let Some(data) = snapshot() else { return };
    let Some(base) = data.base_by_name("Vaal Regalia") else {
        return;
    };

    let low = pool::build(&data, base, 1, Slot::Prefix);
    let high = pool::build(&data, base, 86, Slot::Prefix);

    assert!(
        high.len() > low.len(),
        "ilvl 86 should offer more than ilvl 1: {} vs {}",
        high.len(),
        low.len()
    );
}
