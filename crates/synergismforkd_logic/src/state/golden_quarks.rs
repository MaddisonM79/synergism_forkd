//! Golden-quark state slice — the GQ currency and the ~80 named
//! GQ upgrades.
//!
//! Mirrors `player.goldenQuarks` and `player.singularityUpgrades`
//! from the legacy schema. Backs [`crate::mechanics::gq_upgrade_cost`],
//! [`crate::mechanics::gq_upgrade_levels`], and
//! [`crate::mechanics::golden_quark_upgrades`].
//!
//! The legacy schema keys upgrades by name; this slice indexes them
//! by position. The `GQ_*` constants below give the name → index
//! mapping (index = legacy `goldenQuarkUpgrades` key order). Each
//! entry carries the full GQ-upgrade shape (level, freeLevel,
//! maxLevel, canExceedCap, qualityOfLife, specialCostForm) so the
//! cost / effect dispatchers don't need to look it up elsewhere.

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use synergismforkd_bignum::Decimal;

/// Fixed cardinality of the GQ-upgrade array — one slot per key of the
/// legacy `goldenQuarkUpgrades` const
/// (`legacy/core_split/packages/web_ui/src/singularity.ts`), verified by
/// brace-walk against both legacy snapshots (80 keys, identical order).
pub const GOLDEN_QUARK_UPGRADES_LEN: usize = 80;

// GQ-upgrade name → index convention. Index `i` is the i-th key of the
// legacy `goldenQuarkUpgrades` const (the object `player.goldenQuarkUpgrades`
// is built from). This is the canonical mapping the UI tier must match;
// logic indexes `GoldenQuarksState::upgrades` through these constants.
/// `goldenQuarks1` — index 0.
pub const GQ_GOLDEN_QUARKS_1: usize = 0;
/// `goldenQuarks2` — index 1.
pub const GQ_GOLDEN_QUARKS_2: usize = 1;
/// `goldenQuarks3` — index 2.
pub const GQ_GOLDEN_QUARKS_3: usize = 2;
/// `starterPack` — index 3.
pub const GQ_STARTER_PACK: usize = 3;
/// `wowPass` — index 4.
pub const GQ_WOW_PASS: usize = 4;
/// `cookies` — index 5.
pub const GQ_COOKIES: usize = 5;
/// `cookies2` — index 6.
pub const GQ_COOKIES_2: usize = 6;
/// `cookies3` — index 7.
pub const GQ_COOKIES_3: usize = 7;
/// `cookies4` — index 8.
pub const GQ_COOKIES_4: usize = 8;
/// `cookies5` — index 9.
pub const GQ_COOKIES_5: usize = 9;
/// `ascensions` — index 10.
pub const GQ_ASCENSIONS: usize = 10;
/// `corruptionFourteen` — index 11.
pub const GQ_CORRUPTION_FOURTEEN: usize = 11;
/// `corruptionFifteen` — index 12.
pub const GQ_CORRUPTION_FIFTEEN: usize = 12;
/// `singOfferings1` — index 13.
pub const GQ_SING_OFFERINGS_1: usize = 13;
/// `singOfferings2` — index 14.
pub const GQ_SING_OFFERINGS_2: usize = 14;
/// `singOfferings3` — index 15.
pub const GQ_SING_OFFERINGS_3: usize = 15;
/// `singObtainium1` — index 16.
pub const GQ_SING_OBTAINIUM_1: usize = 16;
/// `singObtainium2` — index 17.
pub const GQ_SING_OBTAINIUM_2: usize = 17;
/// `singObtainium3` — index 18.
pub const GQ_SING_OBTAINIUM_3: usize = 18;
/// `singCubes1` — index 19.
pub const GQ_SING_CUBES_1: usize = 19;
/// `singCubes2` — index 20.
pub const GQ_SING_CUBES_2: usize = 20;
/// `singCubes3` — index 21.
pub const GQ_SING_CUBES_3: usize = 21;
/// `singCitadel` — index 22.
pub const GQ_SING_CITADEL: usize = 22;
/// `singCitadel2` — index 23.
pub const GQ_SING_CITADEL_2: usize = 23;
/// `octeractUnlock` — index 24.
pub const GQ_OCTERACT_UNLOCK: usize = 24;
/// `singOcteractPatreonBonus` — index 25.
pub const GQ_SING_OCTERACT_PATREON_BONUS: usize = 25;
/// `offeringAutomatic` — index 26.
pub const GQ_OFFERING_AUTOMATIC: usize = 26;
/// `intermediatePack` — index 27.
pub const GQ_INTERMEDIATE_PACK: usize = 27;
/// `advancedPack` — index 28.
pub const GQ_ADVANCED_PACK: usize = 28;
/// `expertPack` — index 29.
pub const GQ_EXPERT_PACK: usize = 29;
/// `masterPack` — index 30.
pub const GQ_MASTER_PACK: usize = 30;
/// `divinePack` — index 31.
pub const GQ_DIVINE_PACK: usize = 31;
/// `wowPass2` — index 32.
pub const GQ_WOW_PASS_2: usize = 32;
/// `wowPass3` — index 33.
pub const GQ_WOW_PASS_3: usize = 33;
/// `potionBuff` — index 34.
pub const GQ_POTION_BUFF: usize = 34;
/// `potionBuff2` — index 35.
pub const GQ_POTION_BUFF_2: usize = 35;
/// `potionBuff3` — index 36.
pub const GQ_POTION_BUFF_3: usize = 36;
/// `singChallengeExtension` — index 37.
pub const GQ_SING_CHALLENGE_EXTENSION: usize = 37;
/// `singChallengeExtension2` — index 38.
pub const GQ_SING_CHALLENGE_EXTENSION_2: usize = 38;
/// `singChallengeExtension3` — index 39.
pub const GQ_SING_CHALLENGE_EXTENSION_3: usize = 39;
/// `singQuarkImprover1` — index 40.
pub const GQ_SING_QUARK_IMPROVER_1: usize = 40;
/// `singQuarkHepteract` — index 41.
pub const GQ_SING_QUARK_HEPTERACT: usize = 41;
/// `singQuarkHepteract2` — index 42.
pub const GQ_SING_QUARK_HEPTERACT_2: usize = 42;
/// `singQuarkHepteract3` — index 43.
pub const GQ_SING_QUARK_HEPTERACT_3: usize = 43;
/// `singOcteractGain` — index 44.
pub const GQ_SING_OCTERACT_GAIN: usize = 44;
/// `singOcteractGain2` — index 45.
pub const GQ_SING_OCTERACT_GAIN_2: usize = 45;
/// `singOcteractGain3` — index 46.
pub const GQ_SING_OCTERACT_GAIN_3: usize = 46;
/// `singOcteractGain4` — index 47.
pub const GQ_SING_OCTERACT_GAIN_4: usize = 47;
/// `singOcteractGain5` — index 48.
pub const GQ_SING_OCTERACT_GAIN_5: usize = 48;
/// `platonicTau` — index 49.
pub const GQ_PLATONIC_TAU: usize = 49;
/// `platonicAlpha` — index 50.
pub const GQ_PLATONIC_ALPHA: usize = 50;
/// `platonicDelta` — index 51.
pub const GQ_PLATONIC_DELTA: usize = 51;
/// `platonicPhi` — index 52.
pub const GQ_PLATONIC_PHI: usize = 52;
/// `singFastForward` — index 53.
pub const GQ_SING_FAST_FORWARD: usize = 53;
/// `singFastForward2` — index 54.
pub const GQ_SING_FAST_FORWARD_2: usize = 54;
/// `singAscensionSpeed` — index 55.
pub const GQ_SING_ASCENSION_SPEED: usize = 55;
/// `singAscensionSpeed2` — index 56.
pub const GQ_SING_ASCENSION_SPEED_2: usize = 56;
/// `ultimatePen` — index 57.
pub const GQ_ULTIMATE_PEN: usize = 57;
/// `halfMind` — index 58.
pub const GQ_HALF_MIND: usize = 58;
/// `oneMind` — index 59.
pub const GQ_ONE_MIND: usize = 59;
/// `wowPass4` — index 60.
pub const GQ_WOW_PASS_4: usize = 60;
/// `blueberries` — index 61.
pub const GQ_BLUEBERRIES: usize = 61;
/// `singAmbrosiaLuck` — index 62.
pub const GQ_SING_AMBROSIA_LUCK: usize = 62;
/// `singAmbrosiaLuck2` — index 63.
pub const GQ_SING_AMBROSIA_LUCK_2: usize = 63;
/// `singAmbrosiaLuck3` — index 64.
pub const GQ_SING_AMBROSIA_LUCK_3: usize = 64;
/// `singAmbrosiaLuck4` — index 65.
pub const GQ_SING_AMBROSIA_LUCK_4: usize = 65;
/// `singAmbrosiaGeneration` — index 66.
pub const GQ_SING_AMBROSIA_GENERATION: usize = 66;
/// `singAmbrosiaGeneration2` — index 67.
pub const GQ_SING_AMBROSIA_GENERATION_2: usize = 67;
/// `singAmbrosiaGeneration3` — index 68.
pub const GQ_SING_AMBROSIA_GENERATION_3: usize = 68;
/// `singAmbrosiaGeneration4` — index 69.
pub const GQ_SING_AMBROSIA_GENERATION_4: usize = 69;
/// `singBonusTokens1` — index 70.
pub const GQ_SING_BONUS_TOKENS_1: usize = 70;
/// `singBonusTokens2` — index 71.
pub const GQ_SING_BONUS_TOKENS_2: usize = 71;
/// `singBonusTokens3` — index 72.
pub const GQ_SING_BONUS_TOKENS_3: usize = 72;
/// `singBonusTokens4` — index 73.
pub const GQ_SING_BONUS_TOKENS_4: usize = 73;
/// `singInfiniteShopUpgrades` — index 74.
pub const GQ_SING_INFINITE_SHOP_UPGRADES: usize = 74;
/// `singTalismanBonusRunes1` — index 75.
pub const GQ_SING_TALISMAN_BONUS_RUNES_1: usize = 75;
/// `singTalismanBonusRunes2` — index 76.
pub const GQ_SING_TALISMAN_BONUS_RUNES_2: usize = 76;
/// `singTalismanBonusRunes3` — index 77.
pub const GQ_SING_TALISMAN_BONUS_RUNES_3: usize = 77;
/// `singTalismanBonusRunes4` — index 78.
pub const GQ_SING_TALISMAN_BONUS_RUNES_4: usize = 78;
/// `favoriteUpgrade` — index 79.
pub const GQ_FAVORITE_UPGRADE: usize = 79;

/// Special-cost-form selector for one GQ upgrade — pinned here
/// alongside the state so the storage matches the dispatch shape
/// in [`crate::mechanics::gq_upgrade_cost::GQUpgradeSpecialCostForm`].
/// Stored as a `u8` for `Copy` + small footprint:
/// `0 = Exponential2, 1 = Cubic, 2 = Quadratic, 3 = None`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StoredSpecialCostForm {
    /// `Exponential2` form — soft sqrt(overcap) × `2^level`.
    Exponential2,
    /// `Cubic` form — overcap × `((level+1)^3 - level^3)` delta.
    Cubic,
    /// `Quadratic` form — overcap × `((level+1)^2 - level^2)` delta.
    Quadratic,
    /// Default linear branch (no special form).
    #[default]
    None,
}

/// One GQ upgrade's per-player state. Mirrors the legacy
/// `player.singularityUpgrades.<name>` shape.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct GoldenQuarkUpgrade {
    /// Purchased level.
    pub level: f64,
    /// Accumulated free levels.
    pub free_level: f64,
    /// Base maxLevel (`-1` for unlimited).
    pub max_level: f64,
    /// Whether this upgrade benefits from overclock-perk cap
    /// expansion.
    pub can_exceed_cap: bool,
    /// Quality-of-life flag — when true, the upgrade survives
    /// `noSingularityUpgrades` and `sadisticPrequel`.
    pub quality_of_life: bool,
    /// Cost-formula shape.
    pub special_cost_form: StoredSpecialCostForm,
    /// Base coefficient (`costPerLevel`) — used by the cost formula.
    pub cost_per_level: f64,
}

/// Slice of `GameState` for the golden-quark feature.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GoldenQuarksState {
    /// `player.goldenQuarks` — the currency balance.
    pub golden_quarks: Decimal,
    /// `player.quarksThisSingularity` — drives `calculate_base_golden_quarks`.
    pub quarks_this_singularity: f64,
    /// `player.goldenQuarksTimer` — GQ-export accumulator (seconds);
    /// disabled when `export_gq_per_hour == 0`, else clamped to 168 h.
    pub golden_quarks_timer: f64,
    /// Per-upgrade state. Indexed by the `GQ_*` constants (index =
    /// legacy `goldenQuarkUpgrades` key order).
    #[serde(with = "BigArray")]
    pub upgrades: [GoldenQuarkUpgrade; GOLDEN_QUARK_UPGRADES_LEN],
}

impl Default for GoldenQuarksState {
    fn default() -> Self {
        Self {
            golden_quarks: Decimal::zero(),
            quarks_this_singularity: 0.0,
            golden_quarks_timer: 0.0,
            upgrades: [GoldenQuarkUpgrade::default(); GOLDEN_QUARK_UPGRADES_LEN],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_80_upgrade_slots() {
        let s = GoldenQuarksState::default();
        assert_eq!(s.upgrades.len(), GOLDEN_QUARK_UPGRADES_LEN);
        assert_eq!(s.golden_quarks.to_number(), 0.0);
    }

    #[test]
    fn gq_index_convention_sentinels() {
        // Coverage: the last named slot pins the array length.
        assert_eq!(GQ_FAVORITE_UPGRADE, GOLDEN_QUARK_UPGRADES_LEN - 1);
        // StatLine anchors (legacy `goldenQuarkUpgrades` key order).
        assert_eq!(GQ_INTERMEDIATE_PACK, 27);
        assert_eq!(GQ_SING_ASCENSION_SPEED, 55);
        assert_eq!(GQ_SING_ASCENSION_SPEED_2, 56);
    }

    #[test]
    fn upgrade_default_is_zeroed() {
        let u = GoldenQuarkUpgrade::default();
        assert_eq!(u.level, 0.0);
        assert!(!u.can_exceed_cap);
        assert!(matches!(u.special_cost_form, StoredSpecialCostForm::None));
    }
}
