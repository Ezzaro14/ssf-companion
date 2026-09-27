//! over real pools: every action is a probability distribution.

use std::path::Path;

use ssf_craft::context::CraftContext;
use ssf_craft::mechanics::{Action, is_distribution};
use ssf_craft::methods;
use ssf_craft::options::SolveOptions;
use ssf_craft::state::State;
use ssf_craft::target::TargetSet;
use ssf_data::GameData;

fn snapshot() -> Option<GameData> {
    let path = Path::new("../../assets/index.bin");
    if !path.exists() {
        eprintln!("no snapshot; skipping. run `cargo xtask build-data`");
        return None;
    }
    ssf_data::snapshot::read(path).ok()
}

/// Every action every method offers on a white Vaal Regalia at item level 86.
fn white_actions(data: &GameData) -> Vec<Action> {
    let Some(base) = data.base_by_name("Vaal Regalia") else {
        panic!("base `Vaal Regalia` not found - has the dump changed?");
    };

    let targets = TargetSet::default();
    let options = SolveOptions::default();
    let ctx = CraftContext::new(data, base, 86, &targets, &options);

    let mut actions = Vec::new();
    for method in methods::registry(&options) {
        method.build(&ctx, State::default(), &mut actions);
    }
    actions
}

#[test]
fn every_action_is_a_distribution() {
    let Some(data) = snapshot() else { return };

    for action in &white_actions(&data) {
        assert!(
            is_distribution(&action.outcomes),
            "{}: outcomes are not a distribution: {:?}",
            action.label,
            action.outcomes
        );
    }
}

#[test]
fn a_white_base_offers_only_the_moves_that_are_legal_on_it() {
    let Some(data) = snapshot() else { return };

    let mut offered: Vec<&str> = white_actions(&data).iter().map(|a| a.method).collect();
    offered.sort_unstable();

    assert_eq!(offered, ["alchemy", "transmutation"]);
}
