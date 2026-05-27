// Parity tests for the campaign-token bonus formulas, lifted from
// packages/web_ui/src/Campaign.ts. Sweeps cover every staircase knee and
// every piecewise band boundary across the 14 bonuses.

import { describe, expect, it } from 'vitest'
import * as logic from '../../src/mechanics/campaignTokenRewards'

// ─── Old implementations (verbatim from web_ui Campaign.ts) ───────────────

const oldTimeThresholdReqs = [20, 100, 250, 500, 1000, 2000, 3500, 5000]
const oldBonusRune6ThresholdReqs = [500, 750, 1000, 1250, 1500, 1750, 2000, 3000, 4000, 6000, 8000, 10000]

const oldTutorialBonus = (t: number) => ({
  cubeBonus: 1 + 0.25 * +(t > 0),
  obtainiumBonus: 1 + 0.2 * +(t > 0),
  offeringBonus: 1 + 0.2 * +(t > 0)
})
const oldCubeBonus = (t: number) =>
  1
  + 0.4 * 1 / 25 * Math.min(t, 25)
  + 0.6 * (1 - Math.exp(-Math.max(t - 25, 0) / 500))
  + 1 * (1 - Math.exp(-Math.max(t - 2500, 0) / 5000))
const oldObtainiumBonus = (t: number) =>
  1
  + 0.1 * 1 / 25 * Math.min(t, 25)
  + 0.4 * (1 - Math.exp(-Math.max(t - 25, 0) / 500))
  + 0.5 * (1 - Math.exp(-Math.max(t - 2500, 0) / 5000))
const oldOfferingBonus = (t: number) =>
  1
  + 0.1 * 1 / 25 * Math.min(t, 25)
  + 0.4 * (1 - Math.exp(-Math.max(t - 25, 0) / 500))
  + 0.5 * (1 - Math.exp(-Math.max(t - 2500, 0) / 5000))
const oldAscensionScoreMultiplier = (t: number) =>
  1
  + 0.2 * 1 / 100 * Math.min(t, 100)
  + 0.3 * (1 - Math.exp(-Math.max(t - 100, 0) / 1000))
  + 0.5 * (1 - Math.exp(-Math.max(t - 2500, 0) / 5000))
const oldTimeThresholdReduction = (t: number) => {
  for (let i = 0; i < oldTimeThresholdReqs.length; i++) {
    if (t < oldTimeThresholdReqs[i]) return i / 4
  }
  return 2
}
const oldQuarkBonus = (t: number) => {
  if (t < 100) return 1
  return 1
    + 0.05 * Math.min(t - 100, 100) / 100
    + 0.05 * (1 - Math.exp(-Math.max(t - 200, 0) / 3000))
    + 0.1 * (1 - Math.exp(-Math.max(t - 2500, 0) / 10000))
}
const oldTaxMultiplier = (t: number) => {
  if (t < 250) return 1
  return 1
    - 0.05 * 1 / 250 * Math.min(t - 250, 250)
    - 0.15 * (1 - Math.exp(-Math.max(t - 500, 0) / 1250))
    - 0.05 * (1 - Math.exp(-Math.max(t - 4000, 0) / 5000))
}
const oldC15Bonus = (t: number) => {
  if (t < 250) return 1
  return 1
    + 0.05 * 1 / 250 * Math.min(t - 250, 250)
    + 0.95 * (1 - Math.exp(-Math.max(t - 500, 0) / 1250))
}
const oldBonusRune6 = (t: number) => {
  for (let i = 0; i < oldBonusRune6ThresholdReqs.length; i++) {
    if (t < oldBonusRune6ThresholdReqs[i]) return i
  }
  return 12
}
const oldGoldenQuarkBonus = (t: number) => {
  if (t < 500) return 1
  return 1
    + 0.05 * 1 / 500 * Math.min(t - 500, 500)
    + 0.05 * (1 - Math.exp(-Math.max(t - 1000, 0) / 2500))
}
const oldOcteractBonus = (t: number) => {
  if (t < 1000) return 1
  return 1
    + 0.1 * 1 / 1000 * Math.min(t - 1000, 1000)
    + 0.15 * (1 - Math.exp(-Math.max(t - 2000, 0) / 4000))
}
const oldAmbrosiaLuckBonus = (t: number) => {
  if (t < 2000) return 0
  return 10
    + 40 * 1 / 2000 * Math.min(t - 2000, 2000)
    + 50 * (1 - Math.exp(-Math.max(t - 4000, 0) / 2500))
}
const oldBlueberrySpeedBonus = (t: number) => {
  if (t < 2000) return 1
  return 1
    + 0.02 * 1 / 2000 * Math.min(t - 2000, 2000)
    + 0.03 * (1 - Math.exp(-Math.max(t - 4000, 0) / 2000))
}

const closeEnough = (a: number, b: number): boolean => {
  if (a === b) return true
  if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < 1e-12
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < 1e-12
}

// Comprehensive token grid: covers all the knees:
// - 0 (no tokens)
// - 25 (cube/obt/off knee)
// - 100 (asc/quark knee)
// - 200 (quark band 2)
// - 250 (tax/c15 knee)
// - 500 (tax/c15/goldenQuark knee + first timeThreshold)
// - 1000 (octeract knee + timeThreshold)
// - 2000 (ambrosia/blueberry knee + timeThreshold)
// - 2500 (cube/obt/off/asc/quark band 3)
// - 4000 (tax/ambrosia/blueberry band 3)
// - All bonusRune6 thresholds
// - Some past-everything values
const tokenGrid = [
  0,
  1,
  24,
  25,
  26,
  99,
  100,
  101,
  199,
  200,
  201,
  249,
  250,
  251,
  499,
  500,
  501,
  749,
  750,
  999,
  1000,
  1001,
  1249,
  1250,
  1499,
  1500,
  1749,
  1750,
  1999,
  2000,
  2001,
  2499,
  2500,
  2501,
  2999,
  3000,
  3499,
  3500,
  3999,
  4000,
  4001,
  4999,
  5000,
  5999,
  6000,
  7999,
  8000,
  9999,
  10000,
  10001,
  50000,
  1e6
]

// ─── Pure scalar bonuses ──────────────────────────────────────────────────

describe('parity: campaign bonus formulas', () => {
  const cases: { name: string; new: (t: number) => number; old: (t: number) => number }[] = [
    { name: 'campaignCubeBonus', new: logic.campaignCubeBonus, old: oldCubeBonus },
    { name: 'campaignObtainiumBonus', new: logic.campaignObtainiumBonus, old: oldObtainiumBonus },
    { name: 'campaignOfferingBonus', new: logic.campaignOfferingBonus, old: oldOfferingBonus },
    {
      name: 'campaignAscensionScoreMultiplier',
      new: logic.campaignAscensionScoreMultiplier,
      old: oldAscensionScoreMultiplier
    },
    {
      name: 'campaignTimeThresholdReduction',
      new: logic.campaignTimeThresholdReduction,
      old: oldTimeThresholdReduction
    },
    { name: 'campaignQuarkBonus', new: logic.campaignQuarkBonus, old: oldQuarkBonus },
    { name: 'campaignTaxMultiplier', new: logic.campaignTaxMultiplier, old: oldTaxMultiplier },
    { name: 'campaignC15Bonus', new: logic.campaignC15Bonus, old: oldC15Bonus },
    { name: 'campaignBonusRune6', new: logic.campaignBonusRune6, old: oldBonusRune6 },
    { name: 'campaignGoldenQuarkBonus', new: logic.campaignGoldenQuarkBonus, old: oldGoldenQuarkBonus },
    { name: 'campaignOcteractBonus', new: logic.campaignOcteractBonus, old: oldOcteractBonus },
    { name: 'campaignAmbrosiaLuckBonus', new: logic.campaignAmbrosiaLuckBonus, old: oldAmbrosiaLuckBonus },
    { name: 'campaignBlueberrySpeedBonus', new: logic.campaignBlueberrySpeedBonus, old: oldBlueberrySpeedBonus }
  ]
  for (const c of cases) {
    for (const t of tokenGrid) {
      it(`${c.name} tokens=${t}`, () => {
        expect(closeEnough(c.new(t), c.old(t))).toBe(true)
      })
    }
  }
})

// ─── tutorialBonus (3-field object) ───────────────────────────────────────

describe('parity: campaignTutorialBonus (3-field object)', () => {
  for (const t of [0, 1, 100, 1000, 10000]) {
    it(`tokens=${t}`, () => {
      const next = logic.tutorialBonus(t)
      const old = oldTutorialBonus(t)
      expect(closeEnough(next.cubeBonus, old.cubeBonus)).toBe(true)
      expect(closeEnough(next.obtainiumBonus, old.obtainiumBonus)).toBe(true)
      expect(closeEnough(next.offeringBonus, old.offeringBonus)).toBe(true)
    })
  }
})
