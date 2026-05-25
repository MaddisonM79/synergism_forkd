// Small singularity-related helpers, lifted from
// packages/web_ui/src/singularity.ts. None of these warrant their own module
// but they share the "tiny pure helper called by singularity UI / cost
// flow" theme:
//
//   - maxSingularityLookahead: how many singularities the buy-multi prompt
//     can preview, given the player's three lookahead-bonus upgrades.
//   - goldenQuarkCost: wraps the GQ-cost result with a `costReduction`
//     diff against a fixed 10000-GQ baseline (for the UI's "you saved X
//     GQ" badge).
//   - calculateNextSpike: finds the next singularity-penalty threshold the
//     player will cross, accounting for shop/ambrosia reductions.

// Singularity counts at which a new penalty tier activates. Sorted
// ascending; calculateNextSpike walks the array and returns the first
// threshold past the player's current adjusted count.
const singularityPenaltyThresholds = [11, 26, 37, 51, 101, 151, 201, 216, 230, 270]

// Base cost of one golden quark, used as the baseline for the "you saved X"
// display in the GQ-buy prompt.
const GOLDEN_QUARK_BASE_COST = 10000

export interface MaxSingularityLookaheadInput {
  /**
   * True when the buy-multi prompt is in "show me what's possible" mode.
   * The legacy `nonZero` parameter — when false, lookahead is hardcoded 0
   * (player is just viewing the current sing, not previewing forward).
   */
  nonZero: boolean
  /** getGQUpgradeEffect('singFastForward', 'lookahead'). */
  singFastForwardLookahead: number
  /** getGQUpgradeEffect('singFastForward2', 'lookahead'). */
  singFastForward2Lookahead: number
  /** getOcteractUpgradeEffect('octeractFastForward', 'lookahead'). */
  octeractFastForwardLookahead: number
}

/**
 * Max number of singularities the buy-multi prompt previews. Always returns
 * 0 when `nonZero` is false; otherwise sums the three lookahead bonuses
 * (default 1 + sum of GQ + octeract effects).
 */
export function maxSingularityLookahead (input: MaxSingularityLookaheadInput): number {
  if (!input.nonZero) {
    return 0
  }
  return 1 + input.singFastForwardLookahead + input.singFastForward2Lookahead + input.octeractFastForwardLookahead
}

export interface GoldenQuarkCostResult {
  /** The actual per-GQ cost (passed through). */
  cost: number
  /**
   * `max(0, 10000 - cost)` — how much cheaper than the 10000-GQ baseline
   * the current cost is. Used by the UI to display "you saved X" badges.
   */
  costReduction: number
}

/**
 * Wraps a calculated GQ cost with its `costReduction` diff against the
 * 10000-GQ baseline. The reduction is floored at 0 — if cost > 10000 (e.g.
 * inside a debuffing challenge), no reduction is shown.
 */
export function goldenQuarkCost (cost: number): GoldenQuarkCostResult {
  return {
    cost,
    costReduction: Math.max(0, GOLDEN_QUARK_BASE_COST - cost)
  }
}

export interface CalculateNextSpikeInput {
  /** The player's raw singularity count being evaluated. */
  singularityCount: number
  /**
   * Shop/ambrosia singularity reductions. Subtracted from each threshold
   * when checking if the player has crossed it — matches the
   * constitutiveSingularityCount logic used elsewhere in singularity math.
   */
  singularityReductions: number
}

/**
 * Returns the next singularity-penalty threshold the player will cross,
 * or `-1` if they're past all of them. Each threshold is offset by the
 * player's singularityReductions, so the spike fires later for players
 * with reduction upgrades.
 */
export function calculateNextSpike (input: CalculateNextSpikeInput): number {
  for (const sing of singularityPenaltyThresholds) {
    if (sing + input.singularityReductions > input.singularityCount) {
      return sing + input.singularityReductions
    }
  }
  return -1
}
