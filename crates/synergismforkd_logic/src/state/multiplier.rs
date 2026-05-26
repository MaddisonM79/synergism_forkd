//! Multiplier-purchase state slice.
//!
//! Mirrors `MultiplierState` from the legacy TS
//! `packages/logic/src/state/schema.ts`. Same shape as `AcceleratorState`
//! with different field names — the flag pattern is identical.

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` read/written by the multiplier-purchase machinery.
///
/// See [`crate::state::AcceleratorState`] for the rationale on `f64` for the
/// owned-count field.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiplierState {
    /// Total multipliers owned.
    pub multiplier_bought: f64,
    /// Cost of the next multiplier.
    pub multiplier_cost: Decimal,
    /// The spend resource (coins).
    pub coins: Decimal,
    /// Set false once any multiplier is owned; gates a
    /// no-multiplier-prestige achievement.
    pub prestige_no_multiplier: bool,
    /// Same flag, transcension lineage.
    pub transcend_no_multiplier: bool,
    /// Same flag, reincarnation lineage.
    pub reincarnate_no_multiplier: bool,
}
