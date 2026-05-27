//! Particle-buildings state slice.
//!
//! Mirrors `ParticleBuildingsState` from the legacy TS
//! `packages/logic/src/state/schema.ts`. Five tiers, same shape as
//! [`crate::state::ProducerFamilyState`] — owned / cost / generated per
//! tier. Particle buildings have their own cost curve
//! (`base * 2^buyingTo` + a quadratic-in-exponent tail) and no
//! per-position "didn't buy" achievement gates, but the state layout
//! is identical, so we share the [`ProducerTier`] data type.
//!
//! The spend resource (`reincarnation_points`) is **not** stored here —
//! `state.upgrades.reincarnation_points` is canonical.
//! `buy_particle_building` reads/writes it as a separate `&mut Decimal`
//! parameter. (Ledger Finding 1 — duplicate-field collapse.)

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

use crate::state::ProducerTier;

/// Slice of `GameState` read/written by the particle-building-purchase
/// machinery. Five tiers, accessed via `tiers[index - 1]` (1-based
/// legacy convention preserved through the public accessor methods).
///
/// Out-of-bounds indices are compile-time `[_; 5]` errors. (Anvil F15
/// follow-on: previously the 15 flat fields lived on this struct.)
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ParticleBuildingsState {
    /// Five-tier particle-building ladder. Indexed `0..=4` directly;
    /// the 1-based public accessors ([`Self::owned`], [`Self::cost`],
    /// [`Self::set_owned`], [`Self::set_cost`]) subtract 1 to match
    /// the legacy convention.
    pub tiers: [ProducerTier; 5],
}

impl ParticleBuildingsState {
    /// Internal 0-based index from the 1-based public index. Indices
    /// outside `1..=5` fall through to the fifth tier (matching the
    /// legacy TS fall-through); a debug assertion catches the mistake
    /// during development.
    #[inline]
    fn tier_index(index: u8) -> usize {
        debug_assert!(
            matches!(index, 1..=5),
            "particle index out of range: {index}"
        );
        match index {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 3,
            _ => 4,
        }
    }

    /// Read the owned count for tier `index` (1..=5).
    #[must_use]
    pub fn owned(&self, index: u8) -> f64 {
        self.tiers[Self::tier_index(index)].owned
    }

    /// Read the cost cache for tier `index` (1..=5). Same out-of-range
    /// behavior as [`Self::owned`].
    #[must_use]
    pub fn cost(&self, index: u8) -> Decimal {
        self.tiers[Self::tier_index(index)].cost
    }

    /// Write the owned count for tier `index` (1..=5). Same out-of-range
    /// behavior as [`Self::owned`].
    pub fn set_owned(&mut self, index: u8, value: f64) {
        self.tiers[Self::tier_index(index)].owned = value;
    }

    /// Write the cost cache for tier `index` (1..=5). Same out-of-range
    /// behavior as [`Self::owned`].
    pub fn set_cost(&mut self, index: u8, value: Decimal) {
        self.tiers[Self::tier_index(index)].cost = value;
    }
}
