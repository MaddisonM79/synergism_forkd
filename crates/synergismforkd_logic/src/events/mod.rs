//! Core event enum — one variant per tick outcome or purchase confirmation.
//! The UI tier consumes the event stream and orchestrates side effects.

use synergismforkd_bignum::Decimal;

/// Which producer family a [`CoreEvent::ProducersPurchased`] event refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProducerType {
    /// Coin tier (base game).
    Coin,
    /// Diamonds tier (prestige).
    Diamonds,
    /// Mythos tier (transcension).
    Mythos,
    /// Particles tier (reincarnation).
    Particles,
}

/// Which resource tier a [`CoreEvent::UpgradePurchased`] event refers to.
/// Mirrors the legacy `UpgradeTier` string union — coin / prestige
/// (Diamonds) / transcend (Mythos) / reincarnation (Particles).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeTier {
    /// Bought with coins.
    Coin,
    /// Bought with prestige points (Diamonds layer).
    Prestige,
    /// Bought with transcend points (Mythos layer).
    Transcend,
    /// Bought with reincarnation points (Particles layer).
    Reincarnation,
}

/// Events emitted by mechanic functions. The closed set lets the UI dispatch
/// on the variant without a string-typed kind field, and `#[non_exhaustive]`
/// means new variants can land without breaking downstream `match` arms.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum CoreEvent {
    /// Accelerators were purchased — `after - before` units in total at
    /// a coin cost of `spent`.
    AcceleratorsPurchased {
        /// Owned count before the purchase loop ran.
        before: f64,
        /// Owned count after the purchase loop ran.
        after: f64,
        /// Coins removed from the player's balance.
        spent: Decimal,
    },
    /// Multipliers were purchased — same shape as
    /// [`CoreEvent::AcceleratorsPurchased`].
    MultipliersPurchased {
        /// Owned count before the purchase loop ran.
        before: f64,
        /// Owned count after the purchase loop ran.
        after: f64,
        /// Coins removed from the player's balance.
        spent: Decimal,
    },
    /// One position of a producer family was purchased.
    ProducersPurchased {
        /// Which family (Coin / Diamonds / Mythos / Particles).
        producer_type: ProducerType,
        /// Tier index, 1..=5 (1-based to match the legacy `buyMax(index)`
        /// parameter).
        index: u8,
        /// Owned count before the purchase loop ran.
        before: f64,
        /// Owned count after the purchase loop ran.
        after: f64,
        /// Family resource removed from the player's balance.
        spent: Decimal,
    },
    /// One of the five particle buildings was purchased.
    ParticleBuildingsPurchased {
        /// Tier index, 1..=5.
        index: u8,
        /// Owned count before the purchase loop ran.
        before: f64,
        /// Owned count after the purchase loop ran.
        after: f64,
        /// Reincarnation points removed from the player's balance.
        spent: Decimal,
    },
    /// One crystal upgrade leveled up (zero-or-more levels at once via
    /// the closed-form max-affordable solve).
    CrystalUpgradePurchased {
        /// 1-based crystal-upgrade index.
        i: u8,
        /// Level before the purchase.
        before: f64,
        /// Level after the purchase (includes any +10 bonus from owning
        /// upgrade-73 while in a reincarnation challenge).
        after: f64,
        /// Prestige shards removed from the player's balance.
        spent: Decimal,
    },
    /// A single-bit upgrade was purchased. The `spent` value is the cost
    /// in the tier's currency.
    UpgradePurchased {
        /// Which resource tier paid for the upgrade.
        tier: UpgradeTier,
        /// Upgrade position in the bitmap.
        pos: u32,
        /// Currency removed from the player's balance.
        spent: Decimal,
    },
    /// One tier of the tesseract (ascension) building family was
    /// purchased. `spent` is in `wow_tesseracts` (an `f64` because the
    /// resource caps out well below `1e308`).
    TesseractBuildingsPurchased {
        /// Tier index, 1..=5.
        index: u8,
        /// Owned count before the purchase loop ran.
        before: f64,
        /// Owned count after the purchase loop ran.
        after: f64,
        /// `wow_tesseracts` removed from the player's balance.
        spent: f64,
    },
}
