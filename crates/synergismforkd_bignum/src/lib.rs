#![cfg_attr(not(test), deny(clippy::unwrap_used))]

//! Synergism Forkd — bignum re-export.
//!
//! Thin wrapper around the maintained [`break-eternity-rs`] fork of
//! `cozyGalvinism/break-eternity`. Every Synergism Forkd crate that
//! touches big numbers depends on this crate, never on the upstream
//! directly — that keeps the underlying impl swappable without churning
//! call sites.
//!
//! See the upstream `Decimal` docs for the full API: arithmetic via
//! operator overloads (`+`, `-`, `*`, `/`), constants (`Decimal::zero`,
//! `one`, `ten`, `nan`, `inf`, `maximum`), comparison via [`Ord`] /
//! [`PartialOrd`], and the BE.js-equivalent helpers (`log10`, `ln`,
//! `pow`, `tetrate`, `iteratedexp`, `iteratedlog`, `slog`, `sqrt`,
//! `cbrt`, `gamma`, `factorial`, `lambertw`, …).
//!
//! [`break-eternity-rs`]: https://crates.io/crates/break-eternity-rs

pub use break_eternity::{
    decimal_places, sign, to_fixed, ArithmeticError, ArithmeticErrorKind, BreakEternityError,
    Decimal, TetrationMode, COMPARE_EPSILON, EXPN1, EXPONENT_LIMIT, FIRST_NEG_LAYER,
    LAYER_REDUCTION_THRESHOLD, MAX_ES_IN_A_ROW, MAX_FLOAT_PRECISION, MAX_POWERS_OF_TEN,
    NUMBER_EXP_MAX, NUMBER_EXP_MIN, OMEGA, TWO_PI,
};

// The whole arithmetic style across the workspace assumes `Decimal: Copy`
// (pass by value, no `&` or `.clone()`). Codify the assumption — if a
// future `break-eternity-rs` removes `Copy`, this fails at compile time
// instead of silently turning every call site into a clone.
const _: () = {
    const fn assert_copy<T: Copy>() {}
    assert_copy::<Decimal>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_and_one_round_trip_to_f64() {
        assert_eq!(Decimal::zero().to_number(), 0.0);
        assert_eq!(Decimal::one().to_number(), 1.0);
    }

    #[test]
    fn small_arithmetic_matches_f64() {
        // `Decimal` is `Copy`, so passing by value is the idiomatic call form.
        let a = Decimal::from_finite(2.0);
        let b = Decimal::from_finite(3.0);
        assert_eq!((a + b).to_number(), 5.0);
        assert_eq!((a * b).to_number(), 6.0);
    }

    #[test]
    fn handles_magnitudes_past_f64_max() {
        // 1e400 is past f64::MAX (~1.8e308); a normal f64 would be inf.
        let huge = Decimal::from_mantissa_exponent(1.0, 400.0);
        assert!(huge.to_number().is_infinite() || huge.to_number() > 1e300);
        assert!(huge.exponent() >= 400.0);
    }
}
