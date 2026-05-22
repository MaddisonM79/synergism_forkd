import { describe, expect, it } from 'vitest'
import {
  buyTesseractBuilding,
  calculateTessBuildingsInBudget,
  getTesseractCost,
  type BuyTesseractBuildingInput,
  type TesseractBuildingIndex,
  type TesseractBuildings
} from '../src/mechanics/tesseractBuildings'
import type { TesseractBuildingsState } from '../src/state/schema'

// Per-tier base cost — keep in lockstep with TESSERACT_BUILDING_COSTS in the
// source file. Tests assert costs via the formula intCost * n^3 so this table
// is only documentation.
const BASE_COSTS = [1, 10, 100, 1000, 10000] as const

const makeState = (
  ownedPerTier: [number, number, number, number, number] = [0, 0, 0, 0, 0],
  wowTesseracts = 0
): TesseractBuildingsState => ({
  wowTesseracts,
  ascendBuilding1: { owned: ownedPerTier[0], cost: BASE_COSTS[0] * Math.pow(1 + ownedPerTier[0], 3) },
  ascendBuilding2: { owned: ownedPerTier[1], cost: BASE_COSTS[1] * Math.pow(1 + ownedPerTier[1], 3) },
  ascendBuilding3: { owned: ownedPerTier[2], cost: BASE_COSTS[2] * Math.pow(1 + ownedPerTier[2], 3) },
  ascendBuilding4: { owned: ownedPerTier[3], cost: BASE_COSTS[3] * Math.pow(1 + ownedPerTier[3], 3) },
  ascendBuilding5: { owned: ownedPerTier[4], cost: BASE_COSTS[4] * Math.pow(1 + ownedPerTier[4], 3) }
})

describe('calculateTessBuildingsInBudget', () => {
  describe('documented anchor cases', () => {
    it('[0,0,0,0,0] + 100 → [3,1,0,0,0]', () => {
      expect(calculateTessBuildingsInBudget([0, 0, 0, 0, 0], 100)).toEqual([3, 1, 0, 0, 0])
    })

    it('[null,0,0,0,0] + 100 → [null,2,0,0,0]', () => {
      expect(calculateTessBuildingsInBudget([null, 0, 0, 0, 0], 100)).toEqual([null, 2, 0, 0, 0])
    })

    it('[3,1,0,0,0] + 143 → [4,1,0,0,0] (one shy of affording the next tier-2)', () => {
      // tier-1 cost for 4th = 1*4^3 = 64; tier-2 cost for 2nd = 10*2^3 = 80. Sum = 144.
      expect(calculateTessBuildingsInBudget([3, 1, 0, 0, 0], 64 + 80 - 1)).toEqual([4, 1, 0, 0, 0])
    })

    it('[3,1,0,0,0] + 144 → [4,2,0,0,0]', () => {
      expect(calculateTessBuildingsInBudget([3, 1, 0, 0, 0], 64 + 80)).toEqual([4, 2, 0, 0, 0])
    })

    it('[9,100,100,0,100] + 1000 → [9,100,100,1,100]', () => {
      expect(calculateTessBuildingsInBudget([9, 100, 100, 0, 100], 1000)).toEqual([9, 100, 100, 1, 100])
    })

    it('[9,100,100,0,100] + 2000 → [10,100,100,1,100]', () => {
      expect(calculateTessBuildingsInBudget([9, 100, 100, 0, 100], 2000)).toEqual([10, 100, 100, 1, 100])
    })
  })

  describe('degenerate inputs', () => {
    it('budget=0 → unchanged', () => {
      expect(calculateTessBuildingsInBudget([5, 5, 5, 5, 5], 0)).toEqual([5, 5, 5, 5, 5])
    })

    it('all-null → unchanged', () => {
      const all: TesseractBuildings = [null, null, null, null, null]
      expect(calculateTessBuildingsInBudget(all, 1e100)).toEqual(all)
    })

    it('budget below current cheapest next-price → unchanged', () => {
      // Cheapest next-price is tier-1 4th = 1*4^3 = 64. Budget 63 < 64.
      expect(calculateTessBuildingsInBudget([3, 3, 3, 3, 3], 63)).toEqual([3, 3, 3, 3, 3])
    })
  })

  describe('large budget performance bound', () => {
    it('[0,0,0,0,0] + 1e46 completes in under 1 second', () => {
      const start = performance.now()
      const result = calculateTessBuildingsInBudget([0, 0, 0, 0, 0], 1e46)
      const elapsed = performance.now() - start
      expect(elapsed).toBeLessThan(1000)
      // Sanity: result should have non-trivial counts across all tiers.
      for (const n of result) {
        expect(n).not.toBeNull()
        expect(n).toBeGreaterThan(0)
      }
    })
  })

  describe('input immutability', () => {
    it('does not mutate the input array', () => {
      const input: TesseractBuildings = [0, 0, 0, 0, 0]
      const snapshot = [...input]
      calculateTessBuildingsInBudget(input, 1e6)
      expect(input).toEqual(snapshot)
    })
  })
})

describe('getTesseractCost', () => {
  it('anchors: index=1, amount=10, owned=0, infinite budget → buy 10 at cumulative cost 3025', () => {
    // cost(10) = 1 * (10*11/2)^2 = 55^2 = 3025
    const state = makeState([0, 0, 0, 0, 0], Number.MAX_SAFE_INTEGER)
    const [actualBuy, actualCost] = getTesseractCost(1, { amount: 10 }, state)
    expect(actualBuy).toBe(10)
    expect(actualCost).toBe(3025)
  })

  it('subtracts the cumulative subCost when buying from a non-zero base', () => {
    // From owned=5 (subCost = 1*15^2 = 225) buying 5 more to 10 (cumCost=3025).
    // Net cost = 3025 - 225 = 2800.
    const state = makeState([5, 0, 0, 0, 0], Number.MAX_SAFE_INTEGER)
    const [actualBuy, actualCost] = getTesseractCost(1, { amount: 5 }, state)
    expect(actualBuy).toBe(10)
    expect(actualCost).toBe(2800)
  })

  it('honors the higher-tier cost multiplier', () => {
    // Tier 5 base 10000. cost(2) - cost(0) = 10000 * (2*3/2)^2 = 10000 * 9 = 90000.
    const state = makeState([0, 0, 0, 0, 0], Number.MAX_SAFE_INTEGER)
    const [actualBuy, actualCost] = getTesseractCost(5, { amount: 2 }, state)
    expect(actualBuy).toBe(2)
    expect(actualCost).toBe(90000)
  })

  it('caps actualBuy when budget is the limiting factor (checkCanAfford default true)', () => {
    // Budget 100 on tier 1: largest n with n*(n+1)/2 <= sqrt(100) = 10 → n=4 (4*5/2=10).
    const state = makeState([0, 0, 0, 0, 0], 100)
    const [actualBuy, actualCost] = getTesseractCost(1, { amount: 1000 }, state)
    expect(actualBuy).toBe(4)
    expect(actualCost).toBe(100)
  })

  it('checkCanAfford=false buys the full amount regardless of budget', () => {
    const state = makeState([0, 0, 0, 0, 0], 0)
    const [actualBuy, actualCost] = getTesseractCost(1, { amount: 5, checkCanAfford: false }, state)
    expect(actualBuy).toBe(5)
    expect(actualCost).toBe(225) // 1 * (5*6/2)^2 = 225
  })

  it('buyFrom override replaces the state-derived starting count', () => {
    const state = makeState([2, 0, 0, 0, 0], Number.MAX_SAFE_INTEGER)
    // Override starts at 5 instead of 2; buy 5 more to 10.
    const [actualBuy, actualCost] = getTesseractCost(1, { amount: 5, buyFrom: 5 }, state)
    expect(actualBuy).toBe(10)
    expect(actualCost).toBe(2800)
  })
})

const baseBuyInput = (overrides: Partial<BuyTesseractBuildingInput> = {}): BuyTesseractBuildingInput => ({
  index: 1,
  amount: 10,
  ...overrides
})

describe('buyTesseractBuilding', () => {
  it('purchases up to the requested amount when budget allows', () => {
    const state = makeState([0, 0, 0, 0, 0], 1e6)
    const { state: next, events } = buyTesseractBuilding(state, baseBuyInput({ amount: 10 }))
    expect(next.ascendBuilding1.owned).toBe(10)
    expect(next.ascendBuilding1.cost).toBe(1 * Math.pow(11, 3)) // 1331
    expect(next.wowTesseracts).toBe(1e6 - 3025)
    expect(events).toHaveLength(1)
    expect(events[0]?.kind).toBe('tesseract-buildings-purchased')
  })

  it('caps at budget when affordability is the limit', () => {
    const state = makeState([0, 0, 0, 0, 0], 100)
    const { state: next } = buyTesseractBuilding(state, baseBuyInput({ amount: 1000 }))
    expect(next.ascendBuilding1.owned).toBe(4)
    expect(next.wowTesseracts).toBe(0)
  })

  it('is a no-op when amount=0', () => {
    const state = makeState([5, 0, 0, 0, 0], 1e6)
    const { state: next, events } = buyTesseractBuilding(state, baseBuyInput({ amount: 0 }))
    expect(next.ascendBuilding1.owned).toBe(5)
    expect(next.wowTesseracts).toBe(1e6)
    expect(events).toEqual([])
  })

  it('updates only the targeted tier when buying tier 3', () => {
    const state = makeState([1, 2, 0, 0, 0], 1e9)
    const { state: next } = buyTesseractBuilding(state, baseBuyInput({ index: 3, amount: 4 }))
    expect(next.ascendBuilding3.owned).toBe(4)
    // Untouched tiers unchanged.
    expect(next.ascendBuilding1.owned).toBe(1)
    expect(next.ascendBuilding2.owned).toBe(2)
    expect(next.ascendBuilding4.owned).toBe(0)
    expect(next.ascendBuilding5.owned).toBe(0)
    // Untouched costs preserved.
    expect(next.ascendBuilding1.cost).toBe(state.ascendBuilding1.cost)
    expect(next.ascendBuilding2.cost).toBe(state.ascendBuilding2.cost)
  })

  it('emits an event whose spent equals the wowTesseracts delta', () => {
    const state = makeState([0, 0, 0, 0, 0], 1e6)
    const { state: next, events } = buyTesseractBuilding(state, baseBuyInput({ amount: 10 }))
    const spent = state.wowTesseracts - next.wowTesseracts
    expect(events[0]?.kind).toBe('tesseract-buildings-purchased')
    if (events[0]?.kind === 'tesseract-buildings-purchased') {
      expect(events[0].spent).toBe(spent)
      expect(events[0].before).toBe(0)
      expect(events[0].after).toBe(10)
      expect(events[0].index).toBe(1)
    }
  })

  it('does not mutate the input state', () => {
    const state = makeState([3, 0, 0, 0, 0], 1e6)
    const snapshot = {
      wowTesseracts: state.wowTesseracts,
      ascendBuilding1Owned: state.ascendBuilding1.owned,
      ascendBuilding1Cost: state.ascendBuilding1.cost,
      ascendBuilding2Owned: state.ascendBuilding2.owned,
      ascendBuilding2Cost: state.ascendBuilding2.cost
    }
    buyTesseractBuilding(state, baseBuyInput({ amount: 100 }))
    expect(state.wowTesseracts).toBe(snapshot.wowTesseracts)
    expect(state.ascendBuilding1.owned).toBe(snapshot.ascendBuilding1Owned)
    expect(state.ascendBuilding1.cost).toBe(snapshot.ascendBuilding1Cost)
    expect(state.ascendBuilding2.owned).toBe(snapshot.ascendBuilding2Owned)
    expect(state.ascendBuilding2.cost).toBe(snapshot.ascendBuilding2Cost)
  })

  it('each tier index buys its own slot', () => {
    for (const index of [1, 2, 3, 4, 5] as TesseractBuildingIndex[]) {
      const state = makeState([0, 0, 0, 0, 0], 1e18)
      const { state: next } = buyTesseractBuilding(state, baseBuyInput({ index, amount: 3 }))
      // The targeted tier's owned bumps to 3.
      const tierOwned = [
        next.ascendBuilding1.owned,
        next.ascendBuilding2.owned,
        next.ascendBuilding3.owned,
        next.ascendBuilding4.owned,
        next.ascendBuilding5.owned
      ]
      expect(tierOwned[index - 1]).toBe(3)
      // The other tiers stay at zero.
      for (let i = 0; i < 5; i++) {
        if (i !== index - 1) expect(tierOwned[i]).toBe(0)
      }
    }
  })
})
