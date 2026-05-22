// Parity test for the 8 calculate*PlatonicBlessing functions.
//
// Pre-migration source: packages/web_ui/src/PlatonicCubes.ts at HEAD. The OLD
// functions are transcribed below as pure helpers taking a single
// platonicBlessings field — only the read of `player.platonicBlessings.*`
// changes; arithmetic is byte-identical.

import { describe, expect, it } from 'vitest'
import {
  calculateAscensionScorePlatonicBlessing as newCalcAscensionScore,
  calculateCubeMultiplierPlatonicBlessing as newCalcCubeMult,
  calculateGlobalSpeedPlatonicBlessing as newCalcGlobalSpeed,
  calculateHypercubeBlessingMultiplierPlatonicBlessing as newCalcHypercubeBlessingMult,
  calculateHypercubeMultiplierPlatonicBlessing as newCalcHypercubeMult,
  calculatePlatonicMultiplierPlatonicBlessing as newCalcPlatonicMult,
  calculateTaxPlatonicBlessing as newCalcTax,
  calculateTesseractMultiplierPlatonicBlessing as newCalcTesseractMult
} from '../../src/mechanics/cubes/platonicBlessings'
import type { PlatonicBlessings } from '../../src/state/schema'

// ─── OLD reference impls ──────────────────────────────────────────────────

const oldCalcCubeMult = (cubes: number): number => {
  const DR = 1 / 5
  const effectPerBlessing = 2 / 4e6
  const limit = 4e6
  if (cubes < limit) return 1 + effectPerBlessing * cubes
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(cubes, DR)
}

const oldCalcTesseractMult = (tesseracts: number): number => {
  const DR = 1 / 5
  const effectPerBlessing = 1.5 / 4e6
  const limit = 4e6
  if (tesseracts < limit) return 1 + effectPerBlessing * tesseracts
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(tesseracts, DR)
}

const oldCalcHypercubeMult = (hypercubes: number): number => {
  const DR = 1 / 5
  const effectPerBlessing = 1 / 4e6
  const limit = 4e6
  if (hypercubes < limit) return 1 + effectPerBlessing * hypercubes
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(hypercubes, DR)
}

const oldCalcPlatonicMult = (platonics: number): number => {
  const DR = 1 / 5
  const effectPerBlessing = 1 / 8e4
  const limit = 8e4
  if (platonics < limit) return 1 + effectPerBlessing * platonics
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(platonics, DR)
}

const oldCalcHypercubeBlessingMult = (hypercubeBonus: number): number => {
  const DR = 1 / 16
  const effectPerBlessing = 1 / 1e4
  const limit = 1e4
  if (hypercubeBonus < limit) return 1 + effectPerBlessing * hypercubeBonus
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(hypercubeBonus, DR)
}

const oldCalcTax = (taxes: number): number => {
  const factor = Math.pow(Math.log10(1 + taxes), 1.5)
  return factor / (125 + factor)
}

const oldCalcAscensionScore = (globalSpeed: number): number => {
  const DR1 = 1 / 4
  const DR2 = 1 / 8
  const limit1 = 1e4
  const limit2 = 1e20
  const effectPerBlessing = 1 / 1e4
  if (globalSpeed < limit1) return 1 + effectPerBlessing * globalSpeed
  if (limit1 <= globalSpeed && globalSpeed < limit2) {
    const limitMult = Math.pow(limit1, 1 - DR1)
    return 1 + effectPerBlessing * limitMult * Math.pow(globalSpeed, DR1)
  }
  const limitMult1 = Math.pow(limit1, 1 - DR1)
  const limitMult2 = Math.pow(limit2, DR1 - DR2)
  return 1 + effectPerBlessing * limitMult1 * limitMult2 * Math.pow(globalSpeed, DR2)
}

const oldCalcGlobalSpeed = (globalSpeed: number): number => {
  const DR = 1 / 8
  const limit = 1e4
  const effectPerBlessing = 1 / 1e4
  if (globalSpeed < limit) return 1 + effectPerBlessing * globalSpeed
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(globalSpeed, DR)
}

// ─── Helpers ──────────────────────────────────────────────────────────────

const baseBlessings = (overrides: Partial<PlatonicBlessings> = {}): PlatonicBlessings => ({
  cubes: 0,
  tesseracts: 0,
  hypercubes: 0,
  platonics: 0,
  hypercubeBonus: 0,
  taxes: 0,
  globalSpeed: 0,
  ...overrides
})

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// Grids — sweep below, at, and above the DR transition (4e6 for cube/tess/hyper,
// 8e4 for platonics, 1e4 for hypercubeBonus / globalSpeed, all values for
// taxes since it has no break).
const generalGrid = [0, 1, 100, 1e3, 1e4, 1e6, 3.99e6, 4e6, 4.01e6, 1e7, 1e10, 1e15]
const platonicGrid = [0, 1, 100, 1e3, 7.99e4, 8e4, 8.01e4, 1e6, 1e10]
const smallLimitGrid = [0, 1, 100, 1e3, 9999, 1e4, 1.01e4, 1e6, 1e10]
const ascensionGrid = [0, 1, 100, 9999, 1e4, 1.01e4, 1e10, 1e19, 1e20, 1.01e20, 1e25, 1e40]
const taxGrid = [0, 1, 10, 100, 1e6, 1e15, 1e40]

describe('parity: PlatonicCubes blessing calculators', () => {
  describe('calculateCubeMultiplierPlatonicBlessing', () => {
    it.each(generalGrid)('cubes=%s', (cubes) => {
      expect(closeEnough(
        newCalcCubeMult(baseBlessings({ cubes })),
        oldCalcCubeMult(cubes)
      )).toBe(true)
    })
  })
  describe('calculateTesseractMultiplierPlatonicBlessing', () => {
    it.each(generalGrid)('tesseracts=%s', (tesseracts) => {
      expect(closeEnough(
        newCalcTesseractMult(baseBlessings({ tesseracts })),
        oldCalcTesseractMult(tesseracts)
      )).toBe(true)
    })
  })
  describe('calculateHypercubeMultiplierPlatonicBlessing', () => {
    it.each(generalGrid)('hypercubes=%s', (hypercubes) => {
      expect(closeEnough(
        newCalcHypercubeMult(baseBlessings({ hypercubes })),
        oldCalcHypercubeMult(hypercubes)
      )).toBe(true)
    })
  })
  describe('calculatePlatonicMultiplierPlatonicBlessing', () => {
    it.each(platonicGrid)('platonics=%s', (platonics) => {
      expect(closeEnough(
        newCalcPlatonicMult(baseBlessings({ platonics })),
        oldCalcPlatonicMult(platonics)
      )).toBe(true)
    })
  })
  describe('calculateHypercubeBlessingMultiplierPlatonicBlessing', () => {
    it.each(smallLimitGrid)('hypercubeBonus=%s', (hypercubeBonus) => {
      expect(closeEnough(
        newCalcHypercubeBlessingMult(baseBlessings({ hypercubeBonus })),
        oldCalcHypercubeBlessingMult(hypercubeBonus)
      )).toBe(true)
    })
  })
  describe('calculateTaxPlatonicBlessing', () => {
    it.each(taxGrid)('taxes=%s', (taxes) => {
      expect(closeEnough(
        newCalcTax(baseBlessings({ taxes })),
        oldCalcTax(taxes)
      )).toBe(true)
    })
  })
  describe('calculateAscensionScorePlatonicBlessing', () => {
    it.each(ascensionGrid)('globalSpeed=%s (crosses limit1=1e4 and limit2=1e20)', (globalSpeed) => {
      expect(closeEnough(
        newCalcAscensionScore(baseBlessings({ globalSpeed })),
        oldCalcAscensionScore(globalSpeed)
      )).toBe(true)
    })
  })
  describe('calculateGlobalSpeedPlatonicBlessing', () => {
    it.each(smallLimitGrid)('globalSpeed=%s', (globalSpeed) => {
      expect(closeEnough(
        newCalcGlobalSpeed(baseBlessings({ globalSpeed })),
        oldCalcGlobalSpeed(globalSpeed)
      )).toBe(true)
    })
  })
})
