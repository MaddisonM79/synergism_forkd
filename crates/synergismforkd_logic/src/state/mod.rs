//! Sliced game-state types. Each mechanic family owns a slice.
//!
//! The composed `GameState` lands once enough slices exist for it to be
//! meaningful — for now each slice stands alone, and mechanic functions
//! take only the slice they touch.

pub mod accelerator;
pub mod blessings;
pub mod crystal_upgrades;
pub mod multiplier;
pub mod particle_buildings;
pub mod producer;
pub mod upgrades;

pub use accelerator::AcceleratorState;
pub use blessings::{BlessingValues, PlatonicBlessings};
pub use crystal_upgrades::CrystalUpgradesState;
pub use multiplier::MultiplierState;
pub use particle_buildings::ParticleBuildingsState;
pub use producer::{BuyAmount, ProducerFamilyState};
pub use upgrades::UpgradesState;
