//! Runes state slice.
//!
//! Mirrors `player.runelevels`, `player.runeexp`, `player.runeShards`,
//! `player.runeBlessingLevels`, and `player.runeSpiritLevels`. Backs
//! [`crate::mechanics::rune_levels`], [`crate::mechanics::rune_exp_multiplier`],
//! [`crate::mechanics::rune_upgrade_progression`],
//! [`crate::mechanics::rune_effects`],
//! [`crate::mechanics::rune_blessing_effects`],
//! [`crate::mechanics::rune_spirit_effects`], and
//! [`crate::mechanics::rune_level_bonuses`].

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Number of rune slots. Legacy synergism has 7: speed,
/// duplication, prism, thrift, superiorIntellect, antiquities,
/// finiteDescent.
pub const RUNE_COUNT: usize = 7;

/// Index of the Speed rune in [`RunesState::rune_levels`] etc.
pub const RUNE_SPEED: usize = 0;
/// Index of the Duplication rune.
pub const RUNE_DUPLICATION: usize = 1;
/// Index of the Prism rune.
pub const RUNE_PRISM: usize = 2;
/// Index of the Thrift rune.
pub const RUNE_THRIFT: usize = 3;
/// Index of the Superior Intellect rune.
pub const RUNE_SUPERIOR_INTELLECT: usize = 4;
/// Index of the Antiquities rune (5th-prestige rune).
pub const RUNE_ANTIQUITIES: usize = 5;
/// Index of the Finite Descent rune (6th-prestige rune).
pub const RUNE_FINITE_DESCENT: usize = 6;

/// Slice of `GameState` for rune levels + XP + blessings + spirits
/// + the rune-shards spend resource.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RunesState {
    /// Per-rune level. Indexed `0..=6` to match the legacy rune
    /// enum order.
    pub rune_levels: [f64; RUNE_COUNT],
    /// Per-rune EXP accumulator. Indices match `rune_levels`.
    pub rune_exp: [f64; RUNE_COUNT],
    /// `player.runeShards` — currency spent to level runes.
    pub rune_shards: Decimal,
    /// Per-rune blessing level (`player.runeBlessingLevels`).
    pub rune_blessing_levels: [f64; RUNE_COUNT],
    /// Per-rune spirit level (`player.runeSpiritLevels`).
    pub rune_spirit_levels: [f64; RUNE_COUNT],
    /// Per-rune cached "free level" bonuses accumulated from
    /// talismans / ant upgrades / etc. (kept here so the tick
    /// layer doesn't recompute every read).
    pub rune_free_levels: [f64; RUNE_COUNT],
}

impl Default for RunesState {
    fn default() -> Self {
        Self {
            rune_levels: [0.0; RUNE_COUNT],
            rune_exp: [0.0; RUNE_COUNT],
            rune_shards: Decimal::zero(),
            rune_blessing_levels: [0.0; RUNE_COUNT],
            rune_spirit_levels: [0.0; RUNE_COUNT],
            rune_free_levels: [0.0; RUNE_COUNT],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_7_rune_slots() {
        let s = RunesState::default();
        assert_eq!(s.rune_levels.len(), 7);
        assert_eq!(s.rune_blessing_levels.len(), 7);
    }
}
