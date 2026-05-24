// Parity tests for the 7 simple-counter cases of addTimers.
// Old bodies transcribed verbatim from packages/web_ui/src/Helper.ts
// (addTimers, prestige / transcension / reincarnation / ascension /
// singularity / quarks / goldenQuarks cases pre-migration).

import { describe, expect, it } from 'vitest'
import {
  advanceAscensionTimer as newAdvanceAscension,
  type AdvanceAscensionTimerInput,
  type AdvanceAscensionTimerResult,
  advanceGoldenQuarksTimer as newAdvanceGoldenQuarks,
  type AdvanceGoldenQuarksTimerInput,
  advanceQuarksTimer as newAdvanceQuarks,
  type AdvanceQuarksTimerInput,
  advanceResetCounter as newAdvanceResetCounter,
  advanceSingularityTimer as newAdvanceSingularity,
  type AdvanceSingularityTimerInput,
  type AdvanceSingularityTimerResult
} from '../../src/tick/timers'

// ─── advanceResetCounter ────────────────────────────────────────────────

const oldAdvanceResetCounter = (counter: number, time: number, mult: number): number => {
  return counter + time * mult
}

describe('parity: advanceResetCounter', () => {
  const cases: Array<{ counter: number, time: number, mult: number }> = [
    { counter: 0, time: 0, mult: 1 },
    { counter: 100, time: 0.025, mult: 1 },
    { counter: 500, time: 1, mult: 10 },
    { counter: 1e6, time: 0.5, mult: 0.5 },
    // halfMind gives mult=10
    { counter: 50, time: 0.025, mult: 10 },
    // Slow tick (mult < 1)
    { counter: 50, time: 1, mult: 0.1 }
  ]
  for (const c of cases) {
    it(`counter=${c.counter} time=${c.time} mult=${c.mult}`, () => {
      expect(newAdvanceResetCounter(c.counter, c.time, c.mult))
        .toBe(oldAdvanceResetCounter(c.counter, c.time, c.mult))
    })
  }
})

// ─── advanceAscensionTimer ─────────────────────────────────────────────

const oldAdvanceAscension = (input: AdvanceAscensionTimerInput): AdvanceAscensionTimerResult => {
  // Legacy: timeMultiplier = 1 for ascension case, so the multiplication
  // collapses to `time * 1 * ascensionSpeedMulti`.
  return {
    ascensionCounter: input.ascensionCounter + input.time * input.ascensionSpeedMulti,
    ascensionCounterReal: input.ascensionCounterReal + input.time
  }
}

describe('parity: advanceAscensionTimer', () => {
  const cases: Array<{ name: string, input: AdvanceAscensionTimerInput }> = [
    {
      name: 'baseline (no ascensions, no speed)',
      input: { time: 0.025, ascensionCounter: 0, ascensionCounterReal: 0, ascensionSpeedMulti: 1 }
    },
    {
      name: 'oneMind active (speed=10)',
      input: { time: 1, ascensionCounter: 500, ascensionCounterReal: 500, ascensionSpeedMulti: 10 }
    },
    {
      name: 'fractional speed multiplier',
      input: { time: 0.5, ascensionCounter: 100, ascensionCounterReal: 200, ascensionSpeedMulti: 2.5 }
    },
    {
      name: 'real-time diverges from scaled-time',
      input: { time: 0.025, ascensionCounter: 1000, ascensionCounterReal: 100, ascensionSpeedMulti: 50 }
    }
  ]
  for (const c of cases) {
    it(c.name, () => {
      const newR = newAdvanceAscension(c.input)
      const oldR = oldAdvanceAscension(c.input)
      expect(newR.ascensionCounter).toBe(oldR.ascensionCounter)
      expect(newR.ascensionCounterReal).toBe(oldR.ascensionCounterReal)
    })
  }
})

// ─── advanceSingularityTimer ───────────────────────────────────────────

const oldAdvanceSingularity = (input: AdvanceSingularityTimerInput): AdvanceSingularityTimerResult => {
  // Legacy: timeMultiplier = 1 for singularity case.
  const ascensionCounterRealReal = input.ascensionCounterRealReal + input.time
  const singularityCounter = input.singularityCounter + input.time * input.singularitySpeedMulti
  let singChallengeTimer: number
  if (input.insideSingularityChallenge) {
    singChallengeTimer = input.singChallengeTimer + input.time * input.singularitySpeedMulti
  } else {
    singChallengeTimer = 0
  }
  return { ascensionCounterRealReal, singularityCounter, singChallengeTimer }
}

describe('parity: advanceSingularityTimer', () => {
  const cases: Array<{ name: string, input: AdvanceSingularityTimerInput }> = [
    {
      name: 'baseline (no singularities, not in challenge)',
      input: {
        time: 0.025,
        ascensionCounterRealReal: 0,
        singularityCounter: 0,
        singChallengeTimer: 0,
        insideSingularityChallenge: false,
        singularitySpeedMulti: 1
      }
    },
    {
      name: 'inside challenge accumulates challenge timer',
      input: {
        time: 1,
        ascensionCounterRealReal: 1000,
        singularityCounter: 1000,
        singChallengeTimer: 500,
        insideSingularityChallenge: true,
        singularitySpeedMulti: 1
      }
    },
    {
      name: 'leaving challenge resets challenge timer to 0',
      input: {
        time: 1,
        ascensionCounterRealReal: 1000,
        singularityCounter: 1000,
        singChallengeTimer: 9999,
        insideSingularityChallenge: false,
        singularitySpeedMulti: 1
      }
    },
    {
      name: 'brick-of-lead speed multiplier',
      input: {
        time: 0.025,
        ascensionCounterRealReal: 500,
        singularityCounter: 500,
        singChallengeTimer: 0,
        insideSingularityChallenge: false,
        singularitySpeedMulti: 2.5
      }
    },
    {
      name: 'realReal advances independently of speed multiplier',
      input: {
        time: 10,
        ascensionCounterRealReal: 0,
        singularityCounter: 0,
        singChallengeTimer: 0,
        insideSingularityChallenge: true,
        singularitySpeedMulti: 100
      }
    }
  ]
  for (const c of cases) {
    it(c.name, () => {
      const newR = newAdvanceSingularity(c.input)
      const oldR = oldAdvanceSingularity(c.input)
      expect(newR.ascensionCounterRealReal).toBe(oldR.ascensionCounterRealReal)
      expect(newR.singularityCounter).toBe(oldR.singularityCounter)
      expect(newR.singChallengeTimer).toBe(oldR.singChallengeTimer)
    })
  }
})

// ─── advanceQuarksTimer ────────────────────────────────────────────────

const oldAdvanceQuarks = (input: AdvanceQuarksTimerInput): number => {
  // Legacy: timeMultiplier = 1 for quarks case.
  let q = input.quarkstimer + input.time
  q = q > input.maxQuarkTimer ? input.maxQuarkTimer : q
  return q
}

describe('parity: advanceQuarksTimer', () => {
  const cases: Array<{ name: string, input: AdvanceQuarksTimerInput }> = [
    {
      name: 'baseline well under cap',
      input: { time: 0.025, quarkstimer: 0, maxQuarkTimer: 90000 }
    },
    {
      name: 'increment that crosses cap clamps to maxQuarkTimer',
      input: { time: 1000, quarkstimer: 89500, maxQuarkTimer: 90000 }
    },
    {
      name: 'already at cap stays at cap',
      input: { time: 0.025, quarkstimer: 90000, maxQuarkTimer: 90000 }
    },
    {
      name: 'extended cap from Research 8x20',
      input: { time: 1, quarkstimer: 100000, maxQuarkTimer: 180000 }
    }
  ]
  for (const c of cases) {
    it(c.name, () => {
      expect(newAdvanceQuarks(c.input)).toBe(oldAdvanceQuarks(c.input))
    })
  }
})

// ─── advanceGoldenQuarksTimer ──────────────────────────────────────────

const GOLDEN_QUARKS_CAP = 3600 * 168

const oldAdvanceGoldenQuarks = (input: AdvanceGoldenQuarksTimerInput): number => {
  if (input.exportGQPerHour === 0) return input.goldenQuarksTimer
  let g = input.goldenQuarksTimer + input.time
  g = g > GOLDEN_QUARKS_CAP ? GOLDEN_QUARKS_CAP : g
  return g
}

describe('parity: advanceGoldenQuarksTimer', () => {
  const cases: Array<{ name: string, input: AdvanceGoldenQuarksTimerInput }> = [
    {
      name: 'exportGQPerHour=0 short-circuits (timer unchanged)',
      input: { time: 100, goldenQuarksTimer: 500, exportGQPerHour: 0 }
    },
    {
      name: 'baseline accumulation',
      input: { time: 0.025, goldenQuarksTimer: 0, exportGQPerHour: 1 }
    },
    {
      name: 'increment crosses 168-hour cap',
      input: { time: 10000, goldenQuarksTimer: GOLDEN_QUARKS_CAP - 100, exportGQPerHour: 5 }
    },
    {
      name: 'already at cap stays at cap',
      input: { time: 1, goldenQuarksTimer: GOLDEN_QUARKS_CAP, exportGQPerHour: 5 }
    }
  ]
  for (const c of cases) {
    it(c.name, () => {
      expect(newAdvanceGoldenQuarks(c.input)).toBe(oldAdvanceGoldenQuarks(c.input))
    })
  }
})
