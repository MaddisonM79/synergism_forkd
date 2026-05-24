// Parity tests for the crystal + building-power formula cluster. Old bodies
// transcribed verbatim from packages/web_ui/src/Synergism.ts lines 2661-2745.
//
// Sweeps cover representative input combinations for each formula. The
// pipelined ones (calculateBuildingPowerCoinMultiplier, calculateCrystalCoinMultiplier,
// crystalUpgrade3Base, crystalUpgrade3CrystalMultiplier) accept the pre-
// computed base/exponent as input so tests can use known values.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  calculateBuildingPower as newBuildingPower,
  calculateBuildingPowerCoinMultiplier as newBuildingPowerCoinMult,
  calculateCrystalCoinMultiplier as newCrystalCoinMult,
  calculateCrystalExponent as newCrystalExponent,
  crystalUpgrade3Base as newCrystalUpgrade3Base,
  crystalUpgrade3CrystalMultiplier as newCrystalUpgrade3CrystalMult,
  crystalUpgrade3MaxBase as newCrystalUpgrade3MaxBase,
  crystalUpgrade4MaxExponent as newCrystalUpgrade4MaxExponent
} from '../../src/mechanics/crystalAndBuildingPower'

// ─── Old implementations (verbatim) ───────────────────────────────────────

interface OldBuildingPowerInput {
  c8ReincarnationECC: number
  reincarnationShards: Decimal
  research36: number
  research37: number
  research38: number
  buildingCostScaleAntUpgradeBuildingPowerMult: number
  cubeUpgrade12: number
  cubeUpgrade36: number
  inReincarnationChallenge7: boolean
}

const oldBuildingPower = (input: OldBuildingPowerInput): number => {
  const challenge8Bonus = 0.25 * input.c8ReincarnationECC
  let power = 1
  power += (1 - Math.pow(2, -1 / 160)) * Decimal.log(input.reincarnationShards.add(1), 10)
  power += challenge8Bonus
  power *= 1 + (1 / 20) * input.research36
  power *= 1 + (1 / 40) * input.research37
  power *= 1 + (1 / 40) * input.research38
  power *= input.buildingCostScaleAntUpgradeBuildingPowerMult
  power = Math.pow(power, 1 + input.cubeUpgrade12 * 0.09)
  power = Math.pow(power, 1 + input.cubeUpgrade36 * 0.05)
  if (input.inReincarnationChallenge7) {
    power = 1 + 0.05 * power
  }
  return power
}

const oldBuildingPowerCoinMult = (buildingPower: number, totalOwnedCoin: number): Decimal =>
  Decimal.pow(buildingPower, totalOwnedCoin)

const oldCrystalUpgrade4MaxExponent = (research129: number, commonFragments: Decimal, prismCaps: number): number => {
  let exponent = 10
  exponent += 0.05 * research129 * Decimal.log(commonFragments.add(1), 4)
  exponent += prismCaps
  return exponent
}

interface OldCrystalExponentInput {
  crystalUpgrade3MaxExponent: number
  crystalUpgrade3: number
  c3TranscendECC: number
  research28: number
  research29: number
  research30: number
  cubeUpgrade17: number
}

const oldCrystalExponent = (input: OldCrystalExponentInput): number => {
  let exponent = 1 / 3
  exponent += input.crystalUpgrade3MaxExponent * (1 - Math.pow(0.995, input.crystalUpgrade3))
  exponent += 0.04 * input.c3TranscendECC
  exponent += 0.08 * input.research28
  exponent += 0.08 * input.research29
  exponent += 0.04 * input.research30
  exponent += 8 * input.cubeUpgrade17
  return exponent
}

const oldCrystalCoinMult = (prestigeShards: Decimal, crystalExponent: number): Decimal =>
  Decimal.pow(prestigeShards.add(1), crystalExponent)

const oldCrystalUpgrade3MaxBase = (upgrade122: number, research129: number, commonFragments: Decimal): number => {
  let maxBase = 2
  maxBase += upgrade122
  maxBase += 0.001 * research129 * Decimal.log(commonFragments.add(1), 4)
  return maxBase
}

const oldCrystalUpgrade3Base = (maxBase: number, crystalUpgrade2: number): number => {
  return 1 + (maxBase - 1) * (1 - Math.pow(0.999, crystalUpgrade2))
}

const oldCrystalUpgrade3CrystalMult = (base: number, crystalProducersOwned: number): Decimal =>
  Decimal.pow(base, crystalProducersOwned)

// ─── Helpers ──────────────────────────────────────────────────────────────

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)
const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// ─── calculateBuildingPower ──────────────────────────────────────────────

describe('parity: calculateBuildingPower', () => {
  const zeroInput: OldBuildingPowerInput = {
    c8ReincarnationECC: 0,
    reincarnationShards: new Decimal(0),
    research36: 0,
    research37: 0,
    research38: 0,
    buildingCostScaleAntUpgradeBuildingPowerMult: 1,
    cubeUpgrade12: 0,
    cubeUpgrade36: 0,
    inReincarnationChallenge7: false
  }
  const cases: OldBuildingPowerInput[] = [
    zeroInput,
    { ...zeroInput, c8ReincarnationECC: 10 },
    { ...zeroInput, reincarnationShards: new Decimal('1e30') },
    { ...zeroInput, research36: 20, research37: 20, research38: 20 },
    { ...zeroInput, buildingCostScaleAntUpgradeBuildingPowerMult: 2 },
    { ...zeroInput, cubeUpgrade12: 1, cubeUpgrade36: 1 },
    { ...zeroInput, inReincarnationChallenge7: true },
    // Realistic late-game bundle
    {
      c8ReincarnationECC: 30,
      reincarnationShards: new Decimal('1e100'),
      research36: 20,
      research37: 20,
      research38: 20,
      buildingCostScaleAntUpgradeBuildingPowerMult: 5,
      cubeUpgrade12: 5,
      cubeUpgrade36: 10,
      inReincarnationChallenge7: false
    },
    // Same but in challenge 7
    {
      c8ReincarnationECC: 30,
      reincarnationShards: new Decimal('1e100'),
      research36: 20,
      research37: 20,
      research38: 20,
      buildingCostScaleAntUpgradeBuildingPowerMult: 5,
      cubeUpgrade12: 5,
      cubeUpgrade36: 10,
      inReincarnationChallenge7: true
    }
  ]
  for (const input of cases) {
    it(JSON.stringify({ ...input, reincarnationShards: input.reincarnationShards.toString() }), () => {
      expect(closeEnough(newBuildingPower(input), oldBuildingPower(input))).toBe(true)
    })
  }
})

// ─── calculateBuildingPowerCoinMultiplier ────────────────────────────────

describe('parity: calculateBuildingPowerCoinMultiplier', () => {
  const cases = [
    { buildingPower: 1, totalOwnedCoin: 0 },
    { buildingPower: 1.5, totalOwnedCoin: 10 },
    { buildingPower: 2, totalOwnedCoin: 100 },
    { buildingPower: 5, totalOwnedCoin: 1000 },
    { buildingPower: 1.001, totalOwnedCoin: 100000 }
  ]
  for (const c of cases) {
    it(JSON.stringify(c), () => {
      const newRes = newBuildingPowerCoinMult(c.buildingPower, c.totalOwnedCoin)
      const oldRes = oldBuildingPowerCoinMult(c.buildingPower, c.totalOwnedCoin)
      expect(decimalEq(newRes, oldRes)).toBe(true)
    })
  }
})

// ─── crystalUpgrade4MaxExponent ──────────────────────────────────────────

describe('parity: crystalUpgrade4MaxExponent', () => {
  const cases = [
    { research129: 0, commonFragments: new Decimal(0), prismSpiritCrystalCaps: 0 },
    { research129: 0, commonFragments: new Decimal(0), prismSpiritCrystalCaps: 0.5 },
    { research129: 10, commonFragments: new Decimal('1e10'), prismSpiritCrystalCaps: 0 },
    { research129: 10, commonFragments: new Decimal('1e10'), prismSpiritCrystalCaps: 0.5 }
  ]
  for (const c of cases) {
    it(JSON.stringify({ ...c, commonFragments: c.commonFragments.toString() }), () => {
      expect(closeEnough(
        newCrystalUpgrade4MaxExponent(c),
        oldCrystalUpgrade4MaxExponent(c.research129, c.commonFragments, c.prismSpiritCrystalCaps)
      )).toBe(true)
    })
  }
})

// ─── calculateCrystalExponent ────────────────────────────────────────────

describe('parity: calculateCrystalExponent', () => {
  const zeroInput: OldCrystalExponentInput = {
    crystalUpgrade3MaxExponent: 10,
    crystalUpgrade3: 0,
    c3TranscendECC: 0,
    research28: 0,
    research29: 0,
    research30: 0,
    cubeUpgrade17: 0
  }
  const cases: OldCrystalExponentInput[] = [
    zeroInput,
    { ...zeroInput, crystalUpgrade3: 100 },
    { ...zeroInput, crystalUpgrade3: 1000 },
    { ...zeroInput, c3TranscendECC: 30 },
    { ...zeroInput, research28: 10, research29: 10, research30: 10 },
    { ...zeroInput, cubeUpgrade17: 5 },
    { crystalUpgrade3MaxExponent: 20, crystalUpgrade3: 5000, c3TranscendECC: 50, research28: 10, research29: 10, research30: 10, cubeUpgrade17: 10 }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(closeEnough(newCrystalExponent(input), oldCrystalExponent(input))).toBe(true)
    })
  }
})

// ─── calculateCrystalCoinMultiplier ──────────────────────────────────────

describe('parity: calculateCrystalCoinMultiplier', () => {
  const cases = [
    { prestigeShards: new Decimal(0), crystalExponent: 1 / 3 },
    { prestigeShards: new Decimal(100), crystalExponent: 1 },
    { prestigeShards: new Decimal('1e10'), crystalExponent: 0.5 },
    { prestigeShards: new Decimal('1e100'), crystalExponent: 5 }
  ]
  for (const c of cases) {
    it(`prestige=${c.prestigeShards.toString()} exp=${c.crystalExponent}`, () => {
      expect(decimalEq(
        newCrystalCoinMult(c.prestigeShards, c.crystalExponent),
        oldCrystalCoinMult(c.prestigeShards, c.crystalExponent)
      )).toBe(true)
    })
  }
})

// ─── crystalUpgrade3MaxBase ──────────────────────────────────────────────

describe('parity: crystalUpgrade3MaxBase', () => {
  const cases = [
    { upgrade122: 0, research129: 0, commonFragments: new Decimal(0) },
    { upgrade122: 1, research129: 0, commonFragments: new Decimal(0) },
    { upgrade122: 0, research129: 10, commonFragments: new Decimal('1e10') },
    { upgrade122: 1, research129: 20, commonFragments: new Decimal('1e50') }
  ]
  for (const c of cases) {
    it(`u122=${c.upgrade122} r129=${c.research129} cf=${c.commonFragments.toString()}`, () => {
      expect(closeEnough(
        newCrystalUpgrade3MaxBase(c),
        oldCrystalUpgrade3MaxBase(c.upgrade122, c.research129, c.commonFragments)
      )).toBe(true)
    })
  }
})

// ─── crystalUpgrade3Base ─────────────────────────────────────────────────

describe('parity: crystalUpgrade3Base', () => {
  const cases = [
    { maxBase: 2, crystalUpgrade2: 0 },
    { maxBase: 2, crystalUpgrade2: 1000 },
    { maxBase: 3, crystalUpgrade2: 10000 },
    { maxBase: 5, crystalUpgrade2: 100000 }
  ]
  for (const c of cases) {
    it(JSON.stringify(c), () => {
      expect(closeEnough(newCrystalUpgrade3Base(c), oldCrystalUpgrade3Base(c.maxBase, c.crystalUpgrade2))).toBe(true)
    })
  }
})

// ─── crystalUpgrade3CrystalMultiplier ────────────────────────────────────

describe('parity: crystalUpgrade3CrystalMultiplier', () => {
  const cases = [
    { base: 1, crystalProducersOwned: 0 },
    { base: 1.5, crystalProducersOwned: 100 },
    { base: 2, crystalProducersOwned: 500 },
    { base: 3, crystalProducersOwned: 1000 }
  ]
  for (const c of cases) {
    it(JSON.stringify(c), () => {
      expect(decimalEq(
        newCrystalUpgrade3CrystalMult(c),
        oldCrystalUpgrade3CrystalMult(c.base, c.crystalProducersOwned)
      )).toBe(true)
    })
  }
})
