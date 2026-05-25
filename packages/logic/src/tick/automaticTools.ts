// Per-tick auto-tool branches. Lifted from packages/web_ui/src/Helper.ts
// (automaticTools, addObtainium + addOfferings + antSacrifice timer cases).
//
// Covers the pure state mutations of three automaticTools cases:
//   addObtainium             — clamps + adds research-automatic obtainium,
//                              gated by c14 abort and taxmanLastStand
//                              singularity-challenge cap.
//   addOfferings             — fractional auto-offering counter; floor of
//                              accumulated counter is moved into
//                              player.offerings each tick.
//   advanceAntSacrificeTimers — dual antSacrificeTimer / antSacrificeTimerReal
//                              advancement (scaled vs. raw). The
//                              canAutoSacrifice check + sacrificeAnts() side
//                              effect stay in web_ui because they fan out
//                              into un-migrated subsystems (calculateAvailable
//                              RebornELO, antSacrificeRewards, resetAnts,
//                              updateTalismanInventory, achievement awards).
//
// The runeSacrifice case stays entirely in web_ui — its sacrificeTimer
// advancement is one trivial line and the rest is purchase orchestration
// across multiple un-migrated subsystems (RuneBlessings, RuneSpirits,
// Talismans, sacrificeOfferings).
//
// Caller pre-evaluates the obtainium gain by calling the existing
// `calculateResearchAutomaticObtainium(time)` shim (itself already a logic
// call) and passes the result in — that keeps automaticTools.ts decoupled
// from the giant tax/ant/research input bundle.

import type { CoreEvent } from '../events/types'
import { Decimal } from '../math/bignum'

export interface AddObtainiumInput {
  /** player.obtainium — current obtainium balance to credit the gain against. */
  obtainium: Decimal
  /** Pre-evaluated `calculateResearchAutomaticObtainium(time)` — the per-tick
   * obtainium gain before the taxmanLastStand clamp is applied. */
  obtainiumGain: Decimal
  /** player.currentChallenge.ascension — when === 14, the entire branch
   * short-circuits and obtainium is unchanged (no event emitted). */
  ascensionChallenge: number
  /** player.singularityChallenges.taxmanLastStand.enabled — combined with the
   * completions check below, gates a runaway-prevention clamp on the gain. */
  taxmanLastStandEnabled: boolean
  /** player.singularityChallenges.taxmanLastStand.completions — when ≥ 2,
   * clamps obtainiumGain to `min(obtainiumGain, obtainium * 100 + 1)`. */
  taxmanLastStandCompletions: number
}

export interface AddObtainiumResult {
  obtainium: Decimal
  /** `auto-tool-fired` event when the branch did real work (not c14-aborted),
   * empty otherwise. The UI handler reacts by refreshing the Research tab
   * visual when that tab is open. */
  events: CoreEvent[]
}

/**
 * Apply the per-tick automatic obtainium gain with the c14 abort + the
 * taxmanLastStand singularity-challenge clamp. Mirrors the legacy
 * `addObtainium` case of `automaticTools` 1:1.
 */
export function addObtainium (input: AddObtainiumInput): AddObtainiumResult {
  if (input.ascensionChallenge === 14) {
    return { obtainium: input.obtainium, events: [] }
  }

  let gain = input.obtainiumGain
  if (input.taxmanLastStandEnabled && input.taxmanLastStandCompletions >= 2) {
    gain = Decimal.min(gain, input.obtainium.times(100).plus(1))
  }

  return {
    obtainium: input.obtainium.add(gain),
    events: [{ kind: 'auto-tool-fired', tool: 'addObtainium' }]
  }
}

export interface AddOfferingsInput {
  /** Tick delta (seconds). Added to autoOfferingCounter raw. */
  time: number
  /** G.autoOfferingCounter — fractional carry between ticks. Reduced mod 1
   * after each tick so only the integer overflow is moved into offerings. */
  autoOfferingCounter: number
  /** player.offerings — receives `floor(autoOfferingCounter + time)`
   * additional offerings each tick. */
  offerings: Decimal
}

export interface AddOfferingsResult {
  autoOfferingCounter: number
  offerings: Decimal
}

/**
 * Fractional auto-offering counter: any whole-number portion of the
 * accumulated counter is moved into `offerings`, and the fractional
 * remainder stays for next tick. Mirrors the legacy `addOfferings` case
 * of `automaticTools` 1:1. No event is emitted — the legacy branch has
 * no UI-tier side effect, so the per-tick offering credit is observed
 * via the resources display's normal refresh cycle.
 */
export function addOfferings (input: AddOfferingsInput): AddOfferingsResult {
  const advanced = input.autoOfferingCounter + input.time
  return {
    autoOfferingCounter: advanced % 1,
    offerings: input.offerings.add(Math.floor(advanced))
  }
}

export interface AdvanceAntSacrificeTimersInput {
  /** Tick delta (seconds). */
  time: number
  /** Pre-evaluated `getGQUpgradeEffect('halfMind', 'unlocked') ? 10 :
   * calculateGlobalSpeedMult()`. Applied to `antSacrificeTimer` only —
   * `antSacrificeTimerReal` advances by raw `time` regardless. */
  globalDelta: number
  /** player.antSacrificeTimer — scaled in-game timer, advances by
   * `time * globalDelta`. Drives the InGameTime auto-sacrifice mode and
   * the crumbs-per-second metric in sacrifice history. */
  antSacrificeTimer: number
  /** player.antSacrificeTimerReal — raw wall-clock timer, advances by
   * `time` only. Drives the RealTime auto-sacrifice mode and the
   * sacrificeOffCooldown check. */
  antSacrificeTimerReal: number
}

export interface AdvanceAntSacrificeTimersResult {
  antSacrificeTimer: number
  antSacrificeTimerReal: number
}

/**
 * Advance the dual ant-sacrifice timers. Mirrors the timer-advancement
 * lines of the legacy `automaticTools('antSacrifice', time)` case:
 *
 *   player.antSacrificeTimer     += time * globalDelta
 *   player.antSacrificeTimerReal += time
 *
 * The caller still runs the `canAutoSacrifice` check + `sacrificeAnts()`
 * side effect after applying this result — both depend on un-migrated
 * web_ui subsystems (calculateAvailableRebornELO, antSacrificeRewards,
 * resetAnts, achievement awards) and are not part of the pure tick body.
 */
export function advanceAntSacrificeTimers (
  input: AdvanceAntSacrificeTimersInput
): AdvanceAntSacrificeTimersResult {
  return {
    antSacrificeTimer: input.antSacrificeTimer + input.time * input.globalDelta,
    antSacrificeTimerReal: input.antSacrificeTimerReal + input.time
  }
}
