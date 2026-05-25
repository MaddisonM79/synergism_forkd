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
  type AdvanceAllTimersInput,
  type AdvanceAllTimersResult
} from '../../src/tick/timersBundle'
import {
  tackMiddle,
  type TackMiddleInput,
  type TackMiddleResult
} from '../../src/tick/tackMiddle'
import { tackTail, type TackTailInput, type TackTailResult } from '../../src/tick/tackTail'

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
// State threading: every output field of the three bundles is threaded
// into the next tick's input. `offerings` is special — middle reads it
// (as a runeSacrifice gate), tail reads + writes it. We feed tail's
// post-tick offerings into both bundles' inputs on the next tick,
// matching how the web_ui adapter reads `player.offerings` once per
// tick to populate both `buildMiddleInput` and `buildTailInput`.
// ═════════════════════════════════════════════════════════════════════════

// Verbatim oracle — calls the three bundles inline in tackBody's exact
// order. Any divergence between this and `tackBody` is a composition bug.
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

// Union of every field the three bundles mutate. Threaded across ticks.
interface BodyFixtureState {
  // ─── head writebacks ─────────────────────────────────────────────────
  prestigecounter: number
  transcendcounter: number
  reincarnationcounter: number
  ascensionCounter: number
  ascensionCounterReal: number
  quarkstimer: number
  goldenQuarksTimer: number
  octeractTimer: number
  wowOcteracts: number
  totalWowOcteracts: number
  goldenQuarks: number
  quarksThisSingularity: number
  ascensionCounterRealReal: number
  singularityCounter: number
  singChallengeTimer: number
  autoPotionTimer: number
  autoPotionTimerObtainium: number
  ambrosiaTimerG: number
  blueberryTime: number
  ambrosia: number
  lifetimeAmbrosia: number
  ambrosiaSeed: number
  redAmbrosiaTimerG: number
  redAmbrosiaTime: number
  redAmbrosia: number
  lifetimeRedAmbrosia: number
  redAmbrosiaSeed: number

  // ─── middle writebacks ───────────────────────────────────────────────
  sacrificeTimer: number
  antSacrificeTimer: number
  antSacrificeTimerReal: number
  obtainium: Decimal

  // ─── tail writebacks (offerings is also read by middle each tick) ────
  offerings: Decimal
  autoOfferingCounter: number
  sweepState: SweepStates
  timeSinceLastStateChange: number
  autoResetTimerPrestige: number
  autoResetTimerTranscension: number
  autoResetTimerReincarnation: number
}

// Per-fixture defaults for the input fields the bundles read but never write
// (gates, lookups, pre-evaluated effect values). Spread into each tick's
// bundle inputs alongside the threaded mutable state.
type BodyStatics = {
  head: Omit<AdvanceAllTimersInput,
    | 'dt'
    | 'prestigecounter' | 'transcendcounter' | 'reincarnationcounter'
    | 'ascensionCounter' | 'ascensionCounterReal'
    | 'quarkstimer'
    | 'goldenQuarksTimer'
    | 'octeractTimer' | 'wowOcteracts' | 'totalWowOcteracts'
    | 'goldenQuarks' | 'quarksThisSingularity'
    | 'ascensionCounterRealReal' | 'singularityCounter' | 'singChallengeTimer'
    | 'autoPotionTimer' | 'autoPotionTimerObtainium'
    | 'ambrosiaTimerG' | 'blueberryTime' | 'ambrosia' | 'lifetimeAmbrosia' | 'ambrosiaSeed'
    | 'redAmbrosiaTimerG' | 'redAmbrosiaTime' | 'redAmbrosia' | 'lifetimeRedAmbrosia' | 'redAmbrosiaSeed'>
  middle: Omit<TackMiddleInput,
    | 'dt'
    | 'sacrificeTimer'
    | 'antSacrificeTimer' | 'antSacrificeTimerReal'
    | 'obtainium'
    | 'offerings'>
  tail: Omit<TackTailInput,
    | 'dt'
    | 'autoOfferingCounter' | 'offerings'
    | 'sweepState' | 'timeSinceLastStateChange'
    | 'autoResetTimerPrestige' | 'autoResetTimerTranscension' | 'autoResetTimerReincarnation'>
}

const buildHeadInput = (
  s: BodyStatics['head'],
  state: BodyFixtureState,
  dt: number
): AdvanceAllTimersInput => ({
  dt,
  ...s,
  prestigecounter: state.prestigecounter,
  transcendcounter: state.transcendcounter,
  reincarnationcounter: state.reincarnationcounter,
  ascensionCounter: state.ascensionCounter,
  ascensionCounterReal: state.ascensionCounterReal,
  quarkstimer: state.quarkstimer,
  goldenQuarksTimer: state.goldenQuarksTimer,
  octeractTimer: state.octeractTimer,
  wowOcteracts: state.wowOcteracts,
  totalWowOcteracts: state.totalWowOcteracts,
  goldenQuarks: state.goldenQuarks,
  quarksThisSingularity: state.quarksThisSingularity,
  ascensionCounterRealReal: state.ascensionCounterRealReal,
  singularityCounter: state.singularityCounter,
  singChallengeTimer: state.singChallengeTimer,
  autoPotionTimer: state.autoPotionTimer,
  autoPotionTimerObtainium: state.autoPotionTimerObtainium,
  ambrosiaTimerG: state.ambrosiaTimerG,
  blueberryTime: state.blueberryTime,
  ambrosia: state.ambrosia,
  lifetimeAmbrosia: state.lifetimeAmbrosia,
  ambrosiaSeed: state.ambrosiaSeed,
  redAmbrosiaTimerG: state.redAmbrosiaTimerG,
  redAmbrosiaTime: state.redAmbrosiaTime,
  redAmbrosia: state.redAmbrosia,
  lifetimeRedAmbrosia: state.lifetimeRedAmbrosia,
  redAmbrosiaSeed: state.redAmbrosiaSeed
})

const buildMiddleInput = (
  s: BodyStatics['middle'],
  state: BodyFixtureState,
  dt: number
): TackMiddleInput => ({
  dt,
  ...s,
  sacrificeTimer: state.sacrificeTimer,
  antSacrificeTimer: state.antSacrificeTimer,
  antSacrificeTimerReal: state.antSacrificeTimerReal,
  obtainium: state.obtainium,
  offerings: state.offerings
})

const buildTailInput = (
  s: BodyStatics['tail'],
  state: BodyFixtureState,
  dt: number
): TackTailInput => ({
  dt,
  ...s,
  autoOfferingCounter: state.autoOfferingCounter,
  offerings: state.offerings,
  sweepState: state.sweepState,
  timeSinceLastStateChange: state.timeSinceLastStateChange,
  autoResetTimerPrestige: state.autoResetTimerPrestige,
  autoResetTimerTranscension: state.autoResetTimerTranscension,
  autoResetTimerReincarnation: state.autoResetTimerReincarnation
})

const buildBodyInput = (
  s: BodyStatics,
  state: BodyFixtureState,
  dt: number,
  timeWarp: boolean
): TackBodyInput => ({
  timeWarp,
  head: timeWarp ? undefined : buildHeadInput(s.head, state, dt),
  middle: timeWarp ? undefined : buildMiddleInput(s.middle, state, dt),
  tail: buildTailInput(s.tail, state, dt)
})

// Pull all bundle writebacks back into the threaded state. When head/middle
// are skipped (timeWarp), those fields carry over unchanged from `prev`.
const bodyStateFromResult = (
  r: TackBodyResult,
  prev: BodyFixtureState
): BodyFixtureState => ({
  prestigecounter: r.head?.prestigecounter ?? prev.prestigecounter,
  transcendcounter: r.head?.transcendcounter ?? prev.transcendcounter,
  reincarnationcounter: r.head?.reincarnationcounter ?? prev.reincarnationcounter,
  ascensionCounter: r.head?.ascensionCounter ?? prev.ascensionCounter,
  ascensionCounterReal: r.head?.ascensionCounterReal ?? prev.ascensionCounterReal,
  quarkstimer: r.head?.quarkstimer ?? prev.quarkstimer,
  goldenQuarksTimer: r.head?.goldenQuarksTimer ?? prev.goldenQuarksTimer,
  octeractTimer: r.head?.octeractTimer ?? prev.octeractTimer,
  wowOcteracts: r.head?.wowOcteracts ?? prev.wowOcteracts,
  totalWowOcteracts: r.head?.totalWowOcteracts ?? prev.totalWowOcteracts,
  goldenQuarks: r.head?.goldenQuarks ?? prev.goldenQuarks,
  quarksThisSingularity: r.head?.quarksThisSingularity ?? prev.quarksThisSingularity,
  ascensionCounterRealReal: r.head?.ascensionCounterRealReal ?? prev.ascensionCounterRealReal,
  singularityCounter: r.head?.singularityCounter ?? prev.singularityCounter,
  singChallengeTimer: r.head?.singChallengeTimer ?? prev.singChallengeTimer,
  autoPotionTimer: r.head?.autoPotionTimer ?? prev.autoPotionTimer,
  autoPotionTimerObtainium: r.head?.autoPotionTimerObtainium ?? prev.autoPotionTimerObtainium,
  ambrosiaTimerG: r.head?.ambrosiaTimerG ?? prev.ambrosiaTimerG,
  blueberryTime: r.head?.blueberryTime ?? prev.blueberryTime,
  ambrosia: r.head?.ambrosia ?? prev.ambrosia,
  lifetimeAmbrosia: r.head?.lifetimeAmbrosia ?? prev.lifetimeAmbrosia,
  ambrosiaSeed: r.head?.ambrosiaSeed ?? prev.ambrosiaSeed,
  redAmbrosiaTimerG: r.head?.redAmbrosiaTimerG ?? prev.redAmbrosiaTimerG,
  redAmbrosiaTime: r.head?.redAmbrosiaTime ?? prev.redAmbrosiaTime,
  redAmbrosia: r.head?.redAmbrosia ?? prev.redAmbrosia,
  lifetimeRedAmbrosia: r.head?.lifetimeRedAmbrosia ?? prev.lifetimeRedAmbrosia,
  redAmbrosiaSeed: r.head?.redAmbrosiaSeed ?? prev.redAmbrosiaSeed,
  sacrificeTimer: r.middle?.sacrificeTimer ?? prev.sacrificeTimer,
  antSacrificeTimer: r.middle?.antSacrificeTimer ?? prev.antSacrificeTimer,
  antSacrificeTimerReal: r.middle?.antSacrificeTimerReal ?? prev.antSacrificeTimerReal,
  obtainium: r.middle?.obtainium ?? prev.obtainium,
  offerings: r.tail.offerings,
  autoOfferingCounter: r.tail.autoOfferingCounter,
  sweepState: r.tail.sweepState,
  timeSinceLastStateChange: r.tail.timeSinceLastStateChange,
  autoResetTimerPrestige: r.tail.autoResetTimerPrestige,
  autoResetTimerTranscension: r.tail.autoResetTimerTranscension,
  autoResetTimerReincarnation: r.tail.autoResetTimerReincarnation
})

// Snapshot the threaded state into a plain object for deep-equal assertion.
// Decimal fields stringify; SweepStates exposes a discriminated kind/index.
const bodySnapshot = (s: BodyFixtureState) => ({
  prestigecounter: s.prestigecounter,
  transcendcounter: s.transcendcounter,
  reincarnationcounter: s.reincarnationcounter,
  ascensionCounter: s.ascensionCounter,
  ascensionCounterReal: s.ascensionCounterReal,
  quarkstimer: s.quarkstimer,
  goldenQuarksTimer: s.goldenQuarksTimer,
  octeractTimer: s.octeractTimer,
  wowOcteracts: s.wowOcteracts,
  totalWowOcteracts: s.totalWowOcteracts,
  goldenQuarks: s.goldenQuarks,
  quarksThisSingularity: s.quarksThisSingularity,
  ascensionCounterRealReal: s.ascensionCounterRealReal,
  singularityCounter: s.singularityCounter,
  singChallengeTimer: s.singChallengeTimer,
  autoPotionTimer: s.autoPotionTimer,
  autoPotionTimerObtainium: s.autoPotionTimerObtainium,
  ambrosiaTimerG: s.ambrosiaTimerG,
  blueberryTime: s.blueberryTime,
  ambrosia: s.ambrosia,
  lifetimeAmbrosia: s.lifetimeAmbrosia,
  ambrosiaSeed: s.ambrosiaSeed,
  redAmbrosiaTimerG: s.redAmbrosiaTimerG,
  redAmbrosiaTime: s.redAmbrosiaTime,
  redAmbrosia: s.redAmbrosia,
  lifetimeRedAmbrosia: s.lifetimeRedAmbrosia,
  redAmbrosiaSeed: s.redAmbrosiaSeed,
  sacrificeTimer: s.sacrificeTimer,
  antSacrificeTimer: s.antSacrificeTimer,
  antSacrificeTimerReal: s.antSacrificeTimerReal,
  obtainium: s.obtainium.toString(),
  offerings: s.offerings.toString(),
  autoOfferingCounter: s.autoOfferingCounter,
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

interface BodyRunResult {
  finalState: BodyFixtureState
  totalEventCount: number
}

const runBodyTicks = (
  statics: BodyStatics,
  initial: BodyFixtureState,
  dt: number,
  ticks: number,
  timeWarp: boolean,
  fn: (input: TackBodyInput) => TackBodyResult
): BodyRunResult => {
  let state = initial
  let totalEventCount = 0
  for (let i = 0; i < ticks; i++) {
    const input = buildBodyInput(statics, state, dt, timeWarp)
    const result = fn(input)
    state = bodyStateFromResult(result, state)
    totalEventCount += result.events.length
  }
  return { finalState: state, totalEventCount }
}

// ─── Shared baseline statics ───────────────────────────────────────────
// Locked-down singleton: no octeract / ambrosia / red-ambrosia, no
// potions, no sacrifices, no auto-research. Used as the base for the
// quiet fixture and extended for the active / late fixtures.

const BASE_HEAD_STATICS: BodyStatics['head'] = {
  globalTimeMultiplier: 1,
  ascensionSpeedMulti: 1,
  maxQuarkTimer: 90000,
  exportGQPerHour: 0,
  octeractUnlocked: false,
  octeractPerSecond: 0,
  highestSingularityCount: 0,
  singularityCount: 0,
  goldenQuarksMultiplierExcludingBase: 1,
  insideSingularityChallenge: false,
  singularitySpeedMulti: 1,
  autoPotionToggleOffering: false,
  autoPotionToggleObtainium: false,
  offeringPotionCount: 0,
  obtainiumPotionCount: 0,
  autoPotionSpeedMult: 1,
  noSingularityUpgradesCompletions: 0,
  ambrosiaGenerationSpeed: 0,
  ambrosiaLuck: 0,
  bonusAmbrosia: 0,
  timePerAmbrosia: 600,
  ambrosiaAcceleratorMult: 1,
  ambrosiaBrickOfLeadMult: 1,
  noAmbrosiaUpgradesCompletions: 0,
  redAmbrosiaGenerationSpeed: 0,
  redAmbrosiaLuck: 0,
  ambrosiaTimePerRedAmbrosia: 0,
  timePerRedAmbrosia: 100000,
  redAmbrosiaBarRequirementMultiplier: 1
}

const BASE_MIDDLE_STATICS: BodyStatics['middle'] = {
  runeSacrificeEnabled: false,
  autoSacrificeInterval: 1,
  antSacrificeUnlocked: false,
  globalDelta: 1,
  autoSacrificeMode: 'InGameTime',
  crumbsThisSacrifice: new Decimal(0),
  autoSacrificeEnabled: false,
  availableRebornELO: 0,
  onlySacrificeMaxRebornELO: false,
  alwaysSacrificeMaxRebornELO: false,
  autoSacrificeThreshold: 60,
  immortalELOGain: 0,
  immortalELO: 0,
  rebornELO: 0,
  research61: 0,
  obtainiumGain: new Decimal(0),
  ascensionChallenge: 0,
  taxmanLastStandEnabled: false,
  taxmanLastStandCompletions: 0,
  autoResearchToggle: false,
  autoResearch: 0,
  autoResearchMode: 'manual',
  roombaUnlocked: false,
  challengecompletions14: 0
}

const BASE_TAIL_STATICS: BodyStatics['tail'] = {
  highestchallengecompletions3: 0,
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

const BASE_INITIAL: BodyFixtureState = {
  prestigecounter: 0,
  transcendcounter: 0,
  reincarnationcounter: 0,
  ascensionCounter: 0,
  ascensionCounterReal: 0,
  quarkstimer: 0,
  goldenQuarksTimer: 0,
  octeractTimer: 0,
  wowOcteracts: 0,
  totalWowOcteracts: 0,
  goldenQuarks: 0,
  quarksThisSingularity: 0,
  ascensionCounterRealReal: 0,
  singularityCounter: 0,
  singChallengeTimer: 0,
  autoPotionTimer: 0,
  autoPotionTimerObtainium: 0,
  ambrosiaTimerG: 0,
  blueberryTime: 0,
  ambrosia: 0,
  lifetimeAmbrosia: 0,
  ambrosiaSeed: 1,
  redAmbrosiaTimerG: 0,
  redAmbrosiaTime: 0,
  redAmbrosia: 0,
  lifetimeRedAmbrosia: 0,
  redAmbrosiaSeed: 1,
  sacrificeTimer: 0,
  antSacrificeTimer: 0,
  antSacrificeTimerReal: 0,
  obtainium: new Decimal(0),
  offerings: new Decimal(0),
  autoOfferingCounter: 0,
  sweepState: { kind: 'idle' },
  timeSinceLastStateChange: 0,
  autoResetTimerPrestige: 0,
  autoResetTimerTranscension: 0,
  autoResetTimerReincarnation: 0
}

// ─── Fixture D — quiet early game ──────────────────────────────────────
// Pre-singularity, all auto-tools off, no ambrosia / octeract / potions.
// Only the prestige/transcend/reincarnation counters tick up. Every tick
// middle emits `obtainium-multiplier-recompute-requested` (the `else`
// branch of case 3 when research61 !== 1), so events accumulate linearly.

const FIXTURE_D_STATICS: BodyStatics = {
  head: BASE_HEAD_STATICS,
  middle: BASE_MIDDLE_STATICS,
  tail: BASE_TAIL_STATICS
}

const FIXTURE_D_INITIAL: BodyFixtureState = { ...BASE_INITIAL }

// ─── Fixture E — active mid-game ───────────────────────────────────────
// Singularity ≥ 6 (autoPotion gate); octeract unlocked; ambrosia
// unlocked. runeSacrifice firing periodically, addObtainium fed by
// research61 === 1, auto-research roomba enabled, auto-prestige time
// mode firing repeatedly, sweep cycling. Cross-bundle event traffic
// every tick.

const FIXTURE_E_STATICS: BodyStatics = {
  head: {
    ...BASE_HEAD_STATICS,
    highestSingularityCount: 50,
    singularityCount: 50,
    octeractUnlocked: true,
    octeractPerSecond: 1e6,
    autoPotionToggleOffering: true,
    autoPotionToggleObtainium: true,
    offeringPotionCount: 10,
    obtainiumPotionCount: 10,
    autoPotionSpeedMult: 2,
    noSingularityUpgradesCompletions: 5,
    ambrosiaGenerationSpeed: 100,
    ambrosiaLuck: 100,
    bonusAmbrosia: 1
  },
  middle: {
    ...BASE_MIDDLE_STATICS,
    runeSacrificeEnabled: true,
    autoSacrificeInterval: 1,
    antSacrificeUnlocked: true,
    autoSacrificeEnabled: true,
    autoSacrificeMode: 'InGameTime',
    autoSacrificeThreshold: 1,
    crumbsThisSacrifice: new Decimal(1e6),
    availableRebornELO: 100,
    research61: 1,
    obtainiumGain: new Decimal(1e3),
    autoResearchToggle: true,
    autoResearch: 5,
    autoResearchMode: 'cheapest',
    roombaUnlocked: true,
    challengecompletions14: 1
  },
  tail: {
    ...BASE_TAIL_STATICS,
    highestchallengecompletions3: 5,
    shouldRunSweep: true,
    timerStart: 0.5,
    timerExit: 0.5,
    timerEnter: 0.2,
    nextRegularChallengeFromInitial: 1,
    nextRegularChallengeFromActive: 2,
    prestigeMode: 'time',
    toggle15: true,
    autoPrestigeMilestone: 1,
    prestigeamount: 1.5,
    prestigePointGain: new Decimal(1e6),
    coinsThisPrestige: new Decimal(1e17)
  }
}

const FIXTURE_E_INITIAL: BodyFixtureState = {
  ...BASE_INITIAL,
  autoPotionTimer: 100,
  autoPotionTimerObtainium: 100,
  ambrosiaSeed: 42,
  offerings: new Decimal(1e8),
  obtainium: new Decimal(1e6)
}

// ─── Fixture F — timeWarp on (tail-only path) ──────────────────────────
// G.timeWarp === true skips the head + middle bundles entirely; only
// tail runs. Verifies the gating logic in tackBody and that tail-only
// state threading stays consistent over N=1000 ticks. Sweep is active
// and auto-prestige time mode fires repeatedly.

const FIXTURE_F_STATICS: BodyStatics = {
  // head + middle still required at the type level even though timeWarp
  // skips them; buildBodyInput passes undefined so the actual values
  // never reach a bundle call.
  head: BASE_HEAD_STATICS,
  middle: BASE_MIDDLE_STATICS,
  tail: {
    ...BASE_TAIL_STATICS,
    highestchallengecompletions3: 5,
    shouldRunSweep: true,
    timerStart: 0.5,
    timerExit: 0.5,
    timerEnter: 0.2,
    nextRegularChallengeFromInitial: 1,
    nextRegularChallengeFromActive: 2,
    prestigeMode: 'time',
    toggle15: true,
    autoPrestigeMilestone: 1,
    prestigeamount: 1.5,
    prestigePointGain: new Decimal(1e6),
    coinsThisPrestige: new Decimal(1e17)
  }
}

const FIXTURE_F_INITIAL: BodyFixtureState = {
  ...BASE_INITIAL,
  offerings: new Decimal(1e6)
}

describe('parity harness: tackBody over N=1000 ticks', () => {
  const fixtures: Array<{
    name: string
    statics: BodyStatics
    initial: BodyFixtureState
    dt: number
    ticks: number
    timeWarp: boolean
  }> = [
    {
      name: 'D — quiet early game (counters tick, recompute every tick)',
      statics: FIXTURE_D_STATICS,
      initial: FIXTURE_D_INITIAL,
      dt: 0.025,
      ticks: 1000,
      timeWarp: false
    },
    {
      name: 'E — active mid-game (all bundles emitting cross-tick events)',
      statics: FIXTURE_E_STATICS,
      initial: FIXTURE_E_INITIAL,
      dt: 0.025,
      ticks: 1000,
      timeWarp: false
    },
    {
      name: 'F — timeWarp on (tail-only, head+middle skipped)',
      statics: FIXTURE_F_STATICS,
      initial: FIXTURE_F_INITIAL,
      dt: 0.025,
      ticks: 1000,
      timeWarp: true
    }
  ]

  for (const f of fixtures) {
    it(`${f.name} — migrated matches oracle`, () => {
      const migrated = runBodyTicks(f.statics, f.initial, f.dt, f.ticks, f.timeWarp, tackBody)
      const oracle = runBodyTicks(f.statics, f.initial, f.dt, f.ticks, f.timeWarp, oracleTackBody)
      expect(bodySnapshot(migrated.finalState)).toEqual(bodySnapshot(oracle.finalState))
      expect(migrated.totalEventCount).toBe(oracle.totalEventCount)
    })
  }
})
