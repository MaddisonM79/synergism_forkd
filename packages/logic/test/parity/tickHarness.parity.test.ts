// Multi-tick parity harnesses for the migrated tick bundles.
//
// Per-leaf and per-case parity tests cover individual function correctness;
// these harnesses exercise the bundles across N=1000 ticks against a small
// set of fixtures, catching composition-level drift the per-case tests
// can't:
//   - Sweep state machine cycling correctly over many ticks.
//   - Timer accumulators (autoResetTimers, sweep elapsed, octeract /
//     ambrosia / red-ambrosia fractional buckets) staying in sync.
//   - Event counts and ordering invariants across long runs.
//   - Floating-point drift between the migrated bundles and a verbatim
//     inline composition oracle.
//
// Two harnesses live here:
//   1. `tackTail` (Phase 5 partial) — tail-only fixtures.
//   2. `tackBody` (Phase 5) — full head + middle + tail composition,
//      including a `timeWarp=true` fixture exercising the tail-only path.
//
// Scope caveat: the harnesses compare migrated bundles to a verbatim
// composition oracle, not to the pre-Phase-4 legacy in-place tack body
// (which dispatched events between bundles). That dispatch-ordering
// difference is documented in packages/logic/src/tick/tack.ts and is
// self-correcting within a tick. resourceGain + generateAntsAndCrumbs
// run as pre-tick web_ui calls and are out of scope here.

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import { addOfferings } from '../../src/tick/automaticTools'
import { applyAutoResets } from '../../src/tick/autoReset'
import { type SweepStates, tickChallengeSweep } from '../../src/tick/challengeSweep'
import {
  tackBody,
  type TackBodyInput,
  type TackBodyResult
} from '../../src/tick/tack'
import {
  advanceAllTimers,
  type AdvanceAllTimersResult
} from '../../src/tick/timersBundle'
import {
  tackMiddle,
  type TackMiddleResult
} from '../../src/tick/tackMiddle'
import { tackTail, type TackTailInput, type TackTailResult } from '../../src/tick/tackTail'
import {
  BODY_FIXTURES,
  bodySnapshot,
  runBodyTicks
} from '../fixtures/tackBodyFixtures'

// Verbatim three-subsystem oracle — same body as the tackTail
// implementation, used as the comparison target.
const oracleTackTail = (input: TackTailInput): TackTailResult => {
  const events: TackTailResult['events'] = []

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

// ─── Fixture state — only the fields tackTail reads/writes ─────────────
//
// `input.*` fields that are stable across the tick (gates, lookups, timers)
// are reset each iteration; the mutable accumulators (autoOfferingCounter,
// offerings, sweepState, timeSinceLastStateChange, autoResetTimer*) thread
// through tick-to-tick.

interface FixtureState {
  autoOfferingCounter: number
  offerings: Decimal
  sweepState: SweepStates
  timeSinceLastStateChange: number
  autoResetTimerPrestige: number
  autoResetTimerTranscension: number
  autoResetTimerReincarnation: number
}

// Snapshot two states for equality assertion at end-of-tick.
const stateSnapshot = (s: FixtureState) => ({
  autoOfferingCounter: s.autoOfferingCounter,
  offerings: s.offerings.toString(),
  sweepKind: s.sweepState.kind,
  sweepIndex: s.sweepState.kind === 'active' ? s.sweepState.index : undefined,
  sweepToIndex: s.sweepState.kind === 'enter_wait' ? s.sweepState.toIndex : undefined,
  sweepExplored: ('explored' in s.sweepState)
    ? [...s.sweepState.explored].sort((a, b) => a - b)
    : undefined,
  timeSinceLastStateChange: s.timeSinceLastStateChange,
  autoResetTimerPrestige: s.autoResetTimerPrestige,
  autoResetTimerTranscension: s.autoResetTimerTranscension,
  autoResetTimerReincarnation: s.autoResetTimerReincarnation
})

// Build a TackTailInput from the per-fixture constants + the threaded state.
type StaticInputs = Omit<TackTailInput,
  | 'dt'
  | 'autoOfferingCounter'
  | 'offerings'
  | 'sweepState'
  | 'timeSinceLastStateChange'
  | 'autoResetTimerPrestige'
  | 'autoResetTimerTranscension'
  | 'autoResetTimerReincarnation'
>

const buildInput = (statics: StaticInputs, state: FixtureState, dt: number): TackTailInput => ({
  dt,
  ...statics,
  autoOfferingCounter: state.autoOfferingCounter,
  offerings: state.offerings,
  sweepState: state.sweepState,
  timeSinceLastStateChange: state.timeSinceLastStateChange,
  autoResetTimerPrestige: state.autoResetTimerPrestige,
  autoResetTimerTranscension: state.autoResetTimerTranscension,
  autoResetTimerReincarnation: state.autoResetTimerReincarnation
})

// Pull the mutable state out of a result back into a FixtureState.
const stateFromResult = (r: TackTailResult): FixtureState => ({
  autoOfferingCounter: r.autoOfferingCounter,
  offerings: r.offerings,
  sweepState: r.sweepState,
  timeSinceLastStateChange: r.timeSinceLastStateChange,
  autoResetTimerPrestige: r.autoResetTimerPrestige,
  autoResetTimerTranscension: r.autoResetTimerTranscension,
  autoResetTimerReincarnation: r.autoResetTimerReincarnation
})

// ─── Fixtures ──────────────────────────────────────────────────────────

// Fixture A — quiet idle tick. Sweep off, all auto-reset gates blocking.
// addOfferings active (c3 ≥ 1), so the only state change per tick is the
// fractional offering counter accumulating ~0.0125 per 25ms tick.
const FIXTURE_A_STATICS: StaticInputs = {
  highestchallengecompletions3: 1,
  shouldRunSweep: false,
  timerStart: 5,
  timerExit: 30,
  timerEnter: 2,
  initialIndex: 1,
  nextRegularChallengeFromInitial: -1,
  nextRegularChallengeFromActive: -1,
  challenge15AutoExponentCheck: false,
  isFinishedStillValid: false,
  prestigeMode: 'amount',
  toggle15: false,
  autoPrestigeMilestone: 0,
  prestigePoints: new Decimal(0),
  prestigePointGain: new Decimal(0),
  prestigeamount: 1,
  coinsThisPrestige: new Decimal(0),
  transcendMode: 'amount',
  toggle21: false,
  upgrade89: 0,
  transcendPoints: new Decimal(0),
  transcendPointGain: new Decimal(0),
  transcendamount: 1,
  coinsThisTranscension: new Decimal(0),
  reincarnationMode: 'amount',
  toggle27: false,
  research46: 0,
  reincarnationPoints: new Decimal(0),
  reincarnationPointGain: new Decimal(0),
  reincarnationamount: 1,
  transcendShards: new Decimal(0),
  ascensionChallenge: 0,
  transcensionChallenge: 0,
  reincarnationChallenge: 0
}

const FIXTURE_A_INITIAL: FixtureState = {
  autoOfferingCounter: 0,
  offerings: new Decimal(100),
  sweepState: { kind: 'idle' },
  timeSinceLastStateChange: 0,
  autoResetTimerPrestige: 0,
  autoResetTimerTranscension: 0,
  autoResetTimerReincarnation: 0
}

// Fixture B — sweep running through challenges 1-5, no resets firing.
// Loops the active → enter_wait → active cycle. nextRegularChallengeFromActive
// is hardcoded so all six challenges feel "next"; the explored set guards
// against repeats, so the cycle terminates naturally.
const FIXTURE_B_STATICS: StaticInputs = {
  ...FIXTURE_A_STATICS,
  highestchallengecompletions3: 0,
  shouldRunSweep: true,
  timerStart: 0.5, // fast cycles for the harness
  timerExit: 0.5,
  timerEnter: 0.2,
  initialIndex: 1,
  // Return 2,3,4,5 in turn — driven by `explored` set tracking inside
  // the state machine. We pin it to 2 here; the real game's lookup
  // changes per state, but for the harness this exercises one
  // active→enter_wait→active hop repeatedly.
  nextRegularChallengeFromInitial: 1,
  nextRegularChallengeFromActive: 2,
  challenge15AutoExponentCheck: false,
  isFinishedStillValid: false
}

const FIXTURE_B_INITIAL: FixtureState = {
  autoOfferingCounter: 0,
  offerings: new Decimal(1000),
  sweepState: { kind: 'idle' },
  timeSinceLastStateChange: 0,
  autoResetTimerPrestige: 0,
  autoResetTimerTranscension: 0,
  autoResetTimerReincarnation: 0
}

// Fixture C — auto-prestige time mode actively firing.
// Each ~1.5s the timer crosses prestigeamount=1.5 threshold and an
// auto-reset-triggered event fires. (The harness doesn't actually
// reset state since reset() is web_ui-only; the test verifies the
// event/timer pattern stays in sync across the two implementations.)
const FIXTURE_C_STATICS: StaticInputs = {
  ...FIXTURE_A_STATICS,
  highestchallengecompletions3: 1,
  prestigeMode: 'time',
  toggle15: true,
  autoPrestigeMilestone: 1,
  prestigeamount: 1.5,
  coinsThisPrestige: new Decimal(1e17)
}

const FIXTURE_C_INITIAL: FixtureState = {
  autoOfferingCounter: 0,
  offerings: new Decimal(500),
  sweepState: { kind: 'idle' },
  timeSinceLastStateChange: 0,
  autoResetTimerPrestige: 0,
  autoResetTimerTranscension: 0,
  autoResetTimerReincarnation: 0
}

// ─── Harness driver ────────────────────────────────────────────────────

interface RunResult {
  finalState: FixtureState
  totalEventCount: number
}

const runTicks = (
  statics: StaticInputs,
  initial: FixtureState,
  dt: number,
  ticks: number,
  fn: (input: TackTailInput) => TackTailResult
): RunResult => {
  let state = initial
  let totalEventCount = 0
  for (let i = 0; i < ticks; i++) {
    const input = buildInput(statics, state, dt)
    const result = fn(input)
    state = stateFromResult(result)
    totalEventCount += result.events.length
  }
  return { finalState: state, totalEventCount }
}

describe('parity harness: tackTail over N=1000 ticks', () => {
  const fixtures: Array<{
    name: string
    statics: StaticInputs
    initial: FixtureState
    dt: number
    ticks: number
  }> = [
    {
      name: 'A — idle, only addOfferings active',
      statics: FIXTURE_A_STATICS,
      initial: FIXTURE_A_INITIAL,
      dt: 0.025,
      ticks: 1000
    },
    {
      name: 'B — sweep cycling through active/enter_wait',
      statics: FIXTURE_B_STATICS,
      initial: FIXTURE_B_INITIAL,
      dt: 0.025,
      ticks: 1000
    },
    {
      name: 'C — auto-prestige time mode firing repeatedly',
      statics: FIXTURE_C_STATICS,
      initial: FIXTURE_C_INITIAL,
      dt: 0.025,
      ticks: 1000
    }
  ]

  for (const f of fixtures) {
    it(`${f.name} — migrated matches oracle`, () => {
      const migrated = runTicks(f.statics, f.initial, f.dt, f.ticks, tackTail)
      const oracle = runTicks(f.statics, f.initial, f.dt, f.ticks, oracleTackTail)
      expect(stateSnapshot(migrated.finalState)).toEqual(stateSnapshot(oracle.finalState))
      expect(migrated.totalEventCount).toBe(oracle.totalEventCount)
    })
  }
})

// ═════════════════════════════════════════════════════════════════════════
// Phase 5 — full tackBody harness (head + middle + tail composition).
//
// Drives `tackBody` across N=1000 ticks per fixture and compares state +
// event counts against an inline-composed oracle. The oracle calls the
// same three bundle functions in the same order tackBody does, so any
// drift signals a refactor regression in the orchestrator's composition
// (not in any individual bundle — those are covered by their own parity
// tests).
//
// Fixture definitions + the runBodyTicks driver live in
// test/fixtures/tackBodyFixtures.ts so the parity, bench, and budget
// suites all exercise the exact same shapes.
// ═════════════════════════════════════════════════════════════════════════

const oracleTackBody = (input: TackBodyInput): TackBodyResult => {
  const events: TackBodyResult['events'] = []
  let head: AdvanceAllTimersResult | undefined
  let middle: TackMiddleResult | undefined
  if (!input.timeWarp) {
    if (input.head === undefined || input.middle === undefined) {
      throw new Error('oracle: head and middle required when timeWarp === false')
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

describe('parity harness: tackBody over N=1000 ticks', () => {
  for (const f of BODY_FIXTURES) {
    it(`${f.name} — migrated matches oracle`, () => {
      const migrated = runBodyTicks(f.statics, f.initial, f.dt, f.ticks, f.timeWarp, tackBody)
      const oracle = runBodyTicks(f.statics, f.initial, f.dt, f.ticks, f.timeWarp, oracleTackBody)
      expect(bodySnapshot(migrated.finalState)).toEqual(bodySnapshot(oracle.finalState))
      expect(migrated.totalEventCount).toBe(oracle.totalEventCount)
    })
  }
})
