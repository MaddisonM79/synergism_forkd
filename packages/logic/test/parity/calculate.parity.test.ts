// Parity test for the post-aggregation transformations migrated from
// packages/web_ui/src/Calculate.ts. Both functions take precomputed scalars
// — the OLD versions just embed the same arithmetic — so the parity model
// transcribes the branching verbatim.

import { describe, expect, it } from 'vitest'
import Decimal from 'break_infinity.js'
import { CalcECC } from '../../src/mechanics/challenges'
import {
  calculateActualAntSpeedMult as newCalcAntSpeed,
  calculateAscensionSpeedMult as newCalcAscension,
  calculateGlobalSpeedMult as newCalcGlobal,
  calculateObtainium as newCalcObtainium,
  calculateOfferings as newCalcOfferings,
  calculatePositiveSalvage as newCalcPositiveSalvage,
  getReductionValue as newGetReduction
} from '../../src/mechanics/calculate'

const oldCalcGlobal = (normalMult: number, immaculateMult: number, drPower: number): number => {
  let n = normalMult
  if (n > 100) {
    n = Math.pow(n, 0.5) * 10
  } else if (n < 1) {
    n = Math.pow(n, drPower)
  }
  return n * immaculateMult
}

const oldCalcAscension = (base: number, exponentSpread: number): number => {
  return base < 1
    ? Math.pow(base, 1 - exponentSpread)
    : Math.pow(base, 1 + exponentSpread)
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

describe('parity: calculateGlobalSpeedMult', () => {
  // Sweep both DR thresholds: below 1 (drPower branch), at 1 (unchanged),
  // between 1 and 100 (unchanged), above 100 (sqrt*10 branch).
  const normalGrid = [0.001, 0.1, 0.5, 0.999, 1, 1.5, 10, 50, 99.9, 100, 100.1, 500, 1e6]
  const immaculateGrid = [0.5, 1, 2, 10, 1e6]
  const drPowerGrid = [0.5, 0.8, 1, 1.2]

  for (const drPower of drPowerGrid) {
    for (const immaculateMult of immaculateGrid) {
      it.each(normalGrid)(`normalMult=%s drPower=${drPower} imm=${immaculateMult}`, (normalMult) => {
        const newVal = newCalcGlobal({ normalMult, immaculateMult, drPower })
        const oldVal = oldCalcGlobal(normalMult, immaculateMult, drPower)
        expect(closeEnough(newVal, oldVal)).toBe(true)
      })
    }
  }
})

describe('parity: calculateAscensionSpeedMult', () => {
  // Sweep both branches: below 1 (base ^ (1 - spread)), at 1 (boundary —
  // `base < 1` is false so takes the upper branch), above 1.
  const baseGrid = [0.001, 0.1, 0.5, 0.99, 1, 1.5, 10, 100, 1e6, 1e15]
  const spreadGrid = [0, 0.1, 0.5, 1, 2]

  for (const spread of spreadGrid) {
    it.each(baseGrid)(`base=%s spread=${spread}`, (base) => {
      const newVal = newCalcAscension({ base, exponentSpread: spread })
      const oldVal = oldCalcAscension(base, spread)
      expect(closeEnough(newVal, oldVal)).toBe(true)
    })
  }
})

const oldCalcAntSpeed = (base: Decimal, ascensionChallenge: number, platonicUpgrade10: number): Decimal => {
  let exponent = 1
  if (ascensionChallenge === 12) exponent = 0.75
  else if (ascensionChallenge === 13) exponent = 0.23
  else if (ascensionChallenge === 14) exponent = 0.2
  else if (ascensionChallenge === 15) exponent = 0.5
  if (platonicUpgrade10 > 0 && ascensionChallenge === 15) exponent *= 1.25
  return Decimal.pow(base, exponent)
}

const closeEnoughDec = (a: Decimal, b: Decimal, rel = 1e-12): boolean => {
  if (a.eq(b)) return true
  if (a.abs().lt(1) && b.abs().lt(1)) return a.minus(b).abs().lt(rel)
  return a.minus(b).abs().div(Decimal.max(a.abs(), b.abs())).lt(rel)
}

describe('parity: calculateActualAntSpeedMult', () => {
  const bases = [new Decimal(0.5), new Decimal(1), new Decimal(1e6), new Decimal('1e100')]
  // All four penalty challenges + "no challenge" + an unrelated challenge
  // number that should map to exponent=1.
  const ascensionChallenges = [0, 7, 12, 13, 14, 15]
  const platonic10Values = [0, 1]

  for (const ac of ascensionChallenges) {
    for (const p10 of platonic10Values) {
      for (const base of bases) {
        it(`base=${base.toString()} ac=${ac} p10=${p10}`, () => {
          const newVal = newCalcAntSpeed({ base, ascensionChallenge: ac, platonicUpgrade10: p10 })
          const oldVal = oldCalcAntSpeed(base, ac, p10)
          expect(closeEnoughDec(newVal, oldVal)).toBe(true)
        })
      }
    }
  }
})

// ─── getReductionValue parity ──────────────────────────────────────────────

const oldGetReduction = (
  thriftCostDelay: number,
  researchesSum: number,
  cc4: number,
  antBuildingCostScale: number
): number => {
  let reduction = 1
  reduction += thriftCostDelay
  reduction += researchesSum / 200
  reduction += CalcECC('transcend', cc4) / 200
  reduction += antBuildingCostScale
  return reduction
}

describe('parity: getReductionValue', () => {
  const thriftGrid = [0, 0.5, 1, 5]
  const researchSumGrid = [0, 50, 250, 1000] // 0..1000 covers all 5 researches max ~200 each
  const cc4Grid = [0, 50, 100, 500, 1000, 5000]
  const antScaleGrid = [0, 0.05, 0.2, 0.5]

  for (const thrift of thriftGrid) {
    for (const sum of researchSumGrid) {
      for (const cc4 of cc4Grid) {
        for (const ant of antScaleGrid) {
          it(`thrift=${thrift} sum=${sum} cc4=${cc4} ant=${ant}`, () => {
            const newVal = newGetReduction({
              thriftCostDelay: thrift,
              researchesSum: sum,
              challengeCompletions4: cc4,
              antBuildingCostScale: ant
            })
            const oldVal = oldGetReduction(thrift, sum, cc4, ant)
            expect(closeEnough(newVal, oldVal)).toBe(true)
          })
        }
      }
    }
  }
})

// ─── calculateOfferings parity ─────────────────────────────────────────────

const oldCalcOfferings = (
  baseOfferings: number,
  timeMultiplier: number,
  offeringMult: Decimal,
  taxmanEnabled: boolean,
  taxmanCompletions: number,
  currentOfferings: Decimal
): Decimal => {
  if (taxmanEnabled && taxmanCompletions >= 2) {
    return Decimal.min(
      currentOfferings.times(100).plus(1),
      Decimal.max(baseOfferings, offeringMult.times(timeMultiplier))
    )
  }
  return Decimal.max(baseOfferings, offeringMult.times(timeMultiplier))
}

describe('parity: calculateOfferings', () => {
  const fixtures: Array<{
    label: string
    baseOfferings: number
    timeMultiplier: number
    offeringMult: Decimal
    taxmanEnabled: boolean
    taxmanCompletions: number
    currentOfferings: Decimal
  }> = [
    { label: 'plain — base wins', baseOfferings: 1e6, timeMultiplier: 1, offeringMult: new Decimal(100), taxmanEnabled: false, taxmanCompletions: 0, currentOfferings: new Decimal(0) },
    { label: 'plain — mult wins', baseOfferings: 100, timeMultiplier: 5, offeringMult: new Decimal(1e8), taxmanEnabled: false, taxmanCompletions: 0, currentOfferings: new Decimal(0) },
    { label: 'plain — Decimal beats Number floor', baseOfferings: 1e8, timeMultiplier: 10, offeringMult: new Decimal('1e100'), taxmanEnabled: false, taxmanCompletions: 0, currentOfferings: new Decimal(0) },
    { label: 'taxman <2 completions: same as plain', baseOfferings: 100, timeMultiplier: 1, offeringMult: new Decimal(1e6), taxmanEnabled: true, taxmanCompletions: 1, currentOfferings: new Decimal(1000) },
    { label: 'taxman 2+: cap bites', baseOfferings: 100, timeMultiplier: 1, offeringMult: new Decimal(1e30), taxmanEnabled: true, taxmanCompletions: 2, currentOfferings: new Decimal(50) },
    { label: 'taxman 2+: main wins under cap', baseOfferings: 1e6, timeMultiplier: 1, offeringMult: new Decimal(2e6), taxmanEnabled: true, taxmanCompletions: 2, currentOfferings: new Decimal(1e9) },
    { label: 'taxman 5 completions, large current', baseOfferings: 0, timeMultiplier: 1, offeringMult: new Decimal('1e100'), taxmanEnabled: true, taxmanCompletions: 5, currentOfferings: new Decimal(1e6) },
    { label: 'timeMultUsed=false (passed as 1)', baseOfferings: 100, timeMultiplier: 1, offeringMult: new Decimal(50), taxmanEnabled: false, taxmanCompletions: 0, currentOfferings: new Decimal(0) }
  ]

  it.each(fixtures)('$label', (f) => {
    const newVal = newCalcOfferings({
      baseOfferings: f.baseOfferings,
      timeMultiplier: f.timeMultiplier,
      offeringMult: f.offeringMult,
      taxmanLastStandEnabled: f.taxmanEnabled,
      taxmanLastStandCompletions: f.taxmanCompletions,
      currentOfferings: f.currentOfferings
    })
    const oldVal = oldCalcOfferings(
      f.baseOfferings,
      f.timeMultiplier,
      f.offeringMult,
      f.taxmanEnabled,
      f.taxmanCompletions,
      f.currentOfferings
    )
    expect(closeEnoughDec(newVal, oldVal)).toBe(true)
  })
})

// ─── calculateObtainium parity ─────────────────────────────────────────────

const oldCalcObtainium = (
  base: number,
  immaculate: number,
  DR: number,
  timeMultiplier: number,
  baseMults: Decimal,
  inC14: boolean,
  taxmanEnabled: boolean,
  taxmanCompletions: number,
  currentObtainium: Decimal
): Decimal => {
  if (inC14) return new Decimal('0')
  const total = new Decimal(immaculate).times(Decimal.pow(baseMults, DR)).times(timeMultiplier)
  if (taxmanEnabled && taxmanCompletions >= 2) {
    return Decimal.min(currentObtainium.times(100).plus(1), Decimal.max(base, total))
  }
  return Decimal.max(base, total)
}

describe('parity: calculateObtainium', () => {
  const fixtures: Array<{
    label: string
    base: number
    immaculate: number
    DR: number
    timeMultiplier: number
    baseMults: Decimal
    inC14: boolean
    taxmanEnabled: boolean
    taxmanCompletions: number
    currentObtainium: Decimal
  }> = [
    { label: 'C14 short-circuits to 0', base: 1e9, immaculate: 100, DR: 1, timeMultiplier: 5, baseMults: new Decimal(1e10), inC14: true, taxmanEnabled: false, taxmanCompletions: 0, currentObtainium: new Decimal(0) },
    { label: 'normal — total wins over base', base: 100, immaculate: 1, DR: 1, timeMultiplier: 1, baseMults: new Decimal(1e6), inC14: false, taxmanEnabled: false, taxmanCompletions: 0, currentObtainium: new Decimal(0) },
    { label: 'normal — base floor wins', base: 1e10, immaculate: 1, DR: 1, timeMultiplier: 1, baseMults: new Decimal(100), inC14: false, taxmanEnabled: false, taxmanCompletions: 0, currentObtainium: new Decimal(0) },
    { label: 'DR < 1 damps baseMults', base: 0, immaculate: 1, DR: 0.5, timeMultiplier: 1, baseMults: new Decimal(1e20), inC14: false, taxmanEnabled: false, taxmanCompletions: 0, currentObtainium: new Decimal(0) },
    { label: 'DR = 0 means baseMults^0 = 1', base: 0, immaculate: 5, DR: 0, timeMultiplier: 2, baseMults: new Decimal('1e1000'), inC14: false, taxmanEnabled: false, taxmanCompletions: 0, currentObtainium: new Decimal(0) },
    { label: 'taxman <2 completions: same as plain', base: 100, immaculate: 1, DR: 1, timeMultiplier: 1, baseMults: new Decimal(1e6), inC14: false, taxmanEnabled: true, taxmanCompletions: 1, currentObtainium: new Decimal(1e4) },
    { label: 'taxman 2+: cap bites at small currentObtainium', base: 100, immaculate: 1, DR: 1, timeMultiplier: 1, baseMults: new Decimal(1e30), inC14: false, taxmanEnabled: true, taxmanCompletions: 2, currentObtainium: new Decimal(10) },
    { label: 'taxman 2+: main wins under cap', base: 100, immaculate: 1, DR: 1, timeMultiplier: 1, baseMults: new Decimal(1e6), inC14: false, taxmanEnabled: true, taxmanCompletions: 2, currentObtainium: new Decimal(1e9) },
    { label: 'huge Decimal baseMults * timeMult', base: 0, immaculate: 1, DR: 1, timeMultiplier: 100, baseMults: new Decimal('1e500'), inC14: false, taxmanEnabled: false, taxmanCompletions: 0, currentObtainium: new Decimal(0) }
  ]

  it.each(fixtures)('$label', (f) => {
    const newVal = newCalcObtainium({
      baseObtainium: f.base,
      immaculate: f.immaculate,
      DR: f.DR,
      timeMultiplier: f.timeMultiplier,
      baseMults: f.baseMults,
      inAscensionChallenge14: f.inC14,
      taxmanLastStandEnabled: f.taxmanEnabled,
      taxmanLastStandCompletions: f.taxmanCompletions,
      currentObtainium: f.currentObtainium
    })
    const oldVal = oldCalcObtainium(
      f.base,
      f.immaculate,
      f.DR,
      f.timeMultiplier,
      f.baseMults,
      f.inC14,
      f.taxmanEnabled,
      f.taxmanCompletions,
      f.currentObtainium
    )
    expect(closeEnoughDec(newVal, oldVal)).toBe(true)
  })
})

// ─── calculatePositiveSalvage parity ───────────────────────────────────────

const oldCalcPositiveSalvage = (raw: number, mult: number, taxmanEnabled: boolean): number => {
  if (taxmanEnabled) {
    const baseSalvage = 100
    return baseSalvage + (raw * mult) / Math.max(1, Math.log(raw))
  }
  return raw * mult
}

describe('parity: calculatePositiveSalvage', () => {
  const rawGrid = [0, 0.5, 1, 10, 100, 1000, 1e6, 1e10]
  const multGrid = [0.5, 1, 2, 5]
  const taxmanGrid = [false, true]

  for (const taxman of taxmanGrid) {
    for (const mult of multGrid) {
      it.each(rawGrid)(`raw=%s mult=${mult} taxman=${taxman}`, (raw) => {
        const newVal = newCalcPositiveSalvage({
          rawPositiveSalvage: raw,
          positiveSalvageMultiplier: mult,
          taxmanLastStandEnabled: taxman
        })
        const oldVal = oldCalcPositiveSalvage(raw, mult, taxman)
        expect(closeEnough(newVal, oldVal)).toBe(true)
      })
    }
  }
})
