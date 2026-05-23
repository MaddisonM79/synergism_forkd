// Parity tests for the rune-level-bonus + OOM-increase formulas. Old bodies
// transcribed verbatim from packages/web_ui/src/Runes.ts (lines 139-302).
// Each formula is pure arithmetic over its input bundle, so sweeps cover:
// zeros (everything off), a single contribution at a time, and a "everything
// on" combined case.

import { describe, expect, it } from 'vitest'
import {
  bonusRuneLevelsDuplication as newBonusDup,
  bonusRuneLevelsInfiniteAscent as newBonusIA,
  bonusRuneLevelsSpeed as newBonusSpeed,
  duplicationRuneOOMIncrease as newDupOOM,
  firstFiveFreeLevels as newFirstFive,
  prismRuneOOMIncrease as newPrismOOM,
  speedRuneOOMIncrease as newSpeedOOM,
  superiorIntellectRuneOOMIncrease as newSIOOM,
  thriftRuneOOMIncrease as newThriftOOM
} from '../../src/mechanics/runeLevelBonuses'

// ─── Old implementations ──────────────────────────────────────────────────

const oldFirstFive = (input: { freeRunesAntUpgrade: number; constantUpgrade7: number }) =>
  input.freeRunesAntUpgrade + 7 * Math.min(input.constantUpgrade7, 1000)

const oldBonusSpeed = (input: {
  talismanBonus: number
  upgrade27: number
  coinLog1e10Floor: number
  coinLog1e50Floor: number
  upgrade29: number
  totalOwnedCoinsFirstFive: number
}) => {
  return input.talismanBonus
    + (
      input.upgrade27 * (
        Math.min(50, input.coinLog1e10Floor)
        + Math.max(0, Math.min(50, input.coinLog1e50Floor - 10))
      )
    )
    + input.upgrade29 * Math.floor(
      Math.min(100, input.totalOwnedCoinsFirstFive / 400)
    )
}

const oldBonusDup = (input: {
  talismanBonus: number
  upgrade28: number
  totalOwnedCoinsFirstFive: number
  upgrade30: number
  coinLog1e30Floor: number
  coinLog1e300Floor: number
}) => {
  return input.talismanBonus
    + input.upgrade28 * Math.min(
      100,
      Math.floor(input.totalOwnedCoinsFirstFive / 400)
    )
    + (
      input.upgrade30 * (
        Math.min(50, input.coinLog1e30Floor)
        + Math.min(50, input.coinLog1e300Floor)
      )
    )
}

const oldBonusIA = (input: {
  instantUnlock2Bonus: number
  cubeUpgrade73: number
  campaignBonusRune6: number
  talismanBonus: number
  finiteDescentBonus: number
}) =>
  input.instantUnlock2Bonus
  + input.cubeUpgrade73
  + input.campaignBonusRune6
  + input.talismanBonus
  + input.finiteDescentBonus

const oldSpeedOOM = (input: {
  upgrade66: number
  research78: number
  research111: number
  c11AscensionECC: number
  c14AscensionECC: number
  cubeUpgrade16: number
  chronosSpeedOOMBonus: number
  ambrosiaRuneOOMBonus: number
  speedRuneLevelMilestone: number
}) =>
  input.upgrade66 * 2
  + input.research78
  + input.research111
  + input.c11AscensionECC
  + 1.5 * input.c14AscensionECC
  + input.cubeUpgrade16
  + input.chronosSpeedOOMBonus
  + input.ambrosiaRuneOOMBonus
  + input.speedRuneLevelMilestone

const oldDupOOM = (input: {
  c1TranscendECC: number
  upgrade66: number
  research90: number
  research112: number
  c11AscensionECC: number
  c14AscensionECC: number
  exemptionDuplicationOOMBonus: number
  ambrosiaRuneOOMBonus: number
  duplicationRuneLevelMilestone: number
}) =>
  0.75 * input.c1TranscendECC
  + input.upgrade66 * 2
  + input.research90
  + input.research112
  + input.c11AscensionECC
  + 1.5 * input.c14AscensionECC
  + input.exemptionDuplicationOOMBonus
  + input.ambrosiaRuneOOMBonus
  + input.duplicationRuneLevelMilestone

const oldPrismOOM = (input: {
  upgrade66: number
  research79: number
  research113: number
  c11AscensionECC: number
  c14AscensionECC: number
  cubeUpgrade16: number
  mortuusPrismOOMBonus: number
  ambrosiaRuneOOMBonus: number
  prismRuneLevelMilestone: number
}) =>
  input.upgrade66 * 2
  + input.research79
  + input.research113
  + input.c11AscensionECC
  + 1.5 * input.c14AscensionECC
  + input.cubeUpgrade16
  + input.mortuusPrismOOMBonus
  + input.ambrosiaRuneOOMBonus
  + input.prismRuneLevelMilestone

const oldThriftOOM = (input: {
  upgrade66: number
  research77: number
  research114: number
  c11AscensionECC: number
  c14AscensionECC: number
  cubeUpgrade37: number
  midasThriftOOMBonus: number
  ambrosiaRuneOOMBonus: number
  thriftRuneLevelMilestone: number
}) =>
  input.upgrade66 * 2
  + input.research77
  + input.research114
  + input.c11AscensionECC
  + 1.5 * input.c14AscensionECC
  + input.cubeUpgrade37
  + input.midasThriftOOMBonus
  + input.ambrosiaRuneOOMBonus
  + input.thriftRuneLevelMilestone

const oldSIOOM = (input: {
  upgrade66: number
  research115: number
  c11AscensionECC: number
  c14AscensionECC: number
  cubeUpgrade37: number
  polymathSIOOMBonus: number
  ambrosiaRuneOOMBonus: number
  siRuneLevelMilestone: number
}) =>
  input.upgrade66 * 2
  + input.research115
  + input.c11AscensionECC
  + 1.5 * input.c14AscensionECC
  + input.cubeUpgrade37
  + input.polymathSIOOMBonus
  + input.ambrosiaRuneOOMBonus
  + input.siRuneLevelMilestone

// ─── firstFiveFreeLevels (constant + cap) ──────────────────────────────────

describe('parity: firstFiveFreeLevels', () => {
  const cases = [
    { freeRunesAntUpgrade: 0, constantUpgrade7: 0 },
    { freeRunesAntUpgrade: 5, constantUpgrade7: 100 },
    { freeRunesAntUpgrade: 0, constantUpgrade7: 1000 }, // exactly at cap
    { freeRunesAntUpgrade: 100, constantUpgrade7: 5000 }, // over cap (capped at 1000)
    { freeRunesAntUpgrade: 50, constantUpgrade7: 999 } // just under cap
  ]
  for (const input of cases) {
    it(`free=${input.freeRunesAntUpgrade} c7=${input.constantUpgrade7}`, () => {
      expect(newFirstFive(input)).toBe(oldFirstFive(input))
    })
  }
})

// ─── bonusRuneLevelsSpeed (two upgrade terms with log/floor caps) ──────────

describe('parity: bonusRuneLevelsSpeed', () => {
  const cases = [
    // All zeros
    { talismanBonus: 0, upgrade27: 0, coinLog1e10Floor: 0, coinLog1e50Floor: 0, upgrade29: 0, totalOwnedCoinsFirstFive: 0 },
    // upgrade27 only: small log → first term active, second clamped to 0
    { talismanBonus: 0, upgrade27: 1, coinLog1e10Floor: 5, coinLog1e50Floor: 5, upgrade29: 0, totalOwnedCoinsFirstFive: 0 },
    // upgrade27: 1e10-log at cap (50), 1e50-log subceiling-10 → 0 contribution from second term
    { talismanBonus: 0, upgrade27: 2, coinLog1e10Floor: 60, coinLog1e50Floor: 5, upgrade29: 0, totalOwnedCoinsFirstFive: 0 },
    // upgrade27: both terms saturated (50 + 50)
    { talismanBonus: 0, upgrade27: 3, coinLog1e10Floor: 60, coinLog1e50Floor: 70, upgrade29: 0, totalOwnedCoinsFirstFive: 0 },
    // upgrade29 only, sub-cap
    { talismanBonus: 0, upgrade27: 0, coinLog1e10Floor: 0, coinLog1e50Floor: 0, upgrade29: 5, totalOwnedCoinsFirstFive: 200 },
    // upgrade29 at cap (100)
    { talismanBonus: 0, upgrade27: 0, coinLog1e10Floor: 0, coinLog1e50Floor: 0, upgrade29: 5, totalOwnedCoinsFirstFive: 50000 },
    // talismanBonus only
    { talismanBonus: 42, upgrade27: 0, coinLog1e10Floor: 0, coinLog1e50Floor: 0, upgrade29: 0, totalOwnedCoinsFirstFive: 0 },
    // Everything on
    { talismanBonus: 17, upgrade27: 3, coinLog1e10Floor: 60, coinLog1e50Floor: 70, upgrade29: 5, totalOwnedCoinsFirstFive: 50000 }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(newBonusSpeed(input)).toBe(oldBonusSpeed(input))
    })
  }
})

// ─── bonusRuneLevelsDuplication ────────────────────────────────────────────

describe('parity: bonusRuneLevelsDuplication', () => {
  const cases = [
    { talismanBonus: 0, upgrade28: 0, totalOwnedCoinsFirstFive: 0, upgrade30: 0, coinLog1e30Floor: 0, coinLog1e300Floor: 0 },
    { talismanBonus: 0, upgrade28: 2, totalOwnedCoinsFirstFive: 200, upgrade30: 0, coinLog1e30Floor: 0, coinLog1e300Floor: 0 },
    { talismanBonus: 0, upgrade28: 2, totalOwnedCoinsFirstFive: 50000, upgrade30: 0, coinLog1e30Floor: 0, coinLog1e300Floor: 0 }, // capped at 100
    { talismanBonus: 0, upgrade28: 0, totalOwnedCoinsFirstFive: 0, upgrade30: 1, coinLog1e30Floor: 10, coinLog1e300Floor: 5 },
    { talismanBonus: 0, upgrade28: 0, totalOwnedCoinsFirstFive: 0, upgrade30: 1, coinLog1e30Floor: 60, coinLog1e300Floor: 60 }, // both capped at 50
    { talismanBonus: 99, upgrade28: 0, totalOwnedCoinsFirstFive: 0, upgrade30: 0, coinLog1e30Floor: 0, coinLog1e300Floor: 0 },
    { talismanBonus: 50, upgrade28: 3, totalOwnedCoinsFirstFive: 50000, upgrade30: 2, coinLog1e30Floor: 60, coinLog1e300Floor: 60 }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(newBonusDup(input)).toBe(oldBonusDup(input))
    })
  }
})

// ─── bonusRuneLevelsInfiniteAscent (pure 5-term sum) ───────────────────────

describe('parity: bonusRuneLevelsInfiniteAscent', () => {
  const cases = [
    { instantUnlock2Bonus: 0, cubeUpgrade73: 0, campaignBonusRune6: 0, talismanBonus: 0, finiteDescentBonus: 0 },
    { instantUnlock2Bonus: 6, cubeUpgrade73: 0, campaignBonusRune6: 0, talismanBonus: 0, finiteDescentBonus: 0 },
    { instantUnlock2Bonus: 0, cubeUpgrade73: 5, campaignBonusRune6: 0, talismanBonus: 0, finiteDescentBonus: 0 },
    { instantUnlock2Bonus: 0, cubeUpgrade73: 0, campaignBonusRune6: 10, talismanBonus: 0, finiteDescentBonus: 0 },
    { instantUnlock2Bonus: 0, cubeUpgrade73: 0, campaignBonusRune6: 0, talismanBonus: 7, finiteDescentBonus: 0 },
    { instantUnlock2Bonus: 0, cubeUpgrade73: 0, campaignBonusRune6: 0, talismanBonus: 0, finiteDescentBonus: 3 },
    { instantUnlock2Bonus: 6, cubeUpgrade73: 5, campaignBonusRune6: 10, talismanBonus: 7, finiteDescentBonus: 3 }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(newBonusIA(input)).toBe(oldBonusIA(input))
    })
  }
})

// ─── speedRuneOOMIncrease ──────────────────────────────────────────────────

describe('parity: speedRuneOOMIncrease', () => {
  const baseZero = {
    upgrade66: 0,
    research78: 0,
    research111: 0,
    c11AscensionECC: 0,
    c14AscensionECC: 0,
    cubeUpgrade16: 0,
    chronosSpeedOOMBonus: 0,
    ambrosiaRuneOOMBonus: 0,
    speedRuneLevelMilestone: 0
  }
  const cases = [
    baseZero,
    { ...baseZero, upgrade66: 5 }, // ×2 contribution
    { ...baseZero, research78: 3, research111: 7 },
    { ...baseZero, c11AscensionECC: 10, c14AscensionECC: 4 }, // c14 × 1.5
    { ...baseZero, cubeUpgrade16: 5, chronosSpeedOOMBonus: 2.5 },
    { ...baseZero, ambrosiaRuneOOMBonus: 1.25, speedRuneLevelMilestone: 8 },
    {
      upgrade66: 5, research78: 3, research111: 7, c11AscensionECC: 10, c14AscensionECC: 4,
      cubeUpgrade16: 5, chronosSpeedOOMBonus: 2.5, ambrosiaRuneOOMBonus: 1.25, speedRuneLevelMilestone: 8
    }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(newSpeedOOM(input)).toBe(oldSpeedOOM(input))
    })
  }
})

// ─── duplicationRuneOOMIncrease (extra c1Transcend term) ───────────────────

describe('parity: duplicationRuneOOMIncrease', () => {
  const baseZero = {
    c1TranscendECC: 0,
    upgrade66: 0,
    research90: 0,
    research112: 0,
    c11AscensionECC: 0,
    c14AscensionECC: 0,
    exemptionDuplicationOOMBonus: 0,
    ambrosiaRuneOOMBonus: 0,
    duplicationRuneLevelMilestone: 0
  }
  const cases = [
    baseZero,
    { ...baseZero, c1TranscendECC: 8 }, // × 0.75
    { ...baseZero, upgrade66: 5 },
    { ...baseZero, research90: 3, research112: 7 },
    { ...baseZero, c11AscensionECC: 10, c14AscensionECC: 4 },
    { ...baseZero, exemptionDuplicationOOMBonus: 2.5, ambrosiaRuneOOMBonus: 1.25, duplicationRuneLevelMilestone: 8 },
    {
      c1TranscendECC: 8, upgrade66: 5, research90: 3, research112: 7, c11AscensionECC: 10,
      c14AscensionECC: 4, exemptionDuplicationOOMBonus: 2.5, ambrosiaRuneOOMBonus: 1.25, duplicationRuneLevelMilestone: 8
    }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(newDupOOM(input)).toBe(oldDupOOM(input))
    })
  }
})

// ─── prismRuneOOMIncrease ──────────────────────────────────────────────────

describe('parity: prismRuneOOMIncrease', () => {
  const baseZero = {
    upgrade66: 0,
    research79: 0,
    research113: 0,
    c11AscensionECC: 0,
    c14AscensionECC: 0,
    cubeUpgrade16: 0,
    mortuusPrismOOMBonus: 0,
    ambrosiaRuneOOMBonus: 0,
    prismRuneLevelMilestone: 0
  }
  const cases = [
    baseZero,
    { ...baseZero, upgrade66: 5, research79: 3, research113: 7 },
    { ...baseZero, c11AscensionECC: 10, c14AscensionECC: 4, cubeUpgrade16: 5 },
    { ...baseZero, mortuusPrismOOMBonus: 2.5, ambrosiaRuneOOMBonus: 1.25, prismRuneLevelMilestone: 8 },
    {
      upgrade66: 5, research79: 3, research113: 7, c11AscensionECC: 10, c14AscensionECC: 4,
      cubeUpgrade16: 5, mortuusPrismOOMBonus: 2.5, ambrosiaRuneOOMBonus: 1.25, prismRuneLevelMilestone: 8
    }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(newPrismOOM(input)).toBe(oldPrismOOM(input))
    })
  }
})

// ─── thriftRuneOOMIncrease ─────────────────────────────────────────────────

describe('parity: thriftRuneOOMIncrease', () => {
  const baseZero = {
    upgrade66: 0,
    research77: 0,
    research114: 0,
    c11AscensionECC: 0,
    c14AscensionECC: 0,
    cubeUpgrade37: 0,
    midasThriftOOMBonus: 0,
    ambrosiaRuneOOMBonus: 0,
    thriftRuneLevelMilestone: 0
  }
  const cases = [
    baseZero,
    { ...baseZero, upgrade66: 5, research77: 3, research114: 7 },
    { ...baseZero, c11AscensionECC: 10, c14AscensionECC: 4, cubeUpgrade37: 5 },
    { ...baseZero, midasThriftOOMBonus: 2.5, ambrosiaRuneOOMBonus: 1.25, thriftRuneLevelMilestone: 8 },
    {
      upgrade66: 5, research77: 3, research114: 7, c11AscensionECC: 10, c14AscensionECC: 4,
      cubeUpgrade37: 5, midasThriftOOMBonus: 2.5, ambrosiaRuneOOMBonus: 1.25, thriftRuneLevelMilestone: 8
    }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(newThriftOOM(input)).toBe(oldThriftOOM(input))
    })
  }
})

// ─── superiorIntellectRuneOOMIncrease (no research equivalent of 78/90/79/77; just 115) ────

describe('parity: superiorIntellectRuneOOMIncrease', () => {
  const baseZero = {
    upgrade66: 0,
    research115: 0,
    c11AscensionECC: 0,
    c14AscensionECC: 0,
    cubeUpgrade37: 0,
    polymathSIOOMBonus: 0,
    ambrosiaRuneOOMBonus: 0,
    siRuneLevelMilestone: 0
  }
  const cases = [
    baseZero,
    { ...baseZero, upgrade66: 5, research115: 3 },
    { ...baseZero, c11AscensionECC: 10, c14AscensionECC: 4, cubeUpgrade37: 5 },
    { ...baseZero, polymathSIOOMBonus: 2.5, ambrosiaRuneOOMBonus: 1.25, siRuneLevelMilestone: 8 },
    {
      upgrade66: 5, research115: 3, c11AscensionECC: 10, c14AscensionECC: 4, cubeUpgrade37: 5,
      polymathSIOOMBonus: 2.5, ambrosiaRuneOOMBonus: 1.25, siRuneLevelMilestone: 8
    }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(newSIOOM(input)).toBe(oldSIOOM(input))
    })
  }
})
