//! Per-tick accelerator-state aggregator.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/updateAllTick.ts`.
//! Computes the full accelerator stack for the current tick.
//!
//! Takes a `&GameState` (for direct `player.*` reads) plus an
//! [`UpdateAllTickPre`] bundle (for cross-mechanic pre-computed values
//! that aren't in any slice) and `total_multiplier` (the output of
//! `update_all_multiplier` earlier this tick — used only in the
//! `transcensionChallenge == 1` branch of `accelerator_effect`).

use crate::mechanics::challenges::{calc_ecc, ChallengeType};
use crate::state::{GameState, VISCOSITY_INDEX};
use synergismforkd_bignum::Decimal;

/// Cross-mechanic pre-computed values the aggregator needs that don't
/// live in any state slice.
#[derive(Debug, Clone, Copy)]
pub struct UpdateAllTickPre {
    /// `+getAchievementReward('accelerators')`.
    pub accelerators_achievement: f64,
    /// `+getAchievementReward('acceleratorPower')`.
    pub accelerator_power_achievement: f64,
    /// `getRuneEffects('speed', 'multiplicativeAccelerators')`.
    pub multiplicative_accelerators_rune: f64,
    /// `getRuneEffects('speed', 'acceleratorPower')`.
    pub accelerator_power_rune: f64,
    /// `calculateAcceleratorCubeBlessing()`.
    pub accelerator_cube_blessing: f64,
    /// `getHepteractEffects('accelerator').accelerators`.
    pub hepteract_accelerators: f64,
    /// `getHepteractEffects('accelerator').acceleratorMultiplier`.
    pub hepteract_accelerator_mult: f64,
    /// `G.totalAcceleratorBoost`.
    pub total_accelerator_boost: f64,
    /// `G.acceleratorMultiplier`.
    pub accelerator_multiplier: f64,
    /// `G.viscosityPower[player.corruptions.used.viscosity]`.
    pub viscosity_power: f64,
    /// `G.challenge15Rewards.accelerator.value`.
    pub challenge_15_reward_accelerator: f64,
}

impl Default for UpdateAllTickPre {
    /// Identity values — multiplicative effects collapse to `1`, additive
    /// effects to `0`.
    fn default() -> Self {
        Self {
            accelerators_achievement: 0.0,
            accelerator_power_achievement: 0.0,
            multiplicative_accelerators_rune: 1.0,
            accelerator_power_rune: 0.0,
            accelerator_cube_blessing: 0.0,
            hepteract_accelerators: 0.0,
            hepteract_accelerator_mult: 1.0,
            total_accelerator_boost: 0.0,
            accelerator_multiplier: 1.0,
            viscosity_power: 1.0,
            challenge_15_reward_accelerator: 1.0,
        }
    }
}

/// Result of [`update_all_tick`].
#[derive(Debug, Clone, Copy)]
pub struct UpdateAllTickResult {
    /// `G.totalAccelerator`.
    pub total_accelerator: f64,
    /// `G.costDivisor` — always `1`.
    pub cost_divisor: f64,
    /// `G.freeUpgradeAccelerator`.
    pub free_upgrade_accelerator: f64,
    /// `G.freeAccelerator`.
    pub free_accelerator: f64,
    /// `G.tuSevenMulti`.
    pub tu_seven_multi: f64,
    /// `G.acceleratorPower`.
    pub accelerator_power: f64,
    /// `G.acceleratorEffect`.
    pub accelerator_effect: Decimal,
    /// `G.acceleratorEffectDisplay`.
    pub accelerator_effect_display: Decimal,
    /// `G.generatorPower`.
    pub generator_power: Decimal,
}

/// Per-tick accelerator-state aggregator. `total_multiplier` is the
/// post-`update_all_multiplier` value, used only in the
/// `transcensionChallenge == 1` branch of `acceleratorEffect`.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn update_all_tick(
    state: &GameState,
    pre: &UpdateAllTickPre,
    total_multiplier: f64,
) -> UpdateAllTickResult {
    let upgrade = |i: usize| f64::from(state.upgrades.upgrades[i]);
    let research = |i: usize| state.researches.researches[i];
    let c_completions = |i: usize| state.challenges.challenge_completions[i];

    let accelerator_bought = state.accelerator.accelerator_bought;
    let multiplier_bought = state.multiplier.multiplier_bought;
    let prestige_points = state.upgrades.prestige_points;
    let transcend_shards = state.reset_counters.transcend_shards;
    let transcension_challenge = state.challenges.current_transcension_challenge;
    let reincarnation_challenge = state.challenges.current_reincarnation_challenge;
    let platonic_upgrade_6 = state.cube_upgrade_levels.platonic_upgrades[6];
    let prestige_unlocked = state.reset_counters.prestige_unlocked;
    let viscosity_corruption_level = state.corruptions.used.levels[VISCOSITY_INDEX];

    let mut a = 0.0_f64;

    let total_accelerator_init = accelerator_bought;
    let cost_divisor = 1.0_f64;

    if upgrade(8) != 0.0 {
        a += (multiplier_bought / 7.0).floor();
    }
    if upgrade(21) != 0.0 {
        a += 5.0;
    }
    if upgrade(22) != 0.0 {
        a += 4.0;
    }
    if upgrade(23) != 0.0 {
        a += 3.0;
    }
    if upgrade(24) != 0.0 {
        a += 2.0;
    }
    if upgrade(25) != 0.0 {
        a += 1.0;
    }
    if upgrade(32) != 0.0 {
        let log_val = (prestige_points + Decimal::one())
            .log(Decimal::from_finite(1e25))
            .to_number()
            .floor();
        a += 500.0_f64.min(log_val);
    }
    if upgrade(45) != 0.0 {
        let log_val = (transcend_shards + Decimal::one())
            .log10()
            .to_number()
            .floor();
        a += 2500.0_f64.min(log_val);
    }
    a += pre.accelerators_achievement;

    let ecc2tr = calc_ecc(ChallengeType::Transcend, c_completions(2));
    let ecc7r = calc_ecc(ChallengeType::Reincarnation, c_completions(7));

    a += 5.0 * ecc2tr;
    let free_upgrade_accelerator = a;

    a += pre.total_accelerator_boost
        * (5.0
            + 2.0 * research(18)
            + 2.0 * research(19)
            + 3.0 * research(20)
            + pre.accelerator_cube_blessing);

    if prestige_unlocked {
        a *= pre.multiplicative_accelerators_rune;
    }

    a *= pre.accelerator_multiplier;
    a = a.powf(1.0_f64.min((1.0 + platonic_upgrade_6 / 30.0) * pre.viscosity_power));
    a += pre.hepteract_accelerators;
    a *= pre.challenge_15_reward_accelerator;
    a *= pre.hepteract_accelerator_mult;
    a = 1e100_f64.min(a).floor();

    if viscosity_corruption_level >= 15 {
        a = a.powf(0.2);
    }
    if viscosity_corruption_level >= 16 {
        a = 1.0;
    }

    let free_accelerator = a;
    let total_accelerator = total_accelerator_init + free_accelerator;

    let tu_seven_multi = if upgrade(46) > 0.5 { 1.05 } else { 1.0 };

    let mut accelerator_power = (1.1
        + pre.accelerator_power_rune
        + 1.0 / 400.0 * ecc2tr
        + pre.accelerator_power_achievement
        + tu_seven_multi * (pre.total_accelerator_boost / 100.0) * (1.0 + ecc2tr / 20.0))
        .powf(1.0 + 0.04 * ecc7r);

    if reincarnation_challenge != 7 && reincarnation_challenge != 10 {
        if transcension_challenge == 1 {
            accelerator_power *= 25.0 / (50.0 + c_completions(1));
            accelerator_power += 0.55;
            accelerator_power = 1.0_f64.max(accelerator_power);
        }
        if transcension_challenge == 2 {
            accelerator_power = 1.0;
        }
        if transcension_challenge == 3 {
            accelerator_power = 1.0 + accelerator_power / 2.0;
        }
    }
    accelerator_power = 1e300_f64.min(accelerator_power);
    if reincarnation_challenge == 7 {
        accelerator_power = 1.0;
    }
    if reincarnation_challenge == 10 {
        accelerator_power = 1.0;
    }

    let mut accelerator_effect = if transcension_challenge != 1 {
        Decimal::from_finite(accelerator_power).pow(Decimal::from_finite(total_accelerator))
    } else {
        Decimal::from_finite(accelerator_power)
            .pow(Decimal::from_finite(total_accelerator + total_multiplier))
    };
    let accelerator_effect_display = Decimal::from_finite(accelerator_power * 100.0 - 100.0);
    if reincarnation_challenge == 10 {
        accelerator_effect = Decimal::one();
    }

    let mut generator_power = Decimal::one();
    if upgrade(11) > 0.5 && reincarnation_challenge != 7 {
        generator_power = Decimal::from_finite(1.02).pow(Decimal::from_finite(total_accelerator));
    }

    UpdateAllTickResult {
        total_accelerator,
        cost_divisor,
        free_upgrade_accelerator,
        free_accelerator,
        tu_seven_multi,
        accelerator_power,
        accelerator_effect,
        accelerator_effect_display,
        generator_power,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_baseline() {
        let r = update_all_tick(&GameState::default(), &UpdateAllTickPre::default(), 0.0);
        // a = 0 → free_accelerator = 0.
        assert_eq!(r.free_accelerator, 0.0);
        assert_eq!(r.total_accelerator, 0.0);
        assert_eq!(r.tu_seven_multi, 1.0);
        // accelerator_power = (1.1 + 0 + …).powf(1) = 1.1.
        assert!((r.accelerator_power - 1.1).abs() < 1e-12);
    }

    #[test]
    fn upgrade_21_adds_5_to_a() {
        let mut state = GameState::default();
        state.upgrades.upgrades[21] = 1;
        let r = update_all_tick(&state, &UpdateAllTickPre::default(), 0.0);
        // a=5, then a^min(1, 1*1)=5; floor → 5.
        assert_eq!(r.free_accelerator, 5.0);
    }

    #[test]
    fn viscosity_16_zeros_to_1() {
        let mut state = GameState::default();
        state.corruptions.used.levels[VISCOSITY_INDEX] = 16;
        state.upgrades.upgrades[21] = 1;
        let r = update_all_tick(&state, &UpdateAllTickPre::default(), 0.0);
        assert_eq!(r.free_accelerator, 1.0);
    }

    #[test]
    fn reincarnation_chal_10_zeros_effect() {
        let mut state = GameState::default();
        state.challenges.current_reincarnation_challenge = 10;
        let r = update_all_tick(&state, &UpdateAllTickPre::default(), 0.0);
        assert_eq!(r.accelerator_effect.to_number(), 1.0);
        assert_eq!(r.accelerator_power, 1.0);
    }

    #[test]
    fn upgrade_46_sets_tu_seven_multi() {
        let mut state = GameState::default();
        state.upgrades.upgrades[46] = 1;
        let r = update_all_tick(&state, &UpdateAllTickPre::default(), 0.0);
        assert_eq!(r.tu_seven_multi, 1.05);
    }

    #[test]
    fn generator_power_active_with_upgrade_11_outside_chal_7() {
        let mut state = GameState::default();
        state.upgrades.upgrades[11] = 1;
        state.accelerator.accelerator_bought = 100.0;
        let r = update_all_tick(&state, &UpdateAllTickPre::default(), 0.0);
        // 1.02^100 ≈ 7.24.
        let expected = 1.02_f64.powi(100);
        assert!((r.generator_power.to_number() - expected).abs() < 1e-6);
    }
}
