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
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
