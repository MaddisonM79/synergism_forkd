//! Pure subroutines lifted from `packages/web_ui/src/Calculate.ts`
//! (and `getReductionValue` from `Buy.ts`).
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/calculate.ts`. Each
//! function takes its inputs as precomputed values — the surrounding
//! StatLine reductions stay in the UI tier because they aggregate
//! per-line `stat()` callbacks that still read from player/G state.

use crate::mechanics::challenges::{calc_ecc, ChallengeType};
use synergismforkd_bignum::Decimal;

// ─── Global speed multiplier ──────────────────────────────────────────────

/// Inputs to [`calculate_global_speed_mult`].
#[derive(Debug, Clone, Copy)]
pub struct GlobalSpeedMultInput {
    /// Product of the DR-enabled multiplier StatLines.
    pub normal_mult: f64,
    /// Product of the DR-ignored multiplier StatLines. Multiplied
    /// straight through.
    pub immaculate_mult: f64,
    /// Platonic upgrade 7 exponent power — `1 - platonicUpgrades[7] / 30`.
    /// Only used in the `normalMult < 1` branch.
    pub dr_power: f64,
}

/// Combines two precomputed multiplier legs with diminishing-returns
/// thresholds on the normal leg.
#[must_use]
pub fn calculate_global_speed_mult(input: &GlobalSpeedMultInput) -> f64 {
    let mut normal_mult = input.normal_mult;
    if normal_mult > 100.0 {
        normal_mult = normal_mult.powf(0.5) * 10.0;
    } else if normal_mult < 1.0 {
        normal_mult = normal_mult.powf(input.dr_power);
    }
    normal_mult * input.immaculate_mult
}

// ─── Ascension speed multiplier ───────────────────────────────────────────

/// Inputs to [`calculate_ascension_speed_mult`].
#[derive(Debug, Clone, Copy)]
pub struct AscensionSpeedMultInput {
    /// Product of the ascension-speed StatLines.
    pub base: f64,
    /// Sum of three GQ/shop upgrade contributions.
    pub exponent_spread: f64,
}

/// Applies an exponent-spread transformation: `base ^ (1 - spread)`
/// when `base < 1`, `base ^ (1 + spread)` when `base >= 1`.
#[must_use]
pub fn calculate_ascension_speed_mult(input: &AscensionSpeedMultInput) -> f64 {
    if input.base < 1.0 {
        input.base.powf(1.0 - input.exponent_spread)
    } else {
        input.base.powf(1.0 + input.exponent_spread)
    }
}

// ─── Ant speed (with ascension-challenge penalties) ───────────────────────

/// Inputs to [`calculate_actual_ant_speed_mult`].
#[derive(Debug, Clone, Copy)]
pub struct ActualAntSpeedMultInput {
    /// Product of the antSpeedStats StatLines.
    pub base: Decimal,
    /// `player.currentChallenge.ascension`. 12 → 0.75, 13 → 0.23,
    /// 14 → 0.20, 15 → 0.50, else 1.
    pub ascension_challenge: u32,
    /// `player.platonicUpgrades[10]`. When `> 0` AND
    /// `ascensionChallenge == 15`, exponent is multiplied by 1.25.
    pub platonic_upgrade_10: f64,
}

/// Raises the precomputed Decimal base by the challenge-dependent
/// exponent.
#[must_use]
pub fn calculate_actual_ant_speed_mult(input: &ActualAntSpeedMultInput) -> Decimal {
    let mut exponent = match input.ascension_challenge {
        12 => 0.75,
        13 => 0.23,
        14 => 0.20,
        15 => 0.50,
        _ => 1.0,
    };
    if input.platonic_upgrade_10 > 0.0 && input.ascension_challenge == 15 {
        exponent *= 1.25;
    }
    input.base.pow(Decimal::from_finite(exponent))
}

// ─── Reduction value (cost divisor `r`) ───────────────────────────────────

/// Inputs to [`get_reduction_value`].
#[derive(Debug, Clone, Copy)]
pub struct ReductionValueInput {
    /// `getRuneEffects('thrift', 'costDelay')`.
    pub thrift_cost_delay: f64,
    /// Sum of `player.researches[56..60]`. Divided by 200.
    pub researches_sum: f64,
    /// `player.challengecompletions[4]`. Fed through
    /// `CalcECC('transcend', cc4) / 200`.
    pub challenge_completions_4: f64,
    /// `getAntUpgradeEffect(AntUpgrades.BuildingCostScale).buildingCostScale`.
    pub ant_building_cost_scale: f64,
}

/// Cost-divisor `r` aggregator.
#[must_use]
pub fn get_reduction_value(input: &ReductionValueInput) -> f64 {
    1.0 + input.thrift_cost_delay
        + input.researches_sum / 200.0
        + calc_ecc(ChallengeType::Transcend, input.challenge_completions_4) / 200.0
        + input.ant_building_cost_scale
}

// ─── Offerings aggregator ─────────────────────────────────────────────────

/// Inputs to [`calculate_offerings`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateOfferingsInput {
    /// Sum from allBaseOfferingStats.
    pub base_offerings: f64,
    /// Product from offeringObtainiumTimeModifiers when timeMultUsed, else 1.
    pub time_multiplier: f64,
    /// Product from allOfferingStats (Decimal).
    pub offering_mult: Decimal,
    /// `player.singularityChallenges.taxmanLastStand.enabled`.
    pub taxman_last_stand_enabled: bool,
    /// `player.singularityChallenges.taxmanLastStand.completions`.
    pub taxman_last_stand_completions: f64,
    /// `player.offerings` — used by the taxman cap.
    pub current_offerings: Decimal,
}

/// Final offerings for the next reset. Applies the Exalt 8 taxman
/// cap when completions `>= 2`.
#[must_use]
pub fn calculate_offerings(input: &CalculateOfferingsInput) -> Decimal {
    let main = Decimal::from_finite(input.base_offerings)
        .max(input.offering_mult * Decimal::from_finite(input.time_multiplier));
    if input.taxman_last_stand_enabled && input.taxman_last_stand_completions >= 2.0 {
        return (input.current_offerings * Decimal::from_finite(100.0) + Decimal::one()).min(main);
    }
    main
}

// ─── Obtainium aggregator ─────────────────────────────────────────────────

/// Inputs to [`calculate_obtainium`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateObtainiumInput {
    /// Sum from allBaseObtainiumStats.
    pub base_obtainium: f64,
    /// Product from `allObtainiumIgnoreDRStats`.
    pub immaculate: f64,
    /// Corruption "illiteracy" effect — applied as exponent on
    /// baseMults.
    pub dr: f64,
    /// Product from offeringObtainiumTimeModifiers when timeMultUsed,
    /// else 1.
    pub time_multiplier: f64,
    /// Product from allObtainiumStats.
    pub base_mults: Decimal,
    /// `player.currentChallenge.ascension === 14` — short-circuits to 0.
    pub in_ascension_challenge_14: bool,
    /// `player.singularityChallenges.taxmanLastStand.enabled`.
    pub taxman_last_stand_enabled: bool,
    /// `player.singularityChallenges.taxmanLastStand.completions`.
    pub taxman_last_stand_completions: f64,
    /// `player.obtainium` — used by the taxman cap.
    pub current_obtainium: Decimal,
}

/// Final obtainium with C14 zero-out and taxman clamp.
#[must_use]
pub fn calculate_obtainium(input: &CalculateObtainiumInput) -> Decimal {
    if input.in_ascension_challenge_14 {
        return Decimal::zero();
    }
    let total = Decimal::from_finite(input.immaculate)
        * input.base_mults.pow(Decimal::from_finite(input.dr))
        * Decimal::from_finite(input.time_multiplier);
    let main = Decimal::from_finite(input.base_obtainium).max(total);
    if input.taxman_last_stand_enabled && input.taxman_last_stand_completions >= 2.0 {
        return (input.current_obtainium * Decimal::from_finite(100.0) + Decimal::one()).min(main);
    }
    main
}

// ─── Salvage ──────────────────────────────────────────────────────────────

/// Inputs to [`calculate_positive_salvage`].
#[derive(Debug, Clone, Copy)]
pub struct CalculatePositiveSalvageInput {
    /// Sum from `positiveSalvageStats`.
    pub raw_positive_salvage: f64,
    /// Product from [`calculate_positive_salvage_multiplier`].
    pub positive_salvage_multiplier: f64,
    /// `player.singularityChallenges.taxmanLastStand.enabled`.
    pub taxman_last_stand_enabled: bool,
}

/// Total positive salvage. `taxman` branch: `100 + raw × mult /
/// max(1, ln(raw))`. Otherwise `raw × mult`.
#[must_use]
pub fn calculate_positive_salvage(input: &CalculatePositiveSalvageInput) -> f64 {
    if input.taxman_last_stand_enabled {
        let base_salvage = 100.0_f64;
        return base_salvage
            + (input.raw_positive_salvage * input.positive_salvage_multiplier)
                / 1.0_f64.max(input.raw_positive_salvage.ln());
    }
    input.raw_positive_salvage * input.positive_salvage_multiplier
}

/// Inputs to [`calculate_positive_salvage_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct CalculatePositiveSalvageMultiplierInput {
    /// `posSalvagePerkSings.filter(x => x <= highestSingularityCount).length`.
    pub positive_salvage_perk_unlocked_count: f64,
    /// `getTalismanEffects('achievement').positiveSalvageMult`.
    pub talisman_achievement_positive_salvage_mult: f64,
}

/// `1 + perks/100 + talisman_mult`.
#[must_use]
pub fn calculate_positive_salvage_multiplier(
    input: &CalculatePositiveSalvageMultiplierInput,
) -> f64 {
    1.0 + input.positive_salvage_perk_unlocked_count / 100.0
        + input.talisman_achievement_positive_salvage_mult
}

/// Inputs to [`calculate_negative_salvage_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateNegativeSalvageMultiplierInput {
    /// `negSalvagePerkSings.filter(x => x <= highestSingularityCount).length`.
    pub negative_salvage_perk_unlocked_count: f64,
    /// `getTalismanEffects('achievement').negativeSalvageMult`.
    pub talisman_achievement_negative_salvage_mult: f64,
}

/// `1 - perks/100 + talisman_mult`.
#[must_use]
pub fn calculate_negative_salvage_multiplier(
    input: &CalculateNegativeSalvageMultiplierInput,
) -> f64 {
    1.0 - input.negative_salvage_perk_unlocked_count / 100.0
        + input.talisman_achievement_negative_salvage_mult
}

/// Inputs to [`calculate_negative_salvage`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateNegativeSalvageInput {
    /// Sum from `negativeSalvageStats`.
    pub raw_negative_salvage: f64,
    /// Output of [`calculate_negative_salvage_multiplier`].
    pub negative_salvage_multiplier: f64,
}

/// `raw × multiplier`.
#[must_use]
pub fn calculate_negative_salvage(input: &CalculateNegativeSalvageInput) -> f64 {
    input.raw_negative_salvage * input.negative_salvage_multiplier
}

/// Total salvage — sum of positive and negative (negative is
/// already negative-valued).
#[must_use]
pub fn calculate_total_salvage(positive_salvage: f64, negative_salvage: f64) -> f64 {
    positive_salvage + negative_salvage
}

/// `10^(salvage / 30)`. Each point of total salvage shifts the
/// rune-EXP multiplier by ~7.6%.
#[must_use]
pub fn calculate_salvage_rune_exp_multiplier(salvage: f64) -> Decimal {
    Decimal::from_finite(10.0).pow(Decimal::from_finite(salvage / 30.0))
}

// ─── Ambrosia helpers ─────────────────────────────────────────────────────

/// `raw_luck × multiplier`.
#[must_use]
pub fn calculate_ambrosia_luck(raw_luck: f64, multiplier: f64) -> f64 {
    raw_luck * multiplier
}

/// `raw_speed × blueberries`.
#[must_use]
pub fn calculate_ambrosia_generation_speed(raw_speed: f64, blueberries: f64) -> f64 {
    raw_speed * blueberries
}

// ─── Cube multiplier with tau exponent ────────────────────────────────────

/// `base^tauPower`.
#[must_use]
pub fn calculate_cube_multiplier_with_tau(base: f64, tau_power: f64) -> f64 {
    base.powf(tau_power)
}

// ─── Platonic-7 DR power ──────────────────────────────────────────────────

/// `1 - platonicUpgrades[7] / 30`.
#[must_use]
pub fn calculate_platonic_7_upgrade_power(platonic_upgrade_7: f64) -> f64 {
    1.0 - platonic_upgrade_7 / 30.0
}

// ─── Ascension speed exponent spread ──────────────────────────────────────

/// Sum of three GQ/shop upgrade contributions.
#[must_use]
pub fn calculate_ascension_speed_exponent_spread(
    sing_ascension_speed_exponent_spread: f64,
    sing_ascension_speed_2_exponent_spread: f64,
    chronometer_infinity_exponent_spread: f64,
) -> f64 {
    sing_ascension_speed_exponent_spread
        + sing_ascension_speed_2_exponent_spread
        + chronometer_infinity_exponent_spread
}

// ─── StatLine reducers ────────────────────────────────────────────────────

/// Multiplicative-product reducer (f64).
#[must_use]
pub fn product_f64(stats: &[f64]) -> f64 {
    stats.iter().product()
}

/// Additive-sum reducer (f64).
#[must_use]
pub fn sum_f64(stats: &[f64]) -> f64 {
    stats.iter().sum()
}

/// Multiplicative-product reducer (`Decimal`).
#[must_use]
pub fn product_decimal(stats: &[Decimal]) -> Decimal {
    stats.iter().copied().fold(Decimal::one(), |acc, v| acc * v)
}

// ─── Misc helpers ─────────────────────────────────────────────────────────

/// Inputs to [`calculate_total_coin_owned`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateTotalCoinOwnedInput {
    /// `player.firstOwnedCoin`.
    pub first_owned_coin: f64,
    /// `player.secondOwnedCoin`.
    pub second_owned_coin: f64,
    /// `player.thirdOwnedCoin`.
    pub third_owned_coin: f64,
    /// `player.fourthOwnedCoin`.
    pub fourth_owned_coin: f64,
    /// `player.fifthOwnedCoin`.
    pub fifth_owned_coin: f64,
}

/// Sum of the five coin counters.
#[must_use]
pub fn calculate_total_coin_owned(input: &CalculateTotalCoinOwnedInput) -> f64 {
    input.first_owned_coin
        + input.second_owned_coin
        + input.third_owned_coin
        + input.fourth_owned_coin
        + input.fifth_owned_coin
}

// ─── Ascension score bonus multiplier ─────────────────────────────────────

/// Inputs to [`compute_ascension_score_bonus_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct AscensionScoreBonusMultiplierInput {
    /// `G.challenge15Rewards.score.value`.
    pub challenge_15_score_reward: f64,
    /// `calculateAscensionScorePlatonicBlessing()` output.
    pub platonic_blessing_mult: f64,
    /// `player.campaigns.ascensionScoreMultiplier`.
    pub campaign_ascension_score_mult: f64,
    /// `getRuneEffects('finiteDescent', 'ascensionScore')`.
    pub finite_descent_ascension_score: f64,
    /// `player.cubeUpgrades[21]`.
    pub cube_upgrade_21: f64,
    /// `player.cubeUpgrades[31]`.
    pub cube_upgrade_31: f64,
    /// `player.cubeUpgrades[41]`.
    pub cube_upgrade_41: f64,
    /// `+getAchievementReward('ascensionScore')`.
    pub ascension_score_achievement_reward: f64,
    /// `getGQUpgradeEffect('masterPack', 'ascensionScoreMult')`.
    pub master_pack_ascension_score_mult: f64,
    /// Event buff (0 if no event active).
    pub event_buff: f64,
}

/// Bonus multiplier on top of `(baseScore × corruptionMultiplier)`.
#[must_use]
pub fn compute_ascension_score_bonus_multiplier(input: &AscensionScoreBonusMultiplierInput) -> f64 {
    let mut multiplier = 1.0_f64;
    multiplier *= input.challenge_15_score_reward;
    multiplier *= input.platonic_blessing_mult;
    multiplier *= input.campaign_ascension_score_mult;
    multiplier *= input.finite_descent_ascension_score;
    if input.cube_upgrade_21 > 0.0 {
        multiplier *= 1.0 + 0.05 * input.cube_upgrade_21;
    }
    if input.cube_upgrade_31 > 0.0 {
        multiplier *= 1.0 + 0.05 * input.cube_upgrade_31;
    }
    if input.cube_upgrade_41 > 0.0 {
        multiplier *= 1.0 + 0.05 * input.cube_upgrade_41;
    }
    multiplier *= input.ascension_score_achievement_reward;
    multiplier *= input.master_pack_ascension_score_mult;
    multiplier *= 1.0 + input.event_buff;
    multiplier
}

// ─── Ascension score ──────────────────────────────────────────────────────

const BASE_SCORE_ARRAY: [f64; 11] = [
    0.0, 8.0, 10.0, 12.0, 15.0, 20.0, 60.0, 80.0, 120.0, 180.0, 300.0,
];
const TIER_2_SCORE_ARRAY: [f64; 11] = [
    0.0, 10.0, 12.0, 15.0, 20.0, 30.0, 80.0, 120.0, 180.0, 300.0, 450.0,
];
const TIER_3_SCORE_ARRAY: [f64; 11] = [
    0.0, 20.0, 30.0, 50.0, 100.0, 200.0, 250.0, 300.0, 400.0, 500.0, 750.0,
];
const TIER_4_SCORE_ARRAY: [f64; 11] = [
    0.0, 10_000.0, 10_000.0, 10_000.0, 10_000.0, 10_000.0, 2_000.0, 3_000.0, 4_000.0, 5_000.0,
    7_500.0,
];

/// Inputs to [`calculate_ascension_score`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateAscensionScoreInput<'a> {
    /// `player.highestchallengecompletions`, indexed `0..=10`.
    pub highest_challenge_completions: &'a [f64],
    /// `player.cubeUpgrades[56]`. Added to challenges 1-3 baseline.
    pub cube_upgrade_56: f64,
    /// `player.cubeUpgrades[39]`. C10 exponent contributor.
    pub cube_upgrade_39: f64,
    /// `player.platonicUpgrades[5]` — ALPHA.
    pub platonic_upgrade_5: f64,
    /// `player.platonicUpgrades[10]` — BETA.
    pub platonic_upgrade_10: f64,
    /// `player.corruptions.used.totalCorruptionAscensionMultiplier`.
    pub corruption_multiplier: f64,
    /// `getAntUpgradeEffect(AntUpgrades.AscensionScore).ascensionScoreBase`.
    pub ant_upgrade_ascension_score_base: f64,
    /// `getGQUpgradeEffect('expertPack', 'ascensionScoreMult')`.
    pub expert_pack_ascension_score_mult: f64,
    /// Output of [`compute_ascension_score_bonus_multiplier`].
    pub bonus_multiplier: f64,
}

/// Result of [`calculate_ascension_score`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateAscensionScoreResult {
    /// Pre-multiplier base score.
    pub base_score: f64,
    /// Corruption mult (pass-through).
    pub corruption_multiplier: f64,
    /// Bonus mult (pass-through).
    pub bonus_multiplier: f64,
    /// `base × corruption × bonus` with `1e23` softcap and expertPack.
    pub effective_score: f64,
}

/// Pre-cube ascension score.
#[must_use]
pub fn calculate_ascension_score(
    input: &CalculateAscensionScoreInput<'_>,
) -> CalculateAscensionScoreResult {
    let mut base_score = 0.0_f64;
    let mut challenge_score_arrays_1 = BASE_SCORE_ARRAY;
    challenge_score_arrays_1[1] += input.cube_upgrade_56;
    challenge_score_arrays_1[2] += input.cube_upgrade_56;
    challenge_score_arrays_1[3] += input.cube_upgrade_56;

    for i in 1..=10_usize {
        let completions = input
            .highest_challenge_completions
            .get(i)
            .copied()
            .unwrap_or(0.0);
        base_score += challenge_score_arrays_1[i] * completions;
        if i <= 5 && completions >= 75.0 {
            base_score += TIER_2_SCORE_ARRAY[i] * (completions - 75.0);
            if completions >= 750.0 {
                base_score += TIER_3_SCORE_ARRAY[i] * (completions - 750.0);
            }
            if completions >= 9_000.0 {
                base_score += TIER_4_SCORE_ARRAY[i] * (completions - 9_000.0);
            }
        }
        if (6..=10).contains(&i) && completions >= 25.0 {
            base_score += TIER_2_SCORE_ARRAY[i] * (completions - 25.0);
            if completions >= 60.0 {
                base_score += TIER_3_SCORE_ARRAY[i] * (completions - 60.0);
            }
        }
    }

    base_score += input.ant_upgrade_ascension_score_base;

    let c10_completions = input
        .highest_challenge_completions
        .get(10)
        .copied()
        .unwrap_or(0.0);
    base_score *= (1.03
        + 0.005 * input.cube_upgrade_39
        + 0.0025 * (input.platonic_upgrade_5 + input.platonic_upgrade_10))
        .powf(c10_completions);

    let mut effective_score = base_score * input.corruption_multiplier * input.bonus_multiplier;
    if effective_score > 1e23 {
        effective_score = effective_score.powf(0.5) * 1e23_f64.powf(0.5);
    }
    effective_score *= input.expert_pack_ascension_score_mult;

    CalculateAscensionScoreResult {
        base_score,
        corruption_multiplier: input.corruption_multiplier,
        bonus_multiplier: input.bonus_multiplier,
        effective_score,
    }
}

// ─── CalcCorruptionStuff ──────────────────────────────────────────────────

/// Inputs to [`calc_corruption_stuff`].
#[derive(Debug, Clone, Copy)]
pub struct CalcCorruptionStuffInput {
    /// Output of [`calculate_ascension_score`].
    pub scores: CalculateAscensionScoreResult,
    /// `calculateCubeMultiplierWithTau()`.
    pub cube_multiplier: f64,
    /// `calculateTesseractMultiplier()`.
    pub tesseract_multiplier: f64,
    /// `calculateHypercubeMultiplier()`.
    pub hypercube_multiplier: f64,
    /// `calculatePlatonicMultiplier()`.
    pub platonic_multiplier: f64,
    /// `calculateHepteractMultiplier()`.
    pub hepteract_multiplier: f64,
    /// `G.challenge15Rewards.hepteractsUnlocked.value` — gates hept gain.
    pub hepteracts_unlocked: f64,
    /// `player.singularityCount` — floor for tesseract gain.
    pub singularity_count: f64,
}

/// Result of [`calc_corruption_stuff`].
#[derive(Debug, Clone, Copy)]
pub struct CalcCorruptionStuffResult {
    /// Wow cubes gained.
    pub wow_cubes: f64,
    /// Wow tesseracts gained.
    pub wow_tesseracts: f64,
    /// Wow hypercubes gained.
    pub wow_hypercubes: f64,
    /// Wow platonic cubes gained.
    pub wow_platonic_cubes: f64,
    /// Wow hepteracts gained.
    pub wow_hepteracts: f64,
    /// Pass-through.
    pub base_score: f64,
    /// Pass-through.
    pub bonus_multiplier: f64,
    /// Pass-through.
    pub corruption_multiplier: f64,
    /// Pass-through (floored).
    pub effective_score: f64,
}

/// Cube gains for the current ascension, gated on effective-score
/// thresholds. Each final count is floored and clamped to `1e300`.
/// Tesseracts also have a `max(singularityCount, ...)` floor.
#[must_use]
pub fn calc_corruption_stuff(input: &CalcCorruptionStuffInput) -> CalcCorruptionStuffResult {
    let effective_score = input.scores.effective_score;

    let cube_gain = input.cube_multiplier;

    let mut tesseract_gain = 1.0_f64;
    if effective_score >= 100_000.0 {
        tesseract_gain += 0.5;
    }
    tesseract_gain *= input.tesseract_multiplier;

    let mut hypercube_gain = if effective_score >= 1e9 { 1.0 } else { 0.0 };
    hypercube_gain *= input.hypercube_multiplier;

    let mut platonic_gain = if effective_score >= 2.666e12 {
        1.0
    } else {
        0.0
    };
    platonic_gain *= input.platonic_multiplier;

    let mut hepteract_gain = if input.hepteracts_unlocked != 0.0 && effective_score >= 1.666e17 {
        1.0
    } else {
        0.0
    };
    hepteract_gain *= input.hepteract_multiplier;

    CalcCorruptionStuffResult {
        wow_cubes: 1e300_f64.min(cube_gain.floor()),
        wow_tesseracts: 1e300_f64.min(input.singularity_count.max(tesseract_gain.floor())),
        wow_hypercubes: 1e300_f64.min(hypercube_gain.floor()),
        wow_platonic_cubes: 1e300_f64.min(platonic_gain.floor()),
        wow_hepteracts: 1e300_f64.min(hepteract_gain.floor()),
        base_score: input.scores.base_score.floor(),
        bonus_multiplier: input.scores.bonus_multiplier,
        corruption_multiplier: input.scores.corruption_multiplier,
        effective_score: effective_score.floor(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_speed_mult_sqrt_branch_above_100() {
        let r = calculate_global_speed_mult(&GlobalSpeedMultInput {
            normal_mult: 400.0,
            immaculate_mult: 1.0,
            dr_power: 1.0,
        });
        // sqrt(400) * 10 = 200
        assert_eq!(r, 200.0);
    }

    #[test]
    fn global_speed_mult_dr_branch_below_1() {
        let r = calculate_global_speed_mult(&GlobalSpeedMultInput {
            normal_mult: 0.5,
            immaculate_mult: 1.0,
            dr_power: 0.5,
        });
        // 0.5^0.5 = sqrt(0.5) ≈ 0.707
        assert!((r - 0.5_f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn ascension_speed_mult_below_1_uses_negative_spread() {
        // base=0.5, spread=0.2 → 0.5^0.8
        let r = calculate_ascension_speed_mult(&AscensionSpeedMultInput {
            base: 0.5,
            exponent_spread: 0.2,
        });
        assert!((r - 0.5_f64.powf(0.8)).abs() < 1e-12);
    }

    #[test]
    fn ant_speed_uses_challenge_penalty() {
        let r = calculate_actual_ant_speed_mult(&ActualAntSpeedMultInput {
            base: Decimal::from_finite(100.0),
            ascension_challenge: 13,
            platonic_upgrade_10: 0.0,
        });
        // 100^0.23
        assert!((r.to_number() - 100.0_f64.powf(0.23)).abs() < 1e-9);
    }

    #[test]
    fn ant_speed_platonic_10_boosts_c15() {
        let plain = calculate_actual_ant_speed_mult(&ActualAntSpeedMultInput {
            base: Decimal::from_finite(100.0),
            ascension_challenge: 15,
            platonic_upgrade_10: 0.0,
        });
        let boosted = calculate_actual_ant_speed_mult(&ActualAntSpeedMultInput {
            base: Decimal::from_finite(100.0),
            ascension_challenge: 15,
            platonic_upgrade_10: 1.0,
        });
        // boosted exponent = 0.5 * 1.25 = 0.625 > 0.5
        assert!(boosted.to_number() > plain.to_number());
    }

    #[test]
    fn reduction_value_combines_inputs() {
        let r = get_reduction_value(&ReductionValueInput {
            thrift_cost_delay: 0.1,
            researches_sum: 100.0,
            challenge_completions_4: 0.0,
            ant_building_cost_scale: 0.2,
        });
        // 1 + 0.1 + 0.5 + 0 + 0.2 = 1.8
        assert!((r - 1.8).abs() < 1e-12);
    }

    #[test]
    fn offerings_taxman_clamp() {
        // current=10, mult=10000 → clamp at 10*100+1 = 1001
        let r = calculate_offerings(&CalculateOfferingsInput {
            base_offerings: 0.0,
            time_multiplier: 1.0,
            offering_mult: Decimal::from_finite(10_000.0),
            taxman_last_stand_enabled: true,
            taxman_last_stand_completions: 2.0,
            current_offerings: Decimal::from_finite(10.0),
        });
        assert_eq!(r.to_number(), 1_001.0);
    }

    #[test]
    fn obtainium_c14_zeros_out() {
        let r = calculate_obtainium(&CalculateObtainiumInput {
            base_obtainium: 1e6,
            immaculate: 1e3,
            dr: 1.0,
            time_multiplier: 1.0,
            base_mults: Decimal::from_finite(1e10),
            in_ascension_challenge_14: true,
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 0.0,
            current_obtainium: Decimal::zero(),
        });
        assert_eq!(r.to_number(), 0.0);
    }

    #[test]
    fn positive_salvage_taxman_log_damped() {
        // raw=1000, mult=1.0 → 100 + 1000 / max(1, ln(1000)) = 100 + 1000/6.908 = 100 + 144.76 = 244.76
        let r = calculate_positive_salvage(&CalculatePositiveSalvageInput {
            raw_positive_salvage: 1_000.0,
            positive_salvage_multiplier: 1.0,
            taxman_last_stand_enabled: true,
        });
        let expected = 100.0 + 1_000.0 / 1_000.0_f64.ln();
        assert!((r - expected).abs() < 1e-9);
    }

    #[test]
    fn total_coin_owned_sums_all_five() {
        let r = calculate_total_coin_owned(&CalculateTotalCoinOwnedInput {
            first_owned_coin: 1.0,
            second_owned_coin: 2.0,
            third_owned_coin: 3.0,
            fourth_owned_coin: 4.0,
            fifth_owned_coin: 5.0,
        });
        assert_eq!(r, 15.0);
    }

    #[test]
    fn ascension_score_zero_when_no_completions() {
        let result = calculate_ascension_score(&CalculateAscensionScoreInput {
            highest_challenge_completions: &[0.0; 11],
            cube_upgrade_56: 0.0,
            cube_upgrade_39: 0.0,
            platonic_upgrade_5: 0.0,
            platonic_upgrade_10: 0.0,
            corruption_multiplier: 1.0,
            ant_upgrade_ascension_score_base: 0.0,
            expert_pack_ascension_score_mult: 1.0,
            bonus_multiplier: 1.0,
        });
        assert_eq!(result.base_score, 0.0);
    }

    #[test]
    fn ascension_score_softcap_at_1e23() {
        let result = calculate_ascension_score(&CalculateAscensionScoreInput {
            highest_challenge_completions: &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            cube_upgrade_56: 0.0,
            cube_upgrade_39: 0.0,
            platonic_upgrade_5: 0.0,
            platonic_upgrade_10: 0.0,
            corruption_multiplier: 1.0,
            ant_upgrade_ascension_score_base: 2e23,
            expert_pack_ascension_score_mult: 1.0,
            bonus_multiplier: 1.0,
        });
        // Past 1e23 → sqrt(2e23) * sqrt(1e23) = sqrt(2) * 1e23
        let expected = 2e23_f64.sqrt() * 1e23_f64.sqrt();
        assert!((result.effective_score - expected).abs() / expected < 1e-9);
    }

    #[test]
    fn corruption_stuff_thresholds() {
        let scores = CalculateAscensionScoreResult {
            base_score: 1e10,
            corruption_multiplier: 1.0,
            bonus_multiplier: 1.0,
            effective_score: 1e10,
        };
        let r = calc_corruption_stuff(&CalcCorruptionStuffInput {
            scores,
            cube_multiplier: 1.0,
            tesseract_multiplier: 1.0,
            hypercube_multiplier: 1.0,
            platonic_multiplier: 1.0,
            hepteract_multiplier: 1.0,
            hepteracts_unlocked: 1.0,
            singularity_count: 0.0,
        });
        // 1e10 >= 1e9 → hypercube=1, but < 2.666e12 → platonic=0, < 1.666e17 → hept=0
        assert_eq!(r.wow_hypercubes, 1.0);
        assert_eq!(r.wow_platonic_cubes, 0.0);
        assert_eq!(r.wow_hepteracts, 0.0);
        // tesseract: >= 1e5 → +0.5 → 1.5, floor=1
        assert_eq!(r.wow_tesseracts, 1.0);
    }

    #[test]
    fn corruption_stuff_singularity_floor_on_tesseract() {
        let scores = CalculateAscensionScoreResult {
            base_score: 0.0,
            corruption_multiplier: 1.0,
            bonus_multiplier: 1.0,
            effective_score: 0.0,
        };
        let r = calc_corruption_stuff(&CalcCorruptionStuffInput {
            scores,
            cube_multiplier: 1.0,
            tesseract_multiplier: 1.0,
            hypercube_multiplier: 1.0,
            platonic_multiplier: 1.0,
            hepteract_multiplier: 1.0,
            hepteracts_unlocked: 1.0,
            singularity_count: 50.0,
        });
        // Tesseract floored at 50
        assert_eq!(r.wow_tesseracts, 50.0);
    }
}
