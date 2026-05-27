//! Ant-sacrifice reward calculators (offering + obtainium +
//! immortal-ELO gain + taxman-last-stand clamp).
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/antSacrificeRewardCalc.ts`.
//! The Statistics-aggregator reductions stay in the UI tier; logic
//! owns the per-call arithmetic once the caller has those
//! reductions in hand.

use synergismforkd_bignum::Decimal;

// ─── Immortal-ELO gain ────────────────────────────────────────────────────

/// Inputs to [`calculate_immortal_elo_gain`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateImmortalELOGainInput {
    /// Result of `calculateEffectiveAntELO`.
    pub effective_elo: f64,
    /// `player.ants.immortalELO`.
    pub immortal_elo: f64,
}

/// Floor-clamped delta `max(0, effective_elo - immortal_elo)`.
#[must_use]
pub fn calculate_immortal_elo_gain(input: &CalculateImmortalELOGainInput) -> f64 {
    (input.effective_elo - input.immortal_elo).max(0.0)
}

// ─── Taxman-last-stand clamp ──────────────────────────────────────────────

/// Inputs to [`apply_taxman_last_stand_clamp`].
#[derive(Debug, Clone, Copy)]
pub struct ApplyTaxmanLastStandClampInput {
    /// Pre-clamp final reward.
    pub final_reward: Decimal,
    /// Currently-held resource amount (offerings / obtainium etc.).
    pub current_resource: Decimal,
    /// Whether the `taxmanLastStand` challenge is enabled.
    pub taxman_last_stand_enabled: bool,
    /// Completions count for `taxmanLastStand`; the clamp engages
    /// at `>= 2`.
    pub taxman_last_stand_completions: f64,
}

/// Caps `final_reward` at `current_resource * 100 + 1` when the
/// `taxmanLastStand` challenge is at `>= 2` completions. Otherwise
/// passes through unchanged.
#[must_use]
pub fn apply_taxman_last_stand_clamp(input: &ApplyTaxmanLastStandClampInput) -> Decimal {
    if input.taxman_last_stand_enabled && input.taxman_last_stand_completions >= 2.0 {
        return (input.current_resource * Decimal::from_finite(100.0) + Decimal::one())
            .min(input.final_reward);
    }
    input.final_reward
}

// ─── Ant-sacrifice offering / obtainium ───────────────────────────────────

/// Inputs to [`calculate_ant_sacrifice_offering`].
#[derive(Debug, Clone, Copy)]
pub struct AntSacrificeOfferingInput {
    /// `calculateAntSacrificeMultiplier()`.
    pub ant_sac_mult: Decimal,
    /// Reborn-ELO stage modifier `antSacrificeOfferingMult`.
    pub stage_mult: f64,
    /// Reduce-of `offeringObtainiumTimeModifiers`.
    pub time_multiplier: f64,
    /// `calculateOfferings(false)` — without the time-mult
    /// double-application.
    pub offering_mult: Decimal,
    /// `player.offerings` — current balance, for the taxman clamp.
    pub current_offerings: Decimal,
    /// `player.singularityChallenges.taxmanLastStand.enabled`.
    pub taxman_last_stand_enabled: bool,
    /// `player.singularityChallenges.taxmanLastStand.completions`.
    pub taxman_last_stand_completions: f64,
}

/// Per-sacrifice offering reward:
///
/// ```text
/// offering_mult × (1 × ant_sac_mult × stage_mult × time_multiplier)
/// ```
///
/// then clamp by `current_offerings × 100 + 1` when
/// `taxmanLastStand ≥ 2`.
#[must_use]
pub fn calculate_ant_sacrifice_offering(input: &AntSacrificeOfferingInput) -> Decimal {
    let overall_sacrifice_multiplier = input.ant_sac_mult
        * Decimal::from_finite(input.stage_mult)
        * Decimal::from_finite(input.time_multiplier);
    let final_offerings = input.offering_mult * overall_sacrifice_multiplier;
    apply_taxman_last_stand_clamp(&ApplyTaxmanLastStandClampInput {
        final_reward: final_offerings,
        current_resource: input.current_offerings,
        taxman_last_stand_enabled: input.taxman_last_stand_enabled,
        taxman_last_stand_completions: input.taxman_last_stand_completions,
    })
}

/// Inputs to [`calculate_ant_sacrifice_obtainium`].
#[derive(Debug, Clone, Copy)]
pub struct AntSacrificeObtainiumInput {
    /// `calculateAntSacrificeMultiplier()`.
    pub ant_sac_mult: Decimal,
    /// Reborn-ELO stage modifier `antSacrificeObtainiumMult`.
    pub stage_mult: f64,
    /// Reduce-of `offeringObtainiumTimeModifiers`.
    pub time_multiplier: f64,
    /// `calculateObtainium(false)` — without the time-mult
    /// double-application.
    pub obtainium_mult: Decimal,
    /// `player.obtainium` — current balance, for the taxman clamp.
    pub current_obtainium: Decimal,
    /// `player.singularityChallenges.taxmanLastStand.enabled`.
    pub taxman_last_stand_enabled: bool,
    /// `player.singularityChallenges.taxmanLastStand.completions`.
    pub taxman_last_stand_completions: f64,
}

/// Mirrors [`calculate_ant_sacrifice_offering`] for obtainium.
#[must_use]
pub fn calculate_ant_sacrifice_obtainium(input: &AntSacrificeObtainiumInput) -> Decimal {
    let overall_sacrifice_multiplier = input.ant_sac_mult
        * Decimal::from_finite(input.stage_mult)
        * Decimal::from_finite(input.time_multiplier);
    let final_obtainium = input.obtainium_mult * overall_sacrifice_multiplier;
    apply_taxman_last_stand_clamp(&ApplyTaxmanLastStandClampInput {
        final_reward: final_obtainium,
        current_resource: input.current_obtainium,
        taxman_last_stand_enabled: input.taxman_last_stand_enabled,
        taxman_last_stand_completions: input.taxman_last_stand_completions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn immortal_elo_gain_floors_at_zero() {
        let result = calculate_immortal_elo_gain(&CalculateImmortalELOGainInput {
            effective_elo: 100.0,
            immortal_elo: 200.0,
        });
        assert_eq!(result, 0.0);
    }

    #[test]
    fn immortal_elo_gain_returns_delta() {
        let result = calculate_immortal_elo_gain(&CalculateImmortalELOGainInput {
            effective_elo: 200.0,
            immortal_elo: 50.0,
        });
        assert_eq!(result, 150.0);
    }

    #[test]
    fn taxman_clamp_disabled_passes_through() {
        let result = apply_taxman_last_stand_clamp(&ApplyTaxmanLastStandClampInput {
            final_reward: Decimal::from_finite(1e10),
            current_resource: Decimal::from_finite(1.0),
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 5.0,
        });
        assert_eq!(result.to_number(), 1e10);
    }

    #[test]
    fn taxman_clamp_engaged_at_2_completions() {
        // current = 10, clamp = 10*100+1 = 1001; reward = 1e10 → clamp to 1001
        let result = apply_taxman_last_stand_clamp(&ApplyTaxmanLastStandClampInput {
            final_reward: Decimal::from_finite(1e10),
            current_resource: Decimal::from_finite(10.0),
            taxman_last_stand_enabled: true,
            taxman_last_stand_completions: 2.0,
        });
        assert_eq!(result.to_number(), 1001.0);
    }

    #[test]
    fn ant_sacrifice_offering_combines_multipliers() {
        let result = calculate_ant_sacrifice_offering(&AntSacrificeOfferingInput {
            ant_sac_mult: Decimal::from_finite(2.0),
            stage_mult: 1.5,
            time_multiplier: 3.0,
            offering_mult: Decimal::from_finite(10.0),
            current_offerings: Decimal::zero(),
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 0.0,
        });
        // 10 * 2 * 1.5 * 3 = 90
        assert_eq!(result.to_number(), 90.0);
    }
}
