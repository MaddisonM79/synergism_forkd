//! Summation / cost helpers.
//!
//! Verbatim port of `legacy_core_split/packages/logic/src/math/summations.ts`.
//! Generic algebraic primitives used by the upgrade-cost code in the legacy
//! `web_ui`.
//!
//! Error path: the TS version threw `Error('SUMMATIONS_QUADRATIC_IMPROPER')`
//! etc. Logic isn't allowed to call i18next, so the TS version threw plain
//! `Error` with machine-readable codes; this port returns
//! [`Result<_, SummationError>`] with one variant per code. The guards fire
//! only on invalid inputs (programmer error), so in practice they're
//! dev-facing.

use core::fmt;

/// Errors raised by summation / cost helpers. Mirrors the string codes
/// thrown by the legacy TS implementation 1:1 for parity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SummationError {
    /// `solve_quadratic` was called with `a < 0`.
    QuadraticImproper,
    /// `solve_quadratic` was called with a negative discriminant.
    QuadraticDeterminant,
    /// `calculate_cubic_sum_data` was called with `total_to_spend < 0`.
    CubicSumNegative,
}

impl fmt::Display for SummationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            SummationError::QuadraticImproper => "SUMMATIONS_QUADRATIC_IMPROPER",
            SummationError::QuadraticDeterminant => "SUMMATIONS_QUADRATIC_DETERMINANT",
            SummationError::CubicSumNegative => "SUMMATIONS_CUBIC_SUM_NEGATIVE",
        };
        f.write_str(code)
    }
}

impl std::error::Error for SummationError {}

// ─── Linear non-linear-cost summation (cost = base * (1 + level * d)) ──────

/// Result of [`calculate_summation_non_linear`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalculateSummationNonLinearResult {
    /// Highest level the player can reach with the resource available
    /// (capped at `base_level + buy_amount`).
    pub level_can_buy: f64,
    /// Total cost spent to reach `level_can_buy` from `base_level`.
    pub cost: f64,
}

/// If costs grow as `base * (1 + level * diff_per_level)` from `base_level`,
/// compute how many levels can be bought with `resource_available` (capped
/// at `base_level + buy_amount`) and the total cost spent.
///
/// `NaN` values for `resource_available` are normalized to 0 to match the
/// `resourceAvailable || 0` falsy-coalesce in the TS source.
pub fn calculate_summation_non_linear(
    base_level: f64,
    base_cost: f64,
    resource_available: f64,
    diff_per_level: f64,
    buy_amount: f64,
) -> CalculateSummationNonLinearResult {
    let c = diff_per_level / 2.0;
    let mut resource_available = if resource_available.is_nan() {
        0.0
    } else {
        resource_available
    };
    let already_spent = base_cost * (c * base_level.powi(2) + base_level * (1.0 - c));
    resource_available += already_spent;
    let v = resource_available / base_cost;
    let mut buy_to_level = if c > 0.0 {
        ((c - 1.0) / (2.0 * c) + ((1.0 - c).powi(2) + 4.0 * c * v).powf(0.5) / (2.0 * c))
            .floor()
            .max(0.0)
    } else {
        v.floor()
    };

    buy_to_level = buy_to_level.min(buy_amount + base_level);
    buy_to_level = buy_to_level.max(base_level);
    let mut total_cost =
        base_cost * (c * buy_to_level.powi(2) + buy_to_level * (1.0 - c)) - already_spent;
    if buy_to_level == base_level {
        total_cost = base_cost * (1.0 + 2.0 * c * base_level);
    }
    CalculateSummationNonLinearResult {
        level_can_buy: buy_to_level,
        cost: total_cost,
    }
}

// ─── Cubic summation + quadratic solver ────────────────────────────────────

/// Sum of the first `n` positive cubes — closed form `(n(n+1)/2)^2`.
/// Returns 0 if `n == 0`, or -1 if `n` is negative / non-integer (matches
/// the validation behavior of the original `web_ui` helper).
pub fn calculate_summation_cubic(n: f64) -> f64 {
    if n < 0.0 {
        return -1.0;
    }
    if n.fract() != 0.0 || !n.is_finite() {
        return -1.0;
    }
    ((n * (n + 1.0)) / 2.0).powi(2)
}

/// Real-root solver for `a * n^2 + b * n + c = 0`. `a` must be nonneg and
/// the discriminant must be non-negative; otherwise returns
/// [`SummationError`].
///
/// `positive` selects which root to return: `true` →
/// `(-b + sqrt(disc)) / (2a)`, `false` → `(-b - sqrt(disc)) / (2a)`. When the
/// discriminant is 0 both forms collapse to `-b / (2a)`.
///
/// # Errors
///
/// - [`SummationError::QuadraticImproper`] when `a < 0` (the solver only
///   handles upward-opening parabolas).
/// - [`SummationError::QuadraticDeterminant`] when `b² - 4ac < 0` (no
///   real roots).
pub fn solve_quadratic(a: f64, b: f64, c: f64, positive: bool) -> Result<f64, SummationError> {
    if a < 0.0 {
        return Err(SummationError::QuadraticImproper);
    }
    let determinant = b.powi(2) - 4.0 * a * c;
    if determinant < 0.0 {
        return Err(SummationError::QuadraticDeterminant);
    }

    if determinant == 0.0 {
        return Ok(-b / (2.0 * a));
    }
    let root = determinant.sqrt();
    Ok(if positive {
        (-b + root) / (2.0 * a)
    } else {
        (-b - root) / (2.0 * a)
    })
}

/// Result of [`calculate_cubic_sum_data`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalculateCubicSumDataResult {
    /// Highest level the player can reach (capped at `max_level`).
    pub level_can_buy: f64,
    /// Actual amount that would be spent reaching `level_can_buy`.
    pub cost: f64,
}

/// Cubic-cost upgrade-batch solver: if level `i` costs `base_cost * (i+1)^3`,
/// compute how many levels can be bought given `initial_level` already owned
/// and `amount_to_spend` more available, capped at `max_level`. Returns the
/// `{level_can_buy, cost}` pair where `cost` is the actual amount spent.
///
/// # Errors
///
/// - [`SummationError::CubicSumNegative`] when `already_spent +
///   amount_to_spend < 0` (programmer-error guard).
/// - Any [`SummationError`] propagated from [`solve_quadratic`] used
///   internally (in practice only `QuadraticDeterminant` for pathological
///   inputs since `a = 1.0` always).
pub fn calculate_cubic_sum_data(
    initial_level: f64,
    base_cost: f64,
    amount_to_spend: f64,
    max_level: f64,
) -> Result<CalculateCubicSumDataResult, SummationError> {
    if initial_level >= max_level {
        return Ok(CalculateCubicSumDataResult {
            level_can_buy: max_level,
            cost: 0.0,
        });
    }
    let already_spent = base_cost * calculate_summation_cubic(initial_level);
    let total_to_spend = already_spent + amount_to_spend;

    if total_to_spend < 0.0 {
        return Err(SummationError::CubicSumNegative);
    }

    let determinant_root = (total_to_spend / base_cost).powf(0.5);
    let solution = solve_quadratic(1.0, 1.0, -2.0 * determinant_root, true)?;

    let level_to_buy = max_level.min(solution.floor()).max(initial_level);
    let real_cost = if level_to_buy == initial_level {
        base_cost * (initial_level + 1.0).powi(3)
    } else {
        base_cost * calculate_summation_cubic(level_to_buy) - already_spent
    };

    Ok(CalculateCubicSumDataResult {
        level_can_buy: level_to_buy,
        cost: real_cost,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── calculate_summation_cubic ─────────────────────────────────────────

    #[test]
    fn cubic_sum_closed_form() {
        assert_eq!(calculate_summation_cubic(0.0), 0.0);
        assert_eq!(calculate_summation_cubic(1.0), 1.0);
        assert_eq!(calculate_summation_cubic(2.0), 9.0); // 1 + 8
        assert_eq!(calculate_summation_cubic(3.0), 36.0); // 1 + 8 + 27
        assert_eq!(calculate_summation_cubic(4.0), 100.0); // 1 + 8 + 27 + 64
        assert_eq!(calculate_summation_cubic(10.0), 3025.0);
    }

    #[test]
    fn cubic_sum_rejects_negative_and_non_integer() {
        assert_eq!(calculate_summation_cubic(-1.0), -1.0);
        assert_eq!(calculate_summation_cubic(-0.5), -1.0);
        assert_eq!(calculate_summation_cubic(1.5), -1.0);
        assert_eq!(calculate_summation_cubic(f64::INFINITY), -1.0);
    }

    // ─── solve_quadratic ───────────────────────────────────────────────────

    #[test]
    fn quadratic_real_roots() {
        // x^2 - 1 = 0 → ±1
        assert_eq!(solve_quadratic(1.0, 0.0, -1.0, true), Ok(1.0));
        assert_eq!(solve_quadratic(1.0, 0.0, -1.0, false), Ok(-1.0));
    }

    #[test]
    fn quadratic_double_root() {
        // x^2 - 2x + 1 = 0 → 1 (double root)
        assert_eq!(solve_quadratic(1.0, -2.0, 1.0, true), Ok(1.0));
        assert_eq!(solve_quadratic(1.0, -2.0, 1.0, false), Ok(1.0));
    }

    #[test]
    fn quadratic_rejects_negative_a() {
        assert_eq!(
            solve_quadratic(-1.0, 0.0, 0.0, true),
            Err(SummationError::QuadraticImproper)
        );
    }

    #[test]
    fn quadratic_rejects_negative_determinant() {
        // x^2 + 1 = 0 has no real roots
        assert_eq!(
            solve_quadratic(1.0, 0.0, 1.0, true),
            Err(SummationError::QuadraticDeterminant)
        );
    }

    // ─── calculate_summation_non_linear ────────────────────────────────────

    #[test]
    fn linear_cost_zero_resource_is_noop() {
        let result = calculate_summation_non_linear(0.0, 10.0, 0.0, 1.0, 100.0);
        assert_eq!(result.level_can_buy, 0.0);
    }

    #[test]
    fn linear_cost_purchases_within_buy_amount() {
        // base_cost = 10, diff_per_level = 0, so every level costs 10.
        // resource = 50 → can afford 5 levels, capped at buy_amount = 3.
        let result = calculate_summation_non_linear(0.0, 10.0, 50.0, 0.0, 3.0);
        assert_eq!(result.level_can_buy, 3.0);
    }

    #[test]
    fn linear_cost_handles_nan_resource() {
        // NaN normalizes to 0 (matches `resourceAvailable || 0` in TS).
        let result = calculate_summation_non_linear(0.0, 10.0, f64::NAN, 1.0, 100.0);
        assert_eq!(result.level_can_buy, 0.0);
    }

    // ─── calculate_cubic_sum_data ──────────────────────────────────────────

    #[test]
    fn cubic_sum_data_returns_max_when_already_capped() {
        let result = calculate_cubic_sum_data(10.0, 1.0, 0.0, 10.0).expect("at-cap is Ok");
        assert_eq!(result.level_can_buy, 10.0);
        assert_eq!(result.cost, 0.0);
    }

    #[test]
    fn cubic_sum_data_purchases_correct_levels() {
        // Cost of level i (1-indexed) is base * i^3.
        // Sum of first 3 cubes is 36; with base = 1 and 36 to spend from
        // level 0, the player should reach level 3.
        let result = calculate_cubic_sum_data(0.0, 1.0, 36.0, 100.0).expect("valid input");
        assert_eq!(result.level_can_buy, 3.0);
        assert_eq!(result.cost, 36.0);
    }

    #[test]
    fn cubic_sum_data_rejects_negative_spend() {
        let result = calculate_cubic_sum_data(5.0, 1.0, -1e9, 100.0);
        assert_eq!(result, Err(SummationError::CubicSumNegative));
    }

    #[test]
    fn summation_error_display_matches_legacy_codes() {
        assert_eq!(
            SummationError::QuadraticImproper.to_string(),
            "SUMMATIONS_QUADRATIC_IMPROPER"
        );
        assert_eq!(
            SummationError::QuadraticDeterminant.to_string(),
            "SUMMATIONS_QUADRATIC_DETERMINANT"
        );
        assert_eq!(
            SummationError::CubicSumNegative.to_string(),
            "SUMMATIONS_CUBIC_SUM_NEGATIVE"
        );
    }
}
