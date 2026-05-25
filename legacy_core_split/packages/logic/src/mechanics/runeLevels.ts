// Rune EXP / level math, lifted from packages/web_ui/src/Runes.ts. Each
// function takes the small per-rune snapshot (`costCoefficient`,
// `levelsPerOOM`, sometimes `currentEXP` and `level`) and is pure Decimal
// math. The web_ui side owns the rune data table and the offering-spend
// flow; this module owns the closed-form EXP↔level inversion plus the
// "given a budget, how many levels can I buy" planner.

import { Decimal } from '../math/bignum'

/**
 * EXP required to *reach* a target rune level, starting from 0 EXP.
 * Formula: `costCoefficient * (10^(level/levelsPerOOM) - 1)`.
 *
 * The `-1` zeroes out at level 0. `levelsPerOOM` is the number of levels
 * between each 10x-EXP step.
 */
export function runeEXPToLevel (costCoefficient: Decimal, level: number, levelsPerOOM: number): Decimal {
  return costCoefficient.times(Decimal.pow(10, level / levelsPerOOM).minus(1))
}

/**
 * EXP still needed to *reach* a target rune level, given the rune's current
 * EXP. Clamped at zero (you don't lose EXP by querying a level you've
 * already passed).
 */
export function runeEXPLeftToLevel (
  costCoefficient: Decimal,
  targetLevel: number,
  levelsPerOOM: number,
  currentRuneEXP: Decimal
): Decimal {
  return Decimal.max(0, runeEXPToLevel(costCoefficient, targetLevel, levelsPerOOM).minus(currentRuneEXP))
}

/**
 * Offerings required to reach a target rune level given the per-offering
 * EXP rate. Floored at 1 — the UI never displays "0 offerings to next
 * level" for an unowned level even if floating-point imprecision would say
 * so.
 */
export function runeOfferingsToLevel (
  costCoefficient: Decimal,
  targetLevel: number,
  levelsPerOOM: number,
  currentRuneEXP: Decimal,
  runeEXPPerOffering: Decimal
): Decimal {
  return Decimal.max(
    1,
    runeEXPLeftToLevel(costCoefficient, targetLevel, levelsPerOOM, currentRuneEXP)
      .div(runeEXPPerOffering)
      .ceil()
  )
}

/**
 * Closed-form inverse of `runeEXPToLevel`: given a rune's current EXP,
 * returns the integer level reached. Equivalent to
 * `floor(levelsPerOOM * log10(EXP/costCoeff + 1))`.
 *
 * Used by the rune-EXP→level resync (after gaining EXP). Does NOT include
 * the float-imprecision +1 bump that web_ui's `updateLevelsFromEXP` does
 * afterward — that fix-up uses `runeEXPLeftToLevel` to detect when the
 * floor undercounted by exactly one.
 */
export function runeLevelFromEXP (currentRuneEXP: Decimal, costCoefficient: Decimal, levelsPerOOM: number): number {
  return Math.floor(levelsPerOOM * Decimal.log10(currentRuneEXP.div(costCoefficient).plus(1)))
}

export interface MaxRuneLevelPurchaseInput {
  /** rune.costCoefficient. */
  costCoefficient: Decimal
  /** rune.levelsPerOOM + rune.levelsPerOOMIncrease() — combined slope. */
  levelsPerOOM: number
  /** rune.level — current purchased level (not the floored-from-EXP level). */
  currentLevel: number
  /** rune.runeEXP — current accumulated EXP. */
  currentRuneEXP: Decimal
  /** rune.runeEXPPerOffering(currentLevel) — already evaluated by caller. */
  runeEXPPerOffering: Decimal
  /** Offerings budget the player wants to spend. */
  budget: Decimal
  /**
   * rune.isUnlocked() — returning 0/0/0 below an unlocked rune is the
   * legacy behavior the caller relies on for UI display gating.
   */
  isUnlocked: boolean
}

export interface MaxRuneLevelPurchaseResult {
  /** Number of levels gained — `0` when locked or budget is negative. */
  levels: number
  /** Total EXP required to reach `currentLevel + levels`. */
  expRequired: Decimal
  /** Offerings actually consumed (≥1 if any level affordable). */
  offerings: Decimal
}

/**
 * Plans the largest level purchase affordable with a given offerings budget.
 *
 * Algorithm: convert `budget` to "total available EXP" by adding
 * `budget * runeEXPPerOffering` to `currentRuneEXP`. Invert the EXP→level
 * formula in closed form to find the maximum level reachable, floor it,
 * subtract currentLevel. If that's 0, fall back to "what's the cost of the
 * very next level" so the UI has something to display.
 *
 * Returns {levels: 0, exp: 0, offerings: 0} for locked runes / negative
 * budgets — matching the legacy null-case shape.
 */
export function maxRuneLevelPurchase (input: MaxRuneLevelPurchaseInput): MaxRuneLevelPurchaseResult {
  if (!input.isUnlocked || input.budget.lt(0)) {
    return { levels: 0, expRequired: new Decimal(0), offerings: new Decimal(0) }
  }

  const totalEXPAvailable = input.budget.times(input.runeEXPPerOffering).add(input.currentRuneEXP)
  // Same closed-form invert as runeLevelFromEXP, but with the budget-augmented
  // EXP rather than just the current EXP.
  const maxLevel = Math.floor(input.levelsPerOOM * Decimal.log10(totalEXPAvailable.div(input.costCoefficient).plus(1)))
  const levelsGained = Math.max(0, maxLevel - input.currentLevel)

  if (levelsGained === 0) {
    // Budget too small to buy a level; report cost-to-next-level so the UI
    // can display the gap.
    const nextLevelEXP = runeEXPToLevel(input.costCoefficient, input.currentLevel + 1, input.levelsPerOOM)
    const offeringsRequired = Decimal.max(
      1,
      nextLevelEXP.minus(input.currentRuneEXP).div(input.runeEXPPerOffering).ceil()
    )
    return { levels: 1, expRequired: nextLevelEXP, offerings: offeringsRequired }
  }

  const expRequired = runeEXPToLevel(input.costCoefficient, input.currentLevel + levelsGained, input.levelsPerOOM)
  // Recompute offerings — the planner may have undershot the budget if the
  // last level didn't quite fit.
  const offeringsRequired = Decimal.max(
    1,
    expRequired.minus(input.currentRuneEXP).div(input.runeEXPPerOffering).ceil()
  )
  return { levels: levelsGained, expRequired, offerings: offeringsRequired }
}
