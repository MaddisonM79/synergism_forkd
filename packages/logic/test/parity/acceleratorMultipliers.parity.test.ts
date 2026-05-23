// Parity tests for the accelerator-boost and accelerator-multiplier
// formulas lifted from packages/web_ui/src/Calculate.ts. The old web_ui
// versions wrote to G.freeAcceleratorBoost / G.totalAcceleratorBoost /
// G.acceleratorMultiplier; the new versions return the values and let
// the web_ui shim do the G writes. The `old*` impls below transcribe the
// computation without the G side effects so the parity check is a pure
// value compare.

import { describe, expect, it } from 'vitest'
import {
  calculateAcceleratorMultiplier as newAccelMult,
  calculateTotalAcceleratorBoost as newAccelBoost
} from '../../src/mechanics/acceleratorMultipliers'
import { CalcECC } from '../../src/mechanics/challenges'

// ─── Old implementations (verbatim from packages/web_ui/src/Calculate.ts,
//     minus the G writes — they compute the same values) ─────────────────

interface OldAccelBoostArgs {
  upgrade26: number
  upgrade31: number
  totalCoinOwned: number
  achievementAccelBoosts: number
  research93: number
  sumOfRuneLevels: number
  research3: number
  challengeCompletions14: number
  research16: number
  research17: number
  research88: number
  antBuildingAcceleratorBoostMult: number
  research127: number
  research142: number
  research157: number
  research172: number
  research187: number
  research200: number
  cubeUpgrade50: number
  hepteractEffectiveAcceleratorBoost: number
  upgrade73: number
  inReincarnationChallenge: boolean
  acceleratorBoostBought: number
}

const oldAccelBoost = (a: OldAccelBoostArgs): { freeAcceleratorBoost: number; totalAcceleratorBoost: number } => {
  let b = 0
  if (a.upgrade26 > 0.5) b += 1
  if (a.upgrade31 > 0.5) b += (Math.floor(a.totalCoinOwned / 2000) * 100) / 100
  b += a.achievementAccelBoosts
  b += a.research93 * Math.floor((1 / 20) * a.sumOfRuneLevels)
  b *= 1
    + (1 / 5)
      * a.research3
      * (1 + (1 / 2) * CalcECC('ascension', a.challengeCompletions14))
  b *= 1 + (1 / 20) * a.research16 + (1 / 20) * a.research17
  b *= 1 + (1 / 20) * a.research88
  b *= a.antBuildingAcceleratorBoostMult
  b *= 1 + (1 / 100) * a.research127
  b *= 1 + (0.8 / 100) * a.research142
  b *= 1 + (0.6 / 100) * a.research157
  b *= 1 + (0.4 / 100) * a.research172
  b *= 1 + (0.2 / 100) * a.research187
  b *= 1 + (0.01 / 100) * a.research200
  b *= 1 + (0.01 / 100) * a.cubeUpgrade50
  b *= 1 + (1 / 1000) * a.hepteractEffectiveAcceleratorBoost
  if (a.upgrade73 > 0.5 && a.inReincarnationChallenge) b *= 2
  b = Math.min(1e100, Math.floor(b))
  return {
    freeAcceleratorBoost: b,
    totalAcceleratorBoost: (Math.floor(a.acceleratorBoostBought + b) * 100) / 100
  }
}

interface OldAccelMultArgs {
  research1: number
  challengeCompletions14: number
  research6: number
  research7: number
  research8: number
  research9: number
  research10: number
  research86: number
  research126: number
  research141: number
  research156: number
  research171: number
  research186: number
  research200: number
  cubeUpgrade50: number
  upgrade21: number
  upgrade22: number
  upgrade23: number
  upgrade24: number
  upgrade25: number
  upgrade50: number
  inTranscensionOrReincarnationChallenge: boolean
}

const oldAccelMult = (a: OldAccelMultArgs): number => {
  let m = 1
  m *= 1
    + (1 / 5)
      * a.research1
      * (1 + (1 / 2) * CalcECC('ascension', a.challengeCompletions14))
  m *= 1
    + (1 / 20) * a.research6
    + (1 / 25) * a.research7
    + (1 / 40) * a.research8
    + (3 / 200) * a.research9
    + (1 / 200) * a.research10
  m *= 1 + (1 / 20) * a.research86
  m *= 1 + (1 / 100) * a.research126
  m *= 1 + (0.8 / 100) * a.research141
  m *= 1 + (0.6 / 100) * a.research156
  m *= 1 + (0.4 / 100) * a.research171
  m *= 1 + (0.2 / 100) * a.research186
  m *= 1 + (0.01 / 100) * a.research200
  m *= 1 + (0.01 / 100) * a.cubeUpgrade50
  m *= Math.pow(1.01, a.upgrade21 + a.upgrade22 + a.upgrade23 + a.upgrade24 + a.upgrade25)
  if (a.inTranscensionOrReincarnationChallenge && a.upgrade50 > 0.5) m *= 1.25
  return m
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: calculateTotalAcceleratorBoost', () => {
  // Representative state slices. Bool gates × research patterns × hepteract.
  const cases: OldAccelBoostArgs[] = [
    // All zeros — baseline
    {
      upgrade26: 0, upgrade31: 0, totalCoinOwned: 0, achievementAccelBoosts: 0,
      research93: 0, sumOfRuneLevels: 0, research3: 0, challengeCompletions14: 0,
      research16: 0, research17: 0, research88: 0, antBuildingAcceleratorBoostMult: 1,
      research127: 0, research142: 0, research157: 0, research172: 0,
      research187: 0, research200: 0, cubeUpgrade50: 0,
      hepteractEffectiveAcceleratorBoost: 0, upgrade73: 0,
      inReincarnationChallenge: false, acceleratorBoostBought: 0
    },
    // upgrade26 alone
    {
      upgrade26: 1, upgrade31: 0, totalCoinOwned: 0, achievementAccelBoosts: 0,
      research93: 0, sumOfRuneLevels: 0, research3: 0, challengeCompletions14: 0,
      research16: 0, research17: 0, research88: 0, antBuildingAcceleratorBoostMult: 1,
      research127: 0, research142: 0, research157: 0, research172: 0,
      research187: 0, research200: 0, cubeUpgrade50: 0,
      hepteractEffectiveAcceleratorBoost: 0, upgrade73: 0,
      inReincarnationChallenge: false, acceleratorBoostBought: 0
    },
    // upgrade31 with totalCoinOwned just past 2000 boundary
    {
      upgrade26: 1, upgrade31: 1, totalCoinOwned: 5000, achievementAccelBoosts: 2,
      research93: 0, sumOfRuneLevels: 0, research3: 0, challengeCompletions14: 0,
      research16: 0, research17: 0, research88: 0, antBuildingAcceleratorBoostMult: 1,
      research127: 0, research142: 0, research157: 0, research172: 0,
      research187: 0, research200: 0, cubeUpgrade50: 0,
      hepteractEffectiveAcceleratorBoost: 0, upgrade73: 0,
      inReincarnationChallenge: false, acceleratorBoostBought: 7
    },
    // Mid-game: some researches + ant building mult + cc14 contribution
    {
      upgrade26: 1, upgrade31: 1, totalCoinOwned: 1e10, achievementAccelBoosts: 10,
      research93: 1, sumOfRuneLevels: 200, research3: 5, challengeCompletions14: 5,
      research16: 1, research17: 1, research88: 1, antBuildingAcceleratorBoostMult: 1.5,
      research127: 10, research142: 10, research157: 10, research172: 10,
      research187: 10, research200: 100, cubeUpgrade50: 10,
      hepteractEffectiveAcceleratorBoost: 100, upgrade73: 0,
      inReincarnationChallenge: false, acceleratorBoostBought: 50
    },
    // Reincarnation-challenge × upgrade73 doubles
    {
      upgrade26: 1, upgrade31: 1, totalCoinOwned: 1e10, achievementAccelBoosts: 10,
      research93: 1, sumOfRuneLevels: 200, research3: 5, challengeCompletions14: 5,
      research16: 1, research17: 1, research88: 1, antBuildingAcceleratorBoostMult: 1.5,
      research127: 10, research142: 10, research157: 10, research172: 10,
      research187: 10, research200: 100, cubeUpgrade50: 10,
      hepteractEffectiveAcceleratorBoost: 100, upgrade73: 1,
      inReincarnationChallenge: true, acceleratorBoostBought: 50
    },
    // Reincarnation-challenge but upgrade73 off — no doubling
    {
      upgrade26: 1, upgrade31: 1, totalCoinOwned: 1e10, achievementAccelBoosts: 10,
      research93: 1, sumOfRuneLevels: 200, research3: 5, challengeCompletions14: 5,
      research16: 1, research17: 1, research88: 1, antBuildingAcceleratorBoostMult: 1.5,
      research127: 10, research142: 10, research157: 10, research172: 10,
      research187: 10, research200: 100, cubeUpgrade50: 10,
      hepteractEffectiveAcceleratorBoost: 100, upgrade73: 0,
      inReincarnationChallenge: true, acceleratorBoostBought: 50
    },
    // Massive late-game — exercises the 1e100 floor cap
    {
      upgrade26: 1, upgrade31: 1, totalCoinOwned: 1e150, achievementAccelBoosts: 100,
      research93: 1, sumOfRuneLevels: 1e6, research3: 100, challengeCompletions14: 100,
      research16: 100, research17: 100, research88: 100, antBuildingAcceleratorBoostMult: 1e10,
      research127: 1000, research142: 1000, research157: 1000, research172: 1000,
      research187: 1000, research200: 10000, cubeUpgrade50: 1000,
      hepteractEffectiveAcceleratorBoost: 1e8, upgrade73: 1,
      inReincarnationChallenge: true, acceleratorBoostBought: 1e10
    }
  ]

  it.each(cases.map((c, i) => [i, c] as [number, OldAccelBoostArgs]))('case %i', (_i, args) => {
    const next = newAccelBoost(args)
    const old = oldAccelBoost(args)
    expect(closeEnough(next.freeAcceleratorBoost, old.freeAcceleratorBoost)).toBe(true)
    expect(closeEnough(next.totalAcceleratorBoost, old.totalAcceleratorBoost)).toBe(true)
  })
})

describe('parity: calculateAcceleratorMultiplier', () => {
  const cases: OldAccelMultArgs[] = [
    // Baseline — all zeros
    {
      research1: 0, challengeCompletions14: 0,
      research6: 0, research7: 0, research8: 0, research9: 0, research10: 0,
      research86: 0, research126: 0, research141: 0, research156: 0, research171: 0,
      research186: 0, research200: 0, cubeUpgrade50: 0,
      upgrade21: 0, upgrade22: 0, upgrade23: 0, upgrade24: 0, upgrade25: 0, upgrade50: 0,
      inTranscensionOrReincarnationChallenge: false
    },
    // Some early researches
    {
      research1: 5, challengeCompletions14: 0,
      research6: 5, research7: 5, research8: 5, research9: 5, research10: 5,
      research86: 0, research126: 0, research141: 0, research156: 0, research171: 0,
      research186: 0, research200: 0, cubeUpgrade50: 0,
      upgrade21: 1, upgrade22: 1, upgrade23: 1, upgrade24: 1, upgrade25: 1, upgrade50: 0,
      inTranscensionOrReincarnationChallenge: false
    },
    // CalcECC contribution (challengeCompletions14)
    {
      research1: 5, challengeCompletions14: 5,
      research6: 5, research7: 5, research8: 5, research9: 5, research10: 5,
      research86: 5, research126: 5, research141: 5, research156: 5, research171: 5,
      research186: 5, research200: 50, cubeUpgrade50: 10,
      upgrade21: 1, upgrade22: 1, upgrade23: 1, upgrade24: 1, upgrade25: 1, upgrade50: 1,
      inTranscensionOrReincarnationChallenge: false
    },
    // In challenge × upgrade50 → 1.25× multiplier
    {
      research1: 5, challengeCompletions14: 5,
      research6: 5, research7: 5, research8: 5, research9: 5, research10: 5,
      research86: 5, research126: 5, research141: 5, research156: 5, research171: 5,
      research186: 5, research200: 50, cubeUpgrade50: 10,
      upgrade21: 1, upgrade22: 1, upgrade23: 1, upgrade24: 1, upgrade25: 1, upgrade50: 1,
      inTranscensionOrReincarnationChallenge: true
    },
    // In challenge but upgrade50 off — no 1.25×
    {
      research1: 5, challengeCompletions14: 5,
      research6: 5, research7: 5, research8: 5, research9: 5, research10: 5,
      research86: 5, research126: 5, research141: 5, research156: 5, research171: 5,
      research186: 5, research200: 50, cubeUpgrade50: 10,
      upgrade21: 1, upgrade22: 1, upgrade23: 1, upgrade24: 1, upgrade25: 1, upgrade50: 0,
      inTranscensionOrReincarnationChallenge: true
    },
    // High-research late game
    {
      research1: 100, challengeCompletions14: 100,
      research6: 100, research7: 100, research8: 100, research9: 100, research10: 100,
      research86: 100, research126: 100, research141: 100, research156: 100, research171: 100,
      research186: 100, research200: 10000, cubeUpgrade50: 1000,
      upgrade21: 1, upgrade22: 1, upgrade23: 1, upgrade24: 1, upgrade25: 1, upgrade50: 1,
      inTranscensionOrReincarnationChallenge: true
    }
  ]

  it.each(cases.map((c, i) => [i, c] as [number, OldAccelMultArgs]))('case %i', (_i, args) => {
    expect(closeEnough(newAccelMult(args), oldAccelMult(args))).toBe(true)
  })
})
