//! Cube + platonic upgrade-level state slice.
//!
//! Holds `player.cubeUpgrades` (~80 upgrade levels) and
//! `player.platonicUpgrades` (~25 upgrade levels). Both are
//! 1-indexed in the legacy with index 0 unused — preserved here.
//! Read by virtually every formula module that takes a
//! `cubeUpgradeN`/`platonicUpgradeN` scalar input.

/// Slice of `GameState` holding cube + platonic upgrade levels.
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

/// Fixed cardinality of the cube-upgrade array — `80 + 1` for the
/// legacy 1-indexed convention. Tier B item 12.
pub const CUBE_UPGRADES_LEN: usize = 81;

/// Fixed cardinality of the platonic-upgrade array — `25 + 1` for
/// the legacy 1-indexed convention. Tier B item 12.
pub const PLATONIC_UPGRADES_LEN: usize = 26;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CubeUpgradeLevelsState {
    /// `player.cubeUpgrades` — per-cube-upgrade level. 1-indexed
    /// (index 0 unused) to match the legacy shape.
    #[serde(with = "BigArray")]
    pub cube_upgrades: [f64; CUBE_UPGRADES_LEN],
    /// `player.platonicUpgrades` — per-platonic-upgrade level.
    /// 1-indexed. Stays inside serde's default 0..=32 length window,
    /// so no `BigArray` attribute needed.
    pub platonic_upgrades: [f64; PLATONIC_UPGRADES_LEN],
}

impl Default for CubeUpgradeLevelsState {
    fn default() -> Self {
        Self {
            cube_upgrades: [0.0; CUBE_UPGRADES_LEN],
            platonic_upgrades: [0.0; PLATONIC_UPGRADES_LEN],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_legacy_widths() {
        let s = CubeUpgradeLevelsState::default();
        assert_eq!(s.cube_upgrades.len(), CUBE_UPGRADES_LEN);
        assert_eq!(s.platonic_upgrades.len(), PLATONIC_UPGRADES_LEN);
    }
}
