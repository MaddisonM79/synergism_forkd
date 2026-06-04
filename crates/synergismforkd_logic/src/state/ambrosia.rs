//! Ambrosia (blueberry) state slice.
//!
//! Mirrors `player.ambrosia`, `player.lifetimeAmbrosia`,
//! `player.blueberryTime`, `player.ambrosiaRNG`,
//! `player.spentBlueberries`, and `player.ambrosiaUpgrades`. Backs
//! [`crate::mechanics::ambrosia`] and
//! [`crate::mechanics::blueberry_upgrades`].

/// One ambrosia upgrade's per-player state. Mirrors
/// `player.ambrosiaUpgrades.<name>`.
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct AmbrosiaUpgrade {
    /// Purchased level.
    pub level: f64,
    /// Accumulated free levels.
    pub free_level: f64,
}

/// Fixed cardinality of the ambrosia-upgrade array — one slot per key of
/// the legacy `ambrosiaUpgrades` const
/// (`legacy/core_split/packages/web_ui/src/BlueberryUpgrades.ts`), verified
/// by brace-walk against both legacy snapshots (36 keys, identical order).
pub const AMBROSIA_UPGRADES_LEN: usize = 36;

// Ambrosia (blueberry) upgrade name → index convention. Index `i` is the
// i-th key of the legacy `ambrosiaUpgrades` const (the object
// `player.ambrosiaUpgrades` is built from). This is the canonical mapping
// the UI tier must match; logic indexes `AmbrosiaState::upgrades` through
// these constants. Const names drop the leading `ambrosia` and prefix
// `AMBROSIA_`.
/// `ambrosiaTutorial` — index 0.
pub const AMBROSIA_TUTORIAL: usize = 0;
/// `ambrosiaQuarks1` — index 1.
pub const AMBROSIA_QUARKS_1: usize = 1;
/// `ambrosiaCubes1` — index 2.
pub const AMBROSIA_CUBES_1: usize = 2;
/// `ambrosiaLuck1` — index 3.
pub const AMBROSIA_LUCK_1: usize = 3;
/// `ambrosiaQuarkCube1` — index 4.
pub const AMBROSIA_QUARK_CUBE_1: usize = 4;
/// `ambrosiaLuckCube1` — index 5.
pub const AMBROSIA_LUCK_CUBE_1: usize = 5;
/// `ambrosiaCubeQuark1` — index 6.
pub const AMBROSIA_CUBE_QUARK_1: usize = 6;
/// `ambrosiaLuckQuark1` — index 7.
pub const AMBROSIA_LUCK_QUARK_1: usize = 7;
/// `ambrosiaCubeLuck1` — index 8.
pub const AMBROSIA_CUBE_LUCK_1: usize = 8;
/// `ambrosiaQuarkLuck1` — index 9.
pub const AMBROSIA_QUARK_LUCK_1: usize = 9;
/// `ambrosiaQuarks2` — index 10.
pub const AMBROSIA_QUARKS_2: usize = 10;
/// `ambrosiaCubes2` — index 11.
pub const AMBROSIA_CUBES_2: usize = 11;
/// `ambrosiaLuck2` — index 12.
pub const AMBROSIA_LUCK_2: usize = 12;
/// `ambrosiaQuarks3` — index 13.
pub const AMBROSIA_QUARKS_3: usize = 13;
/// `ambrosiaCubes3` — index 14.
pub const AMBROSIA_CUBES_3: usize = 14;
/// `ambrosiaLuck3` — index 15.
pub const AMBROSIA_LUCK_3: usize = 15;
/// `ambrosiaLuck4` — index 16.
pub const AMBROSIA_LUCK_4: usize = 16;
/// `ambrosiaPatreon` — index 17.
pub const AMBROSIA_PATREON: usize = 17;
/// `ambrosiaObtainium1` — index 18.
pub const AMBROSIA_OBTAINIUM_1: usize = 18;
/// `ambrosiaOffering1` — index 19.
pub const AMBROSIA_OFFERING_1: usize = 19;
/// `ambrosiaHyperflux` — index 20.
pub const AMBROSIA_HYPERFLUX: usize = 20;
/// `ambrosiaBaseOffering1` — index 21.
pub const AMBROSIA_BASE_OFFERING_1: usize = 21;
/// `ambrosiaBaseObtainium1` — index 22.
pub const AMBROSIA_BASE_OBTAINIUM_1: usize = 22;
/// `ambrosiaBaseOffering2` — index 23.
pub const AMBROSIA_BASE_OFFERING_2: usize = 23;
/// `ambrosiaBaseObtainium2` — index 24.
pub const AMBROSIA_BASE_OBTAINIUM_2: usize = 24;
/// `ambrosiaSingReduction1` — index 25.
pub const AMBROSIA_SING_REDUCTION_1: usize = 25;
/// `ambrosiaInfiniteShopUpgrades1` — index 26.
pub const AMBROSIA_INFINITE_SHOP_UPGRADES_1: usize = 26;
/// `ambrosiaInfiniteShopUpgrades2` — index 27.
pub const AMBROSIA_INFINITE_SHOP_UPGRADES_2: usize = 27;
/// `ambrosiaSingReduction2` — index 28.
pub const AMBROSIA_SING_REDUCTION_2: usize = 28;
/// `ambrosiaTalismanBonusRuneLevel` — index 29.
pub const AMBROSIA_TALISMAN_BONUS_RUNE_LEVEL: usize = 29;
/// `ambrosiaRuneOOMBonus` — index 30.
pub const AMBROSIA_RUNE_OOM_BONUS: usize = 30;
/// `ambrosiaBrickOfLead` — index 31.
pub const AMBROSIA_BRICK_OF_LEAD: usize = 31;
/// `ambrosiaFreeLuckUpgrades` — index 32.
pub const AMBROSIA_FREE_LUCK_UPGRADES: usize = 32;
/// `ambrosiaFreeGenerationUpgrades` — index 33.
pub const AMBROSIA_FREE_GENERATION_UPGRADES: usize = 33;
/// `ambrosiaFreeRedLuckUpgrades` — index 34.
pub const AMBROSIA_FREE_RED_LUCK_UPGRADES: usize = 34;
/// `ambrosiaFreeQuarkUpgrades` — index 35.
pub const AMBROSIA_FREE_QUARK_UPGRADES: usize = 35;

/// Slice of `GameState` for the ambrosia/blueberry feature.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AmbrosiaState {
    /// `player.ambrosia` — current balance (resets on use).
    pub ambrosia: f64,
    /// `player.lifetimeAmbrosia` — all-time ambrosia earned.
    pub lifetime_ambrosia: f64,
    /// `player.blueberryTime` — generation-bar accumulator (seconds).
    pub blueberry_time: f64,
    /// `G.ambrosiaTimer` — sub-bar 1/8 s granule accumulator that feeds
    /// `blueberry_time`. Distinct from `blueberry_time` itself.
    pub ambrosia_timer_g: f64,
    /// `player.spentBlueberries` — count of blueberries allocated
    /// to upgrades.
    pub spent_blueberries: f64,
    /// Per-upgrade state. Indexed by the `AMBROSIA_*` constants
    /// (index = legacy `ambrosiaUpgrades` key order).
    #[serde(with = "BigArray")]
    pub upgrades: [AmbrosiaUpgrade; AMBROSIA_UPGRADES_LEN],
}

impl Default for AmbrosiaState {
    fn default() -> Self {
        Self {
            ambrosia: 0.0,
            lifetime_ambrosia: 0.0,
            blueberry_time: 0.0,
            ambrosia_timer_g: 0.0,
            spent_blueberries: 0.0,
            upgrades: [AmbrosiaUpgrade::default(); AMBROSIA_UPGRADES_LEN],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_36_upgrade_slots() {
        let s = AmbrosiaState::default();
        assert_eq!(s.upgrades.len(), AMBROSIA_UPGRADES_LEN);
        assert_eq!(s.lifetime_ambrosia, 0.0);
    }

    #[test]
    fn ambrosia_index_convention_sentinels() {
        // Coverage: the last named slot pins the array length.
        assert_eq!(AMBROSIA_FREE_QUARK_UPGRADES, AMBROSIA_UPGRADES_LEN - 1);
        // Mult anchors (legacy `ambrosiaUpgrades` key order).
        assert_eq!(AMBROSIA_LUCK_1, 3);
        assert_eq!(AMBROSIA_PATREON, 17);
        assert_eq!(AMBROSIA_BRICK_OF_LEAD, 31);
    }
}
