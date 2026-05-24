// Parity tests for the ant-sacrifice reward calculators. Old bodies
// transcribed verbatim from packages/web_ui/src/Features/Ants/AntSacrifice/
// Rewards/Offerings/calculate-offerings.ts, Obtainium/calculate-obtainium.ts,
// ELO/ImmortalELO/lib/calculate.ts.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  applyTaxmanLastStandClamp as newClamp,
  calculateAntSacrificeObtainium as newObtainium,
  calculateAntSacrificeOffering as newOffering,
  calculateImmortalELOGain as newImmortal
} from '../../src/mechanics/antSacrificeRewardCalc'

const oldImmortal = (effectiveELO: number, immortalELO: number): number =>
  Math.max(0, effectiveELO - immortalELO)

const oldClamp = (
  finalReward: Decimal,
  currentResource: Decimal,
  enabled: boolean,
  completions: number
): Decimal => {
  if (enabled && completions >= 2) {
    return Decimal.min(currentResource.times(100).plus(1), finalReward)
  }
  return finalReward
}

const oldOffering = (
  antSacMult: Decimal,
  stageMult: number,
  timeMultiplier: number,
  offeringMult: Decimal,
  currentOfferings: Decimal,
  taxmanEnabled: boolean,
  taxmanCompletions: number
): Decimal => {
  const overallSacrificeMultiplier = Decimal.fromString('1').times(antSacMult).times(stageMult).times(timeMultiplier)
  const finalOfferings = offeringMult.times(overallSacrificeMultiplier)
  return (taxmanEnabled && taxmanCompletions >= 2)
    ? Decimal.min(currentOfferings.times(100).plus(1), finalOfferings)
    : finalOfferings
}

const oldObtainium = (
  antSacMult: Decimal,
  stageMult: number,
  timeMultiplier: number,
  obtainiumMult: Decimal,
  currentObtainium: Decimal,
  taxmanEnabled: boolean,
  taxmanCompletions: number
): Decimal => {
  const overallSacrificeMultiplier = Decimal.fromString('1').times(antSacMult).times(stageMult).times(timeMultiplier)
  const finalObtainium = obtainiumMult.times(overallSacrificeMultiplier)
  return (taxmanEnabled && taxmanCompletions >= 2)
    ? Decimal.min(currentObtainium.times(100).plus(1), finalObtainium)
    : finalObtainium
}

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

// ─── calculateImmortalELOGain ────────────────────────────────────────────

describe('parity: calculateImmortalELOGain', () => {
  const cases = [
    { effectiveELO: 0, immortalELO: 0 },
    { effectiveELO: 100, immortalELO: 50 },
    { effectiveELO: 100, immortalELO: 100 },
    { effectiveELO: 50, immortalELO: 100 }, // negative → clamped to 0
    { effectiveELO: 1_000_000, immortalELO: 250_000 }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(newImmortal(input)).toBe(oldImmortal(input.effectiveELO, input.immortalELO))
    })
  }
})

// ─── applyTaxmanLastStandClamp ───────────────────────────────────────────

describe('parity: applyTaxmanLastStandClamp', () => {
  const cases = [
    // Disabled — always passes through
    { final: new Decimal('1e100'), current: new Decimal('1'), enabled: false, completions: 0 },
    { final: new Decimal('1e100'), current: new Decimal('1'), enabled: false, completions: 10 },
    // Enabled but < 2 completions — passes through
    { final: new Decimal('1e100'), current: new Decimal('1'), enabled: true, completions: 0 },
    { final: new Decimal('1e100'), current: new Decimal('1'), enabled: true, completions: 1 },
    // Enabled with ≥ 2 completions — clamp engages
    { final: new Decimal('1e100'), current: new Decimal('1'), enabled: true, completions: 2 },
    { final: new Decimal('1e100'), current: new Decimal('1e50'), enabled: true, completions: 5 },
    // Final reward < clamp cap — passes through despite clamp engagement
    { final: new Decimal('10'), current: new Decimal('100'), enabled: true, completions: 2 }
  ]
  for (const c of cases) {
    it(`${c.final.toString()} clamp by ${c.current.toString()} enabled=${c.enabled} compl=${c.completions}`, () => {
      const newRes = newClamp({
        finalReward: c.final,
        currentResource: c.current,
        taxmanLastStandEnabled: c.enabled,
        taxmanLastStandCompletions: c.completions
      })
      const oldRes = oldClamp(c.final, c.current, c.enabled, c.completions)
      expect(decimalEq(newRes, oldRes)).toBe(true)
    })
  }
})

// ─── calculateAntSacrificeOffering ───────────────────────────────────────

describe('parity: calculateAntSacrificeOffering', () => {
  const cases = [
    {
      antSacMult: new Decimal(1),
      stageMult: 1,
      timeMultiplier: 1,
      offeringMult: new Decimal(100),
      currentOfferings: new Decimal(10),
      taxmanEnabled: false,
      taxmanCompletions: 0
    },
    {
      antSacMult: new Decimal('1e10'),
      stageMult: 1.05,
      timeMultiplier: 2,
      offeringMult: new Decimal('1e50'),
      currentOfferings: new Decimal('1e30'),
      taxmanEnabled: true,
      taxmanCompletions: 5
    },
    {
      antSacMult: new Decimal(100),
      stageMult: 1,
      timeMultiplier: 1,
      offeringMult: new Decimal(1000),
      currentOfferings: new Decimal(0),
      taxmanEnabled: true,
      taxmanCompletions: 2
    }
  ]
  for (const c of cases) {
    it(JSON.stringify({ ...c, antSacMult: c.antSacMult.toString(), offeringMult: c.offeringMult.toString(), currentOfferings: c.currentOfferings.toString() }), () => {
      const newRes = newOffering({
        ...c,
        taxmanLastStandEnabled: c.taxmanEnabled,
        taxmanLastStandCompletions: c.taxmanCompletions
      })
      const oldRes = oldOffering(c.antSacMult, c.stageMult, c.timeMultiplier, c.offeringMult, c.currentOfferings, c.taxmanEnabled, c.taxmanCompletions)
      expect(decimalEq(newRes, oldRes)).toBe(true)
    })
  }
})

// ─── calculateAntSacrificeObtainium ──────────────────────────────────────

describe('parity: calculateAntSacrificeObtainium', () => {
  const cases = [
    {
      antSacMult: new Decimal(1),
      stageMult: 1,
      timeMultiplier: 1,
      obtainiumMult: new Decimal(100),
      currentObtainium: new Decimal(10),
      taxmanEnabled: false,
      taxmanCompletions: 0
    },
    {
      antSacMult: new Decimal('1e10'),
      stageMult: 1.05,
      timeMultiplier: 2,
      obtainiumMult: new Decimal('1e50'),
      currentObtainium: new Decimal('1e30'),
      taxmanEnabled: true,
      taxmanCompletions: 5
    }
  ]
  for (const c of cases) {
    it(JSON.stringify({ ...c, antSacMult: c.antSacMult.toString(), obtainiumMult: c.obtainiumMult.toString(), currentObtainium: c.currentObtainium.toString() }), () => {
      const newRes = newObtainium({
        ...c,
        taxmanLastStandEnabled: c.taxmanEnabled,
        taxmanLastStandCompletions: c.taxmanCompletions
      })
      const oldRes = oldObtainium(c.antSacMult, c.stageMult, c.timeMultiplier, c.obtainiumMult, c.currentObtainium, c.taxmanEnabled, c.taxmanCompletions)
      expect(decimalEq(newRes, oldRes)).toBe(true)
    })
  }
})
