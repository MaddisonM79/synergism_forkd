//! Ants state slice.
//!
//! Mirrors `PlayerAnts` from
//! `legacy_core_split/packages/web_ui/src/Features/Ants/structs/structs.ts`.
//! Backs every formula in [`crate::mechanics::ant_producers`],
//! [`crate::mechanics::ant_masteries`],
//! [`crate::mechanics::ant_upgrade_levels`],
//! [`crate::mechanics::ant_upgrades`],
//! [`crate::mechanics::ant_reborn_elo`],
//! [`crate::mechanics::ant_sacrifice_reward_calc`], and
//! [`crate::mechanics::ant_sacrifice_rewards`].

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// One ant-producer's per-player state. Mirrors the legacy
/// `PlayerAntProducers` record entry.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct PlayerAntProducer {
    /// `purchased` — number of this producer bought.
    pub purchased: f64,
    /// `generated` — number of this producer auto-generated from
    /// the next-tier producer's production.
    pub generated: Decimal,
}

/// One ant-mastery's per-player state. Mirrors the legacy
/// `PlayerAntMasteries` record entry.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct PlayerAntMastery {
    /// Current mastery level (`0..=12`).
    pub mastery: u8,
    /// All-time highest mastery level reached (`0..=12`).
    pub highest_mastery: u8,
}

/// Single entry on the reborn-ELO leaderboard.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct RebornELOEntry {
    /// Effective ant ELO at the sacrifice.
    pub elo: f64,
    /// Sacrifice ID that produced the entry — useful for
    /// timestamping in the UI.
    pub sacrifice_id: u32,
}

/// Auto-sacrifice mode selector. Mirrors the legacy
/// `AutoSacrificeModes` enum.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoSacrificeMode {
    /// Auto-sacrifice disabled.
    Off,
    /// Trigger when reborn ELO crosses the threshold.
    ELO,
    /// Trigger on a fixed timer.
    Timer,
    /// Trigger after every N sacrifices.
    Sacrifice,
}

/// Ant-feature toggles. Mirrors the legacy `toggles` field on
/// `PlayerAnts`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct AntsToggles {
    /// Autobuy producers on tick.
    pub autobuy_producers: bool,
    /// Autobuy masteries when affordable.
    pub autobuy_masteries: bool,
    /// Autobuy ant upgrades when affordable.
    pub autobuy_upgrades: bool,
    /// Use the "max buy" mode for producers.
    pub max_buy_producers: bool,
    /// Use the "max buy" mode for upgrades.
    pub max_buy_upgrades: bool,
    /// Auto-sacrifice is enabled.
    pub auto_sacrifice_enabled: bool,
    /// Threshold value (interpretation depends on
    /// `auto_sacrifice_mode`).
    pub auto_sacrifice_threshold: f64,
    /// Active auto-sacrifice mode.
    pub auto_sacrifice_mode: AutoSacrificeMode,
    /// Always sacrifice at max reborn ELO, regardless of threshold.
    pub always_sacrifice_max_reborn_elo: bool,
    /// Only sacrifice when at max reborn ELO.
    pub only_sacrifice_max_reborn_elo: bool,
}

impl Default for AntsToggles {
    fn default() -> Self {
        Self {
            autobuy_producers: false,
            autobuy_masteries: false,
            autobuy_upgrades: false,
            max_buy_producers: false,
            max_buy_upgrades: false,
            auto_sacrifice_enabled: false,
            auto_sacrifice_threshold: 0.0,
            auto_sacrifice_mode: AutoSacrificeMode::Off,
            always_sacrifice_max_reborn_elo: false,
            only_sacrifice_max_reborn_elo: false,
        }
    }
}

/// Slice of `GameState` read/written by the ant mechanics.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AntsState {
    /// Per-producer state, indexed `0..=8` (Workers .. HolySpirit).
    pub producers: [PlayerAntProducer; 9],
    /// Per-mastery state, indexed `0..=8`.
    pub masteries: [PlayerAntMastery; 9],
    /// Per-upgrade level, indexed `0..=15`.
    pub upgrades: [f64; 16],
    /// Current crumb balance — the ant-tier currency.
    pub crumbs: Decimal,
    /// Crumbs accumulated during the current sacrifice.
    pub crumbs_this_sacrifice: Decimal,
    /// All-time crumbs earned.
    pub crumbs_ever_made: Decimal,
    /// `immortalELO` — peak effective ELO ever achieved.
    pub immortal_elo: f64,
    /// `rebornELO` — current ELO that contributes to the reward
    /// stages / leaderboard.
    pub reborn_elo: f64,
    /// Top-N daily reborn-ELO leaderboard. Capped at
    /// `LEADERBOARD_WEIGHTS.len()` entries (5) by the calculator —
    /// `SmallVec<[_; 5]>` keeps the storage inline so the slice clones
    /// without a heap touch.
    pub highest_reborn_elo_daily: smallvec::SmallVec<[RebornELOEntry; 5]>,
    /// Top-N all-time reborn-ELO leaderboard. Same cap, same inline
    /// storage.
    pub highest_reborn_elo_ever: smallvec::SmallVec<[RebornELOEntry; 5]>,
    /// Total quarks earned from ant sacrifices.
    pub quarks_gained_from_ants: f64,
    /// All-time count of ant sacrifices performed.
    pub ant_sacrifice_count: f64,
    /// Sacrifice-ID generator — monotonically increasing.
    pub current_sacrifice_id: u32,
    /// Auto-buy + auto-sacrifice toggles.
    pub toggles: AntsToggles,
}

impl Default for PlayerAntProducer {
    fn default() -> Self {
        Self {
            purchased: 0.0,
            generated: Decimal::zero(),
        }
    }
}

impl Default for AntsState {
    fn default() -> Self {
        Self {
            producers: [PlayerAntProducer::default(); 9],
            masteries: [PlayerAntMastery::default(); 9],
            upgrades: [0.0; 16],
            crumbs: Decimal::zero(),
            crumbs_this_sacrifice: Decimal::zero(),
            crumbs_ever_made: Decimal::zero(),
            immortal_elo: 0.0,
            reborn_elo: 0.0,
            highest_reborn_elo_daily: smallvec::SmallVec::new(),
            highest_reborn_elo_ever: smallvec::SmallVec::new(),
            quarks_gained_from_ants: 0.0,
            ant_sacrifice_count: 0.0,
            current_sacrifice_id: 0,
            toggles: AntsToggles::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_starts_empty() {
        let s = AntsState::default();
        assert_eq!(s.producers[0].purchased, 0.0);
        assert_eq!(s.masteries[0].mastery, 0);
        assert_eq!(s.upgrades[0], 0.0);
        assert_eq!(s.crumbs.to_number(), 0.0);
        assert!(s.highest_reborn_elo_daily.is_empty());
    }

    #[test]
    fn producers_array_has_9_slots() {
        let s = AntsState::default();
        assert_eq!(s.producers.len(), 9);
        assert_eq!(s.masteries.len(), 9);
    }

    #[test]
    fn upgrades_array_has_16_slots() {
        let s = AntsState::default();
        assert_eq!(s.upgrades.len(), 16);
    }

    #[test]
    fn toggles_default_off() {
        let t = AntsToggles::default();
        assert!(!t.autobuy_producers);
        assert!(!t.auto_sacrifice_enabled);
        assert_eq!(t.auto_sacrifice_mode, AutoSacrificeMode::Off);
    }
}
