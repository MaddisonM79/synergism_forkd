//! Event-buff state slice — daily/event coupon tracking.
//!
//! Mirrors `player.usedCoupons` and `player.dayCheck` from the
//! legacy schema. Backs [`crate::mechanics::event_buffs`] and
//! [`crate::mechanics::potion_bonuses`].

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

/// Slice of `GameState` for event buffs + daily-reset tracking.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EventBuffsState {
    /// `player.usedCoupons` — per-event coupon-redemption log.
    /// Keyed by event ID; the value is `1` if the coupon has been
    /// consumed in the current daily cycle.
    pub used_coupons: HashMap<String, u8>,
    /// `player.dayCheck` — Unix-time-seconds timestamp at the
    /// start of the current daily cycle. Used to detect rollover.
    pub day_check: f64,
    /// `player.daysSinceLogin` — convenience counter for the
    /// daily-streak feature.
    pub days_since_login: f64,
}

impl Default for EventBuffsState {
    fn default() -> Self {
        Self {
            used_coupons: HashMap::new(),
            day_check: 0.0,
            days_since_login: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_starts_empty() {
        let s = EventBuffsState::default();
        assert!(s.used_coupons.is_empty());
        assert_eq!(s.day_check, 0.0);
    }
}
