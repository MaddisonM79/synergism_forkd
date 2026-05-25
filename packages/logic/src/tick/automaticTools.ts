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
 * The caller composes this with `checkAntSacrificeReady` (also in this
 * file) to decide whether to dispatch the `sacrificeAnts()` side effect.
 * The sacrifice itself stays in web_ui — it depends on un-migrated
 * subsystems (resetAnts, talisman inventory, achievement awards).
 */
export function advanceAntSacrificeTimers (
  input: AdvanceAntSacrificeTimersInput
): AdvanceAntSacrificeTimersResult {
  return {
    antSacrificeTimer: input.antSacrificeTimer + input.time * input.globalDelta,
    antSacrificeTimerReal: input.antSacrificeTimerReal + input.time
  }
}

/** Auto-sacrifice trigger modes. Mirrors the AutoSacrificeModes enum in
 * packages/web_ui/src/Features/Ants/toggles/structs/sacrifice.ts as a
 * string union — keeps the enum in web_ui (it's UI configuration) while
 * the logic API uses semantic names. */
export type AutoSacrificeMode = 'InGameTime' | 'RealTime' | 'ImmortalELOGain' | 'MaxRebornELO'

/** Crumbs threshold for the auto-sacrifice cooldown gate. Mirrors
 * MINIMUM_CRUMBS_FOR_SACRIFICE in
 * packages/web_ui/src/Features/Ants/AntSacrifice/constants.ts. */
export const MINIMUM_CRUMBS_FOR_SACRIFICE = 1e40
/** Seconds between auto-sacrifices. Mirrors
 * MINIMUM_SECONDS_DELAY_BETWEEN_SACRIFICES in the same constants file. */
export const MINIMUM_SECONDS_DELAY_BETWEEN_SACRIFICES = 0.05
/** "Reborn ELO maxed" tolerance — mirrors the inline 0.001 used by both
 * the maxRebornELO derivation and the MaxRebornELO mode's check. */
export const REBORN_ELO_MAXED_TOLERANCE = 0.001

export interface CheckAntSacrificeReadyInput {
  // ─── Mode gate ─────────────────────────────────────────────────────
  /** player.ants.toggles.autoSacrificeMode — picks which per-mode check
   * to run for the universal AND. */
  mode: AutoSacrificeMode

  // ─── Universal-check inputs ────────────────────────────────────────
  /** player.ants.crumbsThisSacrifice — passed through unchanged from
   * the legacy `crumbs` argument. Tested against
   * `MINIMUM_CRUMBS_FOR_SACRIFICE` via `Decimal.gte`. */
  crumbsThisSacrifice: Decimal
  /** Pre-advanced player.antSacrificeTimerReal — the wall-clock timer
   * after the per-tick advancement. Tested against
   * `MINIMUM_SECONDS_DELAY_BETWEEN_SACRIFICES`. */
  antSacrificeTimerReal: number
  /** player.ants.toggles.autoSacrificeEnabled — master gate; when
   * false, no mode can sacrifice. */
  autoSacrificeEnabled: boolean

  // ─── Max-reborn-ELO toggle inputs ──────────────────────────────────
  /** Pre-evaluated `calculateAvailableRebornELO()`. Drives the
   * `maxRebornELO = availableRebornELO < 0.001` derivation that the
   * `onlySacrificeMaxRebornELO` and `alwaysSacrificeMaxRebornELO`
   * toggles read. */
  availableRebornELO: number
  /** player.ants.toggles.onlySacrificeMaxRebornELO — when true with
   * `maxRebornELO` false, the entire check returns false (no mode
   * matters). */
  onlySacrificeMaxRebornELO: boolean
  /** player.ants.toggles.alwaysSacrificeMaxRebornELO — when true,
   * `maxRebornELO` alone satisfies the mode-specific check (universal
   * AND still applies). */
  alwaysSacrificeMaxRebornELO: boolean

  // ─── Mode-specific inputs ──────────────────────────────────────────
  /** Pre-advanced player.antSacrificeTimer — used by the `InGameTime`
   * mode's `sacrificeCheck`. */
  antSacrificeTimer: number
  /** player.ants.toggles.autoSacrificeThreshold — threshold shared by
   * the `InGameTime`, `RealTime`, and `ImmortalELOGain` modes. */
  autoSacrificeThreshold: number
  /** Pre-evaluated `antSacrificeRewards().immortalELO`
   * (`calculateImmortalELOGain()` from the logic shim). Used by the
   * `ImmortalELOGain` mode. */
  immortalELOGain: number
  /** player.ants.immortalELO. Used by the `MaxRebornELO` mode (delta
   * `immortalELO - rebornELO ≤ tolerance`). */
  immortalELO: number
  /** player.ants.rebornELO — the second half of the same delta. */
  rebornELO: number
}

export interface CheckAntSacrificeReadyResult {
  /** `[{ kind: 'ant-sacrifice-triggered' }]` when canAutoSacrifice's
   * conditions are met; `[]` otherwise. UI dispatcher invokes
   * `sacrificeAnts()` on the event. */
  events: CoreEvent[]
}

/**
 * Mirrors `canAutoSacrifice` in
 * packages/web_ui/src/Features/Ants/Automation/sacrifice.ts (boolean
 * predicate) — but emits an event instead of returning a boolean, to
 * match the established logic-tier convention used by every other
 * trigger case (autoPotion, auto-reset, etc.).
 *
 * Decision logic (bug-for-bug):
 *   1. If `onlySacrificeMaxRebornELO` is on AND we haven't maxed reborn
 *      ELO, no sacrifice — return `[]` immediately.
 *   2. Compute `universalChecks = crumbs ≥ MIN_CRUMBS && timerReal ≥
 *      MIN_DELAY && autoSacrificeEnabled`.
 *   3. Compute `specificCheck` per the active mode:
 *        - InGameTime:    antSacrificeTimer   ≥ autoSacrificeThreshold
 *        - RealTime:      antSacrificeTimerReal ≥ autoSacrificeThreshold
 *        - ImmortalELOGain: immortalELOGain  ≥ autoSacrificeThreshold
 *        - MaxRebornELO:  (immortalELO − rebornELO) ≤ 0.001
 *   4. If `alwaysSacrificeMaxRebornELO` is on:
 *        emit when `universalChecks && (maxRebornELO || specificCheck)`.
 *      Otherwise:
 *        emit when `universalChecks && specificCheck`.
 */
export function checkAntSacrificeReady (
  input: CheckAntSacrificeReadyInput
): CheckAntSacrificeReadyResult {
  const maxRebornELO = input.availableRebornELO < REBORN_ELO_MAXED_TOLERANCE

  if (input.onlySacrificeMaxRebornELO && !maxRebornELO) {
    return { events: [] }
  }

  const universalChecks = input.crumbsThisSacrifice.gte(MINIMUM_CRUMBS_FOR_SACRIFICE)
    && input.antSacrificeTimerReal >= MINIMUM_SECONDS_DELAY_BETWEEN_SACRIFICES
    && input.autoSacrificeEnabled

  let specificCheck = false
  switch (input.mode) {
    case 'InGameTime':
      specificCheck = input.antSacrificeTimer >= input.autoSacrificeThreshold
      break
    case 'RealTime':
      specificCheck = input.antSacrificeTimerReal >= input.autoSacrificeThreshold
      break
    case 'ImmortalELOGain':
      specificCheck = input.immortalELOGain >= input.autoSacrificeThreshold
      break
    case 'MaxRebornELO':
      specificCheck = (input.immortalELO - input.rebornELO) <= REBORN_ELO_MAXED_TOLERANCE
      break
  }

  const ready = input.alwaysSacrificeMaxRebornELO
    ? universalChecks && (maxRebornELO || specificCheck)
    : universalChecks && specificCheck

  return { events: ready ? [{ kind: 'ant-sacrifice-triggered' }] : [] }
}
