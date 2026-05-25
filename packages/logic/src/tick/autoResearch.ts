// Per-tick auto-research dispatch. Lifted from packages/web_ui/src/Synergism.ts
// (`tack` body, the two if-blocks at ~lines 4068-4110 — manual mode + Roomba).
//
// Each branch becomes a single CoreEvent the UI dispatcher translates back
// into the existing DOM-bound side effects:
//   manual:  buyResearch(autoResearch, true, false) + updateResearchAuto
//   cheapest: bounded while-loop reading isResearchUnlocked, buyResearch,
//             researchData[i].maxLevel, updateResearchRoomba — extracted into
//             runRoombaResearchSweep(maxCount) in web_ui.
//
// The two modes are mutually exclusive — only one branch can fire per tick.
// The maxCount calc for Roomba lives here because it's pure math
// (CalcECC is already in logic).

import type { CoreEvent } from '../events/types'
import { CalcECC } from '../mechanics/challenges'

export type AutoResearchMode = 'manual' | 'cheapest'

export interface ProcessAutoResearchTickInput {
  /** player.autoResearchToggle — master gate. When false, no branch fires. */
  autoResearchToggle: boolean
  /** player.autoResearch — current research index; gate (`> 0`) for both
   * modes. Zero means "no research selected" — short-circuit. */
  autoResearch: number
  /** player.autoResearchMode — selects manual vs Roomba branch. The
   * legacy code stores this as a string literal in the savefile. */
  autoResearchMode: AutoResearchMode
  /** Pre-evaluated `roombaResearchEnabled()` from web_ui — `cubeUpgrades[9]
   * === 1 || highestSingularityCount > 10`. Pre-eval'd by the caller so
   * the logic function stays decoupled from cube/singularity state. */
  roombaUnlocked: boolean
  /** player.challengecompletions[14] — feeds `CalcECC('ascension', x)`
   * for the Roomba `maxCount` (`1 + Math.floor(CalcECC(...))`). */
  challengecompletions14: number
}

export interface ProcessAutoResearchTickResult {
  /** Either:
   *   - `[]` — no branch fired (toggle off, no research selected, or
   *           Roomba mode without the unlock)
   *   - `[{ kind: 'auto-research-manual-requested' }]` — manual mode
   *   - `[{ kind: 'auto-research-roomba-requested', maxCount }]` — Roomba
   *
   * Never returns more than one event — the manual + Roomba branches are
   * mutually exclusive on `autoResearchMode`. */
  events: CoreEvent[]
}

/**
 * Per-tick auto-research dispatcher. Mirrors the two if-blocks at the
 * end of the !timeWarp section in `tack` (Synergism.ts).
 *
 * Manual mode (`autoResearchMode === 'manual'`):
 *   if (autoResearchToggle && autoResearch > 0)
 *     → emit `auto-research-manual-requested`
 *
 * Roomba mode (`autoResearchMode === 'cheapest'`):
 *   if (autoResearchToggle && autoResearch > 0 && roombaUnlocked)
 *     → emit `auto-research-roomba-requested` with maxCount =
 *       `1 + floor(CalcECC('ascension', challengecompletions14))`
 *
 * The actual `buyResearch` / `updateResearchAuto` / Roomba while-loop
 * remain in web_ui (DOM-bound) and are invoked by tickEventHandlers on
 * the respective event.
 */
export function processAutoResearchTick (
  input: ProcessAutoResearchTickInput
): ProcessAutoResearchTickResult {
  if (!input.autoResearchToggle || input.autoResearch <= 0) {
    return { events: [] }
  }
  if (input.autoResearchMode === 'manual') {
    return { events: [{ kind: 'auto-research-manual-requested' }] }
  }
  if (input.autoResearchMode === 'cheapest' && input.roombaUnlocked) {
    const maxCount = 1 + Math.floor(CalcECC('ascension', input.challengecompletions14))
    return { events: [{ kind: 'auto-research-roomba-requested', maxCount }] }
  }
  return { events: [] }
}
