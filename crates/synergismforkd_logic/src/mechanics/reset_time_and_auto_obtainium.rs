//! Reset-time-threshold + automatic-research-obtainium calculator.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/resetTimeAndAutoObtainium.ts`
//! (lifted from the legacy
//! `packages/web_ui/src/Calculate.ts`'s `resetTimeThreshold` +
//! `calculateResearchAutomaticObtainium`).
//!
//! [`reset_time_threshold`] is a 2-input affine function.
//! [`calculate_research_automatic_obtainium`] is the offline
//! obtainium gain from research idle production; the caller
//! pre-extracts every input including the already-computed
//! obtainium / base-obtainium / ant-sacrifice-obtainium /
//! global-speed multipliers from the calculate-side logic.

use synergismforkd_bignum::Decimal;

/// Inputs to [`reset_time_threshold`].
#[derive(Debug, Clone, Copy)]
pub struct ResetTimeThresholdInput {
    /// `player.campaigns.timeThresholdReduction` — subtracted from
    /// the base `10`.
    pub campaign_time_threshold_reduction: f64,
}

/// Reset-time threshold (in seconds):
/// `10 - campaign_time_threshold_reduction`.
#[must_use]
pub fn reset_time_threshold(input: &ResetTimeThresholdInput) -> f64 {
    10.0 - input.campaign_time_threshold_reduction
}

/// Inputs to [`calculate_research_automatic_obtainium`].
#[derive(Debug, Clone, Copy)]
pub struct ResearchAutomaticObtainiumInput {
    /// Tick delta time in seconds.
    pub delta_time: f64,
    /// `player.currentChallenge.ascension` — short-circuits to 0
    /// when `== 14`.
    pub ascension_challenge: u32,
    /// `player.researches[61]` — `×0.5` contribution to the
    /// multiplier.
    pub research_61: f64,
    /// `player.researches[62]` — `×0.1` contribution.
    pub research_62: f64,
    /// `player.cubeUpgrades[3]` — `×0.8` contribution.
    pub cube_upgrade_3: f64,
    /// `player.cubeUpgrades[47]` — non-zero enables the
    /// ant-sacrifice branch.
    pub cube_upgrade_47: f64,
    /// Pre-evaluated `calculateObtainium(false)`.
    pub resource_mult: Decimal,
    /// Pre-evaluated `calculateGlobalSpeedMult()`.
    pub global_speed_mult: f64,
    /// Pre-evaluated `resetTimeThreshold()` — used as the
    /// time-penalty divisor.
    pub reset_time_divisor: f64,
    /// `player.reincarnationcounter` — capped at
    /// `reset_time_divisor`.
    pub reincarnation_counter: f64,
    /// Pre-evaluated `calculateBaseObtainium()`.
    pub base_obtainium: f64,
    /// Pre-evaluated
    /// `calculateAntSacrificeObtainium(antSacrificeStageMult, false)`.
    pub ant_sacrifice_obtainium: Decimal,
    /// `player.antSacrificeTimer` — capped at `reset_time_divisor` in
    /// the ant branch.
    pub ant_sacrifice_timer: f64,
}

/// Per-tick automatic obtainium from research idle gain. Returns `0`
/// in challenge 14 OR when the per-upgrade multiplier is `0` (no
/// enabling researches / cube upgrades).
///
/// Compares three obtainium sources (base / current-resource ×
/// time-penalty / ant-sacrifice × time-penalty when `cube_upgrade_47
/// > 0`) and takes the max, scaled by
/// `delta_time / reset_time_divisor × multiplier`.
#[must_use]
pub fn calculate_research_automatic_obtainium(input: &ResearchAutomaticObtainiumInput) -> Decimal {
    if input.ascension_challenge == 14 {
        return Decimal::zero();
    }

    let multiplier = 0.5 * input.research_61 + 0.1 * input.research_62 + 0.8 * input.cube_upgrade_3;

    if multiplier == 0.0 {
        return Decimal::zero();
    }

    let time_penalty_mult = 1.0_f64.min(input.reincarnation_counter / input.reset_time_divisor);
    let non_base_value = input.resource_mult
        * Decimal::from_finite(input.global_speed_mult)
        * Decimal::from_finite(time_penalty_mult);

    let mut non_base_ant_value = Decimal::zero();
    if input.cube_upgrade_47 > 0.0 {
        let ant_time_penalty_mult =
            1.0_f64.min(input.ant_sacrifice_timer / input.reset_time_divisor);
        non_base_ant_value = input.ant_sacrifice_obtainium
            * Decimal::from_finite(input.global_speed_mult)
            * Decimal::from_finite(ant_time_penalty_mult);
    }

    Decimal::from_finite(input.base_obtainium).max(non_base_value.max(non_base_ant_value))
        * Decimal::from_finite(input.delta_time)
        / Decimal::from_finite(input.reset_time_divisor)
        * Decimal::from_finite(multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_time_threshold_base_is_10() {
        let result = reset_time_threshold(&ResetTimeThresholdInput {
            campaign_time_threshold_reduction: 0.0,
        });
        assert_eq!(result, 10.0);
    }

    #[test]
    fn reset_time_threshold_subtracts_reduction() {
        let result = reset_time_threshold(&ResetTimeThresholdInput {
            campaign_time_threshold_reduction: 2.0,
        });
        assert_eq!(result, 8.0);
    }

    fn baseline_auto_input() -> ResearchAutomaticObtainiumInput {
        ResearchAutomaticObtainiumInput {
            delta_time: 1.0,
            ascension_challenge: 0,
            research_61: 0.0,
            research_62: 0.0,
            cube_upgrade_3: 0.0,
            cube_upgrade_47: 0.0,
            resource_mult: Decimal::from_finite(1.0),
            global_speed_mult: 1.0,
            reset_time_divisor: 10.0,
            reincarnation_counter: 5.0,
            base_obtainium: 1.0,
            ant_sacrifice_obtainium: Decimal::zero(),
            ant_sacrifice_timer: 0.0,
        }
    }

    #[test]
    fn auto_obtainium_returns_zero_in_challenge_14() {
        let input = ResearchAutomaticObtainiumInput {
            ascension_challenge: 14,
            research_61: 1.0, // would otherwise multiply
            ..baseline_auto_input()
        };
        assert_eq!(
            calculate_research_automatic_obtainium(&input),
            Decimal::zero()
        );
    }

    #[test]
    fn auto_obtainium_returns_zero_with_no_multiplier_inputs() {
        // All three multiplier sources are zero → result is zero
        let input = baseline_auto_input();
        assert_eq!(
            calculate_research_automatic_obtainium(&input),
            Decimal::zero()
        );
    }

    #[test]
    fn auto_obtainium_with_research_61_produces_positive() {
        let input = ResearchAutomaticObtainiumInput {
            research_61: 1.0, // multiplier = 0.5
            ..baseline_auto_input()
        };
        let result = calculate_research_automatic_obtainium(&input);
        assert!(result > Decimal::zero());
    }

    #[test]
    fn auto_obtainium_picks_max_of_three_sources() {
        // Base = 1, resource_mult × global × time_penalty = 1 * 1 * 0.5 = 0.5,
        // ant branch disabled.
        // max(1, 0.5, 0) = 1 → 1 * delta_time / divisor * mult = 1*1/10*0.5 = 0.05
        let input = ResearchAutomaticObtainiumInput {
            research_61: 1.0,
            base_obtainium: 1.0,
            ..baseline_auto_input()
        };
        let result = calculate_research_automatic_obtainium(&input);
        assert!((result.to_number() - 0.05).abs() < 1e-12);
    }

    #[test]
    fn auto_obtainium_ant_branch_only_active_with_cube_upgrade_47() {
        let without_47 = ResearchAutomaticObtainiumInput {
            research_61: 1.0,
            ant_sacrifice_obtainium: Decimal::from_finite(1e10),
            ant_sacrifice_timer: 10.0,
            ..baseline_auto_input()
        };
        let with_47 = ResearchAutomaticObtainiumInput {
            cube_upgrade_47: 1.0,
            ..without_47
        };
        let r_without = calculate_research_automatic_obtainium(&without_47);
        let r_with = calculate_research_automatic_obtainium(&with_47);
        // With 47, the ant branch dominates → larger result.
        assert!(r_with > r_without);
    }
}
