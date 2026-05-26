//! EXP / level math + max-purchase planner for rune blessings and rune
//! spirits.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/runeUpgradeProgression.ts`.
//! The EXP↔level math is bit-identical between blessings and spirits —
//! only the `cost_coefficient` and `levels_per_oom` differ per rune — so
//! this module owns the closed-form invert plus the budget-aware
//! purchase planner once, and the legacy `RuneBlessings.ts` /
//! `RuneSpirits.ts` shim into it.
//!
//! Precedent: [`crate::mechanics::rune_levels`] does the same for
//! top-level runes. The shapes are similar but not identical:
//! - top-level runes have no per-call buy-amount cap (`upper_limit`)
//! - top-level runes have an `is_unlocked` gate; blessings/spirits gate
//!   at the UI layer before calling, so we don't expose it here
//! - blessings use a dynamic `min_offerings_floor` derived from the
//!   rune's current EXP (to avoid integer-precision loss at
//!   `MAX_SAFE_INTEGER` boundaries); spirits use a constant `1`. The
//!   caller passes the already-evaluated `Decimal` so this module
//!   stays pure-input.

use synergismforkd_bignum::Decimal;

/// EXP required to **reach** a target level, starting from 0 EXP.
/// Formula: `cost_coefficient * (10^(level / levels_per_oom) - 1)`.
#[must_use]
pub fn rune_upgrade_exp_to_level(
    cost_coefficient: Decimal,
    level: f64,
    levels_per_oom: f64,
) -> Decimal {
    cost_coefficient
        * (Decimal::from_finite(10.0).pow(Decimal::from_finite(level / levels_per_oom))
            - Decimal::one())
}

/// EXP still needed to reach a target level, given the rune-upgrade's
/// current EXP. Clamped at zero — querying a level you've already
/// passed returns `0`, not negative debt.
#[must_use]
pub fn rune_upgrade_exp_left_to_level(
    cost_coefficient: Decimal,
    target_level: f64,
    levels_per_oom: f64,
    current_rune_exp: Decimal,
) -> Decimal {
    (rune_upgrade_exp_to_level(cost_coefficient, target_level, levels_per_oom) - current_rune_exp)
        .max(Decimal::zero())
}

/// Result of [`rune_upgrade_level_from_exp`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuneUpgradeLevelFromExp {
    /// Integer level reached from EXP via the closed-form invert.
    pub levels: f64,
    /// `true` when `runeUpgradeEXPToLevel(coeff, levels + 1, oom) <=
    /// current_exp` — the floor undercounted by exactly one due to
    /// `log10` float imprecision, and the caller should use `levels + 1`.
    pub needs_float_bump: bool,
}

/// Closed-form inverse of [`rune_upgrade_exp_to_level`]: given current
/// EXP, returns the integer level reached, plus a `needs_float_bump`
/// flag.
///
/// Floating-point imprecision in `log10` can leave the floor one short
/// when current EXP exactly equals the EXP-for-`level + 1` boundary.
/// The flag encodes whether
/// `rune_upgrade_exp_to_level(coeff, levels + 1, oom) <= current_exp` —
/// when `true`, the caller should use `levels + 1` instead.
#[must_use]
pub fn rune_upgrade_level_from_exp(
    current_rune_exp: Decimal,
    cost_coefficient: Decimal,
    levels_per_oom: f64,
) -> RuneUpgradeLevelFromExp {
    let levels = (levels_per_oom
        * (current_rune_exp / cost_coefficient + Decimal::one())
            .log10()
            .to_number())
    .floor();
    let needs_float_bump =
        rune_upgrade_exp_to_level(cost_coefficient, levels + 1.0, levels_per_oom)
            <= current_rune_exp;
    RuneUpgradeLevelFromExp {
        levels,
        needs_float_bump,
    }
}

/// Inputs to [`max_rune_upgrade_purchase`].
#[derive(Debug, Clone, Copy)]
pub struct MaxRuneUpgradePurchaseInput {
    /// Per-upgrade cost coefficient.
    pub cost_coefficient: Decimal,
    /// Per-upgrade levels-per-OOM slope.
    pub levels_per_oom: f64,
    /// Current purchased level.
    pub current_level: f64,
    /// Current accumulated EXP.
    pub current_rune_exp: Decimal,
    /// EXP yielded per offering — caller pre-evaluates.
    pub rune_exp_per_offering: Decimal,
    /// Offerings budget the player wants to spend.
    pub budget: Decimal,
    /// Per-call cap on levels purchased (`player.runeBlessingBuyAmount`
    /// / `player.runeSpiritBuyAmount`). Unlike top-level runes,
    /// blessings and spirits respect a player-chosen "buy at most N
    /// levels" cap.
    pub upper_limit: f64,
    /// Minimum offerings to display when the budget can't afford even
    /// one level.
    /// - Blessings:
    ///   `ceil(current_rune_exp / (rune_exp_per_offering * MAX_SAFE_INTEGER))`
    ///   so tiny EXP increments are still representable.
    /// - Spirits: `Decimal::one()` (legacy behavior).
    ///
    /// The caller computes this fresh each call.
    pub min_offerings_floor: Decimal,
}

/// Result of [`max_rune_upgrade_purchase`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaxRuneUpgradePurchaseResult {
    /// Number of levels gained — `1` even when budget can't afford one,
    /// so the UI displays cost-to-next-level. `0` only on
    /// negative-budget short-circuit.
    pub levels: f64,
    /// Total EXP required to reach `current_level + levels` (or
    /// `current_level + 1` in the can't-afford-one fallback).
    pub exp_required: Decimal,
    /// Offerings actually required — floored at `min_offerings_floor`.
    pub offerings: Decimal,
}

/// Plans the largest affordable purchase capped by `upper_limit`.
/// Mirrors [`crate::mechanics::rune_levels::max_rune_level_purchase`]
/// but adds the per-call cap and the dynamic `min_offerings_floor`.
///
/// Algorithm: convert `budget` to total-available-EXP by adding
/// `budget * rune_exp_per_offering` to current EXP, invert the EXP→level
/// formula in closed form, floor it, subtract `current_level`, cap at
/// `upper_limit`. If the result is `0`, fall back to "cost of the very
/// next level" so the UI has something to display.
///
/// Returns `{levels: 0, exp_required: 0, offerings: 0}` for negative
/// budgets — matching the legacy null-case shape.
#[must_use]
pub fn max_rune_upgrade_purchase(
    input: MaxRuneUpgradePurchaseInput,
) -> MaxRuneUpgradePurchaseResult {
    if input.budget < Decimal::zero() {
        return MaxRuneUpgradePurchaseResult {
            levels: 0.0,
            exp_required: Decimal::zero(),
            offerings: Decimal::zero(),
        };
    }

    let total_exp_available = input.budget * input.rune_exp_per_offering + input.current_rune_exp;
    let max_level = (input.levels_per_oom
        * (total_exp_available / input.cost_coefficient + Decimal::one())
            .log10()
            .to_number())
    .floor();
    let levels_gained = (max_level - input.current_level)
        .max(0.0)
        .min(input.upper_limit);

    if levels_gained == 0.0 {
        let next_level_exp = rune_upgrade_exp_to_level(
            input.cost_coefficient,
            input.current_level + 1.0,
            input.levels_per_oom,
        );
        let offerings_required = ((next_level_exp - input.current_rune_exp)
            / input.rune_exp_per_offering)
            .ceil()
            .max(input.min_offerings_floor);
        return MaxRuneUpgradePurchaseResult {
            levels: 1.0,
            exp_required: next_level_exp,
            offerings: offerings_required,
        };
    }

    let exp_required = rune_upgrade_exp_to_level(
        input.cost_coefficient,
        input.current_level + levels_gained,
        input.levels_per_oom,
    );
    let offerings_required = ((exp_required - input.current_rune_exp)
        / input.rune_exp_per_offering)
        .ceil()
        .max(input.min_offerings_floor);
    MaxRuneUpgradePurchaseResult {
        levels: levels_gained,
        exp_required,
        offerings: offerings_required,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_to_level_matches_runes_formula() {
        // 100 * (10^1 - 1) = 900.
        let result = rune_upgrade_exp_to_level(Decimal::from_finite(100.0), 5.0, 5.0);
        assert!((result.to_number() - 900.0).abs() < 1e-9);
    }

    #[test]
    fn exp_left_to_level_clamps_at_zero() {
        let result = rune_upgrade_exp_left_to_level(
            Decimal::from_finite(100.0),
            5.0,
            5.0,
            Decimal::from_finite(10_000.0),
        );
        assert_eq!(result, Decimal::zero());
    }

    #[test]
    fn level_from_exp_no_bump_when_below_next_threshold() {
        let coeff = Decimal::from_finite(100.0);
        let oom = 5.0;
        // EXP for level 7 minus a small amount → should report 7 without bump.
        let exp = rune_upgrade_exp_to_level(coeff, 7.0, oom);
        let just_below = exp - Decimal::one();
        let result = rune_upgrade_level_from_exp(just_below, coeff, oom);
        assert!(result.levels >= 6.0);
        // needs_float_bump should be false since we're below the next
        // threshold by more than float imprecision.
        assert!(!result.needs_float_bump);
    }

    #[test]
    fn level_from_exp_flags_bump_at_exact_threshold() {
        let coeff = Decimal::from_finite(100.0);
        let oom = 5.0;
        // EXP for level 7 — log10 might give 6 due to float imprecision.
        let exp = rune_upgrade_exp_to_level(coeff, 7.0, oom);
        let result = rune_upgrade_level_from_exp(exp, coeff, oom);
        // Either levels == 7 (no bump needed) or levels == 6 (bump needed).
        if (result.levels - 6.0).abs() < 1e-9 {
            assert!(result.needs_float_bump);
        } else {
            assert!((result.levels - 7.0).abs() < 1e-9);
        }
    }

    fn baseline() -> MaxRuneUpgradePurchaseInput {
        MaxRuneUpgradePurchaseInput {
            cost_coefficient: Decimal::from_finite(100.0),
            levels_per_oom: 5.0,
            current_level: 0.0,
            current_rune_exp: Decimal::zero(),
            rune_exp_per_offering: Decimal::one(),
            budget: Decimal::from_finite(900.0),
            upper_limit: 100.0,
            min_offerings_floor: Decimal::one(),
        }
    }

    #[test]
    fn negative_budget_returns_zeroes() {
        let input = MaxRuneUpgradePurchaseInput {
            budget: Decimal::from_finite(-1.0),
            ..baseline()
        };
        let result = max_rune_upgrade_purchase(input);
        assert_eq!(result.levels, 0.0);
        assert_eq!(result.exp_required, Decimal::zero());
    }

    #[test]
    fn affordable_budget_reaches_target_level() {
        // Same setup as the rune_levels test: 900 budget → 5 levels.
        let result = max_rune_upgrade_purchase(baseline());
        assert_eq!(result.levels, 5.0);
    }

    #[test]
    fn upper_limit_caps_levels_gained() {
        let input = MaxRuneUpgradePurchaseInput {
            budget: Decimal::from_finite(900.0),
            upper_limit: 2.0,
            ..baseline()
        };
        let result = max_rune_upgrade_purchase(input);
        assert_eq!(result.levels, 2.0);
    }

    #[test]
    fn unaffordable_budget_returns_one_level_fallback() {
        let input = MaxRuneUpgradePurchaseInput {
            budget: Decimal::from_finite(1.0),
            ..baseline()
        };
        let result = max_rune_upgrade_purchase(input);
        assert_eq!(result.levels, 1.0);
        assert!(result.exp_required > Decimal::one());
    }

    #[test]
    fn min_offerings_floor_is_respected() {
        // Tiny budget — falls back to next-level, offerings would be 1
        // by the natural calc; floor of 5 should override.
        let input = MaxRuneUpgradePurchaseInput {
            budget: Decimal::from_finite(1.0),
            min_offerings_floor: Decimal::from_finite(5.0),
            ..baseline()
        };
        let result = max_rune_upgrade_purchase(input);
        assert!(result.offerings >= Decimal::from_finite(5.0));
    }
}
