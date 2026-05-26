//! Ambrosia (blueberry) state slice.
//!
//! Mirrors `player.ambrosia`, `player.lifetimeAmbrosia`,
//! `player.blueberryTime`, `player.ambrosiaRNG`,
//! `player.spentBlueberries`, and `player.ambrosiaUpgrades`. Backs
//! [`crate::mechanics::ambrosia`] and
//! [`crate::mechanics::blueberry_upgrades`].

/// One ambrosia upgrade's per-player state. Mirrors
/// `player.ambrosiaUpgrades.<name>`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AmbrosiaUpgrade {
    /// Purchased level.
    pub level: f64,
    /// Accumulated free levels.
    pub free_level: f64,
}

/// Slice of `GameState` for the ambrosia/blueberry feature.
#[derive(Debug, Clone, PartialEq)]
pub struct AmbrosiaState {
    /// `player.ambrosia` — current balance (resets on use).
    pub ambrosia: f64,
    /// `player.lifetimeAmbrosia` — all-time ambrosia earned.
    pub lifetime_ambrosia: f64,
    /// `player.blueberryTime` — generation-bar accumulator (seconds).
    pub blueberry_time: f64,
    /// `player.ambrosiaRNG` — RNG seed for ambrosia-luck rolls.
    pub ambrosia_rng: f64,
    /// `player.spentBlueberries` — count of blueberries allocated
    /// to upgrades.
    pub spent_blueberries: f64,
    /// Per-upgrade state. UI maintains the name ↔ index mapping.
    pub upgrades: Vec<AmbrosiaUpgrade>,
}

impl AmbrosiaState {
    /// Build with `n_upgrades` slots. Legacy synergism has ~35
    /// named ambrosia upgrades.
    #[must_use]
    pub fn new(n_upgrades: usize) -> Self {
        Self {
            ambrosia: 0.0,
            lifetime_ambrosia: 0.0,
            blueberry_time: 0.0,
            ambrosia_rng: 0.0,
            spent_blueberries: 0.0,
            upgrades: vec![AmbrosiaUpgrade::default(); n_upgrades],
        }
    }
}

impl Default for AmbrosiaState {
    fn default() -> Self {
        Self::new(35)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_35_upgrade_slots() {
        let s = AmbrosiaState::default();
        assert_eq!(s.upgrades.len(), 35);
        assert_eq!(s.lifetime_ambrosia, 0.0);
    }
}
