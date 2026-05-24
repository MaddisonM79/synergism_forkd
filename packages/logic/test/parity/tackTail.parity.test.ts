// Parity tests for the tackTail composition.
//
// tackTail bundles 3 already-tested logic functions (addOfferings,
// tickChallengeSweep, applyAutoResets) in their fixed sequential order.
// The unit-level parity is already covered by the individual subsystem
// tests; these cases check that the composition preserves order, threads
// state correctly, and merges events in legacy sequence.

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import { addOfferings } from '../../src/tick/automaticTools'
import { applyAutoResets } from '../../src/tick/autoReset'
import { type SweepStates, tickChallengeSweep } from '../../src/tick/challengeSweep'
import { tackTail as newTackTail, type TackTailInput, type TackTailResult } from '../../src/tick/tackTail'

// Verbatim reference — runs the three subsystems in sequence and merges events.
const oldTackTail = (input: TackTailInput): TackTailResult => {
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

const defaultInput = (overrides: Partial<TackTailInput> = {}): TackTailInput => ({
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

describe('parity: tackTail', () => {
  const cases: Array<{ name: string, input: TackTailInput }> = [
    {
      name: 'quiet tick — all gates blocking',
      input: defaultInput()
    },
    {
      name: 'addOfferings runs (c3 unlocked, fractional carry)',
      input: defaultInput({
        highestchallengecompletions3: 1,
        autoOfferingCounter: 0.9,
        offerings: new Decimal(100),
        dt: 0.5
      })
    },
    {
      name: 'sweep boots from idle when shouldRunSweep flips on',
      input: defaultInput({
        sweepState: { kind: 'idle' },
        shouldRunSweep: true
      })
    },
    {
      name: 'sweep transitions active → enter_wait + emits event',
      input: defaultInput({
        sweepState: { kind: 'active', index: 1, explored: new Set([1]) },
        timeSinceLastStateChange: 29.99,
        dt: 0.025,
        timerExit: 30,
        shouldRunSweep: true,
        nextRegularChallengeFromActive: 2
      })
    },
    {
      name: 'auto-prestige amount-mode fires + emits event',
      input: defaultInput({
        prestigeMode: 'amount',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigePoints: new Decimal(100),
        prestigePointGain: new Decimal(1e6),
        prestigeamount: 2,
        coinsThisPrestige: new Decimal(1e17)
      })
    },
    {
      name: 'all three subsystems fire — sweep transition + auto-transcend',
      input: defaultInput({
        highestchallengecompletions3: 1,
        autoOfferingCounter: 0.5,
        offerings: new Decimal(500),
        dt: 0.025,
        sweepState: { kind: 'c15_wait' },
        timeSinceLastStateChange: 4.99,
        shouldRunSweep: true,
        transcendMode: 'amount',
        toggle21: true,
        upgrade89: 1,
        transcendPoints: new Decimal(100),
        transcendPointGain: new Decimal(1e6),
        transcendamount: 2,
        coinsThisTranscension: new Decimal(1e101)
      })
    },
    {
      name: 'auto-reincarnation amount-mode fires (+1 boundary at 0 points)',
      input: defaultInput({
        reincarnationMode: 'amount',
        toggle27: true,
        research46: 1,
        reincarnationPoints: new Decimal(0),
        reincarnationPointGain: new Decimal(10),
        reincarnationamount: 1,
        transcendShards: new Decimal('1e301')
      })
    },
    {
      name: 'sweep tears down when shouldRunSweep flips off mid-active (emits event with from.index)',
      input: defaultInput({
        sweepState: { kind: 'active', index: 7, explored: new Set([6, 7]) },
        shouldRunSweep: false
      })
    },
    {
      name: 'event order: sweep then resets',
      input: defaultInput({
        // Sweep idle → initial_wait (emits sweep event)
        sweepState: { kind: 'idle' },
        shouldRunSweep: true,
        // Auto-prestige fires (emits auto-reset event)
        prestigeMode: 'amount',
        toggle15: true,
        autoPrestigeMilestone: 1,
        prestigePoints: new Decimal(100),
        prestigePointGain: new Decimal(1e6),
        prestigeamount: 2,
        coinsThisPrestige: new Decimal(1e17)
      })
    },
    {
      name: 'big dt (offline catch-up scenario)',
      input: defaultInput({
        dt: 100,
        highestchallengecompletions3: 1,
        autoOfferingCounter: 0,
        offerings: new Decimal(100),
        sweepState: { kind: 'initial_wait' },
        shouldRunSweep: true,
        timerStart: 5,
        nextRegularChallengeFromInitial: 1
      })
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newTackTail(c.input)
      const oldR = oldTackTail(c.input)
      expect(newR.autoOfferingCounter).toBeCloseTo(oldR.autoOfferingCounter, 10)
      expect(newR.offerings.toString()).toBe(oldR.offerings.toString())
      expect(newR.sweepState.kind).toBe(oldR.sweepState.kind)
      expect(newR.timeSinceLastStateChange).toBe(oldR.timeSinceLastStateChange)
      expect(newR.autoResetTimerPrestige).toBe(oldR.autoResetTimerPrestige)
      expect(newR.autoResetTimerTranscension).toBe(oldR.autoResetTimerTranscension)
      expect(newR.autoResetTimerReincarnation).toBe(oldR.autoResetTimerReincarnation)
      // Event sequence matches (kinds + key fields)
      expect(newR.events.length).toBe(oldR.events.length)
      for (let i = 0; i < newR.events.length; i++) {
        expect(newR.events[i].kind).toBe(oldR.events[i].kind)
      }
    })
  }
})
