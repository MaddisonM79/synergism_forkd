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

/// Fixed cardinality of the octeract-upgrade array — one slot per key
/// of the legacy `octeractUpgrades` const
/// (`legacy/core_split/packages/web_ui/src/Octeracts.ts`), verified by
/// brace-walk against both legacy snapshots (47 keys, identical order).
pub const OCTERACT_UPGRADES_LEN: usize = 47;

// Octeract-upgrade name → index convention. Index `i` is the i-th key of
// the legacy `octeractUpgrades` const (the object `player.octUpgrades` is
// built from). This is the canonical mapping the UI tier must match;
// logic indexes `OcteractUpgradesState::upgrades` through these constants.
// Const names drop the leading `octeract` and prefix `OCTERACT_`.
/// `octeractStarter` — index 0.
pub const OCTERACT_STARTER: usize = 0;
/// `octeractGain` — index 1.
pub const OCTERACT_GAIN: usize = 1;
/// `octeractGain2` — index 2.
pub const OCTERACT_GAIN_2: usize = 2;
/// `octeractQuarkGain` — index 3.
pub const OCTERACT_QUARK_GAIN: usize = 3;
/// `octeractQuarkGain2` — index 4.
pub const OCTERACT_QUARK_GAIN_2: usize = 4;
/// `octeractCorruption` — index 5.
pub const OCTERACT_CORRUPTION: usize = 5;
/// `octeractGQCostReduce` — index 6.
pub const OCTERACT_GQ_COST_REDUCE: usize = 6;
/// `octeractExportQuarks` — index 7.
pub const OCTERACT_EXPORT_QUARKS: usize = 7;
/// `octeractImprovedDaily` — index 8.
pub const OCTERACT_IMPROVED_DAILY: usize = 8;
/// `octeractImprovedDaily2` — index 9.
pub const OCTERACT_IMPROVED_DAILY_2: usize = 9;
/// `octeractImprovedDaily3` — index 10.
pub const OCTERACT_IMPROVED_DAILY_3: usize = 10;
/// `octeractImprovedQuarkHept` — index 11.
pub const OCTERACT_IMPROVED_QUARK_HEPT: usize = 11;
/// `octeractImprovedGlobalSpeed` — index 12.
pub const OCTERACT_IMPROVED_GLOBAL_SPEED: usize = 12;
/// `octeractImprovedAscensionSpeed` — index 13.
pub const OCTERACT_IMPROVED_ASCENSION_SPEED: usize = 13;
/// `octeractImprovedAscensionSpeed2` — index 14.
pub const OCTERACT_IMPROVED_ASCENSION_SPEED_2: usize = 14;
/// `octeractImprovedFree` — index 15.
pub const OCTERACT_IMPROVED_FREE: usize = 15;
/// `octeractImprovedFree2` — index 16.
pub const OCTERACT_IMPROVED_FREE_2: usize = 16;
/// `octeractImprovedFree3` — index 17.
pub const OCTERACT_IMPROVED_FREE_3: usize = 17;
/// `octeractImprovedFree4` — index 18.
pub const OCTERACT_IMPROVED_FREE_4: usize = 18;
/// `octeractSingUpgradeCap` — index 19.
pub const OCTERACT_SING_UPGRADE_CAP: usize = 19;
/// `octeractOfferings1` — index 20.
pub const OCTERACT_OFFERINGS_1: usize = 20;
/// `octeractObtainium1` — index 21.
pub const OCTERACT_OBTAINIUM_1: usize = 21;
/// `octeractAscensions` — index 22.
pub const OCTERACT_ASCENSIONS: usize = 22;
/// `octeractAscensions2` — index 23.
pub const OCTERACT_ASCENSIONS_2: usize = 23;
/// `octeractAscensionsOcteractGain` — index 24.
pub const OCTERACT_ASCENSIONS_OCTERACT_GAIN: usize = 24;
/// `octeractFastForward` — index 25.
pub const OCTERACT_FAST_FORWARD: usize = 25;
/// `octeractAutoPotionSpeed` — index 26.
pub const OCTERACT_AUTO_POTION_SPEED: usize = 26;
/// `octeractAutoPotionEfficiency` — index 27.
pub const OCTERACT_AUTO_POTION_EFFICIENCY: usize = 27;
/// `octeractOneMindImprover` — index 28.
pub const OCTERACT_ONE_MIND_IMPROVER: usize = 28;
/// `octeractAmbrosiaLuck` — index 29.
pub const OCTERACT_AMBROSIA_LUCK: usize = 29;
/// `octeractAmbrosiaLuck2` — index 30.
pub const OCTERACT_AMBROSIA_LUCK_2: usize = 30;
/// `octeractAmbrosiaLuck3` — index 31.
pub const OCTERACT_AMBROSIA_LUCK_3: usize = 31;
/// `octeractAmbrosiaLuck4` — index 32.
pub const OCTERACT_AMBROSIA_LUCK_4: usize = 32;
/// `octeractAmbrosiaGeneration` — index 33.
pub const OCTERACT_AMBROSIA_GENERATION: usize = 33;
/// `octeractAmbrosiaGeneration2` — index 34.
pub const OCTERACT_AMBROSIA_GENERATION_2: usize = 34;
/// `octeractAmbrosiaGeneration3` — index 35.
pub const OCTERACT_AMBROSIA_GENERATION_3: usize = 35;
/// `octeractAmbrosiaGeneration4` — index 36.
pub const OCTERACT_AMBROSIA_GENERATION_4: usize = 36;
/// `octeractBonusTokens1` — index 37.
pub const OCTERACT_BONUS_TOKENS_1: usize = 37;
/// `octeractBonusTokens2` — index 38.
pub const OCTERACT_BONUS_TOKENS_2: usize = 38;
/// `octeractBonusTokens3` — index 39.
pub const OCTERACT_BONUS_TOKENS_3: usize = 39;
/// `octeractBonusTokens4` — index 40.
pub const OCTERACT_BONUS_TOKENS_4: usize = 40;
/// `octeractBlueberries` — index 41.
pub const OCTERACT_BLUEBERRIES: usize = 41;
/// `octeractInfiniteShopUpgrades` — index 42.
pub const OCTERACT_INFINITE_SHOP_UPGRADES: usize = 42;
/// `octeractTalismanLevelCap1` — index 43.
pub const OCTERACT_TALISMAN_LEVEL_CAP_1: usize = 43;
/// `octeractTalismanLevelCap2` — index 44.
pub const OCTERACT_TALISMAN_LEVEL_CAP_2: usize = 44;
/// `octeractTalismanLevelCap3` — index 45.
pub const OCTERACT_TALISMAN_LEVEL_CAP_3: usize = 45;
/// `octeractTalismanLevelCap4` — index 46.
pub const OCTERACT_TALISMAN_LEVEL_CAP_4: usize = 46;

/// Slice of `GameState` for the octeract upgrades + octeract timer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OcteractUpgradesState {
    /// `player.octeractTimer` — accumulator that drives octeract
    /// generation.
    pub octeract_timer: f64,
    /// Per-upgrade state. Indexed by the `OCTERACT_*` constants
    /// (index = legacy `octeractUpgrades` key order).
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
    fn default_has_47_upgrade_slots() {
        let s = OcteractUpgradesState::default();
        assert_eq!(s.upgrades.len(), OCTERACT_UPGRADES_LEN);
    }

    #[test]
    fn octeract_index_convention_sentinels() {
        // Coverage: the last named slot pins the array length.
        assert_eq!(OCTERACT_TALISMAN_LEVEL_CAP_4, OCTERACT_UPGRADES_LEN - 1);
        // StatLine anchors (legacy `octeractUpgrades` key order).
        assert_eq!(OCTERACT_IMPROVED_GLOBAL_SPEED, 12);
        assert_eq!(OCTERACT_IMPROVED_ASCENSION_SPEED, 13);
        assert_eq!(OCTERACT_IMPROVED_ASCENSION_SPEED_2, 14);
        assert_eq!(OCTERACT_AUTO_POTION_SPEED, 26);
    }
}
