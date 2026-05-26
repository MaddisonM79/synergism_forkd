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
}
