// Per-tier coin production aggregation, lifted from the first half of
// packages/web_ui/src/Tax.ts (the `G.produceFirst..Fifth` chunk that
// pre-dates the tax exponent). Pure Decimal math: each tier's output is
//   (generated + owned) * globalCoinMulti * coinTierMulti * produceCoin
// then clamped to 0 below 0.0001 to suppress sub-noise contributions.
//
// The five tiers (first/second/third/fourth/fifth) have identical formulas
// — they correspond to the five base coin producers. `total` is the sum of
// the (post-clamp) tier outputs; `perSecond` is `total * 40` (the legacy
// 40 Hz tick rate factor that's still hardcoded into the original formula).

import { Decimal } from '../math/bignum'

// Below this threshold, a per-tier output snaps to 0 — suppresses
// extremely-small-but-nonzero values that would otherwise pollute the
// total before they're meaningful.
const TIER_NOISE_FLOOR = 0.0001
// Hardcoded 40 Hz tick rate factor that maps total-per-tick to per-second
// in the legacy display. Lives here rather than in web_ui because it's
// baked into the math contract callers depend on.
const TICKS_PER_SECOND = 40

export interface PerCoinTierInput {
  /** player.<tier>GeneratedCoin — the only field guaranteed to be Decimal. */
  generated: Decimal
  /**
   * player.<tier>OwnedCoin — `number` per the Player schema, fed straight
   * to Decimal.add (which accepts both).
   */
  owned: number
  /** G.coin<Tier>Multi — per-tier coin multiplier. */
  coinMulti: Decimal
  /**
   * player.<tier>ProduceCoin — `number` per the Player schema, fed to
   * Decimal.times.
   */
  produceCoin: number
}

export interface CalculateCoinProductionInput {
  first: PerCoinTierInput
  second: PerCoinTierInput
  third: PerCoinTierInput
  fourth: PerCoinTierInput
  fifth: PerCoinTierInput
  /** G.globalCoinMultiplier — applied to every tier. */
  globalCoinMultiplier: Decimal
}

export interface CalculateCoinProductionResult {
  /** Per-tier output, clamped to 0 below TIER_NOISE_FLOOR. */
  first: Decimal
  second: Decimal
  third: Decimal
  fourth: Decimal
  fifth: Decimal
  /**
   * Sum of pre-clamp tier outputs — matches the legacy `G.produceTotal`
   * which is computed BEFORE the per-tier 0.0001 clamps. (The clamps only
   * reset the per-tier displays, not the aggregate.)
   */
  total: Decimal
  /** total * 40 — converts the per-tick total to per-second for display. */
  perSecond: Decimal
}

function tierOutput (tier: PerCoinTierInput, globalCoinMultiplier: Decimal): Decimal {
  return tier.generated.add(tier.owned).times(globalCoinMultiplier).times(tier.coinMulti).times(tier.produceCoin)
}

function clampNoise (value: Decimal): Decimal {
  return value.lte(TIER_NOISE_FLOOR) ? new Decimal(0) : value
}

/**
 * Per-tier coin production with noise-floor clamping. The aggregate `total`
 * uses the pre-clamp values (matching legacy behavior); each per-tier field
 * in the result is post-clamp. `perSecond` is the per-tick total scaled by
 * the hardcoded 40 Hz tick rate.
 */
export function calculateCoinProduction (input: CalculateCoinProductionInput): CalculateCoinProductionResult {
  const first = tierOutput(input.first, input.globalCoinMultiplier)
  const second = tierOutput(input.second, input.globalCoinMultiplier)
  const third = tierOutput(input.third, input.globalCoinMultiplier)
  const fourth = tierOutput(input.fourth, input.globalCoinMultiplier)
  const fifth = tierOutput(input.fifth, input.globalCoinMultiplier)

  // Aggregate uses the pre-clamp values — the clamps only affect the
  // per-tier display, not the total.
  const total = first.add(second).add(third).add(fourth).add(fifth)

  return {
    first: clampNoise(first),
    second: clampNoise(second),
    third: clampNoise(third),
    fourth: clampNoise(fourth),
    fifth: clampNoise(fifth),
    total,
    perSecond: total.times(TICKS_PER_SECOND)
  }
}
