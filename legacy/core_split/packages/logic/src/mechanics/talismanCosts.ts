// Talisman fragment-cost progression formulas, lifted from
// packages/web_ui/src/Talismans.ts. Both formulas are pure functions over a
// Decimal `baseMult` and an integer `level` — no player state, no globals.
// They return the fragment cost map for the NEXT level of the talisman.
//
// `regularCostProgression` is the cubic-tier formula used by most talismans:
// each fragment tier starts contributing at a level threshold (shard at 0,
// common at 30, uncommon at 60, etc.) and the base multiplier itself ramps
// up at higher levels (120/150/180). `exponentialCostProgression` is the
// alternative used by a handful of talismans whose cost grows as `ratio^level`
// instead of `level^3` — same tier-threshold scheme, different growth curve.

import { Decimal } from '../math/bignum'

export type TalismanCraftItems =
  | 'shard'
  | 'commonFragment'
  | 'uncommonFragment'
  | 'rareFragment'
  | 'epicFragment'
  | 'legendaryFragment'
  | 'mythicalFragment'

export type TalismanCraftCosts = Record<TalismanCraftItems, Decimal>

/**
 * Cubic-tier cost progression. For each tier, the cost is
 * `floor((level - threshold)^3 / divisor + 1) * priceMult`, clamped at zero
 * below the tier threshold. `priceMult` itself grows piecewise past levels
 * 120 / 150 / 180. Returns a 0-cost map until each tier unlocks (shard at 0,
 * common at 30, uncommon at 60, rare at 90, epic at 120, legendary/mythical
 * at 150).
 */
export function regularCostProgression (baseMult: Decimal, level: number): TalismanCraftCosts {
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
    shard: Decimal.max(0, shardCost),
    commonFragment: Decimal.max(0, commonCost),
    uncommonFragment: Decimal.max(0, uncommonCost),
    rareFragment: Decimal.max(0, rareCost),
    epicFragment: Decimal.max(0, epicCost),
    legendaryFragment: Decimal.max(0, legendaryCost),
    mythicalFragment: Decimal.max(0, mythicalCost)
  }
}

/**
 * Exponential cost progression. For each tier the cost is
 * `floor(ratio^(level - threshold) * baseMult * tierConstant)`. The tier
 * constants are fixed (100 / 50 / 25 / 20 / 15 / 10 / 5) and the tier
 * thresholds match `regularCostProgression`. `ratio` is supplied by the
 * caller — common values are 2, 10, 1e5, 1e8 (each used by one talisman).
 */
export function exponentialCostProgression (
  baseMult: Decimal,
  level: number,
  ratio: number
): TalismanCraftCosts {
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
