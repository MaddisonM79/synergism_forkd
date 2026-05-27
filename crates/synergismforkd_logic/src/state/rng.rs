//! Per-system PRNG state.
//!
//! Each game system that needs deterministic randomness gets its own
//! `Xoshiro256PlusPlus` instance, keyed by [`RngPurpose`]. Drawing from one
//! purpose's generator never disturbs another's sequence, so unrelated
//! subsystems stay independent.
//!
//! This slice replaces the legacy `player.seed[]` integer array. The legacy
//! scheme reinstantiated `MersenneTwister(seed)` on every roll and advanced
//! the seed by `+ 1`; the Rust port keeps real PRNG state per purpose and
//! advances it by drawing from it.
//!
//! `RngState` owns the generators outright — no shared `Rc`/`Arc` indirection.
//! Game code accesses a specific purpose's generator with [`RngState::draw`]
//! and feeds the returned `&mut Xoshiro256PlusPlus` into the helpers in
//! [`crate::math::rng`].

use serde::{Deserialize, Serialize};

use rand::rngs::OsRng;
use rand::RngCore;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

/// Identifies a per-system PRNG sequence. Adding a new entry is the canonical
/// way to give a subsystem its own statistically independent random stream —
/// extend the enum and bump [`RngPurpose::COUNT`].
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum RngPurpose {
    /// Promo-code redemption rolls. Legacy `Seed.PromoCodes = 0`.
    PromoCodes = 0,
    /// Ambrosia loot rolls. Legacy `Seed.Ambrosia = 1`.
    Ambrosia = 1,
    /// Red-ambrosia loot rolls. Legacy `Seed.RedAmbrosia = 2`.
    RedAmbrosia = 2,
}

impl RngPurpose {
    /// Count of `RngPurpose` variants. Update when adding a variant.
    pub const COUNT: usize = 3;
}

/// One `Xoshiro256PlusPlus` per [`RngPurpose`]. Fixed-arity array keyed by the
/// enum's `repr(usize)` discriminant — O(1) lookup, no allocator pressure,
/// and the layout stays deterministic for future save serialization.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RngState {
    rngs: [Xoshiro256PlusPlus; RngPurpose::COUNT],
}

impl RngState {
    /// Build an `RngState` from a single master `u64` seed. Each per-purpose
    /// generator is keyed off a value drawn from a master `Xoshiro256PlusPlus` so the
    /// sequences are statistically independent but the whole state is
    /// reproducible from one seed.
    pub fn from_seed(seed: u64) -> Self {
        let mut master = Xoshiro256PlusPlus::seed_from_u64(seed);
        Self::seed_array_from(&mut master)
    }

    /// Build an `RngState` from system entropy. Each per-purpose generator
    /// gets a fresh seed drawn from `OsRng`-backed master state.
    ///
    /// # Panics
    ///
    /// Panics if the operating-system RNG (`OsRng`) is unavailable. On
    /// Unix this calls `getrandom(2)`; on Windows it calls
    /// `BCryptGenRandom`. On `wasm32` it requires the `js` feature on the
    /// `getrandom` crate, which the `synergismforkd_ui_web` crate wires
    /// for its WASM target. In practice the syscall path is reliable; the
    /// WASM path requires the JS shim to be linked or the program would
    /// have failed earlier.
    pub fn from_entropy() -> Self {
        let mut master = Xoshiro256PlusPlus::from_rng(OsRng).expect("OsRng should not fail");
        Self::seed_array_from(&mut master)
    }

    fn seed_array_from<R: RngCore>(master: &mut R) -> Self {
        let rngs = std::array::from_fn(|_| {
            let mut child_seed = [0u8; 32];
            master.fill_bytes(&mut child_seed);
            Xoshiro256PlusPlus::from_seed(child_seed)
        });
        Self { rngs }
    }

    /// Mutable handle to the PRNG dedicated to `purpose`. Pass the returned
    /// reference to any [`crate::math::rng`] helper.
    pub fn draw(&mut self, purpose: RngPurpose) -> &mut Xoshiro256PlusPlus {
        &mut self.rngs[purpose as usize]
    }
}

impl Default for RngState {
    /// Deterministic — equivalent to `RngState::from_seed(0)`. This keeps
    /// `GameState::default()` (used pervasively in tests) non-panicking
    /// and reproducible. Use [`RngState::from_entropy`] when fresh
    /// system entropy is required; that constructor can panic and is
    /// explicitly opt-in.
    fn default() -> Self {
        Self::from_seed(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::rng::next_f64;

    #[test]
    fn from_seed_is_deterministic() {
        let mut a = RngState::from_seed(0xC0FFEE);
        let mut b = RngState::from_seed(0xC0FFEE);
        for purpose in [
            RngPurpose::PromoCodes,
            RngPurpose::Ambrosia,
            RngPurpose::RedAmbrosia,
        ] {
            for _ in 0..10 {
                assert_eq!(next_f64(a.draw(purpose)), next_f64(b.draw(purpose)));
            }
        }
    }

    #[test]
    fn purposes_are_independent() {
        let mut state = RngState::from_seed(7);

        // Snapshot Ambrosia's first 5 draws.
        let baseline: Vec<f64> = (0..5)
            .map(|_| next_f64(state.draw(RngPurpose::Ambrosia)))
            .collect();

        // Pull a bunch from RedAmbrosia.
        let mut other = RngState::from_seed(7);
        for _ in 0..50 {
            let _ = next_f64(other.draw(RngPurpose::RedAmbrosia));
        }
        // Ambrosia sequence in `other` should still match `baseline`.
        let actual: Vec<f64> = (0..5)
            .map(|_| next_f64(other.draw(RngPurpose::Ambrosia)))
            .collect();
        assert_eq!(baseline, actual);
    }

    #[test]
    fn different_seeds_produce_different_sequences() {
        let mut a = RngState::from_seed(1);
        let mut b = RngState::from_seed(2);
        let a_first = next_f64(a.draw(RngPurpose::Ambrosia));
        let b_first = next_f64(b.draw(RngPurpose::Ambrosia));
        assert_ne!(a_first, b_first);
    }

    #[test]
    fn from_entropy_yields_distinct_states() {
        // Vanishingly small probability of collision — exercise the OsRng
        // path end-to-end.
        let mut a = RngState::from_entropy();
        let mut b = RngState::from_entropy();
        assert_ne!(
            next_f64(a.draw(RngPurpose::Ambrosia)),
            next_f64(b.draw(RngPurpose::Ambrosia))
        );
    }
}
