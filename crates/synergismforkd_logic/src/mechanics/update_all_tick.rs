//! Per-tick accelerator-state aggregator.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/updateAllTick.ts`.
//! Computes the full accelerator stack for the current tick. The UI
//! tier pre-evaluates all effect inputs.

use crate::mechanics::challenges::{calc_ecc, ChallengeType};
use synergismforkd_bignum::Decimal;

/// Inputs to [`update_all_tick`].
#[derive(Debug, Clone, Copy)]
pub struct UpdateAllTickInput {
    /// `player.acceleratorBought`.
    pub accelerator_bought: f64,
    /// `player.multiplierBought`.
    pub multiplier_bought: f64,
    /// `player.upgrades[8]`.
    pub upgrade_8: f64,
    /// `player.upgrades[11]`.
    pub upgrade_11: f64,
    /// `player.upgrades[21]`.
    pub upgrade_21: f64,
    /// `player.upgrades[22]`.
    pub upgrade_22: f64,
    /// `player.upgrades[23]`.
    pub upgrade_23: f64,
    /// `player.upgrades[24]`.
    pub upgrade_24: f64,
    /// `player.upgrades[25]`.
    pub upgrade_25: f64,
    /// `player.upgrades[32]`.
    pub upgrade_32: f64,
    /// `player.upgrades[45]`.
    pub upgrade_45: f64,
    /// `player.upgrades[46]`.
    pub upgrade_46: f64,
    /// `player.prestigePoints`.
    pub prestige_points: Decimal,
    /// `player.transcendShards`.
    pub transcend_shards: Decimal,
    /// `player.challengecompletions[1]`.
    pub c1_completions: f64,
    /// `player.challengecompletions[2]`.
    pub c2_completions: f64,
    /// `player.challengecompletions[7]`.
    pub c7_completions: f64,
    /// `player.currentChallenge.transcension`.
    pub transcension_challenge: u32,
    /// `player.currentChallenge.reincarnation`.
    pub reincarnation_challenge: u32,
    /// `player.researches[18]`.
    pub research_18: f64,
    /// `player.researches[19]`.
    pub research_19: f64,
    /// `player.researches[20]`.
    pub research_20: f64,
    /// `player.platonicUpgrades[6]`.
    pub platonic_upgrade_6: f64,
    /// `player.unlocks.prestige`.
    pub prestige_unlocked: bool,
    /// `player.corruptions.used.viscosity`.
    pub viscosity_corruption_level: u32,
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
pub fn update_all_tick(input: &UpdateAllTickInput, total_multiplier: f64) -> UpdateAllTickResult {
    let mut a = 0.0_f64;

    let total_accelerator_init = input.accelerator_bought;
    let cost_divisor = 1.0_f64;

    if input.upgrade_8 != 0.0 {
        a += (input.multiplier_bought / 7.0).floor();
    }
    if input.upgrade_21 != 0.0 {
        a += 5.0;
    }
    if input.upgrade_22 != 0.0 {
        a += 4.0;
    }
    if input.upgrade_23 != 0.0 {
        a += 3.0;
    }
    if input.upgrade_24 != 0.0 {
        a += 2.0;
    }
    if input.upgrade_25 != 0.0 {
        a += 1.0;
    }
    if input.upgrade_32 != 0.0 {
        let log_val = (input.prestige_points + Decimal::one())
            .log(Decimal::from_finite(1e25))
            .to_number()
            .floor();
        a += 500.0_f64.min(log_val);
    }
    if input.upgrade_45 != 0.0 {
        let log_val = (input.transcend_shards + Decimal::one())
            .log10()
            .to_number()
            .floor();
        a += 2500.0_f64.min(log_val);
    }
    a += input.accelerators_achievement;

    let ecc2tr = calc_ecc(ChallengeType::Transcend, input.c2_completions);
    let ecc7r = calc_ecc(ChallengeType::Reincarnation, input.c7_completions);

    a += 5.0 * ecc2tr;
    let free_upgrade_accelerator = a;

    a += input.total_accelerator_boost
        * (5.0
            + 2.0 * input.research_18
            + 2.0 * input.research_19
            + 3.0 * input.research_20
            + input.accelerator_cube_blessing);

    if input.prestige_unlocked {
        a *= input.multiplicative_accelerators_rune;
    }

    a *= input.accelerator_multiplier;
    a = a.powf(1.0_f64.min((1.0 + input.platonic_upgrade_6 / 30.0) * input.viscosity_power));
    a += input.hepteract_accelerators;
    a *= input.challenge_15_reward_accelerator;
    a *= input.hepteract_accelerator_mult;
    a = 1e100_f64.min(a).floor();

    if input.viscosity_corruption_level >= 15 {
        a = a.powf(0.2);
    }
    if input.viscosity_corruption_level >= 16 {
        a = 1.0;
    }

    let free_accelerator = a;
    let total_accelerator = total_accelerator_init + free_accelerator;

    let tu_seven_multi = if input.upgrade_46 > 0.5 { 1.05 } else { 1.0 };

    let mut accelerator_power = (1.1
        + input.accelerator_power_rune
        + 1.0 / 400.0 * ecc2tr
        + input.accelerator_power_achievement
        + tu_seven_multi * (input.total_accelerator_boost / 100.0) * (1.0 + ecc2tr / 20.0))
        .powf(1.0 + 0.04 * ecc7r);

    if input.reincarnation_challenge != 7 && input.reincarnation_challenge != 10 {
        if input.transcension_challenge == 1 {
            accelerator_power *= 25.0 / (50.0 + input.c1_completions);
            accelerator_power += 0.55;
            accelerator_power = 1.0_f64.max(accelerator_power);
        }
        if input.transcension_challenge == 2 {
            accelerator_power = 1.0;
        }
        if input.transcension_challenge == 3 {
            accelerator_power = 1.0 + accelerator_power / 2.0;
        }
    }
    accelerator_power = 1e300_f64.min(accelerator_power);
    if input.reincarnation_challenge == 7 {
        accelerator_power = 1.0;
    }
    if input.reincarnation_challenge == 10 {
        accelerator_power = 1.0;
    }

    let mut accelerator_effect = if input.transcension_challenge != 1 {
        Decimal::from_finite(accelerator_power).pow(Decimal::from_finite(total_accelerator))
    } else {
        Decimal::from_finite(accelerator_power)
            .pow(Decimal::from_finite(total_accelerator + total_multiplier))
    };
    let accelerator_effect_display = Decimal::from_finite(accelerator_power * 100.0 - 100.0);
    if input.reincarnation_challenge == 10 {
        accelerator_effect = Decimal::one();
    }

    let mut generator_power = Decimal::one();
    if input.upgrade_11 > 0.5 && input.reincarnation_challenge != 7 {
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

    fn zero_input() -> UpdateAllTickInput {
        UpdateAllTickInput {
            accelerator_bought: 0.0,
            multiplier_bought: 0.0,
            upgrade_8: 0.0,
            upgrade_11: 0.0,
            upgrade_21: 0.0,
            upgrade_22: 0.0,
            upgrade_23: 0.0,
            upgrade_24: 0.0,
            upgrade_25: 0.0,
            upgrade_32: 0.0,
            upgrade_45: 0.0,
            upgrade_46: 0.0,
            prestige_points: Decimal::zero(),
            transcend_shards: Decimal::zero(),
            c1_completions: 0.0,
            c2_completions: 0.0,
            c7_completions: 0.0,
            transcension_challenge: 0,
            reincarnation_challenge: 0,
            research_18: 0.0,
            research_19: 0.0,
            research_20: 0.0,
            platonic_upgrade_6: 0.0,
            prestige_unlocked: false,
            viscosity_corruption_level: 0,
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

    #[test]
    fn zero_inputs_baseline() {
        let r = update_all_tick(&zero_input(), 0.0);
        // a = 0 → free_accelerator = 0
        assert_eq!(r.free_accelerator, 0.0);
        assert_eq!(r.total_accelerator, 0.0);
        assert_eq!(r.tu_seven_multi, 1.0);
        // accelerator_power = (1.1 + 0 + ...).powf(1) = 1.1
        assert!((r.accelerator_power - 1.1).abs() < 1e-12);
    }

    #[test]
    fn upgrade_21_adds_5_to_a() {
        let mut inp = zero_input();
        inp.upgrade_21 = 1.0;
        let r = update_all_tick(&inp, 0.0);
        // a=5, then a^min(1, 1*1)=5; floor → 5
        assert_eq!(r.free_accelerator, 5.0);
    }

    #[test]
    fn viscosity_16_zeros_to_1() {
        let mut inp = zero_input();
        inp.viscosity_corruption_level = 16;
        inp.upgrade_21 = 1.0;
        let r = update_all_tick(&inp, 0.0);
        assert_eq!(r.free_accelerator, 1.0);
    }

    #[test]
    fn reincarnation_chal_10_zeros_effect() {
        let mut inp = zero_input();
        inp.reincarnation_challenge = 10;
        let r = update_all_tick(&inp, 0.0);
        assert_eq!(r.accelerator_effect.to_number(), 1.0);
        assert_eq!(r.accelerator_power, 1.0);
    }

    #[test]
    fn upgrade_46_sets_tu_seven_multi() {
        let mut inp = zero_input();
        inp.upgrade_46 = 1.0;
        let r = update_all_tick(&inp, 0.0);
        assert_eq!(r.tu_seven_multi, 1.05);
    }

    #[test]
    fn generator_power_active_with_upgrade_11_outside_chal_7() {
        let mut inp = zero_input();
        inp.upgrade_11 = 1.0;
        inp.accelerator_bought = 100.0;
        let r = update_all_tick(&inp, 0.0);
        // 1.02^100 ≈ 7.24
        let expected = 1.02_f64.powi(100);
        assert!((r.generator_power.to_number() - expected).abs() < 1e-6);
    }
}
