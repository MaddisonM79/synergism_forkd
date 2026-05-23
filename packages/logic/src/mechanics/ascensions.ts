// Ascension-related formulas lifted from packages/web_ui/src/Calculate.ts.
// Currently just the per-reset ascension count. Web_ui collects the
// `ascensionCountMultStats` StatLine values into an array and passes them in;
// logic reduces and floors.

export interface CalculateAscensionCountInput {
  /**
   * player.singularityChallenges.limitedAscensions.enabled — when true the
   * count is capped at 1 (Exalt 3 forces one ascension at a time).
   */
  limitedAscensionsEnabled: boolean
  /**
   * Precomputed multiplier contributions (web_ui:
   *   ascensionCountMultStats.map(s => s.stat())
   * ). Product is floored to give the final count.
   */
  ascensionCountMults: number[]
}

/**
 * `1` when Exalt 3 is active; otherwise
 * `floor(prod(ascensionCountMults))`. Multiplier contributions can include
 * fractional and >1 boosts; flooring handles the off-by-one rounding.
 */
export function calculateAscensionCount(input: CalculateAscensionCountInput): number {
  if (input.limitedAscensionsEnabled) {
    return 1
  }
  return Math.floor(input.ascensionCountMults.reduce((a, b) => a * b, 1))
}
