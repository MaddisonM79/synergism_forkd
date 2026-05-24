// EXP/level math + max-purchase planner for rune blessings and rune spirits.
// The EXP↔level math is bit-identical between the two — only the
// `costCoefficient` and `levelsPerOOM` differ per rune — so this module owns
// the closed-form invert plus the budget-aware purchase planner once, and
// both web_ui sides (RuneBlessings.ts / RuneSpirits.ts) shim into it.
//
// Precedent: packages/logic/src/mechanics/runeLevels.ts does the same for
// top-level runes. The shapes are similar but not identical:
//   - top-level runes have no per-call buy-amount cap (`upperLimit`)
//   - top-level runes have an `isUnlocked` gate; blessings/spirits gate at
//     the web_ui layer before calling, so we don't expose it here
//   - blessings use a dynamic `minOfferingsFloor` derived from the rune's
//     current EXP (to avoid integer-precision loss at MAX_SAFE_INTEGER
//     boundaries); spirits use a constant `1`. The caller passes the
//     already-evaluated Decimal so this module stays pure-input.

import { Decimal } from '../math/bignum'

/**
 * EXP required to *reach* a target level, starting from 0 EXP.
 * Formula: `costCoefficient * (10^(level/levelsPerOOM) - 1)`.
 *
 * The `-1` zeroes out at level 0. `levelsPerOOM` is the number of levels
 * between each 10x-EXP step.
 */
export function runeUpgradeEXPToLevel (
  costCoefficient: Decimal,
  level: number,
  levelsPerOOM: number
): Decimal {
  return costCoefficient.times(Decimal.pow(10, level / levelsPerOOM).minus(1))
}

/**
 * EXP still needed to reach a target level, given the rune-upgrade's current
 * EXP. Clamped at zero — querying a level you've already passed returns 0,
 * not negative debt.
 */
export function runeUpgradeEXPLeftToLevel (
  costCoefficient: Decimal,
  targetLevel: number,
  levelsPerOOM: number,
  currentRuneEXP: Decimal
): Decimal {
  return Decimal.max(
    0,
    runeUpgradeEXPToLevel(costCoefficient, targetLevel, levelsPerOOM).minus(currentRuneEXP)
  )
}

/**
 * Closed-form inverse of `runeUpgradeEXPToLevel`: given current EXP, returns
 * the integer level reached, plus a `needsFloatBump` flag.
 *
 * Floating-point imprecision in `Decimal.log10` can leave the floor one short
 * when current EXP exactly equals the EXP-for-`level+1` boundary. The flag
 * encodes whether `runeUpgradeEXPToLevel(coeff, levels+1, oom) <= currentEXP`
 * — when true, the caller should use `levels + 1` instead. (Legacy code in
 * web_ui's `updateLevelsFromEXP` computes this via
 * `computeEXPLeftToLevel(level+1).eq(0)`, which is equivalent because
 * EXPLeftToLevel clamps at zero.)
 */
export function runeUpgradeLevelFromEXP (
  currentRuneEXP: Decimal,
  costCoefficient: Decimal,
  levelsPerOOM: number
): { levels: number; needsFloatBump: boolean } {
  const levels = Math.floor(
    levelsPerOOM * Decimal.log10(currentRuneEXP.div(costCoefficient).plus(1))
  )
  const needsFloatBump = runeUpgradeEXPToLevel(costCoefficient, levels + 1, levelsPerOOM).lte(currentRuneEXP)
  return { levels, needsFloatBump }
}

export interface MaxRuneUpgradePurchaseInput {
  /** Per-upgrade cost coefficient (Decimal). */
  costCoefficient: Decimal
  /** Per-upgrade levels-per-OOM slope. */
  levelsPerOOM: number
  /** Current purchased level. */
  currentLevel: number
  /** Current accumulated EXP. */
  currentRuneEXP: Decimal
  /** EXP yielded per offering — caller pre-evaluates. */
  runeEXPPerOffering: Decimal
  /** Offerings budget the player wants to spend. */
  budget: Decimal
  /**
   * Per-call cap on levels purchased (player.runeBlessingBuyAmount /
   * player.runeSpiritBuyAmount). Unlike top-level runes, blessings and
   * spirits respect a player-chosen "buy at most N levels" cap.
   */
  upperLimit: number
  /**
   * Minimum offerings to display when the budget can't afford even one level.
   * - Blessings: `ceil(currentRuneEXP / (runeEXPPerOffering * Number.MAX_SAFE_INTEGER))`
   *   so tiny EXP increments are still representable.
   * - Spirits: `new Decimal(1)` (legacy behavior).
   * The caller computes this fresh each call.
   */
  minOfferingsFloor: Decimal
}

export interface MaxRuneUpgradePurchaseResult {
  /** Number of levels gained — `1` even when budget can't afford one, so the
   * UI displays cost-to-next-level. `0` only on negative-budget short-circuit. */
  levels: number
  /** Total EXP required to reach `currentLevel + levels` (or `currentLevel + 1`
   * in the can't-afford-one fallback). */
  expRequired: Decimal
  /** Offerings actually required — floored at `minOfferingsFloor`. */
  offerings: Decimal
}

/**
 * Plans the largest affordable purchase capped by `upperLimit`. Mirrors
 * `maxRuneLevelPurchase` from runeLevels.ts but adds the per-call cap and
 * the dynamic `minOfferingsFloor`.
 *
 * Algorithm: convert `budget` to total-available-EXP by adding
 * `budget * runeEXPPerOffering` to current EXP, invert the EXP→level formula
 * in closed form, floor it, subtract currentLevel, cap at `upperLimit`. If
 * the result is 0, fall back to "cost of the very next level" so the UI has
 * something to display.
 *
 * Returns {levels: 0, exp: 0, offerings: 0} for negative budgets — matching
 * the legacy null-case shape.
 */
export function maxRuneUpgradePurchase (input: MaxRuneUpgradePurchaseInput): MaxRuneUpgradePurchaseResult {
  if (input.budget.lt(0)) {
    return { levels: 0, expRequired: new Decimal(0), offerings: new Decimal(0) }
  }

  const totalEXPAvailable = input.budget.times(input.runeEXPPerOffering).add(input.currentRuneEXP)
  const maxLevel = Math.floor(
    input.levelsPerOOM * Decimal.log10(totalEXPAvailable.div(input.costCoefficient).plus(1))
  )
  const levelsGained = Math.min(input.upperLimit, Math.max(0, maxLevel - input.currentLevel))

  if (levelsGained === 0) {
    const nextLevelEXP = runeUpgradeEXPToLevel(
      input.costCoefficient,
      input.currentLevel + 1,
      input.levelsPerOOM
    )
    const offeringsRequired = Decimal.max(
      input.minOfferingsFloor,
      nextLevelEXP.minus(input.currentRuneEXP).div(input.runeEXPPerOffering).ceil()
    )
    return { levels: 1, expRequired: nextLevelEXP, offerings: offeringsRequired }
  }

  const expRequired = runeUpgradeEXPToLevel(
    input.costCoefficient,
    input.currentLevel + levelsGained,
    input.levelsPerOOM
  )
  const offeringsRequired = Decimal.max(
    input.minOfferingsFloor,
    expRequired.minus(input.currentRuneEXP).div(input.runeEXPPerOffering).ceil()
  )
  return { levels: levelsGained, expRequired, offerings: offeringsRequired }
}
