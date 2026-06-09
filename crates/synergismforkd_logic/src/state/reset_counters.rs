//! Reset / prestige counters + ascension timers state slice.
//!
//! Mirrors the player.X reset-count fields, the four real-time
//! ascension counters, and the unlock-gate booleans. Backs the
//! ascension-related formulas and is read across the tick layer.

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` for reset counters + ascension timers.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    /// `player.prestigecounter` — prestige reset timer (game-time
    /// seconds; advanced by the Phase 5 head timers).
    pub prestige_counter: f64,
    /// `player.transcendcounter` — transcension reset timer.
    pub transcend_counter: f64,
    /// `player.reincarnationcounter` — reincarnation reset timer.
    pub reincarnation_counter: f64,
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
    /// `player.unlocks.tesseracts` — set when `highest[11] > 0`
    /// (first c11 completion). Gates tesseract buildings + related UI.
    pub tesseracts_unlocked: bool,
    /// `player.unlocks.spirits` — set when `highest[12] > 0`.
    /// Gates spirit-rune content.
    pub spirits_unlocked: bool,
    /// `player.unlocks.hypercubes` — set when `highest[13] > 0`.
    /// Gates hypercube content.
    pub hypercubes_unlocked: bool,
    /// `player.unlocks.platonics` — set when `highest[14] > 0`.
    /// Gates platonic-cube content.
    pub platonics_unlocked: bool,
    /// `player.unlocks.coinone`..`coinfour` — coin-producer visibility gates,
    /// set when `coins` cross `500 / 1e4 / 1e5 / 4e6` (Synergism.ts:3976-3989).
    /// Reset to `false` only on singularity (Reset.ts:888).
    pub coin_one_unlocked: bool,
    /// See [`Self::coin_one_unlocked`].
    pub coin_two_unlocked: bool,
    /// See [`Self::coin_one_unlocked`].
    pub coin_three_unlocked: bool,
    /// See [`Self::coin_one_unlocked`].
    pub coin_four_unlocked: bool,
    /// `player.unlocks.generation` — generator-tab gate, set when buying
    /// generator 1 (prestige points `>= 1e12`, Automation.ts:8). Singularity-only
    /// reset.
    pub generation_unlocked: bool,
    /// `player.unlocks.rrow1`..`rrow4` — research-row visibility gates. In this
    /// TS version they are granted at singularity milestones (Reset.ts:970+) and
    /// reset on singularity; wired with the singularity layer.
    pub research_row_1_unlocked: bool,
    /// See [`Self::research_row_1_unlocked`].
    pub research_row_2_unlocked: bool,
    /// See [`Self::research_row_1_unlocked`].
    pub research_row_3_unlocked: bool,
    /// See [`Self::research_row_1_unlocked`].
    pub research_row_4_unlocked: bool,
    /// `player.unlocks.anthill` — ant feature gate, set on first `highest[8] > 0`
    /// (Synergism.ts:3692). Singularity-only reset.
    pub anthill_unlocked: bool,
    /// `player.unlocks.talismans` — talisman feature gate, set on first
    /// `highest[9] > 0` (Synergism.ts:3695). Singularity-only reset.
    pub talismans_unlocked: bool,
    /// `player.unlocks.blessings` — cube-blessing gate, set on first
    /// `highest[9] > 0` (Synergism.ts:3696). Singularity-only reset.
    pub blessings_unlocked: bool,
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
            prestige_counter: 0.0,
            transcend_counter: 0.0,
            reincarnation_counter: 0.0,
            prestige_shards: Decimal::zero(),
            transcend_shards: Decimal::zero(),
            reincarnation_shards: Decimal::zero(),
            prestige_unlocked: false,
            transcend_unlocked: false,
            reincarnate_unlocked: false,
            ascension_unlocked: false,
            achievements_unlocked: false,
            tesseracts_unlocked: false,
            spirits_unlocked: false,
            hypercubes_unlocked: false,
            platonics_unlocked: false,
            coin_one_unlocked: false,
            coin_two_unlocked: false,
            coin_three_unlocked: false,
            coin_four_unlocked: false,
            generation_unlocked: false,
            research_row_1_unlocked: false,
            research_row_2_unlocked: false,
            research_row_3_unlocked: false,
            research_row_4_unlocked: false,
            anthill_unlocked: false,
            talismans_unlocked: false,
            blessings_unlocked: false,
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
