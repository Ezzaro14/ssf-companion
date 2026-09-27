//! Value iteration over a cyclic state graph. Knows nothing about the game.

use std::sync::atomic::AtomicBool;
use std::time::Duration;

use rustc_hash::FxHashMap;

/// The solver's only view of a state: an opaque number the caller chose.
pub type StateId = u64;

pub const INF: f64 = f64::INFINITY;
pub const UNREACHABLE: f64 = 1e9;

/// One action from a state: what it costs and where it might leave you.
#[derive(Debug, Clone)]
pub struct Transition {
    /// In whatever unit the caller is minimising. The solver never learns which.
    pub cost: f64,
    /// The caller's handle on this action, handed back in the policy.
    pub tag: u32,
    /// Probability and resulting state. Must sum to one.
    pub outcomes: Vec<(f64, StateId)>,
}

/// Anything shaped like "states, goals, and stochastic actions between them".
pub trait Problem {
    fn is_goal(&self, state: StateId) -> bool;
    /// Append every action available from `state`.
    fn actions(&self, state: StateId, out: &mut Vec<Transition>);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Exploring,
    Pruning,
    Compiling,
    Sweeping,
    Reading,
}

pub trait Progress: Sync {
    fn report(&self, phase: Phase, detail: &str, states: usize, sweeps: u32);
}

/// A `Progress` that reports nothing.
pub struct Silent;

impl Progress for Silent {
    fn report(&self, _phase: Phase, _detail: &str, _states: usize, _sweeps: u32) {}
}

#[derive(Debug, Clone)]
pub struct Budget {
    pub max_states: usize,
    pub max_sweeps: u32,
    pub tolerance: f64,
    pub time_limit: Duration,
    pub parallel: bool,
    pub accelerate: bool,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            max_states: 500_000,
            max_sweeps: 10_000,
            tolerance: 1e-10,
            time_limit: Duration::from_secs(30),
            parallel: true,
            accelerate: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Solution {
    pub value: FxHashMap<StateId, f64>,
    /// State to the winning action's tag.
    pub policy: FxHashMap<StateId, u32>,
    pub sweeps: u32,
    pub converged: bool,
}

#[must_use]
pub fn solve(
    _problem: &impl Problem,
    _start: StateId,
    _budget: &Budget,
    _progress: &impl Progress,
    _cancel: &AtomicBool,
) -> Solution {
    todo!()
}
