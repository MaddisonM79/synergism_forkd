//! Ambrosia-family formulas.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/ambrosia.ts`. The
//! external effect lookups (`getShopUpgradeEffects`,
//! `getAmbrosiaUpgradeEffects`, `getSingularityChallengeEffect`,
//! `getGQUpgradeEffect`, `getOcteractUpgradeEffect`) stay in the UI
//! tier — callers precompute them and pass scalar/array inputs.

// ─── Threshold counters ───────────────────────────────────────────────────

const DIGIT_REDUCTION: i32 = 4;

/// Number of crossed "digit-reduction" thresholds. The first
/// threshold is at 10,000 lifetime ambrosia, and they alternate at
/// `10^(N + DIGIT_REDUCTION)` and `3 × 10^(N + DIGIT_REDUCTION)`.
/// Returns `0` below the first threshold.
#[must_use]
pub fn calculate_number_of_thresholds(lifetime_ambrosia: f64) -> f64 {
    let num_digits = if lifetime_ambrosia > 0.0 {
        1.0 + lifetime_ambrosia.log10().floor()
    } else {
        0.0
    };
    let mantissa = (lifetime_ambrosia / 10.0_f64.powf(num_digits - 1.0)).floor();
    let extra_reduction = if mantissa >= 3.0 { 1.0 } else { 0.0 };
    0.0_f64.max(2.0 * (num_digits - f64::from(DIGIT_REDUCTION)) - 1.0 + extra_reduction)
}

/// Distance (in lifetime ambrosia) to the next threshold. Mirrors
/// the two alternating threshold forms — `10^N` and `3 × 10^N`.
#[must_use]
pub fn calculate_to_next_threshold(lifetime_ambrosia: f64) -> f64 {
    let num_thresholds = calculate_number_of_thresholds(lifetime_ambrosia);
    if num_thresholds == 0.0 {
        return 10_000.0 - lifetime_ambrosia;
    }
    if (num_thresholds % 2.0).abs() < f64::EPSILON {
        return 10.0_f64.powf(num_thresholds / 2.0 + f64::from(DIGIT_REDUCTION))
            - lifetime_ambrosia;
    }
    3.0 * 10.0_f64.powf((num_thresholds - 1.0) / 2.0 + f64::from(DIGIT_REDUCTION))
        - lifetime_ambrosia
}

// ─── Required blueberry / red ambrosia times ──────────────────────────────

/// Inputs to [`calculate_required_blueberry_time`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateRequiredBlueberryTimeInput {
    /// `G.TIME_PER_AMBROSIA` — base time-per-bar constant (currently
    /// 45).
    pub time_per_ambrosia: f64,
    /// `player.lifetimeAmbrosia` — drives both the `+1/300` linear
    /// ramp and the `>= 10000` power scaling.
    pub lifetime_ambrosia: f64,
    /// `getShopUpgradeEffects('shopAmbrosiaAccelerator',
    /// 'ambrosiaPointRequirementMult')`.
    pub accelerator_mult: f64,
    /// `getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead',
    /// 'barRequirementMult')`.
    pub brick_of_lead_mult: f64,
}

/// Time (in seconds, rounded up past 10k) required to fill one
/// ambrosia bar. Base ramps linearly by `lifetime/300`, multiplied
/// by the shop accelerator and brick-of-lead, with a
/// `(lifetime / 10000)^log10(4)` power kick above 10,000.
#[must_use]
pub fn calculate_required_blueberry_time(input: &CalculateRequiredBlueberryTimeInput) -> f64 {
    let mut val = input.time_per_ambrosia;
    val += (input.lifetime_ambrosia / 300.0).floor();
    val *= input.accelerator_mult;
    val *= input.brick_of_lead_mult;
    if input.lifetime_ambrosia >= 10_000.0 {
        let extra_scaling_power = 4.0_f64.log10();
        val *= (input.lifetime_ambrosia / 10_000.0).powf(extra_scaling_power);
        return val.ceil();
    }
    val
}

/// Inputs to [`calculate_required_red_ambrosia_time`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateRequiredRedAmbrosiaTimeInput {
    /// `G.TIME_PER_RED_AMBROSIA` — base time-per-bar constant
    /// (currently 100,000).
    pub time_per_red_ambrosia: f64,
    /// `player.lifetimeRedAmbrosia` — drives the `+200` per linear
    /// ramp.
    pub lifetime_red_ambrosia: f64,
    /// `getSingularityChallengeEffect('limitedTime',
    /// 'barRequirementMultiplier')`.
    pub bar_requirement_multiplier: f64,
}

/// Time (in seconds) required to fill one red ambrosia bar. Linear
/// in `lifetime_red_ambrosia` then multiplied by the
/// limitedTime-challenge effect, capped at
/// `1e6 * bar_requirement_multiplier`.
#[must_use]
pub fn calculate_required_red_ambrosia_time(input: &CalculateRequiredRedAmbrosiaTimeInput) -> f64 {
    let mut val = input.time_per_red_ambrosia;
    val += 200.0 * input.lifetime_red_ambrosia;
    let max = 1e6 * input.bar_requirement_multiplier;
    val *= input.bar_requirement_multiplier;
    max.min(val)
}

// ─── Singularity milestone blueberries ────────────────────────────────────

/// Number of blueberries unlocked by all-time max singularity
/// count. Stepwise: `64 → 1, 128 → 2, 192 → 3, 256 → 4, 270 → 5`.
#[must_use]
pub fn calculate_singularity_milestone_blueberries(highest_singularity_count: f64) -> f64 {
    if highest_singularity_count >= 270.0 {
        return 5.0;
    }
    if highest_singularity_count >= 256.0 {
        return 4.0;
    }
    if highest_singularity_count >= 192.0 {
        return 3.0;
    }
    if highest_singularity_count >= 128.0 {
        return 2.0;
    }
    if highest_singularity_count >= 64.0 {
        return 1.0;
    }
    0.0
}

// ─── Ambrosia cube / quark multipliers ────────────────────────────────────

/// Shared input for the cube / quark mult formulas.
#[derive(Debug, Clone, Copy)]
pub struct AmbrosiaMultInput {
    /// `player.singularityChallenges.noAmbrosiaUpgrades.enabled` —
    /// when true, effective ambrosia is 0.
    pub no_ambrosia_upgrades_enabled: bool,
    /// `player.lifetimeAmbrosia`.
    pub lifetime_ambrosia: f64,
}

/// Cube multiplier from lifetime ambrosia. Three additive tiers:
/// `floor(eff/66)/100` (cap 1.5), `+floor(eff/666)/100` (cap 1.5) if
/// `eff >= 10000`, `+floor(eff/6666)/100` (uncapped) if `eff >=
/// 100000`. The `noAmbrosiaUpgrades` Exalt zeros out effective
/// ambrosia.
#[must_use]
pub fn calculate_ambrosia_cube_mult(input: &AmbrosiaMultInput) -> f64 {
    let effective_ambrosia = if input.no_ambrosia_upgrades_enabled {
        0.0
    } else {
        input.lifetime_ambrosia
    };
    let mut multiplier = 1.0_f64;
    multiplier += 1.5_f64.min((effective_ambrosia / 66.0).floor() / 100.0);
    if effective_ambrosia >= 10_000.0 {
        multiplier += 1.5_f64.min((effective_ambrosia / 666.0).floor() / 100.0);
    }
    if effective_ambrosia >= 100_000.0 {
        multiplier += (effective_ambrosia / 6_666.0).floor() / 100.0;
    }
    multiplier
}

/// Quark multiplier from lifetime ambrosia. Same three-tier shape
/// as the cube mult but smaller divisors and a `0.3` per-tier cap
/// on the first two.
#[must_use]
pub fn calculate_ambrosia_quark_mult(input: &AmbrosiaMultInput) -> f64 {
    let effective_ambrosia = if input.no_ambrosia_upgrades_enabled {
        0.0
    } else {
        input.lifetime_ambrosia
    };
    let mut multiplier = 1.0_f64;
    multiplier += 0.3_f64.min((effective_ambrosia / 1_666.0).floor() / 100.0);
    if effective_ambrosia >= 50_000.0 {
        multiplier += 0.3_f64.min((effective_ambrosia / 16_666.0).floor() / 100.0);
    }
    if effective_ambrosia >= 500_000.0 {
        multiplier += (effective_ambrosia / 166_666.0).floor() / 100.0;
    }
    multiplier
}

// ─── Upgrade composers ────────────────────────────────────────────────────
//
// Each composer takes the 4 precomputed effect values for the
// relevant upgrade chain (singAmbrosiaGeneration / 2/3/4 etc.) and
// reduces them — multiplicative for generation-speed, additive for
// luck.

/// Multiplies the four `singAmbrosiaGeneration[1..4]`
/// `ambrosiaBarSpeedMult` effects together.
#[must_use]
pub fn calculate_ambrosia_generation_singularity_upgrade(speed_mults: &[f64]) -> f64 {
    speed_mults.iter().product()
}

/// Sums the four `singAmbrosiaLuck[1..4]` `ambrosiaLuck` effects.
#[must_use]
pub fn calculate_ambrosia_luck_singularity_upgrade(luck_values: &[f64]) -> f64 {
    luck_values.iter().sum()
}

/// Multiplies the four `octeractAmbrosiaGeneration[1..4]` speed-mult
/// effects.
#[must_use]
pub fn calculate_ambrosia_generation_octeract_upgrade(speed_mults: &[f64]) -> f64 {
    speed_mults.iter().product()
}

/// Sums the four `octeractAmbrosiaLuck[1..4]` luck effects.
#[must_use]
pub fn calculate_ambrosia_luck_octeract_upgrade(luck_values: &[f64]) -> f64 {
    luck_values.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_below_10000_is_zero() {
        assert_eq!(calculate_number_of_thresholds(0.0), 0.0);
        assert_eq!(calculate_number_of_thresholds(9_999.0), 0.0);
    }

    #[test]
    fn thresholds_at_10000_is_one() {
        // 10000 → numDigits=5, mantissa=1 → 2*(5-4) - 1 + 0 = 1
        assert_eq!(calculate_number_of_thresholds(10_000.0), 1.0);
    }

    #[test]
    fn thresholds_at_30000_is_two() {
        // 30000 → numDigits=5, mantissa=3 → 2*(5-4) - 1 + 1 = 2
        assert_eq!(calculate_number_of_thresholds(30_000.0), 2.0);
    }

    #[test]
    fn to_next_threshold_below_10000() {
        // 0 ambrosia → next is 10000
        assert_eq!(calculate_to_next_threshold(0.0), 10_000.0);
    }

    #[test]
    fn required_blueberry_time_below_10000_no_power_kick() {
        let result = calculate_required_blueberry_time(&CalculateRequiredBlueberryTimeInput {
            time_per_ambrosia: 45.0,
            lifetime_ambrosia: 600.0,
            accelerator_mult: 1.0,
            brick_of_lead_mult: 1.0,
        });
        // 45 + floor(600/300) = 47; *1*1 = 47
        assert_eq!(result, 47.0);
    }

    #[test]
    fn required_blueberry_time_above_10000_applies_power_kick() {
        let result = calculate_required_blueberry_time(&CalculateRequiredBlueberryTimeInput {
            time_per_ambrosia: 45.0,
            lifetime_ambrosia: 10_000.0,
            accelerator_mult: 1.0,
            brick_of_lead_mult: 1.0,
        });
        // 45 + floor(10000/300) = 45 + 33 = 78
        // 78 * (10000/10000)^log10(4) = 78 * 1 = 78
        assert_eq!(result, 78.0);
    }

    #[test]
    fn required_red_ambrosia_time_caps_at_1e6_times_mult() {
        let result = calculate_required_red_ambrosia_time(&CalculateRequiredRedAmbrosiaTimeInput {
            time_per_red_ambrosia: 100_000.0,
            lifetime_red_ambrosia: 1e6,
            bar_requirement_multiplier: 1.0,
        });
        // val = 100000 + 200*1e6 = 200,100,000; capped at 1e6 = 1,000,000
        assert_eq!(result, 1_000_000.0);
    }

    #[test]
    fn singularity_milestone_blueberries_steps() {
        assert_eq!(calculate_singularity_milestone_blueberries(0.0), 0.0);
        assert_eq!(calculate_singularity_milestone_blueberries(63.0), 0.0);
        assert_eq!(calculate_singularity_milestone_blueberries(64.0), 1.0);
        assert_eq!(calculate_singularity_milestone_blueberries(128.0), 2.0);
        assert_eq!(calculate_singularity_milestone_blueberries(192.0), 3.0);
        assert_eq!(calculate_singularity_milestone_blueberries(256.0), 4.0);
        assert_eq!(calculate_singularity_milestone_blueberries(270.0), 5.0);
        assert_eq!(calculate_singularity_milestone_blueberries(999.0), 5.0);
    }

    #[test]
    fn ambrosia_cube_mult_at_zero_is_one() {
        let result = calculate_ambrosia_cube_mult(&AmbrosiaMultInput {
            no_ambrosia_upgrades_enabled: false,
            lifetime_ambrosia: 0.0,
        });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn ambrosia_cube_mult_zeroed_by_exalt() {
        let result = calculate_ambrosia_cube_mult(&AmbrosiaMultInput {
            no_ambrosia_upgrades_enabled: true,
            lifetime_ambrosia: 1e9,
        });
        // Effective = 0 → mult = 1 + 0 = 1
        assert_eq!(result, 1.0);
    }

    #[test]
    fn ambrosia_cube_mult_first_tier_caps_at_1p5() {
        // eff/66/100 capped at 1.5 → need eff/66 >= 150 → eff >= 9900
        let result = calculate_ambrosia_cube_mult(&AmbrosiaMultInput {
            no_ambrosia_upgrades_enabled: false,
            lifetime_ambrosia: 9_999.0,
        });
        // 1 + min(1.5, floor(9999/66)/100) = 1 + min(1.5, 151/100) = 1 + 1.5 = 2.5
        assert!((result - 2.5).abs() < 1e-9);
    }

    #[test]
    fn ambrosia_quark_mult_third_tier_unlocks_at_500k() {
        let just_below = calculate_ambrosia_quark_mult(&AmbrosiaMultInput {
            no_ambrosia_upgrades_enabled: false,
            lifetime_ambrosia: 499_999.0,
        });
        let at = calculate_ambrosia_quark_mult(&AmbrosiaMultInput {
            no_ambrosia_upgrades_enabled: false,
            lifetime_ambrosia: 500_000.0,
        });
        // The third tier kicks in at 500k → at > just_below
        assert!(at > just_below);
    }

    #[test]
    fn generation_singularity_upgrade_multiplies() {
        let result = calculate_ambrosia_generation_singularity_upgrade(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(result, 24.0);
    }

    #[test]
    fn generation_singularity_upgrade_empty_is_one() {
        assert_eq!(calculate_ambrosia_generation_singularity_upgrade(&[]), 1.0);
    }

    #[test]
    fn luck_singularity_upgrade_sums() {
        let result = calculate_ambrosia_luck_singularity_upgrade(&[5.0, 10.0, 15.0, 20.0]);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn luck_octeract_upgrade_sums_empty_is_zero() {
        assert_eq!(calculate_ambrosia_luck_octeract_upgrade(&[]), 0.0);
    }
}
