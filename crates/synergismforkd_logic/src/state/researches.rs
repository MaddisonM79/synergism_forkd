//! Researches state slice.
//!
//! Mirrors the `player.researches`, `player.researchPoints`, and
//! `player.obtainium` fields. Backs [`crate::mechanics::researches`]
//! and is read by many of the per-tick aggregators.

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` read/written by research mechanics.
///
/// The legacy `researches` array is 1-indexed with index 0 unused;
/// this slice preserves that — callers pass `1..=N` and the vec
/// holds the value at the matching position. Length is taken at
/// runtime from the data table.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ResearchesState {
    /// Per-research level. 1-indexed (index 0 is unused and held
    /// at `0` to preserve the legacy shape).
    pub researches: Vec<f64>,
    /// Unspent obtainium — the spend resource.
    pub obtainium: Decimal,
    /// All-time research points earned (for stat tracking).
    pub research_points: f64,
}

impl ResearchesState {
    /// Build with `n_researches + 1` slots (the `+1` preserves the
    /// 1-indexed legacy shape with index 0 unused).
    #[must_use]
    pub fn new(n_researches: usize) -> Self {
        Self {
            researches: vec![0.0; n_researches + 1],
            obtainium: Decimal::zero(),
            research_points: 0.0,
        }
    }
}

impl Default for ResearchesState {
    fn default() -> Self {
        // Legacy synergism has 200 researches; allocate 201 slots
        // (index 0 unused).
        Self::new(200)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_201_slots() {
        let s = ResearchesState::default();
        assert_eq!(s.researches.len(), 201);
        assert_eq!(s.obtainium.to_number(), 0.0);
    }

    #[test]
    fn new_with_custom_count() {
        let s = ResearchesState::new(10);
        assert_eq!(s.researches.len(), 11);
    }
}
