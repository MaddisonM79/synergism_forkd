// Per-tick auto-tool branches. Lifted from packages/web_ui/src/Helper.ts
// (automaticTools, addObtainium + addOfferings cases).
//
// Covers the two simple cases of automaticTools — counter math with at most
// one UI-tier visual refresh:
//   addObtainium  — clamps + adds research-automatic obtainium, gated by c14
//                   abort and taxmanLastStand singularity-challenge cap.
//   addOfferings  — fractional auto-offering counter; floor of accumulated
//                   counter is moved into player.offerings each tick.
//
// The two complex cases (runeSacrifice, antSacrifice) stay in web_ui — they
// fan out into multiple un-migrated subsystems (RuneBlessings, RuneSpirits,
// Talismans, sacrificeOfferings on the rune side; sacrificeAnts on the ant
// side). They'll migrate once those subsystems land in logic.
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
