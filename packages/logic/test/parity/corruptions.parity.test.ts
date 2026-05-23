// Parity tests for the corruption pure-math migrated from
// packages/web_ui/src/Corruptions.ts. The OLD implementations are
// transcribed below; the original functions read player/G/octeract state
// directly, the migration takes those as named-field inputs.

import { describe, expect, it } from 'vitest'
import {
  calculateCorruptionDifficultyScore as newDifficultyScore,
  calculateCorruptionRawMultiplier as newRawMult,
  clipCorruptionLevel as newClipLevel,
  type CorruptionRawMultiplierInput,
  corruptionScoreMults,
  droughtEffect as newDrought,
  hyperchallengeEffect as newHyperchallenge,
  illiteracyEffect as newIlliteracy,
  type MaxCorruptionLevelInput,
  maxCorruptionLevel as newMaxLevel,
  viscosityEffect as newViscosity
} from '../../src/mechanics/corruptions'

// ─── maxCorruptionLevel ────────────────────────────────────────────────────

const oldMaxLevel = (i: MaxCorruptionLevelInput): number => {
  let max = 0
  if (i.challenge11Completions > 0) max += 5
  if (i.challenge12Completions > 0) max += 2
  if (i.challenge13Completions > 0) max += 2
  if (i.challenge14Completions > 0) max += 2
  if (i.platonicUpgrade5 > 0) max += 1
  if (i.platonicUpgrade10 > 0) max += 1
  if (i.platonicTauUnlocked) max = Math.max(13, max)
  if (i.corruptionFourteenUnlocked) max += 1
  max += i.octeractCorruptionCapIncrease
  return max
}

describe('parity: maxCorruptionLevel', () => {
  // Sweep the bit-flag-ish challenge/platonic/unlock combinations.
  // Each input is independent: testing all 2^7 × octeract-grid would be huge,
  // so pick representative configurations exercising every branch.
  const cases: MaxCorruptionLevelInput[] = [
    // All zero
    {
      challenge11Completions: 0,
      challenge12Completions: 0,
      challenge13Completions: 0,
      challenge14Completions: 0,
      platonicUpgrade5: 0,
      platonicUpgrade10: 0,
      platonicTauUnlocked: false,
      corruptionFourteenUnlocked: false,
      octeractCorruptionCapIncrease: 0
    },
    // Just challenge 11
    {
      challenge11Completions: 1,
      challenge12Completions: 0,
      challenge13Completions: 0,
      challenge14Completions: 0,
      platonicUpgrade5: 0,
      platonicUpgrade10: 0,
      platonicTauUnlocked: false,
      corruptionFourteenUnlocked: false,
      octeractCorruptionCapIncrease: 0
    },
    // All challenges (sum: 5+2+2+2 = 11)
    {
      challenge11Completions: 100,
      challenge12Completions: 50,
      challenge13Completions: 25,
      challenge14Completions: 10,
      platonicUpgrade5: 0,
      platonicUpgrade10: 0,
      platonicTauUnlocked: false,
      corruptionFourteenUnlocked: false,
      octeractCorruptionCapIncrease: 0
    },
    // All challenges + both platonics (sum: 11+2 = 13)
    {
      challenge11Completions: 1,
      challenge12Completions: 1,
      challenge13Completions: 1,
      challenge14Completions: 1,
      platonicUpgrade5: 1,
      platonicUpgrade10: 1,
      platonicTauUnlocked: false,
      corruptionFourteenUnlocked: false,
      octeractCorruptionCapIncrease: 0
    },
    // platonicTau floors at 13 when sum is less
    {
      challenge11Completions: 0,
      challenge12Completions: 0,
      challenge13Completions: 0,
      challenge14Completions: 0,
      platonicUpgrade5: 0,
      platonicUpgrade10: 0,
      platonicTauUnlocked: true,
      corruptionFourteenUnlocked: false,
      octeractCorruptionCapIncrease: 0
    },
    // platonicTau preserves higher accumulated sum
    {
      challenge11Completions: 1,
      challenge12Completions: 1,
      challenge13Completions: 1,
      challenge14Completions: 1,
      platonicUpgrade5: 1,
      platonicUpgrade10: 1,
      platonicTauUnlocked: true, // sum 13 already, no floor change
      corruptionFourteenUnlocked: false,
      octeractCorruptionCapIncrease: 0
    },
    // corruptionFourteen + octeract on top
    {
      challenge11Completions: 1,
      challenge12Completions: 1,
      challenge13Completions: 1,
      challenge14Completions: 1,
      platonicUpgrade5: 1,
      platonicUpgrade10: 1,
      platonicTauUnlocked: true,
      corruptionFourteenUnlocked: true,
      octeractCorruptionCapIncrease: 3
    },
    // octeract-only contribution
    {
      challenge11Completions: 0,
      challenge12Completions: 0,
      challenge13Completions: 0,
      challenge14Completions: 0,
      platonicUpgrade5: 0,
      platonicUpgrade10: 0,
      platonicTauUnlocked: false,
      corruptionFourteenUnlocked: false,
      octeractCorruptionCapIncrease: 5
    }
  ]

  for (const [idx, c] of cases.entries()) {
    it(`case ${idx}`, () => {
      expect(newMaxLevel(c)).toBe(oldMaxLevel(c))
    })
  }
})

// ─── viscosityEffect ───────────────────────────────────────────────────────

describe('parity: viscosityEffect', () => {
  const cases = [
    { basePower: 1, platonicUpgrade6: 0 }, // 1 * 1 = 1 → clamped
    { basePower: 0.9, platonicUpgrade6: 0 }, // 0.9
    { basePower: 0.5, platonicUpgrade6: 10 }, // 0.5 * (1 + 10/30) ≈ 0.667
    { basePower: 0.8, platonicUpgrade6: 30 }, // 0.8 * 2 = 1.6 → clamped to 1
    { basePower: 0.01, platonicUpgrade6: 0 } // tiny base
  ]
  for (const c of cases) {
    it(`base=${c.basePower} p6=${c.platonicUpgrade6}`, () => {
      const oldVal = Math.min(c.basePower * (1 + c.platonicUpgrade6 / 30), 1)
      expect(newViscosity(c)).toBe(oldVal)
    })
  }
})

// ─── droughtEffect ─────────────────────────────────────────────────────────

describe('parity: droughtEffect', () => {
  const cases = [
    { baseSalvage: 1, platonicUpgrade13: 0 },
    { baseSalvage: 0.5, platonicUpgrade13: 0 },
    { baseSalvage: 0.5, platonicUpgrade13: 1 }, // halved → 0.25
    { baseSalvage: 0.1, platonicUpgrade13: 5 } // halved → 0.05
  ]
  for (const c of cases) {
    it(`base=${c.baseSalvage} p13=${c.platonicUpgrade13}`, () => {
      const oldVal = c.platonicUpgrade13 > 0 ? c.baseSalvage * 0.5 : c.baseSalvage
      expect(newDrought(c)).toBe(oldVal)
    })
  }
})

// ─── illiteracyEffect ──────────────────────────────────────────────────────

describe('parity: illiteracyEffect', () => {
  const cases = [
    // No obtainium → multiplier stays 1
    { basePower: 0.9, platonicUpgrade9: 5, obtainiumLog10OrNull: null },
    // Below cap on log10
    { basePower: 0.8, platonicUpgrade9: 10, obtainiumLog10OrNull: 50 },
    // At cap (100)
    { basePower: 0.5, platonicUpgrade9: 100, obtainiumLog10OrNull: 100 },
    // Above cap (gets clamped to 100)
    { basePower: 0.5, platonicUpgrade9: 100, obtainiumLog10OrNull: 500 },
    // Clamps to 1 from above
    { basePower: 0.99, platonicUpgrade9: 100, obtainiumLog10OrNull: 100 }
  ]
  for (const c of cases) {
    it(`base=${c.basePower} p9=${c.platonicUpgrade9} log10=${c.obtainiumLog10OrNull}`, () => {
      const multiplier = c.obtainiumLog10OrNull === null
        ? 1
        : 1 + (1 / 100) * c.platonicUpgrade9 * Math.min(100, c.obtainiumLog10OrNull)
      const oldVal = Math.min(c.basePower * multiplier, 1)
      expect(newIlliteracy(c)).toBe(oldVal)
    })
  }
})

// ─── hyperchallengeEffect ──────────────────────────────────────────────────

describe('parity: hyperchallengeEffect', () => {
  const cases = [
    { baseEffect: 1, platonicUpgrade8: 0 }, // 1 / 1 = 1
    { baseEffect: 5, platonicUpgrade8: 0 }, // 5
    { baseEffect: 5, platonicUpgrade8: 5 }, // 5 / (1 + 2) = 1.667
    { baseEffect: 0.5, platonicUpgrade8: 10 }, // 0.5 / 5 = 0.1 → floored to 1
    { baseEffect: 100, platonicUpgrade8: 1 } // 100 / 1.4 ≈ 71.4
  ]
  for (const c of cases) {
    it(`base=${c.baseEffect} p8=${c.platonicUpgrade8}`, () => {
      const divisor = 1 + 2 / 5 * c.platonicUpgrade8
      const oldVal = Math.max(1, c.baseEffect / divisor)
      expect(newHyperchallenge(c)).toBe(oldVal)
    })
  }
})

// ─── calculateCorruptionRawMultiplier ──────────────────────────────────────
//
// Old (in-class) implementation transcribed below, using the
// corruptionScoreMults table as #corruptionScoreMults. bonusMult was always 1
// in the original code, so we drop it from the new signature.

const oldScoreMults = [1, 3, 4, 5, 6, 7, 7.75, 8.5, 9.25, 10, 10.75, 11.5, 12.25, 13, 16, 20, 25, 33, 35]

const oldRawMult = (i: CorruptionRawMultiplierInput): number => {
  const scoreMultLength = oldScoreMults.length
  if (i.totalLevel < scoreMultLength - 1) {
    const portionAboveLevel = Math.ceil(i.totalLevel) - i.totalLevel
    return Math.pow(
      oldScoreMults[Math.floor(i.totalLevel)] + i.bonusVal
        + portionAboveLevel
          * (oldScoreMults[Math.ceil(i.totalLevel)] - oldScoreMults[Math.floor(i.totalLevel)]),
      i.viscosityPower
    )
  } else {
    return Math.pow(
      (oldScoreMults[scoreMultLength - 1] + i.bonusVal)
        * Math.pow(1.2, i.totalLevel - scoreMultLength + 1),
      i.viscosityPower
    )
  }
}

describe('parity: corruptionScoreMults', () => {
  it('matches the legacy table values', () => {
    expect([...corruptionScoreMults]).toEqual(oldScoreMults)
  })
})

describe('parity: calculateCorruptionRawMultiplier', () => {
  const cases: CorruptionRawMultiplierInput[] = [
    // Integer levels below table length, no bonus, no viscosity exponent
    { totalLevel: 0, bonusVal: 0, viscosityPower: 1 },
    { totalLevel: 5, bonusVal: 0, viscosityPower: 1 },
    { totalLevel: 13, bonusVal: 0, viscosityPower: 1 },
    // Interpolation between table entries
    { totalLevel: 0.5, bonusVal: 0, viscosityPower: 1 },
    { totalLevel: 5.25, bonusVal: 0, viscosityPower: 1 },
    { totalLevel: 12.5, bonusVal: 0, viscosityPower: 1 },
    // Boundary: totalLevel = scoreMultLength - 1 (= 18) → tail branch
    { totalLevel: 17.99, bonusVal: 0, viscosityPower: 1 },
    { totalLevel: 18, bonusVal: 0, viscosityPower: 1 },
    { totalLevel: 18.5, bonusVal: 0, viscosityPower: 1 },
    // Far tail with 1.2^x extrapolation
    { totalLevel: 30, bonusVal: 0, viscosityPower: 1 },
    { totalLevel: 100, bonusVal: 0, viscosityPower: 1 },
    // bonusVal contributions
    { totalLevel: 3, bonusVal: 2.5, viscosityPower: 1 },
    { totalLevel: 12.4, bonusVal: 10, viscosityPower: 1 },
    { totalLevel: 25, bonusVal: 5, viscosityPower: 1 },
    // viscosityPower != 1 (P4x2 path)
    { totalLevel: 10, bonusVal: 0, viscosityPower: 3 },
    { totalLevel: 10, bonusVal: 0, viscosityPower: 3.4 }, // 3 + 0.04 * 10
    { totalLevel: 15.5, bonusVal: 3, viscosityPower: 3.2 },
    { totalLevel: 25, bonusVal: 7, viscosityPower: 3.8 }
  ]
  for (const [idx, c] of cases.entries()) {
    it(`case ${idx} (level=${c.totalLevel} bonus=${c.bonusVal} vp=${c.viscosityPower})`, () => {
      expect(newRawMult(c)).toBeCloseTo(oldRawMult(c), 10)
    })
  }
})

// ─── calculateCorruptionDifficultyScore ────────────────────────────────────

const oldDifficultyScore = (levels: number[]): number => {
  let basePoints = 400
  for (const lvl of levels) {
    basePoints += 16 * Math.pow(lvl, 2)
  }
  return basePoints
}

describe('parity: calculateCorruptionDifficultyScore', () => {
  const cases: number[][] = [
    [], // empty (just 400)
    [0, 0, 0, 0, 0, 0, 0, 0], // all zeros
    [1, 1, 1, 1, 1, 1, 1, 1], // each contributing 16
    [11, 11, 11, 11, 11, 11, 11, 11], // c15 loadout
    [5, 3, 7, 0, 0, 9, 12, 4], // mixed
    [100, 0, 0, 0, 0, 0, 0, 0], // one large
    [25, 13, 18, 22, 30, 8, 16, 19] // varied with bonuses applied
  ]
  for (const [idx, c] of cases.entries()) {
    it(`case ${idx} (sum=${c.reduce((a, b) => a + b, 0)})`, () => {
      expect(newDifficultyScore(c)).toBe(oldDifficultyScore(c))
    })
  }
})

// ─── clipCorruptionLevel ───────────────────────────────────────────────────
//
// Old behavior: validateNonnegativeInteger reset to 0 on non-integer/NaN/etc,
// then Math.max(0, x) then Math.min(maxLevel, x). Equivalent to:

const oldClip = (level: number, maxLevel: number): number => {
  let v = level
  if (!Number.isFinite(v) || Number.isNaN(v) || !Number.isInteger(v)) {
    v = 0
  }
  v = Math.max(0, v)
  v = Math.min(maxLevel, v)
  return v
}

describe('parity: clipCorruptionLevel', () => {
  const cases: Array<[number, number]> = [
    [0, 13],
    [5, 13],
    [13, 13],
    [14, 13], // clamps to max
    [-1, 13], // negative integer clamps to 0
    [-100, 13],
    [3.5, 13], // non-integer → 0
    [Number.NaN, 13],
    [Number.POSITIVE_INFINITY, 13],
    [Number.NEGATIVE_INFINITY, 13],
    [0, 0],
    [5, 0], // maxLevel = 0
    [11, 11], // exactly at max
    [100, 20]
  ]
  for (const [level, maxLevel] of cases) {
    it(`level=${level} maxLevel=${maxLevel}`, () => {
      expect(newClipLevel(level, maxLevel)).toBe(oldClip(level, maxLevel))
    })
  }
})
