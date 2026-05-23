// Parity tests for the potion-consumed base bonuses lifted from
// packages/web_ui/src/Calculate.ts. Each old impl transcribes the
// `findInsertionIndex` binary search verbatim, then computes the
// `{amount, toNext}` pair the same way as the production code.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  calculateObtainiumPotionBaseObtainium as newObtainiumPotion,
  calculateOfferingPotionBaseOfferings as newOfferingPotion,
  calculatePotionValue as newPotionValue
} from '../../src/mechanics/potionBonuses'

// ─── Threshold tables (verbatim from old Calculate.ts) ─────────────────────

const oldOfferingPotionThresholds = [
  1,
  10,
  25,
  50,
  100,
  500,
  1000,
  10000,
  5e4,
  1e5,
  1e6,
  1e7,
  1e8,
  1e9,
  1e10,
  1e11,
  1e12,
  1e13,
  1e14,
  1e15
]

const oldObtainiumPotionThresholds = [
  1, 20, 50, 250, 1000, 20000, 4e5, 1e7, 4e8, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15
]

// ─── Old implementations (verbatim from packages/web_ui/src/Utility.ts and
//     Calculate.ts) ───

const oldFindInsertionIndex = (target: number, array: number[]): number => {
  if (array.length === 0 || target < array[0]) {
    return 0
  }
  if (target >= array[array.length - 1]) {
    return array.length
  }

  let low = 0
  let high = array.length - 1

  while (low < high) {
    const mid = Math.floor((low + high + 1) / 2)
    if (array[mid] <= target) {
      low = mid
    } else {
      high = mid - 1
    }
  }

  return low + 1
}

const oldOfferingPotion = (consumed: number) => {
  const amount = oldFindInsertionIndex(consumed, oldOfferingPotionThresholds)
  return {
    amount,
    toNext: amount < oldOfferingPotionThresholds.length
      ? oldOfferingPotionThresholds[amount] - consumed
      : Number.POSITIVE_INFINITY
  }
}

const oldObtainiumPotion = (consumed: number) => {
  const amount = oldFindInsertionIndex(consumed, oldObtainiumPotionThresholds)
  return {
    amount,
    toNext: amount < oldObtainiumPotionThresholds.length
      ? oldObtainiumPotionThresholds[amount] - consumed
      : Number.POSITIVE_INFINITY
  }
}

// Build a grid that hits every threshold boundary (value, value - 1, value + 1)
// plus 0 and a value beyond the final threshold.
const boundaryGrid = (thresholds: number[]): number[] => {
  const out = new Set<number>([0, 1e16])
  for (const t of thresholds) {
    out.add(Math.max(0, t - 1))
    out.add(t)
    out.add(t + 1)
  }
  return [...out].sort((a, b) => a - b)
}

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: calculateOfferingPotionBaseOfferings', () => {
  it.each(boundaryGrid(oldOfferingPotionThresholds))('consumed=%s', (consumed) => {
    expect(newOfferingPotion(consumed)).toEqual(oldOfferingPotion(consumed))
  })
})

describe('parity: calculateObtainiumPotionBaseObtainium', () => {
  it.each(boundaryGrid(oldObtainiumPotionThresholds))('consumed=%s', (consumed) => {
    expect(newObtainiumPotion(consumed)).toEqual(oldObtainiumPotion(consumed))
  })
})

// ─── calculatePotionValue ──────────────────────────────────────────────────

// Old impl (verbatim from packages/web_ui/src/Calculate.ts) with the i18next-
// free dependencies inlined as scalars.
const oldFastForwardResourcesGlobal = (
  resetTime: number,
  fastForwardAmount: Decimal,
  resourceMult: Decimal,
  baseResource: number,
  halfMindUnlocked: boolean,
  globalSpeedMult: number,
  resetTimeThreshold: number
): Decimal => {
  let timeMultiplier: Decimal = new Decimal('1')
  const deltaTime = fastForwardAmount.times(
    halfMindUnlocked ? 10 : globalSpeedMult
  )
  timeMultiplier = Decimal.min(
    deltaTime.times(2 * resetTime).div(resetTimeThreshold ** 2),
    deltaTime.div(resetTimeThreshold)
  )
  // NOTE: dead code in original — `.times(...)` discarded, preserved verbatim.
  timeMultiplier.times(halfMindUnlocked ? globalSpeedMult / 10 : 1)
  return Decimal.max(fastForwardAmount.times(baseResource), resourceMult.times(timeMultiplier))
}

const oldPotionValue = (
  resetTime: number,
  resourceMult: Decimal,
  baseResource: number,
  halfMindUnlocked: boolean,
  globalSpeedMult: number,
  resetTimeThreshold: number,
  potionMultipliers: number
): Decimal => {
  const potionTimeValue = new Decimal(7200)
  const fastForwardMult = oldFastForwardResourcesGlobal(
    resetTime,
    potionTimeValue,
    resourceMult,
    baseResource,
    halfMindUnlocked,
    globalSpeedMult,
    resetTimeThreshold
  )
  return fastForwardMult.times(potionMultipliers)
}

const decimalClose = (a: Decimal, b: Decimal, rel = 1e-12): boolean => {
  if (a.equals(b)) return true
  const diff = a.minus(b).abs()
  const max = Decimal.max(a.abs(), b.abs())
  if (max.lessThan(1)) return diff.lessThan(rel)
  return diff.div(max).lessThan(rel)
}

describe('parity: calculatePotionValue', () => {
  // Sweeps: halfMind branch flip, resetTime around threshold², various
  // resourceMult magnitudes, baseResource scales, and potionMultipliers
  // values typical of the early-game (~1) and stacked late-game (~10⁵).
  const halfMindGrid = [true, false]
  const resetTimeGrid = [0, 1, 60, 600, 3600, 86400]
  const globalSpeedMultGrid = [1, 5, 100, 1e6]
  const resetTimeThresholdGrid = [5, 10]
  const resourceMultGrid = [new Decimal(1), new Decimal('1e10'), new Decimal('1e100')]
  const baseResourceGrid = [0, 1, 1e6]
  const potionMultsGrid = [1, 10, 1e5]

  for (const halfMindUnlocked of halfMindGrid) {
    for (const resetTimeThreshold of resetTimeThresholdGrid) {
      for (const globalSpeedMult of globalSpeedMultGrid) {
        for (const resourceMult of resourceMultGrid) {
          for (const baseResource of baseResourceGrid) {
            for (const potionMultipliers of potionMultsGrid) {
              it.each(resetTimeGrid)(
                `halfMind=${halfMindUnlocked} thresh=${resetTimeThreshold} gsm=${globalSpeedMult} rm=${resourceMult.toString()} baseRes=${baseResource} mults=${potionMultipliers} resetTime=%s`,
                (resetTime) => {
                  const next = newPotionValue({
                    resetTime,
                    resourceMult,
                    baseResource,
                    halfMindUnlocked,
                    globalSpeedMult,
                    resetTimeThreshold,
                    potionMultipliers
                  })
                  const old = oldPotionValue(
                    resetTime,
                    resourceMult,
                    baseResource,
                    halfMindUnlocked,
                    globalSpeedMult,
                    resetTimeThreshold,
                    potionMultipliers
                  )
                  expect(decimalClose(next, old)).toBe(true)
                }
              )
            }
          }
        }
      }
    }
  }
})
