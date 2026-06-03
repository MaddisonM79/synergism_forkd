//! TS↔Rust parity harness.
//!
//! The golden vectors in `fixtures/parity_vectors.json` are produced from
//! the frozen legacy TS by `parity/generate_parity_vectors.mjs`. Each test
//! replays a vector's inputs through the Rust port and asserts the output
//! matches the TS reference — verifying the ported formulas against the
//! source of truth, not just against Rust-side expectations.
//!
//! Only deterministic (RNG-free), dependency-free functions are covered:
//! the Rust RNG (Xoshiro per-purpose) diverges from the legacy
//! MersenneTwister by design, and Decimal-typed mechanics need
//! `break_infinity.js` installed (a separate fixture once wired).
//!
//! Lives as a `#[cfg(test)]` module (not `tests/`) so `serde` /
//! `serde_json` are exercised by the lib-test target — keeping the
//! workspace `unused_crate_dependencies` lint satisfied without a pile of
//! `as _` shims.

use serde::Deserialize;

use synergismforkd_logic::mechanics::singularity_milestones as sm;
use synergismforkd_logic::{
    calculate_cubic_sum_data, calculate_sigmoid, calculate_sigmoid_exponential,
    calculate_summation_cubic, calculate_summation_non_linear, smallest_inc, solve_quadratic,
};

const VECTORS: &str = include_str!("../fixtures/parity_vectors.json");

#[derive(Deserialize)]
struct Vectors {
    // ── math: scalar ──
    calculate_summation_cubic: Vec<CubicCase>,
    solve_quadratic: Vec<QuadCase>,
    calculate_sigmoid: Vec<SigmoidCase>,
    calculate_sigmoid_exponential: Vec<SigExpCase>,
    smallest_inc: Vec<SmallestIncCase>,
    // ── math: struct-returning ──
    calculate_cubic_sum_data: Vec<CubicSumDataCase>,
    calculate_summation_non_linear: Vec<NonLinearCase>,
    // ── singularity milestones ──
    calculate_singularity_quark_milestone_multiplier: Vec<SingCountCase>,
    calculate_base_golden_quarks: Vec<BaseGqCase>,
    calculate_singularity_ambrosia_luck_milestone_bonus: Vec<HighSingCase>,
    calculate_dilated_five_leaf_bonus: Vec<HighSingCase>,
    derpsmith_cornucopia_bonus: Vec<HighSingCase>,
    calculate_immaculate_alchemy_bonus: Vec<SingCountCase>,
    inheritance_tokens: Vec<HighSingCase>,
    sum_of_exalt_completions: Vec<ExaltCase>,
    singularity_bonus_token_mult: Vec<HighSingCase>,
}

#[derive(Deserialize)]
struct CubicCase {
    n: f64,
    result: f64,
}

#[derive(Deserialize)]
struct QuadCase {
    a: f64,
    b: f64,
    c: f64,
    positive: bool,
    result: f64,
}

#[derive(Deserialize)]
struct SigmoidCase {
    constant: f64,
    factor: f64,
    divisor: f64,
    result: f64,
}

#[derive(Deserialize)]
struct SigExpCase {
    constant: f64,
    coefficient: f64,
    result: f64,
}

#[derive(Deserialize)]
struct SmallestIncCase {
    x: f64,
    result: f64,
}

#[derive(Deserialize)]
struct CubicSumDataCase {
    initial_level: f64,
    base_cost: f64,
    amount_to_spend: f64,
    max_level: f64,
    level_can_buy: f64,
    cost: f64,
}

#[derive(Deserialize)]
struct NonLinearCase {
    base_level: f64,
    base_cost: f64,
    resource_available: f64,
    diff_per_level: f64,
    buy_amount: f64,
    level_can_buy: f64,
    cost: f64,
}

#[derive(Deserialize)]
struct SingCountCase {
    singularity_count: f64,
    result: f64,
}

#[derive(Deserialize)]
struct BaseGqCase {
    singularity: f64,
    quarks_this_singularity: f64,
    highest_singularity_count: f64,
    result: f64,
}

#[derive(Deserialize)]
struct HighSingCase {
    highest_singularity_count: f64,
    result: f64,
}

#[derive(Deserialize)]
struct ExaltCase {
    completions: Vec<f64>,
    result: f64,
}

/// Assert the Rust output matches the TS reference within a tolerance
/// that absorbs last-ULP `pow` / op-order noise but catches a real
/// formula divergence (which would be orders of magnitude larger).
#[track_caller]
fn assert_parity(label: &str, rust: f64, ts: f64) {
    let tol = 1e-9 * ts.abs() + 1e-12;
    let delta = (rust - ts).abs();
    assert!(
        delta <= tol,
        "{label}: rust={rust} ts={ts} (Δ={delta}, tol={tol})"
    );
}

#[test]
fn math_functions_match_typescript() {
    let v: Vectors = serde_json::from_str(VECTORS).expect("fixture is valid JSON");
    assert!(
        !v.calculate_summation_cubic.is_empty() && !v.calculate_cubic_sum_data.is_empty(),
        "fixture should carry math vectors"
    );

    for c in &v.calculate_summation_cubic {
        assert_parity(
            &format!("calculate_summation_cubic({})", c.n),
            calculate_summation_cubic(c.n),
            c.result,
        );
    }
    for c in &v.solve_quadratic {
        let rust = solve_quadratic(c.a, c.b, c.c, c.positive)
            .expect("solve_quadratic returned Err for a real-root fixture case");
        assert_parity(
            &format!("solve_quadratic({}, {}, {}, {})", c.a, c.b, c.c, c.positive),
            rust,
            c.result,
        );
    }
    for c in &v.calculate_sigmoid {
        assert_parity(
            &format!(
                "calculate_sigmoid({}, {}, {})",
                c.constant, c.factor, c.divisor
            ),
            calculate_sigmoid(c.constant, c.factor, c.divisor),
            c.result,
        );
    }
    for c in &v.calculate_sigmoid_exponential {
        assert_parity(
            &format!(
                "calculate_sigmoid_exponential({}, {})",
                c.constant, c.coefficient
            ),
            calculate_sigmoid_exponential(c.constant, c.coefficient),
            c.result,
        );
    }
    for c in &v.smallest_inc {
        assert_parity(
            &format!("smallest_inc({})", c.x),
            smallest_inc(c.x),
            c.result,
        );
    }
    for c in &v.calculate_cubic_sum_data {
        let r =
            calculate_cubic_sum_data(c.initial_level, c.base_cost, c.amount_to_spend, c.max_level)
                .expect("calculate_cubic_sum_data returned Err for a fixture case");
        let label = format!(
            "calculate_cubic_sum_data({}, {}, {}, {})",
            c.initial_level, c.base_cost, c.amount_to_spend, c.max_level
        );
        assert_parity(
            &format!("{label}.level_can_buy"),
            r.level_can_buy,
            c.level_can_buy,
        );
        assert_parity(&format!("{label}.cost"), r.cost, c.cost);
    }
    for c in &v.calculate_summation_non_linear {
        let r = calculate_summation_non_linear(
            c.base_level,
            c.base_cost,
            c.resource_available,
            c.diff_per_level,
            c.buy_amount,
        );
        let label = format!(
            "calculate_summation_non_linear({}, {}, {}, {}, {})",
            c.base_level, c.base_cost, c.resource_available, c.diff_per_level, c.buy_amount
        );
        assert_parity(
            &format!("{label}.level_can_buy"),
            r.level_can_buy,
            c.level_can_buy,
        );
        assert_parity(&format!("{label}.cost"), r.cost, c.cost);
    }
}

#[test]
fn singularity_milestones_match_typescript() {
    let v: Vectors = serde_json::from_str(VECTORS).expect("fixture is valid JSON");
    assert!(
        !v.calculate_base_golden_quarks.is_empty() && !v.inheritance_tokens.is_empty(),
        "fixture should carry singularity vectors"
    );

    for c in &v.calculate_singularity_quark_milestone_multiplier {
        assert_parity(
            &format!("quark_milestone_multiplier({})", c.singularity_count),
            sm::calculate_singularity_quark_milestone_multiplier(c.singularity_count),
            c.result,
        );
    }
    for c in &v.calculate_base_golden_quarks {
        let rust = sm::calculate_base_golden_quarks(&sm::CalculateBaseGoldenQuarksInput {
            singularity: c.singularity,
            quarks_this_singularity: c.quarks_this_singularity,
            highest_singularity_count: c.highest_singularity_count,
        });
        assert_parity(
            &format!(
                "base_golden_quarks({}, {}, {})",
                c.singularity, c.quarks_this_singularity, c.highest_singularity_count
            ),
            rust,
            c.result,
        );
    }
    for c in &v.calculate_singularity_ambrosia_luck_milestone_bonus {
        assert_parity(
            &format!("ambrosia_luck_milestone({})", c.highest_singularity_count),
            sm::calculate_singularity_ambrosia_luck_milestone_bonus(c.highest_singularity_count),
            c.result,
        );
    }
    for c in &v.calculate_dilated_five_leaf_bonus {
        assert_parity(
            &format!("dilated_five_leaf({})", c.highest_singularity_count),
            sm::calculate_dilated_five_leaf_bonus(c.highest_singularity_count),
            c.result,
        );
    }
    for c in &v.derpsmith_cornucopia_bonus {
        assert_parity(
            &format!("derpsmith_cornucopia({})", c.highest_singularity_count),
            sm::derpsmith_cornucopia_bonus(c.highest_singularity_count),
            c.result,
        );
    }
    for c in &v.calculate_immaculate_alchemy_bonus {
        assert_parity(
            &format!("immaculate_alchemy({})", c.singularity_count),
            sm::calculate_immaculate_alchemy_bonus(c.singularity_count),
            c.result,
        );
    }
    for c in &v.inheritance_tokens {
        assert_parity(
            &format!("inheritance_tokens({})", c.highest_singularity_count),
            sm::inheritance_tokens(c.highest_singularity_count),
            c.result,
        );
    }
    for c in &v.sum_of_exalt_completions {
        assert_parity(
            &format!("sum_of_exalt_completions({:?})", c.completions),
            sm::sum_of_exalt_completions(&c.completions),
            c.result,
        );
    }
    for c in &v.singularity_bonus_token_mult {
        assert_parity(
            &format!("bonus_token_mult({})", c.highest_singularity_count),
            sm::singularity_bonus_token_mult(c.highest_singularity_count),
            c.result,
        );
    }
}
