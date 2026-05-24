// Parity test for the research cost tables + polynomial cost solvers.
// Old bodies transcribed verbatim from packages/web_ui/src/Research.ts
// (lines 27-69, 72-113, 129-146, 155-158). Data tables: sampled indices
// across each band plus length checks. Solvers: exhaustive sweep of degree
// 1 and 2 across budgets and (currLevel, maxLevel, buyTo) combos.

import Decimal, { type DecimalSource } from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  polyBuyToLevel as newPolyBuyToLevel,
  polyCostForLevels as newPolyCostForLevels,
  researchBaseCosts as newResearchBaseCosts,
  researchLevelCostRanges as newResearchLevelCostRanges,
  researchMaxLevels as newResearchMaxLevels
} from '../../src/mechanics/researches'

// ─── Old data tables (verbatim from packages/web_ui/src/Research.ts) ──────

// dprint-ignore
const oldResearchBaseCosts: DecimalSource[] = [
  Number.POSITIVE_INFINITY,
  1, 1, 1, 1, 1,
  1, 1e2, 1e4, 1e6, 1e8,
  2, 2e2, 2e4, 2e6, 2e8,
  4e4, 4e8, 10, 1e5, 1e9,
  100, 100, 1e4, 2e3, 2e5,
  40, 200, 50, 5000, 20000000,
  777, 7777, 50000, 500000, 5000000,
  2e3, 2e6, 2e9, 1e5, 1e9,
  1, 1, 5, 25, 125,
  2, 5, 320, 1280, 2.5e9,
  10, 2e3, 4e5, 8e7, 2e9,
  5, 400, 1e4, 3e6, 9e8,
  100, 2500, 100, 2000, 2e5,
  1, 20, 3e3, 4e5, 5e7,
  10, 40, 160, 1000, 10000,
  4e9, 7e9, 1e10, 1.2e10, 1.5e10,
  1e12, 1e13, 1e12, 4e12, 7e12,
  1e13, 1e13, 4e13, 6e13, 1e14,
  8e13, 1e14, 2e14, 2e14, 1e15,
  4e12, 3e13, 8e13, 7.777e18, 7.777e20,
  2e14, 3e14, 1e16, 3e16, 1e16,
  1e17, 3e17, 5e16, 1.2e17, 1e18,
  1e18, 2e18, 3e18, 4e18, 1e19,
  1e19, 2e19, 1e21, 5e21, 1e22,
  1e21, 1e22, 1e22, 1e20, 7.777e32,
  5e8, 5e12, 5e16, 5e20, 5e24,
  1e25, 2e25, 4e25, 8e25, 1e26,
  4e26, 8e26, 1e27, 2e27, 1e28,
  5e9, 5e15, 5e21, 5e27, 1e28,
  1e29, 2e29, 4e29, 8e29, 1e27,
  2e30, 4e30, 8e30, 1e31, 2e31,
  5e31, 1e32, 2e32, 4e32, 8e32,
  1e33, 2e33, 4e33, 8e33, 1e34,
  3e34, 1e35, 3e35, 6e37, 1e36,
  3e36, 1e37, 3e37, 1e38, 3e38,
  1e39, 3e39, 1e40, 3e40, 1e50,
  3e41, 1e42, 3e42, 6e42, 1e43,
  3e43, 1e44, 3e44, 1e45, 3e45,
  2e46, 6e46, 2e47, 6e47, 1e64,
  6e48, 2e49, 1e50, 1e51, 4e56
]

// dprint-ignore
const oldResearchMaxLevels: DecimalSource[] = [
  0, 1, 1, 1, 1, 1,
  10, 10, 10, 10, 10,
  10, 10, 10, 10, 10,
  10, 10, 1, 1, 1,
  25, 25, 25, 20, 20,
  10, 10, 10, 10, 10,
  12, 12, 10, 10, 10,
  10, 10, 10, 1, 1,
  1, 1, 1, 1, 1,
  1, 1, 1, 1, 1,
  10, 10, 10, 10, 10,
  20, 20, 20, 20, 20,
  1, 5, 4, 5, 5,
  10, 10, 10, 10, 10,
  1, 1, 1, 1, 1,
  10, 15, 15, 15, 15,
  10, 1, 20, 20, 20,
  20, 20, 20, 20, 10,
  20, 20, 20, 20, 1,
  20, 7, 7, 3, 2,
  10, 12, 10, 10, 1,
  10, 10, 20, 25, 25,
  15, 15, 15, 15, 30,
  2, 10, 10, 100, 100,
  25, 25, 25, 1, 5,
  10, 10, 10, 10, 1,
  10, 10, 10, 1, 1,
  25, 25, 25, 15, 1,
  10, 10, 10, 10, 1,
  10, 1, 25, 10, 1,
  25, 25, 1, 15, 1,
  10, 10, 10, 1, 1,
  10, 10, 10, 10, 1,
  25, 25, 25, 100000, 1,
  10, 10, 10, 1, 1,
  10, 3, 6, 10, 5,
  25, 25, 1, 15, 1,
  20, 20, 20, 1, 1,
  20, 1, 50, 50, 10,
  25, 25, 25, 15, 100000
]

// ─── Old solvers ───────────────────────────────────────────────────────────

const oldPolyBuyToLevel = (
  degree: number
): (budget: Decimal, baseCost: Decimal, currLevel: number, maxLevel: number) => number => {
  return (budget: Decimal, baseCost: Decimal, currLevel: number, maxLevel: number) => {
    const effectiveBudget = budget.add(baseCost.times(Math.pow(currLevel, degree)))
    return Math.min(maxLevel, Decimal.floor(Decimal.pow(effectiveBudget.div(baseCost), 1 / degree)).toNumber())
  }
}

const oldPolyCostForLevels = (
  degree: number
): (baseCost: Decimal, currLevel: number, buyTo: number) => Decimal => {
  return (baseCost: Decimal, currLevel: number, buyTo: number) => {
    if (currLevel === buyTo) {
      return new Decimal(0)
    }
    return baseCost.times(Math.pow(buyTo, degree) - Math.pow(currLevel, degree))
  }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

describe('parity: researchBaseCosts table', () => {
  it('length matches', () => {
    expect(newResearchBaseCosts.length).toBe(oldResearchBaseCosts.length)
    expect(newResearchBaseCosts.length).toBe(201)
  })
  // Sample across each tier band: tutorial (0-5), early (6-40), mid (41-75),
  // ascension (126-130), challenge bands (141, 156, 171, 186), final (200).
  const sampleIndices = [
    0, 1, 5, 6, 10, 17, 25, 40, 50, 75, 100, 125, 126, 130, 141, 156, 171, 186, 199, 200
  ]
  for (const i of sampleIndices) {
    it(`index ${i}`, () => {
      expect(decimalEq(
        new Decimal(newResearchBaseCosts[i]),
        new Decimal(oldResearchBaseCosts[i])
      )).toBe(true)
    })
  }
})

describe('parity: researchMaxLevels table', () => {
  it('length matches', () => {
    expect(newResearchMaxLevels.length).toBe(oldResearchMaxLevels.length)
    expect(newResearchMaxLevels.length).toBe(201)
  })
  const sampleIndices = [
    0, 1, 5, 6, 10, 17, 25, 40, 50, 75, 100, 125, 126, 130, 141, 156, 171, 186, 199, 200
  ]
  for (const i of sampleIndices) {
    it(`index ${i}`, () => {
      expect(Number(newResearchMaxLevels[i])).toBe(Number(oldResearchMaxLevels[i]))
    })
  }
})

describe('parity: researchLevelCostRanges', () => {
  it('shape matches', () => {
    expect(newResearchLevelCostRanges.length).toBe(2)
    expect(newResearchLevelCostRanges[0].range).toEqual([0, 199])
    expect(newResearchLevelCostRanges[1].range).toEqual([200, 200])
  })
})

describe('parity: polyBuyToLevel(degree=1)', () => {
  const newSolver = newPolyBuyToLevel(1)
  const oldSolver = oldPolyBuyToLevel(1)
  const budgets = [new Decimal(0), new Decimal(1), new Decimal(100), new Decimal('1e10'), new Decimal('1e30')]
  const baseCosts = [new Decimal(1), new Decimal(100), new Decimal('1e9'), new Decimal('1e20')]
  const combos = [
    { currLevel: 0, maxLevel: 1 },
    { currLevel: 0, maxLevel: 25 },
    { currLevel: 5, maxLevel: 25 },
    { currLevel: 20, maxLevel: 25 },
    { currLevel: 0, maxLevel: 100000 }
  ]
  for (const budget of budgets) {
    for (const baseCost of baseCosts) {
      for (const { currLevel, maxLevel } of combos) {
        it(`budget=${budget.toString()} base=${baseCost.toString()} curr=${currLevel} max=${maxLevel}`, () => {
          expect(newSolver(budget, baseCost, currLevel, maxLevel))
            .toBe(oldSolver(budget, baseCost, currLevel, maxLevel))
        })
      }
    }
  }
})

describe('parity: polyBuyToLevel(degree=2)', () => {
  const newSolver = newPolyBuyToLevel(2)
  const oldSolver = oldPolyBuyToLevel(2)
  const budgets = [new Decimal(0), new Decimal(100), new Decimal('1e30'), new Decimal('1e100')]
  const baseCosts = [new Decimal('1e50'), new Decimal('4e56')]
  const combos = [
    { currLevel: 0, maxLevel: 100000 },
    { currLevel: 100, maxLevel: 100000 },
    { currLevel: 50000, maxLevel: 100000 }
  ]
  for (const budget of budgets) {
    for (const baseCost of baseCosts) {
      for (const { currLevel, maxLevel } of combos) {
        it(`budget=${budget.toString()} base=${baseCost.toString()} curr=${currLevel} max=${maxLevel}`, () => {
          expect(newSolver(budget, baseCost, currLevel, maxLevel))
            .toBe(oldSolver(budget, baseCost, currLevel, maxLevel))
        })
      }
    }
  }
})

describe('parity: polyCostForLevels(degree=1)', () => {
  const newSolver = newPolyCostForLevels(1)
  const oldSolver = oldPolyCostForLevels(1)
  const baseCosts = [new Decimal(1), new Decimal(100), new Decimal('1e9'), new Decimal('1e20')]
  const combos = [
    { currLevel: 0, buyTo: 0 }, // identity case
    { currLevel: 0, buyTo: 1 },
    { currLevel: 0, buyTo: 25 },
    { currLevel: 5, buyTo: 25 },
    { currLevel: 25, buyTo: 25 } // identity case
  ]
  for (const baseCost of baseCosts) {
    for (const { currLevel, buyTo } of combos) {
      it(`base=${baseCost.toString()} curr=${currLevel} buyTo=${buyTo}`, () => {
        expect(decimalEq(
          newSolver(baseCost, currLevel, buyTo),
          oldSolver(baseCost, currLevel, buyTo)
        )).toBe(true)
      })
    }
  }
})

describe('parity: polyCostForLevels(degree=2)', () => {
  const newSolver = newPolyCostForLevels(2)
  const oldSolver = oldPolyCostForLevels(2)
  const baseCosts = [new Decimal('1e50'), new Decimal('4e56')]
  const combos = [
    { currLevel: 0, buyTo: 0 }, // identity case
    { currLevel: 0, buyTo: 100 },
    { currLevel: 100, buyTo: 50000 },
    { currLevel: 50000, buyTo: 100000 }
  ]
  for (const baseCost of baseCosts) {
    for (const { currLevel, buyTo } of combos) {
      it(`base=${baseCost.toString()} curr=${currLevel} buyTo=${buyTo}`, () => {
        expect(decimalEq(
          newSolver(baseCost, currLevel, buyTo),
          oldSolver(baseCost, currLevel, buyTo)
        )).toBe(true)
      })
    }
  }
})
