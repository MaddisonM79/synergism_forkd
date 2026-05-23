// Ambrosia-family formulas lifted from packages/web_ui/src/Calculate.ts:
//   - threshold counters over player.lifetimeAmbrosia
//   - blueberry / red-ambrosia time-per-tick formulas
//   - cube / quark ambrosia multipliers
//   - composers over the 4 singularity-GQ and 4 octeract upgrade levels
//
// External effect lookups (getShopUpgradeEffects, getAmbrosiaUpgradeEffects,
// getSingularityChallengeEffect, getGQUpgradeEffect, getOcteractUpgradeEffect)
// stay in web_ui — callers precompute them and pass scalar/array inputs in.

// ─── Threshold counters ────────────────────────────────────────────────────

const digitReduction = 4

/**
 * Number of crossed "digit-reduction" thresholds. The first threshold is at
 * 10000 lifetime ambrosia, and they alternate at 10^(N+digitReduction) and
 * 3 × 10^(N+digitReduction). Returns 0 below the first threshold.
 */
export function calculateNumberOfThresholds(lifetimeAmbrosia: number): number {
  const numDigits = lifetimeAmbrosia > 0 ? 1 + Math.floor(Math.log10(lifetimeAmbrosia)) : 0
  const matissa = Math.floor(lifetimeAmbrosia / Math.pow(10, numDigits - 1))
  const extraReduction = matissa >= 3 ? 1 : 0
  return Math.max(0, 2 * (numDigits - digitReduction) - 1 + extraReduction)
}

/**
 * Distance (in lifetime ambrosia) to the next threshold. Mirrors the two
 * alternating threshold forms — 10^N and 3 × 10^N.
 */
export function calculateToNextThreshold(lifetimeAmbrosia: number): number {
  const numThresholds = calculateNumberOfThresholds(lifetimeAmbrosia)
  if (numThresholds === 0) {
    return 10000 - lifetimeAmbrosia
  }
  if (numThresholds % 2 === 0) {
    return Math.pow(10, numThresholds / 2 + digitReduction) - lifetimeAmbrosia
  }
  return 3 * Math.pow(10, (numThresholds - 1) / 2 + digitReduction) - lifetimeAmbrosia
}

// ─── Required blueberry / red ambrosia times ───────────────────────────────

export interface CalculateRequiredBlueberryTimeInput {
  /** G.TIME_PER_AMBROSIA — base time-per-bar constant (currently 45). */
  timePerAmbrosia: number
  /** player.lifetimeAmbrosia — drives both the +1/300 linear ramp and the >=10000 power scaling. */
  lifetimeAmbrosia: number
  /** getShopUpgradeEffects('shopAmbrosiaAccelerator', 'ambrosiaPointRequirementMult'). */
  acceleratorMult: number
  /** getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead', 'barRequirementMult'). */
  brickOfLeadMult: number
}

/**
 * Time (in seconds, rounded up past 10k) required to fill one ambrosia bar.
 * Base ramps linearly by lifetime/300, multiplied by the shop accelerator
 * and brick-of-lead, with a `(lifetime/10000)^log10(4)` power kick above
 * 10000.
 */
export function calculateRequiredBlueberryTime(input: CalculateRequiredBlueberryTimeInput): number {
  let val = input.timePerAmbrosia
  val += Math.floor(input.lifetimeAmbrosia / 300)
  val *= input.acceleratorMult
  val *= input.brickOfLeadMult
  if (input.lifetimeAmbrosia >= 10000) {
    const extraScalingPower = Math.log10(4)
    val *= Math.pow(input.lifetimeAmbrosia / 10000, extraScalingPower)
    return Math.ceil(val)
  }
  return val
}

export interface CalculateRequiredRedAmbrosiaTimeInput {
  /** G.TIME_PER_RED_AMBROSIA — base time-per-bar constant (currently 100,000). */
  timePerRedAmbrosia: number
  /** player.lifetimeRedAmbrosia — drives the +200 per linear ramp. */
  lifetimeRedAmbrosia: number
  /** getSingularityChallengeEffect('limitedTime', 'barRequirementMultiplier'). */
  barRequirementMultiplier: number
}

/**
 * Time (in seconds) required to fill one red ambrosia bar. Linear in
 * `lifetimeRedAmbrosia` then multiplied by the limitedTime-challenge effect,
 * capped at `1e6 * barRequirementMultiplier`.
 */
export function calculateRequiredRedAmbrosiaTime(input: CalculateRequiredRedAmbrosiaTimeInput): number {
  let val = input.timePerRedAmbrosia
  val += 200 * input.lifetimeRedAmbrosia
  const max = 1e6 * input.barRequirementMultiplier
  val *= input.barRequirementMultiplier
  return Math.min(max, val)
}

// ─── Singularity milestone blueberries ─────────────────────────────────────

/**
 * Number of blueberries unlocked by all-time max singularity count.
 * Stepwise: 64 → 1, 128 → 2, 192 → 3, 256 → 4, 270 → 5.
 */
export function calculateSingularityMilestoneBlueberries(highestSingularityCount: number): number {
  if (highestSingularityCount >= 270) return 5
  if (highestSingularityCount >= 256) return 4
  if (highestSingularityCount >= 192) return 3
  if (highestSingularityCount >= 128) return 2
  if (highestSingularityCount >= 64) return 1
  return 0
}

// ─── Ambrosia cube / quark multipliers ─────────────────────────────────────

export interface AmbrosiaMultInput {
  /** player.singularityChallenges.noAmbrosiaUpgrades.enabled — when true, effective ambrosia is 0. */
  noAmbrosiaUpgradesEnabled: boolean
  /** player.lifetimeAmbrosia. */
  lifetimeAmbrosia: number
}

/**
 * Cube multiplier from lifetime ambrosia. Three additive tiers:
 *   - floor(eff/66)/100, capped at 1.5
 *   - +floor(eff/666)/100 (capped 1.5), if eff >= 10000
 *   - +floor(eff/6666)/100 (uncapped), if eff >= 100000
 * The `noAmbrosiaUpgrades` Exalt zeros out effective ambrosia.
 */
export function calculateAmbrosiaCubeMult(input: AmbrosiaMultInput): number {
  const effectiveAmbrosia = input.noAmbrosiaUpgradesEnabled ? 0 : input.lifetimeAmbrosia
  let multiplier = 1
  multiplier += Math.min(1.5, Math.floor(effectiveAmbrosia / 66) / 100)
  if (effectiveAmbrosia >= 10000) {
    multiplier += Math.min(1.5, Math.floor(effectiveAmbrosia / 666) / 100)
  }
  if (effectiveAmbrosia >= 100000) {
    multiplier += Math.floor(effectiveAmbrosia / 6666) / 100
  }
  return multiplier
}

/**
 * Quark multiplier from lifetime ambrosia. Same three-tier shape as the cube
 * mult but smaller divisors and a 0.3 per-tier cap on the first two.
 */
export function calculateAmbrosiaQuarkMult(input: AmbrosiaMultInput): number {
  const effectiveAmbrosia = input.noAmbrosiaUpgradesEnabled ? 0 : input.lifetimeAmbrosia
  let multiplier = 1
  multiplier += Math.min(0.3, Math.floor(effectiveAmbrosia / 1666) / 100)
  if (effectiveAmbrosia >= 50000) {
    multiplier += Math.min(0.3, Math.floor(effectiveAmbrosia / 16666) / 100)
  }
  if (effectiveAmbrosia >= 500000) {
    multiplier += Math.floor(effectiveAmbrosia / 166666) / 100
  }
  return multiplier
}

// ─── Upgrade composers ─────────────────────────────────────────────────────
//
// Each composer takes the 4 precomputed effect values for the relevant
// upgrade chain (singAmbrosiaGeneration / singAmbrosiaGeneration2/3/4 etc.)
// and reduces them — multiplicative for generation-speed, additive for luck.

/**
 * Multiplies the four `singAmbrosiaGeneration[1..4]` `ambrosiaBarSpeedMult`
 * effects together. Caller (web_ui) collects them via getGQUpgradeEffect.
 */
export function calculateAmbrosiaGenerationSingularityUpgrade(speedMults: number[]): number {
  return speedMults.reduce((a, b) => a * b, 1)
}

/**
 * Sums the four `singAmbrosiaLuck[1..4]` `ambrosiaLuck` effects.
 */
export function calculateAmbrosiaLuckSingularityUpgrade(luckValues: number[]): number {
  return luckValues.reduce((a, b) => a + b, 0)
}

/**
 * Multiplies the four `octeractAmbrosiaGeneration[1..4]` speed-mult effects.
 */
export function calculateAmbrosiaGenerationOcteractUpgrade(speedMults: number[]): number {
  return speedMults.reduce((a, b) => a * b, 1)
}

/**
 * Sums the four `octeractAmbrosiaLuck[1..4]` luck effects.
 */
export function calculateAmbrosiaLuckOcteractUpgrade(luckValues: number[]): number {
  return luckValues.reduce((a, b) => a + b, 0)
}
