// Parity tests for gqUpgradeCostTNL, lifted from
// packages/web_ui/src/singularity.ts. Sweeps cover: each of the four
// cost-form branches (Exponential2, Cubic, Quadratic, default), the
// overcap multiplier kicking in past base maxLevel, the no-max-level
// progression at 100/400 (default branch only), and the maxed-out
// early-return.

import { describe, expect, it } from 'vitest'
import {
  gqUpgradeCostTNL as newCostTNL,
  type GQUpgradeCostTNLInput,
  type GQUpgradeSpecialCostForm
} from '../../src/mechanics/gqUpgradeCost'

// ─── Old implementation (verbatim from packages/web_ui/src/singularity.ts) ─

interface OldInput {
  level: number
  maxLevel: number
  computedMaxLevel: number
  costPerLevel: number
  specialCostForm: GQUpgradeSpecialCostForm
}

const oldCostTNL = (input: OldInput): number => {
  let costMultiplier = 1

  if (input.computedMaxLevel === input.level) {
    return 0
  }

  if (input.computedMaxLevel > input.maxLevel && input.level >= input.maxLevel) {
    costMultiplier *= Math.pow(4, input.level - input.maxLevel + 1)
  }

  if (input.specialCostForm === 'Exponential2') {
    return input.costPerLevel * Math.sqrt(costMultiplier) * Math.pow(2, input.level)
  }

  if (input.specialCostForm === 'Cubic') {
    return input.costPerLevel * costMultiplier * (Math.pow(input.level + 1, 3) - Math.pow(input.level, 3))
  }

  if (input.specialCostForm === 'Quadratic') {
    return input.costPerLevel * costMultiplier * (Math.pow(input.level + 1, 2) - Math.pow(input.level, 2))
  }

  costMultiplier *= input.maxLevel === -1 && input.level >= 100 ? input.level / 50 : 1
  costMultiplier *= input.maxLevel === -1 && input.level >= 400 ? input.level / 100 : 1

  return input.computedMaxLevel === input.level
    ? 0
    : Math.ceil(input.costPerLevel * (1 + input.level) * costMultiplier)
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// ─── Per-branch tests ─────────────────────────────────────────────────────

describe('parity: gqUpgradeCostTNL (Exponential2)', () => {
  const levelGrid = [0, 1, 5, 10, 20, 30]
  for (const level of levelGrid) {
    for (const maxLevel of [10, 100]) {
      // computedMaxLevel = maxLevel + overclock bonus
      for (const computedExtra of [0, 5, 20]) {
        const input: GQUpgradeCostTNLInput = {
          level,
          maxLevel,
          computedMaxLevel: maxLevel + computedExtra,
          costPerLevel: 100,
          specialCostForm: 'Exponential2'
        }
        it(`level=${level} maxLevel=${maxLevel} extra=${computedExtra}`, () => {
          expect(closeEnough(newCostTNL(input), oldCostTNL(input))).toBe(true)
        })
      }
    }
  }
})

describe('parity: gqUpgradeCostTNL (Cubic)', () => {
  const levelGrid = [0, 1, 5, 10, 100, 500]
  for (const level of levelGrid) {
    for (const maxLevel of [10, 100, 1000]) {
      for (const computedExtra of [0, 1, 10]) {
        const input: GQUpgradeCostTNLInput = {
          level,
          maxLevel,
          computedMaxLevel: maxLevel + computedExtra,
          costPerLevel: 50,
          specialCostForm: 'Cubic'
        }
        it(`level=${level} maxLevel=${maxLevel} extra=${computedExtra}`, () => {
          expect(closeEnough(newCostTNL(input), oldCostTNL(input))).toBe(true)
        })
      }
    }
  }
})

describe('parity: gqUpgradeCostTNL (Quadratic)', () => {
  const levelGrid = [0, 1, 5, 10, 100, 500]
  for (const level of levelGrid) {
    for (const maxLevel of [10, 100, 1000]) {
      for (const computedExtra of [0, 1, 10]) {
        const input: GQUpgradeCostTNLInput = {
          level,
          maxLevel,
          computedMaxLevel: maxLevel + computedExtra,
          costPerLevel: 50,
          specialCostForm: 'Quadratic'
        }
        it(`level=${level} maxLevel=${maxLevel} extra=${computedExtra}`, () => {
          expect(closeEnough(newCostTNL(input), oldCostTNL(input))).toBe(true)
        })
      }
    }
  }
})

describe('parity: gqUpgradeCostTNL (default branch, finite maxLevel)', () => {
  const levelGrid = [0, 1, 5, 10, 99, 100, 101, 400, 401, 500]
  for (const level of levelGrid) {
    for (const maxLevel of [1000, 5000]) {
      for (const computedExtra of [0, 5]) {
        const input: GQUpgradeCostTNLInput = {
          level,
          maxLevel,
          computedMaxLevel: maxLevel + computedExtra,
          costPerLevel: 100,
          specialCostForm: null
        }
        it(`level=${level} maxLevel=${maxLevel} extra=${computedExtra}`, () => {
          expect(closeEnough(newCostTNL(input), oldCostTNL(input))).toBe(true)
        })
      }
    }
  }
})

describe('parity: gqUpgradeCostTNL (default branch, no-max-level progression)', () => {
  // maxLevel = -1 triggers the level/50 (≥100) and level/100 (≥400)
  // multipliers. Sweep across both boundaries.
  const levelGrid = [0, 1, 50, 99, 100, 101, 200, 399, 400, 401, 800, 5000]
  for (const level of levelGrid) {
    // When maxLevel === -1, computedMaxLevel is also -1 (no overcap path
    // applies); base case has level !== computedMaxLevel.
    const input: GQUpgradeCostTNLInput = {
      level,
      maxLevel: -1,
      computedMaxLevel: -1,
      costPerLevel: 25,
      specialCostForm: null
    }
    it(`level=${level} (no-max progression)`, () => {
      expect(closeEnough(newCostTNL(input), oldCostTNL(input))).toBe(true)
    })
  }
})

describe('parity: gqUpgradeCostTNL (maxed-out → 0)', () => {
  const inputs: GQUpgradeCostTNLInput[] = [
    { level: 10, maxLevel: 10, computedMaxLevel: 10, costPerLevel: 100, specialCostForm: null },
    { level: 100, maxLevel: 50, computedMaxLevel: 100, costPerLevel: 100, specialCostForm: 'Cubic' },
    { level: 5, maxLevel: 5, computedMaxLevel: 5, costPerLevel: 100, specialCostForm: 'Exponential2' }
  ]
  for (const input of inputs) {
    it(`level=${input.level} = computedMax`, () => {
      expect(newCostTNL(input)).toBe(0)
      expect(oldCostTNL(input)).toBe(0)
    })
  }
})
