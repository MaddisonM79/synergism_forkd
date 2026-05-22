// Parity test for CalcECC.
//
// Pre-migration source: packages/web_ui/src/Challenges.ts at HEAD. The OLD
// function is verbatim — it takes only (type, completions) and has no
// external dependencies, so the parity transcription is identical to the
// new implementation modulo file location.

import { describe, expect, it } from 'vitest'
import { CalcECC as newCalcECC } from '../../src/mechanics/challenges'

type Type = 'transcend' | 'reincarnation' | 'ascension'

const oldCalcECC = (type: Type, completions: number): number => {
  let effective = 0
  if (type === 'transcend') {
    effective += Math.min(100, completions)
    effective += 1 / 20 * (Math.min(1000, Math.max(100, completions)) - 100)
    effective += 1 / 100 * (Math.max(1000, completions) - 1000)
    return effective
  }
  if (type === 'reincarnation') {
    effective += Math.min(25, completions)
    effective += 1 / 2 * (Math.min(75, Math.max(25, completions)) - 25)
    effective += 1 / 10 * (Math.max(75, completions) - 75)
    return effective
  }
  // ascension
  effective += Math.min(10, completions)
  effective += 1 / 2 * (Math.max(10, completions) - 10)
  return effective
}

describe('parity: CalcECC', () => {
  const types: Type[] = ['transcend', 'reincarnation', 'ascension']
  // Sample across each piecewise segment for every type, including
  // boundary values (10/25/75/100/1000) and well past the last knee.
  const completionsGrid = [0, 1, 5, 9, 10, 11, 24, 25, 26, 50, 74, 75, 76, 99, 100, 101, 500, 999, 1000, 1001, 5000, 100000]

  for (const type of types) {
    describe(type, () => {
      it.each(completionsGrid)('completions=%i', (completions) => {
        expect(newCalcECC(type, completions)).toBe(oldCalcECC(type, completions))
      })
    })
  }
})
