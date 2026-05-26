//! Hypercube blessing effect formulas.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/cubes/hypercubeBlessings.ts`.
//! 10 pure multiplier-yield functions. Eight of them follow the
//! soft-cap+DR shape and additionally scale `effect_per_blessing` by an
//! amplifier sourced from the platonic-blessings layer
//! ([`crate::mechanics::platonic_blessings::calculate_hypercube_blessing_multiplier_platonic_blessing`]).
//! Callers precompute that amplifier and pass it as the second arg.
//!
//! The two outliers — `salvage` and `ant_elo` — are amplifier-free
//! logarithms.

use crate::state::BlessingValues;

/// Shared soft-cap+DR body used by 8 of the 10 functions. Limit is
/// fixed at `1000` across all of them; only the `DR` varies.
fn soft_cap_dr(count: f64, dr: f64, platonic_amplifier: f64) -> f64 {
    let effect_per_blessing = platonic_amplifier / 1_000.0;
    let limit = 1_000.0;
    if count < limit {
        return 1.0 + effect_per_blessing * count;
    }
    let limit_mult = limit.powf(1.0 - dr);
    1.0 + effect_per_blessing * limit_mult * count.powf(dr)
}

#[must_use]
pub fn calculate_accelerator_hypercube_blessing(
    state: &BlessingValues,
    platonic_amplifier: f64,
) -> f64 {
    soft_cap_dr(state.accelerator, 1.0 / 12.0, platonic_amplifier)
}

#[must_use]
pub fn calculate_multiplier_hypercube_blessing(
    state: &BlessingValues,
    platonic_amplifier: f64,
) -> f64 {
    soft_cap_dr(state.multiplier, 1.0 / 12.0, platonic_amplifier)
}

#[must_use]
pub fn calculate_offering_hypercube_blessing(
    state: &BlessingValues,
    platonic_amplifier: f64,
) -> f64 {
    soft_cap_dr(state.offering, 1.0 / 6.0, platonic_amplifier)
}

#[must_use]
pub fn calculate_obtainium_hypercube_blessing(
    state: &BlessingValues,
    platonic_amplifier: f64,
) -> f64 {
    soft_cap_dr(state.obtainium, 1.0 / 6.0, platonic_amplifier)
}

#[must_use]
pub fn calculate_ant_speed_hypercube_blessing(
    state: &BlessingValues,
    platonic_amplifier: f64,
) -> f64 {
    soft_cap_dr(state.ant_speed, 1.0 / 2.0, platonic_amplifier)
}

#[must_use]
pub fn calculate_ant_sacrifice_hypercube_blessing(
    state: &BlessingValues,
    platonic_amplifier: f64,
) -> f64 {
    soft_cap_dr(state.ant_sacrifice, 1.0 / 12.0, platonic_amplifier)
}

#[must_use]
pub fn calculate_rune_effectiveness_hypercube_blessing(
    state: &BlessingValues,
    platonic_amplifier: f64,
) -> f64 {
    soft_cap_dr(state.talisman_bonus, 1.0 / 64.0, platonic_amplifier)
}

#[must_use]
pub fn calculate_global_speed_hypercube_blessing(
    state: &BlessingValues,
    platonic_amplifier: f64,
) -> f64 {
    soft_cap_dr(state.global_speed, 1.0 / 64.0, platonic_amplifier)
}

/// `salvage` doesn't take the platonic amplifier — it's an
/// amplifier-free log-scale curve.
#[must_use]
pub fn calculate_salvage_hypercube_blessing(state: &BlessingValues) -> f64 {
    let factor = (state.rune_exp + 1.0).log10().powf(1.25);
    let cap = 3.0 / 2.0;
    1.0 + cap * factor / (40.0 + factor)
}

/// `ant_elo` doesn't take the platonic amplifier — log10 of
/// `ant_elo + 1` divided by 25.
#[must_use]
pub fn calculate_ant_elo_hypercube_blessing(state: &BlessingValues) -> f64 {
    1.0 + (state.ant_elo + 1.0).log10() / 25.0
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
    fn zero_blessings_yield_one() {
        assert_eq!(
            calculate_accelerator_hypercube_blessing(&zero_blessings(), 1.0),
            1.0
        );
        assert_eq!(calculate_salvage_hypercube_blessing(&zero_blessings()), 1.0);
        assert_eq!(calculate_ant_elo_hypercube_blessing(&zero_blessings()), 1.0);
    }

    #[test]
    fn accelerator_softer_dr_than_tesseract() {
        // hypercube uses DR=1/12 vs tesseract's 1/6 — at the same input
        // hypercube grows more slowly past the limit.
        let state = BlessingValues {
            accelerator: 2_000.0,
            ..zero_blessings()
        };
        let result = calculate_accelerator_hypercube_blessing(&state, 1_000.0);
        // effect_per_blessing = 1; limit_mult = 1000^(11/12); count^(1/12)
        let expected = 1.0 + 1_000.0_f64.powf(11.0 / 12.0) * 2_000.0_f64.powf(1.0 / 12.0);
        assert!((result - expected).abs() < 1e-9);
    }

    #[test]
    fn salvage_with_high_rune_exp_approaches_cap() {
        // factor large → approaches cap of 3/2 → 1 + 1.5 ≈ 2.5
        let state = BlessingValues {
            rune_exp: 1e30,
            ..zero_blessings()
        };
        let result = calculate_salvage_hypercube_blessing(&state);
        // factor = log10(1e30+1)^1.25 ≈ 30^1.25 ≈ 70.18; 1.5 * 70 / 110 ≈ 0.96
        // result ≈ 1.96
        assert!(result > 1.5);
        assert!(result < 2.5);
    }

    #[test]
    fn ant_elo_at_999_is_one_plus_3_over_25() {
        let state = BlessingValues {
            ant_elo: 999.0,
            ..zero_blessings()
        };
        let result = calculate_ant_elo_hypercube_blessing(&state);
        assert!((result - (1.0 + 3.0 / 25.0)).abs() < 1e-9);
    }
}
