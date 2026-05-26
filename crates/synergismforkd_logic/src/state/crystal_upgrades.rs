//! Crystal-upgrades state slice.
//!
//! Mirrors `CrystalUpgradesState` from the legacy TS
//! `packages/logic/src/state/schema.ts`. `prestige_shards` is the spend
//! resource; `crystal_upgrades[u]` holds the current level for each
//! crystal-upgrade index (0-based). Callers pass 1-based `i` as input —
//! the mechanic function does the `-1` internally.

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` read/written by `buy_crystal_upgrades`.
#[derive(Debug, Clone, PartialEq)]
pub struct CrystalUpgradesState {
    /// Spend resource — `player.prestigeShards` in the legacy schema.
    pub prestige_shards: Decimal,
    /// Per-upgrade level. Indexed 0-based internally; the public buy
    /// function takes a 1-based `i` to match the legacy convention.
    pub crystal_upgrades: Vec<f64>,
}
