import { describe, expect, it } from 'vitest'
import { Decimal } from '../src/math/bignum'
import {
  getProducerCost,
  type GetProducerCostInput,
  type ProducerIndex,
  type ProducerType
} from '../src/mechanics/producers'

const baseInput: GetProducerCostInput = {
  costDivisor: 1,
  inTranscensionChallenge4: false,
  inReincarnationChallenge8: false,
  inReincarnationChallenge10: false,
  challengecompletions4: 0,
  challengecompletions8: 0
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

const allTypes: ProducerType[] = ['Coin', 'Diamonds', 'Mythos', 'Particles']
const allIndices: ProducerIndex[] = [1, 2, 3, 4, 5]

describe('getProducerCost', () => {
  describe('anchor values per type at index 1, buyingTo=1', () => {
    // First purchase at buyingTo=1 decrements to 0 internally — the formula
    // collapses to `originalCost * 1.25^0 = originalCost`. (The +1
    // correction below the precision threshold adds 0/10^exp = 0.)
    it('Coin index 1 starts at 100', () => {
      closeEnough(getProducerCost(1, 'Coin', 1, baseInput), new Decimal(100))
    })
    it('Diamonds index 1 starts at 100', () => {
      closeEnough(getProducerCost(1, 'Diamonds', 1, baseInput), new Decimal(100))
    })
    it('Mythos index 1 starts at 1', () => {
      closeEnough(getProducerCost(1, 'Mythos', 1, baseInput), new Decimal(1))
    })
    it('Particles index 1 starts at 1', () => {
      closeEnough(getProducerCost(1, 'Particles', 1, baseInput), new Decimal(1))
    })
  })

  describe('anchor values: index 2, 5 cover the cost array', () => {
    it('Coin index 2 starts at 1000', () => {
      closeEnough(getProducerCost(2, 'Coin', 1, baseInput), new Decimal(1000))
    })
    it('Coin index 5 starts at 8e6', () => {
      closeEnough(getProducerCost(5, 'Coin', 1, baseInput), new Decimal(8e6))
    })
    it('Diamonds index 3 starts at 1e15', () => {
      closeEnough(getProducerCost(3, 'Diamonds', 1, baseInput), new Decimal(1e15))
    })
    it('Mythos index 5 starts at 1e16', () => {
      closeEnough(getProducerCost(5, 'Mythos', 1, baseInput), new Decimal(1e16))
    })
  })

  describe('monotonicity across the small/mid range, all types', () => {
    const samples = [1, 2, 5, 10, 50, 100, 500, 999]
    for (const type of allTypes) {
      for (const index of allIndices) {
        it(`${type} index ${index}`, () => {
          let prev = new Decimal(0)
          for (const n of samples) {
            const cost = getProducerCost(index, type, n, baseInput)
            expect(cost.gte(prev)).toBe(true)
            prev = cost
          }
        })
      }
    }
  })

  describe('challenge 4 (transcension) amplifies Coin and Diamonds', () => {
    it('Coin cost above the kick-in threshold rises substantially', () => {
      // Threshold = max(1000 - 10*completions, 0). With completions=0 it's
      // exactly 1000; sample comfortably above to land in the mlog10125 bump.
      const without = getProducerCost(1, 'Coin', 1100, baseInput)
      const withC4 = getProducerCost(1, 'Coin', 1100, { ...baseInput, inTranscensionChallenge4: true })
      expect(withC4.gte(without)).toBe(true)
      expect(withC4.div(without).gt(2)).toBe(true)
    })
    it('Mythos is unaffected by challenge 4', () => {
      const without = getProducerCost(1, 'Mythos', 1100, baseInput)
      const withC4 = getProducerCost(1, 'Mythos', 1100, { ...baseInput, inTranscensionChallenge4: true })
      closeEnough(withC4, without)
    })
  })

  describe('challenge 8 (reincarnation) amplifies Coin / Diamonds / Mythos', () => {
    it('Coin with completions=1 above threshold is heavier', () => {
      const without = getProducerCost(1, 'Coin', 2000, baseInput)
      const withC8 = getProducerCost(1, 'Coin', 2000, {
        ...baseInput,
        inReincarnationChallenge8: true,
        challengecompletions8: 1
      })
      expect(withC8.gte(without)).toBe(true)
    })
    it('Particles is unaffected by challenge 8', () => {
      const without = getProducerCost(1, 'Particles', 2000, baseInput)
      const withC8 = getProducerCost(1, 'Particles', 2000, {
        ...baseInput,
        inReincarnationChallenge8: true,
        challengecompletions8: 1
      })
      closeEnough(withC8, without)
    })
  })

  describe('challenge 10 (reincarnation) amplifies Coin and Diamonds', () => {
    it('Diamonds above the r*25000 threshold rises', () => {
      const buyingTo = 30000
      const without = getProducerCost(1, 'Diamonds', buyingTo, baseInput)
      const withC10 = getProducerCost(1, 'Diamonds', buyingTo, {
        ...baseInput,
        inReincarnationChallenge10: true
      })
      expect(withC10.gte(without)).toBe(true)
    })
    it('Mythos is unaffected by challenge 10', () => {
      const without = getProducerCost(1, 'Mythos', 30000, baseInput)
      const withC10 = getProducerCost(1, 'Mythos', 30000, {
        ...baseInput,
        inReincarnationChallenge10: true
      })
      closeEnough(withC10, without)
    })
  })

  describe('costDivisor (r) shifts the bracket thresholds', () => {
    it('r=2 raises the first factorial bracket from 1000 to 2000', () => {
      const before = getProducerCost(1, 'Coin', 1500, { ...baseInput, costDivisor: 2 })
      const after = getProducerCost(1, 'Coin', 1500, baseInput)
      // With r=1 the 1500 sample is past the r*1000 bracket; with r=2 it isn't.
      expect(after.gte(before)).toBe(true)
    })
  })

  describe('buymax (1e15) diminishing branch', () => {
    it('returns finite Decimal at the breakpoint for Coin index 1', () => {
      const cost = getProducerCost(1, 'Coin', 1e15, baseInput)
      expect(Number.isFinite(cost.mantissa)).toBe(true)
      expect(Number.isFinite(cost.exponent)).toBe(true)
    })
    it('monotone across the buymax boundary', () => {
      const below = getProducerCost(1, 'Coin', 1e15, baseInput)
      const above = getProducerCost(1, 'Coin', 1.0001e15, baseInput)
      expect(above.gte(below)).toBe(true)
    })
  })

  describe('input immutability', () => {
    it('does not mutate the input object', () => {
      const input = { ...baseInput }
      const snapshot = JSON.stringify(input)
      getProducerCost(3, 'Coin', 1500, input)
      expect(JSON.stringify(input)).toBe(snapshot)
    })
  })
})
