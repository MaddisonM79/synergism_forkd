//! Ambrosia (blueberry) state slice.
//!
//! Mirrors `player.ambrosia`, `player.lifetimeAmbrosia`,
//! `player.blueberryTime`, `player.ambrosiaRNG`,
//! `player.spentBlueberries`, and `player.ambrosiaUpgrades`. Backs
//! [`crate::mechanics::ambrosia`] and
//! [`crate::mechanics::blueberry_upgrades`].

/// One ambrosia upgrade's per-player state. Mirrors
/// `player.ambrosiaUpgrades.<name>`.
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct AmbrosiaUpgrade {
    /// Purchased level.
    pub level: f64,
    /// Accumulated free levels.
    pub free_level: f64,
}

/// Fixed cardinality of the ambrosia-upgrade array. Tier B item 12.
pub const AMBROSIA_UPGRADES_LEN: usize = 35;

/// Slice of `GameState` for the ambrosia/blueberry feature.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AmbrosiaState {
    /// `player.ambrosia` — current balance (resets on use).
    pub ambrosia: f64,
    /// `player.lifetimeAmbrosia` — all-time ambrosia earned.
    pub lifetime_ambrosia: f64,
    /// `player.blueberryTime` — generation-bar accumulator (seconds).
    pub blueberry_time: f64,
    /// `G.ambrosiaTimer` — sub-bar 1/8 s granule accumulator that feeds
    /// `blueberry_time`. Distinct from `blueberry_time` itself.
    pub ambrosia_timer_g: f64,
    /// `player.ambrosiaRNG` — RNG seed for ambrosia-luck rolls.
    pub ambrosia_rng: f64,
    /// `player.spentBlueberries` — count of blueberries allocated
    /// to upgrades.
    pub spent_blueberries: f64,
    /// Per-upgrade state. UI maintains the name ↔ index mapping.
    #[serde(with = "BigArray")]
    pub upgrades: [AmbrosiaUpgrade; AMBROSIA_UPGRADES_LEN],
}

impl Default for AmbrosiaState {
    fn default() -> Self {
        Self {
            ambrosia: 0.0,
            lifetime_ambrosia: 0.0,
            blueberry_time: 0.0,
            ambrosia_timer_g: 0.0,
            ambrosia_rng: 0.0,
            spent_blueberries: 0.0,
            upgrades: [AmbrosiaUpgrade::default(); AMBROSIA_UPGRADES_LEN],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_35_upgrade_slots() {
        let s = AmbrosiaState::default();
        assert_eq!(s.upgrades.len(), AMBROSIA_UPGRADES_LEN);
        assert_eq!(s.lifetime_ambrosia, 0.0);
    }
}
