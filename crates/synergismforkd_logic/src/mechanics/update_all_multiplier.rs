//! Per-tick multiplier-state aggregator.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/updateAllMultiplier.ts`.
//! Computes the full multiplier stack for the current tick. The UI
//! tier pre-evaluates every rune / hepteract / cube-blessing /
//! achievement-reward / ant-effect input and passes plain values
//! in.

use crate::mechanics::challenges::{calc_ecc, ChallengeType};
use synergismforkd_bignum::Decimal;

/// Inputs to [`update_all_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct UpdateAllMultiplierInput {
    // ─── Direct player state ──────────────────────────────────────────
    /// `player.upgrades[7]` — when `> 0`, adds `min(4, 1 +
    /// log10(fifthOwnedCoin + 1))`.
    pub upgrade_7: f64,
    /// `player.upgrades[9]` — when `> 0`, adds `floor(acceleratorBought / 10)`.
    pub upgrade_9: f64,
    /// `player.upgrades[21..25]` — each contributes `+1` when `> 0`
    /// and feeds the `1.01^count` factor.
    pub upgrade_21: f64,
    /// See `upgrade_21`.
    pub upgrade_22: f64,
    /// See `upgrade_21`.
    pub upgrade_23: f64,
    /// See `upgrade_21`.
    pub upgrade_24: f64,
    /// See `upgrade_21`.
    pub upgrade_25: f64,
    /// `player.upgrades[33]` — when `> 0`, adds `totalAcceleratorBoost`.
    pub upgrade_33: f64,
    /// `player.upgrades[34]` — `+0.03` per level multiplicatively.
    pub upgrade_34: f64,
    /// `player.upgrades[35]` — `+0.02` per level multiplicatively.
    pub upgrade_35: f64,
    /// `player.upgrades[49]` — when `> 0`, adds `min(50,
    /// log1e10(transcendPoints + 1))`.
    pub upgrade_49: f64,
    /// `player.upgrades[50]` — when `> 0.5` inside a t/r challenge,
    /// multiplies by `1.25`.
    pub upgrade_50: f64,
    /// `player.upgrades[68]` — when `> 0`, adds `min(2500,
    /// floor(log10(taxdivisor) / 1000))`.
    pub upgrade_68: f64,
    /// `player.acceleratorBought`.
    pub accelerator_bought: f64,
    /// `player.multiplierBought` — added directly to `freeMultiplier`
    /// for `totalMultiplier`.
    pub multiplier_bought: f64,
    /// `player.fifthOwnedCoin`.
    pub fifth_owned_coin: f64,
    /// `player.challengecompletions[1]`.
    pub c1_completions: f64,
    /// `player.challengecompletions[7]`.
    pub c7_completions: f64,
    /// `player.challengecompletions[14]`.
    pub c14_completions: f64,
    /// `player.transcendPoints`.
    pub transcend_points: Decimal,
    /// `player.transcendShards`.
    pub transcend_shards: Decimal,
    /// `player.researches[2]`.
    pub research_2: f64,
    /// `player.researches[11]`.
    pub research_11: f64,
    /// `player.researches[12]`.
    pub research_12: f64,
    /// `player.researches[13]`.
    pub research_13: f64,
    /// `player.researches[14]`.
    pub research_14: f64,
    /// `player.researches[15]`.
    pub research_15: f64,
    /// `player.researches[33]`.
    pub research_33: f64,
    /// `player.researches[34]`.
    pub research_34: f64,
    /// `player.researches[35]`.
    pub research_35: f64,
    /// `player.researches[87]`.
    pub research_87: f64,
    /// `player.researches[89]`.
    pub research_89: f64,
    /// `player.researches[94]` — multiplies the `20 ×
    /// floor(sumOfRuneLevels / 8)` additive.
    pub research_94: f64,
    /// `player.researches[128]`.
    pub research_128: f64,
    /// `player.researches[143]`.
    pub research_143: f64,
    /// `player.researches[158]`.
    pub research_158: f64,
    /// `player.researches[173]`.
    pub research_173: f64,
    /// `player.researches[188]`.
    pub research_188: f64,
    /// `player.researches[200]`.
    pub research_200: f64,
    /// `player.cubeUpgrades[50]`.
    pub cube_upgrade_50: f64,
    /// `player.platonicUpgrades[6]`.
    pub platonic_upgrade_6: f64,
    /// `player.currentChallenge.transcension`.
    pub transcension_challenge: u32,
    /// `player.currentChallenge.reincarnation`.
    pub reincarnation_challenge: u32,
    /// `player.corruptions.used.viscosity`.
    pub viscosity_corruption_level: u32,
    // ─── Pre-evaluated effects ────────────────────────────────────────
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
    // ─── G inputs ─────────────────────────────────────────────────────
    /// `G.totalAcceleratorBoost`.
    pub total_accelerator_boost: f64,
    /// `G.taxdivisor`.
    pub taxdivisor: Decimal,
    /// `G.viscosityPower[player.corruptions.used.viscosity]`.
    pub viscosity_power: f64,
    /// `G.challenge15Rewards.multiplier.value`.
    pub challenge_15_reward_multiplier: f64,
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
pub fn update_all_multiplier(input: &UpdateAllMultiplierInput) -> UpdateAllMultiplierResult {
    let mut a = 0.0_f64;

    if input.upgrade_7 > 0.0 {
        a += 4.0_f64.min(1.0 + (input.fifth_owned_coin + 1.0).log10().floor());
    }
    if input.upgrade_9 > 0.0 {
        a += (input.accelerator_bought / 10.0).floor();
    }
    if input.upgrade_21 > 0.0 {
        a += 1.0;
    }
    if input.upgrade_22 > 0.0 {
        a += 1.0;
    }
    if input.upgrade_23 > 0.0 {
        a += 1.0;
    }
    if input.upgrade_24 > 0.0 {
        a += 1.0;
    }
    if input.upgrade_25 > 0.0 {
        a += 1.0;
    }
    if input.upgrade_33 > 0.0 {
        a += input.total_accelerator_boost;
    }
    if input.upgrade_49 > 0.0 {
        let log_val = (input.transcend_points + Decimal::one()).log(Decimal::from_finite(1e10));
        a += 50.0_f64.min(log_val.to_number().floor());
    }
    if input.upgrade_68 > 0.0 {
        a += 2500.0_f64.min(input.taxdivisor.log10().to_number().floor() / 1_000.0);
    }
    if input.c1_completions > 0.0 {
        a += 1.0;
    }

    a += input.multipliers_achievement;
    a += 20.0 * input.research_94 * (input.sum_of_rune_levels / 8.0).floor();

    let free_upgrade_multiplier = 1e100_f64.min(a);

    let ecc14a = calc_ecc(ChallengeType::Ascension, input.c14_completions);
    let ecc1tr = calc_ecc(ChallengeType::Transcend, input.c1_completions);
    let ecc7r = calc_ecc(ChallengeType::Reincarnation, input.c7_completions);

    a *= 1.01_f64.powf(
        input.upgrade_21
            + input.upgrade_22
            + input.upgrade_23
            + input.upgrade_24
            + input.upgrade_25,
    );
    a *= 1.0 + 0.03 * input.upgrade_34 + 0.02 * input.upgrade_35;
    a *= 1.0 + (1.0 / 5.0) * input.research_2 * (1.0 + (1.0 / 2.0) * ecc14a);
    a *= 1.0
        + (1.0 / 20.0) * input.research_11
        + (1.0 / 25.0) * input.research_12
        + (1.0 / 40.0) * input.research_13
        + (3.0 / 200.0) * input.research_14
        + (1.0 / 200.0) * input.research_15;
    a *= input.multiplicative_multipliers_rune;
    a *= 1.0 + (1.0 / 20.0) * input.research_87;
    a *= 1.0 + (1.0 / 100.0) * input.research_128;
    a *= 1.0 + (0.8 / 100.0) * input.research_143;
    a *= 1.0 + (0.6 / 100.0) * input.research_158;
    a *= 1.0 + (0.4 / 100.0) * input.research_173;
    a *= 1.0 + (0.2 / 100.0) * input.research_188;
    a *= 1.0 + (0.01 / 100.0) * input.research_200;
    a *= 1.0 + (0.01 / 100.0) * input.cube_upgrade_50;
    a *= input.ant_multiplier_mult;
    a *= input.multiplier_cube_blessing;

    if (input.transcension_challenge != 0 || input.reincarnation_challenge != 0)
        && input.upgrade_50 > 0.5
    {
        a *= 1.25;
    }
    a = a.powf(1.0_f64.min((1.0 + input.platonic_upgrade_6 / 30.0) * input.viscosity_power));
    a += input.hepteract_multiplier;
    a *= input.challenge_15_reward_multiplier;
    a *= input.hepteract_multiplier_mult;
    a = 1e100_f64.min(a).floor();

    if input.viscosity_corruption_level >= 15 {
        a = a.powf(0.2);
    }
    if input.viscosity_corruption_level >= 16 {
        a = 1.0;
    }

    let free_multiplier = a;
    let total_multiplier = free_multiplier + input.multiplier_bought;
    let challenge_one_log = 3.0_f64;

    let mut b = 0.0_f64;
    b += (input.transcend_shards + Decimal::one())
        .log(Decimal::from_finite(3.0))
        .to_number();
    b += input.multiplier_boosts_rune;
    b += 2.0 * ecc1tr;
    b *= 1.0 + (11.0 * input.research_33) / 100.0;
    b *= 1.0 + (11.0 * input.research_34) / 100.0;
    b *= 1.0 + (11.0 * input.research_35) / 100.0;
    b *= 1.0 + input.research_89 / 5.0;
    b *= input.multiplier_boosts_rune_blessing;

    let total_multiplier_boost = b.floor().powf(1.0 + ecc7r * 0.04);

    let c7 = if input.c7_completions > 0.5 {
        1.25
    } else {
        1.0
    };

    let mut multiplier_power = 2.0 + 0.02 * total_multiplier_boost * c7;

    if input.reincarnation_challenge != 7 && input.reincarnation_challenge != 10 {
        if input.transcension_challenge == 1 {
            multiplier_power = 1.0;
        }
        if input.transcension_challenge == 2 {
            multiplier_power = 1.25 + 0.001_2 * b * c7;
        }
    }
    multiplier_power = 1e300_f64.min(multiplier_power);

    if input.reincarnation_challenge == 7 {
        multiplier_power = 1.0;
    }
    if input.reincarnation_challenge == 10 {
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

    fn zero_input() -> UpdateAllMultiplierInput {
        UpdateAllMultiplierInput {
            upgrade_7: 0.0,
            upgrade_9: 0.0,
            upgrade_21: 0.0,
            upgrade_22: 0.0,
            upgrade_23: 0.0,
            upgrade_24: 0.0,
            upgrade_25: 0.0,
            upgrade_33: 0.0,
            upgrade_34: 0.0,
            upgrade_35: 0.0,
            upgrade_49: 0.0,
            upgrade_50: 0.0,
            upgrade_68: 0.0,
            accelerator_bought: 0.0,
            multiplier_bought: 0.0,
            fifth_owned_coin: 0.0,
            c1_completions: 0.0,
            c7_completions: 0.0,
            c14_completions: 0.0,
            transcend_points: Decimal::zero(),
            transcend_shards: Decimal::zero(),
            research_2: 0.0,
            research_11: 0.0,
            research_12: 0.0,
            research_13: 0.0,
            research_14: 0.0,
            research_15: 0.0,
            research_33: 0.0,
            research_34: 0.0,
            research_35: 0.0,
            research_87: 0.0,
            research_89: 0.0,
            research_94: 0.0,
            research_128: 0.0,
            research_143: 0.0,
            research_158: 0.0,
            research_173: 0.0,
            research_188: 0.0,
            research_200: 0.0,
            cube_upgrade_50: 0.0,
            platonic_upgrade_6: 0.0,
            transcension_challenge: 0,
            reincarnation_challenge: 0,
            viscosity_corruption_level: 0,
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

    #[test]
    fn zero_inputs_produce_baseline() {
        let r = update_all_multiplier(&zero_input());
        // a starts 0; multiplicative mults are 1 → a = 0 → free_multiplier = 0
        assert_eq!(r.free_multiplier, 0.0);
        assert_eq!(r.total_multiplier, 0.0);
        // 2 + 0.02 * 0 * 1 = 2 → multiplier_power = 2
        assert_eq!(r.multiplier_power, 2.0);
        assert_eq!(r.challenge_one_log, 3.0);
    }

    #[test]
    fn viscosity_16_zeros_free_multiplier_to_one() {
        let mut inp = zero_input();
        inp.viscosity_corruption_level = 16;
        inp.upgrade_21 = 1.0;
        let r = update_all_multiplier(&inp);
        assert_eq!(r.free_multiplier, 1.0);
    }

    #[test]
    fn reincarnation_chal_7_forces_multiplier_power_to_1() {
        let mut inp = zero_input();
        inp.reincarnation_challenge = 7;
        let r = update_all_multiplier(&inp);
        assert_eq!(r.multiplier_power, 1.0);
    }

    #[test]
    fn upgrade_21_through_25_compound_multiplicatively() {
        let mut inp = zero_input();
        inp.upgrade_21 = 1.0;
        inp.upgrade_22 = 1.0;
        let r = update_all_multiplier(&inp);
        // a = 2; pow(1.01, 2) ≈ 1.0201; floor → 2
        assert_eq!(r.free_multiplier, 2.0);
    }
}
