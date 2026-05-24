// Parity tests for the ambrosia + redAmbrosia addTimers cases and the
// underlying seededRandom helper.
//
// Old bodies transcribed verbatim from packages/web_ui/src/Helper.ts
// (addTimers, ambrosia + redAmbrosia switch cases pre-migration) and
// packages/web_ui/src/RNG.ts (seededRandom). The legacy code reads/writes
// player.seed[index] directly; the parity harness threads the seed
// through a local variable instead.

import { MersenneTwister } from 'fast-mersenne-twister'
import { describe, expect, it } from 'vitest'
import { calculateRequiredBlueberryTime, calculateRequiredRedAmbrosiaTime } from '../../src/mechanics/ambrosia'
import {
  seededRandom as newSeededRandom,
  type SeededRandomResult
} from '../../src/math/rng'
import {
  advanceAmbrosiaTimer as newAdvanceAmbrosia,
  type AdvanceAmbrosiaTimerInput,
  type AdvanceAmbrosiaTimerResult,
  advanceRedAmbrosiaTimer as newAdvanceRedAmbrosia,
  type AdvanceRedAmbrosiaTimerInput,
  type AdvanceRedAmbrosiaTimerResult
} from '../../src/tick/timers'

// ─── seededRandom ──────────────────────────────────────────────────────

const oldSeededRandom = (seed: number): SeededRandomResult => ({
  value: MersenneTwister(seed).random(),
  newSeed: seed + 1
})

describe('parity: seededRandom', () => {
  const seeds = [0, 1, 42, 1024, 99999, 2147483647, 1735689600]
  for (const seed of seeds) {
    it(`seed=${seed}`, () => {
      const a = newSeededRandom(seed)
      const b = oldSeededRandom(seed)
      expect(a.value).toBe(b.value)
      expect(a.newSeed).toBe(b.newSeed)
    })
  }
})

// ─── advanceAmbrosiaTimer ──────────────────────────────────────────────

const oldAdvanceAmbrosia = (input: AdvanceAmbrosiaTimerInput): AdvanceAmbrosiaTimerResult => {
  if (input.noSingularityUpgradesCompletions <= 0) {
    return {
      ambrosiaTimerG: input.ambrosiaTimerG,
      blueberryTime: input.blueberryTime,
      ambrosia: input.ambrosia,
      lifetimeAmbrosia: input.lifetimeAmbrosia,
      seed: input.seed,
      events: []
    }
  }
  if (input.ambrosiaGenerationSpeed === 0) {
    return {
      ambrosiaTimerG: input.ambrosiaTimerG,
      blueberryTime: input.blueberryTime,
      ambrosia: input.ambrosia,
      lifetimeAmbrosia: input.lifetimeAmbrosia,
      seed: input.seed,
      events: []
    }
  }

  let ambrosiaTimerG = input.ambrosiaTimerG + input.time * input.timeMultiplier
  if (ambrosiaTimerG < 0.125) {
    return {
      ambrosiaTimerG,
      blueberryTime: input.blueberryTime,
      ambrosia: input.ambrosia,
      lifetimeAmbrosia: input.lifetimeAmbrosia,
      seed: input.seed,
      events: []
    }
  }

  let blueberryTime = input.blueberryTime + Math.floor(8 * ambrosiaTimerG) / 8 * input.ambrosiaGenerationSpeed
  ambrosiaTimerG %= 0.125

  let ambrosia = input.ambrosia
  let lifetimeAmbrosia = input.lifetimeAmbrosia
  let seed = input.seed
  let totalGained = 0

  let timeToAmbrosia = calculateRequiredBlueberryTime({
    timePerAmbrosia: input.timePerAmbrosia,
    lifetimeAmbrosia,
    acceleratorMult: input.acceleratorMult,
    brickOfLeadMult: input.brickOfLeadMult
  })

  while (blueberryTime >= timeToAmbrosia) {
    const rng = newSeededRandom(seed)
    seed = rng.newSeed
    const ambrosiaMult = Math.floor(input.ambrosiaLuck / 100)
    const luckMult = rng.value < input.ambrosiaLuck / 100 - Math.floor(input.ambrosiaLuck / 100) ? 1 : 0
    const ambrosiaToGain = (ambrosiaMult + luckMult) + input.bonusAmbrosia

    ambrosia += ambrosiaToGain
    lifetimeAmbrosia += ambrosiaToGain
    totalGained += ambrosiaToGain
    blueberryTime -= timeToAmbrosia

    timeToAmbrosia = calculateRequiredBlueberryTime({
      timePerAmbrosia: input.timePerAmbrosia,
      lifetimeAmbrosia,
      acceleratorMult: input.acceleratorMult,
      brickOfLeadMult: input.brickOfLeadMult
    })
  }

  return {
    ambrosiaTimerG,
    blueberryTime,
    ambrosia,
    lifetimeAmbrosia,
    seed,
    events: [{ kind: 'ambrosia-gained', amount: totalGained }]
  }
}

const defaultAmbrosia = (overrides: Partial<AdvanceAmbrosiaTimerInput> = {}): AdvanceAmbrosiaTimerInput => ({
  time: 0.025,
  timeMultiplier: 1,
  noSingularityUpgradesCompletions: 1,
  ambrosiaGenerationSpeed: 1,
  ambrosiaTimerG: 0,
  blueberryTime: 0,
  ambrosia: 0,
  lifetimeAmbrosia: 0,
  seed: 42,
  ambrosiaLuck: 100,
  bonusAmbrosia: 0,
  timePerAmbrosia: 45,
  acceleratorMult: 1,
  brickOfLeadMult: 1,
  ...overrides
})

describe('parity: advanceAmbrosiaTimer', () => {
  const cases: Array<{ name: string, input: AdvanceAmbrosiaTimerInput }> = [
    {
      name: 'gate — completions=0 (feature locked)',
      input: defaultAmbrosia({ noSingularityUpgradesCompletions: 0, ambrosiaTimerG: 0.5 })
    },
    {
      name: 'gate — ambrosiaGenerationSpeed=0',
      input: defaultAmbrosia({ ambrosiaGenerationSpeed: 0, ambrosiaTimerG: 0.5 })
    },
    {
      name: 'sub-1/8s timer accumulates but no work',
      input: defaultAmbrosia({ time: 0.05, ambrosiaTimerG: 0.05, blueberryTime: 0 })
    },
    {
      name: 'fractional timer rounds to nearest 1/8',
      input: defaultAmbrosia({
        time: 0.025,
        ambrosiaTimerG: 0.125,
        ambrosiaGenerationSpeed: 8,
        blueberryTime: 0
      })
    },
    {
      name: 'gains 1 ambrosia (single bar)',
      input: defaultAmbrosia({
        time: 0,
        ambrosiaTimerG: 0.125,
        ambrosiaGenerationSpeed: 1000, // jump blueberryTime past 45
        blueberryTime: 0,
        ambrosiaLuck: 100,
        bonusAmbrosia: 0,
        lifetimeAmbrosia: 0
      })
    },
    {
      name: 'gains multiple ambrosia bars in one tick',
      input: defaultAmbrosia({
        time: 0,
        ambrosiaTimerG: 0.625, // 5*0.125 = 5 ticks worth
        ambrosiaGenerationSpeed: 100,
        blueberryTime: 0,
        ambrosiaLuck: 100,
        lifetimeAmbrosia: 0
      })
    },
    {
      name: 'ambrosiaLuck >= 200 (ambrosiaMult=2, luckMult always 0 from floor)',
      input: defaultAmbrosia({
        time: 0,
        ambrosiaTimerG: 0.125,
        ambrosiaGenerationSpeed: 1000,
        ambrosiaLuck: 200,
        lifetimeAmbrosia: 0,
        seed: 1
      })
    },
    {
      name: 'fractional ambrosiaLuck (150) — luckMult depends on RNG roll',
      input: defaultAmbrosia({
        time: 0,
        ambrosiaTimerG: 0.125,
        ambrosiaGenerationSpeed: 1000,
        ambrosiaLuck: 150,
        lifetimeAmbrosia: 0,
        seed: 7
      })
    },
    {
      name: 'bonusAmbrosia adds to each iteration',
      input: defaultAmbrosia({
        time: 0,
        ambrosiaTimerG: 0.25,
        ambrosiaGenerationSpeed: 100,
        bonusAmbrosia: 5,
        lifetimeAmbrosia: 0
      })
    },
    {
      name: 'lifetimeAmbrosia >= 10000 (power-scaling kicks in)',
      input: defaultAmbrosia({
        time: 0,
        ambrosiaTimerG: 0.125,
        ambrosiaGenerationSpeed: 1e6,
        lifetimeAmbrosia: 50000,
        ambrosiaLuck: 100
      })
    },
    {
      name: 'acceleratorMult inflates threshold (fewer gains)',
      input: defaultAmbrosia({
        time: 0,
        ambrosiaTimerG: 0.125,
        ambrosiaGenerationSpeed: 200,
        acceleratorMult: 3,
        lifetimeAmbrosia: 0
      })
    },
    {
      name: 'brickOfLeadMult inflates threshold',
      input: defaultAmbrosia({
        time: 0,
        ambrosiaTimerG: 0.125,
        ambrosiaGenerationSpeed: 200,
        brickOfLeadMult: 2,
        lifetimeAmbrosia: 0
      })
    },
    {
      name: 'big multi-bar tick — threshold ramps with lifetimeAmbrosia',
      input: defaultAmbrosia({
        time: 0,
        ambrosiaTimerG: 1.0, // 8 chunks
        ambrosiaGenerationSpeed: 100,
        ambrosiaLuck: 250, // mult=2, luckMult depends on RNG
        bonusAmbrosia: 1,
        lifetimeAmbrosia: 500,
        seed: 1000
      })
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newAdvanceAmbrosia(c.input)
      const oldR = oldAdvanceAmbrosia(c.input)
      expect(newR.ambrosiaTimerG).toBeCloseTo(oldR.ambrosiaTimerG, 10)
      expect(newR.blueberryTime).toBeCloseTo(oldR.blueberryTime, 10)
      expect(newR.ambrosia).toBe(oldR.ambrosia)
      expect(newR.lifetimeAmbrosia).toBe(oldR.lifetimeAmbrosia)
      expect(newR.seed).toBe(oldR.seed)
      expect(newR.events).toEqual(oldR.events)
    })
  }
})

// ─── advanceRedAmbrosiaTimer ───────────────────────────────────────────

const oldAdvanceRedAmbrosia = (input: AdvanceRedAmbrosiaTimerInput): AdvanceRedAmbrosiaTimerResult => {
  if (input.noAmbrosiaUpgradesCompletions <= 0) {
    return {
      redAmbrosiaTimerG: input.redAmbrosiaTimerG,
      redAmbrosiaTime: input.redAmbrosiaTime,
      redAmbrosia: input.redAmbrosia,
      lifetimeRedAmbrosia: input.lifetimeRedAmbrosia,
      seed: input.seed,
      bonusAmbrosiaTime: 0,
      events: []
    }
  }

  let redAmbrosiaTimerG = input.redAmbrosiaTimerG + input.time * input.timeMultiplier
  if (redAmbrosiaTimerG < 0.125) {
    return {
      redAmbrosiaTimerG,
      redAmbrosiaTime: input.redAmbrosiaTime,
      redAmbrosia: input.redAmbrosia,
      lifetimeRedAmbrosia: input.lifetimeRedAmbrosia,
      seed: input.seed,
      bonusAmbrosiaTime: 0,
      events: []
    }
  }

  let redAmbrosiaTime = input.redAmbrosiaTime
    + Math.floor(8 * redAmbrosiaTimerG) / 8 * input.redAmbrosiaGenerationSpeed
  redAmbrosiaTimerG %= 0.125

  let redAmbrosia = input.redAmbrosia
  let lifetimeRedAmbrosia = input.lifetimeRedAmbrosia
  let seed = input.seed
  let totalGained = 0
  let bonusAmbrosiaTime = 0

  let timeToRedAmbrosia = calculateRequiredRedAmbrosiaTime({
    timePerRedAmbrosia: input.timePerRedAmbrosia,
    lifetimeRedAmbrosia,
    barRequirementMultiplier: input.barRequirementMultiplier
  })

  while (redAmbrosiaTime >= timeToRedAmbrosia) {
    const rng = newSeededRandom(seed)
    seed = rng.newSeed
    const redAmbrosiaMult = Math.floor(input.redAmbrosiaLuck / 100)
    const luckMult = rng.value < input.redAmbrosiaLuck / 100 - Math.floor(input.redAmbrosiaLuck / 100) ? 1 : 0
    const redAmbrosiaToGain = redAmbrosiaMult + luckMult

    redAmbrosia += redAmbrosiaToGain
    lifetimeRedAmbrosia += redAmbrosiaToGain
    totalGained += redAmbrosiaToGain
    bonusAmbrosiaTime += redAmbrosiaToGain * input.ambrosiaTimePerRedAmbrosia
    redAmbrosiaTime -= timeToRedAmbrosia

    timeToRedAmbrosia = calculateRequiredRedAmbrosiaTime({
      timePerRedAmbrosia: input.timePerRedAmbrosia,
      lifetimeRedAmbrosia,
      barRequirementMultiplier: input.barRequirementMultiplier
    })
  }

  return {
    redAmbrosiaTimerG,
    redAmbrosiaTime,
    redAmbrosia,
    lifetimeRedAmbrosia,
    seed,
    bonusAmbrosiaTime,
    events: [{ kind: 'red-ambrosia-gained', amount: totalGained }]
  }
}

const defaultRedAmbrosia = (overrides: Partial<AdvanceRedAmbrosiaTimerInput> = {}): AdvanceRedAmbrosiaTimerInput => ({
  time: 0.025,
  timeMultiplier: 1,
  noAmbrosiaUpgradesCompletions: 1,
  redAmbrosiaGenerationSpeed: 1,
  redAmbrosiaTimerG: 0,
  redAmbrosiaTime: 0,
  redAmbrosia: 0,
  lifetimeRedAmbrosia: 0,
  seed: 100,
  redAmbrosiaLuck: 100,
  ambrosiaTimePerRedAmbrosia: 0,
  timePerRedAmbrosia: 100000,
  barRequirementMultiplier: 1,
  ...overrides
})

describe('parity: advanceRedAmbrosiaTimer', () => {
  const cases: Array<{ name: string, input: AdvanceRedAmbrosiaTimerInput }> = [
    {
      name: 'gate — completions=0 (feature locked)',
      input: defaultRedAmbrosia({ noAmbrosiaUpgradesCompletions: 0, redAmbrosiaTimerG: 0.5 })
    },
    {
      name: 'sub-1/8s timer accumulates',
      input: defaultRedAmbrosia({ time: 0.05, redAmbrosiaTimerG: 0.05 })
    },
    {
      name: 'gains 1 red ambrosia + bonus blueberry time',
      input: defaultRedAmbrosia({
        time: 0,
        redAmbrosiaTimerG: 0.125,
        redAmbrosiaGenerationSpeed: 1e7,
        redAmbrosiaTime: 0,
        ambrosiaTimePerRedAmbrosia: 30,
        redAmbrosiaLuck: 100
      })
    },
    {
      name: 'gains 1 red ambrosia with 0 ambrosiaTimePerRedAmbrosia (no bonus)',
      input: defaultRedAmbrosia({
        time: 0,
        redAmbrosiaTimerG: 0.125,
        redAmbrosiaGenerationSpeed: 1e7,
        redAmbrosiaTime: 0,
        ambrosiaTimePerRedAmbrosia: 0,
        redAmbrosiaLuck: 100
      })
    },
    {
      name: 'redAmbrosiaLuck = 175 (mult=1, fractional luck from RNG)',
      input: defaultRedAmbrosia({
        time: 0,
        redAmbrosiaTimerG: 0.125,
        redAmbrosiaGenerationSpeed: 1e7,
        redAmbrosiaLuck: 175,
        seed: 7,
        ambrosiaTimePerRedAmbrosia: 30
      })
    },
    {
      name: 'redAmbrosiaLuck = 300 (mult=3)',
      input: defaultRedAmbrosia({
        time: 0,
        redAmbrosiaTimerG: 0.125,
        redAmbrosiaGenerationSpeed: 1e7,
        redAmbrosiaLuck: 300,
        seed: 7,
        ambrosiaTimePerRedAmbrosia: 30
      })
    },
    {
      name: 'multi-bar tick — threshold ramps with lifetimeRedAmbrosia (linear ramp +200/each)',
      input: defaultRedAmbrosia({
        time: 0,
        redAmbrosiaTimerG: 0.5,
        redAmbrosiaGenerationSpeed: 1e7,
        redAmbrosiaTime: 0,
        ambrosiaTimePerRedAmbrosia: 60,
        redAmbrosiaLuck: 100,
        seed: 99,
        lifetimeRedAmbrosia: 0
      })
    },
    {
      name: 'barRequirementMultiplier inflates threshold',
      input: defaultRedAmbrosia({
        time: 0,
        redAmbrosiaTimerG: 0.125,
        redAmbrosiaGenerationSpeed: 5e5,
        barRequirementMultiplier: 2,
        redAmbrosiaLuck: 100
      })
    },
    {
      name: 'barRequirementMultiplier caps val to max',
      input: defaultRedAmbrosia({
        time: 0,
        redAmbrosiaTimerG: 0.125,
        redAmbrosiaGenerationSpeed: 1e10,
        lifetimeRedAmbrosia: 1e7, // would push val past cap
        barRequirementMultiplier: 1,
        redAmbrosiaLuck: 100
      })
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newAdvanceRedAmbrosia(c.input)
      const oldR = oldAdvanceRedAmbrosia(c.input)
      expect(newR.redAmbrosiaTimerG).toBeCloseTo(oldR.redAmbrosiaTimerG, 10)
      expect(newR.redAmbrosiaTime).toBeCloseTo(oldR.redAmbrosiaTime, 10)
      expect(newR.redAmbrosia).toBe(oldR.redAmbrosia)
      expect(newR.lifetimeRedAmbrosia).toBe(oldR.lifetimeRedAmbrosia)
      expect(newR.seed).toBe(oldR.seed)
      expect(newR.bonusAmbrosiaTime).toBeCloseTo(oldR.bonusAmbrosiaTime, 10)
      expect(newR.events).toEqual(oldR.events)
    })
  }
})
