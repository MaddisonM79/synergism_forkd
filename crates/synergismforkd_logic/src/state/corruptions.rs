//! Corruptions state slice.
//!
//! Mirrors `player.corruptions.used` and `player.corruptions.next`
//! from the legacy schema. Backs [`crate::mechanics::corruptions`]
//! and is read by virtually every ascension-related formula.
//!
//! Corruption-name indexes match the legacy `CorruptionIndices` enum
//! (`viscosity = 0`, …, `recession = 7`). The first 8 slots are the
//! eight named corruptions; the trailing slots are spare for future
//! corruptions added by mechanic-tier extensions.

/// Index of the "viscosity" corruption (`CorruptionIndices.viscosity`).
use serde::{Deserialize, Serialize};

pub const VISCOSITY_INDEX: usize = 0;
/// Index of the "dilation" corruption (`CorruptionIndices.dilation`).
pub const DILATION_INDEX: usize = 1;
/// Index of the "hyperchallenge" corruption (`CorruptionIndices.hyperchallenge`).
pub const HYPERCHALLENGE_INDEX: usize = 2;
/// Index of the "illiteracy" corruption (`CorruptionIndices.illiteracy`).
pub const ILLITERACY_INDEX: usize = 3;
/// Index of the "deflation" corruption (`CorruptionIndices.deflation`).
pub const DEFLATION_INDEX: usize = 4;
/// Index of the "extinction" corruption (`CorruptionIndices.extinction`).
pub const EXTINCTION_INDEX: usize = 5;
/// Index of the "drought" corruption (`CorruptionIndices.drought`).
pub const DROUGHT_INDEX: usize = 6;
/// Index of the "recession" corruption (`CorruptionIndices.recession`).
pub const RECESSION_INDEX: usize = 7;

/// One per-corruption-type loadout. Mirrors the 14 named corruption
/// fields on `player.corruptions.used` / `.next`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct CorruptionLoadout {
    /// Active corruption levels indexed by the legacy `CorruptionIndices`
    /// values — see the module-level `*_INDEX` constants. Slots 0..=7
    /// are the eight named corruptions; 8..=13 are spare.
    pub levels: [u32; 14],
    /// Cached total ascension-score corruption multiplier — derived
    /// but held here so the tick layer doesn't recompute it every
    /// formula read.
    pub total_corruption_ascension_multiplier: f64,
}

/// Slice of `GameState` for corruption state.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct CorruptionsState {
    /// Currently-applied corruption loadout (drives this-ascension
    /// formulas).
    pub used: CorruptionLoadout,
    /// Pending loadout — applies on next ascension.
    pub next: CorruptionLoadout,
    /// `corruptionShownStats` — UI display preference. Plain bool
    /// here for parity; the UI sources it.
    pub corruption_shown_stats: bool,
}

impl Default for CorruptionsState {
    fn default() -> Self {
        let baseline = CorruptionLoadout {
            total_corruption_ascension_multiplier: 1.0,
            ..CorruptionLoadout::default()
        };
        Self {
            used: baseline,
            next: baseline,
            corruption_shown_stats: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_zero_levels() {
        let s = CorruptionsState::default();
        assert_eq!(s.used.levels, [0; 14]);
        assert_eq!(s.next.levels, [0; 14]);
    }

    #[test]
    fn default_corruption_mult_is_1() {
        let s = CorruptionsState::default();
        assert_eq!(s.used.total_corruption_ascension_multiplier, 1.0);
    }
}
