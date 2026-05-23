// Parity tests for calculateCoinProduction, lifted from
// packages/web_ui/src/Tax.ts. Each `oldXxx` transcribes the pre-migration
// per-tier formula and noise-floor clamp verbatim. Sweeps cover: the
// 0.0001 noise-floor boundary, the pre-clamp/post-clamp aggregation
// difference, and the fifth-tier production formula at the high end.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  calculateCoinProduction as newCoinProd,
  type CalculateCoinProductionInput,
  type PerCoinTierInput
} from '../../src/mechanics/coinProduction'

// ─── Old implementation (verbatim from packages/web_ui/src/Tax.ts) ─────────

const oldTierOutput = (tier: PerCoinTierInput, globalMult: Decimal): Decimal =>
  tier.generated.add(tier.owned).times(globalMult).times(tier.coinMulti).times(tier.produceCoin)

const oldCalculateCoinProduction = (input: CalculateCoinProductionInput) => {
  let first = oldTierOutput(input.first, input.globalCoinMultiplier)
  let second = oldTierOutput(input.second, input.globalCoinMultiplier)
  let third = oldTierOutput(input.third, input.globalCoinMultiplier)
  let fourth = oldTierOutput(input.fourth, input.globalCoinMultiplier)
  let fifth = oldTierOutput(input.fifth, input.globalCoinMultiplier)
  const total = first.add(second).add(third).add(fourth).add(fifth)

  if (first.lte(0.0001)) first = new Decimal(0)
  if (second.lte(0.0001)) second = new Decimal(0)
  if (third.lte(0.0001)) third = new Decimal(0)
  if (fourth.lte(0.0001)) fourth = new Decimal(0)
  if (fifth.lte(0.0001)) fifth = new Decimal(0)

  return { first, second, third, fourth, fifth, total, perSecond: total.times(40) }
}

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

const expectAllEqual = (
  next: ReturnType<typeof newCoinProd>,
  old: ReturnType<typeof oldCalculateCoinProduction>
) => {
  expect(decimalEq(next.first, old.first)).toBe(true)
  expect(decimalEq(next.second, old.second)).toBe(true)
  expect(decimalEq(next.third, old.third)).toBe(true)
  expect(decimalEq(next.fourth, old.fourth)).toBe(true)
  expect(decimalEq(next.fifth, old.fifth)).toBe(true)
  expect(decimalEq(next.total, old.total)).toBe(true)
  expect(decimalEq(next.perSecond, old.perSecond)).toBe(true)
}

// Standard zeroed tier — for selectively setting one field at a time.
const zeroTier: PerCoinTierInput = {
  generated: new Decimal(0),
  owned: 0,
  coinMulti: new Decimal(1),
  produceCoin: 1
}

describe('parity: calculateCoinProduction (all-zero baseline)', () => {
  it('all tiers zero → 0 everywhere', () => {
    const input: CalculateCoinProductionInput = {
      first: zeroTier,
      second: zeroTier,
      third: zeroTier,
      fourth: zeroTier,
      fifth: zeroTier,
      globalCoinMultiplier: new Decimal(1)
    }
    expectAllEqual(newCoinProd(input), oldCalculateCoinProduction(input))
  })
})

describe('parity: calculateCoinProduction (noise-floor boundary)', () => {
  // Tier output values around 0.0001 — verify the clamp fires correctly.
  // Picking generated values that, after the multipliers, land just below
  // and just above the threshold.
  const cases = [
    // generated=0.00005, owned=0, mult=1, produce=1 → 0.00005 (clamped to 0)
    { generated: 0.00005, expectedClamp: true },
    // generated=0.0001, owned=0 → 0.0001 (exactly at threshold; .lte → clamped)
    { generated: 0.0001, expectedClamp: true },
    // generated=0.0002 → > 0.0001 (kept)
    { generated: 0.0002, expectedClamp: false },
    // generated=1 → way above threshold
    { generated: 1, expectedClamp: false }
  ]
  for (const c of cases) {
    it(`generated=${c.generated} (clamp=${c.expectedClamp})`, () => {
      const tier: PerCoinTierInput = { ...zeroTier, generated: new Decimal(c.generated) }
      const input: CalculateCoinProductionInput = {
        first: tier,
        second: zeroTier,
        third: zeroTier,
        fourth: zeroTier,
        fifth: zeroTier,
        globalCoinMultiplier: new Decimal(1)
      }
      const next = newCoinProd(input)
      const old = oldCalculateCoinProduction(input)
      expectAllEqual(next, old)
      // Sanity check: clamp expectation matches reality.
      if (c.expectedClamp) {
        expect(next.first.eq(0)).toBe(true)
      } else {
        expect(next.first.gt(0)).toBe(true)
      }
    })
  }
})

describe('parity: calculateCoinProduction (one-tier-each)', () => {
  // Test each of the five tier slots independently to verify the per-tier
  // formula is identical (no accidental cross-tier copy-paste bug).
  const tiers: ('first' | 'second' | 'third' | 'fourth' | 'fifth')[] = [
    'first',
    'second',
    'third',
    'fourth',
    'fifth'
  ]
  const nonZeroTier: PerCoinTierInput = {
    generated: new Decimal(100),
    owned: 50,
    coinMulti: new Decimal(2),
    produceCoin: 3
  }
  for (const tierName of tiers) {
    it(`only ${tierName} non-zero`, () => {
      const input: CalculateCoinProductionInput = {
        first: zeroTier,
        second: zeroTier,
        third: zeroTier,
        fourth: zeroTier,
        fifth: zeroTier,
        globalCoinMultiplier: new Decimal(5),
        [tierName]: nonZeroTier
      }
      expectAllEqual(newCoinProd(input), oldCalculateCoinProduction(input))
    })
  }
})

describe('parity: calculateCoinProduction (large values)', () => {
  // Stress test — every tier active at scales where Decimal precision matters.
  const cases: CalculateCoinProductionInput[] = [
    {
      first: { generated: new Decimal(1e10), owned: 100, coinMulti: new Decimal(2), produceCoin: 5 },
      second: { generated: new Decimal(1e20), owned: 1000, coinMulti: new Decimal(3), produceCoin: 10 },
      third: { generated: new Decimal(1e30), owned: 10000, coinMulti: new Decimal(4), produceCoin: 20 },
      fourth: { generated: new Decimal(1e40), owned: 100000, coinMulti: new Decimal(5), produceCoin: 50 },
      fifth: { generated: new Decimal(1e100), owned: 1000000, coinMulti: new Decimal('1e10'), produceCoin: 100 },
      globalCoinMultiplier: new Decimal('1e5')
    },
    // Extreme global multiplier
    {
      first: { generated: new Decimal(1), owned: 0, coinMulti: new Decimal(1), produceCoin: 1 },
      second: { generated: new Decimal(2), owned: 0, coinMulti: new Decimal(1), produceCoin: 1 },
      third: { generated: new Decimal(3), owned: 0, coinMulti: new Decimal(1), produceCoin: 1 },
      fourth: { generated: new Decimal(4), owned: 0, coinMulti: new Decimal(1), produceCoin: 1 },
      fifth: { generated: new Decimal(5), owned: 0, coinMulti: new Decimal(1), produceCoin: 1 },
      globalCoinMultiplier: new Decimal('1e200')
    }
  ]
  for (let i = 0; i < cases.length; i++) {
    it(`large-value mix #${i}`, () => {
      expectAllEqual(newCoinProd(cases[i]), oldCalculateCoinProduction(cases[i]))
    })
  }
})

describe('parity: calculateCoinProduction (pre-clamp total)', () => {
  // The aggregate uses PRE-clamp values. Verify a setup where some tiers
  // are below the noise floor but contribute to total nonetheless.
  it('tiny tier contributes to total despite clamp', () => {
    const tiny: PerCoinTierInput = {
      generated: new Decimal(0.00005),
      owned: 0,
      coinMulti: new Decimal(1),
      produceCoin: 1
    }
    const large: PerCoinTierInput = {
      generated: new Decimal(1000),
      owned: 0,
      coinMulti: new Decimal(1),
      produceCoin: 1
    }
    const input: CalculateCoinProductionInput = {
      first: tiny,
      second: large,
      third: tiny,
      fourth: large,
      fifth: tiny,
      globalCoinMultiplier: new Decimal(1)
    }
    const next = newCoinProd(input)
    const old = oldCalculateCoinProduction(input)
    expectAllEqual(next, old)
    // Sanity: total should be > 2000 (the two large tiers) + tiny contributions
    expect(next.total.gt(2000)).toBe(true)
  })
})
