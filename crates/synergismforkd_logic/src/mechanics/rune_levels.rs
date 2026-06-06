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

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::state::RunesState;

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

// ─── Manual buy ───────────────────────────────────────────────────────────

/// Inputs to [`buy_rune_levels`].
#[derive(Debug, Clone, Copy)]
pub struct BuyRuneLevelsInput {
    /// Rune index (`0..7`, via the `RUNE_*` constants). Out-of-range is a
    /// no-op.
    pub index: usize,
    /// `rune.costCoefficient` (UI-tier rune data table).
    pub cost_coefficient: Decimal,
    /// `getLevelsPerOOM(rune)` = `rune.levelsPerOOM + rune.levelsPerOOMIncrease()`
    /// — the combined slope, pre-evaluated by the caller (UI-tier).
    pub levels_per_oom: f64,
    /// `getRuneEXPPerOffering(rune)` = `universalRuneEXPMult(level)` — the
    /// per-offering EXP rate, pre-evaluated by the caller.
    pub rune_exp_per_offering: Decimal,
    /// Target levels to add this purchase — the `offeringbuyamount` toggle
    /// (1/10/100/…) or [`max_rune_level_purchase`]`.levels` for buy-max (a
    /// UI-tier decision). The `budget` caps how many actually land.
    pub levels_to_add: f64,
    /// Offerings the caller authorizes spending (the legacy `budget`; for a
    /// plain manual click this is the whole offerings balance — keep it
    /// `<= offerings`, the legacy quirk is to bank free EXP otherwise).
    pub budget: Decimal,
}

/// Buy rune levels for rune `index` by spending offerings — a port of the
/// legacy `sacrificeOfferings` core (`levelRune` + `updateLevelsFromEXP`).
/// Adds EXP toward `level + levels_to_add`; if `budget` can't reach that
/// target it banks partial EXP, and the level is re-derived from EXP (so a
/// 10-level request on a 3-level budget lands 3 levels). Spends from
/// `offerings`; emits [`CoreEvent::RuneLevelsPurchased`].
///
/// Faithful-at-current-state deferrals:
/// - **unlock**: `rune.isUnlocked()` reads achievements / researches
///   (UI-tier), so — like the octeract / GQ / blueberry gates — the caller
///   checks it; this buy is ungated on unlock;
/// - the buy-amount toggle and the buy-max planner
///   ([`max_rune_level_purchase`], already ported) live caller-side and feed
///   `levels_to_add`;
/// - rune EXP is held as `f64` in state (`RunesState::rune_exp`), so it is
///   widened to `Decimal` for the math and narrowed back on store.
#[must_use]
pub fn buy_rune_levels(
    runes: &mut RunesState,
    offerings: &mut Decimal,
    input: BuyRuneLevelsInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events = SmallVec::new();
    if input.index >= runes.rune_levels.len()
        || input.budget <= Decimal::zero()
        || input.levels_to_add < 1.0
    {
        return events;
    }

    let before = runes.rune_levels[input.index];
    let current_exp = Decimal::from_finite(runes.rune_exp[input.index]);
    let target_level = before + input.levels_to_add;

    // levelRune: spend toward `target_level`, banking partial EXP when the
    // budget falls short of the full jump.
    let exp_left = rune_exp_left_to_level(
        input.cost_coefficient,
        target_level,
        input.levels_per_oom,
        current_exp,
    );
    let offerings_required = (exp_left / input.rune_exp_per_offering)
        .ceil()
        .max(Decimal::one());

    let (new_exp, budget_used) = if offerings_required > input.budget {
        (
            current_exp + input.budget * input.rune_exp_per_offering,
            input.budget,
        )
    } else {
        (
            rune_exp_to_level(input.cost_coefficient, target_level, input.levels_per_oom),
            offerings_required,
        )
    };
    runes.rune_exp[input.index] = new_exp.to_number();
    *offerings -= budget_used;

    // updateLevelsFromEXP: re-derive the integer level, with the legacy
    // off-by-one fix-up when EXP sits exactly on a level boundary.
    let levels = rune_level_from_exp(new_exp, input.cost_coefficient, input.levels_per_oom);
    let exp_left_next = rune_exp_left_to_level(
        input.cost_coefficient,
        levels + 1.0,
        input.levels_per_oom,
        new_exp,
    );
    runes.rune_levels[input.index] = if exp_left_next == Decimal::zero() {
        levels + 1.0
    } else {
        levels
    };

    // Legacy clamp: never let offerings go negative.
    *offerings = (*offerings).max(Decimal::zero());

    events.push(CoreEvent::RuneLevelsPurchased {
        index: input.index as u32,
        before,
        after: runes.rune_levels[input.index],
        spent: budget_used,
    });
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{RUNE_COUNT, RUNE_SPEED};

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

    // ─── Manual buy ───────────────────────────────────────────────────────

    fn speed_buy_input(levels_to_add: f64, budget: f64) -> BuyRuneLevelsInput {
        BuyRuneLevelsInput {
            index: RUNE_SPEED,
            cost_coefficient: Decimal::from_finite(100.0),
            levels_per_oom: 5.0,
            rune_exp_per_offering: Decimal::from_finite(10.0),
            levels_to_add,
            budget: Decimal::from_finite(budget),
        }
    }

    #[test]
    fn buy_rune_levels_adds_levels_and_spends() {
        // coeff 100, lpo 5, perOffering 10: reaching level 5 needs 900 EXP =
        // 90 offerings; budget 1000 affords it in full.
        let mut runes = RunesState::default();
        let mut offerings = Decimal::from_finite(1000.0);
        let events = buy_rune_levels(&mut runes, &mut offerings, speed_buy_input(5.0, 1000.0));
        assert_eq!(runes.rune_levels[RUNE_SPEED], 5.0);
        assert!((runes.rune_exp[RUNE_SPEED] - 900.0).abs() < 1e-6);
        assert!((offerings.to_number() - 910.0).abs() < 1e-9);
        assert!(matches!(
            events.as_slice(),
            [CoreEvent::RuneLevelsPurchased { index: 0, .. }]
        ));
    }

    #[test]
    fn buy_rune_levels_partial_when_budget_short() {
        // Budget 30 (< 90 needed for level 5): banks 300 EXP → lands level 3,
        // spends the whole 30.
        let mut runes = RunesState::default();
        let mut offerings = Decimal::from_finite(30.0);
        let events = buy_rune_levels(&mut runes, &mut offerings, speed_buy_input(5.0, 30.0));
        assert_eq!(runes.rune_levels[RUNE_SPEED], 3.0);
        assert!((runes.rune_exp[RUNE_SPEED] - 300.0).abs() < 1e-6);
        assert_eq!(offerings.to_number(), 0.0);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_rune_levels_out_of_range_is_noop() {
        let mut runes = RunesState::default();
        let mut offerings = Decimal::from_finite(1000.0);
        let mut input = speed_buy_input(5.0, 1000.0);
        input.index = RUNE_COUNT;
        let events = buy_rune_levels(&mut runes, &mut offerings, input);
        assert!(events.is_empty());
        assert_eq!(offerings.to_number(), 1000.0);
    }

    #[test]
    fn buy_rune_levels_zero_budget_is_noop() {
        let mut runes = RunesState::default();
        let mut offerings = Decimal::from_finite(1000.0);
        let events = buy_rune_levels(&mut runes, &mut offerings, speed_buy_input(5.0, 0.0));
        assert!(events.is_empty());
        assert_eq!(runes.rune_levels[RUNE_SPEED], 0.0);
    }

    #[test]
    fn buy_rune_levels_below_one_level_is_noop() {
        let mut runes = RunesState::default();
        let mut offerings = Decimal::from_finite(1000.0);
        let events = buy_rune_levels(&mut runes, &mut offerings, speed_buy_input(0.0, 1000.0));
        assert!(events.is_empty());
    }
}
