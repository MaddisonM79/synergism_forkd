// Parity tests for the ascension-count calculation lifted from
// packages/web_ui/src/Calculate.ts. Sweeps cover both gate states and a
// variety of multiplier arrays (including ones whose product is <1 to
// exercise the `Math.floor` rounding boundary).

import { describe, expect, it } from 'vitest'
import { calculateAscensionCount as newAscCount } from '../../src/mechanics/ascensions'

const oldAscCount = (limitedAscensionsEnabled: boolean, mults: number[]): number => {
  if (limitedAscensionsEnabled) {
    return 1
  }
  return Math.floor(mults.reduce((a, b) => a * b, 1))
}

const cases: [boolean, number[]][] = [
  [false, []],
  [false, [1]],
  [false, [1, 1, 1]],
  [false, [2]],
  [false, [2, 3]],
  [false, [1.5, 1.5, 2]],
  [false, [0.5, 1, 1]],
  [false, [0.99, 1, 1]],
  [false, [10, 10, 10]],
  [true, [10, 10, 10]],
  [true, []],
  [true, [0.0001]]
]

describe('parity: calculateAscensionCount', () => {
  it.each(cases)('enabled=%s mults=%j', (enabled, mults) => {
    expect(newAscCount({ limitedAscensionsEnabled: enabled, ascensionCountMults: mults }))
      .toBe(oldAscCount(enabled, mults))
  })
})
