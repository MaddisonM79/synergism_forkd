//! Red-ambrosia-derived bonuses.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/redAmbrosiaBonuses.ts`.
//! Four functions, each reads `player.lifetimeRedAmbrosia` and
//! combines it with a red-ambrosia-upgrade gate / exponent. The
//! cookie-29 luck function also reads `player.cubeUpgrades[79]` as
//! its second gate. The `getRedAmbrosiaUpgradeEffects(...)` lookups
//! stay in the UI tier — callers pass scalar inputs.

// ─── Cookie upgrade 29 luck ───────────────────────────────────────────────

/// Inputs to [`calculate_cookie_upgrade_29_luck`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateCookieUpgrade29LuckInput {
    /// `player.cubeUpgrades[79]` — gates the bonus. `0` → no bonus.
    pub cube_upgrade_79: f64,
    /// `player.lifetimeRedAmbrosia` — feeds the `log10` in the
    /// formula.
    pub lifetime_red_ambrosia: f64,
}

/// `10 × log10(lifetimeRedAmbrosia)^2` once both gates are non-zero,
/// else `0`.
#[must_use]
pub fn calculate_cookie_upgrade_29_luck(input: &CalculateCookieUpgrade29LuckInput) -> f64 {
    if input.cube_upgrade_79 == 0.0 || input.lifetime_red_ambrosia == 0.0 {
        return 0.0;
    }
    10.0 * input.lifetime_red_ambrosia.log10().powi(2)
}

// ─── Red ambrosia cube bonus ──────────────────────────────────────────────

/// Inputs to [`calculate_red_ambrosia_cubes`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateRedAmbrosiaCubesInput {
    /// Truthy when `getRedAmbrosiaUpgradeEffects('redAmbrosiaCube',
    /// 'unlockedRedAmbrosiaCube')` is set. Falsy → returns `1`.
    pub unlocked: bool,
    /// `player.lifetimeRedAmbrosia`.
    pub lifetime_red_ambrosia: f64,
    /// `getRedAmbrosiaUpgradeEffects('redAmbrosiaCubeImprover',
    /// 'extraExponent')` — added to the base `0.4` exponent.
    pub extra_exponent: f64,
}

/// `1 + lifetimeRedAmbrosia^(0.4 + extraExponent) / 100` once the
/// unlock is set; otherwise `1`.
#[must_use]
pub fn calculate_red_ambrosia_cubes(input: &CalculateRedAmbrosiaCubesInput) -> f64 {
    if !input.unlocked {
        return 1.0;
    }
    let exponent = 0.4 + input.extra_exponent;
    1.0 + input.lifetime_red_ambrosia.powf(exponent) / 100.0
}

// ─── Red ambrosia obtainium / offering bonus ──────────────────────────────

/// Inputs to [`calculate_red_ambrosia_obtainium`] and
/// [`calculate_red_ambrosia_offering`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateRedAmbrosiaResourceInput {
    /// Truthy when the corresponding red-ambrosia upgrade is
    /// unlocked. Falsy → returns `1`.
    pub unlocked: bool,
    /// `player.lifetimeRedAmbrosia`.
    pub lifetime_red_ambrosia: f64,
}

/// `1 + lifetimeRedAmbrosia^0.6 / 100` when unlocked; else `1`.
#[must_use]
pub fn calculate_red_ambrosia_obtainium(input: &CalculateRedAmbrosiaResourceInput) -> f64 {
    if !input.unlocked {
        return 1.0;
    }
    1.0 + input.lifetime_red_ambrosia.powf(0.6) / 100.0
}

/// Same formula as the obtainium bonus, gated on the offering
/// unlock instead.
#[must_use]
pub fn calculate_red_ambrosia_offering(input: &CalculateRedAmbrosiaResourceInput) -> f64 {
    if !input.unlocked {
        return 1.0;
    }
    1.0 + input.lifetime_red_ambrosia.powf(0.6) / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_29_luck_zero_when_gate_off() {
        let result = calculate_cookie_upgrade_29_luck(&CalculateCookieUpgrade29LuckInput {
            cube_upgrade_79: 0.0,
            lifetime_red_ambrosia: 1e6,
        });
        assert_eq!(result, 0.0);
    }

    #[test]
    fn cookie_29_luck_zero_when_no_ambrosia() {
        let result = calculate_cookie_upgrade_29_luck(&CalculateCookieUpgrade29LuckInput {
            cube_upgrade_79: 1.0,
            lifetime_red_ambrosia: 0.0,
        });
        assert_eq!(result, 0.0);
    }

    #[test]
    fn cookie_29_luck_log10_squared() {
        let result = calculate_cookie_upgrade_29_luck(&CalculateCookieUpgrade29LuckInput {
            cube_upgrade_79: 1.0,
            lifetime_red_ambrosia: 100.0,
        });
        // 10 * log10(100)^2 = 10 * 2^2 = 40
        assert!((result - 40.0).abs() < 1e-9);
    }

    #[test]
    fn red_ambrosia_cubes_locked_is_one() {
        let result = calculate_red_ambrosia_cubes(&CalculateRedAmbrosiaCubesInput {
            unlocked: false,
            lifetime_red_ambrosia: 1e6,
            extra_exponent: 0.0,
        });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn red_ambrosia_cubes_uses_exponent() {
        let result = calculate_red_ambrosia_cubes(&CalculateRedAmbrosiaCubesInput {
            unlocked: true,
            lifetime_red_ambrosia: 10_000.0,
            extra_exponent: 0.0,
        });
        // 1 + 10000^0.4 / 100 = 1 + 39.811... / 100 ≈ 1.398
        let expected = 1.0 + 10_000.0_f64.powf(0.4) / 100.0;
        assert!((result - expected).abs() < 1e-9);
    }

    #[test]
    fn red_ambrosia_obtainium_locked_is_one() {
        let result = calculate_red_ambrosia_obtainium(&CalculateRedAmbrosiaResourceInput {
            unlocked: false,
            lifetime_red_ambrosia: 1e6,
        });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn red_ambrosia_obtainium_uses_0p6_exponent() {
        let result = calculate_red_ambrosia_obtainium(&CalculateRedAmbrosiaResourceInput {
            unlocked: true,
            lifetime_red_ambrosia: 10_000.0,
        });
        let expected = 1.0 + 10_000.0_f64.powf(0.6) / 100.0;
        assert!((result - expected).abs() < 1e-9);
    }

    #[test]
    fn red_ambrosia_offering_matches_obtainium_formula() {
        let result_o = calculate_red_ambrosia_obtainium(&CalculateRedAmbrosiaResourceInput {
            unlocked: true,
            lifetime_red_ambrosia: 7_500.0,
        });
        let result_f = calculate_red_ambrosia_offering(&CalculateRedAmbrosiaResourceInput {
            unlocked: true,
            lifetime_red_ambrosia: 7_500.0,
        });
        assert_eq!(result_o, result_f);
    }
}
