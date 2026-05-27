//! Researches state slice.
//!
//! Mirrors the `player.researches`, `player.researchPoints`, and
//! `player.obtainium` fields. Backs [`crate::mechanics::researches`]
//! and is read by many of the per-tick aggregators.

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use synergismforkd_bignum::Decimal;

/// Fixed cardinality of the researches array — `200 + 1` for the
/// legacy 1-indexed convention (index 0 unused). Tier B item 12 /
/// Anvil F4.
pub const RESEARCHES_LEN: usize = 201;

/// Slice of `GameState` read/written by research mechanics.
///
/// The legacy `researches` array is 1-indexed with index 0 unused;
/// this slice preserves that — callers pass `1..=N` and the array
/// holds the value at the matching position.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ResearchesState {
    /// Per-research level. 1-indexed (index 0 is unused and held
    /// at `0` to preserve the legacy shape).
    #[serde(with = "BigArray")]
    pub researches: [f64; RESEARCHES_LEN],
    /// Unspent obtainium — the spend resource.
    pub obtainium: Decimal,
    /// All-time research points earned (for stat tracking).
    pub research_points: f64,
}

impl Default for ResearchesState {
    fn default() -> Self {
        Self {
            researches: [0.0; RESEARCHES_LEN],
            obtainium: Decimal::zero(),
            research_points: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_201_slots() {
        let s = ResearchesState::default();
        assert_eq!(s.researches.len(), RESEARCHES_LEN);
        assert_eq!(s.obtainium.to_number(), 0.0);
    }

    #[test]
    fn default_obtainium_is_zero() {
        let s = ResearchesState::default();
        assert_eq!(s.obtainium, Decimal::zero());
        assert_eq!(s.research_points, 0.0);
    }
}
