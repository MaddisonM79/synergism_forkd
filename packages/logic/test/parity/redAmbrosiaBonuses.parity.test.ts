// Parity tests for the red-ambrosia bonuses lifted from
// packages/web_ui/src/Calculate.ts. Sweeps cross both gate states (unlocked
// true/false), the cubeUpgrade[79] === 0 gate for cookie luck, and a grid of
// lifetimeRedAmbrosia spanning 0 → 1e9 plus a few small/decimal values where
// the log10 / Math.pow curves are sensitive.

import { describe, expect, it } from 'vitest'
import {
  calculateCookieUpgrade29Luck as newCookie29,
  calculateRedAmbrosiaCubes as newRedCubes,
  calculateRedAmbrosiaObtainium as newRedObtainium,
  calculateRedAmbrosiaOffering as newRedOffering
} from '../../src/mechanics/redAmbrosiaBonuses'

// ─── Old implementations (verbatim from packages/web_ui/src/Calculate.ts) ───

const oldCookie29 = (cubeUpgrade79: number, lifetimeRedAmbrosia: number): number => {
  if (cubeUpgrade79 === 0 || lifetimeRedAmbrosia === 0) {
    return 0
  } else {
    return 10 * Math.pow(Math.log10(lifetimeRedAmbrosia), 2)
  }
}

const oldRedCubes = (unlocked: boolean, lifetimeRedAmbrosia: number, extraExponent: number): number => {
  if (unlocked) {
    const exponent = 0.4 + extraExponent
    return 1 + Math.pow(lifetimeRedAmbrosia, exponent) / 100
  } else {
    return 1
  }
}

const oldRedObtainium = (unlocked: boolean, lifetimeRedAmbrosia: number): number => {
  if (unlocked) {
    return 1 + Math.pow(lifetimeRedAmbrosia, 0.6) / 100
  } else {
    return 1
  }
}

const oldRedOffering = (unlocked: boolean, lifetimeRedAmbrosia: number): number => {
  if (unlocked) {
    return 1 + Math.pow(lifetimeRedAmbrosia, 0.6) / 100
  } else {
    return 1
  }
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

const lifetimeRedAmbrosiaGrid = [0, 1, 2, 10, 100, 1000, 10000, 1e5, 1e6, 1e9]
const extraExponentGrid = [0, 0.05, 0.1, 0.2, 0.5]

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: calculateCookieUpgrade29Luck', () => {
  // Sweep cubeUpgrade79 across 0 (gate off), 1, and a higher value (gate is
  // boolean-ish: only the === 0 path matters).
  const cubeUpgrade79Grid = [0, 1, 2, 5]
  for (const cubeUpgrade79 of cubeUpgrade79Grid) {
    it.each(lifetimeRedAmbrosiaGrid)(`cu79=${cubeUpgrade79} lifetimeRed=%i`, (lifetime) => {
      const next = newCookie29({ cubeUpgrade79, lifetimeRedAmbrosia: lifetime })
      const old = oldCookie29(cubeUpgrade79, lifetime)
      expect(closeEnough(next, old)).toBe(true)
    })
  }
})

describe('parity: calculateRedAmbrosiaCubes', () => {
  const unlockedGrid = [true, false]
  for (const unlocked of unlockedGrid) {
    for (const extraExponent of extraExponentGrid) {
      it.each(lifetimeRedAmbrosiaGrid)(
        `unlocked=${unlocked} extra=${extraExponent} lifetimeRed=%i`,
        (lifetime) => {
          const next = newRedCubes({ unlocked, lifetimeRedAmbrosia: lifetime, extraExponent })
          const old = oldRedCubes(unlocked, lifetime, extraExponent)
          expect(closeEnough(next, old)).toBe(true)
        }
      )
    }
  }
})

describe('parity: calculateRedAmbrosiaObtainium', () => {
  const unlockedGrid = [true, false]
  for (const unlocked of unlockedGrid) {
    it.each(lifetimeRedAmbrosiaGrid)(`unlocked=${unlocked} lifetimeRed=%i`, (lifetime) => {
      const next = newRedObtainium({ unlocked, lifetimeRedAmbrosia: lifetime })
      const old = oldRedObtainium(unlocked, lifetime)
      expect(closeEnough(next, old)).toBe(true)
    })
  }
})

describe('parity: calculateRedAmbrosiaOffering', () => {
  const unlockedGrid = [true, false]
  for (const unlocked of unlockedGrid) {
    it.each(lifetimeRedAmbrosiaGrid)(`unlocked=${unlocked} lifetimeRed=%i`, (lifetime) => {
      const next = newRedOffering({ unlocked, lifetimeRedAmbrosia: lifetime })
      const old = oldRedOffering(unlocked, lifetime)
      expect(closeEnough(next, old)).toBe(true)
    })
  }
})
