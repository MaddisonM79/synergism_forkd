//! Per-tick auto-research dispatch. Direct port of
//! `legacy/core_split/packages/logic/src/tick/autoResearch.ts`.
//!
//! Emits one intent event (manual single-slot vs cheapest "Roomba"); the
//! UI tier translates it into the `buyResearch` / Roomba while-loop side
//! effects. The two modes are mutually exclusive.

use smallvec::SmallVec;

use crate::events::CoreEvent;
use crate::mechanics::challenges::{calc_ecc, ChallengeType};
use crate::state::AutoResearchMode;

/// Inputs to [`process_auto_research_tick`].
pub(crate) struct ProcessAutoResearchTickInput {
    /// `player.autoResearchToggle` — master gate.
    pub auto_research_toggle: bool,
    /// `player.autoResearch` — selected research slot; `0` short-circuits.
    pub auto_research_selected: u32,
    /// `player.autoResearchMode` — manual vs cheapest.
    pub auto_research_mode: AutoResearchMode,
    /// Pre-evaluated `roombaResearchEnabled()` — Roomba-mode unlock.
    pub roomba_unlocked: bool,
    /// `player.challengecompletions[14]` — feeds `calc_ecc` for the
    /// Roomba `max_count`.
    pub challengecompletions_14: f64,
}

/// Per-tick auto-research dispatcher. Returns `AutoResearchManualRequested`
/// (manual mode) or `AutoResearchRoombaRequested { max_count }` (cheapest
/// mode, when the Roomba unlock passes); empty when the toggle is off, no
/// research is selected, or Roomba mode lacks its unlock.
pub(crate) fn process_auto_research_tick(
    input: &ProcessAutoResearchTickInput,
) -> SmallVec<[CoreEvent; 1]> {
    let mut events = SmallVec::new();
    if !input.auto_research_toggle || input.auto_research_selected == 0 {
        return events;
    }
    match input.auto_research_mode {
        AutoResearchMode::Manual => events.push(CoreEvent::AutoResearchManualRequested),
        AutoResearchMode::Cheapest => {
            if input.roomba_unlocked {
                let max_count =
                    1 + calc_ecc(ChallengeType::Ascension, input.challengecompletions_14).floor()
                        as u32;
                events.push(CoreEvent::AutoResearchRoombaRequested { max_count });
            }
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> ProcessAutoResearchTickInput {
        ProcessAutoResearchTickInput {
            auto_research_toggle: true,
            auto_research_selected: 5,
            auto_research_mode: AutoResearchMode::Manual,
            roomba_unlocked: false,
            challengecompletions_14: 0.0,
        }
    }

    #[test]
    fn no_event_when_toggle_off() {
        let e = process_auto_research_tick(&ProcessAutoResearchTickInput {
            auto_research_toggle: false,
            ..input()
        });
        assert!(e.is_empty());
    }

    #[test]
    fn no_event_when_no_research_selected() {
        let e = process_auto_research_tick(&ProcessAutoResearchTickInput {
            auto_research_selected: 0,
            ..input()
        });
        assert!(e.is_empty());
    }

    #[test]
    fn manual_mode_emits_manual_request() {
        let e = process_auto_research_tick(&input());
        assert_eq!(e.len(), 1);
        assert!(matches!(e[0], CoreEvent::AutoResearchManualRequested));
    }

    #[test]
    fn cheapest_mode_emits_roomba_when_unlocked() {
        let e = process_auto_research_tick(&ProcessAutoResearchTickInput {
            auto_research_mode: AutoResearchMode::Cheapest,
            roomba_unlocked: true,
            challengecompletions_14: 0.0,
            ..input()
        });
        assert_eq!(e.len(), 1);
        // calc_ecc(Ascension, 0) == 0 → max_count = 1.
        assert!(matches!(
            e[0],
            CoreEvent::AutoResearchRoombaRequested { max_count: 1 }
        ));
    }

    #[test]
    fn cheapest_mode_silent_without_roomba_unlock() {
        let e = process_auto_research_tick(&ProcessAutoResearchTickInput {
            auto_research_mode: AutoResearchMode::Cheapest,
            roomba_unlocked: false,
            ..input()
        });
        assert!(e.is_empty());
    }
}
