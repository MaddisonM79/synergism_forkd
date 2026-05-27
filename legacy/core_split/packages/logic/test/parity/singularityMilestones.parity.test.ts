// Parity tests for the singularity-milestone bonuses lifted from
// packages/web_ui/src/Calculate.ts. Each `oldXxx` transcribes the
// pre-migration body verbatim (including the threshold-array literals as they
// were spelled out in web_ui); each test sweeps a grid that crosses every
// threshold boundary in the relevant array, plus a few in-between and
// extreme values.

import { describe, expect, it } from 'vitest'
import {
  calculateBaseGoldenQuarks as newBaseGoldenQuarks,
  calculateDilatedFiveLeafBonus as newDilatedFiveLeaf,
  calculateImmaculateAlchemyBonus as newImmaculateAlchemy,
  calculateSingularityAmbrosiaLuckMilestoneBonus as newSingAmbrosiaLuck,
  calculateSingularityQuarkMilestoneMultiplier as newSingQuarkMult,
  derpsmithCornucopiaBonus as newDerpsmith,
  inheritanceTokens as newInheritanceTokens,
  singularityBonusTokenMult as newSingularityBonusTokenMult,
  sumOfExaltCompletions as newSumOfExaltCompletions
} from '../../src/mechanics/singularityMilestones'

// ─── Old implementations (verbatim from packages/web_ui/src/Calculate.ts) ───

// dprint-ignore
const oldSingQuarkMilestoneThresholds = [
  5, 7, 10, 20, 35, 50, 65, 80, 90, 100, 121, 144, 150, 160, 166, 169, 170,
  175, 180, 190, 196, 200, 201, 202, 203, 204, 205, 210, 213, 216, 219, 225,
  228, 231, 234, 237, 240, 244, 248, 252, 256, 260, 264, 268, 272, 276, 280,
  284, 288, 290
]

const oldAmbrosiaLuckSingThresholds1 = [35, 42, 49, 56, 63, 70, 77]
const oldAmbrosiaLuckSingThresholds2 = [135, 142, 149, 156, 163, 170, 177]

const oldDerpsmithSingCounts = [
  18, 38, 58, 78, 88, 98, 118, 148, 178, 188, 198, 208, 218, 228, 238, 248
]

const oldImmaculateAlchemyThresholds = [50, 90, 130, 170, 200, 217, 235, 253, 271, 289]

const oldInheritanceLevels = [2, 5, 10, 17, 26, 37, 50, 65, 82, 101, 220, 240, 260, 270, 277]
const oldInheritanceTokenValues = [1, 10, 25, 40, 75, 100, 150, 200, 250, 300, 350, 400, 500, 600, 750]

const oldBonusTokenLevels = [41, 58, 113, 163, 229]

const oldSingQuarkMult = (singularityCount: number): number => {
  let multiplier = 1
  for (const sing of oldSingQuarkMilestoneThresholds) {
    if (singularityCount >= sing) {
      multiplier *= 1.05
    }
  }
  return multiplier
}

const oldBaseGoldenQuarks = (singularity: number, quarksThisSingularity: number, highestSingularityCount: number): number => {
  const minimumValue = 100 * Math.pow(1.04, singularity)
  const contributionFromQuarks = quarksThisSingularity / 1e5
  const firstTenBonus = 10 * Math.min(highestSingularityCount, 10)
  return Math.floor(minimumValue + contributionFromQuarks + firstTenBonus)
}

const oldSingAmbrosiaLuck = (highestSingularityCount: number): number => {
  let bonus = 0
  for (const sing of oldAmbrosiaLuckSingThresholds1) {
    if (highestSingularityCount >= sing) bonus += 5
  }
  for (const sing of oldAmbrosiaLuckSingThresholds2) {
    if (highestSingularityCount >= sing) bonus += 6
  }
  return bonus
}

const oldDilatedFiveLeaf = (highestSingularityCount: number): number => {
  const singThresholds = [100, 150, 200, 225, 250, 255, 260, 265, 269, 272]
  for (let i = 0; i < singThresholds.length; i++) {
    if (highestSingularityCount < singThresholds[i]) return i / 100
  }
  return singThresholds.length / 100
}

const oldDerpsmithCornucopia = (highestSingularityCount: number): number => {
  let counter = 0
  for (const sing of oldDerpsmithSingCounts) {
    if (highestSingularityCount >= sing) counter += 1
  }
  return 1 + (counter * highestSingularityCount) / 100
}

const oldImmaculateAlchemy = (singularityCount: number): number => {
  let bonus = 1
  for (let i = 0; i < oldImmaculateAlchemyThresholds.length; i++) {
    if (singularityCount >= oldImmaculateAlchemyThresholds[i]) bonus += 0.4
  }
  return bonus
}

const oldInheritance = (highestSingularityCount: number): number => {
  for (let i = 15; i > 0; i--) {
    if (highestSingularityCount >= oldInheritanceLevels[i]) {
      return oldInheritanceTokenValues[i]
    }
  }
  return 0
}

const oldSingularityBonusToken = (highestSingularityCount: number): number => {
  for (let i = 5; i > 0; i--) {
    if (highestSingularityCount >= oldBonusTokenLevels[i - 1]) {
      return 1 + 0.02 * i
    }
  }
  return 1
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// Build a grid that hits every threshold boundary (value, value - 1, value + 1)
// plus 0 and a high cap.
const boundaryGrid = (thresholds: number[]): number[] => {
  const out = new Set<number>([0, 1, 500])
  for (const t of thresholds) {
    out.add(Math.max(0, t - 1))
    out.add(t)
    out.add(t + 1)
  }
  return [...out].sort((a, b) => a - b)
}

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: calculateSingularityQuarkMilestoneMultiplier', () => {
  it.each(boundaryGrid(oldSingQuarkMilestoneThresholds))('singularityCount=%i', (count) => {
    expect(closeEnough(newSingQuarkMult(count), oldSingQuarkMult(count))).toBe(true)
  })
})

describe('parity: calculateBaseGoldenQuarks', () => {
  const singularityGrid = [0, 1, 5, 10, 25, 50, 100, 200, 290]
  const quarksGrid = [0, 1, 1e4, 1e5, 1e6, 1e9, 1e12]
  const highestGrid = [0, 1, 5, 10, 11, 50, 290]

  for (const singularity of singularityGrid) {
    for (const quarks of quarksGrid) {
      it.each(highestGrid)(`s=${singularity} q=${quarks} highest=%i`, (highest) => {
        const next = newBaseGoldenQuarks({
          singularity,
          quarksThisSingularity: quarks,
          highestSingularityCount: highest
        })
        const old = oldBaseGoldenQuarks(singularity, quarks, highest)
        expect(next).toBe(old)
      })
    }
  }
})

describe('parity: calculateSingularityAmbrosiaLuckMilestoneBonus', () => {
  const grid = boundaryGrid([...oldAmbrosiaLuckSingThresholds1, ...oldAmbrosiaLuckSingThresholds2])
  it.each(grid)('highest=%i', (highest) => {
    expect(newSingAmbrosiaLuck(highest)).toBe(oldSingAmbrosiaLuck(highest))
  })
})

describe('parity: calculateDilatedFiveLeafBonus', () => {
  const grid = boundaryGrid([100, 150, 200, 225, 250, 255, 260, 265, 269, 272])
  it.each(grid)('highest=%i', (highest) => {
    expect(closeEnough(newDilatedFiveLeaf(highest), oldDilatedFiveLeaf(highest))).toBe(true)
  })
})

describe('parity: derpsmithCornucopiaBonus', () => {
  const grid = boundaryGrid(oldDerpsmithSingCounts)
  it.each(grid)('highest=%i', (highest) => {
    expect(closeEnough(newDerpsmith(highest), oldDerpsmithCornucopia(highest))).toBe(true)
  })
})

describe('parity: calculateImmaculateAlchemyBonus', () => {
  const grid = boundaryGrid(oldImmaculateAlchemyThresholds)
  it.each(grid)('singularityCount=%i', (count) => {
    expect(closeEnough(newImmaculateAlchemy(count), oldImmaculateAlchemy(count))).toBe(true)
  })
})

describe('parity: inheritanceTokens', () => {
  const grid = boundaryGrid(oldInheritanceLevels)
  it.each(grid)('highest=%i', (highest) => {
    expect(newInheritanceTokens(highest)).toBe(oldInheritance(highest))
  })
})

describe('parity: singularityBonusTokenMult', () => {
  const grid = boundaryGrid(oldBonusTokenLevels)
  it.each(grid)('highest=%i', (highest) => {
    expect(closeEnough(newSingularityBonusTokenMult(highest), oldSingularityBonusToken(highest))).toBe(true)
  })
})

// ─── sumOfExaltCompletions ─────────────────────────────────────────────────

const oldSumOfExaltCompletions = (completionsList: number[]): number => {
  let sum = 0
  for (const completions of completionsList) {
    sum += completions
  }
  return sum
}

describe('parity: sumOfExaltCompletions', () => {
  const cases: [number[]][] = [
    [[]],
    [[0]],
    [[1]],
    [[1, 2, 3, 4, 5]],
    [[0, 0, 0, 0]],
    [[15, 15, 15, 15, 15, 15, 15, 15]],
    [[10, 0, 5, 0, 3, 12, 0]]
  ]
  it.each(cases)('list=%j', (list) => {
    expect(newSumOfExaltCompletions(list)).toBe(oldSumOfExaltCompletions(list))
  })
})
