// Parity tests for the corruption pure-math migrated from
// packages/web_ui/src/Corruptions.ts. The OLD implementations are
// transcribed below; the original functions read player/G/octeract state
// directly, the migration takes those as named-field inputs.

import { describe, expect, it } from 'vitest'
import {
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
