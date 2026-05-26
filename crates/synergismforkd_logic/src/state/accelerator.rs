//! Accelerator-purchase state slice.
//!
//! Mirrors `AcceleratorState` from the legacy TS
//! `packages/logic/src/state/schema.ts`. The `*_no_accelerator` flags are
//! achievement gates flipped to `false` once any accelerator is owned.

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` read/written by the accelerator-purchase machinery.
///
/// `accelerator_bought` is an `f64` rather than an integer because the
/// high-end buy loop walks past `2^53` using [`smallest_inc`] and relies on
/// the same float semantics as the legacy TS implementation.
///
/// [`smallest_inc`]: crate::math::smallest_inc
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AcceleratorState {
    /// Total accelerators owned. Walked past `MAX_SAFE_INTEGER` in the
    /// high-end binary-search branch.
    pub accelerator_bought: f64,
    /// Cost of the next accelerator (cached so the UI can render without
    /// recomputing).
    pub accelerator_cost: Decimal,
    /// The spend resource (coins) — shared with the multiplier slice in the
    /// composed `GameState`.
    pub coins: Decimal,
    /// Set false once any accelerator is owned; gates a
    /// no-accelerator-prestige achievement.
    pub prestige_no_accelerator: bool,
    /// Same flag, transcension lineage.
    pub transcend_no_accelerator: bool,
    /// Same flag, reincarnation lineage.
    pub reincarnate_no_accelerator: bool,
}

impl Default for AcceleratorState {
    /// Zeroed counters; achievement flags start `true` because no
    /// accelerator has been purchased yet.
    fn default() -> Self {
        Self {
            accelerator_bought: 0.0,
            accelerator_cost: Decimal::zero(),
            coins: Decimal::zero(),
            prestige_no_accelerator: true,
            transcend_no_accelerator: true,
            reincarnate_no_accelerator: true,
        }
    }
}
