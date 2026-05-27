//! Crystal-upgrades state slice.
//!
//! Mirrors `CrystalUpgradesState` from the legacy TS
//! `packages/logic/src/state/schema.ts`. `prestige_shards` is the spend
//! resource; `crystal_upgrades[u]` holds the current level for each
//! crystal-upgrade index (0-based). Callers pass 1-based `i` as input —
//! the mechanic function does the `-1` internally.

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Default crystal-upgrade slot count. Matches the legacy
/// `crystalUpgrades: [0, 0, 0, 0, 0, 0, 0, 0]` initial state.
pub const CRYSTAL_UPGRADES_DEFAULT_LEN: usize = 8;

/// Slice of `GameState` read/written by `buy_crystal_upgrades`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CrystalUpgradesState {
    /// Spend resource — `player.prestigeShards` in the legacy schema.
    pub prestige_shards: Decimal,
    /// Per-upgrade level. Indexed 0-based internally; the public buy
    /// function takes a 1-based `i` to match the legacy convention.
    /// Fixed cardinality at compile time. (Tier B item 12 / Anvil F4.)
    pub crystal_upgrades: [f64; CRYSTAL_UPGRADES_DEFAULT_LEN],
}

impl Default for CrystalUpgradesState {
    /// Zero shards, all-zero upgrade levels at the legacy slot count.
    fn default() -> Self {
        Self {
            prestige_shards: Decimal::zero(),
            crystal_upgrades: [0.0; CRYSTAL_UPGRADES_DEFAULT_LEN],
        }
    }
}
