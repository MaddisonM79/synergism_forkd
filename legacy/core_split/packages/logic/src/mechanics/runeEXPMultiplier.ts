// Universal rune-EXP-per-offering multiplier, lifted from
// packages/web_ui/src/Runes.ts. The shape:
//
//   (additive multiplier) × (product of all-rune multipliers) × (recycleMult)
//
// Three groups of inputs:
//   1. Additive: base 1, plus C1 completion bonus + per-C1 scaling,
//      researches 5x2 / 5x3, particle upgrade 3x1 (which scales with the
//      caller's `purchasedLevels` to discourage low-level dump-and-respec).
//   2. Multiplicative: researches 4x16 / 4x17, cube upgrade 32 × ascension
//      counter, constant upgrade 8, challenge-15 rune-EXP reward multiplier.
//   3. Recycle: the inverse of recycle/salvage chance — passed in from
//      web_ui's calculateSalvageRuneEXPMultiplier.

import { Decimal } from '../math/bignum'

export interface UniversalRuneEXPMultInput {
  /** rune.level — the per-rune `purchasedLevels` parameter. Feeds the
   *  particle-upgrade-3x1 additive contribution. */
  purchasedLevels: number
  /** player.highestchallengecompletions[1]. Bonus is `min(1, n) + 0.04 * n`. */
  c1Completions: number
  /** player.researches[22]. +0.6 per level. */
  research22: number
  /** player.researches[23]. +0.3 per level. */
  research23: number
  /** player.upgrades[71]. Particle upgrade 3x1: adds `n * purchasedLevels / 25`. */
  upgrade71: number
  /** player.researches[91]. ×(1 + n/20). */
  research91: number
  /** player.researches[92]. ×(1 + n/20). */
  research92: number
  /** player.ascensionCounter — seconds in current ascension. Feeds the cube-upgrade-32 bonus. */
  ascensionCounter: number
  /** player.cubeUpgrades[32]. Bonus ×(1 + ascensionCounter/1000 * n). */
  cubeUpgrade32: number
  /** player.constantUpgrades[8]. ×(1 + n/10). */
  constantUpgrade8: number
  /** G.challenge15Rewards.runeExp.value — number multiplier from C15 reward formula. */
  challenge15RuneExpReward: number
  /**
   * calculateSalvageRuneEXPMultiplier() — the recycle/salvage multiplier
   * (the inverse of effective recycle chance). Passed in to avoid pulling
   * the whole salvage math chain into this module.
   */
  salvageRuneEXPMultiplier: Decimal
}

/**
 * Universal multiplier applied to the base EXP-per-offering for every rune.
 * Pure function over the input bundle.
 *
 * Returns `additive × multiplicative × recycle`, where `additive` and
 * `multiplicative` are themselves a sum-of-contributions and a
 * product-of-contributions as documented above. Result type is Decimal
 * because the C15 reward and salvage multiplier can both exceed Number range.
 */
export function universalRuneEXPMult (input: UniversalRuneEXPMultInput): Decimal {
  const allRuneExpAdditiveMultiplier = 1
    // C1 completion: +1 for any completion, +0.04 per completion
    + Math.min(1, input.c1Completions)
    + (0.4 / 10) * input.c1Completions
    // Research 5x2
    + 0.6 * input.research22
    // Research 5x3
    + 0.3 * input.research23
    // Particle upgrade 3x1 — scales with purchasedLevels
    + (input.upgrade71 * input.purchasedLevels) / 25

  const allRuneExpMultiplier = [
    // Research 4x16
    1 + input.research91 / 20,
    // Research 4x17
    1 + input.research92 / 20,
    // Cube Upgrade 32 × ascension time
    1 + (input.ascensionCounter / 1000) * input.cubeUpgrade32,
    // Constant Upgrade 8
    1 + (1 / 10) * input.constantUpgrade8,
    // Challenge 15 reward
    input.challenge15RuneExpReward
  ].reduce((x, y) => x.times(y), new Decimal('1'))

  return allRuneExpMultiplier.times(allRuneExpAdditiveMultiplier).times(input.salvageRuneEXPMultiplier)
}
