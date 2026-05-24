// Multi-tick parity harness for the advanceAllTimers composition.
//
// Per-case parity tests (timers.parity, autoPotion.parity,
// ambrosiaTimers.parity, octeractTimer.parity) cover individual
// function correctness; this harness exercises advanceAllTimers across
// N=1000 ticks against a small set of fixtures, catching
// composition-level drift the per-case tests can't:
//   - Counter timers staying in sync across long runs.
//   - Octeract timer + GQ-giveaway interaction over many seconds.
//   - autoPotion threshold crossings emitting paired events.
//   - Ambrosia / red-ambrosia chunked timers + RNG advancement.
//   - The redAmbrosia → ambrosia bonus-time feedback loop firing
//     correctly when bonusAmbrosiaTime > 0.
//   - Floating-point drift between the migrated `advanceAllTimers`
//     and the verbatim eleven-subsystem oracle.
//
// All eleven cases run in legacy relative order: prestige,
// transcension, reincarnation, ascension, quarks, goldenQuarks,
// octeracts, singularity, autoPotion, ambrosia, redAmbrosia (with
// bonus-time feedback).

import { describe, expect, it } from 'vitest'
import type { CoreEvent } from '../../src/events/types'
import {
  advanceAllTimers,
  type AdvanceAllTimersInput,
  type AdvanceAllTimersResult
} from '../../src/tick/timersBundle'
import {
  advanceAmbrosiaTimer,
  advanceAscensionTimer,
  advanceAutoPotionTimer,
  advanceGoldenQuarksTimer,
  advanceOcteractTimer,
  advanceQuarksTimer,
  advanceRedAmbrosiaTimer,
  advanceResetCounter,
  advanceSingularityTimer
} from '../../src/tick/timers'

// Verbatim ten-subsystem oracle — same body as the advanceAllTimers
// implementation, used as the comparison target.
const oracleAdvanceAllTimers = (input: AdvanceAllTimersInput): AdvanceAllTimersResult => {
  const events: CoreEvent[] = []

  const prestigecounter = advanceResetCounter(input.prestigecounter, input.dt, input.globalTimeMultiplier)
  const transcendcounter = advanceResetCounter(input.transcendcounter, input.dt, input.globalTimeMultiplier)
  const reincarnationcounter = advanceResetCounter(input.reincarnationcounter, input.dt, input.globalTimeMultiplier)

  const ascR = advanceAscensionTimer({
    time: input.dt,
    ascensionCounter: input.ascensionCounter,
    ascensionCounterReal: input.ascensionCounterReal,
    ascensionSpeedMulti: input.ascensionSpeedMulti
  })

  const quarkstimer = advanceQuarksTimer({
    time: input.dt,
    quarkstimer: input.quarkstimer,
    maxQuarkTimer: input.maxQuarkTimer
  })

  const goldenQuarksTimer = advanceGoldenQuarksTimer({
    time: input.dt,
    goldenQuarksTimer: input.goldenQuarksTimer,
    exportGQPerHour: input.exportGQPerHour
  })

  const octR = advanceOcteractTimer({
    time: input.dt,
    timeMultiplier: 1,
    octeractUnlocked: input.octeractUnlocked,
    octeractTimer: input.octeractTimer,
    wowOcteracts: input.wowOcteracts,
    totalWowOcteracts: input.totalWowOcteracts,
    goldenQuarks: input.goldenQuarks,
    quarksThisSingularity: input.quarksThisSingularity,
    perSecond: input.octeractPerSecond,
    highestSingularityCount: input.highestSingularityCount,
    singularityCount: input.singularityCount,
    goldenQuarksMultiplierExcludingBase: input.goldenQuarksMultiplierExcludingBase
  })
  for (const e of octR.events) events.push(e)

  const singR = advanceSingularityTimer({
    time: input.dt,
    ascensionCounterRealReal: input.ascensionCounterRealReal,
    singularityCounter: input.singularityCounter,
    singChallengeTimer: input.singChallengeTimer,
    insideSingularityChallenge: input.insideSingularityChallenge,
    singularitySpeedMulti: input.singularitySpeedMulti
  })

  const autoPotionR = advanceAutoPotionTimer({
    time: input.dt,
    timeMultiplier: 1,
    highestSingularityCount: input.highestSingularityCount,
    autoPotionTimer: input.autoPotionTimer,
    autoPotionTimerObtainium: input.autoPotionTimerObtainium,
    toggleOffering: input.autoPotionToggleOffering,
    toggleObtainium: input.autoPotionToggleObtainium,
    offeringPotionCount: input.offeringPotionCount,
    obtainiumPotionCount: input.obtainiumPotionCount,
    autoPotionSpeedMult: input.autoPotionSpeedMult
  })
  for (const e of autoPotionR.events) events.push(e)

  const ambR = advanceAmbrosiaTimer({
    time: input.dt,
    timeMultiplier: 1,
    noSingularityUpgradesCompletions: input.noSingularityUpgradesCompletions,
    ambrosiaGenerationSpeed: input.ambrosiaGenerationSpeed,
    ambrosiaTimerG: input.ambrosiaTimerG,
    blueberryTime: input.blueberryTime,
    ambrosia: input.ambrosia,
    lifetimeAmbrosia: input.lifetimeAmbrosia,
    seed: input.ambrosiaSeed,
    ambrosiaLuck: input.ambrosiaLuck,
    bonusAmbrosia: input.bonusAmbrosia,
    timePerAmbrosia: input.timePerAmbrosia,
    acceleratorMult: input.ambrosiaAcceleratorMult,
    brickOfLeadMult: input.ambrosiaBrickOfLeadMult
  })
  for (const e of ambR.events) events.push(e)

  const redR = advanceRedAmbrosiaTimer({
    time: input.dt,
    timeMultiplier: 1,
    noAmbrosiaUpgradesCompletions: input.noAmbrosiaUpgradesCompletions,
    redAmbrosiaGenerationSpeed: input.redAmbrosiaGenerationSpeed,
    redAmbrosiaTimerG: input.redAmbrosiaTimerG,
    redAmbrosiaTime: input.redAmbrosiaTime,
    redAmbrosia: input.redAmbrosia,
    lifetimeRedAmbrosia: input.lifetimeRedAmbrosia,
    seed: input.redAmbrosiaSeed,
    redAmbrosiaLuck: input.redAmbrosiaLuck,
    ambrosiaTimePerRedAmbrosia: input.ambrosiaTimePerRedAmbrosia,
    timePerRedAmbrosia: input.timePerRedAmbrosia,
    barRequirementMultiplier: input.redAmbrosiaBarRequirementMultiplier
  })
  for (const e of redR.events) events.push(e)

  let ambrosiaTimerG = ambR.ambrosiaTimerG
  let blueberryTime = ambR.blueberryTime
  let ambrosia = ambR.ambrosia
  let lifetimeAmbrosia = ambR.lifetimeAmbrosia
  let ambrosiaSeed = ambR.seed
  if (redR.bonusAmbrosiaTime > 0) {
    const bonusR = advanceAmbrosiaTimer({
      time: redR.bonusAmbrosiaTime,
      timeMultiplier: 1,
      noSingularityUpgradesCompletions: input.noSingularityUpgradesCompletions,
      ambrosiaGenerationSpeed: input.ambrosiaGenerationSpeed,
      ambrosiaTimerG,
      blueberryTime,
      ambrosia,
      lifetimeAmbrosia,
      seed: ambrosiaSeed,
      ambrosiaLuck: input.ambrosiaLuck,
      bonusAmbrosia: input.bonusAmbrosia,
      timePerAmbrosia: input.timePerAmbrosia,
      acceleratorMult: input.ambrosiaAcceleratorMult,
      brickOfLeadMult: input.ambrosiaBrickOfLeadMult
    })
    ambrosiaTimerG = bonusR.ambrosiaTimerG
    blueberryTime = bonusR.blueberryTime
    ambrosia = bonusR.ambrosia
    lifetimeAmbrosia = bonusR.lifetimeAmbrosia
    ambrosiaSeed = bonusR.seed
    for (const e of bonusR.events) events.push(e)
  }

  return {
    prestigecounter,
    transcendcounter,
    reincarnationcounter,
    ascensionCounter: ascR.ascensionCounter,
    ascensionCounterReal: ascR.ascensionCounterReal,
    quarkstimer,
    goldenQuarksTimer,
    octeractTimer: octR.octeractTimer,
    wowOcteracts: octR.wowOcteracts,
    totalWowOcteracts: octR.totalWowOcteracts,
    goldenQuarks: octR.goldenQuarks,
    quarksThisSingularity: octR.quarksThisSingularity,
    ascensionCounterRealReal: singR.ascensionCounterRealReal,
    singularityCounter: singR.singularityCounter,
    singChallengeTimer: singR.singChallengeTimer,
    autoPotionTimer: autoPotionR.autoPotionTimer,
    autoPotionTimerObtainium: autoPotionR.autoPotionTimerObtainium,
    ambrosiaTimerG,
    blueberryTime,
    ambrosia,
    lifetimeAmbrosia,
    ambrosiaSeed,
    redAmbrosiaTimerG: redR.redAmbrosiaTimerG,
    redAmbrosiaTime: redR.redAmbrosiaTime,
    redAmbrosia: redR.redAmbrosia,
    lifetimeRedAmbrosia: redR.lifetimeRedAmbrosia,
    redAmbrosiaSeed: redR.seed,
    events
  }
}

// ─── Fixture state — only the fields advanceAllTimers reads/writes ─────
//
// Static inputs (gates, speed multipliers, caps, pre-evaluated lookups)
// stay constant across the run; the mutable accumulators thread through
// tick-to-tick.

interface FixtureState {
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
}

type StaticInputs = Omit<AdvanceAllTimersInput, 'dt' | keyof FixtureState>

const buildInput = (statics: StaticInputs, state: FixtureState, dt: number): AdvanceAllTimersInput => ({
  dt,
  ...statics,
  ...state
})

const stateFromResult = (r: AdvanceAllTimersResult): FixtureState => ({
  prestigecounter: r.prestigecounter,
  transcendcounter: r.transcendcounter,
  reincarnationcounter: r.reincarnationcounter,
  ascensionCounter: r.ascensionCounter,
  ascensionCounterReal: r.ascensionCounterReal,
  quarkstimer: r.quarkstimer,
  goldenQuarksTimer: r.goldenQuarksTimer,
  octeractTimer: r.octeractTimer,
  wowOcteracts: r.wowOcteracts,
  totalWowOcteracts: r.totalWowOcteracts,
  goldenQuarks: r.goldenQuarks,
  quarksThisSingularity: r.quarksThisSingularity,
  ascensionCounterRealReal: r.ascensionCounterRealReal,
  singularityCounter: r.singularityCounter,
  singChallengeTimer: r.singChallengeTimer,
  autoPotionTimer: r.autoPotionTimer,
  autoPotionTimerObtainium: r.autoPotionTimerObtainium,
  ambrosiaTimerG: r.ambrosiaTimerG,
  blueberryTime: r.blueberryTime,
  ambrosia: r.ambrosia,
  lifetimeAmbrosia: r.lifetimeAmbrosia,
  ambrosiaSeed: r.ambrosiaSeed,
  redAmbrosiaTimerG: r.redAmbrosiaTimerG,
  redAmbrosiaTime: r.redAmbrosiaTime,
  redAmbrosia: r.redAmbrosia,
  lifetimeRedAmbrosia: r.lifetimeRedAmbrosia,
  redAmbrosiaSeed: r.redAmbrosiaSeed
})

// ─── Fixtures ──────────────────────────────────────────────────────────

// Fixture A — early-game quiet idle. Most blueberry/red-ambrosia/
// octeract gates blocking; only the basic counter timers advance.
const FIXTURE_A_STATICS: StaticInputs = {
  globalTimeMultiplier: 1,
  ascensionSpeedMulti: 1,
  maxQuarkTimer: 90000, // 25h
  exportGQPerHour: 0,   // GQ timer doesn't advance
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
  ambrosiaLuck: 100,
  bonusAmbrosia: 0,
  timePerAmbrosia: 600,
  ambrosiaAcceleratorMult: 1,
  ambrosiaBrickOfLeadMult: 1,
  noAmbrosiaUpgradesCompletions: 0,
  redAmbrosiaGenerationSpeed: 0,
  redAmbrosiaLuck: 100,
  ambrosiaTimePerRedAmbrosia: 0,
  timePerRedAmbrosia: 100000,
  redAmbrosiaBarRequirementMultiplier: 1
}

const FIXTURE_A_INITIAL: FixtureState = {
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
  ambrosiaSeed: 12345,
  redAmbrosiaTimerG: 0,
  redAmbrosiaTime: 0,
  redAmbrosia: 0,
  lifetimeRedAmbrosia: 0,
  redAmbrosiaSeed: 67890
}

// Fixture B — mid-game with ambrosia active. Generation speed and
// luck non-zero, so ambrosia bars mint repeatedly across 1000 ticks.
// Red ambrosia disabled (locked by completion gate), GQ-export timer
// running but capped at 168h, octeract running below sing-160 gate.
const FIXTURE_B_STATICS: StaticInputs = {
  ...FIXTURE_A_STATICS,
  globalTimeMultiplier: 2,
  exportGQPerHour: 1, // GQ export timer active
  octeractUnlocked: true,
  octeractPerSecond: 1.5,
  highestSingularityCount: 100, // below 160 threshold
  singularityCount: 100,
  goldenQuarksMultiplierExcludingBase: 1,
  noSingularityUpgradesCompletions: 1,
  ambrosiaGenerationSpeed: 1,
  ambrosiaLuck: 150 // ~1.5 ambrosia per bar (mult=1, luckMult=0/1)
}

const FIXTURE_B_INITIAL: FixtureState = {
  ...FIXTURE_A_INITIAL,
  goldenQuarks: 1000
}

// Fixture C — late-game with red ambrosia bonus feedback active.
// Both ambrosia + red-ambrosia generators running; the red-ambrosia
// bonus feeds back into ambrosia via the second advance call inside
// the bundle. Singularity 200, so the octeract GQ-giveaway block
// also fires (highestSingularityCount >= 160), exercising the
// goldenQuarksMultiplierExcludingBase path.
const FIXTURE_C_STATICS: StaticInputs = {
  ...FIXTURE_B_STATICS,
  highestSingularityCount: 200,
  singularityCount: 200,
  goldenQuarksMultiplierExcludingBase: 5, // arbitrary stat product
  noAmbrosiaUpgradesCompletions: 1,
  redAmbrosiaGenerationSpeed: 0.5,
  redAmbrosiaLuck: 100,
  ambrosiaTimePerRedAmbrosia: 0.5 // bonus-time feedback active
}

const FIXTURE_C_INITIAL: FixtureState = {
  ...FIXTURE_A_INITIAL,
  goldenQuarks: 5000,
  quarksThisSingularity: 1e9
}

// Fixture D — singularity challenge running. insideSingularityChallenge
// is true, so singChallengeTimer accumulates instead of resetting.
// Ascension oneMind active (ascensionSpeedMulti = 10), and the
// singularitySpeedMulti is non-trivial.
const FIXTURE_D_STATICS: StaticInputs = {
  ...FIXTURE_A_STATICS,
  ascensionSpeedMulti: 10,
  insideSingularityChallenge: true,
  singularitySpeedMulti: 2
}

const FIXTURE_D_INITIAL: FixtureState = {
  ...FIXTURE_A_INITIAL
}

// ─── Harness driver ────────────────────────────────────────────────────

interface RunResult {
  finalState: FixtureState
  totalEventCount: number
  /** Per-tick event count for tighter parity comparison. */
  perTickEventCounts: number[]
}

const runTicks = (
  statics: StaticInputs,
  initial: FixtureState,
  dt: number,
  ticks: number,
  fn: (input: AdvanceAllTimersInput) => AdvanceAllTimersResult
): RunResult => {
  let state = initial
  let totalEventCount = 0
  const perTickEventCounts: number[] = []
  for (let i = 0; i < ticks; i++) {
    const input = buildInput(statics, state, dt)
    const result = fn(input)
    state = stateFromResult(result)
    totalEventCount += result.events.length
    perTickEventCounts.push(result.events.length)
  }
  return { finalState: state, totalEventCount, perTickEventCounts }
}

describe('parity harness: advanceAllTimers over N=1000 ticks', () => {
  const fixtures: Array<{
    name: string
    statics: StaticInputs
    initial: FixtureState
    dt: number
    ticks: number
  }> = [
    {
      name: 'A — idle, only counter timers advance',
      statics: FIXTURE_A_STATICS,
      initial: FIXTURE_A_INITIAL,
      dt: 0.025,
      ticks: 1000
    },
    {
      name: 'B — ambrosia + octeract active (below sing-160)',
      statics: FIXTURE_B_STATICS,
      initial: FIXTURE_B_INITIAL,
      dt: 0.025,
      ticks: 1000
    },
    {
      name: 'C — sing-200 + red-ambrosia bonus feedback + GQ giveaway',
      statics: FIXTURE_C_STATICS,
      initial: FIXTURE_C_INITIAL,
      dt: 0.025,
      ticks: 1000
    },
    {
      name: 'D — inside singularity challenge, oneMind ascension',
      statics: FIXTURE_D_STATICS,
      initial: FIXTURE_D_INITIAL,
      dt: 0.025,
      ticks: 1000
    }
  ]

  for (const f of fixtures) {
    it(`${f.name} — migrated matches oracle`, () => {
      const migrated = runTicks(f.statics, f.initial, f.dt, f.ticks, advanceAllTimers)
      const oracle = runTicks(f.statics, f.initial, f.dt, f.ticks, oracleAdvanceAllTimers)
      expect(migrated.finalState).toEqual(oracle.finalState)
      expect(migrated.totalEventCount).toBe(oracle.totalEventCount)
      expect(migrated.perTickEventCounts).toEqual(oracle.perTickEventCounts)
    })
  }
})
