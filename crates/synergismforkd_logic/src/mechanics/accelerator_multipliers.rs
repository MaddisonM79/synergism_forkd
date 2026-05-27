//! Total free-accelerator-boost and accelerator-multiplier formulas.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/acceleratorMultipliers.ts`
//! (lifted from the legacy `packages/web_ui/src/Calculate.ts`). Both
//! functions read a lot of player / G state; the UI shim collects
//! every input field and passes them in scalar form.
//!
//! G side-effect note: the legacy
//! `calculateTotalAcceleratorBoost` and
//! `calculateAcceleratorMultiplier` wrote to
//! `G.freeAcceleratorBoost` / `G.totalAcceleratorBoost` /
//! `G.acceleratorMultiplier` directly. Logic stays pure — it returns
//! the computed values; the UI shim does the writes.

use super::challenges::{calc_ecc, ChallengeType};

// ─── Total accelerator boost (free + total) ────────────────────────────────

/// Inputs to [`calculate_total_accelerator_boost`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateTotalAcceleratorBoostInput {
    /// `player.upgrades[26]` — `+1` free boost when `> 0.5`.
    pub upgrade_26: f64,
    /// `player.upgrades[31]` — adds `floor(totalCoinOwned / 2000)`
    /// when `> 0.5`.
    pub upgrade_31: f64,
    /// Precomputed by `calculateTotalCoinOwned` in the legacy UI.
    pub total_coin_owned: f64,
    /// `+getAchievementReward('accelBoosts')` — the achievement
    /// reward already comes back as a number.
    pub achievement_accel_boosts: f64,
    /// `player.researches[93]` — multiplies
    /// `floor(sum_of_rune_levels / 20)`.
    pub research_93: f64,
    /// `sumOfRuneLevels()` in the legacy UI.
    pub sum_of_rune_levels: f64,
    /// `player.researches[3]`.
    pub research_3: f64,
    /// `player.challengecompletions[14]` — feeds
    /// `CalcECC('ascension', cc14)`.
    pub challenge_completions_14: f64,
    /// `player.researches[16]`.
    pub research_16: f64,
    /// `player.researches[17]`.
    pub research_17: f64,
    /// `player.researches[88]`.
    pub research_88: f64,
    /// `getAntUpgradeEffect(AntUpgrades.AcceleratorBoosts).acceleratorBoostMult`.
    pub ant_building_accelerator_boost_mult: f64,
    /// `player.researches[127]`.
    pub research_127: f64,
    /// `player.researches[142]`.
    pub research_142: f64,
    /// `player.researches[157]`.
    pub research_157: f64,
    /// `player.researches[172]`.
    pub research_172: f64,
    /// `player.researches[187]`.
    pub research_187: f64,
    /// `player.researches[200]`.
    pub research_200: f64,
    /// `player.cubeUpgrades[50]`.
    pub cube_upgrade_50: f64,
    /// `hepteractEffective('acceleratorBoost')` in the legacy UI.
    pub hepteract_effective_accelerator_boost: f64,
    /// `player.upgrades[73]` — doubles boost when also in a
    /// reincarnation challenge.
    pub upgrade_73: f64,
    /// `true` when
    /// `player.currentChallenge.reincarnation !== 0`.
    pub in_reincarnation_challenge: bool,
    /// `player.acceleratorBoostBought` — added to free boost for the
    /// total.
    pub accelerator_boost_bought: f64,
}

/// Result of [`calculate_total_accelerator_boost`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalculateTotalAcceleratorBoostResult {
    /// What the UI assigns to `G.freeAcceleratorBoost`.
    pub free_accelerator_boost: f64,
    /// What the UI assigns to `G.totalAcceleratorBoost`.
    pub total_accelerator_boost: f64,
}

/// Computes the "free" accelerator-boost amount from upgrades,
/// achievements, researches, rune levels, ant building effects,
/// hepteract effectiveness, and various cube upgrades. The
/// reincarnation-challenge × `upgrade_73` gate doubles the result.
/// Floored, then capped at `1e100`.
///
/// `total_accelerator_boost = floor(bought + free * 100) / 100` —
/// the original floor-to-1-decimal that the legacy UI assigned to
/// `G.totalAcceleratorBoost`.
#[must_use]
pub fn calculate_total_accelerator_boost(
    input: &CalculateTotalAcceleratorBoostInput,
) -> CalculateTotalAcceleratorBoostResult {
    let mut b = 0.0_f64;
    if input.upgrade_26 > 0.5 {
        b += 1.0;
    }
    if input.upgrade_31 > 0.5 {
        b += ((input.total_coin_owned / 2_000.0).floor() * 100.0) / 100.0;
    }
    b += input.achievement_accel_boosts;

    b += input.research_93 * (1.0 / 20.0 * input.sum_of_rune_levels).floor();
    b *= 1.0
        + (1.0 / 5.0)
            * input.research_3
            * (1.0
                + (1.0 / 2.0) * calc_ecc(ChallengeType::Ascension, input.challenge_completions_14));
    b *= 1.0 + (1.0 / 20.0) * input.research_16 + (1.0 / 20.0) * input.research_17;
    b *= 1.0 + (1.0 / 20.0) * input.research_88;
    b *= input.ant_building_accelerator_boost_mult;
    b *= 1.0 + (1.0 / 100.0) * input.research_127;
    b *= 1.0 + (0.8 / 100.0) * input.research_142;
    b *= 1.0 + (0.6 / 100.0) * input.research_157;
    b *= 1.0 + (0.4 / 100.0) * input.research_172;
    b *= 1.0 + (0.2 / 100.0) * input.research_187;
    b *= 1.0 + (0.01 / 100.0) * input.research_200;
    b *= 1.0 + (0.01 / 100.0) * input.cube_upgrade_50;
    b *= 1.0 + (1.0 / 1_000.0) * input.hepteract_effective_accelerator_boost;
    if input.upgrade_73 > 0.5 && input.in_reincarnation_challenge {
        b *= 2.0;
    }
    b = 1e100_f64.min(b.floor());

    let free_accelerator_boost = b;
    let total_accelerator_boost =
        ((input.accelerator_boost_bought + free_accelerator_boost).floor() * 100.0) / 100.0;
    CalculateTotalAcceleratorBoostResult {
        free_accelerator_boost,
        total_accelerator_boost,
    }
}

// ─── Accelerator multiplier ────────────────────────────────────────────────

/// Inputs to [`calculate_accelerator_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateAcceleratorMultiplierInput {
    /// `player.researches[1]`.
    pub research_1: f64,
    /// `player.challengecompletions[14]` — feeds
    /// `CalcECC('ascension', cc14)`.
    pub challenge_completions_14: f64,
    /// `player.researches[6]`.
    pub research_6: f64,
    /// `player.researches[7]`.
    pub research_7: f64,
    /// `player.researches[8]`.
    pub research_8: f64,
    /// `player.researches[9]`.
    pub research_9: f64,
    /// `player.researches[10]`.
    pub research_10: f64,
    /// `player.researches[86]`.
    pub research_86: f64,
    /// `player.researches[126]`.
    pub research_126: f64,
    /// `player.researches[141]`.
    pub research_141: f64,
    /// `player.researches[156]`.
    pub research_156: f64,
    /// `player.researches[171]`.
    pub research_171: f64,
    /// `player.researches[186]`.
    pub research_186: f64,
    /// `player.researches[200]`.
    pub research_200: f64,
    /// `player.cubeUpgrades[50]`.
    pub cube_upgrade_50: f64,
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
    /// `player.upgrades[50]` — combined with the in-challenge gate,
    /// multiplies by `1.25`.
    pub upgrade_50: f64,
    /// `true` when
    /// `player.currentChallenge.transcension !== 0` OR
    /// `player.currentChallenge.reincarnation !== 0`.
    pub in_transcension_or_reincarnation_challenge: bool,
}

/// Compounding multiplier built from research levels, cube upgrades,
/// the `21..=25` upgrade pentad (`1.01^sum`), and an optional `1.25×`
/// from `upgrade[50]` while in a transcension or reincarnation
/// challenge.
#[must_use]
pub fn calculate_accelerator_multiplier(input: &CalculateAcceleratorMultiplierInput) -> f64 {
    let mut multiplier = 1.0_f64;
    multiplier *= 1.0
        + (1.0 / 5.0)
            * input.research_1
            * (1.0
                + (1.0 / 2.0) * calc_ecc(ChallengeType::Ascension, input.challenge_completions_14));
    multiplier *= 1.0
        + (1.0 / 20.0) * input.research_6
        + (1.0 / 25.0) * input.research_7
        + (1.0 / 40.0) * input.research_8
        + (3.0 / 200.0) * input.research_9
        + (1.0 / 200.0) * input.research_10;
    multiplier *= 1.0 + (1.0 / 20.0) * input.research_86;
    multiplier *= 1.0 + (1.0 / 100.0) * input.research_126;
    multiplier *= 1.0 + (0.8 / 100.0) * input.research_141;
    multiplier *= 1.0 + (0.6 / 100.0) * input.research_156;
    multiplier *= 1.0 + (0.4 / 100.0) * input.research_171;
    multiplier *= 1.0 + (0.2 / 100.0) * input.research_186;
    multiplier *= 1.0 + (0.01 / 100.0) * input.research_200;
    multiplier *= 1.0 + (0.01 / 100.0) * input.cube_upgrade_50;
    multiplier *= 1.01_f64.powf(
        input.upgrade_21
            + input.upgrade_22
            + input.upgrade_23
            + input.upgrade_24
            + input.upgrade_25,
    );
    if input.in_transcension_or_reincarnation_challenge && input.upgrade_50 > 0.5 {
        multiplier *= 1.25;
    }
    multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_boost_input() -> CalculateTotalAcceleratorBoostInput {
        CalculateTotalAcceleratorBoostInput {
            upgrade_26: 0.0,
            upgrade_31: 0.0,
            total_coin_owned: 0.0,
            achievement_accel_boosts: 0.0,
            research_93: 0.0,
            sum_of_rune_levels: 0.0,
            research_3: 0.0,
            challenge_completions_14: 0.0,
            research_16: 0.0,
            research_17: 0.0,
            research_88: 0.0,
            ant_building_accelerator_boost_mult: 1.0,
            research_127: 0.0,
            research_142: 0.0,
            research_157: 0.0,
            research_172: 0.0,
            research_187: 0.0,
            research_200: 0.0,
            cube_upgrade_50: 0.0,
            hepteract_effective_accelerator_boost: 0.0,
            upgrade_73: 0.0,
            in_reincarnation_challenge: false,
            accelerator_boost_bought: 0.0,
        }
    }

    fn zero_mult_input() -> CalculateAcceleratorMultiplierInput {
        CalculateAcceleratorMultiplierInput {
            research_1: 0.0,
            challenge_completions_14: 0.0,
            research_6: 0.0,
            research_7: 0.0,
            research_8: 0.0,
            research_9: 0.0,
            research_10: 0.0,
            research_86: 0.0,
            research_126: 0.0,
            research_141: 0.0,
            research_156: 0.0,
            research_171: 0.0,
            research_186: 0.0,
            research_200: 0.0,
            cube_upgrade_50: 0.0,
            upgrade_21: 0.0,
            upgrade_22: 0.0,
            upgrade_23: 0.0,
            upgrade_24: 0.0,
            upgrade_25: 0.0,
            upgrade_50: 0.0,
            in_transcension_or_reincarnation_challenge: false,
        }
    }

    // ─── total_accelerator_boost ───────────────────────────────────────────

    #[test]
    fn boost_baseline_is_zero() {
        let result = calculate_total_accelerator_boost(&zero_boost_input());
        assert_eq!(result.free_accelerator_boost, 0.0);
        assert_eq!(result.total_accelerator_boost, 0.0);
    }

    #[test]
    fn boost_upgrade_26_adds_one() {
        let input = CalculateTotalAcceleratorBoostInput {
            upgrade_26: 1.0,
            ..zero_boost_input()
        };
        let result = calculate_total_accelerator_boost(&input);
        assert_eq!(result.free_accelerator_boost, 1.0);
        assert_eq!(result.total_accelerator_boost, 1.0);
    }

    #[test]
    fn boost_includes_accelerator_boost_bought_in_total() {
        let input = CalculateTotalAcceleratorBoostInput {
            upgrade_26: 1.0,
            accelerator_boost_bought: 5.0,
            ..zero_boost_input()
        };
        let result = calculate_total_accelerator_boost(&input);
        // 5 + 1 = 6, floored × 100 / 100 = 6
        assert_eq!(result.total_accelerator_boost, 6.0);
    }

    #[test]
    fn boost_double_in_reincarnation_with_upgrade_73() {
        let input = CalculateTotalAcceleratorBoostInput {
            upgrade_26: 1.0,
            upgrade_73: 1.0,
            in_reincarnation_challenge: true,
            ..zero_boost_input()
        };
        let result = calculate_total_accelerator_boost(&input);
        // 1 * 2 = 2
        assert_eq!(result.free_accelerator_boost, 2.0);
    }

    #[test]
    fn boost_caps_at_1e100() {
        // 1e100 * massive multiplier should still cap.
        let input = CalculateTotalAcceleratorBoostInput {
            achievement_accel_boosts: 1e100,
            research_127: 1e6, // 1 + 1e4 → massive multiplier
            ..zero_boost_input()
        };
        let result = calculate_total_accelerator_boost(&input);
        assert!(result.free_accelerator_boost <= 1e100);
    }

    // ─── accelerator_multiplier ────────────────────────────────────────────

    #[test]
    fn multiplier_baseline_is_one() {
        let result = calculate_accelerator_multiplier(&zero_mult_input());
        assert_eq!(result, 1.0);
    }

    #[test]
    fn multiplier_research_6_at_one_adds_5_percent() {
        let input = CalculateAcceleratorMultiplierInput {
            research_6: 1.0,
            ..zero_mult_input()
        };
        let result = calculate_accelerator_multiplier(&input);
        assert!((result - 1.05).abs() < 1e-12);
    }

    #[test]
    fn multiplier_upgrade_pentad_uses_1_01_pow_sum() {
        // upgrades 21..=25 sum to 5 → 1.01^5
        let input = CalculateAcceleratorMultiplierInput {
            upgrade_21: 1.0,
            upgrade_22: 1.0,
            upgrade_23: 1.0,
            upgrade_24: 1.0,
            upgrade_25: 1.0,
            ..zero_mult_input()
        };
        let result = calculate_accelerator_multiplier(&input);
        assert!((result - 1.01_f64.powi(5)).abs() < 1e-12);
    }

    #[test]
    fn multiplier_upgrade_50_inside_challenge_adds_25_percent() {
        let input = CalculateAcceleratorMultiplierInput {
            upgrade_50: 1.0,
            in_transcension_or_reincarnation_challenge: true,
            ..zero_mult_input()
        };
        let result = calculate_accelerator_multiplier(&input);
        assert_eq!(result, 1.25);
    }

    #[test]
    fn multiplier_upgrade_50_outside_challenge_does_nothing() {
        let input = CalculateAcceleratorMultiplierInput {
            upgrade_50: 1.0,
            in_transcension_or_reincarnation_challenge: false,
            ..zero_mult_input()
        };
        let result = calculate_accelerator_multiplier(&input);
        assert_eq!(result, 1.0);
    }
}
