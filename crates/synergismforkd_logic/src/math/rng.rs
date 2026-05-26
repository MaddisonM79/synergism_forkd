//! General-purpose seeded RNG helpers.
//!
//! These helpers are stateless: they take any `&mut R` where `R: RngCore` and
//! return a value. Callers control which PRNG to draw from — game systems use
//! the per-purpose generators in [`crate::state::rng::RngState`], ad-hoc
//! callers (UI animations, dev tools, tests) can pass [`rand::thread_rng()`]
//! directly.
//!
//! This module replaces the legacy `packages/logic/src/math/rng.ts`. The
//! legacy implementation built a fresh `MersenneTwister(seed)` on every roll
//! and advanced the seed by `+ 1`, producing a degenerate
//! `MT(0), MT(1), MT(2), …` sequence. The Rust port uses a real PRNG
//! (`ChaCha8Rng`) with proper state, drawn through these helpers.

use rand::Rng;
use rand::RngCore;

/// Uniform sample in `[0.0, 1.0)`.
///
/// Equivalent to `MersenneTwister(seed).random()` from the legacy port — but
/// without the per-call reseed and without the `+1` counter advance.
pub fn next_f64<R: RngCore>(rng: &mut R) -> f64 {
    rng.gen::<f64>()
}

/// Inclusive integer sample in `[min, max]`. Matches the shape of the legacy
/// `seededBetween` helper.
///
/// # Panics
/// Panics if `min > max`. Callers responsible for the invariant.
pub fn next_inclusive<R: RngCore>(rng: &mut R, min: i64, max: i64) -> i64 {
    assert!(min <= max, "next_inclusive: min ({min}) > max ({max})");
    rng.gen_range(min..=max)
}

/// Uniform-random reference into `items`. Returns `None` if the slice is
/// empty.
pub fn pick<'a, T, R: RngCore>(rng: &mut R, items: &'a [T]) -> Option<&'a T> {
    if items.is_empty() {
        None
    } else {
        let idx = rng.gen_range(0..items.len());
        Some(&items[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn next_f64_is_in_unit_interval() {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        for _ in 0..1_000 {
            let v = next_f64(&mut rng);
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn next_inclusive_respects_bounds() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for _ in 0..1_000 {
            let v = next_inclusive(&mut rng, -10, 10);
            assert!((-10..=10).contains(&v));
        }
    }

    #[test]
    fn next_inclusive_single_value_range() {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        assert_eq!(next_inclusive(&mut rng, 7, 7), 7);
    }

    #[test]
    fn pick_returns_none_for_empty_slice() {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let empty: &[i32] = &[];
        assert!(pick(&mut rng, empty).is_none());
    }

    #[test]
    fn pick_returns_a_member_for_non_empty_slice() {
        let mut rng = ChaCha8Rng::seed_from_u64(0);
        let items = [10, 20, 30, 40];
        for _ in 0..100 {
            let v = *pick(&mut rng, &items).unwrap();
            assert!(items.contains(&v));
        }
    }

    #[test]
    fn deterministic_for_fixed_seed() {
        let mut a = ChaCha8Rng::seed_from_u64(12345);
        let mut b = ChaCha8Rng::seed_from_u64(12345);
        for _ in 0..100 {
            assert_eq!(next_f64(&mut a), next_f64(&mut b));
        }
    }
}
