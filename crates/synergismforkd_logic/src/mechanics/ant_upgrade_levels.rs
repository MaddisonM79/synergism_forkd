//! Ant-upgrade level helpers: free-level aggregator + true-level
//! resolver.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/antUpgradeLevels.ts`.
//! `compute_free_ant_upgrade_levels` pre-extracts every player-state
//! input as a numeric field; `calculate_true_ant_level` composes free
//! levels with the corruption divisor and the c11-active branch.
//! Both are pure given inputs.

/// Inputs to [`compute_free_ant_upgrade_levels`].
#[derive(Debug, Clone, Copy)]
pub struct ComputeFreeAntUpgradeLevelsInput {
    /// `CalcECC('reincarnation', player.challengecompletions[9])`.
    pub c9_reincarnation_ecc: f64,
    /// `player.constantUpgrades[6]`.
    pub constant_upgrade_6: f64,
    /// `CalcECC('ascension', player.challengecompletions[11])` —
    /// multiplied by 12.
    pub c11_ascension_ecc: f64,
    /// `player.researches[97]` — `×2`.
    pub research_97: f64,
    /// `player.researches[98]` — `×2`.
    pub research_98: f64,
    /// `player.researches[102]`.
    pub research_102: f64,
    /// `player.researches[132]` — `×2`.
    pub research_132: f64,
    /// `player.researches[200]` — `×(1 / 200)`.
    pub research_200: f64,
    /// `+getAchievementReward('freeAntUpgrades')`.
    pub free_ant_upgrades_achievement_reward: f64,
    /// `Globals.challenge15Rewards.bonusAntLevel.value` — multiplies
    /// the sum.
    pub challenge_15_bonus_ant_level_value: f64,
    /// `player.currentChallenge.ascension === 11` — toggles the c11
    /// bonus tail.
    pub c11_active: bool,
    /// `player.challengecompletions[8]` — `×3` contribution iff
    /// `c11_active`.
    pub c8_completions: f64,
    /// `player.challengecompletions[9]` — `×5` contribution iff
    /// `c11_active`.
    pub c9_completions: f64,
}

/// Total free ant-upgrade levels granted by passive bonuses.
/// Sum-of-sources (research, achievement, challenge ECC), then
/// `×challenge-15` multiplier, then optionally adds the `c11`-active
/// `floor(3*c8 + 5*c9)` tail.
///
/// **Important:** the challenge-15 multiplier applies **before** the
/// c11 tail (the legacy ordering — the c11 add is post-multiplier).
#[must_use]
pub fn compute_free_ant_upgrade_levels(input: &ComputeFreeAntUpgradeLevelsInput) -> f64 {
    let mut bonus_levels = 0.0_f64;
    bonus_levels += input.c9_reincarnation_ecc;
    bonus_levels += (2_000.0 * (1.0 - 0.999_f64.powf(input.constant_upgrade_6))).round();
    bonus_levels += 12.0 * input.c11_ascension_ecc;
    bonus_levels += 2.0 * input.research_97;
    bonus_levels += 2.0 * input.research_98;
    bonus_levels += input.research_102;
    bonus_levels += 2.0 * input.research_132;
    bonus_levels += ((1.0 / 200.0) * input.research_200).floor();
    bonus_levels += input.free_ant_upgrades_achievement_reward;
    bonus_levels *= input.challenge_15_bonus_ant_level_value;

    if input.c11_active {
        bonus_levels += (3.0 * input.c8_completions + 5.0 * input.c9_completions).floor();
    }
    bonus_levels
}

/// Inputs to [`calculate_true_ant_level`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateTrueAntLevelInput {
    /// `player.ants.upgrades[ant_upgrade]` — current purchased level.
    pub current_level: f64,
    /// Result of [`compute_free_ant_upgrade_levels`] (the global
    /// free-level pool — same value across all upgrades). Capped by
    /// `current_level` in the formula.
    pub free_levels: f64,
    /// `antUpgradeData[ant_upgrade].exemptFromCorruption`.
    pub exempt_from_corruption: bool,
    /// `player.corruptions.used.corruptionEffects('extinction')`.
    /// Used as the divisor when not exempt.
    pub corruption_extinction_divisor: f64,
    /// `player.currentChallenge.ascension === 11`. In c11 the
    /// effective level collapses to `min(current, free)` without
    /// the doubling.
    pub c11_active: bool,
}

/// Effective ant-upgrade level. Combines purchased levels + capped
/// free levels (`min(current, free)`), then divides by corruption.
/// `c11` mode caps the contribution to just `min(current, free)`
/// without the additive purchased term.
#[must_use]
pub fn calculate_true_ant_level(input: &CalculateTrueAntLevelInput) -> f64 {
    let corruption_divisor = if input.exempt_from_corruption {
        1.0
    } else {
        input.corruption_extinction_divisor
    };

    if input.c11_active {
        return input.current_level.min(input.free_levels) / corruption_divisor;
    }
    (input.current_level + input.current_level.min(input.free_levels)) / corruption_divisor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_free_input() -> ComputeFreeAntUpgradeLevelsInput {
        ComputeFreeAntUpgradeLevelsInput {
            c9_reincarnation_ecc: 0.0,
            constant_upgrade_6: 0.0,
            c11_ascension_ecc: 0.0,
            research_97: 0.0,
            research_98: 0.0,
            research_102: 0.0,
            research_132: 0.0,
            research_200: 0.0,
            free_ant_upgrades_achievement_reward: 0.0,
            challenge_15_bonus_ant_level_value: 1.0,
            c11_active: false,
            c8_completions: 0.0,
            c9_completions: 0.0,
        }
    }

    #[test]
    fn free_levels_zero_inputs_is_zero() {
        assert_eq!(compute_free_ant_upgrade_levels(&zero_free_input()), 0.0);
    }

    #[test]
    fn free_levels_research_97_at_one_is_two() {
        let input = ComputeFreeAntUpgradeLevelsInput {
            research_97: 1.0,
            ..zero_free_input()
        };
        assert_eq!(compute_free_ant_upgrade_levels(&input), 2.0);
    }

    #[test]
    fn free_levels_challenge_15_multiplies_before_c11_tail() {
        // Sum-pre-mult: research_97=1 → 2; c8=10, c9=10
        // challenge_15 = 2 → 2 * 2 = 4 (challenge mult applied)
        // c11_active: + (3*10 + 5*10) = +80 (added post-multiplier)
        // total = 4 + 80 = 84
        let input = ComputeFreeAntUpgradeLevelsInput {
            research_97: 1.0,
            challenge_15_bonus_ant_level_value: 2.0,
            c11_active: true,
            c8_completions: 10.0,
            c9_completions: 10.0,
            ..zero_free_input()
        };
        assert_eq!(compute_free_ant_upgrade_levels(&input), 84.0);
    }

    #[test]
    fn true_ant_level_exempt_skips_corruption() {
        let result = calculate_true_ant_level(&CalculateTrueAntLevelInput {
            current_level: 100.0,
            free_levels: 50.0,
            exempt_from_corruption: true,
            corruption_extinction_divisor: 10.0,
            c11_active: false,
        });
        // 100 + min(100, 50) = 100 + 50 = 150, / 1 = 150
        assert_eq!(result, 150.0);
    }

    #[test]
    fn true_ant_level_with_corruption_divides() {
        let result = calculate_true_ant_level(&CalculateTrueAntLevelInput {
            current_level: 100.0,
            free_levels: 50.0,
            exempt_from_corruption: false,
            corruption_extinction_divisor: 3.0,
            c11_active: false,
        });
        // 150 / 3 = 50
        assert_eq!(result, 50.0);
    }

    #[test]
    fn true_ant_level_c11_caps_at_min_no_double() {
        let result = calculate_true_ant_level(&CalculateTrueAntLevelInput {
            current_level: 100.0,
            free_levels: 30.0,
            exempt_from_corruption: true,
            corruption_extinction_divisor: 1.0,
            c11_active: true,
        });
        // min(100, 30) / 1 = 30
        assert_eq!(result, 30.0);
    }

    #[test]
    fn true_ant_level_free_levels_capped_at_current() {
        let result = calculate_true_ant_level(&CalculateTrueAntLevelInput {
            current_level: 50.0,
            free_levels: 200.0,
            exempt_from_corruption: true,
            corruption_extinction_divisor: 1.0,
            c11_active: false,
        });
        // 50 + min(50, 200) = 50 + 50 = 100
        assert_eq!(result, 100.0);
    }
}
