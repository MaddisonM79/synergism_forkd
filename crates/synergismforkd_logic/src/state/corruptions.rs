//! Corruptions state slice.
//!
//! Mirrors `player.corruptions.used` and `player.corruptions.next`
//! from the legacy schema. Backs [`crate::mechanics::corruptions`]
//! and is read by virtually every ascension-related formula.

/// One per-corruption-type loadout. Mirrors the 14 named corruption
/// fields on `player.corruptions.used` / `.next`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CorruptionLoadout {
    /// Active corruption levels indexed by corruption ID. Slot 0
    /// is "no corruption" and unused; legacy uses `1..=13`.
    pub levels: [u32; 14],
    /// Cached total ascension-score corruption multiplier — derived
    /// but held here so the tick layer doesn't recompute it every
    /// formula read.
    pub total_corruption_ascension_multiplier: f64,
}

/// Slice of `GameState` for corruption state.
#[derive(Debug, Clone, Copy, PartialEq)]
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
