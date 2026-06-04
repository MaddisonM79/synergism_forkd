//! Red-ambrosia state slice.
//!
//! Mirrors `player.redAmbrosia`, `player.lifetimeRedAmbrosia`,
//! `player.redAmbrosiaTime`, `player.redAmbrosiaRNG`,
//! `player.spentRedAmbrosia`, and `player.redAmbrosiaUpgrades`. Backs
//! [`crate::mechanics::red_ambrosia_bonuses`] and
//! [`crate::mechanics::red_ambrosia_upgrades`].

/// One red-ambrosia upgrade's per-player state. Mirrors
/// `player.redAmbrosiaUpgrades.<name>` — most red-ambrosia upgrades
/// are single-`level` entries (no free-level concept).
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct RedAmbrosiaUpgrade {
    /// Purchased level.
    pub level: f64,
}

/// Fixed cardinality of the red-ambrosia-upgrade array. Tier B item 12.
/// (27 fits inside serde's default 0..=32 length window — no `BigArray`
/// attribute needed.)
pub const RED_AMBROSIA_UPGRADES_LEN: usize = 27;

/// Slice of `GameState` for the red-ambrosia feature.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RedAmbrosiaState {
    /// `player.redAmbrosia` — current balance.
    pub red_ambrosia: f64,
    /// `player.lifetimeRedAmbrosia` — all-time earned.
    pub lifetime_red_ambrosia: f64,
    /// `player.redAmbrosiaTime` — generation-bar accumulator (seconds).
    pub red_ambrosia_time: f64,
    /// `G.redAmbrosiaTimer` — sub-bar 1/8 s granule accumulator that
    /// feeds `red_ambrosia_time`. Distinct from `red_ambrosia_time`.
    pub red_ambrosia_timer_g: f64,
    /// `player.spentRedAmbrosia` — count allocated to upgrades.
    pub spent_red_ambrosia: f64,
    /// Per-upgrade state. UI maintains the name ↔ index mapping.
    pub upgrades: [RedAmbrosiaUpgrade; RED_AMBROSIA_UPGRADES_LEN],
}

impl Default for RedAmbrosiaState {
    fn default() -> Self {
        Self {
            red_ambrosia: 0.0,
            lifetime_red_ambrosia: 0.0,
            red_ambrosia_time: 0.0,
            red_ambrosia_timer_g: 0.0,
            spent_red_ambrosia: 0.0,
            upgrades: [RedAmbrosiaUpgrade::default(); RED_AMBROSIA_UPGRADES_LEN],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_27_upgrade_slots() {
        let s = RedAmbrosiaState::default();
        assert_eq!(s.upgrades.len(), RED_AMBROSIA_UPGRADES_LEN);
    }
}
