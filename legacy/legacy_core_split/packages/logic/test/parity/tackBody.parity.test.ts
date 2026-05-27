// Parity tests for the Phase 4 tackBody composition.
//
// tackBody bundles three already-tested logic functions (advanceAllTimers,
// tackMiddle, tackTail) in their fixed sequential order, with the head +
// middle bundles gated on !timeWarp and tail running always. The
// individual bundle parity is covered by the per-bundle parity tests
// (timersBundle, tackMiddle, tackTail); these cases check that the
// orchestrator preserves order, threads state correctly, and merges
// events in legacy bundle sequence.
//
// Important: this parity test verifies tackBody == "3 bundles in
// sequence with no intermediate event dispatch". It does NOT compare
// against the pre-bundle web_ui structure (which dispatched events
// between bundles). See packages/logic/src/tick/tack.ts header for the
// dispatch-ordering caveat.

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import { tackBody as newTackBody, type TackBodyInput, type TackBodyResult } from '../../src/tick/tack'
import { advanceAllTimers, type AdvanceAllTimersInput } from '../../src/tick/timersBundle'
import { tackMiddle, type TackMiddleInput } from '../../src/tick/tackMiddle'
import { type SweepStates, tackTail, type TackTailInput } from '../../src/tick/tackTail'

const oldTackBody = (input: TackBodyInput): TackBodyResult => {
  const events: TackBodyResult['events'] = []
  let head: TackBodyResult['head']
  let middle: TackBodyResult['middle']
  if (!input.timeWarp) {
    if (input.head === undefined || input.middle === undefined) {
      throw new Error('test stub')
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

const defaultHead = (overrides: Partial<AdvanceAllTimersInput> = {}): AdvanceAllTimersInput => ({
  dt: 0.025,
  globalTimeMultiplier: 1,
  prestigecounter: 0,
  transcendcounter: 0,
  reincarnationcounter: 0,
  ascensionCounter: 0,
  ascensionCounterReal: 0,
  ascensionSpeedMulti: 1,
  quarkstimer: 0,
  maxQuarkTimer: 90000,
  goldenQuarksTimer: 0,
  exportGQPerHour: 0,
  octeractUnlocked: false,
  octeractTimer: 0,
  wowOcteracts: 0,
  totalWowOcteracts: 0,
  goldenQuarks: 0,
  quarksThisSingularity: 0,
  octeractPerSecond: 0,
  highestSingularityCount: 0,
  singularityCount: 0,
  goldenQuarksMultiplierExcludingBase: 1,
  ascensionCounterRealReal: 0,
  singularityCounter: 0,
  singChallengeTimer: 0,
  insideSingularityChallenge: false,
  singularitySpeedMulti: 1,
  autoPotionTimer: 0,
  autoPotionTimerObtainium: 0,
  autoPotionToggleOffering: false,
  autoPotionToggleObtainium: false,
  offeringPotionCount: 0,
  obtainiumPotionCount: 0,
  autoPotionSpeedMult: 1,
  noSingularityUpgradesCompletions: 0,
  ambrosiaGenerationSpeed: 0,
  ambrosiaTimerG: 0,
  blueberryTime: 0,
  ambrosia: 0,
  lifetimeAmbrosia: 0,
  ambrosiaSeed: 1,
  ambrosiaLuck: 0,
  bonusAmbrosia: 0,
  timePerAmbrosia: 600,
  ambrosiaAcceleratorMult: 1,
  ambrosiaBrickOfLeadMult: 1,
  noAmbrosiaUpgradesCompletions: 0,
  redAmbrosiaGenerationSpeed: 0,
  redAmbrosiaTimerG: 0,
  redAmbrosiaTime: 0,
  redAmbrosia: 0,
  lifetimeRedAmbrosia: 0,
  redAmbrosiaSeed: 1,
  redAmbrosiaLuck: 0,
  ambrosiaTimePerRedAmbrosia: 0,
  timePerRedAmbrosia: 100000,
  redAmbrosiaBarRequirementMultiplier: 1,
  ...overrides
})

const defaultMiddle = (overrides: Partial<TackMiddleInput> = {}): TackMiddleInput => ({
  dt: 0.025,
  runeSacrificeEnabled: false,
  sacrificeTimer: 0,
  autoSacrificeInterval: 1,
  offerings: new Decimal(0),
  antSacrificeUnlocked: false,
  globalDelta: 1,
  antSacrificeTimer: 0,
  antSacrificeTimerReal: 0,
  autoSacrificeMode: 'InGameTime',
  crumbsThisSacrifice: new Decimal(0),
  autoSacrificeEnabled: false,
  availableRebornELO: 100,
  onlySacrificeMaxRebornELO: false,
  alwaysSacrificeMaxRebornELO: false,
  autoSacrificeThreshold: 60,
  immortalELOGain: 0,
  immortalELO: 0,
  rebornELO: 0,
  research61: 0,
  obtainium: new Decimal(0),
  obtainiumGain: new Decimal(0),
  ascensionChallenge: 0,
  taxmanLastStandEnabled: false,
  taxmanLastStandCompletions: 0,
  autoResearchToggle: false,
  autoResearch: 0,
  autoResearchMode: 'manual',
  roombaUnlocked: false,
  challengecompletions14: 0,
  ...overrides
})

const defaultTail = (overrides: Partial<TackTailInput> = {}): TackTailInput => ({
  dt: 0.025,
  highestchallengecompletions3: 0,
  autoOfferingCounter: 0,
  offerings: new Decimal(1000),
  sweepState: { kind: 'idle' } as SweepStates,
  timeSinceLastStateChange: 0,
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
  autoResetTimerPrestige: 0,
  transcendMode: 'amount',
  toggle21: false,
  upgrade89: 0,
  transcendPoints: new Decimal(0),
  transcendPointGain: new Decimal(0),
  transcendamount: 1,
  coinsThisTranscension: new Decimal(0),
  autoResetTimerTranscension: 0,
  reincarnationMode: 'amount',
  toggle27: false,
  research46: 0,
  reincarnationPoints: new Decimal(0),
  reincarnationPointGain: new Decimal(0),
  reincarnationamount: 1,
  transcendShards: new Decimal(0),
  autoResetTimerReincarnation: 0,
  ascensionChallenge: 0,
  transcensionChallenge: 0,
  reincarnationChallenge: 0,
  ...overrides
})

describe('parity: tackBody', () => {
  const cases: Array<{ name: string, input: TackBodyInput }> = [
    {
      name: 'quiet tick (timeWarp=false) — only obtainium-recompute fires',
      input: {
        timeWarp: false,
        head: defaultHead(),
        middle: defaultMiddle(),
        tail: defaultTail()
      }
    },
    {
      name: 'timeWarp=true — skip head + middle, only tail runs',
      input: {
        timeWarp: true,
        tail: defaultTail()
      }
    },
    {
      name: 'timeWarp=true with sweep transition — tail emits event, no head/middle',
      input: {
        timeWarp: true,
        tail: defaultTail({
          sweepState: { kind: 'idle' },
          shouldRunSweep: true
        })
      }
    },
    {
      name: 'head fires autoPotion (gate threshold crossed) — event in head section',
      input: {
        timeWarp: false,
        head: defaultHead({
          highestSingularityCount: 10,
          autoPotionTimer: 200,
          autoPotionTimerObtainium: 200,
          autoPotionToggleOffering: true,
          offeringPotionCount: 5,
          autoPotionSpeedMult: 1
        }),
        middle: defaultMiddle(),
        tail: defaultTail()
      }
    },
    {
      name: 'middle fires runeSacrifice — event in middle section',
      input: {
        timeWarp: false,
        head: defaultHead(),
        middle: defaultMiddle({
          runeSacrificeEnabled: true,
          sacrificeTimer: 1,
          autoSacrificeInterval: 1,
          offerings: new Decimal(1e6)
        }),
        tail: defaultTail()
      }
    },
    {
      name: 'all three bundles fire events — order check',
      input: {
        timeWarp: false,
        // Head: autoPotion + ambrosia
        head: defaultHead({
          highestSingularityCount: 10,
          autoPotionTimer: 200,
          autoPotionToggleOffering: true,
          offeringPotionCount: 5,
          autoPotionSpeedMult: 1,
          noSingularityUpgradesCompletions: 5,
          ambrosiaGenerationSpeed: 100,
          ambrosiaTimerG: 1,
          blueberryTime: 1000,
          ambrosiaLuck: 100
        }),
        // Middle: runeSacrifice + autoResearch
        middle: defaultMiddle({
          runeSacrificeEnabled: true,
          sacrificeTimer: 1,
          autoSacrificeInterval: 1,
          offerings: new Decimal(1e6),
          autoResearchToggle: true,
          autoResearch: 5,
          autoResearchMode: 'manual'
        }),
        // Tail: auto-prestige + sweep
        tail: defaultTail({
          sweepState: { kind: 'idle' },
          shouldRunSweep: true,
          prestigeMode: 'amount',
          toggle15: true,
          autoPrestigeMilestone: 1,
          prestigePoints: new Decimal(100),
          prestigePointGain: new Decimal(1e6),
          prestigeamount: 2,
          coinsThisPrestige: new Decimal(1e17)
        })
      }
    },
    {
      name: 'big offline catch-up dt',
      input: {
        timeWarp: false,
        head: defaultHead({
          dt: 60,
          highestSingularityCount: 10,
          autoPotionTimer: 0,
          autoPotionToggleOffering: true,
          offeringPotionCount: 100,
          autoPotionSpeedMult: 1
        }),
        middle: defaultMiddle({
          dt: 60,
          runeSacrificeEnabled: true,
          autoSacrificeInterval: 1,
          offerings: new Decimal(1e9)
        }),
        tail: defaultTail({ dt: 60 })
      }
    },
    {
      name: 'head returns timers, middle picks up offerings unchanged (state thread)',
      input: {
        timeWarp: false,
        head: defaultHead({
          prestigecounter: 500,
          transcendcounter: 200
        }),
        middle: defaultMiddle({
          offerings: new Decimal(500)
        }),
        tail: defaultTail({
          offerings: new Decimal(500)
        })
      }
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newTackBody(c.input)
      const oldR = oldTackBody(c.input)
      expect(newR.events.length).toBe(oldR.events.length)
      for (let i = 0; i < newR.events.length; i++) {
        expect(newR.events[i].kind).toBe(oldR.events[i].kind)
      }
      // head writebacks
      if (oldR.head) {
        expect(newR.head?.prestigecounter).toBe(oldR.head.prestigecounter)
        expect(newR.head?.octeractTimer).toBe(oldR.head.octeractTimer)
        expect(newR.head?.ambrosia).toBe(oldR.head.ambrosia)
      } else {
        expect(newR.head).toBeUndefined()
      }
      // middle writebacks
      if (oldR.middle) {
        expect(newR.middle?.sacrificeTimer).toBe(oldR.middle.sacrificeTimer)
        expect(newR.middle?.antSacrificeTimer).toBe(oldR.middle.antSacrificeTimer)
        expect(newR.middle?.obtainium.toString()).toBe(oldR.middle.obtainium.toString())
      } else {
        expect(newR.middle).toBeUndefined()
      }
      // tail writebacks
      expect(newR.tail.autoOfferingCounter).toBe(oldR.tail.autoOfferingCounter)
      expect(newR.tail.offerings.toString()).toBe(oldR.tail.offerings.toString())
      expect(newR.tail.autoResetTimerPrestige).toBe(oldR.tail.autoResetTimerPrestige)
    })
  }

  it('rejects undefined head when timeWarp=false', () => {
    expect(() => {
      newTackBody({
        timeWarp: false,
        // head missing
        middle: defaultMiddle(),
        tail: defaultTail()
      })
    }).toThrow(/head and middle inputs are required/)
  })

  it('rejects undefined middle when timeWarp=false', () => {
    expect(() => {
      newTackBody({
        timeWarp: false,
        head: defaultHead(),
        // middle missing
        tail: defaultTail()
      })
    }).toThrow(/head and middle inputs are required/)
  })

  it('accepts undefined head + middle when timeWarp=true', () => {
    const r = newTackBody({
      timeWarp: true,
      tail: defaultTail()
    })
    expect(r.head).toBeUndefined()
    expect(r.middle).toBeUndefined()
    expect(r.tail).toBeDefined()
  })
})
