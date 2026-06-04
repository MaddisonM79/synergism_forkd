//! Automation state slice — toggles, modes, thresholds, and per-tick
//! accumulators for the Phase 5 tick-automation layer.
//!
//! Mirrors the scattered `player.toggles[N]`, `player.autoX`,
//! `player.resetToggleModes`, the `G.*` auto-reset / auto-offering
//! timers, and the challenge-sweep machine state from the legacy
//! `tick/` package. These have no single mechanic-family home, so they
//! live together here. Backs the orchestration in [`crate::tick`].
//!
//! Fields that clearly belong to a mechanic family live in that
//! family's slice instead: the prestige / transcend / reincarnation
//! reset counters in [`super::ResetCountersState`]; the quark, golden-
//! quark, ambrosia, red-ambrosia, octeract, and ant-sacrifice timers in
//! their own slices; the auto-research toggle in
//! [`super::ResearchesState`].

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

use crate::events::{AutoResetMode, SweepState};

/// Slice of `GameState` for cross-cutting automation toggles + timers.
///
/// Not `Copy` (holds a [`SweepState`], which carries a `BTreeSet`) and
/// not `#[derive(Default)]` (holds a [`Decimal`], which has no
/// `Default`); the manual [`Default`] impl below sets inert values.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AutomationState {
    // ── Auto-reset toggles / modes / thresholds ──────────────────────
    /// `player.toggles[15]` — auto-prestige enabled.
    pub auto_prestige_enabled: bool,
    /// `player.toggles[21]` — auto-transcend enabled.
    pub auto_transcend_enabled: bool,
    /// `player.toggles[27]` — auto-reincarnate enabled.
    pub auto_reincarnate_enabled: bool,
    /// `player.resetToggleModes.prestige` — amount- vs time-gated.
    pub prestige_reset_mode: AutoResetMode,
    /// `player.resetToggleModes.transcend`.
    pub transcend_reset_mode: AutoResetMode,
    /// `player.resetToggleModes.reincarnation`.
    pub reincarnation_reset_mode: AutoResetMode,
    /// `player.prestigeamount` — exponent (amount mode) or seconds
    /// (time mode) threshold.
    pub prestige_amount: f64,
    /// `player.transcendamount`.
    pub transcend_amount: f64,
    /// `player.reincarnationamount`.
    pub reincarnation_amount: f64,
    /// `G.autoResetTimers.prestige` — time-mode accumulator (seconds).
    pub auto_reset_timer_prestige: f64,
    /// `G.autoResetTimers.transcension`.
    pub auto_reset_timer_transcension: f64,
    /// `G.autoResetTimers.reincarnation`.
    pub auto_reset_timer_reincarnation: f64,

    // ── Auto-potion dispenser ────────────────────────────────────────
    /// `player.toggles[42]` — auto-offering-potion fast mode.
    pub auto_potion_toggle_offering: bool,
    /// `player.toggles[43]` — auto-obtainium-potion fast mode.
    pub auto_potion_toggle_obtainium: bool,
    /// `player.autoPotionTimer` — offering-potion accumulator (seconds).
    pub auto_potion_timer: f64,
    /// `player.autoPotionTimerObtainium` — obtainium-potion accumulator.
    pub auto_potion_timer_obtainium: f64,

    // ── Rune sacrifice ───────────────────────────────────────────────
    /// `player.autoSacrifice` — auto-rune-sacrifice enabled.
    pub rune_sacrifice_auto_enabled: bool,
    /// `player.autoSacrificeInterval` — seconds between auto-sacrifices.
    pub auto_sacrifice_interval: f64,
    /// `G.sacrificeTimer` — auto-rune-sacrifice accumulator (seconds).
    pub sacrifice_timer: f64,

    // ── Auto offerings ───────────────────────────────────────────────
    /// `player.offerings` — offering currency (rune-sacrifice fuel).
    pub offerings: Decimal,
    /// `G.autoOfferingCounter` — fractional auto-offering accumulator.
    pub auto_offering_counter: f64,

    // ── Challenge sweep machine ──────────────────────────────────────
    /// `player.autoChallengeRunning` — sweep armed.
    pub auto_challenge_running: bool,
    /// Challenge-sweep machine state (mirrors the legacy `SweepStates`).
    pub sweep_state: SweepState,
    /// Seconds elapsed since the last sweep-state transition.
    pub sweep_time_since_last_change: f64,
}

impl Default for AutomationState {
    /// Inert defaults — every auto-toggle off, every reset mode
    /// `Amount`, every timer `0`, the sweep machine `Idle`, and zero
    /// offerings.
    fn default() -> Self {
        Self {
            auto_prestige_enabled: false,
            auto_transcend_enabled: false,
            auto_reincarnate_enabled: false,
            prestige_reset_mode: AutoResetMode::Amount,
            transcend_reset_mode: AutoResetMode::Amount,
            reincarnation_reset_mode: AutoResetMode::Amount,
            prestige_amount: 0.0,
            transcend_amount: 0.0,
            reincarnation_amount: 0.0,
            auto_reset_timer_prestige: 0.0,
            auto_reset_timer_transcension: 0.0,
            auto_reset_timer_reincarnation: 0.0,
            auto_potion_toggle_offering: false,
            auto_potion_toggle_obtainium: false,
            auto_potion_timer: 0.0,
            auto_potion_timer_obtainium: 0.0,
            rune_sacrifice_auto_enabled: false,
            auto_sacrifice_interval: 0.0,
            sacrifice_timer: 0.0,
            offerings: Decimal::zero(),
            auto_offering_counter: 0.0,
            auto_challenge_running: false,
            sweep_state: SweepState::Idle,
            sweep_time_since_last_change: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_inert() {
        let s = AutomationState::default();
        assert!(!s.auto_prestige_enabled);
        assert!(!s.auto_challenge_running);
        assert_eq!(s.prestige_reset_mode, AutoResetMode::Amount);
        assert_eq!(s.offerings.to_number(), 0.0);
        assert_eq!(s.sweep_state, SweepState::Idle);
        assert_eq!(s.sacrifice_timer, 0.0);
        assert_eq!(s.auto_offering_counter, 0.0);
    }
}
