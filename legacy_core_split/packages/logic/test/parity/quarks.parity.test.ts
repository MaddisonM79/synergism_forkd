// Parity test for quarkHandler migrated from packages/web_ui/src/Quark.ts.
// The OLD function is transcribed below with the implicit player/octeract
// reads lifted into explicit parameters.

import { describe, expect, it } from 'vitest'
import {
  type QuarkHandlerInput,
  quarkHandler as newQuarkHandler
} from '../../src/mechanics/quarks'

const oldQuarkHandler = (input: QuarkHandlerInput) => {
  let maxTime = 90000
  if (input.research195 > 0) {
    maxTime += 18000 * input.research195
  }
  let baseQuarkPerHour = 5
  // The old loop summed researches[99/100/125/180/195]; the migration takes
  // that sum precomputed.
  baseQuarkPerHour += input.researchesSum
  baseQuarkPerHour *= input.exportQuarkMult
  const quarkPerHour = baseQuarkPerHour
  const capacity = Math.floor(quarkPerHour * maxTime / 3600)
  const quarkGain = Math.floor(input.quarksTimer * quarkPerHour / 3600)
  return {
    maxTime,
    perHour: quarkPerHour,
    capacity,
    gain: quarkGain,
    cubeMult: input.cubeMult
  }
}

describe('parity: quarkHandler', () => {
  const cases: QuarkHandlerInput[] = [
    // Defaults: no researches, no octeract, no timer
    {
      research195: 0,
      researchesSum: 0,
      exportQuarkMult: 1,
      quarksTimer: 0,
      cubeMult: 1
    },
    // research195 extends maxTime
    {
      research195: 5,
      researchesSum: 0,
      exportQuarkMult: 1,
      quarksTimer: 0,
      cubeMult: 1
    },
    // researchesSum bumps base rate; quarksTimer mid-window
    {
      research195: 0,
      researchesSum: 10,
      exportQuarkMult: 1,
      quarksTimer: 45000,
      cubeMult: 1
    },
    // Octeract multiplier active
    {
      research195: 0,
      researchesSum: 0,
      exportQuarkMult: 2.5,
      quarksTimer: 90000,
      cubeMult: 1
    },
    // All bumps + timer at full capacity
    {
      research195: 10,
      researchesSum: 50,
      exportQuarkMult: 3,
      quarksTimer: 90000 + 18000 * 10,
      cubeMult: 1.5
    },
    // Timer way past capacity — gain keeps scaling (clamping is web_ui's job)
    {
      research195: 0,
      researchesSum: 0,
      exportQuarkMult: 1,
      quarksTimer: 1e6,
      cubeMult: 7
    },
    // Floor truncation: rates that produce non-integer perHour
    {
      research195: 0,
      researchesSum: 0,
      exportQuarkMult: 1.337,
      quarksTimer: 3600,
      cubeMult: 1
    }
  ]

  for (const [i, input] of cases.entries()) {
    it(`case ${i}`, () => {
      const newRes = newQuarkHandler(input)
      const oldRes = oldQuarkHandler(input)
      expect(newRes).toEqual(oldRes)
    })
  }

  it('research195 = 0 leaves maxTime at the 90000 baseline', () => {
    expect(newQuarkHandler({
      research195: 0,
      researchesSum: 0,
      exportQuarkMult: 1,
      quarksTimer: 0,
      cubeMult: 1
    }).maxTime).toBe(90000)
  })

  it('cubeMult passes through unchanged', () => {
    const res = newQuarkHandler({
      research195: 0,
      researchesSum: 0,
      exportQuarkMult: 1,
      quarksTimer: 0,
      cubeMult: 42.5
    })
    expect(res.cubeMult).toBe(42.5)
  })

  it('capacity == perHour * maxTime / 3600 (floored)', () => {
    const res = newQuarkHandler({
      research195: 2, // maxTime = 90000 + 36000 = 126000
      researchesSum: 5, // perHour = (5+5) * 2 = 20
      exportQuarkMult: 2,
      quarksTimer: 0,
      cubeMult: 1
    })
    expect(res.maxTime).toBe(126000)
    expect(res.perHour).toBe(20)
    expect(res.capacity).toBe(Math.floor(20 * 126000 / 3600))
  })
})
