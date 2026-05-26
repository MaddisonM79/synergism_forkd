//! Generic-upgrades state slice.
//!
//! Mirrors `UpgradesState` from the legacy TS
//! `packages/logic/src/state/schema.ts`. All four reset-tier resources
//! live here so `buy_upgrades` can dispatch on the upgrade tier without
//! taking four overloads. The seven `*_no_*_upgrades` flags are
//! achievement gates that flip `false` depending on the tier purchased —
//! see the per-tier flip matrix in
//! [`crate::mechanics::upgrades::buy_upgrades`].

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` read/written by `buy_upgrades`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UpgradesState {
    /// `player.coins`.
    pub coins: Decimal,
    /// `player.prestigePoints` (Diamonds-prestige currency).
    pub prestige_points: Decimal,
    /// `player.transcendPoints` (Mythos-prestige currency).
    pub transcend_points: Decimal,
    /// `player.reincarnationPoints` (Particles-prestige currency).
    pub reincarnation_points: Decimal,
    /// Bitmap of owned upgrades; `0` = unowned, `1` = owned. Indexed by
    /// `pos`.
    pub upgrades: Vec<u8>,
    /// Set false when any coin-tier upgrade is purchased; gates the
    /// "no coin upgrades during prestige" achievement.
    pub prestige_no_coin_upgrades: bool,
    /// Same idea, transcension lineage.
    pub transcend_no_coin_upgrades: bool,
    /// Set false when any coin- or prestige-tier upgrade is purchased.
    pub transcend_no_coin_or_prestige_upgrades: bool,
    /// Same idea, reincarnation lineage.
    pub reincarnate_no_coin_upgrades: bool,
    /// Set false when any coin- or prestige-tier upgrade is purchased
    /// (reincarnation lineage).
    pub reincarnate_no_coin_or_prestige_upgrades: bool,
    /// Set false when any coin-, prestige-, or transcend-tier upgrade is
    /// purchased.
    pub reincarnate_no_coin_prestige_or_transcend_upgrades: bool,
    /// Set false when any of the above OR a "generator" upgrade is
    /// purchased. (The legacy code lumps generator-row upgrades in with
    /// the four tier currencies for this gate.)
    pub reincarnate_no_coin_prestige_transcend_or_generator_upgrades: bool,
}

/// Default upgrade-bitmap length. Matches the legacy
/// `Array(141).fill(0)` initial state.
pub const UPGRADES_DEFAULT_LEN: usize = 141;

impl Default for UpgradesState {
    /// Zeroed resources, all-zero upgrade bitmap (`UPGRADES_DEFAULT_LEN`
    /// entries), and every achievement flag set to `true` because no
    /// upgrade has been purchased yet.
    fn default() -> Self {
        Self {
            coins: Decimal::zero(),
            prestige_points: Decimal::zero(),
            transcend_points: Decimal::zero(),
            reincarnation_points: Decimal::zero(),
            upgrades: vec![0; UPGRADES_DEFAULT_LEN],
            prestige_no_coin_upgrades: true,
            transcend_no_coin_upgrades: true,
            transcend_no_coin_or_prestige_upgrades: true,
            reincarnate_no_coin_upgrades: true,
            reincarnate_no_coin_or_prestige_upgrades: true,
            reincarnate_no_coin_prestige_or_transcend_upgrades: true,
            reincarnate_no_coin_prestige_transcend_or_generator_upgrades: true,
        }
    }
}
