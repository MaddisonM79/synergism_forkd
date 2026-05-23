// Static data + polynomial cost solvers for the research tree. Lifted from
// packages/web_ui/src/Research.ts. The unlock predicates stay in web_ui
// because they close over `player.*` and `runes.*`; the resulting per-index
// `researchData` map is composed there from these logic-provided arrays
// plus the local unlock closures.
//
// Index 0 is intentionally unused — research IDs are 1-based — so
// `researchBaseCosts[0] === POSITIVE_INFINITY` and `researchMaxLevels[0] === 0`.
// `new Decimal(POSITIVE_INFINITY)` is valid in break_infinity.js.

import { Decimal, type DecimalSource } from '../math/bignum'

// dprint-ignore
export const researchBaseCosts: DecimalSource[] = [
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
  5e8, 5e12, 5e16, 5e20, 5e24, /*ascension tier */
  1e25, 2e25, 4e25, 8e25, 1e26,
  4e26, 8e26, 1e27, 2e27, 1e28,
  5e9, 5e15, 5e21, 5e27, 1e28, /*challenge 11 tier */
  1e29, 2e29, 4e29, 8e29, 1e27,
  2e30, 4e30, 8e30, 1e31, 2e31,
  5e31, 1e32, 2e32, 4e32, 8e32, /*challenge 12 tier */
  1e33, 2e33, 4e33, 8e33, 1e34,
  3e34, 1e35, 3e35, 6e37, 1e36,
  3e36, 1e37, 3e37, 1e38, 3e38, /*challenge 13 tier */
  1e39, 3e39, 1e40, 3e40, 1e50,
  3e41, 1e42, 3e42, 6e42, 1e43,
  3e43, 1e44, 3e44, 1e45, 3e45, /*challenge 14 tier */
  2e46, 6e46, 2e47, 6e47, 1e64,
  6e48, 2e49, 1e50, 1e51, 4e56
]

// dprint-ignore
export const researchMaxLevels: DecimalSource[] = [
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

/**
 * Factory: returns a "given a budget, what's the max level I can reach"
 * function for the given polynomial degree.
 *
 * The cost from level 0 to level `n` is `baseCost * n^degree`. Inverting:
 * adding the already-paid `baseCost * currLevel^degree` back to the budget
 * gives the total cost the player could pay starting from 0, then
 * `level = (effectiveBudget / baseCost)^(1/degree)` capped at maxLevel.
 *
 * Requires `degree !== 0`; intended for positive `degree`.
 */
export function polyBuyToLevel (
  degree: number
): (budget: Decimal, baseCost: Decimal, currLevel: number, maxLevel: number) => number {
  return (budget: Decimal, baseCost: Decimal, currLevel: number, maxLevel: number) => {
    const effectiveBudget = budget.add(baseCost.times(Math.pow(currLevel, degree)))
    return Math.min(maxLevel, Decimal.floor(Decimal.pow(effectiveBudget.div(baseCost), 1 / degree)).toNumber())
  }
}

/**
 * Factory: returns a "how much does it cost to buy from currLevel to buyTo"
 * function for the given polynomial degree.
 *
 * Cost-to-buy delta: `baseCost * (buyTo^degree - currLevel^degree)`.
 * Returns 0 when `currLevel === buyTo` (avoids potential floating-point
 * noise on the identity diff).
 *
 * Requires `degree !== 0`; intended for positive `degree`.
 */
export function polyCostForLevels (
  degree: number
): (baseCost: Decimal, currLevel: number, buyTo: number) => Decimal {
  return (baseCost: Decimal, currLevel: number, buyTo: number) => {
    if (currLevel === buyTo) {
      return new Decimal(0)
    }
    return baseCost.times(Math.pow(buyTo, degree) - Math.pow(currLevel, degree))
  }
}

export interface RangeLevelAndCost {
  range: [number, number]
  level: (budget: Decimal, baseCost: Decimal, currLevel: number, maxLevel: number) => number
  cost: (baseCost: Decimal, currLevel: number, buyTo: number) => Decimal
}

/**
 * Per-index level/cost solver assignment. `polyCostForLevels(1)` implies
 * constant cost per level; `polyCostForLevels(2)` implies linear growth
 * in cost per level. Index 200 uses degree-2 (its `baseCost = 4e56` would
 * otherwise be impossibly expensive to scale linearly).
 */
export const researchLevelCostRanges: RangeLevelAndCost[] = [
  { range: [0, 199], level: polyBuyToLevel(1), cost: polyCostForLevels(1) },
  { range: [200, 200], level: polyBuyToLevel(2), cost: polyCostForLevels(2) }
]
