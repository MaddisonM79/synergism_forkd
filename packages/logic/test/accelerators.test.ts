import { describe, expect, it } from 'vitest'
import { Decimal } from '../src/math/bignum'
import {
  buyAccelerator,
  getCostAccelerator,
  type BuyAcceleratorInput,
  type GetCostAcceleratorInput
} from '../src/mechanics/accelerators'
import type { AcceleratorState } from '../src/state/schema'

const baseInput: GetCostAcceleratorInput = {
  costDivisor: 1,
  transcendECC: 0,
  inTranscensionChallenge4: false,
  inReincarnationChallenge8: false
}

// Helper: compare two Decimals as "essentially equal" within a tolerance —
// break_infinity stores numbers as mantissa * 10^exponent, so for large
// values exact comparison via .eq() is too strict.
const closeEnough = (a: Decimal, b: Decimal, rel = 1e-9): void => {
  // For values near zero, fall back to absolute tolerance.
  if (a.abs().lt(1) && b.abs().lt(1)) {
    expect(a.minus(b).abs().lt(rel)).toBe(true)
    return
  }
  const diff = a.minus(b).abs()
  const scale = Decimal.max(a.abs(), b.abs())
  expect(diff.div(scale).lt(rel)).toBe(true)
}

describe('getCostAccelerator', () => {
  describe('default input (no challenge, no ECC, costDivisor=1)', () => {
    it('returns 500 for the first accelerator', () => {
      // After --buyingTo, the internal counter is 0, and (4/costDivisor)^0 = 1.
      closeEnough(getCostAccelerator(1, baseInput), new Decimal(500))
    })

    it('returns 2000 for the second (multiplied by 4)', () => {
      closeEnough(getCostAccelerator(2, baseInput), new Decimal(2000))
    })

    it('is monotonically non-decreasing across a wide range', () => {
      const samples = [1, 2, 5, 10, 50, 100, 125, 126, 200, 1000, 2000, 2001, 5000]
      let prev = new Decimal(0)
      for (const n of samples) {
        const cost = getCostAccelerator(n, baseInput)
        expect(cost.gte(prev)).toBe(true)
        prev = cost
      }
    })

    it('applies the factorial branch above 125', () => {
      const below = getCostAccelerator(125, baseInput)
      const justAbove = getCostAccelerator(127, baseInput)
      // Above 125 we multiply by num! * 4^num — should be much larger than
      // the smooth geometric trend.
      const geometric = below.times(Decimal.pow(4, 2))
      expect(justAbove.gt(geometric)).toBe(true)
    })

    it('applies the sum-of-arithmetic branch above 2000', () => {
      const below = getCostAccelerator(2000, baseInput)
      const justAbove = getCostAccelerator(2002, baseInput)
      // Above 2000 we additionally multiply by 2^(sumNum*(sumNum+1)/2).
      // For sumNum=2 that's 2^3 = 8, on top of the factorial growth.
      expect(justAbove.div(below).gt(8)).toBe(true)
    })
  })

  describe('challenge multipliers', () => {
    it('challenge 4 (transcension) amplifies the cost by 10^(n(n+1)/2)', () => {
      const at = (n: number, chal: boolean) =>
        getCostAccelerator(n, { ...baseInput, inTranscensionChallenge4: chal })
      // For buyingTo=2 internally is 1, sumBit=1*2/2=1, factor 10^1=10.
      const ratio = at(2, true).div(at(2, false))
      closeEnough(ratio, new Decimal(10))
    })

    it('challenge 8 (reincarnation) amplifies by 1e50^(n(n+1)/2)', () => {
      const at = (n: number, chal: boolean) =>
        getCostAccelerator(n, { ...baseInput, inReincarnationChallenge8: chal })
      const ratio = at(2, true).div(at(2, false))
      closeEnough(ratio, new Decimal(1e50))
    })
  })

  describe('transcend ECC raises the factorial/sum thresholds', () => {
    it('with transcendECC=10 the factorial branch starts at 175 instead of 125', () => {
      // 125 + 5*10 = 175. At buyingTo=176 internally is 175, which equals
      // the threshold (strict >), so still no factorial. At 177 → internal
      // 176 > 175 → factorial kicks in.
      const justBefore = getCostAccelerator(176, { ...baseInput, transcendECC: 10 })
      const justAfter = getCostAccelerator(177, { ...baseInput, transcendECC: 10 })
      // The factorial-branch jump is large; require the ratio exceeds the
      // smooth 4x step.
      expect(justAfter.div(justBefore).gt(4)).toBe(true)
    })
  })

  describe('buymax (1e15) diminishing branch', () => {
    it('returns a finite Decimal at the breakpoint', () => {
      const cost = getCostAccelerator(1e15, baseInput)
      expect(Number.isFinite(cost.mantissa)).toBe(true)
      expect(Number.isFinite(cost.exponent)).toBe(true)
    })

    it('still monotone across the buymax boundary', () => {
      const below = getCostAccelerator(1e15, baseInput)
      const above = getCostAccelerator(1.0001e15, baseInput)
      expect(above.gte(below)).toBe(true)
    })
  })

  describe('input immutability', () => {
    it('does not mutate the input object', () => {
      const input = { ...baseInput }
      const snapshot = JSON.stringify(input)
      getCostAccelerator(100, input)
      expect(JSON.stringify(input)).toBe(snapshot)
    })
  })
})

const baseBuyInput: BuyAcceleratorInput = {
  autobuyer: false,
  coinbuyamount: 100,
  costDivisor: 1,
  transcendECC: 0,
  inTranscensionChallenge4: false,
  inReincarnationChallenge8: false
}

const makeState = (overrides: Partial<AcceleratorState> = {}): AcceleratorState => ({
  acceleratorBought: 0,
  acceleratorCost: new Decimal(500),
  coins: new Decimal(0),
  prestigenoaccelerator: true,
  transcendnoaccelerator: true,
  reincarnatenoaccelerator: true,
  ...overrides
})

describe('buyAccelerator', () => {
  it('is a no-op when coins are zero', () => {
    const state = makeState()
    const { state: next, events } = buyAccelerator(state, baseBuyInput)
    expect(next.acceleratorBought).toBe(0)
    expect(next.coins.eq(state.coins)).toBe(true)
    expect(events).toEqual([])
  })

  it('purchases at least one accelerator given sufficient coins', () => {
    const state = makeState({ coins: new Decimal(1e6) })
    const { state: next, events } = buyAccelerator(state, baseBuyInput)
    expect(next.acceleratorBought).toBeGreaterThan(0)
    expect(next.coins.lt(state.coins)).toBe(true)
    expect(events).toHaveLength(1)
    expect(events[0]?.kind).toBe('accelerators-purchased')
  })

  it('respects coinbuyamount cap when not in autobuyer mode', () => {
    const state = makeState({ coins: new Decimal(1e30) })
    const { state: next } = buyAccelerator(state, { ...baseBuyInput, coinbuyamount: 10 })
    expect(next.acceleratorBought).toBeLessThanOrEqual(10)
  })

  it('bypasses the coinbuyamount cap in autobuyer mode', () => {
    const state = makeState({ coins: new Decimal(1e30) })
    const noAuto = buyAccelerator(state, { ...baseBuyInput, autobuyer: false, coinbuyamount: 10 })
    const yesAuto = buyAccelerator(state, { ...baseBuyInput, autobuyer: true, coinbuyamount: 10 })
    expect(yesAuto.state.acceleratorBought).toBeGreaterThan(noAuto.state.acceleratorBought)
  })

  it('flips the three no-accelerator flags to false on first purchase', () => {
    const state = makeState({ coins: new Decimal(1e6) })
    expect(state.prestigenoaccelerator).toBe(true)
    expect(state.transcendnoaccelerator).toBe(true)
    expect(state.reincarnatenoaccelerator).toBe(true)
    const { state: next } = buyAccelerator(state, baseBuyInput)
    expect(next.prestigenoaccelerator).toBe(false)
    expect(next.transcendnoaccelerator).toBe(false)
    expect(next.reincarnatenoaccelerator).toBe(false)
  })

  it('leaves the no-accelerator flags untouched when nothing was bought', () => {
    const state = makeState({ coins: new Decimal(0) })
    const { state: next } = buyAccelerator(state, baseBuyInput)
    expect(next.prestigenoaccelerator).toBe(true)
    expect(next.transcendnoaccelerator).toBe(true)
    expect(next.reincarnatenoaccelerator).toBe(true)
  })

  it('emits an event whose `spent` matches the coin delta', () => {
    const state = makeState({ coins: new Decimal(1e6) })
    const { state: next, events } = buyAccelerator(state, baseBuyInput)
    const spent = state.coins.sub(next.coins)
    expect(events[0]?.kind).toBe('accelerators-purchased')
    if (events[0]?.kind === 'accelerators-purchased') {
      expect(events[0].spent.eq(spent)).toBe(true)
      expect(events[0].before).toBe(0)
      expect(events[0].after).toBe(next.acceleratorBought)
    }
  })

  it('does not mutate the input state', () => {
    const state = makeState({ coins: new Decimal(1e6) })
    const snapshot = {
      acceleratorBought: state.acceleratorBought,
      acceleratorCostMantissa: state.acceleratorCost.mantissa,
      acceleratorCostExponent: state.acceleratorCost.exponent,
      coinsMantissa: state.coins.mantissa,
      coinsExponent: state.coins.exponent,
      prestigenoaccelerator: state.prestigenoaccelerator,
      transcendnoaccelerator: state.transcendnoaccelerator,
      reincarnatenoaccelerator: state.reincarnatenoaccelerator
    }
    buyAccelerator(state, baseBuyInput)
    expect(state.acceleratorBought).toBe(snapshot.acceleratorBought)
    expect(state.acceleratorCost.mantissa).toBe(snapshot.acceleratorCostMantissa)
    expect(state.acceleratorCost.exponent).toBe(snapshot.acceleratorCostExponent)
    expect(state.coins.mantissa).toBe(snapshot.coinsMantissa)
    expect(state.coins.exponent).toBe(snapshot.coinsExponent)
    expect(state.prestigenoaccelerator).toBe(snapshot.prestigenoaccelerator)
    expect(state.transcendnoaccelerator).toBe(snapshot.transcendnoaccelerator)
    expect(state.reincarnatenoaccelerator).toBe(snapshot.reincarnatenoaccelerator)
  })

  it('returns events typed as the discriminated union', () => {
    const state = makeState({ coins: new Decimal(1e6) })
    const { events } = buyAccelerator(state, baseBuyInput)
    for (const ev of events) {
      // Exhaustive switch — would fail to compile if a new variant landed
      // and this test wasn't updated.
      switch (ev.kind) {
        case 'accelerators-purchased':
          expect(typeof ev.before).toBe('number')
          expect(typeof ev.after).toBe('number')
          expect(ev.spent).toBeInstanceOf(Decimal)
          break
      }
    }
  })
})
