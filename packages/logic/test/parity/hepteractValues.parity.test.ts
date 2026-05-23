// Parity tests for the hepteract effective/cap helpers, lifted from
// packages/web_ui/src/Hepteracts.ts. Sweeps cover:
//   - hepteractEffective: linear regime (≤ LIMIT), DR regime (> LIMIT),
//     quark special-case pass-through, varying DR exponents
//   - hepteractCap: TIMES_CAP_EXTENDED at 0/1/many
//   - hepteractFinalCap: with and without Exalt 3 doubling

import { describe, expect, it } from 'vitest'
import {
  hepteractCap as newCap,
  hepteractEffective as newEffective,
  type HepteractEffectiveInput,
  hepteractFinalCap as newFinalCap
} from '../../src/mechanics/hepteractValues'

// ─── Old implementations (verbatim from packages/web_ui/src/Hepteracts.ts) ─

const oldEffective = (input: HepteractEffectiveInput): number => {
  if (input.isQuark) {
    return input.rawAmount
  }
  let effectiveValue = Math.min(input.rawAmount, input.limit)
  if (input.rawAmount > input.limit) {
    effectiveValue *= Math.pow(input.rawAmount / input.limit, input.drExponent)
  }
  return effectiveValue
}

const oldCap = (baseCap: number, timesCapExtended: number): number => {
  return Math.pow(2, timesCapExtended) * baseCap
}

const oldFinalCap = (baseCap: number, timesCapExtended: number, exalt3HepteractCap: boolean): number => {
  const specialMultiplier = exalt3HepteractCap ? 2 : 1
  return oldCap(baseCap, timesCapExtended) * specialMultiplier
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// ─── hepteractEffective ────────────────────────────────────────────────────

describe('parity: hepteractEffective (linear regime, ≤ LIMIT)', () => {
  for (const rawAmount of [0, 1, 100, 999, 1000]) {
    for (const drExponent of [0.5, 1, 2]) {
      const input: HepteractEffectiveInput = {
        rawAmount,
        limit: 1000,
        drExponent,
        isQuark: false
      }
      it(`raw=${rawAmount} limit=1000 dr=${drExponent}`, () => {
        expect(closeEnough(newEffective(input), oldEffective(input))).toBe(true)
      })
    }
  }
})

describe('parity: hepteractEffective (DR regime, > LIMIT)', () => {
  // Sweep across the LIMIT boundary and into the DR regime. Different
  // DR exponents: 0.5 (most hepts), 1 (no DR), 2 (extreme).
  for (const rawAmount of [1001, 1500, 2000, 10000, 100000]) {
    for (const drExponent of [0.1, 0.5, 0.75, 1, 1.5]) {
      const input: HepteractEffectiveInput = {
        rawAmount,
        limit: 1000,
        drExponent,
        isQuark: false
      }
      it(`raw=${rawAmount} limit=1000 dr=${drExponent}`, () => {
        expect(closeEnough(newEffective(input), oldEffective(input))).toBe(true)
      })
    }
  }
})

describe('parity: hepteractEffective (quark special-case)', () => {
  // isQuark === true → just returns rawAmount, ignoring limit/dr.
  for (const rawAmount of [0, 100, 1e6, 1e20]) {
    const input: HepteractEffectiveInput = {
      rawAmount,
      limit: 50, // irrelevant when isQuark
      drExponent: 0.5, // irrelevant when isQuark
      isQuark: true
    }
    it(`quark raw=${rawAmount}`, () => {
      expect(newEffective(input)).toBe(oldEffective(input))
      expect(newEffective(input)).toBe(rawAmount)
    })
  }
})

// ─── hepteractCap ──────────────────────────────────────────────────────────

describe('parity: hepteractCap', () => {
  // BASE_CAP * 2^TIMES_CAP_EXTENDED.
  for (const baseCap of [10, 100, 1e5, 1e10]) {
    for (const times of [0, 1, 5, 10, 20]) {
      it(`base=${baseCap} times=${times}`, () => {
        expect(newCap(baseCap, times)).toBe(oldCap(baseCap, times))
      })
    }
  }
})

// ─── hepteractFinalCap (Exalt 3 doubling) ─────────────────────────────────

describe('parity: hepteractFinalCap', () => {
  for (const baseCap of [10, 1000, 1e6]) {
    for (const times of [0, 1, 5]) {
      for (const exalt3 of [true, false]) {
        it(`base=${baseCap} times=${times} exalt3=${exalt3}`, () => {
          expect(newFinalCap(baseCap, times, exalt3)).toBe(oldFinalCap(baseCap, times, exalt3))
        })
      }
    }
  }
})
