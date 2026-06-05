//! Talismans state slice.
//!
//! Mirrors `player.talismanLevels`, `player.talismanRarity`, the
//! per-talisman fragment-allocation arrays, and the talisman shard
//! / fragment balances. Backs [`crate::mechanics::talisman_costs`],
//! [`crate::mechanics::talisman_levels`], and
//! [`crate::mechanics::talisman_effects`].

use serde::{Deserialize, Serialize};

/// Number of talismans in the legacy synergism build. Seven named:
/// Exemption, Chronos, Midas, Metaphysics, Polymath, Mortuus, Plastic.
/// The order matches the legacy `talismans` const
/// (`legacy/core_split/packages/web_ui/src/Talismans.ts`); index `i` is
/// the i-th talisman, and the `TALISMAN_*` constants below give the
/// name → index mapping the UI tier must match.
pub const TALISMAN_COUNT: usize = 7;

/// `exemption` — index 0.
pub const TALISMAN_EXEMPTION: usize = 0;
/// `chronos` — index 1.
pub const TALISMAN_CHRONOS: usize = 1;
/// `midas` — index 2.
pub const TALISMAN_MIDAS: usize = 2;
/// `metaphysics` — index 3.
pub const TALISMAN_METAPHYSICS: usize = 3;
/// `polymath` — index 4.
pub const TALISMAN_POLYMATH: usize = 4;
/// `mortuus` — index 5.
pub const TALISMAN_MORTUUS: usize = 5;
/// `plastic` — index 6.
pub const TALISMAN_PLASTIC: usize = 6;

/// Per-talisman fragment-allocation state. Mirrors the legacy
/// `player.talismanOne..Seven` arrays: a small fixed slot list
/// describing which rune the talisman buffs at the current rarity.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct TalismanRuneAssignment {
    /// `[boolean, 0|1|2|3|4|5]` in legacy. The bool is whether the
    /// slot is allocated; the u8 (`0..=5`) picks which rune.
    pub allocated: bool,
    /// Rune index this slot buffs (`0..=5`, or `0` when unallocated).
    pub rune_id: u8,
}

/// Slice of `GameState` for the talisman feature.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TalismansState {
    /// `player.talismanLevels[0..=6]` — per-talisman level.
    pub talisman_levels: [f64; TALISMAN_COUNT],
    /// `player.talismanRarity[0..=6]` — per-talisman rarity tier.
    pub talisman_rarity: [f64; TALISMAN_COUNT],
    /// Per-talisman rune-allocation slots. Legacy uses 5 slots per
    /// talisman: `[Boolean, 0..=5]` per slot.
    pub rune_assignments: [[TalismanRuneAssignment; 5]; TALISMAN_COUNT],
    /// `player.talismanShards` — shard balance.
    pub talisman_shards: f64,
    /// `player.commonFragments`.
    pub common_fragments: f64,
    /// `player.uncommonFragments`.
    pub uncommon_fragments: f64,
    /// `player.rareFragments`.
    pub rare_fragments: f64,
    /// `player.epicFragments`.
    pub epic_fragments: f64,
    /// `player.legendaryFragments`.
    pub legendary_fragments: f64,
    /// `player.mythicalFragments`.
    pub mythical_fragments: f64,
}

impl Default for TalismansState {
    fn default() -> Self {
        Self {
            talisman_levels: [0.0; TALISMAN_COUNT],
            talisman_rarity: [0.0; TALISMAN_COUNT],
            rune_assignments: [[TalismanRuneAssignment::default(); 5]; TALISMAN_COUNT],
            talisman_shards: 0.0,
            common_fragments: 0.0,
            uncommon_fragments: 0.0,
            rare_fragments: 0.0,
            epic_fragments: 0.0,
            legendary_fragments: 0.0,
            mythical_fragments: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_7_talismans_with_5_slots_each() {
        let s = TalismansState::default();
        assert_eq!(s.talisman_levels.len(), 7);
        assert_eq!(s.rune_assignments[0].len(), 5);
    }

    #[test]
    fn talisman_index_convention_sentinels() {
        // Coverage: the last named slot pins the array length.
        assert_eq!(TALISMAN_PLASTIC, TALISMAN_COUNT - 1);
        // StatLine anchors (legacy `talismans` const order).
        assert_eq!(TALISMAN_EXEMPTION, 0);
        assert_eq!(TALISMAN_CHRONOS, 1);
        assert_eq!(TALISMAN_POLYMATH, 4);
    }
}
