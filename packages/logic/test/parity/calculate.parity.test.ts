// Parity test for the post-aggregation transformations migrated from
// packages/web_ui/src/Calculate.ts. Both functions take precomputed scalars
// — the OLD versions just embed the same arithmetic — so the parity model
// transcribes the branching verbatim.

import { describe, expect, it } from 'vitest'
import Decimal from 'break_infinity.js'
import {
  calculateActualAntSpeedMult as newCalcAntSpeed,
  calculateAscensionSpeedMult as newCalcAscension,
  calculateGlobalSpeedMult as newCalcGlobal
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
