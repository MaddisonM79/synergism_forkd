//! Extended coin-counter state slice.
//!
//! The Coin balance already lives in [`crate::state::UpgradesState`]
//! and [`crate::state::ProducerFamilyState`]; this slice carries the
//! per-prestige-window counters and the lifetime total that several
//! formula reads consume.

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` for "coins earned during this prestige
/// window" counters. The current-balance `coins` field lives in
/// `UpgradesState`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CoinCountersState {
    /// `player.coinsThisPrestige` — coins earned since last prestige.
    pub coins_this_prestige: Decimal,
    /// `player.coinsThisTranscension` — coins earned since last
    /// transcension.
    pub coins_this_transcension: Decimal,
    /// `player.coinsThisReincarnation` — coins earned since last
    /// reincarnation.
    pub coins_this_reincarnation: Decimal,
    /// `player.coinsTotal` — lifetime coins earned.
    pub coins_total: Decimal,
}

impl Default for CoinCountersState {
    fn default() -> Self {
        Self {
            coins_this_prestige: Decimal::zero(),
            coins_this_transcension: Decimal::zero(),
            coins_this_reincarnation: Decimal::zero(),
            coins_total: Decimal::zero(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_zero() {
        let s = CoinCountersState::default();
        assert_eq!(s.coins_this_prestige.to_number(), 0.0);
        assert_eq!(s.coins_total.to_number(), 0.0);
    }
}
