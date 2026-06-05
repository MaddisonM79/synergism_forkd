//! Shop state slice — the ~83 named shop upgrades plus toggles.
//!
//! Mirrors `player.shopUpgrades`, `player.shopPotionsConsumed`, and
//! `player.shopBuyMaxToggle`. Backs [`crate::mechanics::shop_costs`]
//! and [`crate::mechanics::shop_upgrades`].

/// Shop buy-max toggle. Mirrors `player.shopBuyMaxToggle` in the
/// legacy schema.
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShopBuyMaxMode {
    /// Buy exactly one per click.
    #[default]
    One,
    /// Buy the max affordable up to the upgrade cap.
    Max,
}

/// Fixed cardinality of the shop-upgrade array — one slot per key of
/// the legacy `player.shopUpgrades` object
/// (`legacy/core_split/packages/web_ui/src/Synergism.ts`), verified by
/// brace-walk against both legacy snapshots (83 keys, identical order).
pub const SHOP_UPGRADES_LEN: usize = 83;

// Shop-upgrade name → index convention. Index `i` is the i-th key of the
// legacy `player.shopUpgrades` object; this is the canonical mapping the
// UI tier must match. Logic indexes `ShopState::upgrades` through these
// constants. Const names follow the legacy key verbatim (the leading
// `shop` is not doubled).
/// `offeringPotion` — index 0.
pub const SHOP_OFFERING_POTION: usize = 0;
/// `obtainiumPotion` — index 1.
pub const SHOP_OBTAINIUM_POTION: usize = 1;
/// `offeringEX` — index 2.
pub const SHOP_OFFERING_EX: usize = 2;
/// `offeringAuto` — index 3.
pub const SHOP_OFFERING_AUTO: usize = 3;
/// `obtainiumEX` — index 4.
pub const SHOP_OBTAINIUM_EX: usize = 4;
/// `obtainiumAuto` — index 5.
pub const SHOP_OBTAINIUM_AUTO: usize = 5;
/// `instantChallenge` — index 6.
pub const SHOP_INSTANT_CHALLENGE: usize = 6;
/// `antSpeed` — index 7.
pub const SHOP_ANT_SPEED: usize = 7;
/// `cashGrab` — index 8.
pub const SHOP_CASH_GRAB: usize = 8;
/// `shopTalisman` — index 9.
pub const SHOP_TALISMAN: usize = 9;
/// `seasonPass` — index 10.
pub const SHOP_SEASON_PASS: usize = 10;
/// `challengeExtension` — index 11.
pub const SHOP_CHALLENGE_EXTENSION: usize = 11;
/// `challengeTome` — index 12.
pub const SHOP_CHALLENGE_TOME: usize = 12;
/// `cubeToQuark` — index 13.
pub const SHOP_CUBE_TO_QUARK: usize = 13;
/// `tesseractToQuark` — index 14.
pub const SHOP_TESSERACT_TO_QUARK: usize = 14;
/// `hypercubeToQuark` — index 15.
pub const SHOP_HYPERCUBE_TO_QUARK: usize = 15;
/// `seasonPass2` — index 16.
pub const SHOP_SEASON_PASS_2: usize = 16;
/// `seasonPass3` — index 17.
pub const SHOP_SEASON_PASS_3: usize = 17;
/// `chronometer` — index 18.
pub const SHOP_CHRONOMETER: usize = 18;
/// `infiniteAscent` — index 19.
pub const SHOP_INFINITE_ASCENT: usize = 19;
/// `calculator` — index 20.
pub const SHOP_CALCULATOR: usize = 20;
/// `calculator2` — index 21.
pub const SHOP_CALCULATOR_2: usize = 21;
/// `calculator3` — index 22.
pub const SHOP_CALCULATOR_3: usize = 22;
/// `calculator4` — index 23.
pub const SHOP_CALCULATOR_4: usize = 23;
/// `calculator5` — index 24.
pub const SHOP_CALCULATOR_5: usize = 24;
/// `calculator6` — index 25.
pub const SHOP_CALCULATOR_6: usize = 25;
/// `calculator7` — index 26.
pub const SHOP_CALCULATOR_7: usize = 26;
/// `constantEX` — index 27.
pub const SHOP_CONSTANT_EX: usize = 27;
/// `powderEX` — index 28.
pub const SHOP_POWDER_EX: usize = 28;
/// `chronometer2` — index 29.
pub const SHOP_CHRONOMETER_2: usize = 29;
/// `chronometer3` — index 30.
pub const SHOP_CHRONOMETER_3: usize = 30;
/// `seasonPassY` — index 31.
pub const SHOP_SEASON_PASS_Y: usize = 31;
/// `seasonPassZ` — index 32.
pub const SHOP_SEASON_PASS_Z: usize = 32;
/// `challengeTome2` — index 33.
pub const SHOP_CHALLENGE_TOME_2: usize = 33;
/// `instantChallenge2` — index 34.
pub const SHOP_INSTANT_CHALLENGE_2: usize = 34;
/// `cashGrab2` — index 35.
pub const SHOP_CASH_GRAB_2: usize = 35;
/// `chronometerZ` — index 36.
pub const SHOP_CHRONOMETER_Z: usize = 36;
/// `cubeToQuarkAll` — index 37.
pub const SHOP_CUBE_TO_QUARK_ALL: usize = 37;
/// `offeringEX2` — index 38.
pub const SHOP_OFFERING_EX_2: usize = 38;
/// `obtainiumEX2` — index 39.
pub const SHOP_OBTAINIUM_EX_2: usize = 39;
/// `seasonPassLost` — index 40.
pub const SHOP_SEASON_PASS_LOST: usize = 40;
/// `powderAuto` — index 41.
pub const SHOP_POWDER_AUTO: usize = 41;
/// `challenge15Auto` — index 42.
pub const SHOP_CHALLENGE_15_AUTO: usize = 42;
/// `extraWarp` — index 43.
pub const SHOP_EXTRA_WARP: usize = 43;
/// `autoWarp` — index 44.
pub const SHOP_AUTO_WARP: usize = 44;
/// `improveQuarkHept` — index 45.
pub const SHOP_IMPROVE_QUARK_HEPT: usize = 45;
/// `improveQuarkHept2` — index 46.
pub const SHOP_IMPROVE_QUARK_HEPT_2: usize = 46;
/// `improveQuarkHept3` — index 47.
pub const SHOP_IMPROVE_QUARK_HEPT_3: usize = 47;
/// `improveQuarkHept4` — index 48.
pub const SHOP_IMPROVE_QUARK_HEPT_4: usize = 48;
/// `shopImprovedDaily` — index 49.
pub const SHOP_IMPROVED_DAILY: usize = 49;
/// `shopImprovedDaily2` — index 50.
pub const SHOP_IMPROVED_DAILY_2: usize = 50;
/// `shopImprovedDaily3` — index 51.
pub const SHOP_IMPROVED_DAILY_3: usize = 51;
/// `shopImprovedDaily4` — index 52.
pub const SHOP_IMPROVED_DAILY_4: usize = 52;
/// `offeringEX3` — index 53.
pub const SHOP_OFFERING_EX_3: usize = 53;
/// `obtainiumEX3` — index 54.
pub const SHOP_OBTAINIUM_EX_3: usize = 54;
/// `improveQuarkHept5` — index 55.
pub const SHOP_IMPROVE_QUARK_HEPT_5: usize = 55;
/// `seasonPassInfinity` — index 56.
pub const SHOP_SEASON_PASS_INFINITY: usize = 56;
/// `chronometerInfinity` — index 57.
pub const SHOP_CHRONOMETER_INFINITY: usize = 57;
/// `shopSingularityPenaltyDebuff` — index 58.
pub const SHOP_SINGULARITY_PENALTY_DEBUFF: usize = 58;
/// `shopAmbrosiaLuckMultiplier4` — index 59.
pub const SHOP_AMBROSIA_LUCK_MULTIPLIER_4: usize = 59;
/// `shopOcteractAmbrosiaLuck` — index 60.
pub const SHOP_OCTERACT_AMBROSIA_LUCK: usize = 60;
/// `shopAmbrosiaGeneration1` — index 61.
pub const SHOP_AMBROSIA_GENERATION_1: usize = 61;
/// `shopAmbrosiaGeneration2` — index 62.
pub const SHOP_AMBROSIA_GENERATION_2: usize = 62;
/// `shopAmbrosiaGeneration3` — index 63.
pub const SHOP_AMBROSIA_GENERATION_3: usize = 63;
/// `shopAmbrosiaGeneration4` — index 64.
pub const SHOP_AMBROSIA_GENERATION_4: usize = 64;
/// `shopAmbrosiaLuck1` — index 65.
pub const SHOP_AMBROSIA_LUCK_1: usize = 65;
/// `shopAmbrosiaLuck2` — index 66.
pub const SHOP_AMBROSIA_LUCK_2: usize = 66;
/// `shopAmbrosiaLuck3` — index 67.
pub const SHOP_AMBROSIA_LUCK_3: usize = 67;
/// `shopAmbrosiaLuck4` — index 68.
pub const SHOP_AMBROSIA_LUCK_4: usize = 68;
/// `shopCashGrabUltra` — index 69.
pub const SHOP_CASH_GRAB_ULTRA: usize = 69;
/// `shopAmbrosiaAccelerator` — index 70.
pub const SHOP_AMBROSIA_ACCELERATOR: usize = 70;
/// `shopEXUltra` — index 71.
pub const SHOP_EX_ULTRA: usize = 71;
/// `shopChronometerS` — index 72.
pub const SHOP_CHRONOMETER_S: usize = 72;
/// `shopAmbrosiaUltra` — index 73.
pub const SHOP_AMBROSIA_ULTRA: usize = 73;
/// `shopSingularitySpeedup` — index 74.
pub const SHOP_SINGULARITY_SPEEDUP: usize = 74;
/// `shopSingularityPotency` — index 75.
pub const SHOP_SINGULARITY_POTENCY: usize = 75;
/// `shopSadisticRune` — index 76.
pub const SHOP_SADISTIC_RUNE: usize = 76;
/// `shopRedLuck1` — index 77.
pub const SHOP_RED_LUCK_1: usize = 77;
/// `shopRedLuck2` — index 78.
pub const SHOP_RED_LUCK_2: usize = 78;
/// `shopRedLuck3` — index 79.
pub const SHOP_RED_LUCK_3: usize = 79;
/// `shopInfiniteShopUpgrades` — index 80.
pub const SHOP_INFINITE_SHOP_UPGRADES: usize = 80;
/// `shopHorseShoe` — index 81.
pub const SHOP_HORSE_SHOE: usize = 81;
/// `shopPanthema` — index 82.
pub const SHOP_PANTHEMA: usize = 82;

/// Slice of `GameState` for the shop feature.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ShopState {
    /// Per-upgrade purchased level. Indexed by the `SHOP_*` constants
    /// (index = legacy `player.shopUpgrades` key order).
    #[serde(with = "BigArray")]
    pub upgrades: [f64; SHOP_UPGRADES_LEN],
    /// `player.shopPotionsConsumed` — lifetime potion-use count.
    pub shop_potions_consumed: f64,
    /// `player.shopBuyMaxToggle`.
    pub shop_buy_max_toggle: ShopBuyMaxMode,
}

impl Default for ShopState {
    fn default() -> Self {
        Self {
            upgrades: [0.0; SHOP_UPGRADES_LEN],
            shop_potions_consumed: 0.0,
            shop_buy_max_toggle: ShopBuyMaxMode::One,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_83_upgrade_slots() {
        let s = ShopState::default();
        assert_eq!(s.upgrades.len(), SHOP_UPGRADES_LEN);
        assert!(matches!(s.shop_buy_max_toggle, ShopBuyMaxMode::One));
    }

    #[test]
    fn shop_index_convention_sentinels() {
        // Coverage: the last named slot pins the array length.
        assert_eq!(SHOP_PANTHEMA, SHOP_UPGRADES_LEN - 1);
        // StatLine anchors (legacy `player.shopUpgrades` key order).
        assert_eq!(SHOP_OFFERING_POTION, 0);
        assert_eq!(SHOP_OBTAINIUM_POTION, 1);
        assert_eq!(SHOP_OFFERING_AUTO, 3);
        assert_eq!(SHOP_CONSTANT_EX, 27);
        assert_eq!(SHOP_CHRONOMETER, 18);
        assert_eq!(SHOP_CHRONOMETER_2, 29);
        assert_eq!(SHOP_CHRONOMETER_3, 30);
        assert_eq!(SHOP_CHRONOMETER_Z, 36);
        assert_eq!(SHOP_CHRONOMETER_INFINITY, 57);
        assert_eq!(SHOP_CHRONOMETER_S, 72);
    }
}
