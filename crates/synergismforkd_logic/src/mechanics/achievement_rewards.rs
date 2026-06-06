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

/// Legacy indices of the five `antSpeed` reward achievements (a product):
/// #169 grants `log10(crumbs + 10)`, #171/#172 grant `1.2`/`1.25`, #173 grants
/// `1.4` (the same achievement as [`ANT_SACRIFICE_UNLOCK_INDEX`]), and #174
/// grants `1 + immortalELO / 1000`.
const ANT_SPEED_CRUMBS_INDEX: usize = 169;
const ANT_SPEED_FLAT_1_2_INDEX: usize = 171;
const ANT_SPEED_FLAT_1_25_INDEX: usize = 172;
const ANT_SPEED_IMMORTAL_ELO_INDEX: usize = 174;

/// Legacy indices of the two `ascensionScore` reward achievements:
/// #259 grants `1.01 ^ hepteracts.abyss.TIMES_CAP_EXTENDED` and #267 grants
/// `1 + min(log10(ascendShards + 1) / 1e5, 1)`.
const ASCENSION_SCORE_ABYSS_INDEX: usize = 259;
const ASCENSION_SCORE_SHARDS_INDEX: usize = 267;

/// `hepteracts.abyss.TIMES_CAP_EXTENDED` — the abyss hepteract craft is
/// unported (no times-cap-extended state field), so its exponent is
/// neutral-defaulted to 0 (→ `1.01^0 = 1`), faithful at current state. Swap
/// for the real craft value once the abyss hepteract lands.
const ABYSS_TIMES_CAP_EXTENDED: f64 = 0.0;

/// Legacy index of the lone `obtainiumBonus` reward achievement (#468):
/// `1 + 0.02 · max(1, 1 + floor(log10(reincarnationCount)))`.
const OBTAINIUM_BONUS_INDEX: usize = 468;

/// Legacy index of the lone `ascensionRewardScaling` achievement (#204):
/// first challenge-12 completion, group `challenge12`. A single earned-flag
/// bool reward gating the `allCubeStats` AscensionTime overflow term.
const ASCENSION_REWARD_SCALING_INDEX: usize = 204;

/// `wowCubeGain` reward achievement indices (multiplicative product, feeds the
/// AchievementBonus line of `allWowCubeStats`):
/// #189 `1 + 2·min(1, ascensionCount/5e8)`, #193 `1 + log10(ascendShards+1)/400`,
/// #195 `1 + 249·min(1, log10(ascendShards+1)/1e5)` (the same achievement also
/// grants `wowTesseractGain`), #254 flat `1.1`.
const WOW_CUBE_GAIN_ASC_COUNT_INDEX: usize = 189;
const WOW_CUBE_GAIN_SHARDS_400_INDEX: usize = 193;
const WOW_GAIN_SHARDS_249_INDEX: usize = 195;
const WOW_CUBE_GAIN_FLAT_INDEX: usize = 254;

/// `wowTesseractGain`: #195 (shared with `wowCubeGain`) + #255 flat `1.1`.
const WOW_TESSERACT_GAIN_FLAT_INDEX: usize = 255;

/// `wowHypercubeGain`: #253 flat `1.1`.
const WOW_HYPERCUBE_GAIN_FLAT_INDEX: usize = 253;

/// `wowPlatonicGain`: #196 `1 + 19·min(1, log10(ascendShards+1)/1e5)`,
/// #223 `1 + 2·min(1, ascensionCount/2.674e9)`, #256 flat `1.1`.
const WOW_PLATONIC_GAIN_SHARDS_INDEX: usize = 196;
const WOW_PLATONIC_GAIN_ASC_COUNT_INDEX: usize = 223;
const WOW_PLATONIC_GAIN_FLAT_INDEX: usize = 256;

/// `wowHepteractGain`: #258 flat `1.1`, #270 `1 + min(log10(ascendShards+1)/1e6, 1)`.
const WOW_HEPTERACT_GAIN_FLAT_INDEX: usize = 258;
const WOW_HEPTERACT_GAIN_SHARDS_INDEX: usize = 270;

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

/// `getAchievementReward('ascensionRewardScaling')` — `true` once achievement
/// #204 is earned. Gates the `allCubeStats` AscensionTime line's overflow term
/// `(1 + max(0, ascensionCounter / threshold - 1))`. A single earned flag.
#[must_use]
pub fn ascension_reward_scaling(achievements: &[u8; ACHIEVEMENTS_LEN]) -> bool {
    earned(achievements, ASCENSION_REWARD_SCALING_INDEX)
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

/// `+getAchievementReward('antSpeed')` — multiplicative (base 1). The product
/// of the five contributing achievements: #169 grants `log10(crumbs + 10)`,
/// #171/#172 grant `1.2`/`1.25`, #173 grants `1.4`, and #174 grants
/// `1 + immortalELO / 1000`. Feeds the `AchievementBonus` line of the
/// ant-speed StatLine product (`antSpeedStats`). At default state (no
/// achievements earned) this is the product identity `1.0`; even with #169
/// earned at `crumbs = 0` the term is `log10(10) = 1`.
#[must_use]
pub fn ant_speed(achievements: &[u8; ACHIEVEMENTS_LEN], crumbs: Decimal, immortal_elo: f64) -> f64 {
    let mut prod = 1.0;
    if earned(achievements, ANT_SPEED_CRUMBS_INDEX) {
        // `Decimal.log(crumbs + 10, 10)` — log base 10.
        prod *= (crumbs + Decimal::from_finite(10.0)).log10().to_number();
    }
    if earned(achievements, ANT_SPEED_FLAT_1_2_INDEX) {
        prod *= 1.2;
    }
    if earned(achievements, ANT_SPEED_FLAT_1_25_INDEX) {
        prod *= 1.25;
    }
    if earned(achievements, ANT_SACRIFICE_UNLOCK_INDEX) {
        prod *= 1.4;
    }
    if earned(achievements, ANT_SPEED_IMMORTAL_ELO_INDEX) {
        prod *= 1.0 + immortal_elo / 1000.0;
    }
    prod
}

/// `getAchievementReward('ascensionScore')` — multiplicative (base 1). The
/// product of the two contributing achievements: #259 grants
/// `1.01 ^ TIMES_CAP_EXTENDED` (abyss hepteract unported → exponent 0 → 1.0,
/// see [`ABYSS_TIMES_CAP_EXTENDED`]) and #267 grants
/// `1 + min(log10(ascendShards + 1) / 1e5, 1)`. Feeds the ascension-score
/// bonus multiplier (`compute_ascension_score_bonus_multiplier`). At default
/// state (no achievements earned, `ascendShards = 0`) this is the product
/// identity `1.0`.
#[must_use]
pub fn ascension_score(achievements: &[u8; ACHIEVEMENTS_LEN], ascend_shards: Decimal) -> f64 {
    let mut prod = 1.0;
    if earned(achievements, ASCENSION_SCORE_ABYSS_INDEX) {
        prod *= 1.01_f64.powf(ABYSS_TIMES_CAP_EXTENDED);
    }
    if earned(achievements, ASCENSION_SCORE_SHARDS_INDEX) {
        // `Decimal.log(ascendShards + 1, 10)` — log base 10.
        let log10 = (ascend_shards + Decimal::one()).log10().to_number();
        prod *= 1.0 + (log10 / 1e5).min(1.0);
    }
    prod
}

/// `+getAchievementReward('obtainiumBonus')` — multiplicative (base 1). The
/// single contributing achievement (#468) grants
/// `1 + 0.02 · max(1, 1 + floor(log10(reincarnationCount)))`. Feeds the
/// `AchievementBonus` line of the obtainium base-multiplier StatLine
/// product (`allObtainiumStats`). At default state (achievement unearned)
/// this is the product identity `1.0`.
///
/// `log10(0) = -inf` (JS `Math.log10(0)` and Rust `f64::log10` agree), so at
/// `reincarnationCount = 0` the `max(1, …)` clamps the term to `1.02` — but
/// the gate means it only contributes once #468 is earned.
#[must_use]
pub fn obtainium_bonus(achievements: &[u8; ACHIEVEMENTS_LEN], reincarnation_count: f64) -> f64 {
    if earned(achievements, OBTAINIUM_BONUS_INDEX) {
        1.0 + 0.02 * 1.0_f64.max(1.0 + reincarnation_count.log10().floor())
    } else {
        1.0
    }
}

/// `log10(ascendShards + 1)` — the `Decimal.log(player.ascendShards.add(1), 10)`
/// shared by the cube-gain achievement rewards below.
#[inline]
fn ascend_shards_log10(ascend_shards: Decimal) -> f64 {
    (ascend_shards + Decimal::one()).log10().to_number()
}

/// `getAchievementReward('wowCubeGain')` — multiplicative (base 1). Feeds the
/// AchievementBonus line of `allWowCubeStats`. Identity `1.0` at default.
#[must_use]
pub fn wow_cube_gain(
    achievements: &[u8; ACHIEVEMENTS_LEN],
    ascension_count: f64,
    ascend_shards: Decimal,
) -> f64 {
    let log10 = ascend_shards_log10(ascend_shards);
    let mut prod = 1.0;
    if earned(achievements, WOW_CUBE_GAIN_ASC_COUNT_INDEX) {
        prod *= 1.0 + 2.0 * (ascension_count / 5e8).min(1.0);
    }
    if earned(achievements, WOW_CUBE_GAIN_SHARDS_400_INDEX) {
        prod *= 1.0 + log10 / 400.0;
    }
    if earned(achievements, WOW_GAIN_SHARDS_249_INDEX) {
        prod *= 1.0 + 249.0 * (log10 / 1e5).min(1.0);
    }
    if earned(achievements, WOW_CUBE_GAIN_FLAT_INDEX) {
        prod *= 1.1;
    }
    prod
}

/// `getAchievementReward('wowTesseractGain')` — multiplicative (base 1). Feeds
/// the AchievementBonus line of `allTesseractStats`. Identity `1.0` at default.
#[must_use]
pub fn wow_tesseract_gain(achievements: &[u8; ACHIEVEMENTS_LEN], ascend_shards: Decimal) -> f64 {
    let mut prod = 1.0;
    if earned(achievements, WOW_GAIN_SHARDS_249_INDEX) {
        prod *= 1.0 + 249.0 * (ascend_shards_log10(ascend_shards) / 1e5).min(1.0);
    }
    if earned(achievements, WOW_TESSERACT_GAIN_FLAT_INDEX) {
        prod *= 1.1;
    }
    prod
}

/// `getAchievementReward('wowHypercubeGain')` — multiplicative (base 1). Feeds
/// the AchievementBonus line of `allHypercubeStats`. Identity `1.0` at default.
#[must_use]
pub fn wow_hypercube_gain(achievements: &[u8; ACHIEVEMENTS_LEN]) -> f64 {
    if earned(achievements, WOW_HYPERCUBE_GAIN_FLAT_INDEX) {
        1.1
    } else {
        1.0
    }
}

/// `getAchievementReward('wowPlatonicGain')` — multiplicative (base 1). Feeds
/// the AchievementBonus line of `allPlatonicCubeStats`. Identity `1.0` at default.
#[must_use]
pub fn wow_platonic_gain(
    achievements: &[u8; ACHIEVEMENTS_LEN],
    ascension_count: f64,
    ascend_shards: Decimal,
) -> f64 {
    let mut prod = 1.0;
    if earned(achievements, WOW_PLATONIC_GAIN_SHARDS_INDEX) {
        prod *= 1.0 + 19.0 * (ascend_shards_log10(ascend_shards) / 1e5).min(1.0);
    }
    if earned(achievements, WOW_PLATONIC_GAIN_ASC_COUNT_INDEX) {
        prod *= 1.0 + 2.0 * (ascension_count / 2.674e9).min(1.0);
    }
    if earned(achievements, WOW_PLATONIC_GAIN_FLAT_INDEX) {
        prod *= 1.1;
    }
    prod
}

/// `getAchievementReward('wowHepteractGain')` — multiplicative (base 1). Feeds
/// the AchievementBonus line of `allHepteractCubeStats`. Identity `1.0` at default.
#[must_use]
pub fn wow_hepteract_gain(achievements: &[u8; ACHIEVEMENTS_LEN], ascend_shards: Decimal) -> f64 {
    let mut prod = 1.0;
    if earned(achievements, WOW_HEPTERACT_GAIN_FLAT_INDEX) {
        prod *= 1.1;
    }
    if earned(achievements, WOW_HEPTERACT_GAIN_SHARDS_INDEX) {
        prod *= 1.0 + (ascend_shards_log10(ascend_shards) / 1e6).min(1.0);
    }
    prod
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
    fn ascension_reward_scaling_tracks_achievement_204() {
        assert!(!ascension_reward_scaling(&earned_array(&[])));
        assert!(ascension_reward_scaling(&earned_array(&[204])));
    }

    #[test]
    fn wow_gain_rewards_identity_at_default() {
        let none = earned_array(&[]);
        assert_eq!(wow_cube_gain(&none, 0.0, Decimal::zero()), 1.0);
        assert_eq!(wow_tesseract_gain(&none, Decimal::zero()), 1.0);
        assert_eq!(wow_hypercube_gain(&none), 1.0);
        assert_eq!(wow_platonic_gain(&none, 0.0, Decimal::zero()), 1.0);
        assert_eq!(wow_hepteract_gain(&none, Decimal::zero()), 1.0);
    }

    #[test]
    fn wow_gain_rewards_apply_earned_factors() {
        // #254 flat 1.1 on wowCubeGain.
        assert!((wow_cube_gain(&earned_array(&[254]), 0.0, Decimal::zero()) - 1.1).abs() < 1e-12);
        // #189: ascensionCount at the 5e8 cap → 1 + 2·min(1, 1) = 3.
        assert!((wow_cube_gain(&earned_array(&[189]), 5e8, Decimal::zero()) - 3.0).abs() < 1e-9);
        // #195 grants both wowCubeGain and wowTesseractGain (same value).
        let shards = Decimal::from_finite(1e10); // log10(1e10 + 1) ≈ 10
        let c = wow_cube_gain(&earned_array(&[195]), 0.0, shards);
        let t = wow_tesseract_gain(&earned_array(&[195]), shards);
        assert!((c - t).abs() < 1e-9 && c > 1.0);
        // Flat 1.1 rewards.
        assert!((wow_hypercube_gain(&earned_array(&[253])) - 1.1).abs() < 1e-12);
        assert!(
            (wow_platonic_gain(&earned_array(&[256]), 0.0, Decimal::zero()) - 1.1).abs() < 1e-12
        );
        assert!((wow_hepteract_gain(&earned_array(&[258]), Decimal::zero()) - 1.1).abs() < 1e-12);
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
    fn ascension_score_is_product_identity_at_default() {
        // Nothing earned → product identity.
        assert_eq!(ascension_score(&earned_array(&[]), Decimal::zero()), 1.0);
        // #259 earned → 1.01 ^ 0 = 1.0 (abyss term neutral-defaulted).
        assert_eq!(ascension_score(&earned_array(&[259]), Decimal::zero()), 1.0);
        // #267 earned at ascendShards = 0 → 1 + min(log10(1) / 1e5, 1) = 1.0.
        assert_eq!(ascension_score(&earned_array(&[267]), Decimal::zero()), 1.0);
    }

    #[test]
    fn ascension_score_shards_term_scales_with_log10() {
        let a = earned_array(&[ASCENSION_SCORE_SHARDS_INDEX]);
        // ascendShards = 1e1000 → log10 ≈ 1000 → 1000/1e5 = 0.01 → factor 1.01.
        let mid = Decimal::from_mantissa_exponent(1.0, 1000.0);
        assert!((ascension_score(&a, mid) - 1.01).abs() < 1e-6);
        // ascendShards = 1e100000 → log10 ≈ 1e5 → min(1e5/1e5, 1) = 1 → factor 2.0.
        let huge = Decimal::from_mantissa_exponent(1.0, 100_000.0);
        assert!((ascension_score(&a, huge) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn obtainium_bonus_tracks_achievement_468() {
        // Unearned → product identity, regardless of reincarnation count.
        assert_eq!(obtainium_bonus(&earned_array(&[]), 1e9), 1.0);
        // Earned at reincarnationCount = 0 → 1 + 0.02·max(1, 1 + (-inf)) = 1.02.
        assert!((obtainium_bonus(&earned_array(&[468]), 0.0) - 1.02).abs() < 1e-12);
        // Earned at reincarnationCount = 1000 → 1 + 0.02·max(1, 1 + 3) = 1.08.
        assert!((obtainium_bonus(&earned_array(&[468]), 1_000.0) - 1.08).abs() < 1e-12);
    }

    #[test]
    fn ant_speed_is_product_of_its_five_achievements() {
        // Nothing earned → product identity.
        assert_eq!(ant_speed(&earned_array(&[]), Decimal::zero(), 0.0), 1.0);
        // #169 at crumbs = 0 → log10(10) = 1 (neutral).
        assert_eq!(ant_speed(&earned_array(&[169]), Decimal::zero(), 0.0), 1.0);
        // #169 at crumbs = 990 → log10(1000) = 3.
        assert!(
            (ant_speed(&earned_array(&[169]), Decimal::from_finite(990.0), 0.0) - 3.0).abs() < 1e-9
        );
        // Flat terms #171/#172/#173 → 1.2 · 1.25 · 1.4 = 2.1.
        assert!(
            (ant_speed(&earned_array(&[171, 172, 173]), Decimal::zero(), 0.0) - 2.1).abs() < 1e-12
        );
        // #174 at immortalELO = 1000 → 1 + 1 = 2.
        assert!((ant_speed(&earned_array(&[174]), Decimal::zero(), 1_000.0) - 2.0).abs() < 1e-12);
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
