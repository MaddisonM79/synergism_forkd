//! Smallest integer step that changes the value of `x` under IEEE-754
//! doubles.
//!
//! Verbatim port of `legacy/core_split/packages/logic/src/math/smallestInc.ts`
//! (originally attributed to httpsnet in the legacy `web_ui/src/Utility.ts`).
//! Below 2^53 every integer is exactly representable, so the step is 1; above
//! it, doubles lose precision and the step grows as a power of two. The
//! `buyAccelerator` / `buyMultiplier` loops use this to walk the
//! `acceleratorBought` / `multiplierBought` counters past the safe-integer
//! boundary without getting stuck on repeated identical floats.

/// 2^53 - 1, the largest integer where every adjacent integer is also
/// representable as an `f64`. Mirrors JavaScript's `Number.MAX_SAFE_INTEGER`.
pub const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

/// Smallest step that changes the value of `x` as an `f64`.
///
/// For `x <= MAX_SAFE_INTEGER` this is always 1; above that, the step grows
/// as `2^floor(log2(x) - 52)`.
pub fn smallest_inc(x: f64) -> f64 {
    if x <= MAX_SAFE_INTEGER {
        1.0
    } else {
        2.0_f64.powf((x.log2() - 52.0).floor())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_is_one_below_safe_integer() {
        assert_eq!(smallest_inc(0.0), 1.0);
        assert_eq!(smallest_inc(1.0), 1.0);
        assert_eq!(smallest_inc(1_000.0), 1.0);
        assert_eq!(smallest_inc(MAX_SAFE_INTEGER), 1.0);
    }

    #[test]
    fn step_doubles_just_above_safe_integer() {
        // 2^53 is the first integer where odd values are no longer
        // representable; the step becomes 2.
        let v = 2.0_f64.powi(53);
        assert_eq!(smallest_inc(v), 2.0);
    }

    #[test]
    fn step_grows_as_power_of_two() {
        // 2^54 → step 4, 2^55 → step 8, etc.
        assert_eq!(smallest_inc(2.0_f64.powi(54)), 4.0);
        assert_eq!(smallest_inc(2.0_f64.powi(55)), 8.0);
        assert_eq!(smallest_inc(2.0_f64.powi(60)), 256.0);
    }
}
