// Parity tests for the summation/cost helpers lifted from
// packages/web_ui/src/Calculate.ts. The old impls used i18next-translated
// strings in their throw paths — the new versions throw plain codes
// (`SUMMATIONS_*`); the value-comparison sweeps don't exercise those
// throws, and separate `expect.toThrow` cases assert the code identity.

import { describe, expect, it } from 'vitest'
import {
  calculateCubicSumData as newCubicSumData,
  calculateSummationCubic as newSummationCubic,
  calculateSummationNonLinear as newSummationNonLinear,
  solveQuadratic as newSolveQuadratic
} from '../../src/math/summations'

// ─── Old implementations (verbatim, with throw messages replaced by codes
//     to match the new behavior) ─────────────────────────────────────────

const oldSummationNonLinear = (
  baseLevel: number,
  baseCost: number,
  resourceAvailable: number,
  diffPerLevel: number,
  buyAmount: number
): { levelCanBuy: number; cost: number } => {
  const c = diffPerLevel / 2
  resourceAvailable = resourceAvailable || 0
  const alreadySpent = baseCost * (c * Math.pow(baseLevel, 2) + baseLevel * (1 - c))
  resourceAvailable += alreadySpent
  const v = resourceAvailable / baseCost
  let buyToLevel = c > 0
    ? Math.max(
      0,
      Math.floor(
        (c - 1) / (2 * c)
          + Math.pow(Math.pow(1 - c, 2) + 4 * c * v, 1 / 2) / (2 * c)
      )
    )
    : Math.floor(v)

  buyToLevel = Math.min(buyToLevel, buyAmount + baseLevel)
  buyToLevel = Math.max(buyToLevel, baseLevel)
  let totalCost = baseCost * (c * Math.pow(buyToLevel, 2) + buyToLevel * (1 - c))
    - alreadySpent
  if (buyToLevel === baseLevel) {
    totalCost = baseCost * (1 + 2 * c * baseLevel)
  }
  return { levelCanBuy: buyToLevel, cost: totalCost }
}

const oldSummationCubic = (n: number): number => {
  if (n < 0) return -1
  if (!Number.isInteger(n)) return -1
  return Math.pow((n * (n + 1)) / 2, 2)
}

const oldSolveQuadratic = (a: number, b: number, c: number, positive: boolean): number => {
  if (a < 0) {
    throw new Error('SUMMATIONS_QUADRATIC_IMPROPER')
  }
  const determinant = Math.pow(b, 2) - 4 * a * c
  if (determinant < 0) {
    throw new Error('SUMMATIONS_QUADRATIC_DETERMINANT')
  }
  if (determinant === 0) {
    return -b / (2 * a)
  }
  const numeratorPos = -b + Math.sqrt(Math.pow(b, 2) - 4 * a * c)
  const numeratorNeg = -b - Math.sqrt(Math.pow(b, 2) - 4 * a * c)
  return positive ? numeratorPos / (2 * a) : numeratorNeg / (2 * a)
}

const oldCubicSumData = (
  initialLevel: number,
  baseCost: number,
  amountToSpend: number,
  maxLevel: number
) => {
  if (initialLevel >= maxLevel) {
    return { levelCanBuy: maxLevel, cost: 0 }
  }
  const alreadySpent = baseCost * oldSummationCubic(initialLevel)
  const totalToSpend = alreadySpent + amountToSpend

  if (totalToSpend < 0) {
    throw new Error('SUMMATIONS_CUBIC_SUM_NEGATIVE')
  }

  const determinantRoot = Math.pow(totalToSpend / baseCost, 0.5)
  const solution = oldSolveQuadratic(1, 1, -2 * determinantRoot, true)

  const levelToBuy = Math.max(
    Math.min(maxLevel, Math.floor(solution)),
    initialLevel
  )
  const realCost = levelToBuy === initialLevel
    ? baseCost * Math.pow(initialLevel + 1, 3)
    : baseCost * oldSummationCubic(levelToBuy) - alreadySpent

  return { levelCanBuy: levelToBuy, cost: realCost }
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: calculateSummationNonLinear', () => {
  const baseLevels = [0, 1, 10, 100]
  const baseCosts = [1, 10, 100, 1e6]
  const resources = [0, 1, 10, 100, 1e4, 1e8, 1e15]
  const diffs = [0, 0.5, 1, 2]
  const buyAmounts = [1, 10, 100, 1000]
  for (const baseLevel of baseLevels) {
    for (const baseCost of baseCosts) {
      for (const diff of diffs) {
        for (const buyAmount of buyAmounts) {
          it.each(resources)(
            `baseLvl=${baseLevel} baseCost=${baseCost} diff=${diff} buyAmt=${buyAmount} res=%s`,
            (res) => {
              const next = newSummationNonLinear(baseLevel, baseCost, res, diff, buyAmount)
              const old = oldSummationNonLinear(baseLevel, baseCost, res, diff, buyAmount)
              expect(next.levelCanBuy).toBe(old.levelCanBuy)
              expect(closeEnough(next.cost, old.cost)).toBe(true)
            }
          )
        }
      }
    }
  }
})

describe('parity: calculateSummationCubic', () => {
  // Cover the special-return-(-1) paths (negative, non-integer) and the
  // happy path with various positive integers.
  const grid = [-5, -1, 0, 1, 2, 3, 10, 50, 100, 1000]
  it.each(grid)('n=%s', (n) => {
    expect(newSummationCubic(n)).toBe(oldSummationCubic(n))
  })
  it.each([0.5, 1.5, 99.9])('n=%s (non-integer)', (n) => {
    expect(newSummationCubic(n)).toBe(oldSummationCubic(n))
  })
})

describe('parity: solveQuadratic', () => {
  const cases: Array<[number, number, number, boolean]> = [
    [1, 0, -4, true],
    [1, 0, -4, false],
    [1, -5, 6, true],
    [1, -5, 6, false],
    [1, 2, 1, true],
    [1, 2, 1, false],
    [1, 1, -6, true],
    [2, 4, 2, true]
  ]
  it.each(cases)('a=%s b=%s c=%s pos=%s', (a, b, c, positive) => {
    expect(closeEnough(newSolveQuadratic(a, b, c, positive), oldSolveQuadratic(a, b, c, positive))).toBe(true)
  })

  it('throws SUMMATIONS_QUADRATIC_IMPROPER when a < 0', () => {
    expect(() => newSolveQuadratic(-1, 0, 0, true)).toThrow('SUMMATIONS_QUADRATIC_IMPROPER')
  })

  it('throws SUMMATIONS_QUADRATIC_DETERMINANT when discriminant < 0', () => {
    expect(() => newSolveQuadratic(1, 0, 1, true)).toThrow('SUMMATIONS_QUADRATIC_DETERMINANT')
  })
})

describe('parity: calculateCubicSumData', () => {
  const cases: Array<[number, number, number, number]> = [
    [0, 100, 0, 10],
    [0, 100, 100, 10],
    [0, 100, 1000, 10],
    [0, 100, 1e6, 100],
    [5, 50, 50000, 100],
    [10, 100, 0, 10],
    [10, 100, 1, 10],
    [100, 1, 1, 100],
    [50, 1000, 1e9, 200]
  ]
  it.each(cases)('init=%s base=%s spend=%s max=%s', (init, base, spend, max) => {
    const next = newCubicSumData(init, base, spend, max)
    const old = oldCubicSumData(init, base, spend, max)
    expect(next.levelCanBuy).toBe(old.levelCanBuy)
    expect(closeEnough(next.cost, old.cost)).toBe(true)
  })

  it('throws SUMMATIONS_CUBIC_SUM_NEGATIVE when totalToSpend < 0', () => {
    expect(() => newCubicSumData(0, 100, -1, 10)).toThrow('SUMMATIONS_CUBIC_SUM_NEGATIVE')
  })
})
