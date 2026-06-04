//! Per-tick auto-tool leaves — obtainium / offerings / ant + rune
//! sacrifice. Direct port of
//! `legacy/core_split/packages/logic/src/tick/automaticTools.ts`.
//!
//! Each function is a pure leaf returning advanced state + intent
//! events; the actual side effects (`sacrificeAnts`, the rune purchase
//! fan-out) live in the UI tier and react to the emitted events. The
//! orchestrator ([`super::phase_automation`]) threads these into the
//! [`crate::state::GameState`] slices.

use smallvec::SmallVec;

use synergismforkd_bignum::Decimal;

use crate::events::{AutoTool, CoreEvent};
use crate::state::AutoSacrificeMode;

/// Crumbs threshold gating the auto-sacrifice cooldown. Mirrors the
/// legacy `MINIMUM_CRUMBS_FOR_SACRIFICE`.
pub(crate) const MINIMUM_CRUMBS_FOR_SACRIFICE: f64 = 1e40;
/// Minimum real seconds between auto-sacrifices. Mirrors
/// `MINIMUM_SECONDS_DELAY_BETWEEN_SACRIFICES`.
pub(crate) const MINIMUM_SECONDS_DELAY_BETWEEN_SACRIFICES: f64 = 0.05;
/// "Reborn ELO maxed" tolerance. Mirrors the inline `0.001`.
pub(crate) const REBORN_ELO_MAXED_TOLERANCE: f64 = 0.001;

/// `currentChallenge.ascension` value that aborts the obtainium branch.
const ADD_OBTAINIUM_ABORT_CHALLENGE: u32 = 14;
/// `taxmanLastStand.completions` at which the obtainium clamp engages.
const TAXMAN_CLAMP_COMPLETIONS: f64 = 2.0;

/// Inputs to [`add_obtainium`].
pub(crate) struct AddObtainiumInput {
    /// `player.obtainium` — balance to credit.
    pub obtainium: Decimal,
    /// Pre-evaluated `calculateResearchAutomaticObtainium(dt)` gain.
    pub obtainium_gain: Decimal,
    /// `player.currentChallenge.ascension` — `14` aborts the branch.
    pub ascension_challenge: u32,
    /// `taxmanLastStand.enabled` — half of the runaway-clamp gate.
    pub taxman_last_stand_enabled: bool,
    /// `taxmanLastStand.completions` — `>= 2` engages the clamp.
    pub taxman_last_stand_completions: f64,
}

/// Result of [`add_obtainium`].
pub(crate) struct AddObtainiumResult {
    /// Advanced `obtainium`.
    pub obtainium: Decimal,
    /// `AutoToolFired { AddObtainium }` unless the c14 abort fired.
    pub events: SmallVec<[CoreEvent; 1]>,
}

/// Per-tick automatic obtainium gain with the c14 abort + the
/// `taxmanLastStand` clamp (`min(gain, obtainium × 100 + 1)`).
pub(crate) fn add_obtainium(input: &AddObtainiumInput) -> AddObtainiumResult {
    if input.ascension_challenge == ADD_OBTAINIUM_ABORT_CHALLENGE {
        return AddObtainiumResult {
            obtainium: input.obtainium,
            events: SmallVec::new(),
        };
    }

    let mut gain = input.obtainium_gain;
    if input.taxman_last_stand_enabled
        && input.taxman_last_stand_completions >= TAXMAN_CLAMP_COMPLETIONS
    {
        gain = gain.min(input.obtainium * Decimal::from_finite(100.0) + Decimal::one());
    }

    let mut events = SmallVec::new();
    events.push(CoreEvent::AutoToolFired {
        tool: AutoTool::AddObtainium,
    });
    AddObtainiumResult {
        obtainium: input.obtainium + gain,
        events,
    }
}

/// Inputs to [`add_offerings`].
pub(crate) struct AddOfferingsInput {
    /// Tick delta (seconds) — added raw to the counter (the caller halves
    /// it; the legacy tail passes `dt / 2`).
    pub dt: f64,
    /// `G.autoOfferingCounter` — fractional carry between ticks.
    pub auto_offering_counter: f64,
    /// `player.offerings` — receives the whole-number overflow.
    pub offerings: Decimal,
}

/// Result of [`add_offerings`].
pub(crate) struct AddOfferingsResult {
    /// Advanced `auto_offering_counter` (mod 1).
    pub auto_offering_counter: f64,
    /// Advanced `offerings`.
    pub offerings: Decimal,
}

/// Fractional auto-offering counter: the whole portion of the
/// accumulated counter moves into `offerings`; the fraction carries to
/// the next tick. No event (no UI side effect in legacy).
pub(crate) fn add_offerings(input: &AddOfferingsInput) -> AddOfferingsResult {
    let advanced = input.auto_offering_counter + input.dt;
    AddOfferingsResult {
        auto_offering_counter: advanced % 1.0,
        offerings: input.offerings + Decimal::from_finite(advanced.floor()),
    }
}

/// Inputs to [`advance_ant_sacrifice_timers`].
pub(crate) struct AdvanceAntSacrificeTimersInput {
    /// Tick delta (seconds).
    pub dt: f64,
    /// Pre-evaluated global delta (`halfMind` → 10, else global speed).
    /// Applied to the scaled timer only.
    pub global_delta: f64,
    /// `player.antSacrificeTimer` — scaled in-game timer.
    pub ant_sacrifice_timer: f64,
    /// `player.antSacrificeTimerReal` — raw wall-clock timer.
    pub ant_sacrifice_timer_real: f64,
}

/// Result of [`advance_ant_sacrifice_timers`].
pub(crate) struct AdvanceAntSacrificeTimersResult {
    /// Advanced scaled timer.
    pub ant_sacrifice_timer: f64,
    /// Advanced raw timer.
    pub ant_sacrifice_timer_real: f64,
}

/// Advance the dual ant-sacrifice timers: the scaled one by
/// `dt × global_delta`, the raw one by `dt`.
pub(crate) fn advance_ant_sacrifice_timers(
    input: &AdvanceAntSacrificeTimersInput,
) -> AdvanceAntSacrificeTimersResult {
    AdvanceAntSacrificeTimersResult {
        ant_sacrifice_timer: input.ant_sacrifice_timer + input.dt * input.global_delta,
        ant_sacrifice_timer_real: input.ant_sacrifice_timer_real + input.dt,
    }
}

/// Inputs to [`check_ant_sacrifice_ready`].
pub(crate) struct CheckAntSacrificeReadyInput {
    /// `ants.toggles.autoSacrificeMode` — selects the mode-specific check.
    pub mode: AutoSacrificeMode,
    /// `ants.crumbsThisSacrifice` — tested against the crumb threshold.
    pub crumbs_this_sacrifice: Decimal,
    /// Post-advance `antSacrificeTimerReal` — tested against the delay.
    pub ant_sacrifice_timer_real: f64,
    /// `ants.toggles.autoSacrificeEnabled` — master gate.
    pub auto_sacrifice_enabled: bool,
    /// Pre-evaluated `calculateAvailableRebornELO()`.
    pub available_reborn_elo: f64,
    /// `ants.toggles.onlySacrificeMaxRebornELO`.
    pub only_sacrifice_max_reborn_elo: bool,
    /// `ants.toggles.alwaysSacrificeMaxRebornELO`.
    pub always_sacrifice_max_reborn_elo: bool,
    /// Post-advance `antSacrificeTimer` — used by `InGameTime`.
    pub ant_sacrifice_timer: f64,
    /// `ants.toggles.autoSacrificeThreshold` — shared timer/ELO threshold.
    pub auto_sacrifice_threshold: f64,
    /// Pre-evaluated `antSacrificeRewards().immortalELO`.
    pub immortal_elo_gain: f64,
    /// `ants.immortalELO`.
    pub immortal_elo: f64,
    /// `ants.rebornELO`.
    pub reborn_elo: f64,
}

/// Auto-ant-sacrifice readiness predicate. Emits `AntSacrificeTriggered`
/// when the universal gate (crumbs / delay / enabled) AND the active
/// mode's specific check pass (with the `alwaysSacrificeMaxRebornELO`
/// OR-branch and `onlySacrificeMaxRebornELO` short-circuit). Mirrors the
/// legacy `canAutoSacrifice` bug-for-bug.
pub(crate) fn check_ant_sacrifice_ready(
    input: &CheckAntSacrificeReadyInput,
) -> SmallVec<[CoreEvent; 1]> {
    let max_reborn_elo = input.available_reborn_elo < REBORN_ELO_MAXED_TOLERANCE;

    if input.only_sacrifice_max_reborn_elo && !max_reborn_elo {
        return SmallVec::new();
    }

    let universal_checks = input.crumbs_this_sacrifice
        >= Decimal::from_finite(MINIMUM_CRUMBS_FOR_SACRIFICE)
        && input.ant_sacrifice_timer_real >= MINIMUM_SECONDS_DELAY_BETWEEN_SACRIFICES
        && input.auto_sacrifice_enabled;

    let specific_check = match input.mode {
        AutoSacrificeMode::InGameTime => {
            input.ant_sacrifice_timer >= input.auto_sacrifice_threshold
        }
        AutoSacrificeMode::RealTime => {
            input.ant_sacrifice_timer_real >= input.auto_sacrifice_threshold
        }
        AutoSacrificeMode::ImmortalELOGain => {
            input.immortal_elo_gain >= input.auto_sacrifice_threshold
        }
        AutoSacrificeMode::MaxRebornELO => {
            (input.immortal_elo - input.reborn_elo) <= REBORN_ELO_MAXED_TOLERANCE
        }
    };

    let ready = if input.always_sacrifice_max_reborn_elo {
        universal_checks && (max_reborn_elo || specific_check)
    } else {
        universal_checks && specific_check
    };

    let mut events = SmallVec::new();
    if ready {
        events.push(CoreEvent::AntSacrificeTriggered);
    }
    events
}

/// Inputs to [`advance_rune_sacrifice`].
pub(crate) struct AdvanceRuneSacrificeInput {
    /// Tick delta (seconds) — added raw (`timeMultiplier == 1`).
    pub dt: f64,
    /// `player.sacrificeTimer` — fractional accumulator.
    pub sacrifice_timer: f64,
    /// Cached `calculateAutoSacrificeInterval()`.
    pub auto_sacrifice_interval: f64,
    /// `player.offerings` — gate (`> 0`) on whether a sacrifice fires.
    pub offerings: Decimal,
}

/// Result of [`advance_rune_sacrifice`].
pub(crate) struct AdvanceRuneSacrificeResult {
    /// Advanced timer, or `0` when a sacrifice fired.
    pub sacrifice_timer: f64,
    /// `RuneSacrificeTriggered` when the gate fired; empty otherwise.
    pub events: SmallVec<[CoreEvent; 1]>,
}

/// Per-tick rune-sacrifice trigger: advance the timer; when it crosses
/// `auto_sacrifice_interval` with offerings on hand, emit
/// `RuneSacrificeTriggered` and reset the timer.
pub(crate) fn advance_rune_sacrifice(
    input: &AdvanceRuneSacrificeInput,
) -> AdvanceRuneSacrificeResult {
    let advanced = input.sacrifice_timer + input.dt;
    if advanced >= input.auto_sacrifice_interval && input.offerings > Decimal::zero() {
        let mut events = SmallVec::new();
        events.push(CoreEvent::RuneSacrificeTriggered);
        return AdvanceRuneSacrificeResult {
            sacrifice_timer: 0.0,
            events,
        };
    }
    AdvanceRuneSacrificeResult {
        sacrifice_timer: advanced,
        events: SmallVec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_obtainium_credits_gain_and_emits() {
        let r = add_obtainium(&AddObtainiumInput {
            obtainium: Decimal::from_finite(100.0),
            obtainium_gain: Decimal::from_finite(50.0),
            ascension_challenge: 0,
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 0.0,
        });
        assert_eq!(r.obtainium.to_number(), 150.0);
        assert_eq!(r.events.len(), 1);
        assert!(matches!(
            r.events[0],
            CoreEvent::AutoToolFired {
                tool: AutoTool::AddObtainium
            }
        ));
    }

    #[test]
    fn add_obtainium_aborts_in_challenge_14() {
        let r = add_obtainium(&AddObtainiumInput {
            obtainium: Decimal::from_finite(100.0),
            obtainium_gain: Decimal::from_finite(50.0),
            ascension_challenge: 14,
            taxman_last_stand_enabled: false,
            taxman_last_stand_completions: 0.0,
        });
        assert_eq!(r.obtainium.to_number(), 100.0); // unchanged
        assert!(r.events.is_empty());
    }

    #[test]
    fn add_obtainium_taxman_clamps_gain() {
        // cap = 10 × 100 + 1 = 1001; gain 1e9 clamps to 1001 → 10 + 1001.
        let r = add_obtainium(&AddObtainiumInput {
            obtainium: Decimal::from_finite(10.0),
            obtainium_gain: Decimal::from_finite(1e9),
            ascension_challenge: 0,
            taxman_last_stand_enabled: true,
            taxman_last_stand_completions: 2.0,
        });
        assert_eq!(r.obtainium.to_number(), 1011.0);
    }

    #[test]
    fn add_offerings_moves_whole_portion() {
        // 0.7 + 2.5 = 3.2 → 3 offerings, 0.2 carry.
        let r = add_offerings(&AddOfferingsInput {
            dt: 2.5,
            auto_offering_counter: 0.7,
            offerings: Decimal::from_finite(10.0),
        });
        assert!((r.auto_offering_counter - 0.2).abs() < 1e-9);
        assert_eq!(r.offerings.to_number(), 13.0);
    }

    #[test]
    fn ant_sacrifice_timers_advance_scaled_and_raw() {
        let r = advance_ant_sacrifice_timers(&AdvanceAntSacrificeTimersInput {
            dt: 2.0,
            global_delta: 3.0,
            ant_sacrifice_timer: 1.0,
            ant_sacrifice_timer_real: 1.0,
        });
        assert_eq!(r.ant_sacrifice_timer, 7.0); // 1 + 2×3
        assert_eq!(r.ant_sacrifice_timer_real, 3.0); // 1 + 2
    }

    fn ready_input() -> CheckAntSacrificeReadyInput {
        CheckAntSacrificeReadyInput {
            mode: AutoSacrificeMode::InGameTime,
            crumbs_this_sacrifice: Decimal::from_finite(1e45),
            ant_sacrifice_timer_real: 10.0,
            auto_sacrifice_enabled: true,
            available_reborn_elo: 5.0, // not maxed
            only_sacrifice_max_reborn_elo: false,
            always_sacrifice_max_reborn_elo: false,
            ant_sacrifice_timer: 100.0,
            auto_sacrifice_threshold: 50.0,
            immortal_elo_gain: 0.0,
            immortal_elo: 0.0,
            reborn_elo: 0.0,
        }
    }

    #[test]
    fn ant_sacrifice_ready_when_all_conditions_met() {
        let e = check_ant_sacrifice_ready(&ready_input());
        assert_eq!(e.len(), 1);
        assert!(matches!(e[0], CoreEvent::AntSacrificeTriggered));
    }

    #[test]
    fn ant_sacrifice_blocked_when_disabled() {
        let e = check_ant_sacrifice_ready(&CheckAntSacrificeReadyInput {
            auto_sacrifice_enabled: false,
            ..ready_input()
        });
        assert!(e.is_empty());
    }

    #[test]
    fn ant_sacrifice_only_max_reborn_short_circuits() {
        // only-max with not-maxed ELO → blocked even though InGameTime passes.
        let e = check_ant_sacrifice_ready(&CheckAntSacrificeReadyInput {
            only_sacrifice_max_reborn_elo: true,
            available_reborn_elo: 5.0, // not maxed
            ..ready_input()
        });
        assert!(e.is_empty());
    }

    #[test]
    fn ant_sacrifice_below_threshold_does_not_fire() {
        let e = check_ant_sacrifice_ready(&CheckAntSacrificeReadyInput {
            ant_sacrifice_timer: 10.0, // below threshold 50
            ..ready_input()
        });
        assert!(e.is_empty());
    }

    #[test]
    fn rune_sacrifice_fires_when_interval_and_offerings_met() {
        let r = advance_rune_sacrifice(&AdvanceRuneSacrificeInput {
            dt: 5.0,
            sacrifice_timer: 10.0,
            auto_sacrifice_interval: 12.0,
            offerings: Decimal::from_finite(100.0),
        });
        assert_eq!(r.sacrifice_timer, 0.0); // reset on fire
        assert_eq!(r.events.len(), 1);
        assert!(matches!(r.events[0], CoreEvent::RuneSacrificeTriggered));
    }

    #[test]
    fn rune_sacrifice_accumulates_without_offerings() {
        let r = advance_rune_sacrifice(&AdvanceRuneSacrificeInput {
            dt: 5.0,
            sacrifice_timer: 10.0,
            auto_sacrifice_interval: 12.0,
            offerings: Decimal::zero(), // no offerings → no fire
        });
        assert_eq!(r.sacrifice_timer, 15.0);
        assert!(r.events.is_empty());
    }
}
