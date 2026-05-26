//! Cube + platonic upgrade-level state slice.
//!
//! Holds `player.cubeUpgrades` (~80 upgrade levels) and
//! `player.platonicUpgrades` (~25 upgrade levels). Both are
//! 1-indexed in the legacy with index 0 unused — preserved here.
//! Read by virtually every formula module that takes a
//! `cubeUpgradeN`/`platonicUpgradeN` scalar input.

/// Slice of `GameState` holding cube + platonic upgrade levels.
#[derive(Debug, Clone, PartialEq)]
pub struct CubeUpgradeLevelsState {
    /// `player.cubeUpgrades` — per-cube-upgrade level. 1-indexed
    /// (index 0 unused) to match the legacy shape.
    pub cube_upgrades: Vec<f64>,
    /// `player.platonicUpgrades` — per-platonic-upgrade level.
    /// 1-indexed.
    pub platonic_upgrades: Vec<f64>,
}

impl CubeUpgradeLevelsState {
    /// Build with `n_cube_upgrades + 1` cube slots and
    /// `n_platonic_upgrades + 1` platonic slots.
    #[must_use]
    pub fn new(n_cube_upgrades: usize, n_platonic_upgrades: usize) -> Self {
        Self {
            cube_upgrades: vec![0.0; n_cube_upgrades + 1],
            platonic_upgrades: vec![0.0; n_platonic_upgrades + 1],
        }
    }
}

impl Default for CubeUpgradeLevelsState {
    fn default() -> Self {
        // Legacy synergism has 80 cube upgrades and 25 platonic
        // upgrades (the highest indices referenced in the existing
        // formula ports).
        Self::new(80, 25)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_legacy_widths() {
        let s = CubeUpgradeLevelsState::default();
        assert_eq!(s.cube_upgrades.len(), 81);
        assert_eq!(s.platonic_upgrades.len(), 26);
    }
}
