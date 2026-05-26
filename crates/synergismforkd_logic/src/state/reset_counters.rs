//! Reset / prestige counters + ascension timers state slice.
//!
//! Mirrors the player.X reset-count fields, the four real-time
//! ascension counters, and the unlock-gate booleans. Backs the
//! ascension-related formulas and is read across the tick layer.

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` for reset counters + ascension timers.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetCountersState {
    /// `player.prestigeCount`.
    pub prestige_count: f64,
    /// `player.transcendCount`.
    pub transcend_count: f64,
    /// `player.reincarnationCount`.
    pub reincarnation_count: f64,
    /// `player.ascensionCount`.
    pub ascension_count: f64,
    /// `player.ascensionCounter` — in-ascension wall-clock seconds
    /// (game-time speed-mult applied).
    pub ascension_counter: f64,
    /// `player.ascensionCounterReal` — real-time seconds inside
    /// the current ascension (no speed mult).
    pub ascension_counter_real: f64,
    /// `player.ascensionCounterRealReal` — total real wall-clock
    /// time inside the current ascension (includes paused time).
    pub ascension_counter_real_real: f64,
    /// `player.prestigeShards`.
    pub prestige_shards: Decimal,
    /// `player.transcendShards`.
    pub transcend_shards: Decimal,
    /// `player.reincarnationShards`.
    pub reincarnation_shards: Decimal,
    /// `player.unlocks.prestige`.
    pub prestige_unlocked: bool,
    /// `player.unlocks.transcend`.
    pub transcend_unlocked: bool,
    /// `player.unlocks.reincarnate`.
    pub reincarnate_unlocked: bool,
    /// `player.unlocks.ascension` / `unlocks.coinone`-style gates.
    pub ascension_unlocked: bool,
    /// `player.achievementsUnlocked` (UI-gated by an achievement).
    pub achievements_unlocked: bool,
}

impl Default for ResetCountersState {
    fn default() -> Self {
        Self {
            prestige_count: 0.0,
            transcend_count: 0.0,
            reincarnation_count: 0.0,
            ascension_count: 0.0,
            ascension_counter: 0.0,
            ascension_counter_real: 0.0,
            ascension_counter_real_real: 0.0,
            prestige_shards: Decimal::zero(),
            transcend_shards: Decimal::zero(),
            reincarnation_shards: Decimal::zero(),
            prestige_unlocked: false,
            transcend_unlocked: false,
            reincarnate_unlocked: false,
            ascension_unlocked: false,
            achievements_unlocked: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_zero_locked() {
        let s = ResetCountersState::default();
        assert_eq!(s.prestige_count, 0.0);
        assert!(!s.prestige_unlocked);
    }
}
