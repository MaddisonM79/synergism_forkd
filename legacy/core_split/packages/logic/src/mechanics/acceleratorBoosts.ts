import { Decimal } from '../math/bignum'

// Cost formula for accelerator boosts, purchased with prestigePoints. The
// buyAccelerator pair migrated earlier handles the much simpler base-game
// accelerators; boosts are a separate ladder bought once you own
// `player.upgrades[46]`. The cost climbs more aggressively (10^10 per level
// plus a triangle-number kicker, and beyond `1000 * eff` levels the kicker
// grows quadratically — see the threshold branch below).
//
// The accompanying boostAccelerator buy loop still lives in
// packages/web_ui/src/Buy.ts because it calls reset('prestige') inline;
// that part migrates with the broader reset-system overhaul in Phase 6.

const BUYMAX = Math.pow(10, 15)

// Triangle and square-pyramidal sums — closed forms for 1+2+…+n and
// 1²+2²+…+n². Used to expand the cumulative log-exponent contribution as the
// boost level climbs.
const linSum = (n: number): number => n * (n + 1) / 2
const sqrSum = (n: number): number => n * (n + 1) * (2 * n + 1) / 6

export interface GetAcceleratorBoostCostInput {
  /**
   * Cost-delay multiplier from the thrift rune blessing
   * (`getRuneBlessingEffect('thrift').accelBoostCostDelay` in web_ui). Pushes
   * back the level at which the quadratic-in-level growth kicks in: the
   * threshold is `1000 * accelBoostCostDelay`.
   */
  accelBoostCostDelay: number
}

export function getAcceleratorBoostCost(
  level: number,
  input: GetAcceleratorBoostCostInput
): Decimal {
  // Formula is 0-indexed; callers pass 1-indexed level.
  level--
  const base = new Decimal(1e3)
  const eff = input.accelBoostCostDelay

  let cost: Decimal
  if (level > 1000 * eff) {
    cost = base.times(Decimal.pow(
      10,
      10 * level
        + linSum(level) // triangle-number kicker — exponent grows by one more each level
        + sqrSum(level - 1000 * eff) / eff
    ))
  } else {
    cost = base.times(Decimal.pow(10, 10 * level + linSum(level)))
  }

  if (level > BUYMAX) {
    const diminishingExponent = 1 / 8
    const quadrillionCost = getAcceleratorBoostCost(BUYMAX, input)
    const newCost = quadrillionCost.pow(Math.pow(level / BUYMAX, 1 / diminishingExponent))
    // Re-normalize after the in-place mantissa/exponent rewrite.
    const newExtra = newCost.exponent - Math.floor(newCost.exponent)
    newCost.exponent = Math.floor(newCost.exponent)
    newCost.mantissa *= Math.pow(10, newExtra)
    newCost.normalize()
    return Decimal.max(cost, newCost)
  }
  return cost
}
