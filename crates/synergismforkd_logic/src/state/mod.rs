//! Sliced game-state types. Each mechanic family owns a slice.
//!
//! The composed `GameState` lands once enough slices exist for it to be
//! meaningful — for now each slice stands alone, and mechanic functions
//! take only the slice they touch.

pub mod accelerator;
pub mod achievements;
pub mod ants;
pub mod blessings;
pub mod challenges;
pub mod crystal_upgrades;
pub mod multiplier;
pub mod particle_buildings;
pub mod producer;
pub mod researches;
pub mod tesseract_buildings;
pub mod upgrades;

pub use accelerator::AcceleratorState;
pub use achievements::{AchievementsState, ProgressiveAchievementCache};
pub use ants::{
    AntsState, AntsToggles, AutoSacrificeMode, PlayerAntMastery, PlayerAntProducer, RebornELOEntry,
};
pub use blessings::{BlessingValues, PlatonicBlessings};
pub use challenges::ChallengesState;
pub use crystal_upgrades::CrystalUpgradesState;
pub use multiplier::MultiplierState;
pub use particle_buildings::ParticleBuildingsState;
pub use producer::{BuyAmount, ProducerFamilyState};
pub use researches::ResearchesState;
pub use tesseract_buildings::{AscendBuildingState, TesseractBuildingsState};
pub use upgrades::UpgradesState;
