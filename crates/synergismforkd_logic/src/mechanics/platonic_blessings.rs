//! Platonic-cube blessing effect formulas.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/cubes/platonicBlessings.ts`.
//! 8 pure multiplier-yield functions, one per [`PlatonicBlessings`]
//! field. Each follows the same shape — a soft cap (`limit`) above
//! which the field's effect transitions to a power-law with
//! diminishing-returns exponent `dr`. The constants vary per function
//! and are preserved verbatim from the legacy
//! `packages/web_ui/src/PlatonicCubes.ts`.

use crate::state::PlatonicBlessings;

#[must_use]
pub fn calculate_cube_multiplier_platonic_blessing(state: &PlatonicBlessings) -> f64 {
    let dr = 1.0 / 5.0;
    let effect_per_blessing = 2.0 / 4e6;
    let limit = 4e6;
    if state.cubes < limit {
        return 1.0 + effect_per_blessing * state.cubes;
    }
    let limit_mult = limit.powf(1.0 - dr);
    1.0 + effect_per_blessing * limit_mult * state.cubes.powf(dr)
}

#[must_use]
pub fn calculate_tesseract_multiplier_platonic_blessing(state: &PlatonicBlessings) -> f64 {
    let dr = 1.0 / 5.0;
    let effect_per_blessing = 1.5 / 4e6;
    let limit = 4e6;
    if state.tesseracts < limit {
        return 1.0 + effect_per_blessing * state.tesseracts;
    }
    let limit_mult = limit.powf(1.0 - dr);
    1.0 + effect_per_blessing * limit_mult * state.tesseracts.powf(dr)
}

#[must_use]
pub fn calculate_hypercube_multiplier_platonic_blessing(state: &PlatonicBlessings) -> f64 {
    let dr = 1.0 / 5.0;
    let effect_per_blessing = 1.0 / 4e6;
    let limit = 4e6;
    if state.hypercubes < limit {
        return 1.0 + effect_per_blessing * state.hypercubes;
    }
    let limit_mult = limit.powf(1.0 - dr);
    1.0 + effect_per_blessing * limit_mult * state.hypercubes.powf(dr)
}

#[must_use]
pub fn calculate_platonic_multiplier_platonic_blessing(state: &PlatonicBlessings) -> f64 {
    let dr = 1.0 / 5.0;
    let effect_per_blessing = 1.0 / 8e4;
    let limit = 8e4;
    if state.platonics < limit {
        return 1.0 + effect_per_blessing * state.platonics;
    }
    let limit_mult = limit.powf(1.0 - dr);
    1.0 + effect_per_blessing * limit_mult * state.platonics.powf(dr)
}

#[must_use]
pub fn calculate_hypercube_blessing_multiplier_platonic_blessing(state: &PlatonicBlessings) -> f64 {
    let dr = 1.0 / 16.0;
    let effect_per_blessing = 1.0 / 1e4;
    let limit = 1e4;
    if state.hypercube_bonus < limit {
        return 1.0 + effect_per_blessing * state.hypercube_bonus;
    }
    let limit_mult = limit.powf(1.0 - dr);
    1.0 + effect_per_blessing * limit_mult * state.hypercube_bonus.powf(dr)
}

/// Tax effect — bounded between 0 and 1, no hard limit branch.
#[must_use]
pub fn calculate_tax_platonic_blessing(state: &PlatonicBlessings) -> f64 {
    let factor = (1.0 + state.taxes).log10().powf(1.5);
    factor / (125.0 + factor)
}

/// Two-stage diminishing returns: a tighter `DR1` between `limit1` and
/// `limit2`, then an even tighter `DR2` above `limit2`. The `limit_mult`
/// products preserve continuity at the boundaries.
#[must_use]
pub fn calculate_ascension_score_platonic_blessing(state: &PlatonicBlessings) -> f64 {
    let dr1 = 1.0 / 4.0;
    let dr2 = 1.0 / 8.0;
    let limit1 = 1e4;
    let limit2 = 1e20;
    let effect_per_blessing = 1.0 / 1e4;
    if state.global_speed < limit1 {
        return 1.0 + effect_per_blessing * state.global_speed;
    } else if state.global_speed < limit2 {
        let limit_mult = limit1.powf(1.0 - dr1);
        return 1.0 + effect_per_blessing * limit_mult * state.global_speed.powf(dr1);
    }
    let limit_mult_1 = limit1.powf(1.0 - dr1);
    let limit_mult_2 = limit2.powf(dr1 - dr2);
    1.0 + effect_per_blessing * limit_mult_1 * limit_mult_2 * state.global_speed.powf(dr2)
}

#[must_use]
pub fn calculate_global_speed_platonic_blessing(state: &PlatonicBlessings) -> f64 {
    let dr = 1.0 / 8.0;
    let limit = 1e4;
    let effect_per_blessing = 1.0 / 1e4;
    if state.global_speed < limit {
        return 1.0 + effect_per_blessing * state.global_speed;
    }
    let limit_mult = limit.powf(1.0 - dr);
    1.0 + effect_per_blessing * limit_mult * state.global_speed.powf(dr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_platonic() -> PlatonicBlessings {
        PlatonicBlessings {
            cubes: 0.0,
            tesseracts: 0.0,
            hypercubes: 0.0,
            platonics: 0.0,
            hypercube_bonus: 0.0,
            taxes: 0.0,
            global_speed: 0.0,
        }
    }

    #[test]
    fn zero_state_yields_one_for_soft_cap_functions() {
        assert_eq!(
            calculate_cube_multiplier_platonic_blessing(&zero_platonic()),
            1.0
        );
        assert_eq!(
            calculate_tesseract_multiplier_platonic_blessing(&zero_platonic()),
            1.0
        );
        assert_eq!(
            calculate_hypercube_multiplier_platonic_blessing(&zero_platonic()),
            1.0
        );
        assert_eq!(
            calculate_platonic_multiplier_platonic_blessing(&zero_platonic()),
            1.0
        );
    }

    #[test]
    fn tax_at_zero_is_zero() {
        // factor = log10(1)^1.5 = 0 → 0 / (125 + 0) = 0
        let state = zero_platonic();
        assert_eq!(calculate_tax_platonic_blessing(&state), 0.0);
    }

    #[test]
    fn tax_bounded_below_one() {
        // For any finite state.taxes, the result is < 1 (asymptote).
        let state = PlatonicBlessings {
            taxes: 1e100,
            ..zero_platonic()
        };
        let result = calculate_tax_platonic_blessing(&state);
        assert!(result > 0.0);
        assert!(result < 1.0);
    }

    #[test]
    fn cube_blessing_above_limit_uses_dr() {
        // cubes = 1e7 (> 4e6), DR = 1/5
        // effect = 2/4e6, limit_mult = (4e6)^(4/5)
        let state = PlatonicBlessings {
            cubes: 1e7,
            ..zero_platonic()
        };
        let result = calculate_cube_multiplier_platonic_blessing(&state);
        let expected = 1.0 + (2.0 / 4e6) * 4e6_f64.powf(4.0 / 5.0) * 1e7_f64.powf(1.0 / 5.0);
        assert!((result - expected).abs() / expected < 1e-9);
    }

    #[test]
    fn ascension_score_three_tier_continuous() {
        // Check that the function is continuous at limit1 (1e4) and
        // limit2 (1e20).
        let just_below_l1 = PlatonicBlessings {
            global_speed: 9_999.999_999,
            ..zero_platonic()
        };
        let just_above_l1 = PlatonicBlessings {
            global_speed: 10_000.000_001,
            ..zero_platonic()
        };
        let below = calculate_ascension_score_platonic_blessing(&just_below_l1);
        let above = calculate_ascension_score_platonic_blessing(&just_above_l1);
        assert!(
            (below - above).abs() < 1e-3,
            "discontinuous at limit1: {below} vs {above}"
        );
    }
}
