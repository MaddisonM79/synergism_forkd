// Shared tackBody fixture definitions, helpers, and N-tick driver.
//
// Used by both the parity harness (test/parity/tickHarness.parity.test.ts)
// and the perf suite (test/perf/tickHarness.{bench,budget.test}.ts) so
// that all three are exercising the exact same fixture shapes. Changing
// fixture E here updates parity, benches, and budget assertions
// simultaneously — no drift between test tiers.
//
// Three fixtures:
//   D — quiet early game.        Pre-singularity, minimal event volume.
//   E — active mid-game.         All bundles emitting events each tick.
//   F — timeWarp on.             Head + middle skipped; tail-only path.

import { Decimal } from '../../src/math/bignum'
import { type SweepStates } from '../../src/tick/challengeSweep'
import {
  tackBody,
  type TackBodyInput,
  type TackBodyResult
} from '../../src/tick/tack'
import {
  type AdvanceAllTimersInput
} from '../../src/tick/timersBundle'
import { type TackMiddleInput } from '../../src/tick/tackMiddle'
import { type TackTailInput } from '../../src/tick/tackTail'

// Union of every field the three bundles mutate. Threaded across ticks.
export interface BodyFixtureState {
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

// Per-fixture defaults for the input fields the bundles read but never
// write (gates, lookups, pre-evaluated effect values). Spread into each
// tick's bundle inputs alongside the threaded mutable state.
export type BodyStatics = {
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

export const buildHeadInput = (
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

export const buildMiddleInput = (
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

export const buildTailInput = (
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

export const buildBodyInput = (
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

// Pull all bundle writebacks back into the threaded state. When head /
// middle are skipped (timeWarp), those fields carry over unchanged.
export const bodyStateFromResult = (
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

// Snapshot the threaded state into a plain object for deep-equal
// assertion. Decimal fields stringify; SweepStates exposes a
// discriminated kind/index.
export const bodySnapshot = (s: BodyFixtureState) => ({
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

export interface BodyRunResult {
  finalState: BodyFixtureState
  totalEventCount: number
}

// Drive an arbitrary tackBody implementation across N ticks of one
// fixture. Used both by the parity oracle comparison (compare two
// implementations) and by the perf/budget suites (time one
// implementation).
export const runBodyTicks = (
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

// ═════════════════════════════════════════════════════════════════════════
// Shared baseline statics
// ═════════════════════════════════════════════════════════════════════════

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

// ═════════════════════════════════════════════════════════════════════════
// Fixture D — quiet early game
// ═════════════════════════════════════════════════════════════════════════
// Pre-singularity, all auto-tools off, no ambrosia / octeract / potions.
// Only the prestige/transcend/reincarnation counters tick up. Every tick
// middle emits `obtainium-multiplier-recompute-requested` (the `else`
// branch of case 3 when research61 !== 1).

export const FIXTURE_D_STATICS: BodyStatics = {
  head: BASE_HEAD_STATICS,
  middle: BASE_MIDDLE_STATICS,
  tail: BASE_TAIL_STATICS
}

export const FIXTURE_D_INITIAL: BodyFixtureState = { ...BASE_INITIAL }

// ═════════════════════════════════════════════════════════════════════════
// Fixture E — active mid-game
// ═════════════════════════════════════════════════════════════════════════
// Singularity ≥ 6 (autoPotion gate); octeract unlocked; ambrosia
// unlocked. runeSacrifice firing periodically, addObtainium fed by
// research61 === 1, auto-research roomba enabled, auto-prestige time
// mode firing repeatedly, sweep cycling. Cross-bundle event traffic
// every tick.

export const FIXTURE_E_STATICS: BodyStatics = {
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

export const FIXTURE_E_INITIAL: BodyFixtureState = {
  ...BASE_INITIAL,
  autoPotionTimer: 100,
  autoPotionTimerObtainium: 100,
  ambrosiaSeed: 42,
  offerings: new Decimal(1e8),
  obtainium: new Decimal(1e6)
}

// ═════════════════════════════════════════════════════════════════════════
// Fixture F — timeWarp on (tail-only path)
// ═════════════════════════════════════════════════════════════════════════
// G.timeWarp === true skips the head + middle bundles entirely; only
// tail runs. Sweep is active and auto-prestige time mode fires
// repeatedly.

export const FIXTURE_F_STATICS: BodyStatics = {
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

export const FIXTURE_F_INITIAL: BodyFixtureState = {
  ...BASE_INITIAL,
  offerings: new Decimal(1e6)
}

// Canonical descriptor list — keep parity / bench / budget all in sync
// by iterating this rather than redeclaring per file.
export interface FixtureDescriptor {
  name: string
  statics: BodyStatics
  initial: BodyFixtureState
  dt: number
  ticks: number
  timeWarp: boolean
}

export const BODY_FIXTURES: FixtureDescriptor[] = [
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

// Convenience: drive the production tackBody (not an oracle) for one
// fixture. The perf suite uses this as its single hot-path call.
export const runFixtureWithTackBody = (f: FixtureDescriptor): BodyRunResult =>
  runBodyTicks(f.statics, f.initial, f.dt, f.ticks, f.timeWarp, tackBody)
