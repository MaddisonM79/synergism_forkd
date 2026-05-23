// Singularity-milestone bonuses, lifted from packages/web_ui/src/Calculate.ts.
//
// Every function in this file follows the same shape: read a single (or two)
// counter from the player — current `singularityCount` or all-time
// `highestSingularityCount` — count how many entries of a hardcoded threshold
// array have been crossed, and return a numeric bonus. The threshold arrays
// are data (not state) and live here alongside the formulas that use them.

// dprint-ignore
const singQuarkMilestoneThresholds = [
  5, 7, 10, 20, 35, 50, 65, 80, 90, 100, 121, 144, 150, 160, 166, 169, 170,
  175, 180, 190, 196, 200, 201, 202, 203, 204, 205, 210, 213, 216, 219, 225,
  228, 231, 234, 237, 240, 244, 248, 252, 256, 260, 264, 268, 272, 276, 280,
  284, 288, 290
]

const ambrosiaLuckSingThresholds1 = [35, 42, 49, 56, 63, 70, 77]
const ambrosiaLuckSingThresholds2 = [135, 142, 149, 156, 163, 170, 177]

const derpsmithSingCounts = [
  18,
  38,
  58,
  78,
  88,
  98,
  118,
  148,
  178,
  188,
  198,
  208,
  218,
  228,
  238,
  248
]

const immaculateAlchemyThresholds = [50, 90, 130, 170, 200, 217, 235, 253, 271, 289]

const inheritanceLevels = [2, 5, 10, 17, 26, 37, 50, 65, 82, 101, 220, 240, 260, 270, 277]
const inheritanceTokenValues = [1, 10, 25, 40, 75, 100, 150, 200, 250, 300, 350, 400, 500, 600, 750]

const bonusTokenLevels = [41, 58, 113, 163, 229]

const dilatedFiveLeafSingThresholds = [100, 150, 200, 225, 250, 255, 260, 265, 269, 272]

// ─── Quark milestone multiplier ────────────────────────────────────────────

/**
 * Compounds a 1.05× multiplier for every entry of `singQuarkMilestoneThresholds`
 * crossed by the current `player.singularityCount`. Resets with the
 * singularity; uses the live count, not the all-time max.
 */
export function calculateSingularityQuarkMilestoneMultiplier(singularityCount: number): number {
  let multiplier = 1
  for (const sing of singQuarkMilestoneThresholds) {
    if (singularityCount >= sing) {
      multiplier *= 1.05
    }
  }
  return multiplier
}

// ─── Base golden quarks earned at a singularity ────────────────────────────

export interface CalculateBaseGoldenQuarksInput {
  /** The singularity count being entered — exponent base for the minimum value. */
  singularity: number
  /** player.quarksThisSingularity — / 1e5 contribution. */
  quarksThisSingularity: number
  /** player.highestSingularityCount — capped at 10 for the first-ten +10 each bonus. */
  highestSingularityCount: number
}

/**
 * Base GQ award before any milestone / shop / GQ-upgrade multipliers.
 *   floor(100 * 1.04^singularity + quarksThisSingularity/1e5 + 10 * min(highest, 10))
 */
export function calculateBaseGoldenQuarks(input: CalculateBaseGoldenQuarksInput): number {
  const minimumValue = 100 * Math.pow(1.04, input.singularity)
  const contributionFromQuarks = input.quarksThisSingularity / 1e5
  const firstTenBonus = 10 * Math.min(input.highestSingularityCount, 10)
  return Math.floor(minimumValue + contributionFromQuarks + firstTenBonus)
}

// ─── Ambrosia luck milestone bonus ─────────────────────────────────────────

/**
 * Additive ambrosia-luck bonus from two singularity-count threshold tables:
 * +5 per entry of ambrosiaLuckSingThresholds1 crossed, +6 per entry of
 * ambrosiaLuckSingThresholds2 crossed. Uses the all-time max count.
 */
export function calculateSingularityAmbrosiaLuckMilestoneBonus(highestSingularityCount: number): number {
  let bonus = 0
  for (const sing of ambrosiaLuckSingThresholds1) {
    if (highestSingularityCount >= sing) {
      bonus += 5
    }
  }
  for (const sing of ambrosiaLuckSingThresholds2) {
    if (highestSingularityCount >= sing) {
      bonus += 6
    }
  }
  return bonus
}

// ─── Dilated Five Leaf bonus ───────────────────────────────────────────────

/**
 * Returns the fraction (0.00–0.10) representing how many of the
 * dilatedFiveLeafSingThresholds have been crossed by the all-time max
 * singularity count. The first un-crossed threshold's index / 100 is
 * returned; if all thresholds are crossed, returns thresholds.length / 100.
 */
export function calculateDilatedFiveLeafBonus(highestSingularityCount: number): number {
  for (let i = 0; i < dilatedFiveLeafSingThresholds.length; i++) {
    if (highestSingularityCount < dilatedFiveLeafSingThresholds[i]) return i / 100
  }
  return dilatedFiveLeafSingThresholds.length / 100
}

// ─── Derpsmith Cornucopia ──────────────────────────────────────────────────

/**
 * 1 + (count_of_thresholds_crossed × highestSingularityCount) / 100. The
 * count grows in coarse steps from derpsmithSingCounts; the per-count weight
 * is the all-time max singularity count itself.
 */
export function derpsmithCornucopiaBonus(highestSingularityCount: number): number {
  let counter = 0
  for (const sing of derpsmithSingCounts) {
    if (highestSingularityCount >= sing) {
      counter += 1
    }
  }
  return 1 + (counter * highestSingularityCount) / 100
}

// ─── Immaculate Alchemy ────────────────────────────────────────────────────

/**
 * 1 + 0.4 per immaculateAlchemyThreshold crossed by the current
 * singularityCount (not the all-time max).
 */
export function calculateImmaculateAlchemyBonus(singularityCount: number): number {
  let bonus = 1
  for (let i = 0; i < immaculateAlchemyThresholds.length; i++) {
    if (singularityCount >= immaculateAlchemyThresholds[i]) {
      bonus += 0.4
    }
  }
  return bonus
}

// ─── Inheritance Tokens ────────────────────────────────────────────────────

/**
 * Returns the inheritanceTokenValues entry for the highest
 * inheritanceLevels[i] that the player has crossed (1 ≤ i ≤ 15). Returns 0
 * if nothing crossed. Indexes 1..15 — index 0 is unused (matches the
 * original web_ui loop).
 */
export function inheritanceTokens(highestSingularityCount: number): number {
  for (let i = 15; i > 0; i--) {
    if (highestSingularityCount >= inheritanceLevels[i]) {
      return inheritanceTokenValues[i]
    }
  }
  return 0
}

// ─── Sum of exalt completions ──────────────────────────────────────────────

/**
 * Sum of `.completions` across every singularity challenge. Web_ui passes
 * the precomputed array (Object.values(player.singularityChallenges)
 * .map(c => c.completions)) and logic reduces.
 */
export function sumOfExaltCompletions(completionsList: number[]): number {
  return completionsList.reduce((a, b) => a + b, 0)
}

// ─── Singularity bonus token mult ──────────────────────────────────────────

/**
 * Returns 1 + 0.02 × i, where i is the highest index 1..5 such that the
 * player's all-time max singularity count is ≥ bonusTokenLevels[i-1].
 */
export function singularityBonusTokenMult(highestSingularityCount: number): number {
  for (let i = 5; i > 0; i--) {
    if (highestSingularityCount >= bonusTokenLevels[i - 1]) {
      return 1 + 0.02 * i
    }
  }
  return 1
}
