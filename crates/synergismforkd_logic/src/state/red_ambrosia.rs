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
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RedAmbrosiaUpgrade {
    /// Purchased level.
    pub level: f64,
}

/// Slice of `GameState` for the red-ambrosia feature.
#[derive(Debug, Clone, PartialEq)]
pub struct RedAmbrosiaState {
    /// `player.redAmbrosia` — current balance.
    pub red_ambrosia: f64,
    /// `player.lifetimeRedAmbrosia` — all-time earned.
    pub lifetime_red_ambrosia: f64,
    /// `player.redAmbrosiaTime` — generation-bar accumulator (seconds).
    pub red_ambrosia_time: f64,
    /// `player.redAmbrosiaRNG` — RNG seed.
    pub red_ambrosia_rng: f64,
    /// `player.spentRedAmbrosia` — count allocated to upgrades.
    pub spent_red_ambrosia: f64,
    /// Per-upgrade state. UI maintains the name ↔ index mapping.
    pub upgrades: Vec<RedAmbrosiaUpgrade>,
}

impl RedAmbrosiaState {
    /// Build with `n_upgrades` slots. Legacy synergism has 27 named
    /// red-ambrosia upgrades.
    #[must_use]
    pub fn new(n_upgrades: usize) -> Self {
        Self {
            red_ambrosia: 0.0,
            lifetime_red_ambrosia: 0.0,
            red_ambrosia_time: 0.0,
            red_ambrosia_rng: 0.0,
            spent_red_ambrosia: 0.0,
            upgrades: vec![RedAmbrosiaUpgrade::default(); n_upgrades],
        }
    }
}

impl Default for RedAmbrosiaState {
    fn default() -> Self {
        Self::new(27)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_27_upgrade_slots() {
        let s = RedAmbrosiaState::default();
        assert_eq!(s.upgrades.len(), 27);
    }
}
