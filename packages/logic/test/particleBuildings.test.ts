import { describe, expect, it } from 'vitest'
import { Decimal } from '../src/math/bignum'
import {
  buyParticleBuilding,
  getParticleCost,
  type BuyParticleBuildingInput,
  type GetParticleCostInput,
  type ParticleBuildingIndex
} from '../src/mechanics/particleBuildings'
import type { ParticleBuildingsState } from '../src/state/schema'

const ORIGINAL_COSTS: Record<ParticleBuildingIndex, number> = {
  1: 1,
  2: 1e2,
  3: 1e4,
  4: 1e8,
  5: 1e16
}

const baseInput = (index: ParticleBuildingIndex = 1): GetParticleCostInput => ({
  index,
  inAscensionChallenge15: false
})

const closeEnough = (a: Decimal, b: Decimal, rel = 1e-9): void => {
  if (a.abs().lt(1) && b.abs().lt(1)) {
    expect(a.minus(b).abs().lt(rel)).toBe(true)
    return
  }
  const diff = a.minus(b).abs()
  const scale = Decimal.max(a.abs(), b.abs())
  expect(diff.div(scale).lt(rel)).toBe(true)
}

describe('getParticleCost', () => {
  describe('anchor values per position', () => {
    it.each<ParticleBuildingIndex>([1, 2, 3, 4, 5])(
      'index %s returns originalCost for buyingTo=1',
      (index) => {
        closeEnough(getParticleCost(1, baseInput(index)), new Decimal(ORIGINAL_COSTS[index]))
      }
    )

    it('doubles between successive purchases below the DR threshold', () => {
      const a = getParticleCost(5, baseInput(1))
      const b = getParticleCost(6, baseInput(1))
      closeEnough(b.div(a), new Decimal(2))
    })
  })

  describe('monotonicity', () => {
    it('is monotonically non-decreasing across normal and high counts', () => {
      const samples = [1, 2, 10, 100, 1000, 100000, 325000, 325001, 400000, 1e6, 1e10]
      let prev = new Decimal(0)
      for (const n of samples) {
        const cost = getParticleCost(n, baseInput(1))
        expect(cost.gte(prev)).toBe(true)
        prev = cost
      }
    })
  })

  describe('DR (diminishing returns) threshold', () => {
    it('out of ascension challenge 15, threshold is 325000 — below it is plain doubling', () => {
      // buyingTo=325001 → internal 325000, equal to DR (strict >), no quadratic kick yet.
      const at = getParticleCost(325001, baseInput(1))
      const oneLess = getParticleCost(325000, baseInput(1))
      // Plain doubling: a single step should still be a factor of 2.
      closeEnough(at.div(oneLess), new Decimal(2))
    })

    it('out of ascension challenge 15, quadratic kicks in just above 325000', () => {
      const justBelow = getParticleCost(325001, baseInput(1))
      const wellAbove = getParticleCost(325100, baseInput(1))
      // Above DR cost is multiplied by 1.001^(n*(n+1)/2) where n is the
      // distance past DR — at n=99 that's 1.001^4950 ≈ 142, far above the
      // smooth 2^99 doubling alone.
      const smoothDoubling = justBelow.times(Decimal.pow(2, 99))
      expect(wellAbove.gt(smoothDoubling.times(100))).toBe(true)
    })

    it('in ascension challenge 15, DR drops to 1000 — quadratic kicks in much earlier', () => {
      // At buyingTo=2000 (internal 1999): out of C15 is plain 2^1999, in C15
      // also has 1.001^(999*1000/2) on top. The ratio must be >> 1.
      const inC15 = getParticleCost(2000, { ...baseInput(1), inAscensionChallenge15: true })
      const outC15 = getParticleCost(2000, baseInput(1))
      expect(inC15.div(outC15).gt(1e100)).toBe(true)
    })
  })

  describe('buymax (1e15) diminishing branch', () => {
    it('returns a finite Decimal at the breakpoint', () => {
      const cost = getParticleCost(1e15, baseInput(1))
      expect(Number.isFinite(cost.mantissa)).toBe(true)
      expect(Number.isFinite(cost.exponent)).toBe(true)
    })

    it('still monotone across the buymax boundary', () => {
      const below = getParticleCost(1e15, baseInput(1))
      const above = getParticleCost(1.0001e15, baseInput(1))
      expect(above.gte(below)).toBe(true)
    })
  })

  describe('input immutability', () => {
    it('does not mutate the input object', () => {
      const input = baseInput(2)
      const snapshot = JSON.stringify(input)
      getParticleCost(1000, input)
      expect(JSON.stringify(input)).toBe(snapshot)
    })
  })
})

const baseBuyInput = (overrides: Partial<BuyParticleBuildingInput> = {}): BuyParticleBuildingInput => ({
  index: 1,
  autobuyer: false,
  particlebuyamount: 100,
  inAscensionChallenge15: false,
  ...overrides
})

const makeState = (overrides: Partial<ParticleBuildingsState> = {}): ParticleBuildingsState => ({
  reincarnationPoints: new Decimal(0),
  firstOwnedParticles: 0,
  firstCostParticles: new Decimal(ORIGINAL_COSTS[1]),
  secondOwnedParticles: 0,
  secondCostParticles: new Decimal(ORIGINAL_COSTS[2]),
  thirdOwnedParticles: 0,
  thirdCostParticles: new Decimal(ORIGINAL_COSTS[3]),
  fourthOwnedParticles: 0,
  fourthCostParticles: new Decimal(ORIGINAL_COSTS[4]),
  fifthOwnedParticles: 0,
  fifthCostParticles: new Decimal(ORIGINAL_COSTS[5]),
  ...overrides
})

describe('buyParticleBuilding', () => {
  it('is a no-op when reincarnationPoints are zero', () => {
    const state = makeState()
    const { state: next, events } = buyParticleBuilding(state, baseBuyInput())
    expect(next.firstOwnedParticles).toBe(0)
    expect(next.reincarnationPoints.eq(state.reincarnationPoints)).toBe(true)
    expect(events).toEqual([])
  })

  it('purchases at least one building given sufficient points', () => {
    const state = makeState({ reincarnationPoints: new Decimal(1e6) })
    const { state: next, events } = buyParticleBuilding(state, baseBuyInput())
    expect(next.firstOwnedParticles).toBeGreaterThan(0)
    expect(next.reincarnationPoints.lt(state.reincarnationPoints)).toBe(true)
    expect(events).toHaveLength(1)
    expect(events[0]?.kind).toBe('particle-buildings-purchased')
  })

  it('respects particlebuyamount cap when not in autobuyer mode', () => {
    const state = makeState({ reincarnationPoints: new Decimal(1e30) })
    const { state: next } = buyParticleBuilding(state, baseBuyInput({ particlebuyamount: 10 }))
    // Cap is enforced with a smallestInc fudge — at low values it adds 1, so
    // buyingTo can land one above the strict cap.
    expect(next.firstOwnedParticles).toBeLessThanOrEqual(11)
  })

  it('bypasses the particlebuyamount cap in autobuyer mode', () => {
    const state = makeState({ reincarnationPoints: new Decimal(1e30) })
    const noAuto = buyParticleBuilding(state, baseBuyInput({ autobuyer: false, particlebuyamount: 10 }))
    const yesAuto = buyParticleBuilding(state, baseBuyInput({ autobuyer: true, particlebuyamount: 10 }))
    expect(yesAuto.state.firstOwnedParticles).toBeGreaterThan(noAuto.state.firstOwnedParticles)
  })

  it('updates only the targeted position when buying index 3', () => {
    const state = makeState({ reincarnationPoints: new Decimal(1e20) })
    const { state: next } = buyParticleBuilding(state, baseBuyInput({ index: 3 }))
    expect(next.thirdOwnedParticles).toBeGreaterThan(0)
    expect(next.firstOwnedParticles).toBe(0)
    expect(next.secondOwnedParticles).toBe(0)
    expect(next.fourthOwnedParticles).toBe(0)
    expect(next.fifthOwnedParticles).toBe(0)
    expect(next.thirdCostParticles.gt(state.thirdCostParticles)).toBe(true)
    expect(next.firstCostParticles.eq(state.firstCostParticles)).toBe(true)
  })

  it('emits an event whose spent matches the resource delta', () => {
    const state = makeState({ reincarnationPoints: new Decimal(1e6) })
    const { state: next, events } = buyParticleBuilding(state, baseBuyInput())
    const spent = state.reincarnationPoints.sub(next.reincarnationPoints)
    expect(events[0]?.kind).toBe('particle-buildings-purchased')
    if (events[0]?.kind === 'particle-buildings-purchased') {
      expect(events[0].spent.eq(spent)).toBe(true)
      expect(events[0].before).toBe(0)
      expect(events[0].after).toBe(next.firstOwnedParticles)
      expect(events[0].index).toBe(1)
    }
  })

  it('does not mutate the input state', () => {
    const state = makeState({ reincarnationPoints: new Decimal(1e6) })
    const snapshot = {
      reincarnationPointsMantissa: state.reincarnationPoints.mantissa,
      reincarnationPointsExponent: state.reincarnationPoints.exponent,
      firstOwnedParticles: state.firstOwnedParticles,
      firstCostParticlesMantissa: state.firstCostParticles.mantissa,
      firstCostParticlesExponent: state.firstCostParticles.exponent
    }
    buyParticleBuilding(state, baseBuyInput())
    expect(state.reincarnationPoints.mantissa).toBe(snapshot.reincarnationPointsMantissa)
    expect(state.reincarnationPoints.exponent).toBe(snapshot.reincarnationPointsExponent)
    expect(state.firstOwnedParticles).toBe(snapshot.firstOwnedParticles)
    expect(state.firstCostParticles.mantissa).toBe(snapshot.firstCostParticlesMantissa)
    expect(state.firstCostParticles.exponent).toBe(snapshot.firstCostParticlesExponent)
  })

  it('returns events typed as the discriminated union', () => {
    const state = makeState({ reincarnationPoints: new Decimal(1e6) })
    const { events } = buyParticleBuilding(state, baseBuyInput())
    for (const ev of events) {
      switch (ev.kind) {
        case 'particle-buildings-purchased':
          expect(typeof ev.before).toBe('number')
          expect(typeof ev.after).toBe('number')
          expect(typeof ev.index).toBe('number')
          expect(ev.spent).toBeInstanceOf(Decimal)
          break
      }
    }
  })
})
