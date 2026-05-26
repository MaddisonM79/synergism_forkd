//! Sliced game-state types. Each mechanic family owns a slice.
//!
//! The composed `GameState` lands once enough slices exist for it to be
//! meaningful — for now each slice stands alone, and mechanic functions
//! take only the slice they touch.

pub mod accelerator;
pub mod achievements;
pub mod ambrosia;
pub mod ants;
pub mod blessings;
pub mod campaigns;
pub mod challenges;
pub mod coin_counters;
pub mod corruptions;
pub mod crystal_upgrades;
pub mod cube_balances;
pub mod cube_upgrade_levels;
pub mod event_buffs;
pub mod golden_quarks;
pub mod hepteracts;
pub mod level;
pub mod multiplier;
pub mod octeract_upgrades;
pub mod particle_buildings;
pub mod producer;
pub mod quarks;
pub mod red_ambrosia;
pub mod researches;
pub mod reset_counters;
pub mod rng;
pub mod runes;
pub mod shop;
pub mod singularity;
pub mod talismans;
pub mod tesseract_buildings;
pub mod upgrades;

pub use accelerator::AcceleratorState;
pub use achievements::{AchievementsState, ProgressiveAchievementCache};
pub use ambrosia::{AmbrosiaState, AmbrosiaUpgrade};
pub use ants::{
    AntsState, AntsToggles, AutoSacrificeMode, PlayerAntMastery, PlayerAntProducer, RebornELOEntry,
};
pub use blessings::{BlessingValues, PlatonicBlessings};
pub use campaigns::CampaignsState;
pub use challenges::ChallengesState;
pub use coin_counters::CoinCountersState;
pub use corruptions::{CorruptionLoadout, CorruptionsState};
pub use crystal_upgrades::CrystalUpgradesState;
pub use cube_balances::CubeBalancesState;
pub use cube_upgrade_levels::CubeUpgradeLevelsState;
pub use event_buffs::EventBuffsState;
pub use golden_quarks::{GoldenQuarkUpgrade, GoldenQuarksState, StoredSpecialCostForm};
pub use hepteracts::{HepteractCraft, HepteractsState};
pub use level::LevelState;
pub use multiplier::MultiplierState;
pub use octeract_upgrades::{OcteractUpgrade, OcteractUpgradesState};
pub use particle_buildings::ParticleBuildingsState;
pub use producer::{BuyAmount, ProducerFamilyState};
pub use quarks::QuarksState;
pub use red_ambrosia::{RedAmbrosiaState, RedAmbrosiaUpgrade};
pub use researches::ResearchesState;
pub use reset_counters::ResetCountersState;
pub use rng::{RngPurpose, RngState};
pub use runes::{RunesState, RUNE_COUNT};
pub use shop::{ShopBuyMaxMode, ShopState};
pub use singularity::{SingularityChallengeState, SingularityState};
pub use talismans::{TalismanRuneAssignment, TalismansState, TALISMAN_COUNT};
pub use tesseract_buildings::{AscendBuildingState, TesseractBuildingsState};
pub use upgrades::UpgradesState;
