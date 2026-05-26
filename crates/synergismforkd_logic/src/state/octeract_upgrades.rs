//! Octeract upgrade-level state slice.
//!
//! Mirrors `player.octeractUpgrades.<name>` from the legacy schema.
//! Backs [`crate::mechanics::octeract_upgrade_levels`] and
//! [`crate::mechanics::octeracts`].
//!
//! The octeract currency itself lives in
//! [`crate::state::CubeBalancesState`] alongside the other
//! cube-tier balances; this slice holds just the per-upgrade
//! state.

/// One octeract upgrade's per-player state. Mirrors
/// `player.octeractUpgrades.<name>`.
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct OcteractUpgrade {
    /// Purchased level.
    pub level: f64,
    /// Accumulated free levels.
    pub free_level: f64,
    /// Quality-of-life flag — when true, the upgrade survives
    /// `noOcteracts` and `sadisticPrequel`.
    pub quality_of_life: bool,
}

/// Fixed cardinality of the octeract-upgrade array. Tier B item 12.
pub const OCTERACT_UPGRADES_LEN: usize = 42;

/// Slice of `GameState` for the octeract upgrades + octeract timer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OcteractUpgradesState {
    /// `player.octeractTimer` — accumulator that drives octeract
    /// generation.
    pub octeract_timer: f64,
    /// Per-upgrade state. The UI/tier maintains the name ↔ index
    /// mapping.
    #[serde(with = "BigArray")]
    pub upgrades: [OcteractUpgrade; OCTERACT_UPGRADES_LEN],
}

impl Default for OcteractUpgradesState {
    fn default() -> Self {
        Self {
            octeract_timer: 0.0,
            upgrades: [OcteractUpgrade::default(); OCTERACT_UPGRADES_LEN],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_42_upgrade_slots() {
        let s = OcteractUpgradesState::default();
        assert_eq!(s.upgrades.len(), OCTERACT_UPGRADES_LEN);
    }
}
