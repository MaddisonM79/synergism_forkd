//! Producer-family state slice and the per-click purchase cap.
//!
//! Mirrors `ProducerFamilyState` and `BuyAmount` from the legacy TS
//! `packages/logic/src/state/schema.ts`. One `ProducerFamilyState` instance
//! exists per family (Coin / Diamonds / Mythos / Particles) in the composed
//! `GameState` — the shape is family-agnostic.

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` read/written by the producer-purchase machinery.
/// Five positions (first..fifth) each have an owned count plus a current
/// cost; the shared resource is the family's spend currency
/// (coins / `prestigePoints` / `transcendPoints` / `reincarnationPoints`).
#[derive(Debug, Clone, PartialEq)]
pub struct ProducerFamilyState {
    /// Resource the family buys with.
    pub resource: Decimal,
    /// Tier-1 owned count.
    pub first_owned: f64,
    /// Tier-1 next cost.
    pub first_cost: Decimal,
    /// Tier-2 owned count.
    pub second_owned: f64,
    /// Tier-2 next cost.
    pub second_cost: Decimal,
    /// Tier-3 owned count.
    pub third_owned: f64,
    /// Tier-3 next cost.
    pub third_cost: Decimal,
    /// Tier-4 owned count.
    pub fourth_owned: f64,
    /// Tier-4 next cost.
    pub fourth_cost: Decimal,
    /// Tier-5 owned count.
    pub fifth_owned: f64,
    /// Tier-5 next cost.
    pub fifth_cost: Decimal,
}

/// Player-configurable per-click purchase cap. Mirrors the UI's
/// `x1 / x10 / x100 / ...` selector. The discriminants are the actual
/// cap values — call [`BuyAmount::as_f64`] to get the cap as a float for
/// the buy loops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BuyAmount {
    /// One purchase per click.
    One = 1,
    /// Ten purchases per click.
    Ten = 10,
    /// 100 purchases per click.
    Hundred = 100,
    /// 1 000 purchases per click.
    Thousand = 1_000,
    /// 10 000 purchases per click.
    TenThousand = 10_000,
    /// 100 000 purchases per click.
    HundredThousand = 100_000,
}

impl BuyAmount {
    /// The cap as an `f64` — matches the TS `number` shape used by the
    /// purchase loops.
    #[must_use]
    pub fn as_f64(self) -> f64 {
        f64::from(self as u32)
    }
}
