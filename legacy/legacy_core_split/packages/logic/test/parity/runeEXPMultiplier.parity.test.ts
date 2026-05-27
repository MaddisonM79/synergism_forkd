// Parity tests for universalRuneEXPMult, lifted from
// packages/web_ui/src/Runes.ts. The function aggregates three input groups
// (additive, multiplicative, recycle) — the sweep enumerates each input
// in isolation (other inputs zero) to verify per-contribution arithmetic,
// then combines several non-zero inputs to verify the sum × product
// composition matches.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import { universalRuneEXPMult as newMult } from '../../src/mechanics/runeEXPMultiplier'

// ─── Old implementation (verbatim from packages/web_ui/src/Runes.ts) ───────

interface OldInput {
  purchasedLevels: number
  c1Completions: number
  research22: number
  research23: number
  upgrade71: number
  research91: number
  research92: number
  ascensionCounter: number
  cubeUpgrade32: number
  constantUpgrade8: number
  challenge15RuneExpReward: number
  salvageRuneEXPMultiplier: Decimal
}

const oldMult = (input: OldInput): Decimal => {
  const allRuneExpAdditiveMultiplier = 1
    + Math.min(1, input.c1Completions)
    + (0.4 / 10) * input.c1Completions
    + 0.6 * input.research22
    + 0.3 * input.research23
    + (input.upgrade71 * input.purchasedLevels) / 25

  const allRuneExpMultiplier = [
    1 + input.research91 / 20,
    1 + input.research92 / 20,
    1 + (input.ascensionCounter / 1000) * input.cubeUpgrade32,
    1 + (1 / 10) * input.constantUpgrade8,
    input.challenge15RuneExpReward
  ].reduce((x, y) => x.times(y), new Decimal('1'))

  return allRuneExpMultiplier.times(allRuneExpAdditiveMultiplier).times(input.salvageRuneEXPMultiplier)
}

const closeEnoughDecimal = (a: Decimal, b: Decimal, rel = 1e-12): boolean => {
  if (a.eq(b)) return true
  const an = Math.abs(Number(a.toString()))
  const bn = Math.abs(Number(b.toString()))
  if (an < 1 && bn < 1) return Math.abs(an - bn) < rel
  return Math.abs(an - bn) / Math.max(an, bn) < rel
}

const baseZero: OldInput = {
  purchasedLevels: 0,
  c1Completions: 0,
  research22: 0,
  research23: 0,
  upgrade71: 0,
  research91: 0,
  research92: 0,
  ascensionCounter: 0,
  cubeUpgrade32: 0,
  constantUpgrade8: 0,
  challenge15RuneExpReward: 1,
  salvageRuneEXPMultiplier: new Decimal(1)
}

// ─── Per-input parity (other inputs zero) ──────────────────────────────────

describe('parity: universalRuneEXPMult (per-input, others zero)', () => {
  const cases: { name: string; mk: () => OldInput }[] = [
    // c1Completions exercises the min(1, n) + 0.04 * n piecewise
    ...[0, 1, 2, 5, 10, 100].map((n) => ({
      name: `c1Completions=${n}`,
      mk: () => ({ ...baseZero, c1Completions: n })
    })),
    ...[0, 1, 2, 3].map((n) => ({
      name: `research22=${n}`,
      mk: () => ({ ...baseZero, research22: n })
    })),
    ...[0, 1, 3].map((n) => ({
      name: `research23=${n}`,
      mk: () => ({ ...baseZero, research23: n })
    })),
    // upgrade71 scales with purchasedLevels — pair with non-zero pl
    ...[0, 1].map((n) => ({
      name: `upgrade71=${n} purchasedLevels=100`,
      mk: () => ({ ...baseZero, upgrade71: n, purchasedLevels: 100 })
    })),
    ...[0, 5, 20].map((n) => ({
      name: `research91=${n}`,
      mk: () => ({ ...baseZero, research91: n })
    })),
    ...[0, 5, 20].map((n) => ({
      name: `research92=${n}`,
      mk: () => ({ ...baseZero, research92: n })
    })),
    // cubeUpgrade32 needs nonzero ascensionCounter to contribute
    ...[0, 1, 5].map((n) => ({
      name: `cubeUpgrade32=${n} ascCounter=10000`,
      mk: () => ({ ...baseZero, cubeUpgrade32: n, ascensionCounter: 10000 })
    })),
    ...[0, 1, 5, 10].map((n) => ({
      name: `constantUpgrade8=${n}`,
      mk: () => ({ ...baseZero, constantUpgrade8: n })
    })),
    ...[1, 1.5, 5, 1e6].map((n) => ({
      name: `challenge15Reward=${n}`,
      mk: () => ({ ...baseZero, challenge15RuneExpReward: n })
    })),
    ...[new Decimal(1), new Decimal(2), new Decimal('1e10')].map((d) => ({
      name: `salvage=${d.toString()}`,
      mk: () => ({ ...baseZero, salvageRuneEXPMultiplier: d })
    }))
  ]

  for (const { name, mk } of cases) {
    it(name, () => {
      const input = mk()
      expect(closeEnoughDecimal(newMult(input), oldMult(input))).toBe(true)
    })
  }
})

// ─── Composition (all inputs simultaneously non-zero) ──────────────────────

describe('parity: universalRuneEXPMult (composition)', () => {
  const mixes: OldInput[] = [
    {
      purchasedLevels: 50,
      c1Completions: 5,
      research22: 2,
      research23: 3,
      upgrade71: 1,
      research91: 10,
      research92: 5,
      ascensionCounter: 5000,
      cubeUpgrade32: 3,
      constantUpgrade8: 7,
      challenge15RuneExpReward: 2.5,
      salvageRuneEXPMultiplier: new Decimal(3)
    },
    {
      purchasedLevels: 1000,
      c1Completions: 50,
      research22: 5,
      research23: 5,
      upgrade71: 1,
      research91: 20,
      research92: 20,
      ascensionCounter: 100000,
      cubeUpgrade32: 10,
      constantUpgrade8: 20,
      challenge15RuneExpReward: 100,
      salvageRuneEXPMultiplier: new Decimal('1e5')
    },
    {
      purchasedLevels: 0,
      c1Completions: 1,
      research22: 1,
      research23: 1,
      upgrade71: 0,
      research91: 1,
      research92: 1,
      ascensionCounter: 1,
      cubeUpgrade32: 1,
      constantUpgrade8: 1,
      challenge15RuneExpReward: 1.0001,
      salvageRuneEXPMultiplier: new Decimal(1.5)
    }
  ]
  for (let i = 0; i < mixes.length; i++) {
    it(`mix #${i}`, () => {
      const input = mixes[i]
      expect(closeEnoughDecimal(newMult(input), oldMult(input))).toBe(true)
    })
  }
})
