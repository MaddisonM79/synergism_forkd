// Parity test for the 10 calculate*TesseractBlessing functions.
//
// Pre-migration source: packages/web_ui/src/Tesseracts.ts at HEAD. The OLD
// functions are transcribed below with `player.tesseractBlessings.*` reads
// hoisted to the slice parameter and each `calculateX HypercubeBlessing()`
// call hoisted to an explicit `hypercubeBlessingValue` second parameter.
// Arithmetic preserved byte-for-byte.

import { describe, expect, it } from 'vitest'
import Decimal from 'break_infinity.js'
import {
  calculateAcceleratorTesseractBlessing as newCalcAccelerator,
  calculateAntELOTesseractBlessing as newCalcAntELO,
  calculateAntSacrificeTesseractBlessing as newCalcAntSacrifice,
  calculateAntSpeedTesseractBlessing as newCalcAntSpeed,
  calculateGlobalSpeedTesseractBlessing as newCalcGlobalSpeed,
  calculateMultiplierTesseractBlessing as newCalcMultiplier,
  calculateObtainiumTesseractBlessing as newCalcObtainium,
  calculateOfferingTesseractBlessing as newCalcOffering,
  calculateRuneEffectivenessTesseractBlessing as newCalcRuneEffectiveness,
  calculateSalvageTesseractBlessing as newCalcSalvage
} from '../../src/mechanics/cubes/tesseractBlessings'
import type { TesseractBlessings } from '../../src/state/schema'

// ─── OLD reference impls ──────────────────────────────────────────────────

// Standard soft-cap+DR body — the hypercube-blessing value plays the
// `effectPerBlessing` numerator's role.
const oldSoftCap = (count: number, DR: number, hypercubeBlessing: number): number => {
  const effectPerBlessing = hypercubeBlessing / 1000
  const limit = 1000
  if (count < limit) return 1 + effectPerBlessing * count
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(count, DR)
}

const oldCalcAccelerator = (count: number, hyper: number) => oldSoftCap(count, 1 / 6, hyper)
const oldCalcMultiplier = (count: number, hyper: number) => oldSoftCap(count, 1 / 6, hyper)
const oldCalcOffering = (count: number, hyper: number) => oldSoftCap(count, 1 / 3, hyper)
const oldCalcObtainium = (count: number, hyper: number) => oldSoftCap(count, 1 / 3, hyper)
const oldCalcAntSacrifice = (count: number, hyper: number) => oldSoftCap(count, 1 / 6, hyper)
const oldCalcRuneEffectiveness = (count: number, hyper: number) => oldSoftCap(count, 1 / 32, hyper)
const oldCalcGlobalSpeed = (count: number, hyper: number) => oldSoftCap(count, 1 / 32, hyper)

const oldCalcSalvage = (runeExp: number, hyperSalvage: number): number => {
  const factor = Math.pow(Math.log10(runeExp + 1), 1.25)
  const cap = 1 / 2 * hyperSalvage
  return 1 + cap * factor / (20 + factor)
}

const oldCalcAntSpeed = (antSpeed: number, hyperAntSpeed: number): Decimal => {
  const effectPerBlessing = 1 / 1000
  return new Decimal(1 + effectPerBlessing * antSpeed).times(hyperAntSpeed)
}

const oldCalcAntELO = (antELO: number, hyperAntELO: number): number => {
  return 1 + Math.log10(antELO + 1) * hyperAntELO / 100
}

// ─── Helpers ──────────────────────────────────────────────────────────────

const baseBlessings = (overrides: Partial<TesseractBlessings> = {}): TesseractBlessings => ({
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

const generalGrid = [0, 1, 100, 500, 999, 1000, 1001, 1500, 1e4, 1e6, 1e10]
const logOnlyGrid = [0, 1, 10, 100, 1e3, 1e6, 1e10, 1e20]
const hyperValues = [1, 1.5, 2, 5]

describe('parity: Tesseracts blessing calculators', () => {
  describe.each(hyperValues)('hypercubeBlessing=%s', (hyper) => {
    it.each(generalGrid)('calculateAcceleratorTesseractBlessing(%s)', (count) => {
      expect(closeEnoughNum(
        newCalcAccelerator(baseBlessings({ accelerator: count }), hyper),
        oldCalcAccelerator(count, hyper)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateMultiplierTesseractBlessing(%s)', (count) => {
      expect(closeEnoughNum(
        newCalcMultiplier(baseBlessings({ multiplier: count }), hyper),
        oldCalcMultiplier(count, hyper)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateOfferingTesseractBlessing(%s)', (count) => {
      expect(closeEnoughNum(
        newCalcOffering(baseBlessings({ offering: count }), hyper),
        oldCalcOffering(count, hyper)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateObtainiumTesseractBlessing(%s)', (count) => {
      expect(closeEnoughNum(
        newCalcObtainium(baseBlessings({ obtainium: count }), hyper),
        oldCalcObtainium(count, hyper)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateAntSacrificeTesseractBlessing(%s)', (count) => {
      expect(closeEnoughNum(
        newCalcAntSacrifice(baseBlessings({ antSacrifice: count }), hyper),
        oldCalcAntSacrifice(count, hyper)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateRuneEffectivenessTesseractBlessing(%s)', (count) => {
      expect(closeEnoughNum(
        newCalcRuneEffectiveness(baseBlessings({ talismanBonus: count }), hyper),
        oldCalcRuneEffectiveness(count, hyper)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateGlobalSpeedTesseractBlessing(%s)', (count) => {
      expect(closeEnoughNum(
        newCalcGlobalSpeed(baseBlessings({ globalSpeed: count }), hyper),
        oldCalcGlobalSpeed(count, hyper)
      )).toBe(true)
    })
  })

  describe('calculateSalvageTesseractBlessing (log + hypercube cap)', () => {
    it.each(hyperValues)('hypercubeBlessing=%s', (hyper) => {
      for (const runeExp of logOnlyGrid) {
        expect(closeEnoughNum(
          newCalcSalvage(baseBlessings({ runeExp }), hyper),
          oldCalcSalvage(runeExp, hyper)
        )).toBe(true)
      }
    })
  })

  describe('calculateAntSpeedTesseractBlessing (linear * hypercube → Decimal)', () => {
    it.each(hyperValues)('hypercubeBlessing=%s', (hyper) => {
      for (const antSpeed of generalGrid) {
        expect(closeEnoughDec(
          newCalcAntSpeed(baseBlessings({ antSpeed }), hyper),
          oldCalcAntSpeed(antSpeed, hyper)
        )).toBe(true)
      }
    })
  })

  describe('calculateAntELOTesseractBlessing (log * hypercube / 100)', () => {
    it.each(hyperValues)('hypercubeBlessing=%s', (hyper) => {
      for (const antELO of logOnlyGrid) {
        expect(closeEnoughNum(
          newCalcAntELO(baseBlessings({ antELO }), hyper),
          oldCalcAntELO(antELO, hyper)
        )).toBe(true)
      }
    })
  })
})
