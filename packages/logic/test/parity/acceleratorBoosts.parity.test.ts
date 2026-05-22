// Parity test for getAcceleratorBoostCost.
//
// Holds the migrated logic-package implementation byte-equal to the
// pre-migration version that lived in packages/web_ui/src/Buy.ts at commit
// 472937b0~1. The OLD function below is a pure transcription of that
// implementation — only `eff` is hoisted from `getRuneBlessingEffect('thrift')
// .accelBoostCostDelay` into an explicit parameter so the test can drive both.

import { describe, expect, it } from 'vitest'
import Decimal from 'break_infinity.js'
import { getAcceleratorBoostCost as newGetAcceleratorBoostCost } from '../../src/mechanics/acceleratorBoosts'

const linSumOld = (n: number) => n * (n + 1) / 2
const sqrSumOld = (n: number) => n * (n + 1) * (2 * n + 1) / 6

const oldGetAcceleratorBoostCost = (level: number, eff: number): Decimal => {
  // formula starts at 0 but buying starts at 1
  level--
  const buymax = Math.pow(10, 15)
  const base = new Decimal(1e3)

  let cost = base
  if (level > 1000 * eff) {
    cost = base.times(Decimal.pow(
      10,
      10 * level
        + linSumOld(level)
        + sqrSumOld(level - 1000 * eff) / eff
    ))
  } else {
    cost = base.times(Decimal.pow(10, 10 * level + linSumOld(level)))
  }
  if (level > buymax) {
    const diminishingExponent = 1 / 8

    // Old code's recursive call passed the 1-based caller level (`buymax`),
    // which the inner pre-decrement turned into the internal value buymax-1.
    const QuadrillionCost = oldGetAcceleratorBoostCost(buymax, eff)

    const newCost = QuadrillionCost.pow(Math.pow(level / buymax, 1 / diminishingExponent))
    const newExtra = newCost.exponent - Math.floor(newCost.exponent)
    newCost.exponent = Math.floor(newCost.exponent)
    newCost.mantissa *= Math.pow(10, newExtra)
    newCost.normalize()
    return Decimal.max(cost, newCost)
  }
  return cost
}

// Decimals from break_infinity store as mantissa * 10^exponent. Compare with a
// tight relative tolerance because both implementations go through the same
// renormalization but the in-place mutation order differs slightly.
const equalEnough = (a: Decimal, b: Decimal, rel = 1e-12): boolean => {
  if (a.eq(b)) return true
  if (a.abs().lt(1) && b.abs().lt(1)) {
    return a.minus(b).abs().lt(rel)
  }
  const diff = a.minus(b).abs()
  const scale = Decimal.max(a.abs(), b.abs())
  return diff.div(scale).lt(rel)
}

describe('parity: getAcceleratorBoostCost', () => {
  // Grid covers: below threshold, threshold transition, above threshold,
  // and the buymax dim branch. Two effs to exercise the threshold position.
  const levels = [1, 2, 3, 10, 50, 100, 500, 999, 1000, 1001, 1002, 1500, 2000, 5000, 10_000]
  const effs = [1, 2]

  for (const eff of effs) {
    describe(`accelBoostCostDelay=${eff}`, () => {
      it.each(levels)('level=%i matches old impl', (level) => {
        const oldCost = oldGetAcceleratorBoostCost(level, eff)
        const newCost = newGetAcceleratorBoostCost(level, { accelBoostCostDelay: eff })
        expect(equalEnough(oldCost, newCost)).toBe(true)
      })
    })
  }

  // Buymax dim branch — sample just inside and outside.
  it('matches across the buymax (1e15) dim branch', () => {
    const points = [1e15, 1.0001e15, 2e15]
    for (const lvl of points) {
      const oldCost = oldGetAcceleratorBoostCost(lvl, 1)
      const newCost = newGetAcceleratorBoostCost(lvl, { accelBoostCostDelay: 1 })
      expect(equalEnough(oldCost, newCost)).toBe(true)
    }
  })
})
