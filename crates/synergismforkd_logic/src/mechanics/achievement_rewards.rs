//! Achievement reward aggregators — the **numeric** subset of the legacy
//! `getAchievementReward` from
//! `legacy/core_split/packages/web_ui/src/Achievements.ts`.
//!
//! In the TS split this lived in `web_ui` (tangled with achievement
//! tooltips / i18n), so the `logic` aggregators received the reward
//! totals as caller-packed `*Pre` fields. Porting the pure numeric part
//! down into `logic` is what lets the tick self-derive those fields from
//! `&GameState`; the achievement *messages* stay a UI concern.
//!
//! Each reward total is a reduce over the achievements that grant it:
//! `getAchievementReward` builds `achievementsByReward[type]` (the list
//! of achievement indices whose `reward` object has that key) and sums
//! (additive rewards) or multiplies (multiplicative rewards) the
//! per-achievement formula, gated by `player.achievements[index]`.
//! Indices are 0-based array positions into the core_split `achievements`
//! array, matching [`crate::state::AchievementsState::achievements`].
//!
//! This module ports the keys consumed by the Phase 2 aggregator
//! `*Pre` bundles (`accelerators`, `acceleratorPower`, `multipliers`,
//! `crystalMultiplier`). Further keys (`accelBoosts`, `taxReduction`,
//! `particleGain`, `antSacrificeUnlock`, …) land alongside the chunks
//! that consume them.

use synergismforkd_bignum::Decimal;

use crate::state::ACHIEVEMENTS_LEN;

/// Inputs to the achievement-reward aggregators. Groups the earned-flag
/// array with the cross-state values the reward formulas read.
#[derive(Debug, Clone, Copy)]
pub struct AchievementRewardInput<'a> {
    /// `player.achievements` — 0 = unowned, non-zero = unlocked.
    pub achievements: &'a [u8; ACHIEVEMENTS_LEN],
    /// `player.{first,second,third,fourth,fifth}OwnedCoin` — coin
    /// producer owned counts, indexed 0..5.
    pub coin_owned: [f64; 5],
    /// `player.prestigePoints` (Diamonds prestige currency).
    pub prestige_points: Decimal,
}

/// `+getAchievementReward('acceleratorPower')` — additive. All flat
/// per-achievement constants (no state dependence).
/// Legacy indices 3 / 10 / 17 / 24 / 31 / 149.
const ACCELERATOR_POWER: [(usize, f64); 6] = [
    (3, 0.001),
    (10, 0.0015),
    (17, 0.002),
    (24, 0.002),
    (31, 0.003),
    (149, 0.01),
];

/// `+getAchievementReward('accelerators')` — additive. Coin-tier
/// achievements grant `floor(owned / 500)`; the rest are flat.
const ACCELERATORS_COIN: [(usize, usize); 5] = [(5, 0), (12, 1), (19, 2), (26, 3), (33, 4)];
const ACCELERATORS_FLAT: [(usize, f64); 7] = [
    (60, 2.0),
    (61, 4.0),
    (62, 8.0),
    (151, 5.0),
    (152, 12.0),
    (153, 25.0),
    (154, 50.0),
];

/// `+getAchievementReward('multipliers')` — additive. Coin-tier
/// achievements grant `floor(owned / 1000)`; the rest are flat.
const MULTIPLIERS_COIN: [(usize, usize); 5] = [(6, 0), (13, 1), (20, 2), (27, 3), (34, 4)];
const MULTIPLIERS_FLAT: [(usize, f64); 8] = [
    (57, 1.0),
    (58, 2.0),
    (59, 4.0),
    (156, 1.0),
    (158, 1.0),
    (159, 3.0),
    (160, 6.0),
    (161, 10.0),
];

/// `+getAchievementReward('accelBoosts')` — additive. Coin-tier
/// achievements grant `floor(owned / 2000)`.
const ACCEL_BOOSTS_COIN: [(usize, usize); 5] = [(7, 0), (14, 1), (21, 2), (28, 3), (35, 4)];

/// Legacy index of the lone `crystalMultiplier` achievement (#37):
/// `() => Math.max(1, Decimal.log(prestigePoints, e))`.
const CRYSTAL_MULTIPLIER_INDEX: usize = 37;

#[inline]
fn earned(achievements: &[u8; ACHIEVEMENTS_LEN], index: usize) -> bool {
    achievements[index] != 0
}

/// `getAchievementReward('acceleratorPower')`.
#[must_use]
pub fn accelerator_power(input: &AchievementRewardInput) -> f64 {
    let mut sum = 0.0;
    for (index, value) in ACCELERATOR_POWER {
        if earned(input.achievements, index) {
            sum += value;
        }
    }
    sum
}

/// `getAchievementReward('accelerators')`.
#[must_use]
pub fn accelerators(input: &AchievementRewardInput) -> f64 {
    let mut sum = 0.0;
    for (index, tier) in ACCELERATORS_COIN {
        if earned(input.achievements, index) {
            sum += (input.coin_owned[tier] / 500.0).floor();
        }
    }
    for (index, value) in ACCELERATORS_FLAT {
        if earned(input.achievements, index) {
            sum += value;
        }
    }
    sum
}

/// `getAchievementReward('multipliers')`.
#[must_use]
pub fn multipliers(input: &AchievementRewardInput) -> f64 {
    let mut sum = 0.0;
    for (index, tier) in MULTIPLIERS_COIN {
        if earned(input.achievements, index) {
            sum += (input.coin_owned[tier] / 1000.0).floor();
        }
    }
    for (index, value) in MULTIPLIERS_FLAT {
        if earned(input.achievements, index) {
            sum += value;
        }
    }
    sum
}

/// `getAchievementReward('accelBoosts')`.
#[must_use]
pub fn accel_boosts(input: &AchievementRewardInput) -> f64 {
    let mut sum = 0.0;
    for (index, tier) in ACCEL_BOOSTS_COIN {
        if earned(input.achievements, index) {
            sum += (input.coin_owned[tier] / 2000.0).floor();
        }
    }
    sum
}

/// `getAchievementReward('crystalMultiplier')` — multiplicative (base 1).
/// The single contributing achievement grants
/// `max(1, ln(prestigePoints))`.
#[must_use]
pub fn crystal_multiplier(input: &AchievementRewardInput) -> f64 {
    if earned(input.achievements, CRYSTAL_MULTIPLIER_INDEX) {
        // `Decimal.log(x, e)` is the natural log; `ln(0)`/`ln(<1)`
        // floors to 1 via the `max`, so default state yields 1.
        1.0_f64.max(input.prestige_points.ln().to_number())
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn earned_array(indices: &[usize]) -> [u8; ACHIEVEMENTS_LEN] {
        let mut a = [0u8; ACHIEVEMENTS_LEN];
        for &i in indices {
            a[i] = 1;
        }
        a
    }

    #[test]
    fn defaults_are_identity() {
        let a = [0u8; ACHIEVEMENTS_LEN];
        let input = AchievementRewardInput {
            achievements: &a,
            coin_owned: [0.0; 5],
            prestige_points: Decimal::zero(),
        };
        assert_eq!(accelerator_power(&input), 0.0);
        assert_eq!(accelerators(&input), 0.0);
        assert_eq!(multipliers(&input), 0.0);
        assert_eq!(crystal_multiplier(&input), 1.0); // product identity
    }

    #[test]
    fn accelerator_power_sums_earned_constants() {
        let a = earned_array(&[3, 31, 149]); // 0.001 + 0.003 + 0.01
        let input = AchievementRewardInput {
            achievements: &a,
            coin_owned: [0.0; 5],
            prestige_points: Decimal::zero(),
        };
        assert!((accelerator_power(&input) - 0.014).abs() < 1e-9);
    }

    #[test]
    fn accelerators_mixes_coin_floor_and_flat() {
        // idx 5 = floor(coin[0]/500); idx 60 = +2; idx 154 = +50.
        let a = earned_array(&[5, 60, 154]);
        let input = AchievementRewardInput {
            achievements: &a,
            coin_owned: [1500.0, 0.0, 0.0, 0.0, 0.0],
            prestige_points: Decimal::zero(),
        };
        // floor(1500/500)=3, +2, +50 = 55
        assert_eq!(accelerators(&input), 55.0);
    }

    #[test]
    fn multipliers_mixes_coin_floor_and_flat() {
        // idx 6 = floor(coin[0]/1000); idx 59 = +4; idx 161 = +10.
        let a = earned_array(&[6, 59, 161]);
        let input = AchievementRewardInput {
            achievements: &a,
            coin_owned: [2500.0, 0.0, 0.0, 0.0, 0.0],
            prestige_points: Decimal::zero(),
        };
        // floor(2500/1000)=2, +4, +10 = 16
        assert_eq!(multipliers(&input), 16.0);
    }

    #[test]
    fn crystal_multiplier_uses_ln_when_earned() {
        let a = earned_array(&[CRYSTAL_MULTIPLIER_INDEX]);
        let input = AchievementRewardInput {
            achievements: &a,
            coin_owned: [0.0; 5],
            prestige_points: Decimal::from_finite(1e100),
        };
        // ln(1e100) = 100 * ln(10) ≈ 230.2585
        assert!((crystal_multiplier(&input) - 230.2585).abs() < 0.01);
    }

    #[test]
    fn crystal_multiplier_floors_at_one() {
        // earned but tiny prestige → max(1, ln(1)) = max(1, 0) = 1.
        let a = earned_array(&[CRYSTAL_MULTIPLIER_INDEX]);
        let input = AchievementRewardInput {
            achievements: &a,
            coin_owned: [0.0; 5],
            prestige_points: Decimal::one(),
        };
        assert_eq!(crystal_multiplier(&input), 1.0);
    }

    #[test]
    fn accel_boosts_sums_coin_floors() {
        // idx 7 = floor(coin[0]/2000); idx 35 = floor(coin[4]/2000).
        let a = earned_array(&[7, 35]);
        let input = AchievementRewardInput {
            achievements: &a,
            coin_owned: [5000.0, 0.0, 0.0, 0.0, 9000.0],
            prestige_points: Decimal::zero(),
        };
        // floor(5000/2000)=2, floor(9000/2000)=4 → 6
        assert_eq!(accel_boosts(&input), 6.0);
    }
}
