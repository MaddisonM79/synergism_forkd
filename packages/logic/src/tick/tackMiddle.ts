// Bundled "middle"-side composition for the per-tick body. Composes the
// four migrated middle-section cases — runeSacrifice, antSacrifice,
// addObtainium-or-recompute, and processAutoResearchTick — into a single
// logic call so the web_ui adapter makes one call instead of four
// gate-checked dispatches.
//
// Mirrors the four blocks between `tackHeadTimers` and `tackTail` in the
// per-tick body (packages/web_ui/src/Synergism.ts, `tack`):
//
//   1. runeSacrifice — gated by `autoSacrificeToggle && offeringAutoRune`
//                       (pre-evaluated as `runeSacrificeEnabled`).
//   2. antSacrifice  — gated by `getAchievementReward('antSacrificeUnlock')`
//                       (pre-evaluated as `antSacrificeUnlocked`).
//   3. Obtainium     — `research61 === 1` → addObtainium; else emit
//                       `obtainium-multiplier-recompute-requested` so the
//                       dispatcher runs the legacy `calculateObtainium()`
//                       vestigial-calc arm.
//   4. Auto-research — processAutoResearchTick (manual + Roomba modes).
//
// All event ordering matches the legacy in-place tack: runeSacrifice
// events first, then antSacrifice, then obtainium, then auto-research.

import type { CoreEvent } from '../events/types'
import type { Decimal } from '../math/bignum'
import {
  addObtainium,
  advanceAntSacrificeTimers,
  advanceRuneSacrifice,
  type AutoSacrificeMode,
  checkAntSacrificeReady
} from './automaticTools'
import { type AutoResearchMode, processAutoResearchTick } from './autoResearch'

export interface TackMiddleInput {
  /** Tick delta (seconds). */
  dt: number

  // ─── 1. runeSacrifice ────────────────────────────────────────────
  /** Pre-evaluated `player.autoSacrificeToggle &&
   * getShopUpgradeEffects('offeringAuto', 'autoRune')`. The whole case is
   * skipped when false. */
  runeSacrificeEnabled: boolean
  /** player.sacrificeTimer. */
  sacrificeTimer: number
  /** web_ui module-local autoSacrificeInterval cache. Refreshed by
   * `executeRuneAutoSacrifice` in the UI dispatcher when the event fires. */
  autoSacrificeInterval: number
  /** player.offerings — gate (`> 0`) inside advanceRuneSacrifice. Also
   * read by the dispatcher's purchase fan-out. */
  offerings: Decimal

  // ─── 2. antSacrifice ─────────────────────────────────────────────
  /** Pre-evaluated `getAchievementReward('antSacrificeUnlock')`. The
   * whole case is skipped when false. */
  antSacrificeUnlocked: boolean
  /** Pre-evaluated `getGQUpgradeEffect('halfMind', 'unlocked') ? 10 :
   * calculateGlobalSpeedMult()`. Applied to the scaled
   * antSacrificeTimer only. */
  globalDelta: number
  /** player.antSacrificeTimer. */
  antSacrificeTimer: number
  /** player.antSacrificeTimerReal. */
  antSacrificeTimerReal: number
  /** Translated `player.ants.toggles.autoSacrificeMode` (numeric enum
   * → string union). */
  autoSacrificeMode: AutoSacrificeMode
  /** player.ants.crumbsThisSacrifice. */
  crumbsThisSacrifice: Decimal
  /** player.ants.toggles.autoSacrificeEnabled. */
  autoSacrificeEnabled: boolean
  /** Pre-evaluated `calculateAvailableRebornELO()`. */
  availableRebornELO: number
  /** player.ants.toggles.onlySacrificeMaxRebornELO. */
  onlySacrificeMaxRebornELO: boolean
  /** player.ants.toggles.alwaysSacrificeMaxRebornELO. */
  alwaysSacrificeMaxRebornELO: boolean
  /** player.ants.toggles.autoSacrificeThreshold. */
  autoSacrificeThreshold: number
  /** Pre-evaluated `antSacrificeRewards().immortalELO` — only meaningful
   * for `mode === 'ImmortalELOGain'`. Caller passes 0 for other modes
   * (the check function only reads it when the mode demands it). */
  immortalELOGain: number
  /** player.ants.immortalELO. */
  immortalELO: number
  /** player.ants.rebornELO. */
  rebornELO: number

  // ─── 3. Obtainium ────────────────────────────────────────────────
  /** player.researches[61] — `=== 1` routes to addObtainium; otherwise
   * the dispatcher recomputes the obtainium multiplier (legacy vestigial
   * call). */
  research61: number
  /** player.obtainium. */
  obtainium: Decimal
  /** Pre-evaluated `calculateResearchAutomaticObtainium(dt)`. Unused
   * when research61 !== 1 (caller can pass `new Decimal(0)`). */
  obtainiumGain: Decimal
  /** player.currentChallenge.ascension — addObtainium aborts in c14. */
  ascensionChallenge: number
  /** player.singularityChallenges.taxmanLastStand.enabled. */
  taxmanLastStandEnabled: boolean
  /** player.singularityChallenges.taxmanLastStand.completions. */
  taxmanLastStandCompletions: number

  // ─── 4. Auto-research ────────────────────────────────────────────
  /** player.autoResearchToggle — master gate. */
  autoResearchToggle: boolean
  /** player.autoResearch. */
  autoResearch: number
  /** player.autoResearchMode. */
  autoResearchMode: AutoResearchMode
  /** Pre-evaluated `roombaResearchEnabled()`. */
  roombaUnlocked: boolean
  /** player.challengecompletions[14] — feeds CalcECC for Roomba maxCount. */
  challengecompletions14: number
}

export interface TackMiddleResult {
  /** post-runeSacrifice — either advanced or reset to 0 (gate fired). */
  sacrificeTimer: number
  /** post-antSacrifice timer-advance (scaled). Unchanged when
   * antSacrificeUnlocked === false. */
  antSacrificeTimer: number
  /** post-antSacrifice timer-advance (raw). Unchanged when
   * antSacrificeUnlocked === false. */
  antSacrificeTimerReal: number
  /** post-addObtainium balance. Unchanged when research61 !== 1 or the
   * c14 abort fires inside addObtainium. */
  obtainium: Decimal
  /** Composed event list, in legacy tack-body order:
   *   1. rune-sacrifice-triggered           (case 1, when gate fires)
   *   2. ant-sacrifice-triggered            (case 2, when ready)
   *   3. auto-tool-fired (addObtainium)     (case 3 / research61===1)
   *      OR obtainium-multiplier-recompute-requested
   *                                         (case 3 / else branch)
   *   4. auto-research-manual-requested     (case 4, manual mode)
   *      OR auto-research-roomba-requested  (case 4, cheapest mode) */
  events: CoreEvent[]
}

/**
 * Pure composition of the four migrated middle-section cases. Mirrors
 * the inline tack body 1:1 — see the file header for the case-by-case
 * mapping.
 *
 * Cases skip cleanly when their pre-evaluated gates fail:
 *   - runeSacrificeEnabled === false → sacrificeTimer unchanged, no event
 *   - antSacrificeUnlocked === false → both ant timers unchanged, no event
 *   - research61 !== 1               → obtainium unchanged, emit recompute event
 *   - autoResearch === 0 / toggle off → no event
 */
export function tackMiddle (input: TackMiddleInput): TackMiddleResult {
  const events: CoreEvent[] = []

  // ─── 1. runeSacrifice ──────────────────────────────────────────
  let sacrificeTimer = input.sacrificeTimer
  if (input.runeSacrificeEnabled) {
    const r = advanceRuneSacrifice({
      time: input.dt,
      sacrificeTimer,
      autoSacrificeInterval: input.autoSacrificeInterval,
      offerings: input.offerings
    })
    sacrificeTimer = r.sacrificeTimer
    for (const e of r.events) events.push(e)
  }

  // ─── 2. antSacrifice ───────────────────────────────────────────
  let antSacrificeTimer = input.antSacrificeTimer
  let antSacrificeTimerReal = input.antSacrificeTimerReal
  if (input.antSacrificeUnlocked) {
    const timerR = advanceAntSacrificeTimers({
      time: input.dt,
      globalDelta: input.globalDelta,
      antSacrificeTimer,
      antSacrificeTimerReal
    })
    antSacrificeTimer = timerR.antSacrificeTimer
    antSacrificeTimerReal = timerR.antSacrificeTimerReal

    const checkR = checkAntSacrificeReady({
      mode: input.autoSacrificeMode,
      crumbsThisSacrifice: input.crumbsThisSacrifice,
      antSacrificeTimerReal,
      autoSacrificeEnabled: input.autoSacrificeEnabled,
      availableRebornELO: input.availableRebornELO,
      onlySacrificeMaxRebornELO: input.onlySacrificeMaxRebornELO,
      alwaysSacrificeMaxRebornELO: input.alwaysSacrificeMaxRebornELO,
      antSacrificeTimer,
      autoSacrificeThreshold: input.autoSacrificeThreshold,
      immortalELOGain: input.immortalELOGain,
      immortalELO: input.immortalELO,
      rebornELO: input.rebornELO
    })
    for (const e of checkR.events) events.push(e)
  }

  // ─── 3. Obtainium branch ───────────────────────────────────────
  let obtainium = input.obtainium
  if (input.research61 === 1) {
    const obtR = addObtainium({
      obtainium,
      obtainiumGain: input.obtainiumGain,
      ascensionChallenge: input.ascensionChallenge,
      taxmanLastStandEnabled: input.taxmanLastStandEnabled,
      taxmanLastStandCompletions: input.taxmanLastStandCompletions
    })
    obtainium = obtR.obtainium
    for (const e of obtR.events) events.push(e)
  } else {
    events.push({ kind: 'obtainium-multiplier-recompute-requested' })
  }

  // ─── 4. Auto-research ──────────────────────────────────────────
  const arR = processAutoResearchTick({
    autoResearchToggle: input.autoResearchToggle,
    autoResearch: input.autoResearch,
    autoResearchMode: input.autoResearchMode,
    roombaUnlocked: input.roombaUnlocked,
    challengecompletions14: input.challengecompletions14
  })
  for (const e of arR.events) events.push(e)

  return {
    sacrificeTimer,
    antSacrificeTimer,
    antSacrificeTimerReal,
    obtainium,
    events
  }
}
