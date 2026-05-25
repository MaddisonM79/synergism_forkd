// Parity test for the 10 calculate*HypercubeBlessing functions.
//
// Pre-migration source: packages/web_ui/src/Hypercubes.ts at HEAD. The OLD
// functions are transcribed below with `player.hypercubeBlessings.*` reads
// and the `calculateHypercubeBlessingMultiplierPlatonicBlessing()` call both
// hoisted into explicit parameters. Arithmetic preserved byte-for-byte.

import { describe, expect, it } from 'vitest'
import {
  calculateAcceleratorHypercubeBlessing as newCalcAccelerator,
  calculateAntELOHypercubeBlessing as newCalcAntELO,
  calculateAntSacrificeHypercubeBlessing as newCalcAntSacrifice,
  calculateAntSpeedHypercubeBlessing as newCalcAntSpeed,
  calculateGlobalSpeedHypercubeBlessing as newCalcGlobalSpeed,
  calculateMultiplierHypercubeBlessing as newCalcMultiplier,
  calculateObtainiumHypercubeBlessing as newCalcObtainium,
  calculateOfferingHypercubeBlessing as newCalcOffering,
  calculateRuneEffectivenessHypercubeBlessing as newCalcRuneEffectiveness,
  calculateSalvageHypercubeBlessing as newCalcSalvage
} from '../../src/mechanics/cubes/hypercubeBlessings'
import type { HypercubeBlessings } from '../../src/state/schema'

// ─── OLD reference impls ──────────────────────────────────────────────────

// Generic "soft-cap with DR" shape — all 8 amplifier-using functions follow it.
const oldGeneric = (DR: number, count: number, platonicAmplifier: number): number => {
  const effectPerBlessing = platonicAmplifier / 1000
  const limit = 1000
  if (count < limit) return 1 + effectPerBlessing * count
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(count, DR)
}

const oldCalcAccelerator = (count: number, amp: number) => oldGeneric(1 / 12, count, amp)
const oldCalcMultiplier = (count: number, amp: number) => oldGeneric(1 / 12, count, amp)
const oldCalcOffering = (count: number, amp: number) => oldGeneric(1 / 6, count, amp)
const oldCalcObtainium = (count: number, amp: number) => oldGeneric(1 / 6, count, amp)
const oldCalcAntSpeed = (count: number, amp: number) => oldGeneric(1 / 2, count, amp)
const oldCalcAntSacrifice = (count: number, amp: number) => oldGeneric(1 / 12, count, amp)
const oldCalcRuneEffectiveness = (count: number, amp: number) => oldGeneric(1 / 64, count, amp)
const oldCalcGlobalSpeed = (count: number, amp: number) => oldGeneric(1 / 64, count, amp)

// The two without amplifier dependence.
const oldCalcSalvage = (runeExp: number): number => {
  const factor = Math.pow(Math.log10(runeExp + 1), 1.25)
  const cap = 3 / 2
  return 1 + cap * factor / (40 + factor)
}
const oldCalcAntELO = (antELO: number): number => 1 + Math.log10(antELO + 1) / 25

// ─── Helpers ──────────────────────────────────────────────────────────────

const baseBlessings = (overrides: Partial<HypercubeBlessings> = {}): HypercubeBlessings => ({
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

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// Limit transition for all 8 amplifier-using functions is at count=1000.
const generalGrid = [0, 1, 100, 500, 999, 1000, 1001, 1500, 1e4, 1e6, 1e10]
const logOnlyGrid = [0, 1, 10, 100, 1e3, 1e6, 1e10, 1e20]
const amplifierGrid = [1, 1.5, 2, 5]

describe('parity: Hypercubes blessing calculators', () => {
  describe.each(amplifierGrid)('platonicAmplifier=%s', (amp) => {
    it.each(generalGrid)('calculateAcceleratorHypercubeBlessing(%s)', (count) => {
      expect(closeEnough(
        newCalcAccelerator(baseBlessings({ accelerator: count }), amp),
        oldCalcAccelerator(count, amp)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateMultiplierHypercubeBlessing(%s)', (count) => {
      expect(closeEnough(
        newCalcMultiplier(baseBlessings({ multiplier: count }), amp),
        oldCalcMultiplier(count, amp)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateOfferingHypercubeBlessing(%s)', (count) => {
      expect(closeEnough(
        newCalcOffering(baseBlessings({ offering: count }), amp),
        oldCalcOffering(count, amp)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateObtainiumHypercubeBlessing(%s)', (count) => {
      expect(closeEnough(
        newCalcObtainium(baseBlessings({ obtainium: count }), amp),
        oldCalcObtainium(count, amp)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateAntSpeedHypercubeBlessing(%s)', (count) => {
      expect(closeEnough(
        newCalcAntSpeed(baseBlessings({ antSpeed: count }), amp),
        oldCalcAntSpeed(count, amp)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateAntSacrificeHypercubeBlessing(%s)', (count) => {
      expect(closeEnough(
        newCalcAntSacrifice(baseBlessings({ antSacrifice: count }), amp),
        oldCalcAntSacrifice(count, amp)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateRuneEffectivenessHypercubeBlessing(%s)', (count) => {
      expect(closeEnough(
        newCalcRuneEffectiveness(baseBlessings({ talismanBonus: count }), amp),
        oldCalcRuneEffectiveness(count, amp)
      )).toBe(true)
    })
    it.each(generalGrid)('calculateGlobalSpeedHypercubeBlessing(%s)', (count) => {
      expect(closeEnough(
        newCalcGlobalSpeed(baseBlessings({ globalSpeed: count }), amp),
        oldCalcGlobalSpeed(count, amp)
      )).toBe(true)
    })
  })

  describe('calculateSalvageHypercubeBlessing (no amplifier)', () => {
    it.each(logOnlyGrid)('runeExp=%s', (runeExp) => {
      expect(closeEnough(
        newCalcSalvage(baseBlessings({ runeExp })),
        oldCalcSalvage(runeExp)
      )).toBe(true)
    })
  })

  describe('calculateAntELOHypercubeBlessing (no amplifier)', () => {
    it.each(logOnlyGrid)('antELO=%s', (antELO) => {
      expect(closeEnough(
        newCalcAntELO(baseBlessings({ antELO })),
        oldCalcAntELO(antELO)
      )).toBe(true)
    })
  })
})
