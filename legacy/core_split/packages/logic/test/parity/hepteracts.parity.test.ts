// Parity test for the 8 hepteract EFFECTS functions.
//
// Pre-migration source: packages/web_ui/src/Hepteracts.ts at HEAD. Each OLD
// EFFECTS body is transcribed below. The only non-trivial input is the quark
// hepteract's DR exponent, which the OLD reads via `hepteracts.quark.DR +
// hepteracts.quark.DR_INCREASE()`; the new logic version takes it as a
// `drExponent` parameter and callers precompute it.

import { describe, expect, it } from 'vitest'
import {
  abyssHepteractEffects as newAbyss,
  acceleratorBoostHepteractEffects as newAcceleratorBoost,
  acceleratorHepteractEffects as newAccelerator,
  challengeHepteractEffects as newChallenge,
  chronosHepteractEffects as newChronos,
  hyperrealismHepteractEffects as newHyperrealism,
  multiplierHepteractEffects as newMultiplier,
  quarkHepteractEffects as newQuark
} from '../../src/mechanics/cubes/hepteracts'

// ─── OLD reference impls ──────────────────────────────────────────────────

const oldChronos = (hept: number) => ({ ascensionSpeed: 1 + 6 * hept / 10000 })
const oldHyperrealism = (hept: number) => ({ hypercubeMultiplier: 1 + 6 * hept / 10000 })
const oldQuark = (hept: number, drExponent: number) => ({
  quarkMultiplier: Math.pow(1 + 0.2 * Math.log2(1 + hept / 500), drExponent)
})
const oldChallenge = (hept: number) => ({ c15ScoreMultiplier: 1 + 5 * hept / 10000 })
const oldAbyss = (hept: number) => ({ salvage: 0.1 * Math.floor(10 * Math.log2(Math.max(1, hept * 2))) })
const oldAccelerator = (hept: number) => ({
  accelerators: 2000 * hept,
  acceleratorMultiplier: 1 + 3 * hept / 10000
})
const oldAcceleratorBoost = (hept: number) => ({ acceleratorBoostMultiplier: 1 + hept / 1000 })
const oldMultiplier = (hept: number) => ({
  multiplier: 1000 * hept,
  multiplierMultiplier: 1 + 3 * hept / 10000
})

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

const generalGrid = [0, 1, 10, 100, 500, 1000, 5000, 1e4, 1e6, 1e10]

describe('parity: hepteract EFFECTS', () => {
  describe('chronos', () => {
    it.each(generalGrid)('hept=%s', (hept) => {
      expect(closeEnough(newChronos(hept).ascensionSpeed, oldChronos(hept).ascensionSpeed)).toBe(true)
    })
  })

  describe('hyperrealism', () => {
    it.each(generalGrid)('hept=%s', (hept) => {
      expect(closeEnough(
        newHyperrealism(hept).hypercubeMultiplier,
        oldHyperrealism(hept).hypercubeMultiplier
      )).toBe(true)
    })
  })

  describe('quark (varied drExponent)', () => {
    const drValues = [2, 2.5, 3, 5]
    for (const dr of drValues) {
      it.each(generalGrid)(`drExponent=${dr}, hept=%s`, (hept) => {
        expect(closeEnough(
          newQuark(hept, dr).quarkMultiplier,
          oldQuark(hept, dr).quarkMultiplier
        )).toBe(true)
      })
    }
  })

  describe('challenge', () => {
    it.each(generalGrid)('hept=%s', (hept) => {
      expect(closeEnough(
        newChallenge(hept).c15ScoreMultiplier,
        oldChallenge(hept).c15ScoreMultiplier
      )).toBe(true)
    })
  })

  describe('abyss (Math.max guard at hept=0)', () => {
    it.each(generalGrid)('hept=%s', (hept) => {
      expect(closeEnough(newAbyss(hept).salvage, oldAbyss(hept).salvage)).toBe(true)
    })
  })

  describe('accelerator (two-field result)', () => {
    it.each(generalGrid)('hept=%s', (hept) => {
      expect(closeEnough(newAccelerator(hept).accelerators, oldAccelerator(hept).accelerators)).toBe(true)
      expect(closeEnough(
        newAccelerator(hept).acceleratorMultiplier,
        oldAccelerator(hept).acceleratorMultiplier
      )).toBe(true)
    })
  })

  describe('acceleratorBoost', () => {
    it.each(generalGrid)('hept=%s', (hept) => {
      expect(closeEnough(
        newAcceleratorBoost(hept).acceleratorBoostMultiplier,
        oldAcceleratorBoost(hept).acceleratorBoostMultiplier
      )).toBe(true)
    })
  })

  describe('multiplier (two-field result)', () => {
    it.each(generalGrid)('hept=%s', (hept) => {
      expect(closeEnough(newMultiplier(hept).multiplier, oldMultiplier(hept).multiplier)).toBe(true)
      expect(closeEnough(
        newMultiplier(hept).multiplierMultiplier,
        oldMultiplier(hept).multiplierMultiplier
      )).toBe(true)
    })
  })
})
