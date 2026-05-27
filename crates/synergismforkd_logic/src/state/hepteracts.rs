//! Hepteracts state slice.
//!
//! Mirrors `player.hepteractCrafts.<name>` (8 craft types ×
//! `{BAL, CAP, BASE_CAP, AUTO, UNLOCKED}`) plus the overflux orb +
//! powder balances. Backs [`crate::mechanics::hepteract_values`],
//! [`crate::mechanics::hepteract_effects`], and the overflux-bonus
//! mechanics.

/// One hepteract-craft entry. Mirrors a `player.hepteractCrafts.X`
/// shape in the legacy schema.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct HepteractCraft {
    /// `BAL` — current balance for this craft.
    pub bal: f64,
    /// `CAP` — current cap.
    pub cap: f64,
    /// `BASE_CAP` — baseline cap before expansions.
    pub base_cap: f64,
    /// `AUTO` — autocraft enabled.
    pub auto: bool,
    /// `UNLOCKED` — feature gate.
    pub unlocked: bool,
}

/// Slice of `GameState` for hepteract crafts + overflux currency.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct HepteractsState {
    /// `chronos` craft state.
    pub chronos: HepteractCraft,
    /// `hyperrealism` craft state.
    pub hyperrealism: HepteractCraft,
    /// `quark` craft state.
    pub quark: HepteractCraft,
    /// `challenge` craft state.
    pub challenge: HepteractCraft,
    /// `abyss` craft state.
    pub abyss: HepteractCraft,
    /// `accelerator` craft state.
    pub accelerator: HepteractCraft,
    /// `acceleratorBoost` craft state.
    pub accelerator_boost: HepteractCraft,
    /// `multiplier` craft state.
    pub multiplier: HepteractCraft,
    /// `player.overfluxOrbs` — orb balance.
    pub overflux_orbs: f64,
    /// `player.overfluxPowder` — powder balance.
    pub overflux_powder: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_all_zero_and_locked() {
        let s = HepteractsState::default();
        assert_eq!(s.chronos.bal, 0.0);
        assert!(!s.chronos.unlocked);
        assert_eq!(s.overflux_orbs, 0.0);
    }
}
