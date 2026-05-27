//! Cube (wow-cube) blessing effect formulas.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/cubes/cubeBlessings.ts`.
//! 10 pure multiplier-yield functions, one per [`BlessingValues`] field.
//! Each composes the corresponding tesseract-blessing value (as
//! `effect_per_blessing`'s numerator or a direct multiplier) with a
//! per-function `cube_upgrade` level that contributes a `DR_INCREASE`
//! term.
//!
//! Compared to the platonic / hypercube / tesseract layers, cube
//! blessings have:
//! - A per-function `DR_INCREASE = cube_upgrade[N] / K` term that adds
//!   to **both** the limit-mult exponent and the count exponent.
//! - Several `Decimal` returns (`ant_speed`, `ant_sacrifice`) where
//!   downstream code needs to multiply big numbers without
//!   overflowing.
//! - `offering` / `obtainium` use Decimal arithmetic with `.to_number()`
//!   at the end because the multiplication can briefly exceed `1e308`.

use synergismforkd_bignum::Decimal;

use crate::state::BlessingValues;

#[must_use]
pub fn calculate_accelerator_cube_blessing(
    state: &BlessingValues,
    tesseract_blessing: f64,
    cube_upgrade_level: f64,
) -> f64 {
    let dr = 1.0 / 3.0;
    let effect_per_blessing = tesseract_blessing / 500.0;
    let limit = 1_000.0;
    let dr_increase = cube_upgrade_level / 300.0;
    if state.accelerator < limit {
        return (effect_per_blessing * state.accelerator).powf(1.0 + dr_increase);
    }
    let limit_mult = limit.powf(1.0 - dr + dr_increase);
    effect_per_blessing * limit_mult * state.accelerator.powf(dr + dr_increase)
}

#[must_use]
pub fn calculate_multiplier_cube_blessing(
    state: &BlessingValues,
    tesseract_blessing: f64,
    cube_upgrade_level: f64,
) -> f64 {
    let dr = 1.0 / 3.0;
    let effect_per_blessing = tesseract_blessing / 5_000.0;
    let limit = 1_000.0;
    let dr_increase = cube_upgrade_level / 300.0;
    if state.multiplier < limit {
        return (1.0 + effect_per_blessing * state.multiplier).powf(1.0 + dr_increase);
    }
    let limit_mult = limit.powf(1.0 - dr + dr_increase);
    1.0 + effect_per_blessing * limit_mult * state.multiplier.powf(dr + dr_increase)
}

#[must_use]
pub fn calculate_offering_cube_blessing(
    state: &BlessingValues,
    tesseract_blessing: f64,
    cube_upgrade_level: f64,
) -> f64 {
    let dr = 2.0 / 3.0;
    // Decimal arithmetic to avoid intermediate overflow past 1e308 — the
    // final .to_number() clamps via Decimal.min(1e300, …).
    let effect_per_blessing =
        Decimal::from_finite(tesseract_blessing) / Decimal::from_finite(2_000.0);
    let limit = 1_000.0;
    let dr_increase = cube_upgrade_level * 2.0 / 300.0;
    let cap = Decimal::from_finite(1e300);
    if state.offering < limit {
        return cap
            .min(
                (effect_per_blessing * Decimal::from_finite(state.offering) + Decimal::one())
                    .pow(Decimal::from_finite(1.0 + dr_increase)),
            )
            .to_number();
    }
    let limit_mult = Decimal::from_finite(limit).pow(Decimal::from_finite(1.0 - dr + dr_increase));
    cap.min(
        limit_mult
            * effect_per_blessing
            * Decimal::from_finite(state.offering.powf(dr + dr_increase))
            + Decimal::one(),
    )
    .to_number()
}

#[must_use]
pub fn calculate_salvage_cube_blessing(
    state: &BlessingValues,
    tesseract_blessing: f64,
    cube_upgrade_level: f64,
) -> f64 {
    let limit = 1_000.0;
    let effect_multiplier = (1.0 + cube_upgrade_level / 100.0) * tesseract_blessing;
    if state.rune_exp < limit {
        return effect_multiplier * (state.rune_exp * 10.0 / limit);
    }
    let limit_bonus = 10.0;
    effect_multiplier * (limit_bonus + 10.0 * (state.rune_exp / limit).log10())
}

#[must_use]
pub fn calculate_obtainium_cube_blessing(
    state: &BlessingValues,
    tesseract_blessing: f64,
    cube_upgrade_level: f64,
) -> f64 {
    let dr = 2.0 / 3.0;
    let effect_per_blessing =
        Decimal::from_finite(tesseract_blessing) / Decimal::from_finite(2_000.0);
    let limit = 1_000.0;
    let dr_increase = cube_upgrade_level * 2.0 / 300.0;
    let cap = Decimal::from_finite(1e300);
    if state.obtainium < limit {
        return cap
            .min(
                (effect_per_blessing * Decimal::from_finite(state.obtainium) + Decimal::one())
                    .pow(Decimal::from_finite(1.0 + dr_increase)),
            )
            .to_number();
    }
    let limit_mult = Decimal::from_finite(limit).pow(Decimal::from_finite(1.0 - dr + dr_increase));
    cap.min(
        limit_mult
            * effect_per_blessing
            * Decimal::from_finite(state.obtainium.powf(dr + dr_increase))
            + Decimal::one(),
    )
    .to_number()
}

/// `ant_speed` blessing — returns `Decimal` because the multiplication
/// chain crosses the f64 ceiling at late game. The upstream
/// [`crate::mechanics::tesseract_blessings::calculate_ant_speed_tesseract_blessing`]
/// returns a `Decimal` that we pass through here.
///
/// `first_bonus = 0.1` when `ant_speed >= 1`, else `0.1 * ant_speed`
/// (linear ramp from 0 to 0.1 across the first blessing).
#[must_use]
pub fn calculate_ant_speed_cube_blessing(
    state: &BlessingValues,
    tesseract_blessing: Decimal,
    cube_upgrade_level: f64,
) -> Decimal {
    let effect_per_blessing = 1.0 / 1_000.0;
    let exponent_increase = cube_upgrade_level / 40.0;
    let first_bonus = 0.1 * state.ant_speed.min(1.0);
    Decimal::from_finite(1.0 + effect_per_blessing * state.ant_speed + first_bonus)
        .pow(Decimal::from_finite(2.0 + exponent_increase))
        * tesseract_blessing
}

#[must_use]
pub fn calculate_ant_sacrifice_cube_blessing(
    state: &BlessingValues,
    tesseract_blessing: f64,
    cube_upgrade_level: f64,
) -> Decimal {
    let dr = 2.0 / 3.0;
    let effect_per_blessing = tesseract_blessing / 5_000.0;
    let limit = 1_000.0;
    let dr_increase = cube_upgrade_level / 50.0;
    if state.ant_sacrifice < limit {
        return Decimal::from_finite(1.0 + effect_per_blessing * state.ant_sacrifice)
            .pow(Decimal::from_finite(1.0 + dr_increase));
    }
    let limit_mult = limit.powf(1.0 - dr + dr_increase);
    Decimal::from_finite(state.ant_sacrifice).pow(Decimal::from_finite(dr + dr_increase))
        * Decimal::from_finite(effect_per_blessing)
        * Decimal::from_finite(limit_mult)
        + Decimal::one()
}

#[must_use]
pub fn calculate_ant_elo_cube_blessing(
    state: &BlessingValues,
    tesseract_blessing: f64,
    cube_upgrade_level: f64,
) -> f64 {
    let effect_exponent = 1.0 + cube_upgrade_level / 100.0;
    (1.0 + 0.1 * (1.0 + state.ant_elo).log10() * tesseract_blessing).powf(effect_exponent)
}

#[must_use]
pub fn calculate_rune_effectiveness_cube_blessing(
    state: &BlessingValues,
    tesseract_blessing: f64,
    cube_upgrade_level: f64,
) -> f64 {
    let dr = 1.0 / 16.0;
    let effect_per_blessing = tesseract_blessing / 10_000.0;
    let limit = 1_000.0;
    let dr_increase = cube_upgrade_level / 1_600.0;
    if state.talisman_bonus < limit {
        return (1.0 + effect_per_blessing * state.talisman_bonus).powf(1.0 + dr_increase);
    }
    let limit_mult = limit.powf(1.0 - dr + dr_increase);
    1e300_f64
        .min(1.0 + limit_mult * effect_per_blessing * state.talisman_bonus.powf(dr + dr_increase))
}

#[must_use]
pub fn calculate_global_speed_cube_blessing(
    state: &BlessingValues,
    tesseract_blessing: f64,
    cube_upgrade_level: f64,
) -> f64 {
    let dr = 1.0 / 16.0;
    let effect_per_blessing = tesseract_blessing / 1_000.0;
    let limit = 1_000.0;
    let dr_increase = cube_upgrade_level / 1_600.0;
    if state.global_speed < limit {
        return (1.0 + effect_per_blessing * state.global_speed).powf(1.0 + dr_increase);
    }
    let limit_mult = limit.powf(1.0 - dr + dr_increase);
    1e300_f64
        .min(1.0 + limit_mult * effect_per_blessing * state.global_speed.powf(dr + dr_increase))
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
    fn accelerator_zero_state_is_zero() {
        // (0 * x)^anything = 0
        let result = calculate_accelerator_cube_blessing(&zero_blessings(), 1.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn multiplier_zero_state_is_one() {
        // (1 + 0)^1 = 1
        let result = calculate_multiplier_cube_blessing(&zero_blessings(), 1.0, 0.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn offering_zero_state_is_one() {
        // (1 + 0)^1 = 1
        let result = calculate_offering_cube_blessing(&zero_blessings(), 1.0, 0.0);
        assert!((result - 1.0).abs() < 1e-12);
    }

    #[test]
    fn salvage_zero_state_is_zero() {
        let result = calculate_salvage_cube_blessing(&zero_blessings(), 1.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn ant_speed_zero_state_is_one_times_tesseract() {
        // (1 + 0 + 0)^2 * tesseract_blessing = tesseract_blessing
        let result =
            calculate_ant_speed_cube_blessing(&zero_blessings(), Decimal::from_finite(5.0), 0.0);
        assert!((result.to_number() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn ant_sacrifice_zero_state_is_one() {
        let result = calculate_ant_sacrifice_cube_blessing(&zero_blessings(), 1.0, 0.0);
        assert!((result.to_number() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn ant_elo_zero_state_is_one() {
        // (1 + 0.1 * log10(1) * t)^1 = 1
        let result = calculate_ant_elo_cube_blessing(&zero_blessings(), 1.0, 0.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn rune_effectiveness_zero_state_is_one() {
        let result = calculate_rune_effectiveness_cube_blessing(&zero_blessings(), 1.0, 0.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn global_speed_zero_state_is_one() {
        let result = calculate_global_speed_cube_blessing(&zero_blessings(), 1.0, 0.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn accelerator_below_limit_uses_dr_increase_exponent() {
        // accelerator = 500, tesseract = 1, cube_upgrade = 0
        // effectPerBlessing = 1/500 = 0.002; 0.002 * 500 = 1
        // 1^(1 + 0) = 1
        let state = BlessingValues {
            accelerator: 500.0,
            ..zero_blessings()
        };
        let result = calculate_accelerator_cube_blessing(&state, 1.0, 0.0);
        assert!((result - 1.0).abs() < 1e-9);
    }

    #[test]
    fn accelerator_above_limit_uses_dr_branch() {
        // accelerator = 2000, tesseract = 500, cube_upgrade = 0
        // effectPerBlessing = 500/500 = 1
        // DR = 1/3, limit = 1000, limitMult = 1000^(1 - 1/3) = 1000^(2/3) = 100
        // result = 1 * 100 * 2000^(1/3) = 100 * 12.599 ≈ 1259.92
        let state = BlessingValues {
            accelerator: 2_000.0,
            ..zero_blessings()
        };
        let result = calculate_accelerator_cube_blessing(&state, 500.0, 0.0);
        let expected = 100.0 * 2_000.0_f64.powf(1.0 / 3.0);
        assert!((result - expected).abs() / expected < 1e-9);
    }
}
