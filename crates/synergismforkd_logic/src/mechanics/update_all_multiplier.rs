//! Per-tick multiplier-state aggregator.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/updateAllMultiplier.ts`.
//! Computes the full multiplier stack for the current tick.
//!
//! Takes a `&GameState` (for direct `player.*` reads) plus an
//! [`UpdateAllMultiplierPre`] bundle (for cross-mechanic pre-computed
//! values that aren't in any slice).

use crate::mechanics::challenges::{calc_ecc, ChallengeType};
use crate::state::{GameState, VISCOSITY_INDEX};
use synergismforkd_bignum::Decimal;

/// Cross-mechanic pre-computed values the aggregator needs that don't
/// live in any state slice.
#[derive(Debug, Clone, Copy)]
pub struct UpdateAllMultiplierPre {
    /// `+getAchievementReward('multipliers')`.
    pub multipliers_achievement: f64,
    /// `sumOfRuneLevels()`.
    pub sum_of_rune_levels: f64,
    /// `getRuneEffects('duplication', 'multiplicativeMultipliers')`.
    pub multiplicative_multipliers_rune: f64,
    /// `getRuneEffects('duplication', 'multiplierBoosts')`.
    pub multiplier_boosts_rune: f64,
    /// `getRuneBlessingEffect('duplication').multiplierBoosts`.
    pub multiplier_boosts_rune_blessing: f64,
    /// `getAntUpgradeEffect(AntUpgrades.Multipliers).multiplierMult`.
    pub ant_multiplier_mult: f64,
    /// `calculateMultiplierCubeBlessing()`.
    pub multiplier_cube_blessing: f64,
    /// `getHepteractEffects('multiplier').multiplier`.
    pub hepteract_multiplier: f64,
    /// `getHepteractEffects('multiplier').multiplierMultiplier`.
    pub hepteract_multiplier_mult: f64,
    /// `G.totalAcceleratorBoost`.
    pub total_accelerator_boost: f64,
    /// `G.taxdivisor`.
    pub taxdivisor: Decimal,
    /// `G.viscosityPower[player.corruptions.used.viscosity]`.
    pub viscosity_power: f64,
    /// `G.challenge15Rewards.multiplier.value`.
    pub challenge_15_reward_multiplier: f64,
}

impl Default for UpdateAllMultiplierPre {
    /// Identity values — multiplicative effects collapse to `1`, additive
    /// effects to `0`. Passing this default with a default `GameState`
    /// yields the baseline result.
    fn default() -> Self {
        Self {
            multipliers_achievement: 0.0,
            sum_of_rune_levels: 0.0,
            multiplicative_multipliers_rune: 1.0,
            multiplier_boosts_rune: 0.0,
            multiplier_boosts_rune_blessing: 1.0,
            ant_multiplier_mult: 1.0,
            multiplier_cube_blessing: 1.0,
            hepteract_multiplier: 0.0,
            hepteract_multiplier_mult: 1.0,
            total_accelerator_boost: 0.0,
            taxdivisor: Decimal::one(),
            viscosity_power: 1.0,
            challenge_15_reward_multiplier: 1.0,
        }
    }
}

/// Result of [`update_all_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct UpdateAllMultiplierResult {
    /// `G.freeUpgradeMultiplier`.
    pub free_upgrade_multiplier: f64,
    /// `G.freeMultiplier`.
    pub free_multiplier: f64,
    /// `G.totalMultiplier`.
    pub total_multiplier: f64,
    /// `G.challengeOneLog` — constant `3`.
    pub challenge_one_log: f64,
    /// `G.totalMultiplierBoost`.
    pub total_multiplier_boost: f64,
    /// `G.multiplierPower`.
    pub multiplier_power: f64,
    /// `G.multiplierEffect`.
    pub multiplier_effect: Decimal,
}

/// Per-tick multiplier-state aggregator.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn update_all_multiplier(
    state: &GameState,
    pre: &UpdateAllMultiplierPre,
) -> UpdateAllMultiplierResult {
    let upgrade = |i: usize| f64::from(state.upgrades.upgrades[i]);
    let research = |i: usize| state.researches.researches[i];
    let c_completions = |i: usize| state.challenges.challenge_completions[i];

    let accelerator_bought = state.accelerator.accelerator_bought;
    let multiplier_bought = state.multiplier.multiplier_bought;
    let fifth_owned_coin = state.coin_producers.tiers[4].owned;
    let transcend_points = state.upgrades.transcend_points;
    let transcend_shards = state.reset_counters.transcend_shards;
    let cube_upgrade_50 = state.cube_upgrade_levels.cube_upgrades[50];
    let platonic_upgrade_6 = state.cube_upgrade_levels.platonic_upgrades[6];
    let transcension_challenge = state.challenges.current_transcension_challenge;
    let reincarnation_challenge = state.challenges.current_reincarnation_challenge;
    let viscosity_corruption_level = state.corruptions.used.levels[VISCOSITY_INDEX];

    let mut a = 0.0_f64;

    if upgrade(7) > 0.0 {
        a += 4.0_f64.min(1.0 + (fifth_owned_coin + 1.0).log10().floor());
    }
    if upgrade(9) > 0.0 {
        a += (accelerator_bought / 10.0).floor();
    }
    if upgrade(21) > 0.0 {
        a += 1.0;
    }
    if upgrade(22) > 0.0 {
        a += 1.0;
    }
    if upgrade(23) > 0.0 {
        a += 1.0;
    }
    if upgrade(24) > 0.0 {
        a += 1.0;
    }
    if upgrade(25) > 0.0 {
        a += 1.0;
    }
    if upgrade(33) > 0.0 {
        a += pre.total_accelerator_boost;
    }
    if upgrade(49) > 0.0 {
        let log_val = (transcend_points + Decimal::one()).log(Decimal::from_finite(1e10));
        a += 50.0_f64.min(log_val.to_number().floor());
    }
    if upgrade(68) > 0.0 {
        a += 2500.0_f64.min(pre.taxdivisor.log10().to_number().floor() / 1_000.0);
    }
    if c_completions(1) > 0.0 {
        a += 1.0;
    }

    a += pre.multipliers_achievement;
    a += 20.0 * research(94) * (pre.sum_of_rune_levels / 8.0).floor();

    let free_upgrade_multiplier = 1e100_f64.min(a);

    let ecc14a = calc_ecc(ChallengeType::Ascension, c_completions(14));
    let ecc1tr = calc_ecc(ChallengeType::Transcend, c_completions(1));
    let ecc7r = calc_ecc(ChallengeType::Reincarnation, c_completions(7));

    a *= 1.01_f64.powf(upgrade(21) + upgrade(22) + upgrade(23) + upgrade(24) + upgrade(25));
    a *= 1.0 + 0.03 * upgrade(34) + 0.02 * upgrade(35);
    a *= 1.0 + (1.0 / 5.0) * research(2) * (1.0 + (1.0 / 2.0) * ecc14a);
    a *= 1.0
        + (1.0 / 20.0) * research(11)
        + (1.0 / 25.0) * research(12)
        + (1.0 / 40.0) * research(13)
        + (3.0 / 200.0) * research(14)
        + (1.0 / 200.0) * research(15);
    a *= pre.multiplicative_multipliers_rune;
    a *= 1.0 + (1.0 / 20.0) * research(87);
    a *= 1.0 + (1.0 / 100.0) * research(128);
    a *= 1.0 + (0.8 / 100.0) * research(143);
    a *= 1.0 + (0.6 / 100.0) * research(158);
    a *= 1.0 + (0.4 / 100.0) * research(173);
    a *= 1.0 + (0.2 / 100.0) * research(188);
    a *= 1.0 + (0.01 / 100.0) * research(200);
    a *= 1.0 + (0.01 / 100.0) * cube_upgrade_50;
    a *= pre.ant_multiplier_mult;
    a *= pre.multiplier_cube_blessing;

    if (transcension_challenge != 0 || reincarnation_challenge != 0) && upgrade(50) > 0.5 {
        a *= 1.25;
    }
    a = a.powf(1.0_f64.min((1.0 + platonic_upgrade_6 / 30.0) * pre.viscosity_power));
    a += pre.hepteract_multiplier;
    a *= pre.challenge_15_reward_multiplier;
    a *= pre.hepteract_multiplier_mult;
    a = 1e100_f64.min(a).floor();

    if viscosity_corruption_level >= 15 {
        a = a.powf(0.2);
    }
    if viscosity_corruption_level >= 16 {
        a = 1.0;
    }

    let free_multiplier = a;
    let total_multiplier = free_multiplier + multiplier_bought;
    let challenge_one_log = 3.0_f64;

    let mut b = 0.0_f64;
    b += (transcend_shards + Decimal::one())
        .log(Decimal::from_finite(3.0))
        .to_number();
    b += pre.multiplier_boosts_rune;
    b += 2.0 * ecc1tr;
    b *= 1.0 + (11.0 * research(33)) / 100.0;
    b *= 1.0 + (11.0 * research(34)) / 100.0;
    b *= 1.0 + (11.0 * research(35)) / 100.0;
    b *= 1.0 + research(89) / 5.0;
    b *= pre.multiplier_boosts_rune_blessing;

    let total_multiplier_boost = b.floor().powf(1.0 + ecc7r * 0.04);

    let c7 = if c_completions(7) > 0.5 { 1.25 } else { 1.0 };

    let mut multiplier_power = 2.0 + 0.02 * total_multiplier_boost * c7;

    if reincarnation_challenge != 7 && reincarnation_challenge != 10 {
        if transcension_challenge == 1 {
            multiplier_power = 1.0;
        }
        if transcension_challenge == 2 {
            multiplier_power = 1.25 + 0.001_2 * b * c7;
        }
    }
    multiplier_power = 1e300_f64.min(multiplier_power);

    if reincarnation_challenge == 7 {
        multiplier_power = 1.0;
    }
    if reincarnation_challenge == 10 {
        multiplier_power = 1.0;
    }

    let multiplier_effect =
        Decimal::from_finite(multiplier_power).pow(Decimal::from_finite(total_multiplier));

    UpdateAllMultiplierResult {
        free_upgrade_multiplier,
        free_multiplier,
        total_multiplier,
        challenge_one_log,
        total_multiplier_boost,
        multiplier_power,
        multiplier_effect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_produces_baseline() {
        let r = update_all_multiplier(&GameState::default(), &UpdateAllMultiplierPre::default());
        // a starts 0; multiplicative mults are 1 → a = 0 → free_multiplier = 0.
        assert_eq!(r.free_multiplier, 0.0);
        assert_eq!(r.total_multiplier, 0.0);
        // 2 + 0.02 * 0 * 1 = 2 → multiplier_power = 2.
        assert_eq!(r.multiplier_power, 2.0);
        assert_eq!(r.challenge_one_log, 3.0);
    }

    #[test]
    fn viscosity_16_zeros_free_multiplier_to_one() {
        let mut state = GameState::default();
        state.corruptions.used.levels[VISCOSITY_INDEX] = 16;
        state.upgrades.upgrades[21] = 1;
        let r = update_all_multiplier(&state, &UpdateAllMultiplierPre::default());
        assert_eq!(r.free_multiplier, 1.0);
    }

    #[test]
    fn reincarnation_chal_7_forces_multiplier_power_to_1() {
        let mut state = GameState::default();
        state.challenges.current_reincarnation_challenge = 7;
        let r = update_all_multiplier(&state, &UpdateAllMultiplierPre::default());
        assert_eq!(r.multiplier_power, 1.0);
    }

    #[test]
    fn upgrade_21_through_25_compound_multiplicatively() {
        let mut state = GameState::default();
        state.upgrades.upgrades[21] = 1;
        state.upgrades.upgrades[22] = 1;
        let r = update_all_multiplier(&state, &UpdateAllMultiplierPre::default());
        // a = 2; pow(1.01, 2) ≈ 1.0201; floor → 2.
        assert_eq!(r.free_multiplier, 2.0);
    }
}
