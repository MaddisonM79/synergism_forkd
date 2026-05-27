// Phase 4 per-tick orchestrator. Bundles the three migrated tick bundles
// (advanceAllTimers / tackMiddle / tackTail) into a single logic call.
//
// Mirrors the structural shape of the legacy `tack(dt)` in
// packages/web_ui/src/Synergism.ts:
//
//   if (!G.timeWarp) {
//     ... pre-tick orchestration: calculateGlobalSpeedMult,
//         resourceGain, generateAntsAndCrumbs ... (stays in web_ui)
//     tackHeadTimers(dt)   -> advanceAllTimers (this bundle)
//     tackMiddleInline()   -> tackMiddle      (this bundle)
//   }
//   tackTailInline()        -> tackTail        (always — runs regardless of timeWarp)
//
// resourceGain + generateAntsAndCrumbs remain as separate web_ui calls
// because each has its own pre-tick orchestration that mutates G state
// the rest of the tick reads (calculateTotalAcceleratorBoost,
// updateAllTick, updateAllMultiplier, multipliers, calculatetax,
// resetCurrency).
//
// ─── Event-dispatch ordering caveat ─────────────────────────────────────
//
// In the legacy in-place tick, side-effect events fired *interleaved* with
// the tick body — useConsumable (from autoPotion) ran inside the head
// timer sweep, executeRuneAutoSacrifice ran inside the middle, etc. Each
// subsequent subsystem read the *post-event* state.
//
// In this bundle, all events accumulate and dispatch *after* the entire
// tick body returns. This shifts a small amount of state observation:
//
//   - The middle's runeSacrifice gate reads `offerings` from *before* the
//     head autoPotion event grants any.
//   - The tail's addOfferings adds to `offerings` from *before* the middle
//     runeSacrifice event spends any.
//
// The drift per tick is tiny (a fractional re-ordering of offerings/
// obtainium grants and spends) and self-correcting across ticks: any
// difference one tick gets observed and incorporated the next. The trade
// is one tick of latency for a clean, callback-free pure-logic boundary
// suitable for the eventual Rust port. The parity test in
// `tackBody.parity.test.ts` verifies the bundle equals "the 3 logic calls
// in sequence without intermediate dispatch" — not legacy in-place
// behavior.

import type { CoreEvent } from '../events/types'
import type { AdvanceAllTimersInput, AdvanceAllTimersResult } from './timersBundle'
import { advanceAllTimers } from './timersBundle'
import type { TackMiddleInput, TackMiddleResult } from './tackMiddle'
import { tackMiddle } from './tackMiddle'
import type { TackTailInput, TackTailResult } from './tackTail'
import { tackTail } from './tackTail'

export interface TackBodyInput {
  /** `G.timeWarp` — when true, skip the head + middle bundles and only
   * run the tail (matches the legacy `if (!G.timeWarp) { ... }` shape). */
  timeWarp: boolean
  /** Pre-evaluated `AdvanceAllTimersInput` for the head bundle. Required
   * when `timeWarp === false`; the caller can omit it (or pass undefined)
   * when `timeWarp === true` to avoid the per-tick pre-eval cost. */
  head?: AdvanceAllTimersInput
  /** Pre-evaluated `TackMiddleInput` for the middle bundle. Required
   * when `timeWarp === false`; omitted/undefined when `timeWarp === true`. */
  middle?: TackMiddleInput
  /** Pre-evaluated `TackTailInput` for the tail bundle — runs every
   * tick regardless of `timeWarp`. */
  tail: TackTailInput
}

export interface TackBodyResult {
  /** Head writebacks. `undefined` when the head bundle was skipped
   * (timeWarp === true). */
  head: AdvanceAllTimersResult | undefined
  /** Middle writebacks. `undefined` when the middle bundle was skipped
   * (timeWarp === true). */
  middle: TackMiddleResult | undefined
  /** Tail writebacks. Always present — tail runs every tick. */
  tail: TackTailResult
  /** Composed event list in legacy bundle order: head events, then
   * middle events, then tail events. When `timeWarp === true`, only
   * tail events appear. */
  events: CoreEvent[]
}

/**
 * Phase 4 per-tick body. Composes advanceAllTimers + tackMiddle + tackTail
 * 1:1 with the legacy tack walk:
 *
 *   1. If `!timeWarp`:
 *        - run `advanceAllTimers(input.head)` (the 11 timer cases)
 *        - run `tackMiddle(input.middle)` (rune/ant sacrifice, addObtainium,
 *          auto-research dispatch)
 *   2. Always run `tackTail(input.tail)` (addOfferings, sweep, auto-reset)
 *
 * Returns the per-bundle writebacks plus a flat event list ready for the
 * UI dispatcher. The caller is responsible for applying the writebacks
 * back to player/G state and dispatching the events.
 */
export function tackBody (input: TackBodyInput): TackBodyResult {
  const events: CoreEvent[] = []

  let head: AdvanceAllTimersResult | undefined
  let middle: TackMiddleResult | undefined
  if (!input.timeWarp) {
    if (input.head === undefined || input.middle === undefined) {
      throw new Error(
        'tackBody: head and middle inputs are required when timeWarp === false'
      )
    }
    head = advanceAllTimers(input.head)
    for (const e of head.events) events.push(e)
    middle = tackMiddle(input.middle)
    for (const e of middle.events) events.push(e)
  }
  const tail = tackTail(input.tail)
  for (const e of tail.events) events.push(e)

  return { head, middle, tail, events }
}
