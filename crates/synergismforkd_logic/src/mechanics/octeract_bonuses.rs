//! Total-octeract bonuses gated by the `noOcteracts` (Exalt 4)
//! singularity challenge.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/octeractBonuses.ts`.
//! Offering & Obtainium bonuses are derived from the cube bonus, so
//! the caller precomputes that and passes it in.

// ─── Total octeract cube bonus ────────────────────────────────────────────

/// Inputs to [`calculate_total_octeract_cube_bonus`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateTotalOcteractCubeBonusInput {
    /// `player.singularityChallenges.noOcteracts.enabled` — Exalt 4
    /// gate.
    pub exalt_4_enabled: bool,
    /// `player.totalWowOcteracts`.
    pub total_wow_octeracts: f64,
    /// `getSingularityChallengeEffect('noOcteracts', 'octeractPow')`
    /// — additive exponent boost on the log10 branch. Base power is
    /// `2 + octeract_pow`.
    pub octeract_pow: f64,
}

/// Linear ramp from `1` to `3` across 0..1000 octeracts (with a
/// small-value threshold to avoid noise just above 1), then a log10
/// power curve above 1000: `3 × (log10(N) - 2)^(2 + octeractPow)`.
/// Returns `1` inside Exalt 4.
#[must_use]
pub fn calculate_total_octeract_cube_bonus(input: &CalculateTotalOcteractCubeBonusInput) -> f64 {
    if input.exalt_4_enabled {
        return 1.0;
    }
    if input.total_wow_octeracts < 1_000.0 {
        let bonus = 1.0 + (2.0 / 1_000.0) * input.total_wow_octeracts;
        return if bonus > 1.000_01 { bonus } else { 1.0 };
    }
    let power = 2.0 + input.octeract_pow;
    3.0 * (input.total_wow_octeracts.log10() - 2.0).powf(power)
}

// ─── Total octeract quark bonus ───────────────────────────────────────────

/// Inputs to [`calculate_total_octeract_quark_bonus`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateTotalOcteractQuarkBonusInput {
    /// `player.singularityChallenges.noOcteracts.enabled` — Exalt 4
    /// gate.
    pub exalt_4_enabled: bool,
    /// `player.totalWowOcteracts`.
    pub total_wow_octeracts: f64,
}

/// Linear ramp from `1` to `1.20` across 0..1000 octeracts (with
/// the same small-value tolerance as the cube bonus), then linear
/// in `log10(N) - 2` above 1000: `1.1 + 0.1 × (log10(N) - 2)`.
/// Returns `1` inside Exalt 4.
#[must_use]
pub fn calculate_total_octeract_quark_bonus(input: &CalculateTotalOcteractQuarkBonusInput) -> f64 {
    if input.exalt_4_enabled {
        return 1.0;
    }
    if input.total_wow_octeracts < 1_000.0 {
        let bonus = 1.0 + (0.2 / 1_000.0) * input.total_wow_octeracts;
        return if bonus > 1.000_01 { bonus } else { 1.0 };
    }
    1.1 + 0.1 * (input.total_wow_octeracts.log10() - 2.0)
}

// ─── Total octeract offering bonus ────────────────────────────────────────

/// Inputs to [`calculate_total_octeract_offering_bonus`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateTotalOcteractOfferingBonusInput {
    /// `getSingularityChallengeEffect('noOcteracts', 'offeringBonus')`
    /// — truthy once Exalt 4 has been completed enough to grant the
    /// offering bonus. Falsy → returns `1`.
    pub offering_bonus_enabled: bool,
    /// Precomputed cube bonus (caller invokes
    /// [`calculate_total_octeract_cube_bonus`] first). Raised to
    /// the 1.25 power.
    pub cube_bonus: f64,
}

/// `cube_bonus^1.25` once the offering reward of Exalt 4 has been
/// unlocked; otherwise `1`.
#[must_use]
pub fn calculate_total_octeract_offering_bonus(
    input: &CalculateTotalOcteractOfferingBonusInput,
) -> f64 {
    if !input.offering_bonus_enabled {
        return 1.0;
    }
    input.cube_bonus.powf(1.25)
}

// ─── Total octeract obtainium bonus ───────────────────────────────────────

/// Inputs to [`calculate_total_octeract_obtainium_bonus`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateTotalOcteractObtainiumBonusInput {
    /// `getSingularityChallengeEffect('noOcteracts', 'obtainiumBonus')`
    /// — truthy once Exalt 4 has been completed enough. Falsy →
    /// returns `1`.
    pub obtainium_bonus_enabled: bool,
    /// Precomputed cube bonus, raised to the 1.25 power.
    pub cube_bonus: f64,
}

/// `cube_bonus^1.25` once the obtainium reward of Exalt 4 has been
/// unlocked; otherwise `1`. Same formula as the offering bonus.
#[must_use]
pub fn calculate_total_octeract_obtainium_bonus(
    input: &CalculateTotalOcteractObtainiumBonusInput,
) -> f64 {
    if !input.obtainium_bonus_enabled {
        return 1.0;
    }
    input.cube_bonus.powf(1.25)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_bonus_in_exalt_4_is_one() {
        let result = calculate_total_octeract_cube_bonus(&CalculateTotalOcteractCubeBonusInput {
            exalt_4_enabled: true,
            total_wow_octeracts: 1e9,
            octeract_pow: 0.5,
        });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn cube_bonus_linear_below_1000() {
        // 500 → 1 + 2*500/1000 = 2
        let result = calculate_total_octeract_cube_bonus(&CalculateTotalOcteractCubeBonusInput {
            exalt_4_enabled: false,
            total_wow_octeracts: 500.0,
            octeract_pow: 0.0,
        });
        assert_eq!(result, 2.0);
    }

    #[test]
    fn cube_bonus_log_branch_at_1000() {
        // 1000 → 3 * (3 - 2)^2 = 3 * 1 = 3
        let result = calculate_total_octeract_cube_bonus(&CalculateTotalOcteractCubeBonusInput {
            exalt_4_enabled: false,
            total_wow_octeracts: 1_000.0,
            octeract_pow: 0.0,
        });
        assert_eq!(result, 3.0);
    }

    #[test]
    fn cube_bonus_uses_octeract_pow() {
        // 10000 → 3 * (4 - 2)^(2 + 1) = 3 * 8 = 24
        let result = calculate_total_octeract_cube_bonus(&CalculateTotalOcteractCubeBonusInput {
            exalt_4_enabled: false,
            total_wow_octeracts: 10_000.0,
            octeract_pow: 1.0,
        });
        assert_eq!(result, 24.0);
    }

    #[test]
    fn quark_bonus_in_exalt_4_is_one() {
        let result = calculate_total_octeract_quark_bonus(&CalculateTotalOcteractQuarkBonusInput {
            exalt_4_enabled: true,
            total_wow_octeracts: 1e9,
        });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn quark_bonus_log_branch() {
        // 1000 → 1.1 + 0.1*(3 - 2) = 1.2
        let result = calculate_total_octeract_quark_bonus(&CalculateTotalOcteractQuarkBonusInput {
            exalt_4_enabled: false,
            total_wow_octeracts: 1_000.0,
        });
        assert!((result - 1.2).abs() < 1e-12);
    }

    #[test]
    fn offering_bonus_when_locked_returns_one() {
        let result =
            calculate_total_octeract_offering_bonus(&CalculateTotalOcteractOfferingBonusInput {
                offering_bonus_enabled: false,
                cube_bonus: 100.0,
            });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn offering_bonus_uses_cube_pow_1p25() {
        let result =
            calculate_total_octeract_offering_bonus(&CalculateTotalOcteractOfferingBonusInput {
                offering_bonus_enabled: true,
                cube_bonus: 100.0,
            });
        let expected = 100.0_f64.powf(1.25);
        assert!((result - expected).abs() < 1e-9);
    }

    #[test]
    fn obtainium_bonus_matches_offering_formula() {
        let off_in = CalculateTotalOcteractOfferingBonusInput {
            offering_bonus_enabled: true,
            cube_bonus: 50.0,
        };
        let obt_in = CalculateTotalOcteractObtainiumBonusInput {
            obtainium_bonus_enabled: true,
            cube_bonus: 50.0,
        };
        assert_eq!(
            calculate_total_octeract_offering_bonus(&off_in),
            calculate_total_octeract_obtainium_bonus(&obt_in)
        );
    }
}
