// Parity tests for the sigmoid helpers lifted from
// packages/web_ui/src/Calculate.ts. Sweeps constants below/above 1 (the
// curves are inverted when constant < 1), factor/coefficient values around
// zero and far above, and a divisor grid for `calculateSigmoid`.

import { describe, expect, it } from 'vitest'
import {
  calculateSigmoid as newSigmoid,
  calculateSigmoidExponential as newSigmoidExp
} from '../../src/math/sigmoid'

// ─── Old implementations (verbatim from packages/web_ui/src/Calculate.ts) ───

const oldSigmoid = (constant: number, factor: number, divisor: number): number => {
  return 1 + (constant - 1) * (1 - Math.pow(2, -factor / divisor))
}

const oldSigmoidExp = (constant: number, coefficient: number): number => {
  return 1 + (constant - 1) * (1 - Math.exp(-coefficient))
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: calculateSigmoid', () => {
  const constants = [0.5, 1, 1.05, 2, 10, 100, 1e6]
  const factors = [0, 0.5, 1, 10, 100, 1e6, 1e18]
  const divisors = [1, 100, 1e6, 1e18]
  for (const constant of constants) {
    for (const divisor of divisors) {
      it.each(factors)(`constant=${constant} divisor=${divisor} factor=%s`, (factor) => {
        expect(closeEnough(newSigmoid(constant, factor, divisor), oldSigmoid(constant, factor, divisor))).toBe(true)
      })
    }
  }
})

describe('parity: calculateSigmoidExponential', () => {
  const constants = [0.5, 1, 1.05, 2, 20, 40, 1e6, 49000001]
  const coefficients = [0, 0.001, 0.5, 1, 10, 100]
  for (const constant of constants) {
    it.each(coefficients)(`constant=${constant} coeff=%s`, (coeff) => {
      expect(closeEnough(newSigmoidExp(constant, coeff), oldSigmoidExp(constant, coeff))).toBe(true)
    })
  }
})
