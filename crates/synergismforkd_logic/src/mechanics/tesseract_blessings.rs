//! Tesseract blessing effect formulas.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/cubes/tesseractBlessings.ts`.
//! 10 pure multiplier-yield functions. Most follow the soft-cap+DR
//! shape (with the per-blessing-tier hypercube result playing the
//! `effect_per_blessing` numerator's role). Three outliers — `salvage`,
//! `ant_speed`, `ant_elo` — diverge from the shape (log curve with
//! hypercube cap; linear growth × hypercube; log × hypercube).
//!
//! Each function takes the precomputed hypercube-blessing value of the
//! same name as a second parameter; callers thread that through (the
//! tick orchestrator will do the composition once it lands).

use synergismforkd_bignum::Decimal;

use crate::state::BlessingValues;

/// Shared soft-cap+DR body used by 7 of the 10 functions. Limit fixed
/// at `1000`; only `DR` varies. `hypercube_blessing` is divided by 1000
/// to form `effect_per_blessing` — matches the original
/// `calculateXHypercubeBlessing() / 1000` expression in each legacy
/// callee.
fn soft_cap_dr(count: f64, dr: f64, hypercube_blessing: f64) -> f64 {
    let effect_per_blessing = hypercube_blessing / 1_000.0;
    let limit = 1_000.0;
    if count < limit {
        return 1.0 + effect_per_blessing * count;
    }
    let limit_mult = limit.powf(1.0 - dr);
    1.0 + effect_per_blessing * limit_mult * count.powf(dr)
}

#[must_use]
pub fn calculate_accelerator_tesseract_blessing(
    state: &BlessingValues,
    hypercube_blessing: f64,
) -> f64 {
    soft_cap_dr(state.accelerator, 1.0 / 6.0, hypercube_blessing)
}

#[must_use]
pub fn calculate_multiplier_tesseract_blessing(
    state: &BlessingValues,
    hypercube_blessing: f64,
) -> f64 {
    soft_cap_dr(state.multiplier, 1.0 / 6.0, hypercube_blessing)
}

#[must_use]
pub fn calculate_offering_tesseract_blessing(
    state: &BlessingValues,
    hypercube_blessing: f64,
) -> f64 {
    soft_cap_dr(state.offering, 1.0 / 3.0, hypercube_blessing)
}

#[must_use]
pub fn calculate_obtainium_tesseract_blessing(
    state: &BlessingValues,
    hypercube_blessing: f64,
) -> f64 {
    soft_cap_dr(state.obtainium, 1.0 / 3.0, hypercube_blessing)
}

#[must_use]
pub fn calculate_ant_sacrifice_tesseract_blessing(
    state: &BlessingValues,
    hypercube_blessing: f64,
) -> f64 {
    soft_cap_dr(state.ant_sacrifice, 1.0 / 6.0, hypercube_blessing)
}

#[must_use]
pub fn calculate_rune_effectiveness_tesseract_blessing(
    state: &BlessingValues,
    hypercube_blessing: f64,
) -> f64 {
    soft_cap_dr(state.talisman_bonus, 1.0 / 32.0, hypercube_blessing)
}

#[must_use]
pub fn calculate_global_speed_tesseract_blessing(
    state: &BlessingValues,
    hypercube_blessing: f64,
) -> f64 {
    soft_cap_dr(state.global_speed, 1.0 / 32.0, hypercube_blessing)
}

/// Outlier 1: log-based factor with hypercube as cap (no soft-cap
/// branch).
#[must_use]
pub fn calculate_salvage_tesseract_blessing(
    state: &BlessingValues,
    hypercube_salvage_blessing: f64,
) -> f64 {
    let factor = (state.rune_exp + 1.0).log10().powf(1.25);
    let cap = 0.5 * hypercube_salvage_blessing;
    1.0 + cap * factor / (20.0 + factor)
}

/// Outlier 2: linear growth multiplied by the hypercube blessing.
/// Returns `Decimal` (not `f64`) because the result feeds into Decimal
/// arithmetic downstream — matches the legacy signature.
#[must_use]
pub fn calculate_ant_speed_tesseract_blessing(
    state: &BlessingValues,
    hypercube_ant_speed_blessing: f64,
) -> Decimal {
    let effect_per_blessing = 1.0 / 1_000.0;
    Decimal::from_finite(1.0 + effect_per_blessing * state.ant_speed)
        * Decimal::from_finite(hypercube_ant_speed_blessing)
}

/// Outlier 3: log curve scaled by `hypercube / 100`.
#[must_use]
pub fn calculate_ant_elo_tesseract_blessing(
    state: &BlessingValues,
    hypercube_ant_elo_blessing: f64,
) -> f64 {
    1.0 + (state.ant_elo + 1.0).log10() * hypercube_ant_elo_blessing / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_blessings() -> BlessingValues {
        BlessingValues {
            accelerator: 0.0,
            multiplier: 0.0,
            offering: 0.0,
            rune_exp: 0.0,
            obtainium: 0.0,
            ant_speed: 0.0,
            ant_sacrifice: 0.0,
            ant_elo: 0.0,
            talisman_bonus: 0.0,
            global_speed: 0.0,
        }
    }

    #[test]
    fn soft_cap_dr_below_limit_is_linear() {
        let result = soft_cap_dr(500.0, 1.0 / 6.0, 1_000.0);
        // effect_per_blessing = 1; 1 + 500 * 1 = 501
        assert_eq!(result, 501.0);
    }

    #[test]
    fn soft_cap_dr_above_limit_uses_dr_branch() {
        // count = 2000, DR = 1/6, hypercube = 1000
        // effect_per_blessing = 1; limit = 1000; limit_mult = 1000^(5/6)
        // result = 1 + 1 * 1000^(5/6) * 2000^(1/6)
        let result = soft_cap_dr(2_000.0, 1.0 / 6.0, 1_000.0);
        let expected = 1.0 + 1_000.0_f64.powf(5.0 / 6.0) * 2_000.0_f64.powf(1.0 / 6.0);
        assert!((result - expected).abs() < 1e-9);
    }

    #[test]
    fn zero_blessings_yield_one() {
        assert_eq!(
            calculate_accelerator_tesseract_blessing(&zero_blessings(), 1.0),
            1.0
        );
        assert_eq!(
            calculate_multiplier_tesseract_blessing(&zero_blessings(), 1.0),
            1.0
        );
        assert_eq!(
            calculate_global_speed_tesseract_blessing(&zero_blessings(), 1.0),
            1.0
        );
    }

    #[test]
    fn salvage_zero_rune_exp_is_one() {
        // factor = log10(1)^1.25 = 0; result = 1 + 0 = 1
        let result = calculate_salvage_tesseract_blessing(&zero_blessings(), 1.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn ant_speed_tesseract_returns_decimal() {
        let state = BlessingValues {
            ant_speed: 1_000.0,
            ..zero_blessings()
        };
        let result = calculate_ant_speed_tesseract_blessing(&state, 5.0);
        // (1 + 1/1000 * 1000) * 5 = 2 * 5 = 10
        assert!((result.to_number() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn ant_elo_at_999_returns_one_plus_log10_3_over_100() {
        // log10(999 + 1) = 3 → 1 + 3 * 100 / 100 = 4
        let state = BlessingValues {
            ant_elo: 999.0,
            ..zero_blessings()
        };
        let result = calculate_ant_elo_tesseract_blessing(&state, 100.0);
        assert!((result - 4.0).abs() < 1e-9);
    }
}
