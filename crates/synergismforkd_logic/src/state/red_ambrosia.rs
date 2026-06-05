//! Red-ambrosia state slice.
//!
//! Mirrors `player.redAmbrosia`, `player.lifetimeRedAmbrosia`,
//! `player.redAmbrosiaTime`, `player.redAmbrosiaRNG`,
//! `player.spentRedAmbrosia`, and `player.redAmbrosiaUpgrades`. Backs
//! [`crate::mechanics::red_ambrosia_bonuses`] and
//! [`crate::mechanics::red_ambrosia_upgrades`].

/// One red-ambrosia upgrade's per-player state. Mirrors
/// `player.redAmbrosiaUpgrades.<name>` — most red-ambrosia upgrades
/// are single-`level` entries (no free-level concept).
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct RedAmbrosiaUpgrade {
    /// Purchased level.
    pub level: f64,
}

/// Fixed cardinality of the red-ambrosia-upgrade array — one slot per key
/// of the legacy `redAmbrosiaUpgrades` const
/// (`legacy/core_split/packages/web_ui/src/RedAmbrosiaUpgrades.ts`),
/// verified by brace-walk against both legacy snapshots (29 keys, identical
/// order). (29 fits inside serde's default 0..=32 length window — no
/// `BigArray` attribute needed.)
pub const RED_AMBROSIA_UPGRADES_LEN: usize = 29;

// Red-ambrosia upgrade name → index convention. Index `i` is the i-th key
// of the legacy `redAmbrosiaUpgrades` const (the object
// `player.redAmbrosiaUpgrades` is built from). This is the canonical
// mapping the UI tier must match; logic indexes `RedAmbrosiaState::upgrades`
// through these constants.
/// `tutorial` — index 0.
pub const RED_AMBROSIA_TUTORIAL: usize = 0;
/// `conversionImprovement1` — index 1.
pub const RED_AMBROSIA_CONVERSION_IMPROVEMENT_1: usize = 1;
/// `conversionImprovement2` — index 2.
pub const RED_AMBROSIA_CONVERSION_IMPROVEMENT_2: usize = 2;
/// `conversionImprovement3` — index 3.
pub const RED_AMBROSIA_CONVERSION_IMPROVEMENT_3: usize = 3;
/// `freeTutorialLevels` — index 4.
pub const RED_AMBROSIA_FREE_TUTORIAL_LEVELS: usize = 4;
/// `freeLevelsRow2` — index 5.
pub const RED_AMBROSIA_FREE_LEVELS_ROW_2: usize = 5;
/// `freeLevelsRow3` — index 6.
pub const RED_AMBROSIA_FREE_LEVELS_ROW_3: usize = 6;
/// `freeLevelsRow4` — index 7.
pub const RED_AMBROSIA_FREE_LEVELS_ROW_4: usize = 7;
/// `freeLevelsRow5` — index 8.
pub const RED_AMBROSIA_FREE_LEVELS_ROW_5: usize = 8;
/// `blueberryGenerationSpeed` — index 9.
pub const RED_AMBROSIA_BLUEBERRY_GENERATION_SPEED: usize = 9;
/// `regularLuck` — index 10.
pub const RED_AMBROSIA_REGULAR_LUCK: usize = 10;
/// `redGenerationSpeed` — index 11.
pub const RED_AMBROSIA_RED_GENERATION_SPEED: usize = 11;
/// `redLuck` — index 12.
pub const RED_AMBROSIA_RED_LUCK: usize = 12;
/// `redAmbrosiaCube` — index 13.
pub const RED_AMBROSIA_RED_AMBROSIA_CUBE: usize = 13;
/// `redAmbrosiaObtainium` — index 14.
pub const RED_AMBROSIA_RED_AMBROSIA_OBTAINIUM: usize = 14;
/// `redAmbrosiaOffering` — index 15.
pub const RED_AMBROSIA_RED_AMBROSIA_OFFERING: usize = 15;
/// `redAmbrosiaCubeImprover` — index 16.
pub const RED_AMBROSIA_RED_AMBROSIA_CUBE_IMPROVER: usize = 16;
/// `viscount` — index 17.
pub const RED_AMBROSIA_VISCOUNT: usize = 17;
/// `infiniteShopUpgrades` — index 18.
pub const RED_AMBROSIA_INFINITE_SHOP_UPGRADES: usize = 18;
/// `redAmbrosiaAccelerator` — index 19.
pub const RED_AMBROSIA_RED_AMBROSIA_ACCELERATOR: usize = 19;
/// `regularLuck2` — index 20.
pub const RED_AMBROSIA_REGULAR_LUCK_2: usize = 20;
/// `blueberryGenerationSpeed2` — index 21.
pub const RED_AMBROSIA_BLUEBERRY_GENERATION_SPEED_2: usize = 21;
/// `salvageYinYang` — index 22.
pub const RED_AMBROSIA_SALVAGE_YIN_YANG: usize = 22;
/// `blueberries` — index 23.
pub const RED_AMBROSIA_BLUEBERRIES: usize = 23;
/// `redAmbrosiaFreeAccumulator` — index 24.
pub const RED_AMBROSIA_RED_AMBROSIA_FREE_ACCUMULATOR: usize = 24;
/// `freeOfferingUpgrades` — index 25.
pub const RED_AMBROSIA_FREE_OFFERING_UPGRADES: usize = 25;
/// `freeObtainiumUpgrades` — index 26.
pub const RED_AMBROSIA_FREE_OBTAINIUM_UPGRADES: usize = 26;
/// `freeCubeUpgrades` — index 27.
pub const RED_AMBROSIA_FREE_CUBE_UPGRADES: usize = 27;
/// `freeSpeedUpgrades` — index 28.
pub const RED_AMBROSIA_FREE_SPEED_UPGRADES: usize = 28;

/// Slice of `GameState` for the red-ambrosia feature.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RedAmbrosiaState {
    /// `player.redAmbrosia` — current balance.
    pub red_ambrosia: f64,
    /// `player.lifetimeRedAmbrosia` — all-time earned.
    pub lifetime_red_ambrosia: f64,
    /// `player.redAmbrosiaTime` — generation-bar accumulator (seconds).
    pub red_ambrosia_time: f64,
    /// `G.redAmbrosiaTimer` — sub-bar 1/8 s granule accumulator that
    /// feeds `red_ambrosia_time`. Distinct from `red_ambrosia_time`.
    pub red_ambrosia_timer_g: f64,
    /// `player.spentRedAmbrosia` — count allocated to upgrades.
    pub spent_red_ambrosia: f64,
    /// Per-upgrade state. Indexed by the `RED_AMBROSIA_*` constants
    /// (index = legacy `redAmbrosiaUpgrades` key order).
    pub upgrades: [RedAmbrosiaUpgrade; RED_AMBROSIA_UPGRADES_LEN],
}

impl Default for RedAmbrosiaState {
    fn default() -> Self {
        Self {
            red_ambrosia: 0.0,
            lifetime_red_ambrosia: 0.0,
            red_ambrosia_time: 0.0,
            red_ambrosia_timer_g: 0.0,
            spent_red_ambrosia: 0.0,
            upgrades: [RedAmbrosiaUpgrade::default(); RED_AMBROSIA_UPGRADES_LEN],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_29_upgrade_slots() {
        let s = RedAmbrosiaState::default();
        assert_eq!(s.upgrades.len(), RED_AMBROSIA_UPGRADES_LEN);
    }

    #[test]
    fn red_ambrosia_index_convention_sentinels() {
        // Coverage: the last named slot pins the array length.
        assert_eq!(
            RED_AMBROSIA_FREE_SPEED_UPGRADES,
            RED_AMBROSIA_UPGRADES_LEN - 1
        );
        // Mult anchors (legacy `redAmbrosiaUpgrades` key order).
        assert_eq!(RED_AMBROSIA_BLUEBERRY_GENERATION_SPEED, 9);
        assert_eq!(RED_AMBROSIA_REGULAR_LUCK, 10);
        assert_eq!(RED_AMBROSIA_VISCOUNT, 17);
    }
}
