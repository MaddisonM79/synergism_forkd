//! Tesseract (ascension-tier) building state slice.
//!
//! Mirrors `TesseractBuildingsState` + `AscendBuildingState` from the
//! legacy TS `packages/logic/src/state/schema.ts`. All resources here
//! are plain `f64` — `Decimal` isn't needed because buying caps out
//! long before `1e308` per the legacy comment.

/// One position of the ascension-tier building family. Subset of the
/// legacy `player.ascendBuildingN` shape — only the fields the buy
/// machinery touches; generated/multiplier stay in the UI tier until
/// those mechanics migrate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AscendBuildingState {
    /// Count owned.
    pub owned: f64,
    /// Cached cost of the next building.
    pub cost: f64,
}

/// Slice of `GameState` read/written by the tesseract-building-purchase
/// machinery. `wow_tesseracts` is the spend resource (mirrored as an
/// `f64` via `Number(player.wowTesseracts)` at the boundary — the
/// `WowTesseracts` wrapper class stays in the UI tier).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TesseractBuildingsState {
    /// Spend resource — the player's `wowTesseracts` balance.
    pub wow_tesseracts: f64,
    /// Tier-1 ascend building.
    pub ascend_building_1: AscendBuildingState,
    /// Tier-2 ascend building.
    pub ascend_building_2: AscendBuildingState,
    /// Tier-3 ascend building.
    pub ascend_building_3: AscendBuildingState,
    /// Tier-4 ascend building.
    pub ascend_building_4: AscendBuildingState,
    /// Tier-5 ascend building.
    pub ascend_building_5: AscendBuildingState,
}

impl TesseractBuildingsState {
    /// Read a tier's slice by index (1..=5).
    #[must_use]
    pub fn building(&self, index: u8) -> AscendBuildingState {
        debug_assert!(
            matches!(index, 1..=5),
            "tesseract building index out of range: {index}"
        );
        match index {
            1 => self.ascend_building_1,
            2 => self.ascend_building_2,
            3 => self.ascend_building_3,
            4 => self.ascend_building_4,
            _ => self.ascend_building_5,
        }
    }

    /// Write a tier's slice by index (1..=5).
    pub fn set_building(&mut self, index: u8, value: AscendBuildingState) {
        debug_assert!(
            matches!(index, 1..=5),
            "tesseract building index out of range: {index}"
        );
        match index {
            1 => self.ascend_building_1 = value,
            2 => self.ascend_building_2 = value,
            3 => self.ascend_building_3 = value,
            4 => self.ascend_building_4 = value,
            _ => self.ascend_building_5 = value,
        }
    }
}
