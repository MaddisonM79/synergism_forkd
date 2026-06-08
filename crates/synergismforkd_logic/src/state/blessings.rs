//! Cube/tesseract/hypercube blessing-value state slice (shared shape)
//! and the platonic blessing values (distinct shape).
//!
//! Mirrors `CubeBlessings` / `TesseractBlessings` / `HypercubeBlessings` /
//! `PlatonicBlessings` from the legacy TS
//! `packages/logic/src/state/schema.ts`. The first three share the same
//! 10-field shape in the legacy code; the Rust port collapses them into
//! one [`BlessingValues`] struct used by all three layers
//! ([`crate::mechanics::cube_blessings`], `tesseract_blessings`,
//! `hypercube_blessings`) — the composed `GameState` will hold three
//! independent instances, but the field set and the per-field semantics
//! are identical so a shared type avoids three near-identical clones.

/// Shared blessing-values shape — used as the read-only input bundle
/// for cube, tesseract, and hypercube blessing effect functions.
///
/// One field per effect that the blessing layer multiplies into. The
/// composed `GameState` holds one instance per layer (cube, tesseract,
/// hypercube), each independently advanced by the appropriate
/// reset/sacrifice mechanics.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub struct BlessingValues {
    /// Accelerator-blessing count.
    pub accelerator: f64,
    /// Multiplier-blessing count.
    pub multiplier: f64,
    /// Offering-blessing count.
    pub offering: f64,
    /// Rune-EXP-blessing count.
    pub rune_exp: f64,
    /// Obtainium-blessing count.
    pub obtainium: f64,
    /// Ant-speed-blessing count.
    pub ant_speed: f64,
    /// Ant-sacrifice-blessing count.
    pub ant_sacrifice: f64,
    /// Ant-ELO-blessing count.
    pub ant_elo: f64,
    /// Talisman-bonus-blessing count.
    pub talisman_bonus: f64,
    /// Global-speed-blessing count.
    pub global_speed: f64,
}

/// Platonic-blessing values — distinct from the 10-field shape because
/// platonic blessings cover a different set of effects.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub struct PlatonicBlessings {
    /// Cubes-blessing count.
    pub cubes: f64,
    /// Tesseracts-blessing count.
    pub tesseracts: f64,
    /// Hypercubes-blessing count.
    pub hypercubes: f64,
    /// Platonics-blessing count.
    pub platonics: f64,
    /// Hypercube-bonus blessing count.
    pub hypercube_bonus: f64,
    /// Taxes-blessing count.
    pub taxes: f64,
    /// Global-speed blessing count.
    pub global_speed: f64,
}

impl BlessingValues {
    /// Add a cube-open distribution in the legacy `cubeBlessings` key order
    /// (accelerator, multiplier, offering, runeExp, obtainium, antSpeed,
    /// antSacrifice, antELO, talismanBonus, globalSpeed) — the same order the
    /// open-distribution weight/pdf tables use.
    pub(crate) fn add_in_order(&mut self, increments: &[f64; 10]) {
        self.accelerator += increments[0];
        self.multiplier += increments[1];
        self.offering += increments[2];
        self.rune_exp += increments[3];
        self.obtainium += increments[4];
        self.ant_speed += increments[5];
        self.ant_sacrifice += increments[6];
        self.ant_elo += increments[7];
        self.talisman_bonus += increments[8];
        self.global_speed += increments[9];
    }

    /// Sum of all ten blessing counts — the legacy `sumContents(cubeBlessings)`
    /// used for the `1e300` tribute cap on cube opening.
    pub(crate) fn sum(&self) -> f64 {
        self.accelerator
            + self.multiplier
            + self.offering
            + self.rune_exp
            + self.obtainium
            + self.ant_speed
            + self.ant_sacrifice
            + self.ant_elo
            + self.talisman_bonus
            + self.global_speed
    }
}

impl PlatonicBlessings {
    /// Add an 8-slot platonic-open distribution in the legacy `platonicBlessings`
    /// key order (cubes, tesseracts, hypercubes, platonics, hypercubeBonus,
    /// taxes, **scoreBonus**, globalSpeed). The `scoreBonus` slot (index 6) is
    /// dropped: the legacy schema stores it but no effect reads it (the
    /// ascension-score reader reads `globalSpeed`), so this port omits the field
    /// — its distribution share is still computed (for faithful remainder
    /// accounting) and then discarded here.
    pub(crate) fn add_from_eight(&mut self, increments: &[f64; 8]) {
        self.cubes += increments[0];
        self.tesseracts += increments[1];
        self.hypercubes += increments[2];
        self.platonics += increments[3];
        self.hypercube_bonus += increments[4];
        self.taxes += increments[5];
        // increments[6] = scoreBonus — discarded (write-only dead field).
        self.global_speed += increments[7];
    }
}
