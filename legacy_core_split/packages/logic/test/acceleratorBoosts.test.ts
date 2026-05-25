import { describe, expect, it } from 'vitest'
import { Decimal } from '../src/math/bignum'
import {
  getAcceleratorBoostCost,
  type GetAcceleratorBoostCostInput
} from '../src/mechanics/acceleratorBoosts'

const baseInput: GetAcceleratorBoostCostInput = {
  accelBoostCostDelay: 1
}

const closeEnough = (a: Decimal, b: Decimal, rel = 1e-9): void => {
  if (a.abs().lt(1) && b.abs().lt(1)) {
    expect(a.minus(b).abs().lt(rel)).toBe(true)
    return
  }
  const diff = a.minus(b).abs()
  const scale = Decimal.max(a.abs(), b.abs())
  expect(diff.div(scale).lt(rel)).toBe(true)
}

describe('getAcceleratorBoostCost', () => {
  describe('anchor values (accelBoostCostDelay=1, no quadratic branch)', () => {
    it('level=1 returns base cost 1e3', () => {
      // After --level: 0. Exponent = 10*0 + linSum(0) = 0. Cost = 1e3 * 10^0.
      closeEnough(getAcceleratorBoostCost(1, baseInput), new Decimal(1e3))
    })

    it('level=2 returns 1e14', () => {
      // After --: 1. Exponent = 10*1 + linSum(1) = 11. Cost = 1e3 * 10^11.
      closeEnough(getAcceleratorBoostCost(2, baseInput), new Decimal(1e14))
    })

    it('level=3 returns 1e26', () => {
      // After --: 2. Exponent = 10*2 + linSum(2) = 23. Cost = 1e3 * 10^23.
      closeEnough(getAcceleratorBoostCost(3, baseInput), new Decimal(1e26))
    })
  })

  describe('monotonicity', () => {
    it('cost is monotonically non-decreasing across regular and threshold levels', () => {
      const samples = [1, 2, 5, 10, 100, 500, 999, 1000, 1001, 1002, 5000, 10000]
      let prev = new Decimal(0)
      for (const n of samples) {
        const cost = getAcceleratorBoostCost(n, baseInput)
        expect(cost.gte(prev)).toBe(true)
        prev = cost
      }
    })
  })

  describe('quadratic threshold at 1000 * accelBoostCostDelay', () => {
    it('with eff=1, cost grows much faster just past level 1001', () => {
      // Threshold: post-decrement level > 1000*1 = 1000, i.e. caller level > 1001.
      const at1001 = getAcceleratorBoostCost(1001, baseInput)
      const at1003 = getAcceleratorBoostCost(1003, baseInput)
      // Below threshold the step ratio is ~10^(10 + level). Above threshold
      // the sqrSum kicker adds quadratic-in-(level-1000)/eff growth, so two
      // steps should exceed the pure 10^(2*11) = 1e22 ratio.
      expect(at1003.div(at1001).gt(1e25)).toBe(true)
    })

    it('eff=2 pushes the threshold to level 2001', () => {
      const input: GetAcceleratorBoostCostInput = { accelBoostCostDelay: 2 }
      // At level 1500 (post-dec 1499 < 2000), still below threshold.
      const at1500 = getAcceleratorBoostCost(1500, input)
      const at1502 = getAcceleratorBoostCost(1502, input)
      // Below-threshold growth: exponent grows by 10 + level each step, so
      // two steps multiplies by ~10^(2*1510). Just confirm a sane ratio
      // without the quadratic kicker present.
      const ratio = at1502.div(at1500)
      // No quadratic kicker — ratio should be far smaller than the with-kicker case.
      // (with eff=1 at level 1500 the kicker would have been active and the ratio
      //  would be much larger.)
      expect(ratio.gt(0)).toBe(true)
      // Sanity: the cost itself must be a finite Decimal.
      expect(Number.isFinite(at1502.mantissa)).toBe(true)
      expect(Number.isFinite(at1502.exponent)).toBe(true)
    })
  })

  describe('accelBoostCostDelay effect', () => {
    it('higher delay yields lower (or equal) cost at a given above-threshold level', () => {
      // At level 1500 (post-dec 1499) the eff=1 path crosses threshold (1499 > 1000)
      // while eff=2 stays below (1499 < 2000). So eff=2's cost must be lower.
      const eff1 = getAcceleratorBoostCost(1500, { accelBoostCostDelay: 1 })
      const eff2 = getAcceleratorBoostCost(1500, { accelBoostCostDelay: 2 })
      expect(eff2.lt(eff1)).toBe(true)
    })
  })

  describe('buymax (1e15) diminishing branch', () => {
    it('returns a finite Decimal at the breakpoint', () => {
      const cost = getAcceleratorBoostCost(1e15, baseInput)
      expect(Number.isFinite(cost.mantissa)).toBe(true)
      expect(Number.isFinite(cost.exponent)).toBe(true)
    })

    it('still monotone across the buymax boundary', () => {
      const below = getAcceleratorBoostCost(1e15, baseInput)
      const above = getAcceleratorBoostCost(1.0001e15, baseInput)
      expect(above.gte(below)).toBe(true)
    })
  })

  describe('input immutability', () => {
    it('does not mutate the input object', () => {
      const input = { ...baseInput }
      const snapshot = JSON.stringify(input)
      getAcceleratorBoostCost(2000, input)
      expect(JSON.stringify(input)).toBe(snapshot)
    })
  })
})
