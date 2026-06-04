//! Persisted `G.*` derived-cache values.
//!
//! The legacy TS kept these in the mutable `Globals` (`G`) object —
//! recomputed every tick, but read by some consumers *before* the
//! recompute fires, so they carry a deliberate **one-tick lag**. This
//! slice is the Rust home for that category, distinct from the `player.*`
//! state slices. Per the porting plan, more `G.*` aggregator-output
//! values migrate here as the tick learns to self-derive them from
//! `&GameState`.
//!
//! These values are recomputable cache, not authoritative save data; they
//! are serialized only because [`crate::state::GameState`] is serialized
//! whole. On a fresh load they self-correct within the first tick.

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Derived `G.*` cache values that must survive between ticks to preserve
/// the legacy read-before-recompute ordering.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct GCacheState {
    /// `G.taxdivisor` — the coin-production tax divisor.
    ///
    /// Consumed by [`crate::mechanics::update_all_multiplier`] (the
    /// `upgrade[68]` free-multiplier term) near the **top** of the tick,
    /// then recomputed by the tax phase **later** the same tick (it needs
    /// the freshly-aggregated coin multipliers → `produceTotal`). So the
    /// Phase-2 read sees the *prior* tick's value — a faithful one-tick
    /// lag mirroring the legacy mutable global. Legacy default
    /// `new Decimal('1')`.
    pub taxdivisor: Decimal,
}

impl Default for GCacheState {
    /// Matches the legacy `G.taxdivisor: new Decimal('1')` initial value.
    fn default() -> Self {
        Self {
            taxdivisor: Decimal::one(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_taxdivisor_is_one() {
        assert_eq!(GCacheState::default().taxdivisor, Decimal::one());
    }
}
