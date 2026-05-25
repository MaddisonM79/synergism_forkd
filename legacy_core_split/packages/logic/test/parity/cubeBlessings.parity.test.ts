// Parity test for the 10 calculate*CubeBlessing functions.
//
// Pre-migration source: packages/web_ui/src/Cubes.ts at HEAD. The OLD
// functions are transcribed below with `player.cubeBlessings.*` reads, the
// `calculateXTesseractBlessing()` call, and the single
// `player.cubeUpgrades[N]` read each hoisted into explicit parameters.

import { describe, expect, it } from 'vitest'
import Decimal from 'break_infinity.js'
import {
  calculateAcceleratorCubeBlessing as newCalcAccelerator,
  calculateAntELOCubeBlessing as newCalcAntELO,
  calculateAntSacrificeCubeBlessing as newCalcAntSacrifice,
  calculateAntSpeedCubeBlessing as newCalcAntSpeed,
  calculateGlobalSpeedCubeBlessing as newCalcGlobalSpeed,
  calculateMultiplierCubeBlessing as newCalcMultiplier,
  calculateObtainiumCubeBlessing as newCalcObtainium,
  calculateOfferingCubeBlessing as newCalcOffering,
  calculateRuneEffectivenessCubeBlessing as newCalcRuneEffectiveness,
  calculateSalvageCubeBlessing as newCalcSalvage
} from '../../src/mechanics/cubes/cubeBlessings'
import type { CubeBlessings } from '../../src/state/schema'

// ─── OLD reference impls ──────────────────────────────────────────────────

const oldCalcAccelerator = (count: number, tessBlessing: number, cubeUpgrade45: number): number => {
  const DR = 1 / 3
  const effectPerBlessing = tessBlessing / 500
  const limit = 1000
  const DRIncrease = cubeUpgrade45 / 300
  if (count < limit) return Math.pow(effectPerBlessing * count, 1 + DRIncrease)
  const limitMult = Math.pow(limit, 1 - DR + DRIncrease)
  return effectPerBlessing * limitMult * Math.pow(count, DR + DRIncrease)
}

const oldCalcMultiplier = (count: number, tessBlessing: number, cubeUpgrade35: number): number => {
  const DR = 1 / 3
  const effectPerBlessing = tessBlessing / 5000
  const limit = 1000
  const DRIncrease = cubeUpgrade35 / 300
  if (count < limit) return Math.pow(1 + effectPerBlessing * count, 1 + DRIncrease)
  const limitMult = Math.pow(limit, 1 - DR + DRIncrease)
  return 1 + effectPerBlessing * limitMult * Math.pow(count, DR + DRIncrease)
}

const oldCalcOffering = (count: number, tessBlessing: number, cubeUpgrade24: number): number => {
  const DR = 2 / 3
  const effectPerBlessing = new Decimal(tessBlessing).div(2000)
  const limit = 1000
  const DRIncrease = cubeUpgrade24 * 2 / 300
  if (count < limit) {
    return Decimal.min(1e300, Decimal.pow(effectPerBlessing.times(count).plus(1), 1 + DRIncrease)).toNumber()
  }
  const limitMult = Decimal.pow(limit, 1 - DR + DRIncrease)
  return Decimal.min(
    1e300,
    limitMult.times(effectPerBlessing).times(Math.pow(count, DR + DRIncrease)).plus(1)
  ).toNumber()
}

const oldCalcSalvage = (runeExp: number, tessBlessing: number, cubeUpgrade14: number): number => {
  const limit = 1000
  const effectMultiplier = (1 + cubeUpgrade14 / 100) * tessBlessing
  if (runeExp < limit) return effectMultiplier * (runeExp * 10 / limit)
  const limitBonus = 10
  return effectMultiplier * (limitBonus + 10 * Math.log10(runeExp / limit))
}

const oldCalcObtainium = (count: number, tessBlessing: number, cubeUpgrade40: number): number => {
  const DR = 2 / 3
  const effectPerBlessing = new Decimal(tessBlessing).div(2000)
  const limit = 1000
  const DRIncrease = cubeUpgrade40 * 2 / 300
  if (count < limit) {
    return Decimal.min(1e300, Decimal.pow(effectPerBlessing.times(count).plus(1), 1 + DRIncrease)).toNumber()
  }
  const limitMult = Decimal.pow(limit, 1 - DR + DRIncrease)
  return Decimal.min(
    1e300,
    limitMult.times(effectPerBlessing).times(Math.pow(count, DR + DRIncrease)).plus(1)
  ).toNumber()
}

const oldCalcAntSpeed = (count: number, tessBlessing: number, cubeUpgrade22: number): Decimal => {
  const effectPerBlessing = 1 / 1000
  const exponentIncrease = cubeUpgrade22 / 40
  const firstBonus = 0.1 * Math.min(count, 1)
  return Decimal.pow(1 + effectPerBlessing * count + firstBonus, 2 + exponentIncrease).times(tessBlessing)
}

const oldCalcAntSacrifice = (count: number, tessBlessing: number, cubeUpgrade15: number): Decimal => {
  const DR = 2 / 3
  const effectPerBlessing = tessBlessing / 5000
  const limit = 1000
  const DRIncrease = cubeUpgrade15 / 50
  if (count < limit) return Decimal.pow(1 + effectPerBlessing * count, 1 + DRIncrease)
  const limitMult = Math.pow(limit, 1 - DR + DRIncrease)
  return Decimal.pow(count, DR + DRIncrease).times(effectPerBlessing).times(limitMult).add(1)
}

const oldCalcAntELO = (antELO: number, tessBlessing: number, cubeUpgrade25: number): number => {
  const effectExponent = 1 + cubeUpgrade25 / 100
  return Math.pow(1 + 0.1 * Math.log10(1 + antELO) * tessBlessing, effectExponent)
}

const oldCalcRuneEffectiveness = (count: number, tessBlessing: number, cubeUpgrade44: number): number => {
  const DR = 1 / 16
  const effectPerBlessing = tessBlessing / 10000
  const limit = 1000
  const DRIncrease = cubeUpgrade44 / 1600
  if (count < limit) return Math.pow(1 + effectPerBlessing * count, 1 + DRIncrease)
  const limitMult = Math.pow(limit, 1 - DR + DRIncrease)
  return Math.min(1e300, 1 + limitMult * effectPerBlessing * Math.pow(count, DR + DRIncrease))
}

const oldCalcGlobalSpeed = (count: number, tessBlessing: number, cubeUpgrade34: number): number => {
  const DR = 1 / 16
  const effectPerBlessing = tessBlessing / 1000
  const limit = 1000
  const DRIncrease = cubeUpgrade34 / 1600
  if (count < limit) return Math.pow(1 + effectPerBlessing * count, 1 + DRIncrease)
  const limitMult = Math.pow(limit, 1 - DR + DRIncrease)
  return Math.min(1e300, 1 + limitMult * effectPerBlessing * Math.pow(count, DR + DRIncrease))
}

// ─── Helpers ──────────────────────────────────────────────────────────────

const baseBlessings = (overrides: Partial<CubeBlessings> = {}): CubeBlessings => ({
  accelerator: 0,
  multiplier: 0,
  offering: 0,
  runeExp: 0,
  obtainium: 0,
  antSpeed: 0,
  antSacrifice: 0,
  antELO: 0,
  talismanBonus: 0,
  globalSpeed: 0,
  ...overrides
})

const closeEnoughNum = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}
const closeEnoughDec = (a: Decimal, b: Decimal, rel = 1e-12): boolean => {
  if (a.eq(b)) return true
  if (a.abs().lt(1) && b.abs().lt(1)) return a.minus(b).abs().lt(rel)
  const diff = a.minus(b).abs()
  const scale = Decimal.max(a.abs(), b.abs())
  return diff.div(scale).lt(rel)
}

const generalGrid = [0, 1, 100, 500, 999, 1000, 1001, 1500, 1e4, 1e6]
const logOnlyGrid = [0, 1, 10, 100, 1e3, 1e6, 1e10]
const tessValues = [1, 1.5, 2, 5]
const upgradeValues = [0, 1, 10, 30] // various cubeUpgrade levels exercising DRIncrease

describe('parity: Cubes blessing calculators', () => {
  describe.each(tessValues)('tesseractBlessing=%s', (tess) => {
    describe.each(upgradeValues)('cubeUpgrade=%s', (upg) => {
      it.each(generalGrid)('calculateAcceleratorCubeBlessing(%s)', (count) => {
        expect(closeEnoughNum(
          newCalcAccelerator(baseBlessings({ accelerator: count }), tess, upg),
          oldCalcAccelerator(count, tess, upg)
        )).toBe(true)
      })
      it.each(generalGrid)('calculateMultiplierCubeBlessing(%s)', (count) => {
        expect(closeEnoughNum(
          newCalcMultiplier(baseBlessings({ multiplier: count }), tess, upg),
          oldCalcMultiplier(count, tess, upg)
        )).toBe(true)
      })
      it.each(generalGrid)('calculateOfferingCubeBlessing(%s)', (count) => {
        expect(closeEnoughNum(
          newCalcOffering(baseBlessings({ offering: count }), tess, upg),
          oldCalcOffering(count, tess, upg)
        )).toBe(true)
      })
      it.each(generalGrid)('calculateObtainiumCubeBlessing(%s)', (count) => {
        expect(closeEnoughNum(
          newCalcObtainium(baseBlessings({ obtainium: count }), tess, upg),
          oldCalcObtainium(count, tess, upg)
        )).toBe(true)
      })
      it.each(generalGrid)('calculateRuneEffectivenessCubeBlessing(%s)', (count) => {
        expect(closeEnoughNum(
          newCalcRuneEffectiveness(baseBlessings({ talismanBonus: count }), tess, upg),
          oldCalcRuneEffectiveness(count, tess, upg)
        )).toBe(true)
      })
      it.each(generalGrid)('calculateGlobalSpeedCubeBlessing(%s)', (count) => {
        expect(closeEnoughNum(
          newCalcGlobalSpeed(baseBlessings({ globalSpeed: count }), tess, upg),
          oldCalcGlobalSpeed(count, tess, upg)
        )).toBe(true)
      })
    })
  })

  describe('calculateSalvageCubeBlessing', () => {
    it.each(tessValues)('tess=%s, varied upgrade14 and runeExp', (tess) => {
      for (const upg of upgradeValues) {
        for (const runeExp of logOnlyGrid) {
          expect(closeEnoughNum(
            newCalcSalvage(baseBlessings({ runeExp }), tess, upg),
            oldCalcSalvage(runeExp, tess, upg)
          )).toBe(true)
        }
      }
    })
  })

  describe('calculateAntSpeedCubeBlessing (returns Decimal, has firstBonus quirk)', () => {
    it.each(tessValues)('tess=%s', (tess) => {
      for (const upg of upgradeValues) {
        for (const count of generalGrid) {
          expect(closeEnoughDec(
            newCalcAntSpeed(baseBlessings({ antSpeed: count }), tess, upg),
            oldCalcAntSpeed(count, tess, upg)
          )).toBe(true)
        }
      }
    })
  })

  describe('calculateAntSacrificeCubeBlessing (returns Decimal)', () => {
    it.each(tessValues)('tess=%s', (tess) => {
      for (const upg of upgradeValues) {
        for (const count of generalGrid) {
          expect(closeEnoughDec(
            newCalcAntSacrifice(baseBlessings({ antSacrifice: count }), tess, upg),
            oldCalcAntSacrifice(count, tess, upg)
          )).toBe(true)
        }
      }
    })
  })

  describe('calculateAntELOCubeBlessing', () => {
    it.each(tessValues)('tess=%s', (tess) => {
      for (const upg of upgradeValues) {
        for (const count of logOnlyGrid) {
          expect(closeEnoughNum(
            newCalcAntELO(baseBlessings({ antELO: count }), tess, upg),
            oldCalcAntELO(count, tess, upg)
          )).toBe(true)
        }
      }
    })
  })
})
