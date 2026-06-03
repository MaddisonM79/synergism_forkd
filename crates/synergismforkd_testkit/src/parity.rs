//! TS↔Rust parity harness — math functions.
//!
//! The golden vectors in `fixtures/parity_math.json` are produced from
//! the frozen legacy TS by `parity/generate_math_vectors.mjs`. This test
//! replays each vector's inputs through the Rust port and asserts the
//! output matches the TS reference — verifying the ported formulas
//! against the source of truth, not just against Rust-side expectations.
//!
//! Only deterministic (RNG-free) functions are covered: the Rust RNG is
//! Xoshiro per-purpose and intentionally diverges from the legacy
//! MersenneTwister, so RNG-driven outputs can't be bit-compared.
//!
//! Lives as a `#[cfg(test)]` module (not `tests/`) so `serde` /
//! `serde_json` are exercised by the lib-test target — keeping the
//! workspace `unused_crate_dependencies` lint satisfied without a pile of
//! `as _` shims.

use serde::Deserialize;

use synergismforkd_logic::{
    calculate_sigmoid, calculate_sigmoid_exponential, calculate_summation_cubic, smallest_inc,
    solve_quadratic,
};

const VECTORS: &str = include_str!("../fixtures/parity_math.json");

#[derive(Deserialize)]
struct Vectors {
    calculate_summation_cubic: Vec<CubicCase>,
    solve_quadratic: Vec<QuadCase>,
    calculate_sigmoid: Vec<SigmoidCase>,
    calculate_sigmoid_exponential: Vec<SigExpCase>,
    smallest_inc: Vec<SmallestIncCase>,
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

    // Guard against an empty / mis-shaped fixture silently passing.
    assert!(
        !v.calculate_summation_cubic.is_empty() && !v.solve_quadratic.is_empty(),
        "fixture should carry vectors for every function"
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
}
