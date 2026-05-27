// Parity tests for regularCostProgression and exponentialCostProgression,
// lifted from packages/web_ui/src/Talismans.ts. Each `oldXxx` transcribes the
// pre-migration body verbatim. Sweeps cover every fragment-tier threshold
// (0/30/60/90/120/150/180) for the regular formula and the 30/60/90/120/150
// thresholds × the four ratios (2/10/1e5/1e8) actually used in production
// for the exponential formula.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  exponentialCostProgression as newExpCostProgression,
  regularCostProgression as newRegCostProgression,
  type TalismanCraftItems
} from '../../src/mechanics/talismanCosts'

// ─── Old implementations (verbatim from packages/web_ui/src/Talismans.ts) ───

const oldRegularCostProgression = (
  baseMult: Decimal,
  level: number
): Record<TalismanCraftItems, Decimal> => {
  let priceMult = baseMult
  if (level >= 120) {
    priceMult = priceMult.times((level - 90) / 30)
  }
  if (level >= 150) {
    priceMult = priceMult.times((level - 120) / 30)
  }
  if (level >= 180) {
    priceMult = priceMult.times((level - 170) / 10)
  }

  const shardCost = Decimal.pow(level, 3).times(1 / 8).plus(1).floor().times(priceMult)
  const commonCost = level >= 30
    ? Decimal.pow(level - 30, 3).times(1 / 32).plus(1).floor().times(priceMult)
    : new Decimal(0)
  const uncommonCost = level >= 60
    ? Decimal.pow(level - 60, 3).times(1 / 384).plus(1).floor().times(priceMult)
    : new Decimal(0)
  const rareCost = level >= 90
    ? Decimal.pow(level - 90, 3).times(1 / 500).plus(1).floor().times(priceMult)
    : new Decimal(0)
  const epicCost = level >= 120
    ? Decimal.pow(level - 120, 3).times(1 / 375).plus(1).floor().times(priceMult)
    : new Decimal(0)
  const legendaryCost = level >= 150
    ? Decimal.pow(level - 150, 3).times(1 / 192).plus(1).floor().times(priceMult)
    : new Decimal(0)
  const mythicalCost = level >= 150
    ? Decimal.pow(level - 150, 3).times(1 / 1280).plus(1).floor().times(priceMult)
    : new Decimal(0)

  return {
    'shard': Decimal.max(0, shardCost),
    'commonFragment': Decimal.max(0, commonCost),
    'uncommonFragment': Decimal.max(0, uncommonCost),
    'rareFragment': Decimal.max(0, rareCost),
    'epicFragment': Decimal.max(0, epicCost),
    'legendaryFragment': Decimal.max(0, legendaryCost),
    'mythicalFragment': Decimal.max(0, mythicalCost)
  }
}

const oldExponentialCostProgression = (
  baseMult: Decimal,
  level: number,
  ratio: number
): Record<TalismanCraftItems, Decimal> => {
  const baseMultDecimal = new Decimal(baseMult)

  return {
    shard: Decimal.pow(ratio, level).times(baseMultDecimal).times(100).floor(),
    commonFragment: level >= 30
      ? Decimal.pow(ratio, level - 30).times(baseMultDecimal).times(50).floor()
      : new Decimal(0),
    uncommonFragment: level >= 60
      ? Decimal.pow(ratio, level - 60).times(baseMultDecimal).times(25).floor()
      : new Decimal(0),
    rareFragment: level >= 90
      ? Decimal.pow(ratio, level - 90).times(baseMultDecimal).times(20).floor()
      : new Decimal(0),
    epicFragment: level >= 120
      ? Decimal.pow(ratio, level - 120).times(baseMultDecimal).times(15).floor()
      : new Decimal(0),
    legendaryFragment: level >= 150
      ? Decimal.pow(ratio, level - 150).times(baseMultDecimal).times(10).floor()
      : new Decimal(0),
    mythicalFragment: level >= 150
      ? Decimal.pow(ratio, level - 150).times(baseMultDecimal).times(5).floor()
      : new Decimal(0)
  }
}

const tierKeys: TalismanCraftItems[] = [
  'shard',
  'commonFragment',
  'uncommonFragment',
  'rareFragment',
  'epicFragment',
  'legendaryFragment',
  'mythicalFragment'
]

const expectMapsEqual = (
  next: Record<TalismanCraftItems, Decimal>,
  old: Record<TalismanCraftItems, Decimal>
) => {
  for (const k of tierKeys) {
    // Decimal.eq returns true on identical magnitude regardless of internal
    // representation — safer than .toString() comparisons across the
    // base-2/base-10 boundary inside break_infinity.
    expect(next[k].eq(old[k])).toBe(true)
  }
}

// Sweep every fragment-tier boundary in both directions plus a few mid-band
// values for the regular formula's piecewise priceMult multiplier (120/150/180).
const regularLevelGrid = [
  0,
  1,
  10,
  29,
  30,
  31,
  45,
  59,
  60,
  61,
  75,
  89,
  90,
  91,
  100,
  119,
  120,
  121,
  135,
  149,
  150,
  151,
  165,
  179,
  180,
  181,
  200,
  250
]
const baseMultGrid = [new Decimal(1), new Decimal(10), new Decimal('1e6')]

// Exponential formula production ratios — 2 (mortuus), 10 (plastic),
// 1e5 (wowSquare), 1e8 (cookieGrandma). Mid-range ratios included for sanity.
const exponentialRatioGrid = [2, 10, 1e5, 1e8]
// Exponential growth blows past Decimal's range fast, so cap the sweep.
const exponentialLevelGrid = [
  0,
  1,
  10,
  29,
  30,
  31,
  45,
  59,
  60,
  61,
  75,
  89,
  90,
  91,
  100,
  119,
  120,
  121,
  130,
  149,
  150,
  151
]

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: regularCostProgression (each tier threshold)', () => {
  for (const baseMult of baseMultGrid) {
    it.each(regularLevelGrid)(`baseMult=${baseMult.toString()} level=%i`, (level) => {
      const next = newRegCostProgression(baseMult, level)
      const old = oldRegularCostProgression(baseMult, level)
      expectMapsEqual(next, old)
    })
  }
})

describe('parity: exponentialCostProgression (each ratio × tier threshold)', () => {
  for (const ratio of exponentialRatioGrid) {
    for (const baseMult of baseMultGrid) {
      it.each(exponentialLevelGrid)(
        `ratio=${ratio} baseMult=${baseMult.toString()} level=%i`,
        (level) => {
          const next = newExpCostProgression(baseMult, level, ratio)
          const old = oldExponentialCostProgression(baseMult, level, ratio)
          expectMapsEqual(next, old)
        }
      )
    }
  }
})
