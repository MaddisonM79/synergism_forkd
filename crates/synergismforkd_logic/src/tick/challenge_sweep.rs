//! Per-tick challenge-sweep state machine. Direct port of
//! `legacy/core_split/packages/logic/src/tick/challengeSweep.ts`.
//!
//! Cycles the player through Transcension + Reincarnation challenges when
//! the autoChallenge feature is on. Holds two pieces of mutable state —
//! the [`SweepState`] and the wall-clock since the last transition — both
//! threaded in and out each tick. Transition side effects (resetCheck /
//! toggleChallenges) become `ChallengeSweepTransitioned` events carrying
//! the full from/to states for the UI tier to dispatch.
//!
//! The external lookups the transition needs (`getNextRegularChallenge`,
//! `getMaxChallenges`, the c15 auto-exponent guard) are pre-evaluated by
//! the caller and passed in.

use std::collections::BTreeSet;

use smallvec::SmallVec;

use crate::events::{CoreEvent, SweepState};

/// `c15_wait` duration in seconds. Mirrors the legacy hardcoded `5`.
const C15_WAIT_SECONDS: f64 = 5.0;

/// Inputs to [`tick_challenge_sweep`] (the current [`SweepState`] is passed
/// separately). All fields are caller pre-evaluated.
pub(crate) struct TickChallengeSweepInput {
    /// Tick delta (seconds).
    pub dt: f64,
    /// Wall-clock seconds since the last state change.
    pub time_since_last_state_change: f64,
    /// Pre-evaluated `researches[150] > 0 && autoChallengeRunning`.
    pub should_run_sweep: bool,
    /// `initial_wait → active` threshold.
    pub timer_start: f64,
    /// `active → next-stage` threshold.
    pub timer_exit: f64,
    /// `enter_wait → active` threshold.
    pub timer_enter: f64,
    /// `getNextRegularChallenge(initialIndex, {})` — `-1` means all
    /// regular challenges are maxed (→ `finished`).
    pub next_regular_challenge_from_initial: i32,
    /// `getNextRegularChallenge(active.index, active.explored)` — `-1`
    /// means the cycle is exhausted.
    pub next_regular_challenge_from_active: i32,
    /// Pre-evaluated `challenge15AutoExponentCheck()`.
    pub challenge_15_auto_exponent_check: bool,
    /// Pre-evaluated `finished` revalidation guard (c1 + c6 still maxed).
    pub is_finished_still_valid: bool,
}

/// Result of [`tick_challenge_sweep`].
pub(crate) struct TickChallengeSweepResult {
    /// The post-tick sweep state.
    pub state: SweepState,
    /// Post-tick wall-clock since the last change (`0` on transition).
    pub time_since_last_state_change: f64,
    /// `ChallengeSweepTransitioned` (0 or 1) carrying full from/to states.
    pub events: SmallVec<[CoreEvent; 1]>,
}

/// Pure transition function. Returns a value-equal state when there is no
/// transition (every real transition changes the variant), so the driver
/// can detect change by inequality.
fn sweep_transition(
    state: &SweepState,
    elapsed: f64,
    input: &TickChallengeSweepInput,
) -> SweepState {
    match state {
        SweepState::Idle => SweepState::Idle,

        SweepState::InitialWait => {
            if elapsed >= input.timer_start {
                if input.next_regular_challenge_from_initial == -1 {
                    SweepState::Finished
                } else {
                    let idx = input.next_regular_challenge_from_initial as u8;
                    SweepState::Active {
                        index: idx,
                        explored: BTreeSet::from([idx]),
                    }
                }
            } else {
                state.clone()
            }
        }

        SweepState::Active { explored, .. } => {
            if elapsed >= input.timer_exit {
                if input.next_regular_challenge_from_active == -1 {
                    if input.challenge_15_auto_exponent_check {
                        SweepState::C15Wait
                    } else {
                        SweepState::InitialWait
                    }
                } else {
                    SweepState::EnterWait {
                        to_index: input.next_regular_challenge_from_active as u8,
                        explored: explored.clone(),
                    }
                }
            } else {
                state.clone()
            }
        }

        SweepState::EnterWait { to_index, explored } => {
            if elapsed >= input.timer_enter {
                let mut explored = explored.clone();
                explored.insert(*to_index);
                SweepState::Active {
                    index: *to_index,
                    explored,
                }
            } else {
                state.clone()
            }
        }

        SweepState::C15Wait => {
            if elapsed >= C15_WAIT_SECONDS {
                SweepState::InitialWait
            } else {
                SweepState::C15Wait
            }
        }

        SweepState::Finished => {
            if input.is_finished_still_valid {
                SweepState::Finished
            } else {
                SweepState::InitialWait
            }
        }
    }
}

/// Per-tick sweep driver. Boots idle→initial_wait when the feature turns
/// on, tears down to idle when it turns off, otherwise accumulates the
/// timer and runs [`sweep_transition`], emitting a transition event when
/// the state changes.
pub(crate) fn tick_challenge_sweep(
    state: &SweepState,
    input: &TickChallengeSweepInput,
) -> TickChallengeSweepResult {
    let was_enabled = !matches!(state, SweepState::Idle);
    let is_enabled = input.should_run_sweep;

    if !was_enabled && is_enabled {
        let mut events = SmallVec::new();
        events.push(CoreEvent::ChallengeSweepTransitioned {
            from: SweepState::Idle,
            to: SweepState::InitialWait,
        });
        return TickChallengeSweepResult {
            state: SweepState::InitialWait,
            time_since_last_state_change: 0.0,
            events,
        };
    }

    if was_enabled && !is_enabled {
        let mut events = SmallVec::new();
        events.push(CoreEvent::ChallengeSweepTransitioned {
            from: state.clone(),
            to: SweepState::Idle,
        });
        return TickChallengeSweepResult {
            state: SweepState::Idle,
            time_since_last_state_change: 0.0,
            events,
        };
    }

    if !is_enabled {
        return TickChallengeSweepResult {
            state: state.clone(),
            time_since_last_state_change: input.time_since_last_state_change,
            events: SmallVec::new(),
        };
    }

    let elapsed = input.time_since_last_state_change + input.dt;
    let new_state = sweep_transition(state, elapsed, input);

    if new_state != *state {
        let mut events = SmallVec::new();
        events.push(CoreEvent::ChallengeSweepTransitioned {
            from: state.clone(),
            to: new_state.clone(),
        });
        TickChallengeSweepResult {
            state: new_state,
            time_since_last_state_change: 0.0,
            events,
        }
    } else {
        TickChallengeSweepResult {
            state: state.clone(),
            time_since_last_state_change: elapsed,
            events: SmallVec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> TickChallengeSweepInput {
        TickChallengeSweepInput {
            dt: 1.0,
            time_since_last_state_change: 0.0,
            should_run_sweep: true,
            timer_start: 2.0,
            timer_exit: 2.0,
            timer_enter: 2.0,
            next_regular_challenge_from_initial: 1,
            next_regular_challenge_from_active: 2,
            challenge_15_auto_exponent_check: false,
            is_finished_still_valid: true,
        }
    }

    fn transitioned(events: &[CoreEvent]) -> Option<(&SweepState, &SweepState)> {
        events.iter().find_map(|e| match e {
            CoreEvent::ChallengeSweepTransitioned { from, to } => Some((from, to)),
            _ => None,
        })
    }

    #[test]
    fn boots_from_idle_when_enabled() {
        let r = tick_challenge_sweep(&SweepState::Idle, &input());
        assert_eq!(r.state, SweepState::InitialWait);
        assert_eq!(r.time_since_last_state_change, 0.0);
        let (from, to) = transitioned(&r.events).expect("transition event");
        assert_eq!(*from, SweepState::Idle);
        assert_eq!(*to, SweepState::InitialWait);
    }

    #[test]
    fn tears_down_to_idle_when_disabled() {
        let r = tick_challenge_sweep(
            &SweepState::InitialWait,
            &TickChallengeSweepInput {
                should_run_sweep: false,
                ..input()
            },
        );
        assert_eq!(r.state, SweepState::Idle);
        let (_, to) = transitioned(&r.events).expect("transition event");
        assert_eq!(*to, SweepState::Idle);
    }

    #[test]
    fn idle_and_disabled_is_a_noop() {
        let r = tick_challenge_sweep(
            &SweepState::Idle,
            &TickChallengeSweepInput {
                should_run_sweep: false,
                ..input()
            },
        );
        assert_eq!(r.state, SweepState::Idle);
        assert!(r.events.is_empty());
    }

    #[test]
    fn initial_wait_enters_first_challenge() {
        // elapsed = 0 + 2 = 2 >= timer_start 2 → active{1, {1}}.
        let r = tick_challenge_sweep(
            &SweepState::InitialWait,
            &TickChallengeSweepInput {
                time_since_last_state_change: 1.0,
                ..input()
            },
        );
        assert_eq!(
            r.state,
            SweepState::Active {
                index: 1,
                explored: BTreeSet::from([1])
            }
        );
    }

    #[test]
    fn initial_wait_finishes_when_all_maxed() {
        let r = tick_challenge_sweep(
            &SweepState::InitialWait,
            &TickChallengeSweepInput {
                time_since_last_state_change: 1.0,
                next_regular_challenge_from_initial: -1,
                ..input()
            },
        );
        assert_eq!(r.state, SweepState::Finished);
    }

    #[test]
    fn active_moves_to_enter_wait() {
        let r = tick_challenge_sweep(
            &SweepState::Active {
                index: 1,
                explored: BTreeSet::from([1]),
            },
            &TickChallengeSweepInput {
                time_since_last_state_change: 1.0, // +1 = 2 >= timer_exit
                next_regular_challenge_from_active: 2,
                ..input()
            },
        );
        assert_eq!(
            r.state,
            SweepState::EnterWait {
                to_index: 2,
                explored: BTreeSet::from([1])
            }
        );
    }

    #[test]
    fn enter_wait_activates_and_grows_explored() {
        let r = tick_challenge_sweep(
            &SweepState::EnterWait {
                to_index: 2,
                explored: BTreeSet::from([1]),
            },
            &TickChallengeSweepInput {
                time_since_last_state_change: 1.0, // +1 = 2 >= timer_enter
                ..input()
            },
        );
        assert_eq!(
            r.state,
            SweepState::Active {
                index: 2,
                explored: BTreeSet::from([1, 2])
            }
        );
    }

    #[test]
    fn active_exhausted_routes_by_c15_check() {
        // next == -1, c15 check off → initial_wait.
        let restart = tick_challenge_sweep(
            &SweepState::Active {
                index: 5,
                explored: BTreeSet::from([5]),
            },
            &TickChallengeSweepInput {
                time_since_last_state_change: 1.0,
                next_regular_challenge_from_active: -1,
                challenge_15_auto_exponent_check: false,
                ..input()
            },
        );
        assert_eq!(restart.state, SweepState::InitialWait);

        // next == -1, c15 check on → c15_wait.
        let c15 = tick_challenge_sweep(
            &SweepState::Active {
                index: 5,
                explored: BTreeSet::from([5]),
            },
            &TickChallengeSweepInput {
                time_since_last_state_change: 1.0,
                next_regular_challenge_from_active: -1,
                challenge_15_auto_exponent_check: true,
                ..input()
            },
        );
        assert_eq!(c15.state, SweepState::C15Wait);
    }

    #[test]
    fn no_transition_accumulates_timer() {
        // elapsed = 0 + 1 = 1 < timer_start 2 → stay, accumulate.
        let r = tick_challenge_sweep(&SweepState::InitialWait, &input());
        assert_eq!(r.state, SweepState::InitialWait);
        assert_eq!(r.time_since_last_state_change, 1.0);
        assert!(r.events.is_empty());
    }

    #[test]
    fn finished_restarts_when_revalidation_fails() {
        let r = tick_challenge_sweep(
            &SweepState::Finished,
            &TickChallengeSweepInput {
                is_finished_still_valid: false,
                ..input()
            },
        );
        assert_eq!(r.state, SweepState::InitialWait);
    }
}
