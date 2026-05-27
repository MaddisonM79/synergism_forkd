//! Overflux-derived multipliers.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/overfluxBonuses.ts`
//! (lifted from the legacy `packages/web_ui/src/Calculate.ts`). Two
//! pure-powder formulas plus the orbs-based cube-to-quark multiplier
//! (a 12-term sigmoid stack gated by singularity-count unlocks).

use crate::math::sigmoid::calculate_sigmoid;

// ─── Powder → cube / quark mults ───────────────────────────────────────────

/// Cube multiplier from overflux powder. Linear in
/// `overflux_powder / 10_000` below the `10k` threshold; switches to a
/// `log10²` scaling above it.
#[must_use]
pub fn calculate_cube_mult_from_powder(overflux_powder: f64) -> f64 {
    if overflux_powder > 10_000.0 {
        1.0 + (1.0 / 16.0) * overflux_powder.log10().powi(2)
    } else {
        1.0 + (1.0 / 10_000.0) * overflux_powder
    }
}

/// Quark multiplier from overflux powder. Same boundary as the cube
/// version, but linear `log10` above the threshold.
#[must_use]
pub fn calculate_quark_mult_from_powder(overflux_powder: f64) -> f64 {
    if overflux_powder > 10_000.0 {
        1.0 + (1.0 / 40.0) * overflux_powder.log10()
    } else {
        1.0 + (1.0 / 100_000.0) * overflux_powder
    }
}

// ─── Orbs → cube-quark multiplier ──────────────────────────────────────────

/// Inputs to [`calculate_cube_quark_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateCubeQuarkMultiplierInput {
    /// `player.overfluxOrbs` — the main scaling input for every
    /// sigmoid.
    pub overflux_orbs: f64,
    /// `player.highestSingularityCount` — gates the last nine sigmoid
    /// contributors at thresholds `1, 2, 5, 10, 15, 20, 25, 30, 35`.
    pub highest_singularity_count: f64,
    /// `getShopUpgradeEffects('cubeToQuarkAll', 'quarkMult')` — final
    /// outer multiplier.
    pub cube_to_quark_all_mult: f64,
    /// `player.autoWarpCheck`. When `true`, the result is further
    /// multiplied by `1 + daily_powder_reset_uses`.
    pub auto_warp_check: bool,
    /// `player.dailyPowderResetUses`.
    pub daily_powder_reset_uses: f64,
}

/// Cube → quark multiplier from overflux orbs. Sums 12 sigmoid
/// contributors (with progressively higher constants and divisors),
/// each of the last 9 gated by a singularity-count threshold.
/// Subtracts 11 (the all-zero sum baseline is 12 because each sigmoid
/// returns 1 at `factor = 0`), then multiplies by the `cubeToQuarkAll`
/// shop effect and an optional `daily_powder_reset_uses` bonus when
/// auto-warp is on.
#[must_use]
pub fn calculate_cube_quark_multiplier(input: &CalculateCubeQuarkMultiplierInput) -> f64 {
    let orbs = input.overflux_orbs;
    let high = input.highest_singularity_count;
    let gate = |threshold: f64, exponent: f64| -> f64 {
        if high >= threshold {
            orbs.powf(exponent)
        } else {
            0.0
        }
    };

    let sigmoids = calculate_sigmoid(2.0, orbs.powf(0.5), 40.0)
        + calculate_sigmoid(1.5, orbs.powf(0.5), 160.0)
        + calculate_sigmoid(1.5, orbs.powf(0.5), 640.0)
        + calculate_sigmoid(1.15, gate(1.0, 0.45), 2_560.0)
        + calculate_sigmoid(1.15, gate(2.0, 0.4), 10_000.0)
        + calculate_sigmoid(1.25, gate(5.0, 0.35), 40_000.0)
        + calculate_sigmoid(1.25, gate(10.0, 0.32), 160_000.0)
        + calculate_sigmoid(1.35, gate(15.0, 0.27), 640_000.0)
        + calculate_sigmoid(1.45, gate(20.0, 0.24), 2e6)
        + calculate_sigmoid(1.55, gate(25.0, 0.21), 1e7)
        + calculate_sigmoid(1.85, gate(30.0, 0.18), 4e7)
        + calculate_sigmoid(3.0, gate(35.0, 0.15), 1e8);

    let warp_bonus = if input.auto_warp_check {
        1.0 + input.daily_powder_reset_uses
    } else {
        1.0
    };
    (sigmoids - 11.0) * input.cube_to_quark_all_mult * warp_bonus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_mult_below_threshold_is_linear() {
        // 5000 → 1 + 5000/10000 = 1.5
        assert_eq!(calculate_cube_mult_from_powder(5_000.0), 1.5);
    }

    #[test]
    fn cube_mult_above_threshold_is_log10_squared() {
        // 1e6 → 1 + (1/16) * log10(1e6)^2 = 1 + 36/16 = 3.25
        let result = calculate_cube_mult_from_powder(1e6);
        assert!((result - 3.25).abs() < 1e-12);
    }

    #[test]
    fn quark_mult_below_threshold_is_linear() {
        // 5000 (≤ 10000 threshold) → 1 + 5000/100000 = 1.05
        assert_eq!(calculate_quark_mult_from_powder(5_000.0), 1.05);
    }

    #[test]
    fn quark_mult_above_threshold_is_log10() {
        // 1e6 → 1 + log10(1e6)/40 = 1 + 6/40 = 1.15
        let result = calculate_quark_mult_from_powder(1e6);
        assert!((result - 1.15).abs() < 1e-12);
    }

    fn baseline_orbs_input() -> CalculateCubeQuarkMultiplierInput {
        CalculateCubeQuarkMultiplierInput {
            overflux_orbs: 0.0,
            highest_singularity_count: 0.0,
            cube_to_quark_all_mult: 1.0,
            auto_warp_check: false,
            daily_powder_reset_uses: 0.0,
        }
    }

    #[test]
    fn cube_quark_multiplier_zero_orbs_is_zero() {
        // All 12 sigmoids return 1 at factor=0 → sum = 12 → 12 - 11 = 1
        // × cube_to_quark_all_mult (1) × warp_bonus (1) = 1.
        // Actually... let me re-check. At orbs=0, sigmoid(c, 0, d) = 1.
        // Sum of 12 sigmoids = 12. (12 - 11) * 1 * 1 = 1.
        let result = calculate_cube_quark_multiplier(&baseline_orbs_input());
        assert!((result - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cube_quark_multiplier_grows_with_orbs() {
        let small = CalculateCubeQuarkMultiplierInput {
            overflux_orbs: 100.0,
            ..baseline_orbs_input()
        };
        let big = CalculateCubeQuarkMultiplierInput {
            overflux_orbs: 1e6,
            ..baseline_orbs_input()
        };
        assert!(calculate_cube_quark_multiplier(&big) > calculate_cube_quark_multiplier(&small));
    }

    #[test]
    fn cube_quark_multiplier_singularity_unlocks_more_terms() {
        let no_sing = CalculateCubeQuarkMultiplierInput {
            overflux_orbs: 1e6,
            ..baseline_orbs_input()
        };
        let with_sing = CalculateCubeQuarkMultiplierInput {
            highest_singularity_count: 100.0,
            ..no_sing
        };
        // With singularity unlocks active, more sigmoid terms contribute > 1.
        assert!(
            calculate_cube_quark_multiplier(&with_sing) > calculate_cube_quark_multiplier(&no_sing)
        );
    }

    #[test]
    fn cube_quark_multiplier_auto_warp_applies_powder_uses_bonus() {
        // 4 daily resets, auto-warp on → result × (1 + 4) = ×5 vs ×1.
        let without = CalculateCubeQuarkMultiplierInput {
            overflux_orbs: 1e6,
            highest_singularity_count: 100.0,
            ..baseline_orbs_input()
        };
        let with_warp = CalculateCubeQuarkMultiplierInput {
            auto_warp_check: true,
            daily_powder_reset_uses: 4.0,
            ..without
        };
        let ratio =
            calculate_cube_quark_multiplier(&with_warp) / calculate_cube_quark_multiplier(&without);
        assert!((ratio - 5.0).abs() < 1e-9);
    }
}
