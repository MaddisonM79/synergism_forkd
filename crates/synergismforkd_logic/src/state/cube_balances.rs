//! Cube/tesseract/hypercube/platonic/hepteract/abyssal balance
//! state slice.
//!
//! Holds the player's current count of each cube-tier currency.
//! Backs the `cube_upgrades`, `platonic_upgrade_costs`, and tier-
//! blessing mechanics, as well as the per-ascension reward
//! application in [`crate::mechanics::calculate::calc_corruption_stuff`].

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` holding the cube-tier currency balances.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CubeBalancesState {
    /// `player.wowCubes` — base cube currency.
    pub wow_cubes: Decimal,
    /// `player.wowTesseracts` — second-tier cube currency.
    pub wow_tesseracts: Decimal,
    /// `player.wowHypercubes` — third-tier.
    pub wow_hypercubes: Decimal,
    /// `player.wowPlatonicCubes` — fourth-tier.
    pub wow_platonic_cubes: Decimal,
    /// `player.wowHepteracts` — fifth-tier.
    pub wow_hepteracts: Decimal,
    /// `player.wowOcteracts` — sixth-tier.
    pub wow_octeracts: Decimal,
    /// `player.totalWowOcteracts` — lifetime octeract earnings (drives
    /// the totalOcteractBonus formulas).
    pub total_wow_octeracts: Decimal,
    /// `player.wowAbyssals` — special-tier currency awarded by the
    /// `taxmanLastStand` Exalt.
    pub wow_abyssals: f64,
    /// "This ascension" counters — useful for live UI display.
    /// Reset at ascension start.
    pub cubes_this_ascension: Decimal,
    /// `tesseracts_this_ascension` — see [`Self::cubes_this_ascension`].
    pub tesseracts_this_ascension: Decimal,
    /// `hypercubes_this_ascension` — see [`Self::cubes_this_ascension`].
    pub hypercubes_this_ascension: Decimal,
    /// `platonic_cubes_this_ascension` — see [`Self::cubes_this_ascension`].
    pub platonic_cubes_this_ascension: Decimal,
    /// `hepteracts_this_ascension` — see [`Self::cubes_this_ascension`].
    pub hepteracts_this_ascension: Decimal,
    // ── Daily cube-opening counters (quark-from-opening bookkeeping) ──────
    // Reset on a new real-life day (the daily reset is a UI/time-tier
    // concern, not yet ported); `open()` accumulates the cubes opened and
    // the quarks already awarded today, so each open awards only the delta.
    /// `player.cubeOpenedDaily` — cubes opened today.
    pub cube_opened_daily: f64,
    /// `player.cubeQuarkDaily` — quarks already awarded from cube opens today.
    pub cube_quark_daily: f64,
    /// `player.tesseractOpenedDaily`.
    pub tesseract_opened_daily: f64,
    /// `player.tesseractQuarkDaily`.
    pub tesseract_quark_daily: f64,
    /// `player.hypercubeOpenedDaily`.
    pub hypercube_opened_daily: f64,
    /// `player.hypercubeQuarkDaily`.
    pub hypercube_quark_daily: f64,
    /// `player.platonicCubeOpenedDaily`.
    pub platonic_cube_opened_daily: f64,
    /// `player.platonicCubeQuarkDaily`.
    pub platonic_cube_quark_daily: f64,
}

impl Default for CubeBalancesState {
    fn default() -> Self {
        Self {
            wow_cubes: Decimal::zero(),
            wow_tesseracts: Decimal::zero(),
            wow_hypercubes: Decimal::zero(),
            wow_platonic_cubes: Decimal::zero(),
            wow_hepteracts: Decimal::zero(),
            wow_octeracts: Decimal::zero(),
            total_wow_octeracts: Decimal::zero(),
            wow_abyssals: 0.0,
            cubes_this_ascension: Decimal::zero(),
            tesseracts_this_ascension: Decimal::zero(),
            hypercubes_this_ascension: Decimal::zero(),
            platonic_cubes_this_ascension: Decimal::zero(),
            hepteracts_this_ascension: Decimal::zero(),
            cube_opened_daily: 0.0,
            cube_quark_daily: 0.0,
            tesseract_opened_daily: 0.0,
            tesseract_quark_daily: 0.0,
            hypercube_opened_daily: 0.0,
            hypercube_quark_daily: 0.0,
            platonic_cube_opened_daily: 0.0,
            platonic_cube_quark_daily: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_zero() {
        let s = CubeBalancesState::default();
        assert_eq!(s.wow_cubes.to_number(), 0.0);
        assert_eq!(s.wow_abyssals, 0.0);
        assert_eq!(s.total_wow_octeracts.to_number(), 0.0);
    }
}
