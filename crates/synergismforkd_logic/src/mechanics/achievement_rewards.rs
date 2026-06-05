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
//! `crystalMultiplier`, `accelBoosts`), the tax-phase key
//! (`taxReduction`), and the reset-currency key (`particleGain`). Further
//! keys (`antSacrificeUnlock`, …) land alongside the chunks that consume
//! them.

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
    /// `player.challengecompletions[6..=10]` — the five reincarnation /
    /// transcension completion counts read by the `taxReduction`
    /// achievement at index 118 (`0.9925 ^ Σ completions`).
    pub challenge_completions_6_to_10: [f64; 5],
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

/// `getAchievementReward('taxReduction')` — multiplicative (base 1). The
/// flat-constant contributors: indices 45/46 = 0.95, 47 = 0.9,
/// 82/89/96/103/110 = 0.96, 117/124/131 = 0.95. (Index 118 scales with
/// challenge completions and is handled separately.)
const TAX_REDUCTION_FLAT: [(usize, f64); 11] = [
    (45, 0.95),
    (46, 0.95),
    (47, 0.9),
    (82, 0.96),
    (89, 0.96),
    (96, 0.96),
    (103, 0.96),
    (110, 0.96),
    (117, 0.95),
    (124, 0.95),
    (131, 0.95),
];

/// `taxReduction` achievement #118 — `0.9925 ^ (c6+c7+c8+c9+c10)`. The
/// only non-constant tax-reduction reward.
const TAX_REDUCTION_CHALLENGE_INDEX: usize = 118;

/// Legacy index of the lone `particleGain` achievement (#50): `() => 2`.
const PARTICLE_GAIN_INDEX: usize = 50;

/// Legacy index of the lone `antSacrificeUnlock` achievement (#173):
/// `crumbsThisSacrifice >= 1e40`, group `antCrumbs`. A single earned-flag
/// bool reward (`Boolean(player.achievements[173])`), not an aggregator.
const ANT_SACRIFICE_UNLOCK_INDEX: usize = 173;

/// Legacy index of the lone `antELOAdditiveMultiplier` achievement (#484):
/// `() => 0.01`.
const ANT_ELO_ADDITIVE_MULTIPLIER_INDEX: usize = 484;

/// Legacy index of the lone `antELOAdditive` achievement (#485): `() => 25`.
const ANT_ELO_ADDITIVE_INDEX: usize = 485;

/// Legacy index of the lone `antSpeed2UpgradeImprover` achievement (#486):
/// `() => achievementLevel`.
const ANT_SPEED_2_UPGRADE_IMPROVER_INDEX: usize = 486;

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

/// `getAchievementReward('taxReduction')` — multiplicative (base 1). The
/// product of every earned tax-reduction reward: the flat constants in
/// [`TAX_REDUCTION_FLAT`] plus achievement #118's
/// `0.9925 ^ (c6+c7+c8+c9+c10)`.
#[must_use]
pub fn tax_reduction(input: &AchievementRewardInput) -> f64 {
    let mut prod = 1.0;
    for (index, value) in TAX_REDUCTION_FLAT {
        if earned(input.achievements, index) {
            prod *= value;
        }
    }
    if earned(input.achievements, TAX_REDUCTION_CHALLENGE_INDEX) {
        let completions: f64 = input.challenge_completions_6_to_10.iter().sum();
        prod *= 0.9925_f64.powf(completions);
    }
    prod
}

/// `getAchievementReward('particleGain')` — multiplicative (base 1). The
/// single contributing achievement (#50) grants a flat `×2`.
#[must_use]
pub fn particle_gain(input: &AchievementRewardInput) -> f64 {
    if earned(input.achievements, PARTICLE_GAIN_INDEX) {
        2.0
    } else {
        1.0
    }
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

/// `getAchievementReward('antSacrificeUnlock')` — `true` once achievement
/// #173 is earned. Gates the Phase-5 ant-sacrifice automation. Unlike the
/// aggregator rewards this reads a single earned flag, so it takes the
/// achievements array directly rather than the full [`AchievementRewardInput`].
#[must_use]
pub fn ant_sacrifice_unlocked(achievements: &[u8; ACHIEVEMENTS_LEN]) -> bool {
    earned(achievements, ANT_SACRIFICE_UNLOCK_INDEX)
}

/// `getAchievementReward('antELOAdditive')` — `25` once achievement #485 is
/// earned, else `0`. An additive line in the base ant-ELO sum.
#[must_use]
pub fn ant_elo_additive(achievements: &[u8; ACHIEVEMENTS_LEN]) -> f64 {
    if earned(achievements, ANT_ELO_ADDITIVE_INDEX) {
        25.0
    } else {
        0.0
    }
}

/// `getAchievementReward('antELOAdditiveMultiplier')` — `0.01` once
/// achievement #484 is earned, else `0`. An additive line in the ant-ELO
/// multiplier sum.
#[must_use]
pub fn ant_elo_additive_multiplier(achievements: &[u8; ACHIEVEMENTS_LEN]) -> f64 {
    if earned(achievements, ANT_ELO_ADDITIVE_MULTIPLIER_INDEX) {
        0.01
    } else {
        0.0
    }
}

/// `getAchievementReward('antSpeed2UpgradeImprover')` — the player's
/// achievement level once achievement #486 is earned, else `0`. Feeds the
/// AntELO ant-upgrade's sac-count improver.
#[must_use]
pub fn ant_speed_2_upgrade_improver(
    achievements: &[u8; ACHIEVEMENTS_LEN],
    achievement_level: f64,
) -> f64 {
    if earned(achievements, ANT_SPEED_2_UPGRADE_IMPROVER_INDEX) {
        achievement_level
    } else {
        0.0
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

    /// Baseline input — all cross-state values neutral. Tests override the
    /// fields they exercise via struct-update syntax.
    fn input(achievements: &[u8; ACHIEVEMENTS_LEN]) -> AchievementRewardInput<'_> {
        AchievementRewardInput {
            achievements,
            coin_owned: [0.0; 5],
            prestige_points: Decimal::zero(),
            challenge_completions_6_to_10: [0.0; 5],
        }
    }

    #[test]
    fn ant_sacrifice_unlock_tracks_achievement_173() {
        assert!(!ant_sacrifice_unlocked(&earned_array(&[])));
        assert!(ant_sacrifice_unlocked(&earned_array(&[173])));
    }

    #[test]
    fn ant_elo_achievement_rewards_track_their_indices() {
        assert_eq!(ant_elo_additive(&earned_array(&[])), 0.0);
        assert_eq!(ant_elo_additive(&earned_array(&[485])), 25.0);
        assert_eq!(ant_elo_additive_multiplier(&earned_array(&[])), 0.0);
        assert_eq!(ant_elo_additive_multiplier(&earned_array(&[484])), 0.01);
        // antSpeed2UpgradeImprover returns the achievement level when earned.
        assert_eq!(ant_speed_2_upgrade_improver(&earned_array(&[]), 40.0), 0.0);
        assert_eq!(
            ant_speed_2_upgrade_improver(&earned_array(&[486]), 40.0),
            40.0
        );
    }

    #[test]
    fn defaults_are_identity() {
        let a = [0u8; ACHIEVEMENTS_LEN];
        let inp = input(&a);
        assert_eq!(accelerator_power(&inp), 0.0);
        assert_eq!(accelerators(&inp), 0.0);
        assert_eq!(multipliers(&inp), 0.0);
        assert_eq!(crystal_multiplier(&inp), 1.0); // product identity
        assert_eq!(tax_reduction(&inp), 1.0); // product identity
        assert_eq!(particle_gain(&inp), 1.0); // product identity
    }

    #[test]
    fn particle_gain_doubles_when_earned() {
        let a = earned_array(&[PARTICLE_GAIN_INDEX]);
        assert_eq!(particle_gain(&input(&a)), 2.0);
        let none = [0u8; ACHIEVEMENTS_LEN];
        assert_eq!(particle_gain(&input(&none)), 1.0);
    }

    #[test]
    fn accelerator_power_sums_earned_constants() {
        let a = earned_array(&[3, 31, 149]); // 0.001 + 0.003 + 0.01
        assert!((accelerator_power(&input(&a)) - 0.014).abs() < 1e-9);
    }

    #[test]
    fn accelerators_mixes_coin_floor_and_flat() {
        // idx 5 = floor(coin[0]/500); idx 60 = +2; idx 154 = +50.
        let a = earned_array(&[5, 60, 154]);
        let inp = AchievementRewardInput {
            coin_owned: [1500.0, 0.0, 0.0, 0.0, 0.0],
            ..input(&a)
        };
        // floor(1500/500)=3, +2, +50 = 55
        assert_eq!(accelerators(&inp), 55.0);
    }

    #[test]
    fn multipliers_mixes_coin_floor_and_flat() {
        // idx 6 = floor(coin[0]/1000); idx 59 = +4; idx 161 = +10.
        let a = earned_array(&[6, 59, 161]);
        let inp = AchievementRewardInput {
            coin_owned: [2500.0, 0.0, 0.0, 0.0, 0.0],
            ..input(&a)
        };
        // floor(2500/1000)=2, +4, +10 = 16
        assert_eq!(multipliers(&inp), 16.0);
    }

    #[test]
    fn crystal_multiplier_uses_ln_when_earned() {
        let a = earned_array(&[CRYSTAL_MULTIPLIER_INDEX]);
        let inp = AchievementRewardInput {
            prestige_points: Decimal::from_finite(1e100),
            ..input(&a)
        };
        // ln(1e100) = 100 * ln(10) ≈ 230.2585
        assert!((crystal_multiplier(&inp) - 230.2585).abs() < 0.01);
    }

    #[test]
    fn crystal_multiplier_floors_at_one() {
        // earned but tiny prestige → max(1, ln(1)) = max(1, 0) = 1.
        let a = earned_array(&[CRYSTAL_MULTIPLIER_INDEX]);
        let inp = AchievementRewardInput {
            prestige_points: Decimal::one(),
            ..input(&a)
        };
        assert_eq!(crystal_multiplier(&inp), 1.0);
    }

    #[test]
    fn accel_boosts_sums_coin_floors() {
        // idx 7 = floor(coin[0]/2000); idx 35 = floor(coin[4]/2000).
        let a = earned_array(&[7, 35]);
        let inp = AchievementRewardInput {
            coin_owned: [5000.0, 0.0, 0.0, 0.0, 9000.0],
            ..input(&a)
        };
        // floor(5000/2000)=2, floor(9000/2000)=4 → 6
        assert_eq!(accel_boosts(&inp), 6.0);
    }

    #[test]
    fn tax_reduction_multiplies_earned_flat_constants() {
        // idx 45 = 0.95, idx 47 = 0.9, idx 82 = 0.96.
        let a = earned_array(&[45, 47, 82]);
        let expected = 0.95 * 0.9 * 0.96;
        assert!((tax_reduction(&input(&a)) - expected).abs() < 1e-12);
    }

    #[test]
    fn tax_reduction_index_118_scales_with_challenge_completions() {
        // idx 118 → 0.9925 ^ (c6+c7+c8+c9+c10).
        let a = earned_array(&[TAX_REDUCTION_CHALLENGE_INDEX]);
        let inp = AchievementRewardInput {
            challenge_completions_6_to_10: [10.0, 5.0, 5.0, 0.0, 0.0],
            ..input(&a)
        };
        let expected = 0.9925_f64.powf(20.0);
        assert!((tax_reduction(&inp) - expected).abs() < 1e-12);
    }

    #[test]
    fn tax_reduction_combines_flat_and_challenge_factors() {
        let a = earned_array(&[46, TAX_REDUCTION_CHALLENGE_INDEX, 131]);
        let inp = AchievementRewardInput {
            challenge_completions_6_to_10: [4.0, 0.0, 0.0, 0.0, 0.0],
            ..input(&a)
        };
        let expected = 0.95 * 0.95 * 0.9925_f64.powf(4.0);
        assert!((tax_reduction(&inp) - expected).abs() < 1e-12);
    }
}
