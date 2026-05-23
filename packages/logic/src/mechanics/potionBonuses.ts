// Potion-consumption bonuses lifted from packages/web_ui/src/Calculate.ts.
// Both functions count how many fixed potion-consumption thresholds the
// player has crossed (via a binary search) and return that count plus the
// distance to the next threshold. The threshold arrays are pure data and
// move with the formulas.
//
// Also exports `calculatePotionValue` (the actual resource award a potion
// produces), which uses the same `calculateFastForwardResourcesGlobal`
// helper internally.

import { Decimal } from '../math/bignum'

// Binary search returning the insertion index for `target` into `array`
// (assumed sorted ascending). Reproduces the behavior of web_ui's
// `findInsertionIndex` (Utility.ts) — kept local to this module since the
// logic package can't reach into web_ui.
function findInsertionIndex(target: number, array: readonly number[]): number {
  if (array.length === 0 || target < array[0]) {
    return 0
  }
  if (target >= array[array.length - 1]) {
    return array.length
  }

  let low = 0
  let high = array.length - 1

  while (low < high) {
    const mid = Math.floor((low + high + 1) / 2)
    if (array[mid] <= target) {
      low = mid
    } else {
      high = mid - 1
    }
  }

  return low + 1
}

const offeringPotionThresholds: readonly number[] = [
  1,
  10,
  25,
  50,
  100,
  500,
  1000,
  10000,
  5e4,
  1e5,
  1e6,
  1e7,
  1e8,
  1e9,
  1e10,
  1e11,
  1e12,
  1e13,
  1e14,
  1e15
]

const obtainiumPotionThresholds: readonly number[] = [
  1, 20, 50, 250, 1000, 20000, 4e5, 1e7, 4e8, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15
]

export interface PotionBonusResult {
  /** Number of crossed thresholds — drives the base offering/obtainium award. */
  amount: number
  /**
   * Lifetime potions of this type still required before the next threshold,
   * or `Number.POSITIVE_INFINITY` if all thresholds have been crossed.
   */
  toNext: number
}

function computePotionBonus(consumed: number, thresholds: readonly number[]): PotionBonusResult {
  const amount = findInsertionIndex(consumed, thresholds)
  const toNext = amount < thresholds.length
    ? thresholds[amount] - consumed
    : Number.POSITIVE_INFINITY
  return { amount, toNext }
}

/**
 * Offering potion base award. Returns the count of crossed
 * `offeringPotionThresholds` plus the distance to the next one.
 */
export function calculateOfferingPotionBaseOfferings(consumed: number): PotionBonusResult {
  return computePotionBonus(consumed, offeringPotionThresholds)
}

/**
 * Obtainium potion base award. Same shape as the offering version with the
 * obtainium-specific threshold table.
 */
export function calculateObtainiumPotionBaseObtainium(consumed: number): PotionBonusResult {
  return computePotionBonus(consumed, obtainiumPotionThresholds)
}

// ─── Potion value ──────────────────────────────────────────────────────────

interface CalculateFastForwardResourcesGlobalInput {
  resetTime: number
  fastForwardAmount: Decimal
  resourceMult: Decimal
  baseResource: number
  /** getGQUpgradeEffect('halfMind', 'unlocked') — gates whether deltaTime uses a flat 10 or the live globalSpeedMult. */
  halfMindUnlocked: boolean
  /** calculateGlobalSpeedMult() — used for the alt-deltaTime branch and the dead-code correction below. */
  globalSpeedMult: number
  /** resetTimeThreshold() in web_ui, i.e. 10 - player.campaigns.timeThresholdReduction. */
  resetTimeThreshold: number
}

function calculateFastForwardResourcesGlobal(input: CalculateFastForwardResourcesGlobalInput): Decimal {
  let timeMultiplier: Decimal = new Decimal('1')

  const deltaTime = input.fastForwardAmount.times(
    input.halfMindUnlocked ? 10 : input.globalSpeedMult
  )

  // Take the min of two derivative approximations: quadratic-penalty branch
  // (when reset time is below threshold) and linear-penalty branch (above).
  timeMultiplier = Decimal.min(
    deltaTime.times(2 * input.resetTime).div(input.resetTimeThreshold ** 2),
    deltaTime.div(input.resetTimeThreshold)
  )

  // NOTE: preserved verbatim from web_ui — this `.times(...)` call discards
  // its result (should be `timeMultiplier = timeMultiplier.times(...)`).
  // Treated as a no-op so parity stays exact; a real fix is out of scope.
  timeMultiplier.times(input.halfMindUnlocked ? input.globalSpeedMult / 10 : 1)

  return Decimal.max(input.fastForwardAmount.times(input.baseResource), input.resourceMult.times(timeMultiplier))
}

export interface CalculatePotionValueInput {
  /** player.{reset}counter — current run time. */
  resetTime: number
  /** Precomputed `calculateOfferings` or `calculateObtainium` value. */
  resourceMult: Decimal
  /** Precomputed `calculateBaseOfferings` / `calculateBaseObtainium`. */
  baseResource: number
  /** getGQUpgradeEffect('halfMind', 'unlocked'). */
  halfMindUnlocked: boolean
  /** calculateGlobalSpeedMult() in web_ui. */
  globalSpeedMult: number
  /** resetTimeThreshold() in web_ui. */
  resetTimeThreshold: number
  /**
   * Product of the four potion-power multiplier effects: `potionBuff`,
   * `potionBuff2`, `potionBuff3` (singularity GQ) and
   * `octeractAutoPotionEfficiency` (octeract).
   */
  potionMultipliers: number
}

/**
 * Resource award for activating one potion. Combines a 7200-second
 * fast-forward (= 2h) computed against the current run time with the
 * stacked potion-power multipliers.
 */
export function calculatePotionValue(input: CalculatePotionValueInput): Decimal {
  const potionTimeValue = new Decimal(7200)
  const fastForwardMult = calculateFastForwardResourcesGlobal({
    resetTime: input.resetTime,
    fastForwardAmount: potionTimeValue,
    resourceMult: input.resourceMult,
    baseResource: input.baseResource,
    halfMindUnlocked: input.halfMindUnlocked,
    globalSpeedMult: input.globalSpeedMult,
    resetTimeThreshold: input.resetTimeThreshold
  })
  return fastForwardMult.times(input.potionMultipliers)
}
