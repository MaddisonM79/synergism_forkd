// Per-tick challenge-sweep state machine. Lifted from
// packages/web_ui/src/Challenges.ts (`tickChallengeSweep`,
// `sweepTransitionFunc`, `SweepStates`, ~lines 395-575 pre-migration).
//
// The state machine cycles the player through Transcension + Reincarnation
// challenges automatically when the autoChallenge feature is enabled. It
// holds two pieces of mutable state — the current SweepState (one of six
// variants) and the wall-clock since the last state change — both of which
// live in module-locals in web_ui pre-migration. The migrated version
// takes both as input and returns them as output; web_ui keeps the
// module-locals and threads them through.
//
// Side effects (DOM toggleAutoChallengeModeText, async resetCheck on
// challenge exit, toggleChallenges to enter a new challenge) become
// `challenge-sweep-transitioned` events that the UI dispatcher
// translates back into the corresponding calls. The event carries the
// full SweepStates `from` and `to` so the dispatcher can route
// resetCheck by oldState.index (when active) and toggleChallenges by
// newState.index.
//
// External lookups that the transition function needs are pre-evaluated
// by the caller and passed in:
//   - `nextRegularChallengeFromInitial` / `nextRegularChallengeFromActive`
//     wrap the (already-migrated) `getNextRegularChallenge` for the two
//     transitions that consume it.
//   - `challenge15AutoExponentCheck` is the pre-evaluated guard for the
//     active → c15_wait branch.
//   - `isFinishedStillValid` covers the `finished` revalidation check
//     (both c1 and c6 still at max completions).

import type { CoreEvent } from '../events/types'

// ─── State variants ─────────────────────────────────────────────────────

export type SweepStates =
  | { kind: 'idle' }
  | { kind: 'initial_wait' }
  // `enter_wait` and `active` carry the `explored` set so a single cycle
  // doesn't repeat challenges. `c10_detour` (legacy comment in
  // getNextRegularChallenge) uses the explored set to avoid retrying.
  | { kind: 'enter_wait', toIndex: number, explored: Set<number> }
  | { kind: 'active', index: number, explored: Set<number> }
  // `c15_wait` is the 5-second pause when the player can auto-gain
  // challenge-15 exponent (autoAscend + cubeUpgrade 10 + realAscensionTime
  // mode), letting the c15 exponent climb between sweeps.
  | { kind: 'c15_wait' }
  // `finished` parks the machine once every regular challenge (1-10) is
  // maxed; re-enters `initial_wait` if a max-completions cap changes.
  | { kind: 'finished' }

// ─── tickChallengeSweep ─────────────────────────────────────────────────

export interface TickChallengeSweepInput {
  /** Tick delta in seconds (same `dt` passed to other tick subsystems). */
  dt: number

  // ─── Current state (mutable bookkeeping fed back in each tick) ──────

  /** Reference to the current SweepState. The transition function compares
   * with `===` to detect a real transition (some branches return the
   * same reference to mean "no change"). */
  state: SweepStates
  /** Wall-clock seconds since the last state change. Compared against
   * timerStart/timerExit/timerEnter (and a hardcoded 5 for c15_wait). */
  timeSinceLastStateChange: number

  // ─── Enable gate (pre-eval'd `shouldRunSweep()`) ────────────────────

  /** Pre-evaluated `player.researches[150] > 0 && player.autoChallengeRunning`.
   * When this flips false while sweep is running, the machine resets to
   * idle. When it flips true while idle, the machine boots to
   * initial_wait. */
  shouldRunSweep: boolean

  // ─── Timer thresholds (player.autoChallengeTimer) ───────────────────

  /** initial_wait → active threshold. */
  timerStart: number
  /** active → next-stage threshold (enter_wait or initial_wait or c15_wait). */
  timerExit: number
  /** enter_wait → active threshold. */
  timerEnter: number

  // ─── Pre-eval'd transition lookups ──────────────────────────────────

  /** Initial index for the first sweep of a cycle: 1 by default, 10 when
   * highestSingularityCount ≥ 2 AND currentChallenge.ascension !== 0.
   * The caller pre-computes this and the corresponding
   * `getNextRegularChallenge(initialIndex, new Set())` result. Only
   * consulted in the `initial_wait` branch. */
  initialIndex: number
  /** `getNextRegularChallenge(initialIndex, new Set())` — -1 means every
   * regular challenge is already maxed, in which case the machine
   * transitions to `finished`. Only consulted in `initial_wait`. */
  nextRegularChallengeFromInitial: number

  /** `getNextRegularChallenge(state.index, state.explored)` for the
   * `active` branch — -1 means the cycle is exhausted and the next
   * transition is either `c15_wait` (if challenge15AutoExponentCheck
   * passes) or `initial_wait`. */
  nextRegularChallengeFromActive: number

  /** Pre-evaluated `challenge15AutoExponentCheck()` — gates the
   * `active` exhausted → `c15_wait` branch vs. plain restart. */
  challenge15AutoExponentCheck: boolean

  /** Pre-evaluated guard for the `finished` revalidation: true when
   * `highestchallengecompletions[1] === getMaxChallenges(1)` AND the
   * same for index 6. When false, the machine restarts the sweep. */
  isFinishedStillValid: boolean
}

export interface TickChallengeSweepResult {
  state: SweepStates
  timeSinceLastStateChange: number
  /** `challenge-sweep-transitioned` events (0 or 1 per tick) carrying the
   * full from/to states so the UI dispatcher can fire the right
   * resetCheck (based on from.index when from.kind === 'active') and
   * toggleChallenges call (based on to.index when to.kind === 'active'). */
  events: CoreEvent[]
}

/**
 * Pure transition function. Given the current state, the wall-clock
 * elapsed since the last transition, and the pre-eval'd lookups,
 * returns either the same state reference (no transition) or a new
 * state object (transition). Caller compares with `===` to detect
 * a real change.
 */
function sweepTransition (state: SweepStates, elapsed: number, input: TickChallengeSweepInput): SweepStates {
  switch (state.kind) {
    case 'idle':
      // Externally toggled in/out — no time-based transition.
      return state

    case 'initial_wait':
      if (elapsed >= input.timerStart) {
        if (input.nextRegularChallengeFromInitial === -1) {
          // Every regular challenge maxed → park in `finished`.
          return { kind: 'finished' }
        }
        return {
          kind: 'active',
          index: input.nextRegularChallengeFromInitial,
          explored: new Set([input.nextRegularChallengeFromInitial])
        }
      }
      return state

    case 'active':
      if (elapsed >= input.timerExit) {
        if (input.nextRegularChallengeFromActive === -1) {
          // Cycle exhausted.
          if (input.challenge15AutoExponentCheck) {
            return { kind: 'c15_wait' }
          }
          return { kind: 'initial_wait' }
        }
        return {
          kind: 'enter_wait',
          toIndex: input.nextRegularChallengeFromActive,
          explored: state.explored
        }
      }
      return state

    case 'enter_wait':
      if (elapsed >= input.timerEnter) {
        return {
          kind: 'active',
          index: state.toIndex,
          explored: new Set([...state.explored, state.toIndex])
        }
      }
      return state

    case 'c15_wait':
      if (elapsed >= 5) {
        return { kind: 'initial_wait' }
      }
      return state

    case 'finished':
      // Revalidate against current caps — restart sweep if a max changed.
      if (input.isFinishedStillValid) {
        return state
      }
      return { kind: 'initial_wait' }

    default: {
      throw new Error(`Unhandled SweepState kind: ${(state as { kind: string }).kind}`)
    }
  }
}

/**
 * Per-tick driver. Handles the four cases from the legacy body:
 *   1. Idle + shouldRunSweep → boot to initial_wait.
 *   2. Running + !shouldRunSweep → tear down to idle.
 *   3. !shouldRunSweep + already idle → no-op.
 *   4. Running → accumulate timer + sweepTransition + emit event on change.
 */
export function tickChallengeSweep (input: TickChallengeSweepInput): TickChallengeSweepResult {
  const wasEnabled = input.state.kind !== 'idle'
  const isEnabled = input.shouldRunSweep

  if (!wasEnabled && isEnabled) {
    const from: SweepStates = { kind: 'idle' }
    const to: SweepStates = { kind: 'initial_wait' }
    return {
      state: to,
      timeSinceLastStateChange: 0,
      events: [{ kind: 'challenge-sweep-transitioned', from, to }]
    }
  }

  if (wasEnabled && !isEnabled) {
    const from = input.state
    const to: SweepStates = { kind: 'idle' }
    return {
      state: to,
      timeSinceLastStateChange: 0,
      events: [{ kind: 'challenge-sweep-transitioned', from, to }]
    }
  }

  if (!isEnabled) {
    return {
      state: input.state,
      timeSinceLastStateChange: input.timeSinceLastStateChange,
      events: []
    }
  }

  const elapsed = input.timeSinceLastStateChange + input.dt
  const newState = sweepTransition(input.state, elapsed, input)

  if (newState !== input.state) {
    return {
      state: newState,
      timeSinceLastStateChange: 0,
      events: [{ kind: 'challenge-sweep-transitioned', from: input.state, to: newState }]
    }
  }

  return {
    state: input.state,
    timeSinceLastStateChange: elapsed,
    events: []
  }
}
