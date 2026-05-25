// Parity tests for the tackMiddle composition.
//
// tackMiddle bundles 4 already-tested logic functions (advanceRuneSacrifice,
// advanceAntSacrificeTimers + checkAntSacrificeReady, addObtainium,
// processAutoResearchTick) in their fixed sequential order. The unit-level
// parity is covered by the individual parity tests in automaticTools.parity
// and autoResearch.parity; these cases check that the composition preserves
// order, threads state correctly, gates the four cases independently, and
// merges events in legacy sequence.

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import {
  addObtainium,
  advanceAntSacrificeTimers,
  advanceRuneSacrifice,
  checkAntSacrificeReady
} from '../../src/tick/automaticTools'
import { processAutoResearchTick } from '../../src/tick/autoResearch'
import {
  tackMiddle as newTackMiddle,
  type TackMiddleInput,
  type TackMiddleResult
} from '../../src/tick/tackMiddle'

// Verbatim reference — runs the four cases in legacy order.
const oldTackMiddle = (input: TackMiddleInput): TackMiddleResult => {
  const events: TackMiddleResult['events'] = []

  let sacrificeTimer = input.sacrificeTimer
  if (input.runeSacrificeEnabled) {
    const r = advanceRuneSacrifice({
      time: input.dt,
      sacrificeTimer,
      autoSacrificeInterval: input.autoSacrificeInterval,
      offerings: input.offerings
    })
    sacrificeTimer = r.sacrificeTimer
    for (const e of r.events) events.push(e)
  }

  let antSacrificeTimer = input.antSacrificeTimer
  let antSacrificeTimerReal = input.antSacrificeTimerReal
  if (input.antSacrificeUnlocked) {
    const timerR = advanceAntSacrificeTimers({
      time: input.dt,
      globalDelta: input.globalDelta,
      antSacrificeTimer,
      antSacrificeTimerReal
    })
    antSacrificeTimer = timerR.antSacrificeTimer
    antSacrificeTimerReal = timerR.antSacrificeTimerReal

    const checkR = checkAntSacrificeReady({
      mode: input.autoSacrificeMode,
      crumbsThisSacrifice: input.crumbsThisSacrifice,
      antSacrificeTimerReal,
      autoSacrificeEnabled: input.autoSacrificeEnabled,
      availableRebornELO: input.availableRebornELO,
      onlySacrificeMaxRebornELO: input.onlySacrificeMaxRebornELO,
      alwaysSacrificeMaxRebornELO: input.alwaysSacrificeMaxRebornELO,
      antSacrificeTimer,
      autoSacrificeThreshold: input.autoSacrificeThreshold,
      immortalELOGain: input.immortalELOGain,
      immortalELO: input.immortalELO,
      rebornELO: input.rebornELO
    })
    for (const e of checkR.events) events.push(e)
  }

  let obtainium = input.obtainium
  if (input.research61 === 1) {
    const obtR = addObtainium({
      obtainium,
      obtainiumGain: input.obtainiumGain,
      ascensionChallenge: input.ascensionChallenge,
      taxmanLastStandEnabled: input.taxmanLastStandEnabled,
      taxmanLastStandCompletions: input.taxmanLastStandCompletions
    })
    obtainium = obtR.obtainium
    for (const e of obtR.events) events.push(e)
  } else {
    events.push({ kind: 'obtainium-multiplier-recompute-requested' })
  }

  const arR = processAutoResearchTick({
    autoResearchToggle: input.autoResearchToggle,
    autoResearch: input.autoResearch,
    autoResearchMode: input.autoResearchMode,
    roombaUnlocked: input.roombaUnlocked,
    challengecompletions14: input.challengecompletions14
  })
  for (const e of arR.events) events.push(e)

  return { sacrificeTimer, antSacrificeTimer, antSacrificeTimerReal, obtainium, events }
}

const defaultInput = (overrides: Partial<TackMiddleInput> = {}): TackMiddleInput => ({
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

describe('parity: tackMiddle', () => {
  const cases: Array<{ name: string, input: TackMiddleInput }> = [
    {
      name: 'quiet tick — all gates blocking, only obtainium-recompute fires',
      input: defaultInput()
    },
    {
      name: 'runeSacrifice gate off — sacrificeTimer unchanged + no event',
      input: defaultInput({
        runeSacrificeEnabled: false,
        sacrificeTimer: 99,
        autoSacrificeInterval: 0.01,
        offerings: new Decimal(1e10)
      })
    },
    {
      name: 'runeSacrifice gate on, fires + resets',
      input: defaultInput({
        runeSacrificeEnabled: true,
        sacrificeTimer: 0.95,
        autoSacrificeInterval: 1,
        offerings: new Decimal(1e6),
        dt: 0.1
      })
    },
    {
      name: 'runeSacrifice gate on, under threshold',
      input: defaultInput({
        runeSacrificeEnabled: true,
        sacrificeTimer: 0.1,
        autoSacrificeInterval: 1,
        offerings: new Decimal(1e6)
      })
    },
    {
      name: 'antSacrifice gate off — both timers unchanged + no event',
      input: defaultInput({
        antSacrificeUnlocked: false,
        antSacrificeTimer: 100,
        antSacrificeTimerReal: 100,
        globalDelta: 10
      })
    },
    {
      name: 'antSacrifice gate on, timer advances, check fails',
      input: defaultInput({
        antSacrificeUnlocked: true,
        globalDelta: 1,
        antSacrificeTimer: 0,
        antSacrificeTimerReal: 0,
        autoSacrificeEnabled: false
      })
    },
    {
      name: 'antSacrifice gate on, fires (all conditions met)',
      input: defaultInput({
        antSacrificeUnlocked: true,
        globalDelta: 1,
        antSacrificeTimer: 120,
        antSacrificeTimerReal: 60,
        autoSacrificeEnabled: true,
        crumbsThisSacrifice: new Decimal(1e50),
        autoSacrificeMode: 'InGameTime',
        autoSacrificeThreshold: 60
      })
    },
    {
      name: 'research61 === 1 — addObtainium credits balance',
      input: defaultInput({
        research61: 1,
        obtainium: new Decimal(100),
        obtainiumGain: new Decimal(50),
        ascensionChallenge: 0
      })
    },
    {
      name: 'research61 === 1, c14 abort — obtainium unchanged but no recompute',
      input: defaultInput({
        research61: 1,
        obtainium: new Decimal(100),
        obtainiumGain: new Decimal(1e30),
        ascensionChallenge: 14
      })
    },
    {
      name: 'research61 !== 1 — emit obtainium-multiplier-recompute-requested',
      input: defaultInput({
        research61: 0,
        obtainium: new Decimal(100)
      })
    },
    {
      name: 'auto-research manual mode fires',
      input: defaultInput({
        autoResearchToggle: true,
        autoResearch: 5,
        autoResearchMode: 'manual'
      })
    },
    {
      name: 'auto-research Roomba mode fires',
      input: defaultInput({
        autoResearchToggle: true,
        autoResearch: 5,
        autoResearchMode: 'cheapest',
        roombaUnlocked: true,
        challengecompletions14: 25
      })
    },
    {
      name: 'auto-research Roomba mode without unlock — no event',
      input: defaultInput({
        autoResearchToggle: true,
        autoResearch: 5,
        autoResearchMode: 'cheapest',
        roombaUnlocked: false
      })
    },
    {
      name: 'event order — all four cases fire (rune, ant, addObtainium, auto-research)',
      input: defaultInput({
        // Rune fires
        runeSacrificeEnabled: true,
        sacrificeTimer: 1,
        autoSacrificeInterval: 1,
        offerings: new Decimal(1e6),
        // Ant fires
        antSacrificeUnlocked: true,
        globalDelta: 1,
        antSacrificeTimer: 120,
        antSacrificeTimerReal: 60,
        autoSacrificeEnabled: true,
        crumbsThisSacrifice: new Decimal(1e50),
        autoSacrificeMode: 'InGameTime',
        autoSacrificeThreshold: 60,
        // addObtainium fires
        research61: 1,
        obtainium: new Decimal(1e6),
        obtainiumGain: new Decimal(100),
        // Auto-research manual fires
        autoResearchToggle: true,
        autoResearch: 5,
        autoResearchMode: 'manual'
      })
    },
    {
      name: 'event order — rune + ant + obtainium-recompute + auto-research Roomba',
      input: defaultInput({
        runeSacrificeEnabled: true,
        sacrificeTimer: 1,
        autoSacrificeInterval: 1,
        offerings: new Decimal(1e6),
        antSacrificeUnlocked: true,
        globalDelta: 1,
        antSacrificeTimer: 120,
        antSacrificeTimerReal: 60,
        autoSacrificeEnabled: true,
        crumbsThisSacrifice: new Decimal(1e50),
        autoSacrificeMode: 'InGameTime',
        autoSacrificeThreshold: 60,
        // research61 !== 1 → recompute event
        research61: 0,
        autoResearchToggle: true,
        autoResearch: 5,
        autoResearchMode: 'cheapest',
        roombaUnlocked: true,
        challengecompletions14: 12
      })
    },
    {
      name: 'big dt (offline catch-up scenario)',
      input: defaultInput({
        dt: 100,
        runeSacrificeEnabled: true,
        sacrificeTimer: 0,
        autoSacrificeInterval: 1,
        offerings: new Decimal(1e9),
        antSacrificeUnlocked: true,
        globalDelta: 10,
        antSacrificeTimer: 0,
        antSacrificeTimerReal: 0,
        autoSacrificeEnabled: true,
        crumbsThisSacrifice: new Decimal(1e50),
        autoSacrificeMode: 'RealTime',
        autoSacrificeThreshold: 50,
        research61: 1,
        obtainium: new Decimal(1e6),
        obtainiumGain: new Decimal(1e9),
        autoResearchToggle: true,
        autoResearch: 10,
        autoResearchMode: 'cheapest',
        roombaUnlocked: true,
        challengecompletions14: 750
      })
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newTackMiddle(c.input)
      const oldR = oldTackMiddle(c.input)
      expect(newR.sacrificeTimer).toBe(oldR.sacrificeTimer)
      expect(newR.antSacrificeTimer).toBe(oldR.antSacrificeTimer)
      expect(newR.antSacrificeTimerReal).toBe(oldR.antSacrificeTimerReal)
      expect(newR.obtainium.toString()).toBe(oldR.obtainium.toString())
      expect(newR.events).toEqual(oldR.events)
    })
  }
})
