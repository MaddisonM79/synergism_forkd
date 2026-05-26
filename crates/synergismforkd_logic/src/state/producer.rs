//! Producer-family state slice and the per-click purchase cap.
//!
//! Mirrors `ProducerFamilyState` and `BuyAmount` from the legacy TS
//! `packages/logic/src/state/schema.ts`. One `ProducerFamilyState` instance
//! exists per family (Coin / Diamonds / Mythos / Particles) in the composed
//! `GameState` — the shape is family-agnostic.

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` read/written by the producer-purchase machinery.
/// Five positions (first..fifth) each have an owned count plus a current
/// cost. The family's spend currency
/// (coins / `prestigePoints` / `transcendPoints` / `reincarnationPoints`)
/// is **not** stored here — it lives in `state.upgrades`, and `buy_max` /
/// `buy_producer` take it as a separate `&mut Decimal` parameter. (Ledger
/// Finding 1 — duplicate-field collapse. A future refactor may make
/// `ProducerFamilyState` generic over a typed `Currency` to lock the
/// caller-side resource pairing at compile time.)
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ProducerFamilyState {
    /// Tier-1 owned count.
    pub first_owned: f64,
    /// Tier-1 next cost.
    pub first_cost: Decimal,
    /// Tier-1 generated count — auto-generated from the tier-2 producer's
    /// per-tick production. Tracked separately from `first_owned` because
    /// purchased and generated units obey different mechanic gates (only
    /// purchased units count toward "no producer purchased" achievements
    /// and toward certain reset bonuses). Mirrors the legacy
    /// `player.{first}Generated{Family}` field.
    pub first_generated: Decimal,
    /// Tier-2 owned count.
    pub second_owned: f64,
    /// Tier-2 next cost.
    pub second_cost: Decimal,
    /// Tier-2 generated count — see `first_generated`.
    pub second_generated: Decimal,
    /// Tier-3 owned count.
    pub third_owned: f64,
    /// Tier-3 next cost.
    pub third_cost: Decimal,
    /// Tier-3 generated count — see `first_generated`.
    pub third_generated: Decimal,
    /// Tier-4 owned count.
    pub fourth_owned: f64,
    /// Tier-4 next cost.
    pub fourth_cost: Decimal,
    /// Tier-4 generated count — see `first_generated`.
    pub fourth_generated: Decimal,
    /// Tier-5 owned count.
    pub fifth_owned: f64,
    /// Tier-5 next cost.
    pub fifth_cost: Decimal,
    /// Tier-5 generated count — see `first_generated`. Note: the fifth
    /// tier has no "tier 6" producer to feed it, so this field never
    /// actually changes in the cascade, but it's tracked here for shape
    /// uniformity with the other four tiers.
    pub fifth_generated: Decimal,
}

/// Player-configurable per-click purchase cap. Mirrors the UI's
/// `x1 / x10 / x100 / ...` selector.
///
/// Discriminants are not load-bearing — [`BuyAmount::as_f64`] matches
/// arms explicitly so adding a `Custom(u32)` variant later doesn't
/// silently break the `discriminant-as-cap` pun. (Anvil F9.)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuyAmount {
    /// One purchase per click.
    One,
    /// Ten purchases per click.
    Ten,
    /// 100 purchases per click.
    Hundred,
    /// 1 000 purchases per click.
    Thousand,
    /// 10 000 purchases per click.
    TenThousand,
    /// 100 000 purchases per click.
    HundredThousand,
}

impl BuyAmount {
    /// The cap as an `f64` — matches the TS `number` shape used by the
    /// purchase loops.
    #[must_use]
    pub fn as_f64(self) -> f64 {
        match self {
            Self::One => 1.0,
            Self::Ten => 10.0,
            Self::Hundred => 100.0,
            Self::Thousand => 1_000.0,
            Self::TenThousand => 10_000.0,
            Self::HundredThousand => 100_000.0,
        }
    }
}

impl ProducerFamilyState {
    /// Read the owned count for tier `index` (1..=5). Mirrors the
    /// `readOwned` helper in the legacy TS source. In release, indices
    /// outside `1..=5` fall through to the fifth tier (matching the TS
    /// `else state.fifthOwned` default); a debug assertion catches the
    /// mistake during development.
    #[must_use]
    pub fn owned(&self, index: u8) -> f64 {
        debug_assert!(
            matches!(index, 1..=5),
            "producer index out of range: {index}"
        );
        match index {
            1 => self.first_owned,
            2 => self.second_owned,
            3 => self.third_owned,
            4 => self.fourth_owned,
            _ => self.fifth_owned,
        }
    }

    /// Read the cost cache for tier `index` (1..=5). Same out-of-range
    /// behavior as [`Self::owned`].
    #[must_use]
    pub fn cost(&self, index: u8) -> Decimal {
        debug_assert!(
            matches!(index, 1..=5),
            "producer index out of range: {index}"
        );
        match index {
            1 => self.first_cost,
            2 => self.second_cost,
            3 => self.third_cost,
            4 => self.fourth_cost,
            _ => self.fifth_cost,
        }
    }

    /// Write the owned count for tier `index` (1..=5). Same out-of-range
    /// behavior as [`Self::owned`].
    pub fn set_owned(&mut self, index: u8, value: f64) {
        debug_assert!(
            matches!(index, 1..=5),
            "producer index out of range: {index}"
        );
        match index {
            1 => self.first_owned = value,
            2 => self.second_owned = value,
            3 => self.third_owned = value,
            4 => self.fourth_owned = value,
            _ => self.fifth_owned = value,
        }
    }

    /// Write the cost cache for tier `index` (1..=5). Same out-of-range
    /// behavior as [`Self::owned`].
    pub fn set_cost(&mut self, index: u8, value: Decimal) {
        debug_assert!(
            matches!(index, 1..=5),
            "producer index out of range: {index}"
        );
        match index {
            1 => self.first_cost = value,
            2 => self.second_cost = value,
            3 => self.third_cost = value,
            4 => self.fourth_cost = value,
            _ => self.fifth_cost = value,
        }
    }
}
