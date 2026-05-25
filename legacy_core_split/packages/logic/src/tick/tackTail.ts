// Bundled "tail" of the per-tick body. Composes the three migrated
// subsystems that run AFTER the !timeWarp block in the legacy tack
// (Synergism.ts), in their fixed sequential order:
//
//   1. automaticTools('addOfferings', dt/2)  — gated by highestchallengecompletions[3] > 0
//   2. tickChallengeSweep(dt)
//   3. applyAutoResets(dt)  — emits auto-reset-triggered events
//
// Each sub-call is itself already in logic; this orchestrator just
// threads the inputs through and collects all CoreEvents into a single
// array. The web_ui adapter passes player + G + pre-eval'd lookups as
// one flat input bundle and writes the result back. Events flow through
// the central dispatcher in packages/web_ui/src/tickEventHandlers.ts.
//
// Hole context: `calculateOfferings()` runs in legacy tack AFTER this
// tail, but it's a vestigial no-op (computes a value and discards it),
// so the tail boundary stops at applyAutoResets.

import type { CoreEvent } from '../events/types'
import type { Decimal } from '../math/bignum'
import { applyAutoResets } from './autoReset'
import { addOfferings } from './automaticTools'
import { type SweepStates, tickChallengeSweep } from './challengeSweep'

export interface TackTailInput {
  /** Tick delta (seconds). addOfferings gets `dt/2`; sweep + resets get `dt`. */
  dt: number

  // ─── addOfferings inputs ────────────────────────────────────────────
  /** player.highestchallengecompletions[3] — gate (> 0 to run addOfferings).
   * When the gate fails, autoOfferingCounter/offerings pass through unchanged. */
  highestchallengecompletions3: number
  /** G.autoOfferingCounter. */
  autoOfferingCounter: number
  /** player.offerings. */
  offerings: Decimal

  // ─── tickChallengeSweep inputs (mirrors TickChallengeSweepInput minus dt) ─
  sweepState: SweepStates
  timeSinceLastStateChange: number
  shouldRunSweep: boolean
  timerStart: number
  timerExit: number
  timerEnter: number
  initialIndex: number
  nextRegularChallengeFromInitial: number
  nextRegularChallengeFromActive: number
  challenge15AutoExponentCheck: boolean
  isFinishedStillValid: boolean

  // ─── applyAutoResets inputs (mirrors ApplyAutoResetsInput minus dt) ─
  prestigeMode: 'amount' | 'time'
  toggle15: boolean
  autoPrestigeMilestone: number
  prestigePoints: Decimal
  prestigePointGain: Decimal
  prestigeamount: number
  coinsThisPrestige: Decimal
  autoResetTimerPrestige: number
  transcendMode: 'amount' | 'time'
  toggle21: boolean
  upgrade89: number
  transcendPoints: Decimal
  transcendPointGain: Decimal
  transcendamount: number
  coinsThisTranscension: Decimal
  autoResetTimerTranscension: number
  reincarnationMode: 'amount' | 'time'
  toggle27: boolean
  research46: number
  reincarnationPoints: Decimal
  reincarnationPointGain: Decimal
  reincarnationamount: number
  transcendShards: Decimal
  autoResetTimerReincarnation: number
  ascensionChallenge: number
  transcensionChallenge: number
  reincarnationChallenge: number
}

export interface TackTailResult {
  autoOfferingCounter: number
  offerings: Decimal
  sweepState: SweepStates
  timeSinceLastStateChange: number
  autoResetTimerPrestige: number
  autoResetTimerTranscension: number
  autoResetTimerReincarnation: number
  /** Composed event list — sweep transitions then auto-reset triggers,
   * in legacy order. (addOfferings emits nothing — no UI side effect.) */
  events: CoreEvent[]
}

/**
 * Pure composition of the per-tick tail. Mirrors lines ~4128-4192 of the
 * legacy tack body 1:1, just bundled so the web_ui adapter makes one
 * call instead of three (plus inline dispatchers).
 */
export function tackTail (input: TackTailInput): TackTailResult {
  const events: CoreEvent[] = []

  // ─── addOfferings (dt/2, gated) ─────────────────────────────────────
  let autoOfferingCounter = input.autoOfferingCounter
  let offerings = input.offerings
  if (input.highestchallengecompletions3 > 0) {
    const r = addOfferings({
      time: input.dt / 2,
      autoOfferingCounter,
      offerings
    })
    autoOfferingCounter = r.autoOfferingCounter
    offerings = r.offerings
  }

  // ─── tickChallengeSweep ─────────────────────────────────────────────
  const sweep = tickChallengeSweep({
    dt: input.dt,
    state: input.sweepState,
    timeSinceLastStateChange: input.timeSinceLastStateChange,
    shouldRunSweep: input.shouldRunSweep,
    timerStart: input.timerStart,
    timerExit: input.timerExit,
    timerEnter: input.timerEnter,
    initialIndex: input.initialIndex,
    nextRegularChallengeFromInitial: input.nextRegularChallengeFromInitial,
    nextRegularChallengeFromActive: input.nextRegularChallengeFromActive,
    challenge15AutoExponentCheck: input.challenge15AutoExponentCheck,
    isFinishedStillValid: input.isFinishedStillValid
  })
  for (const e of sweep.events) events.push(e)

  // ─── applyAutoResets ────────────────────────────────────────────────
  const resets = applyAutoResets({
    dt: input.dt,
    prestigeMode: input.prestigeMode,
    toggle15: input.toggle15,
    autoPrestigeMilestone: input.autoPrestigeMilestone,
    prestigePoints: input.prestigePoints,
    prestigePointGain: input.prestigePointGain,
    prestigeamount: input.prestigeamount,
    coinsThisPrestige: input.coinsThisPrestige,
    autoResetTimerPrestige: input.autoResetTimerPrestige,
    transcendMode: input.transcendMode,
    toggle21: input.toggle21,
    upgrade89: input.upgrade89,
    transcendPoints: input.transcendPoints,
    transcendPointGain: input.transcendPointGain,
    transcendamount: input.transcendamount,
    coinsThisTranscension: input.coinsThisTranscension,
    autoResetTimerTranscension: input.autoResetTimerTranscension,
    reincarnationMode: input.reincarnationMode,
    toggle27: input.toggle27,
    research46: input.research46,
    reincarnationPoints: input.reincarnationPoints,
    reincarnationPointGain: input.reincarnationPointGain,
    reincarnationamount: input.reincarnationamount,
    transcendShards: input.transcendShards,
    autoResetTimerReincarnation: input.autoResetTimerReincarnation,
    ascensionChallenge: input.ascensionChallenge,
    transcensionChallenge: input.transcensionChallenge,
    reincarnationChallenge: input.reincarnationChallenge
  })
  for (const e of resets.events) events.push(e)

  return {
    autoOfferingCounter,
    offerings,
    sweepState: sweep.state,
    timeSinceLastStateChange: sweep.timeSinceLastStateChange,
    autoResetTimerPrestige: resets.autoResetTimerPrestige,
    autoResetTimerTranscension: resets.autoResetTimerTranscension,
    autoResetTimerReincarnation: resets.autoResetTimerReincarnation,
    events
  }
}
