//! Math helpers — sigmoid curves, summations, integer-step utilities.
//!
//! Mirrors `legacy/core_split/packages/logic/src/math/`. The big-number type
//! itself lives in [`synergismforkd_bignum`] — this module only holds pure
//! `f64` arithmetic ported verbatim from the TS source.

pub mod rng;
pub mod sigmoid;
pub mod smallest_inc;
pub mod summations;

pub use rng::{next_f64, next_inclusive, pick};
pub use sigmoid::{calculate_sigmoid, calculate_sigmoid_exponential};
pub use smallest_inc::{smallest_inc, MAX_SAFE_INTEGER};
pub use summations::{
    calculate_cubic_sum_data, calculate_summation_cubic, calculate_summation_non_linear,
    solve_quadratic, CalculateCubicSumDataResult, CalculateSummationNonLinearResult,
    SummationError,
};
