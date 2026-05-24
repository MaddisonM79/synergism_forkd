// Parity tests for the octeracts addTimers case.
// Old body transcribed verbatim from packages/web_ui/src/Helper.ts
// (addTimers, `octeracts` switch case pre-migration). The legacy code
// reads `calculateGoldenQuarks()` inside the giveaway loop, which is
// equivalent to `calculateBaseGoldenQuarks(qts) * product(stats[1..])`;
// the parity oracle here computes that the same way the migrated logic
// does, since the equivalence is provable from the stat list (only
// stat 0 / `calculateBaseGoldenQuarks` depends on quarksThisSingularity,
// which is the only loop-mutated input).

import { describe, expect, it } from 'vitest'
import { calculateBaseGoldenQuarks } from '../../src/mechanics/singularityMilestones'
import {
  advanceOcteractTimer as newAdvanceOcteract,
  type AdvanceOcteractTimerInput,
  type AdvanceOcteractTimerResult,
  OCTERACT_GIVEAWAY_LEVELS
} from '../../src/tick/timers'

// Verbatim transcription mirroring the legacy switch case body.
const oldAdvanceOcteract = (input: AdvanceOcteractTimerInput): AdvanceOcteractTimerResult => {
  if (!input.octeractUnlocked) {
    return {
      octeractTimer: input.octeractTimer,
      wowOcteracts: input.wowOcteracts,
      totalWowOcteracts: input.totalWowOcteracts,
      goldenQuarks: input.goldenQuarks,
      quarksThisSingularity: input.quarksThisSingularity,
      events: []
    }
  }
  let octeractTimer = input.octeractTimer + input.time * input.timeMultiplier
  if (octeractTimer < 1) {
    return {
      octeractTimer,
      wowOcteracts: input.wowOcteracts,
      totalWowOcteracts: input.totalWowOcteracts,
      goldenQuarks: input.goldenQuarks,
      quarksThisSingularity: input.quarksThisSingularity,
      events: []
    }
  }
  const amountOfGiveaways = octeractTimer - (octeractTimer % 1)
  octeractTimer %= 1

  const wowOcteracts = input.wowOcteracts + amountOfGiveaways * input.perSecond
  const totalWowOcteracts = input.totalWowOcteracts + amountOfGiveaways * input.perSecond

  let goldenQuarks = input.goldenQuarks
  let qts = input.quarksThisSingularity
  if (input.highestSingularityCount >= 160) {
    const frac = 1e-6
    let actualLevel = 0
    for (const level of OCTERACT_GIVEAWAY_LEVELS) {
      if (input.highestSingularityCount >= level) actualLevel += 1
    }
    const quarkFraction = frac * actualLevel
    for (let i = 0; i < amountOfGiveaways; i++) {
      const base = calculateBaseGoldenQuarks({
        singularity: input.singularityCount,
        quarksThisSingularity: qts,
        highestSingularityCount: input.highestSingularityCount
      })
      goldenQuarks += quarkFraction * base * input.goldenQuarksMultiplierExcludingBase
      qts *= 1 - quarkFraction
    }
  }
  return {
    octeractTimer,
    wowOcteracts,
    totalWowOcteracts,
    goldenQuarks,
    quarksThisSingularity: qts,
    events: [{ kind: 'octeract-tick-fired', amountOfGiveaways }]
  }
}

const defaultInput = (overrides: Partial<AdvanceOcteractTimerInput> = {}): AdvanceOcteractTimerInput => ({
  time: 0.025,
  timeMultiplier: 1,
  octeractUnlocked: true,
  octeractTimer: 0,
  wowOcteracts: 0,
  totalWowOcteracts: 0,
  goldenQuarks: 0,
  quarksThisSingularity: 0,
  perSecond: 1,
  highestSingularityCount: 0,
  singularityCount: 0,
  goldenQuarksMultiplierExcludingBase: 1,
  ...overrides
})

describe('parity: advanceOcteractTimer', () => {
  const cases: Array<{ name: string, input: AdvanceOcteractTimerInput }> = [
    {
      name: 'gate — octeractUnlocked=false (no state change, no event)',
      input: defaultInput({ octeractUnlocked: false, octeractTimer: 0.5, time: 0.5 })
    },
    {
      name: 'sub-second timer accumulates only',
      input: defaultInput({ octeractTimer: 0.1, time: 0.5 })
    },
    {
      name: 'crosses 1s — 1 giveaway, no GQ block (highestSing < 160)',
      input: defaultInput({
        octeractTimer: 0.9,
        time: 0.2,
        perSecond: 50,
        highestSingularityCount: 100
      })
    },
    {
      name: 'crosses multiple seconds at once (large dt)',
      input: defaultInput({
        octeractTimer: 0,
        time: 5.7,
        timeMultiplier: 1,
        perSecond: 10,
        highestSingularityCount: 50
      })
    },
    {
      name: 'fractional remainder carries to next tick',
      input: defaultInput({
        octeractTimer: 0.75,
        time: 1.5,
        perSecond: 100
      })
    },
    {
      name: 'GQ block — sing=160 (actualLevel=1, single iteration)',
      input: defaultInput({
        octeractTimer: 0,
        time: 1.5, // → 1 giveaway
        perSecond: 100,
        highestSingularityCount: 160,
        singularityCount: 160,
        quarksThisSingularity: 1e8,
        goldenQuarks: 1000,
        goldenQuarksMultiplierExcludingBase: 5
      })
    },
    {
      name: 'GQ block — sing=210 (actualLevel=6, larger quarkFraction)',
      input: defaultInput({
        octeractTimer: 0,
        time: 1.1,
        perSecond: 1000,
        highestSingularityCount: 210,
        singularityCount: 210,
        quarksThisSingularity: 1e10,
        goldenQuarks: 10000,
        goldenQuarksMultiplierExcludingBase: 50
      })
    },
    {
      name: 'GQ block — sing=249 (actualLevel=10, max scaling)',
      input: defaultInput({
        octeractTimer: 0,
        time: 1.1,
        perSecond: 1e6,
        highestSingularityCount: 249,
        singularityCount: 249,
        quarksThisSingularity: 1e12,
        goldenQuarks: 1e6,
        goldenQuarksMultiplierExcludingBase: 100
      })
    },
    {
      name: 'GQ block — multi-iteration loop (qts decays geometrically)',
      input: defaultInput({
        octeractTimer: 0,
        time: 5.5, // → 5 giveaways
        perSecond: 100,
        highestSingularityCount: 200,
        singularityCount: 200,
        quarksThisSingularity: 1e9,
        goldenQuarks: 5000,
        goldenQuarksMultiplierExcludingBase: 20
      })
    },
    {
      name: 'GQ block — sing=159 (no GQ block, just octeracts)',
      input: defaultInput({
        octeractTimer: 0,
        time: 1.1,
        perSecond: 100,
        highestSingularityCount: 159, // strictly less than 160
        singularityCount: 159,
        quarksThisSingularity: 1e8
      })
    },
    {
      name: 'GQ block — sing=160 exactly (boundary)',
      input: defaultInput({
        octeractTimer: 0,
        time: 1.1,
        perSecond: 100,
        highestSingularityCount: 160,
        singularityCount: 160,
        quarksThisSingularity: 1e8,
        goldenQuarksMultiplierExcludingBase: 10
      })
    },
    {
      name: 'large multi-iter (10 giveaways) — qts decay accumulates',
      input: defaultInput({
        octeractTimer: 0.5,
        time: 10,
        perSecond: 5,
        highestSingularityCount: 230,
        singularityCount: 230,
        quarksThisSingularity: 1e10,
        goldenQuarksMultiplierExcludingBase: 30
      })
    },
    {
      name: 'qts=0 (degenerate but valid — base formula still produces a value)',
      input: defaultInput({
        octeractTimer: 0,
        time: 1.1,
        perSecond: 50,
        highestSingularityCount: 200,
        singularityCount: 200,
        quarksThisSingularity: 0,
        goldenQuarksMultiplierExcludingBase: 10
      })
    }
  ]

  for (const c of cases) {
    it(c.name, () => {
      const newR = newAdvanceOcteract(c.input)
      const oldR = oldAdvanceOcteract(c.input)
      expect(newR.octeractTimer).toBeCloseTo(oldR.octeractTimer, 10)
      expect(newR.wowOcteracts).toBe(oldR.wowOcteracts)
      expect(newR.totalWowOcteracts).toBe(oldR.totalWowOcteracts)
      expect(newR.goldenQuarks).toBeCloseTo(oldR.goldenQuarks, 6)
      expect(newR.quarksThisSingularity).toBeCloseTo(oldR.quarksThisSingularity, 6)
      expect(newR.events).toEqual(oldR.events)
    })
  }
})
