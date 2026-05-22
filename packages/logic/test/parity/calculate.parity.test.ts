// Parity test for the post-aggregation transformations migrated from
// packages/web_ui/src/Calculate.ts. Both functions take precomputed scalars
// — the OLD versions just embed the same arithmetic — so the parity model
// transcribes the branching verbatim.

import { describe, expect, it } from 'vitest'
import {
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
