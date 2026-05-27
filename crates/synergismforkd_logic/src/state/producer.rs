//! Producer-family state slice and the per-click purchase cap.
//!
//! Mirrors `ProducerFamilyState` and `BuyAmount` from the legacy TS
//! `packages/logic/src/state/schema.ts`. One `ProducerFamilyState` instance
//! exists per family (Coin / Diamonds / Mythos / Particles) in the composed
//! `GameState` — the shape is family-agnostic.

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// One position in a producer family — purchased count, next cost,
/// auto-generated count from the next tier's production. (Anvil F15:
/// previously the 15 flat fields lived on `ProducerFamilyState`; the
/// tier shape makes invalid indices a compile error rather than a
/// runtime debug-assert.)
///
/// `purchased` and `generated` are tracked separately because they obey
/// different mechanic gates — only purchased units count toward
/// "no producer purchased" achievements and certain reset bonuses.
/// Mirrors the legacy `player.{first}Owned{Family}` /
/// `player.{first}Generated{Family}` split.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub struct ProducerTier {
    /// Owned count (purchased through `buy_max` / `buy_producer`).
    pub owned: f64,
    /// Cost of the next purchase (cached so the UI can render without
    /// recomputing).
    pub cost: Decimal,
    /// Auto-generated count from the next-tier producer's per-tick
    /// production. The terminal (tier-5) entry never changes in the
    /// cascade because there's no tier-6 to feed it; it's still tracked
    /// here for shape uniformity.
    pub generated: Decimal,
}

/// Slice of `GameState` read/written by the producer-purchase machinery.
/// Five tiers, accessed via `tiers[index - 1]` (1-based legacy
/// convention preserved through the public accessor methods). The
/// family's spend currency (coins / `prestigePoints` /
/// `transcendPoints` / `reincarnationPoints`) is **not** stored here —
/// it lives in `state.upgrades`, and `buy_max` / `buy_producer` take it
/// as a separate `&mut Decimal` parameter. (Ledger Finding 1 —
/// duplicate-field collapse. A future refactor may make
/// `ProducerFamilyState` generic over a typed `Currency` to lock the
/// caller-side resource pairing at compile time.)
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ProducerFamilyState {
    /// Five-tier producer ladder. Indexed `0..=4` directly; the 1-based
    /// public accessors ([`Self::owned`], [`Self::cost`],
    /// [`Self::set_owned`], [`Self::set_cost`]) subtract 1 to match the
    /// legacy convention.
    pub tiers: [ProducerTier; 5],
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
    /// Internal 0-based index from the 1-based public index. Indices
    /// outside `1..=5` fall through to the fifth tier (matching the
    /// legacy TS `else state.fifthOwned` default); a debug assertion
    /// catches the mistake during development.
    #[inline]
    fn tier_index(index: u8) -> usize {
        debug_assert!(
            matches!(index, 1..=5),
            "producer index out of range: {index}"
        );
        match index {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 3,
            _ => 4,
        }
    }

    /// Read the owned count for tier `index` (1..=5). Mirrors the
    /// `readOwned` helper in the legacy TS source.
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
