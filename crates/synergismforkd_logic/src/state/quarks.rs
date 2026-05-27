//! Quarks (world-currency) state slice.
//!
//! Mirrors `player.worlds` and the two per-singularity / lifetime
//! quark counters. The legacy `Quarks` wrapper class encapsulates
//! the balance + a quark-bonus getter; this slice carries the
//! plain Decimal balance + the cached bonus (recomputed by the
//! tick layer).

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` for quark balances.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct QuarksState {
    /// `player.worlds` — current quark balance.
    pub worlds: Decimal,
    /// `player.quarksThisSingularity` — quarks earned during the
    /// current singularity (drives base-GQ award).
    pub quarks_this_singularity: f64,
    /// `player.allTimeQuarks` — lifetime total.
    pub all_time_quarks: f64,
    /// Cached quark-bonus percent (0..100). Refreshed each tick by
    /// the quark-bonus aggregator.
    pub quark_bonus: f64,
}

impl Default for QuarksState {
    fn default() -> Self {
        Self {
            worlds: Decimal::zero(),
            quarks_this_singularity: 0.0,
            all_time_quarks: 0.0,
            quark_bonus: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_zero() {
        let s = QuarksState::default();
        assert_eq!(s.worlds.to_number(), 0.0);
        assert_eq!(s.quark_bonus, 0.0);
    }
}
