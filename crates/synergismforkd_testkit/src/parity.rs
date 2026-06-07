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

use synergismforkd_bignum::Decimal;
use synergismforkd_logic::mechanics::accelerators::{
    get_cost_accelerator, GetCostAcceleratorInput,
};
use synergismforkd_logic::mechanics::multipliers::{get_cost_multiplier, GetCostMultiplierInput};
use synergismforkd_logic::mechanics::singularity_milestones as sm;
use synergismforkd_logic::{
    calculate_cubic_sum_data, calculate_sigmoid, calculate_sigmoid_exponential,
    calculate_summation_cubic, calculate_summation_non_linear, smallest_inc, solve_quadratic,
};

const VECTORS: &str = include_str!("../fixtures/parity_vectors.json");
const DECIMAL_VECTORS: &str = include_str!("../fixtures/parity_decimal.json");
const AGGREGATOR_VECTORS: &str = include_str!("../fixtures/parity_aggregators.json");

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

// ── Decimal-returning mechanics (cost functions) ───────────────────────
// Decimals are carried as strings and reconstructed with
// `Decimal::from_string` (break-eternity-rs >= 0.4), then compared in
// log10 space (exponent + log10(mantissa) == log10(value)) — robust at any
// magnitude and across the 9.9e2/1.0e3 normalization boundary. Values stay
// in the range where the legacy `break_infinity.js` and Rust
// `break-eternity-rs` agree.

#[derive(Deserialize)]
struct DecimalVectors {
    get_cost_accelerator: Vec<CostCase>,
    get_cost_multiplier: Vec<CostCase>,
}

#[derive(Deserialize)]
struct CostCase {
    buying_to: f64,
    cost_divisor: f64,
    transcend_ecc: f64,
    in_transcension_challenge_4: bool,
    in_reincarnation_challenge_8: bool,
    value: String,
}

#[track_caller]
fn assert_decimal_parity(label: &str, rust: Decimal, ts: Decimal) {
    let rust_log = rust.exponent() + rust.mantissa().abs().log10();
    let ts_log = ts.exponent() + ts.mantissa().abs().log10();
    let tol = 1e-9 * ts_log.abs().max(1.0);
    assert!(
        (rust_log - ts_log).abs() <= tol,
        "{label}: rust={rust} ts={ts} (Δlog={})",
        (rust_log - ts_log).abs()
    );
}

#[test]
fn decimal_cost_functions_match_typescript() {
    let v: DecimalVectors = serde_json::from_str(DECIMAL_VECTORS).expect("valid decimal fixture");
    assert!(
        !v.get_cost_accelerator.is_empty() && !v.get_cost_multiplier.is_empty(),
        "fixture should carry decimal vectors"
    );

    for c in &v.get_cost_accelerator {
        let rust = get_cost_accelerator(
            c.buying_to,
            GetCostAcceleratorInput {
                cost_divisor: c.cost_divisor,
                transcend_ecc: c.transcend_ecc,
                in_transcension_challenge_4: c.in_transcension_challenge_4,
                in_reincarnation_challenge_8: c.in_reincarnation_challenge_8,
            },
        );
        let ts = Decimal::from_string(&c.value).expect("ts decimal parses");
        assert_decimal_parity(&format!("get_cost_accelerator({})", c.buying_to), rust, ts);
    }
    for c in &v.get_cost_multiplier {
        let rust = get_cost_multiplier(
            c.buying_to,
            GetCostMultiplierInput {
                cost_divisor: c.cost_divisor,
                transcend_ecc: c.transcend_ecc,
                in_transcension_challenge_4: c.in_transcension_challenge_4,
                in_reincarnation_challenge_8: c.in_reincarnation_challenge_8,
            },
        );
        let ts = Decimal::from_string(&c.value).expect("ts decimal parses");
        assert_decimal_parity(&format!("get_cost_multiplier({})", c.buying_to), rust, ts);
    }
}

// ── calculate.ts COMBINE aggregators ───────────────────────────────────
// The composition layer the audit flagged: TS-anchored (the extracted
// package's own parity tests) but not Rust-anchored. These aggregators take
// flat input structs on both sides, so the same input drives both and the
// outputs compare directly. Generated by
// `parity/generate_aggregator_vectors.mjs`.

#[derive(Deserialize)]
struct GlobalSpeedCase {
    normal_mult: f64,
    immaculate_mult: f64,
    dr_power: f64,
    result: f64,
}

#[derive(Deserialize)]
struct AscensionSpeedCase {
    base: f64,
    exponent_spread: f64,
    result: f64,
}

#[derive(Deserialize)]
struct ReductionCase {
    thrift_cost_delay: f64,
    researches_sum: f64,
    challenge_completions_4: f64,
    ant_building_cost_scale: f64,
    result: f64,
}

#[derive(Deserialize)]
struct OfferingsCase {
    base_offerings: f64,
    time_multiplier: f64,
    offering_mult: String,
    taxman_last_stand_enabled: bool,
    taxman_last_stand_completions: f64,
    current_offerings: String,
    value: String,
}

#[derive(Deserialize)]
struct ObtainiumCase {
    base_obtainium: f64,
    immaculate: f64,
    dr: f64,
    time_multiplier: f64,
    base_mults: String,
    in_ascension_challenge_14: bool,
    taxman_last_stand_enabled: bool,
    taxman_last_stand_completions: f64,
    current_obtainium: String,
    value: String,
}

#[derive(Deserialize)]
struct AggregatorVectors {
    calculate_global_speed_mult: Vec<GlobalSpeedCase>,
    calculate_ascension_speed_mult: Vec<AscensionSpeedCase>,
    get_reduction_value: Vec<ReductionCase>,
    calculate_offerings: Vec<OfferingsCase>,
    calculate_obtainium: Vec<ObtainiumCase>,
}

/// Like [`assert_decimal_parity`] but tolerant of an exact zero (the c14
/// obtainium short-circuit), which has no log10.
#[track_caller]
fn assert_aggregator_decimal(label: &str, rust: Decimal, ts: Decimal) {
    if ts.to_number() <= 0.0 || rust.to_number() <= 0.0 {
        let delta = (rust.to_number() - ts.to_number()).abs();
        assert!(delta < 1e-9, "{label}: rust={rust} ts={ts} (Δ={delta})");
        return;
    }
    assert_decimal_parity(label, rust, ts);
}

#[test]
fn calculate_aggregators_match_typescript() {
    use synergismforkd_logic::mechanics::calculate::{
        calculate_ascension_speed_mult, calculate_global_speed_mult, calculate_obtainium,
        calculate_offerings, get_reduction_value, AscensionSpeedMultInput, CalculateObtainiumInput,
        CalculateOfferingsInput, GlobalSpeedMultInput, ReductionValueInput,
    };

    let v: AggregatorVectors =
        serde_json::from_str(AGGREGATOR_VECTORS).expect("aggregator fixture is valid JSON");
    assert!(
        !v.calculate_global_speed_mult.is_empty()
            && !v.calculate_offerings.is_empty()
            && !v.calculate_obtainium.is_empty(),
        "fixture should carry aggregator vectors"
    );

    for c in &v.calculate_global_speed_mult {
        assert_parity(
            &format!(
                "calculate_global_speed_mult({}, {}, {})",
                c.normal_mult, c.immaculate_mult, c.dr_power
            ),
            calculate_global_speed_mult(&GlobalSpeedMultInput {
                normal_mult: c.normal_mult,
                immaculate_mult: c.immaculate_mult,
                dr_power: c.dr_power,
            }),
            c.result,
        );
    }
    for c in &v.calculate_ascension_speed_mult {
        assert_parity(
            &format!(
                "calculate_ascension_speed_mult({}, {})",
                c.base, c.exponent_spread
            ),
            calculate_ascension_speed_mult(&AscensionSpeedMultInput {
                base: c.base,
                exponent_spread: c.exponent_spread,
            }),
            c.result,
        );
    }
    for c in &v.get_reduction_value {
        assert_parity(
            &format!(
                "get_reduction_value({}, {}, {}, {})",
                c.thrift_cost_delay,
                c.researches_sum,
                c.challenge_completions_4,
                c.ant_building_cost_scale
            ),
            get_reduction_value(&ReductionValueInput {
                thrift_cost_delay: c.thrift_cost_delay,
                researches_sum: c.researches_sum,
                challenge_completions_4: c.challenge_completions_4,
                ant_building_cost_scale: c.ant_building_cost_scale,
            }),
            c.result,
        );
    }
    for c in &v.calculate_offerings {
        let rust = calculate_offerings(&CalculateOfferingsInput {
            base_offerings: c.base_offerings,
            time_multiplier: c.time_multiplier,
            offering_mult: Decimal::from_string(&c.offering_mult).expect("offering_mult parses"),
            taxman_last_stand_enabled: c.taxman_last_stand_enabled,
            taxman_last_stand_completions: c.taxman_last_stand_completions,
            current_offerings: Decimal::from_string(&c.current_offerings)
                .expect("current_offerings parses"),
        });
        let ts = Decimal::from_string(&c.value).expect("ts decimal parses");
        assert_aggregator_decimal(
            &format!("calculate_offerings(base={})", c.base_offerings),
            rust,
            ts,
        );
    }
    for c in &v.calculate_obtainium {
        let rust = calculate_obtainium(&CalculateObtainiumInput {
            base_obtainium: c.base_obtainium,
            immaculate: c.immaculate,
            dr: c.dr,
            time_multiplier: c.time_multiplier,
            base_mults: Decimal::from_string(&c.base_mults).expect("base_mults parses"),
            in_ascension_challenge_14: c.in_ascension_challenge_14,
            taxman_last_stand_enabled: c.taxman_last_stand_enabled,
            taxman_last_stand_completions: c.taxman_last_stand_completions,
            current_obtainium: Decimal::from_string(&c.current_obtainium)
                .expect("current_obtainium parses"),
        });
        let ts = Decimal::from_string(&c.value).expect("ts decimal parses");
        assert_aggregator_decimal(
            &format!("calculate_obtainium(base={})", c.base_obtainium),
            rust,
            ts,
        );
    }
}
