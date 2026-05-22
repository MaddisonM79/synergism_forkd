import { describe, expect, it } from 'vitest'
import { Decimal } from '../src/math/bignum'
import {
  buyMultiplier,
  getCostMultiplier,
  type BuyMultiplierInput,
  type GetCostMultiplierInput
} from '../src/mechanics/multipliers'
import type { MultiplierState } from '../src/state/schema'

const baseInput: GetCostMultiplierInput = {
  costDivisor: 1,
  transcendECC: 0,
  inTranscensionChallenge4: false,
  inReincarnationChallenge8: false
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

describe('getCostMultiplier', () => {
  describe('default input', () => {
    it('returns 1e4 for the first multiplier', () => {
      closeEnough(getCostMultiplier(1, baseInput), new Decimal(1e4))
    })

    it('returns 1e5 for the second (multiplied by 10)', () => {
      closeEnough(getCostMultiplier(2, baseInput), new Decimal(1e5))
    })

    it('is monotonically non-decreasing', () => {
      const samples = [1, 2, 5, 10, 50, 75, 76, 100, 1000, 2000, 2001, 5000]
      let prev = new Decimal(0)
      for (const n of samples) {
        const cost = getCostMultiplier(n, baseInput)
        expect(cost.gte(prev)).toBe(true)
        prev = cost
      }
    })

    it('applies the factorial branch above 75', () => {
      const below = getCostMultiplier(75, baseInput)
      const justAbove = getCostMultiplier(77, baseInput)
      // Factorial branch multiplies by num! * 10^num — well above the
      // smooth 10x geometric trend (two steps would be 100x).
      const geometric = below.times(100)
      expect(justAbove.gt(geometric)).toBe(true)
    })

    it('applies the sum-of-arithmetic branch above 2000', () => {
      const below = getCostMultiplier(2000, baseInput)
      const justAbove = getCostMultiplier(2002, baseInput)
      // Above 2000 we additionally multiply by 2^(sumNum*(sumNum+1)/2).
      // For sumNum=2 → 2^3 = 8.
      expect(justAbove.div(below).gt(8)).toBe(true)
    })
  })

  describe('challenge multipliers', () => {
    it('challenge 4 amplifies by 10^(n(n+1)/2)', () => {
      const at = (n: number, chal: boolean) =>
        getCostMultiplier(n, { ...baseInput, inTranscensionChallenge4: chal })
      // For buyingTo=2 internally is 1, sumBit=1, factor 10.
      closeEnough(at(2, true).div(at(2, false)), new Decimal(10))
    })

    it('challenge 8 amplifies by 1e50^(n(n+1)/2)', () => {
      const at = (n: number, chal: boolean) =>
        getCostMultiplier(n, { ...baseInput, inReincarnationChallenge8: chal })
      closeEnough(at(2, true).div(at(2, false)), new Decimal(1e50))
    })
  })

  describe('transcend ECC raises the factorial/sum thresholds', () => {
    it('with transcendECC=10 the factorial branch starts at 95 instead of 75', () => {
      // 75 + 2*10 = 95. At buyingTo=96 → internal 95, not >95. At 97 → internal 96.
      const justBefore = getCostMultiplier(96, { ...baseInput, transcendECC: 10 })
      const justAfter = getCostMultiplier(97, { ...baseInput, transcendECC: 10 })
      // Factorial-branch jump is large; require ratio > smooth 10x step.
      expect(justAfter.div(justBefore).gt(10)).toBe(true)
    })
  })

  describe('buymax (1e15) diminishing branch', () => {
    it('returns a finite Decimal at the breakpoint', () => {
      const cost = getCostMultiplier(1e15, baseInput)
      expect(Number.isFinite(cost.mantissa)).toBe(true)
      expect(Number.isFinite(cost.exponent)).toBe(true)
    })

    it('still monotone across the buymax boundary', () => {
      const below = getCostMultiplier(1e15, baseInput)
      const above = getCostMultiplier(1.0001e15, baseInput)
      expect(above.gte(below)).toBe(true)
    })
  })

  describe('input immutability', () => {
    it('does not mutate the input object', () => {
      const input = { ...baseInput }
      const snapshot = JSON.stringify(input)
      getCostMultiplier(100, input)
      expect(JSON.stringify(input)).toBe(snapshot)
    })
  })
})

const baseBuyInput: BuyMultiplierInput = {
  autobuyer: false,
  coinbuyamount: 100,
  costDivisor: 1,
  transcendECC: 0,
  inTranscensionChallenge4: false,
  inReincarnationChallenge8: false
}

const makeState = (overrides: Partial<MultiplierState> = {}): MultiplierState => ({
  multiplierBought: 0,
  multiplierCost: new Decimal(1e4),
  coins: new Decimal(0),
  prestigenomultiplier: true,
  transcendnomultiplier: true,
  reincarnatenomultiplier: true,
  ...overrides
})

describe('buyMultiplier', () => {
  it('is a no-op when coins are zero', () => {
    const state = makeState()
    const { state: next, events } = buyMultiplier(state, baseBuyInput)
    expect(next.multiplierBought).toBe(0)
    expect(next.coins.eq(state.coins)).toBe(true)
    expect(events).toEqual([])
  })

  it('purchases at least one multiplier given sufficient coins', () => {
    // First multiplier costs 1e4; give enough to buy several.
    const state = makeState({ coins: new Decimal(1e10) })
    const { state: next, events } = buyMultiplier(state, baseBuyInput)
    expect(next.multiplierBought).toBeGreaterThan(0)
    expect(next.coins.lt(state.coins)).toBe(true)
    expect(events).toHaveLength(1)
    expect(events[0]?.kind).toBe('multipliers-purchased')
  })

  it('respects coinbuyamount cap when not in autobuyer mode', () => {
    const state = makeState({ coins: new Decimal(1e30) })
    const { state: next } = buyMultiplier(state, { ...baseBuyInput, coinbuyamount: 10 })
    expect(next.multiplierBought).toBeLessThanOrEqual(10)
  })

  it('bypasses the coinbuyamount cap in autobuyer mode', () => {
    const state = makeState({ coins: new Decimal(1e30) })
    const noAuto = buyMultiplier(state, { ...baseBuyInput, autobuyer: false, coinbuyamount: 10 })
    const yesAuto = buyMultiplier(state, { ...baseBuyInput, autobuyer: true, coinbuyamount: 10 })
    expect(yesAuto.state.multiplierBought).toBeGreaterThan(noAuto.state.multiplierBought)
  })

  it('flips the three no-multiplier flags to false on first purchase', () => {
    const state = makeState({ coins: new Decimal(1e10) })
    const { state: next } = buyMultiplier(state, baseBuyInput)
    expect(next.prestigenomultiplier).toBe(false)
    expect(next.transcendnomultiplier).toBe(false)
    expect(next.reincarnatenomultiplier).toBe(false)
  })

  it('leaves the no-multiplier flags untouched when nothing was bought', () => {
    const state = makeState({ coins: new Decimal(0) })
    const { state: next } = buyMultiplier(state, baseBuyInput)
    expect(next.prestigenomultiplier).toBe(true)
    expect(next.transcendnomultiplier).toBe(true)
    expect(next.reincarnatenomultiplier).toBe(true)
  })

  it('emits an event whose `spent` matches the coin delta', () => {
    const state = makeState({ coins: new Decimal(1e10) })
    const { state: next, events } = buyMultiplier(state, baseBuyInput)
    const spent = state.coins.sub(next.coins)
    expect(events[0]?.kind).toBe('multipliers-purchased')
    if (events[0]?.kind === 'multipliers-purchased') {
      expect(events[0].spent.eq(spent)).toBe(true)
      expect(events[0].before).toBe(0)
      expect(events[0].after).toBe(next.multiplierBought)
    }
  })

  it('does not mutate the input state', () => {
    const state = makeState({ coins: new Decimal(1e10) })
    const snapshot = {
      multiplierBought: state.multiplierBought,
      multiplierCostMantissa: state.multiplierCost.mantissa,
      multiplierCostExponent: state.multiplierCost.exponent,
      coinsMantissa: state.coins.mantissa,
      coinsExponent: state.coins.exponent,
      prestigenomultiplier: state.prestigenomultiplier
    }
    buyMultiplier(state, baseBuyInput)
    expect(state.multiplierBought).toBe(snapshot.multiplierBought)
    expect(state.multiplierCost.mantissa).toBe(snapshot.multiplierCostMantissa)
    expect(state.multiplierCost.exponent).toBe(snapshot.multiplierCostExponent)
    expect(state.coins.mantissa).toBe(snapshot.coinsMantissa)
    expect(state.coins.exponent).toBe(snapshot.coinsExponent)
    expect(state.prestigenomultiplier).toBe(snapshot.prestigenomultiplier)
  })
})
