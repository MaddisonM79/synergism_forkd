//! Rune EXP / level math for top-level runes.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/runeLevels.ts` (lifted
//! from the legacy `packages/web_ui/src/Runes.ts`). Each function takes
//! a small per-rune snapshot (`cost_coefficient`, `levels_per_oom`,
//! sometimes `current_exp` + `level`) and is pure `Decimal` math. The
//! `web_ui` side owns the rune data table and the offering-spend flow;
//! this module owns the closed-form EXP↔level inversion plus the
//! "given a budget, how many levels can I buy" planner.

use synergismforkd_bignum::Decimal;

/// EXP required to **reach** a target rune level, starting from 0 EXP.
/// Formula: `cost_coefficient * (10^(level / levels_per_oom) - 1)`.
///
/// The `-1` zeroes out at level 0. `levels_per_oom` is the number of
/// levels between each `10×` EXP step.
#[must_use]
pub fn rune_exp_to_level(cost_coefficient: Decimal, level: f64, levels_per_oom: f64) -> Decimal {
    cost_coefficient
        * (Decimal::from_finite(10.0).pow(Decimal::from_finite(level / levels_per_oom))
            - Decimal::one())
}

/// EXP still needed to **reach** a target rune level, given the rune's
/// current EXP. Clamped at zero — querying a level you've already passed
/// returns `0`, not negative debt.
#[must_use]
pub fn rune_exp_left_to_level(
    cost_coefficient: Decimal,
    target_level: f64,
    levels_per_oom: f64,
    current_rune_exp: Decimal,
) -> Decimal {
    (rune_exp_to_level(cost_coefficient, target_level, levels_per_oom) - current_rune_exp)
        .max(Decimal::zero())
}

/// Offerings required to reach a target rune level given the per-offering
/// EXP rate. Floored at `1` — the UI never displays "0 offerings to next
/// level" for an unowned level even if floating-point imprecision would
/// say so.
#[must_use]
pub fn rune_offerings_to_level(
    cost_coefficient: Decimal,
    target_level: f64,
    levels_per_oom: f64,
    current_rune_exp: Decimal,
    rune_exp_per_offering: Decimal,
) -> Decimal {
    (rune_exp_left_to_level(
        cost_coefficient,
        target_level,
        levels_per_oom,
        current_rune_exp,
    ) / rune_exp_per_offering)
        .ceil()
        .max(Decimal::one())
}

/// Closed-form inverse of [`rune_exp_to_level`]: given a rune's current
/// EXP, returns the integer level reached. Equivalent to
/// `floor(levels_per_oom * log10(EXP / cost_coeff + 1))`.
///
/// Used by the rune-EXP→level resync (after gaining EXP). Does **not**
/// include the float-imprecision `+1` bump that the legacy
/// `updateLevelsFromEXP` does afterward — that fix-up uses
/// [`rune_exp_left_to_level`] to detect when the floor undercounted by
/// exactly one.
#[must_use]
pub fn rune_level_from_exp(
    current_rune_exp: Decimal,
    cost_coefficient: Decimal,
    levels_per_oom: f64,
) -> f64 {
    (levels_per_oom
        * (current_rune_exp / cost_coefficient + Decimal::one())
            .log10()
            .to_number())
    .floor()
}

/// Inputs to [`max_rune_level_purchase`]. Mirrors `MaxRuneLevelPurchaseInput`.
#[derive(Debug, Clone, Copy)]
pub struct MaxRuneLevelPurchaseInput {
    /// `rune.costCoefficient`.
    pub cost_coefficient: Decimal,
    /// `rune.levelsPerOOM + rune.levelsPerOOMIncrease()` — combined
    /// slope.
    pub levels_per_oom: f64,
    /// `rune.level` — current purchased level (not the floored-from-EXP
    /// level).
    pub current_level: f64,
    /// `rune.runeEXP` — current accumulated EXP.
    pub current_rune_exp: Decimal,
    /// `rune.runeEXPPerOffering(currentLevel)` — already evaluated by
    /// the caller.
    pub rune_exp_per_offering: Decimal,
    /// Offerings budget the player wants to spend.
    pub budget: Decimal,
    /// `rune.isUnlocked()` — returning `{0, 0, 0}` below an unlocked
    /// rune is the legacy behavior the caller relies on for UI display
    /// gating.
    pub is_unlocked: bool,
}

/// Result of [`max_rune_level_purchase`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaxRuneLevelPurchaseResult {
    /// Number of levels gained — `0` when locked or budget is negative.
    pub levels: f64,
    /// Total EXP required to reach `current_level + levels`.
    pub exp_required: Decimal,
    /// Offerings actually consumed (`≥1` if any level affordable).
    pub offerings: Decimal,
}

/// Plans the largest level purchase affordable with a given offerings
/// budget.
///
/// Algorithm: convert `budget` to "total available EXP" by adding
/// `budget * rune_exp_per_offering` to `current_rune_exp`. Invert the
/// EXP→level formula in closed form to find the maximum level reachable,
/// floor it, subtract `current_level`. If that's `0`, fall back to
/// "what's the cost of the very next level" so the UI has something to
/// display.
///
/// Returns `{levels: 0, exp_required: 0, offerings: 0}` for locked runes
/// / negative budgets — matching the legacy null-case shape.
#[must_use]
pub fn max_rune_level_purchase(input: MaxRuneLevelPurchaseInput) -> MaxRuneLevelPurchaseResult {
    if !input.is_unlocked || input.budget < Decimal::zero() {
        return MaxRuneLevelPurchaseResult {
            levels: 0.0,
            exp_required: Decimal::zero(),
            offerings: Decimal::zero(),
        };
    }

    let total_exp_available = input.budget * input.rune_exp_per_offering + input.current_rune_exp;
    // Same closed-form invert as rune_level_from_exp, but with the
    // budget-augmented EXP rather than just the current EXP.
    let max_level = (input.levels_per_oom
        * (total_exp_available / input.cost_coefficient + Decimal::one())
            .log10()
            .to_number())
    .floor();
    let levels_gained = (max_level - input.current_level).max(0.0);

    if levels_gained == 0.0 {
        // Budget too small to buy a level; report cost-to-next-level so
        // the UI can display the gap.
        let next_level_exp = rune_exp_to_level(
            input.cost_coefficient,
            input.current_level + 1.0,
            input.levels_per_oom,
        );
        let offerings_required = ((next_level_exp - input.current_rune_exp)
            / input.rune_exp_per_offering)
            .ceil()
            .max(Decimal::one());
        return MaxRuneLevelPurchaseResult {
            levels: 1.0,
            exp_required: next_level_exp,
            offerings: offerings_required,
        };
    }

    let exp_required = rune_exp_to_level(
        input.cost_coefficient,
        input.current_level + levels_gained,
        input.levels_per_oom,
    );
    // Recompute offerings — the planner may have undershot the budget if
    // the last level didn't quite fit.
    let offerings_required = ((exp_required - input.current_rune_exp)
        / input.rune_exp_per_offering)
        .ceil()
        .max(Decimal::one());
    MaxRuneLevelPurchaseResult {
        levels: levels_gained,
        exp_required,
        offerings: offerings_required,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rune_exp_to_level_zero_at_zero() {
        let result = rune_exp_to_level(Decimal::from_finite(100.0), 0.0, 5.0);
        assert_eq!(result, Decimal::zero());
    }

    #[test]
    fn rune_exp_to_level_one_oom_at_levels_per_oom() {
        // At level == levels_per_oom: 10^1 - 1 = 9. Cost coeff = 100 → 900.
        let result = rune_exp_to_level(Decimal::from_finite(100.0), 5.0, 5.0);
        assert!((result.to_number() - 900.0).abs() < 1e-9);
    }

    #[test]
    fn rune_exp_to_level_two_oom_at_2x_levels_per_oom() {
        // At level == 2 * levels_per_oom: 10^2 - 1 = 99. Cost coeff = 100 → 9900.
        let result = rune_exp_to_level(Decimal::from_finite(100.0), 10.0, 5.0);
        assert!((result.to_number() - 9900.0).abs() < 1e-9);
    }

    #[test]
    fn rune_exp_left_to_level_clamps_at_zero() {
        // Current EXP exceeds what's needed → 0.
        let result = rune_exp_left_to_level(
            Decimal::from_finite(100.0),
            5.0,
            5.0,
            Decimal::from_finite(10_000.0),
        );
        assert_eq!(result, Decimal::zero());
    }

    #[test]
    fn rune_offerings_to_level_floors_at_one() {
        // 0 EXP needed (already at target) → floored at 1.
        let result = rune_offerings_to_level(
            Decimal::from_finite(100.0),
            5.0,
            5.0,
            Decimal::from_finite(10_000.0),
            Decimal::from_finite(10.0),
        );
        assert_eq!(result, Decimal::one());
    }

    #[test]
    fn rune_level_from_exp_inverts_exp_to_level() {
        let coeff = Decimal::from_finite(100.0);
        let oom = 5.0;
        // EXP for level 7 (between 5 and 10)
        let exp = rune_exp_to_level(coeff, 7.0, oom);
        let level = rune_level_from_exp(exp, coeff, oom);
        // Should recover 7 (or 7-1 due to float imprecision — the
        // legacy code documents this with the float-bump fix-up).
        assert!((level - 7.0).abs() <= 1.0);
    }

    #[test]
    fn max_rune_level_purchase_locked_returns_zeroes() {
        let input = MaxRuneLevelPurchaseInput {
            cost_coefficient: Decimal::from_finite(100.0),
            levels_per_oom: 5.0,
            current_level: 0.0,
            current_rune_exp: Decimal::zero(),
            rune_exp_per_offering: Decimal::one(),
            budget: Decimal::from_finite(1e10),
            is_unlocked: false,
        };
        let result = max_rune_level_purchase(input);
        assert_eq!(result.levels, 0.0);
        assert_eq!(result.exp_required, Decimal::zero());
        assert_eq!(result.offerings, Decimal::zero());
    }

    #[test]
    fn max_rune_level_purchase_negative_budget_returns_zeroes() {
        let input = MaxRuneLevelPurchaseInput {
            cost_coefficient: Decimal::from_finite(100.0),
            levels_per_oom: 5.0,
            current_level: 0.0,
            current_rune_exp: Decimal::zero(),
            rune_exp_per_offering: Decimal::one(),
            budget: Decimal::from_finite(-1.0),
            is_unlocked: true,
        };
        let result = max_rune_level_purchase(input);
        assert_eq!(result.levels, 0.0);
    }

    #[test]
    fn max_rune_level_purchase_returns_levels_for_affordable_budget() {
        // Cost coeff 100, levels_per_oom 5. To reach level 5: 100 * 9 = 900 EXP.
        // Budget = 900 offerings @ 1 EXP each → exactly affordable.
        let input = MaxRuneLevelPurchaseInput {
            cost_coefficient: Decimal::from_finite(100.0),
            levels_per_oom: 5.0,
            current_level: 0.0,
            current_rune_exp: Decimal::zero(),
            rune_exp_per_offering: Decimal::one(),
            budget: Decimal::from_finite(900.0),
            is_unlocked: true,
        };
        let result = max_rune_level_purchase(input);
        assert_eq!(result.levels, 5.0);
        assert!((result.exp_required.to_number() - 900.0).abs() < 1e-6);
    }

    #[test]
    fn max_rune_level_purchase_falls_back_to_next_level_when_unaffordable() {
        // Tiny budget — can't even afford level 1. Should report
        // `levels: 1` with cost-to-next-level for UI display.
        let input = MaxRuneLevelPurchaseInput {
            cost_coefficient: Decimal::from_finite(100.0),
            levels_per_oom: 5.0,
            current_level: 0.0,
            current_rune_exp: Decimal::zero(),
            rune_exp_per_offering: Decimal::one(),
            budget: Decimal::from_finite(1.0),
            is_unlocked: true,
        };
        let result = max_rune_level_purchase(input);
        assert_eq!(result.levels, 1.0);
        assert!(result.exp_required > Decimal::one());
    }
}
