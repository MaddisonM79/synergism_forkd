//! Static data + polynomial cost solvers for the research tree.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/researches.ts`
//! (lifted from the legacy `packages/web_ui/src/Research.ts`). The
//! unlock predicates stay in the UI tier because they close over
//! `player.*` and `runes.*`; the resulting per-index `researchData`
//! map is composed there from these logic-provided arrays plus the
//! local unlock closures.
//!
//! Index 0 is intentionally unused — research IDs are 1-based — so
//! `RESEARCH_BASE_COSTS[0] == f64::INFINITY` and
//! `RESEARCH_MAX_LEVELS[0] == 0`.

use synergismforkd_bignum::Decimal;

/// Per-research base cost. Index 0 is `f64::INFINITY` (1-based
/// convention).
#[rustfmt::skip]
pub const RESEARCH_BASE_COSTS: [f64; 201] = [
    f64::INFINITY,
    1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1e2, 1e4, 1e6, 1e8,
    2.0, 2e2, 2e4, 2e6, 2e8,
    4e4, 4e8, 10.0, 1e5, 1e9,
    100.0, 100.0, 1e4, 2e3, 2e5,
    40.0, 200.0, 50.0, 5000.0, 20_000_000.0,
    777.0, 7777.0, 50_000.0, 500_000.0, 5_000_000.0,
    2e3, 2e6, 2e9, 1e5, 1e9,
    1.0, 1.0, 5.0, 25.0, 125.0,
    2.0, 5.0, 320.0, 1280.0, 2.5e9,
    10.0, 2e3, 4e5, 8e7, 2e9,
    5.0, 400.0, 1e4, 3e6, 9e8,
    100.0, 2500.0, 100.0, 2000.0, 2e5,
    1.0, 20.0, 3e3, 4e5, 5e7,
    10.0, 40.0, 160.0, 1000.0, 10_000.0,
    4e9, 7e9, 1e10, 1.2e10, 1.5e10,
    1e12, 1e13, 1e12, 4e12, 7e12,
    1e13, 1e13, 4e13, 6e13, 1e14,
    8e13, 1e14, 2e14, 2e14, 1e15,
    4e12, 3e13, 8e13, 7.777e18, 7.777e20,
    2e14, 3e14, 1e16, 3e16, 1e16,
    1e17, 3e17, 5e16, 1.2e17, 1e18,
    1e18, 2e18, 3e18, 4e18, 1e19,
    1e19, 2e19, 1e21, 5e21, 1e22,
    1e21, 1e22, 1e22, 1e20, 7.777e32,
    // ascension tier
    5e8, 5e12, 5e16, 5e20, 5e24,
    1e25, 2e25, 4e25, 8e25, 1e26,
    4e26, 8e26, 1e27, 2e27, 1e28,
    // challenge 11 tier
    5e9, 5e15, 5e21, 5e27, 1e28,
    1e29, 2e29, 4e29, 8e29, 1e27,
    2e30, 4e30, 8e30, 1e31, 2e31,
    // challenge 12 tier
    5e31, 1e32, 2e32, 4e32, 8e32,
    1e33, 2e33, 4e33, 8e33, 1e34,
    3e34, 1e35, 3e35, 6e37, 1e36,
    // challenge 13 tier
    3e36, 1e37, 3e37, 1e38, 3e38,
    1e39, 3e39, 1e40, 3e40, 1e50,
    3e41, 1e42, 3e42, 6e42, 1e43,
    // challenge 14 tier
    3e43, 1e44, 3e44, 1e45, 3e45,
    2e46, 6e46, 2e47, 6e47, 1e64,
    6e48, 2e49, 1e50, 1e51, 4e56,
];

/// Per-research max level. Index 0 is `0` (1-based convention).
#[rustfmt::skip]
pub const RESEARCH_MAX_LEVELS: [f64; 201] = [
    0.0,
    1.0, 1.0, 1.0, 1.0, 1.0,
    10.0, 10.0, 10.0, 10.0, 10.0,
    10.0, 10.0, 10.0, 10.0, 10.0,
    10.0, 10.0, 1.0, 1.0, 1.0,
    25.0, 25.0, 25.0, 20.0, 20.0,
    10.0, 10.0, 10.0, 10.0, 10.0,
    12.0, 12.0, 10.0, 10.0, 10.0,
    10.0, 10.0, 10.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, 1.0, 1.0,
    10.0, 10.0, 10.0, 10.0, 10.0,
    20.0, 20.0, 20.0, 20.0, 20.0,
    1.0, 5.0, 4.0, 5.0, 5.0,
    10.0, 10.0, 10.0, 10.0, 10.0,
    1.0, 1.0, 1.0, 1.0, 1.0,
    10.0, 15.0, 15.0, 15.0, 15.0,
    10.0, 1.0, 20.0, 20.0, 20.0,
    20.0, 20.0, 20.0, 20.0, 10.0,
    20.0, 20.0, 20.0, 20.0, 1.0,
    20.0, 7.0, 7.0, 3.0, 2.0,
    10.0, 12.0, 10.0, 10.0, 1.0,
    10.0, 10.0, 20.0, 25.0, 25.0,
    15.0, 15.0, 15.0, 15.0, 30.0,
    2.0, 10.0, 10.0, 100.0, 100.0,
    25.0, 25.0, 25.0, 1.0, 5.0,
    10.0, 10.0, 10.0, 10.0, 1.0,
    10.0, 10.0, 10.0, 1.0, 1.0,
    25.0, 25.0, 25.0, 15.0, 1.0,
    10.0, 10.0, 10.0, 10.0, 1.0,
    10.0, 1.0, 25.0, 10.0, 1.0,
    25.0, 25.0, 1.0, 15.0, 1.0,
    10.0, 10.0, 10.0, 1.0, 1.0,
    10.0, 10.0, 10.0, 10.0, 1.0,
    25.0, 25.0, 25.0, 100_000.0, 1.0,
    10.0, 10.0, 10.0, 1.0, 1.0,
    10.0, 3.0, 6.0, 10.0, 5.0,
    25.0, 25.0, 1.0, 15.0, 1.0,
    20.0, 20.0, 20.0, 1.0, 1.0,
    20.0, 1.0, 50.0, 50.0, 10.0,
    25.0, 25.0, 25.0, 15.0, 100_000.0,
];

/// "Given a budget, what's the max level I can reach" function for
/// the given polynomial `degree`.
///
/// The cost from level 0 to level `n` is `base_cost * n^degree`.
/// Inverting: adding the already-paid `base_cost * curr_level^degree`
/// back to the budget gives the total cost the player could pay
/// starting from 0, then
/// `level = (effective_budget / base_cost)^(1/degree)` capped at
/// `max_level`.
///
/// Requires `degree != 0`; intended for positive `degree`.
#[must_use]
pub fn poly_buy_to_level(
    degree: f64,
    budget: Decimal,
    base_cost: Decimal,
    curr_level: f64,
    max_level: f64,
) -> f64 {
    let effective_budget = budget + base_cost * Decimal::from_finite(curr_level.powf(degree));
    max_level.min(
        (effective_budget / base_cost)
            .pow(Decimal::from_finite(1.0 / degree))
            .floor()
            .to_number(),
    )
}

/// "How much does it cost to buy from `curr_level` to `buy_to`"
/// function for the given polynomial `degree`.
///
/// Cost-to-buy delta:
/// `base_cost * (buy_to^degree - curr_level^degree)`. Returns `0`
/// when `curr_level == buy_to` (avoids potential floating-point
/// noise on the identity diff).
///
/// Requires `degree != 0`; intended for positive `degree`.
#[must_use]
pub fn poly_cost_for_levels(
    degree: f64,
    base_cost: Decimal,
    curr_level: f64,
    buy_to: f64,
) -> Decimal {
    if curr_level == buy_to {
        return Decimal::zero();
    }
    base_cost * Decimal::from_finite(buy_to.powf(degree) - curr_level.powf(degree))
}

/// Index-range → polynomial degree assignment. `degree = 1` implies
/// constant cost per level; `degree = 2` implies linear growth in
/// cost per level. Index 200 uses degree-2 (its `base_cost = 4e56`
/// would otherwise be impossibly expensive to scale linearly).
#[must_use]
pub fn research_polynomial_degree(index: u32) -> f64 {
    if index >= 200 {
        2.0
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_costs_index_0_is_infinity() {
        assert!(RESEARCH_BASE_COSTS[0].is_infinite());
    }

    #[test]
    fn base_costs_index_1_is_one() {
        assert_eq!(RESEARCH_BASE_COSTS[1], 1.0);
    }

    #[test]
    fn base_costs_length_matches_max_levels() {
        assert_eq!(RESEARCH_BASE_COSTS.len(), RESEARCH_MAX_LEVELS.len());
    }

    #[test]
    fn max_levels_index_0_is_zero() {
        assert_eq!(RESEARCH_MAX_LEVELS[0], 0.0);
    }

    #[test]
    fn polynomial_degree_default_is_one() {
        assert_eq!(research_polynomial_degree(50), 1.0);
        assert_eq!(research_polynomial_degree(199), 1.0);
    }

    #[test]
    fn polynomial_degree_200_is_two() {
        assert_eq!(research_polynomial_degree(200), 2.0);
    }

    #[test]
    fn poly_buy_to_level_degree_1_solves_correctly() {
        // base_cost = 100, curr_level = 0, max_level = 100, budget = 500
        // effective_budget = 500 + 100*0 = 500
        // level = floor(500/100)^1 = 5
        let result = poly_buy_to_level(
            1.0,
            Decimal::from_finite(500.0),
            Decimal::from_finite(100.0),
            0.0,
            100.0,
        );
        assert_eq!(result, 5.0);
    }

    #[test]
    fn poly_buy_to_level_caps_at_max() {
        let result = poly_buy_to_level(
            1.0,
            Decimal::from_finite(1e20),
            Decimal::from_finite(100.0),
            0.0,
            10.0,
        );
        assert_eq!(result, 10.0);
    }

    #[test]
    fn poly_cost_for_levels_degree_1_is_linear_delta() {
        let result = poly_cost_for_levels(1.0, Decimal::from_finite(100.0), 5.0, 10.0);
        // 100 * (10 - 5) = 500
        assert_eq!(result.to_number(), 500.0);
    }

    #[test]
    fn poly_cost_for_levels_degree_2_is_quadratic_delta() {
        let result = poly_cost_for_levels(2.0, Decimal::from_finite(100.0), 5.0, 10.0);
        // 100 * (100 - 25) = 7500
        assert_eq!(result.to_number(), 7500.0);
    }

    #[test]
    fn poly_cost_for_levels_at_curr_returns_zero() {
        let result = poly_cost_for_levels(1.0, Decimal::from_finite(100.0), 5.0, 5.0);
        assert_eq!(result, Decimal::zero());
    }
}
