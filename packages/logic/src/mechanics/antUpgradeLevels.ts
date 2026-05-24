// Ant-upgrade level helpers: free-level aggregator + true-level resolver.
// Lifted from:
//   packages/web_ui/src/Features/Ants/AntUpgrades/lib/free-levels.ts
//   packages/web_ui/src/Features/Ants/AntUpgrades/lib/total-levels.ts
//
// computeFreeAntUpgradeLevels pre-extracts every player-state input as a
// numeric field; calculateTrueAntLevel composes free levels with the
// corruption divisor and the c11-active branch. Both are pure given inputs.

export interface ComputeFreeAntUpgradeLevelsInput {
  /** CalcECC('reincarnation', player.challengecompletions[9]). */
  c9ReincarnationECC: number
  /** player.constantUpgrades[6]. */
  constantUpgrade6: number
  /** CalcECC('ascension', player.challengecompletions[11]) — multiplied by 12. */
  c11AscensionECC: number
  /** player.researches[97] — ×2. */
  research97: number
  /** player.researches[98] — ×2. */
  research98: number
  /** player.researches[102]. */
  research102: number
  /** player.researches[132] — ×2. */
  research132: number
  /** player.researches[200] — ×(1/200). */
  research200: number
  /** +getAchievementReward('freeAntUpgrades'). */
  freeAntUpgradesAchievementReward: number
  /** Globals.challenge15Rewards.bonusAntLevel.value — multiplies the sum. */
  challenge15BonusAntLevelValue: number
  /** player.currentChallenge.ascension === 11 — toggles the c11 bonus tail. */
  c11Active: boolean
  /** player.challengecompletions[8] — ×3 contribution iff c11Active. */
  c8Completions: number
  /** player.challengecompletions[9] — ×5 contribution iff c11Active. */
  c9Completions: number
}

/**
 * Total free ant-upgrade levels granted by passive bonuses. Sum-of-sources
 * (research, achievement, challenge ECC), then ×challenge-15-reward
 * multiplier, then optionally adds the c11-active floor-of-(3c8+5c9) tail.
 *
 * Important: the challenge-15 multiplier applies BEFORE the c11 tail (the
 * legacy ordering — the c11 add is post-multiplier).
 */
export function computeFreeAntUpgradeLevels (input: ComputeFreeAntUpgradeLevelsInput): number {
  let bonusLevels = 0
  bonusLevels += input.c9ReincarnationECC
  bonusLevels += Math.round(2000 * (1 - Math.pow(0.999, input.constantUpgrade6)))
  bonusLevels += 12 * input.c11AscensionECC
  bonusLevels += 2 * input.research97
  bonusLevels += 2 * input.research98
  bonusLevels += input.research102
  bonusLevels += 2 * input.research132
  bonusLevels += Math.floor((1 / 200) * input.research200)
  bonusLevels += input.freeAntUpgradesAchievementReward
  bonusLevels *= input.challenge15BonusAntLevelValue

  if (input.c11Active) {
    bonusLevels += Math.floor(
      3 * input.c8Completions + 5 * input.c9Completions
    )
  }
  return bonusLevels
}

export interface CalculateTrueAntLevelInput {
  /** player.ants.upgrades[antUpgrade] — current purchased level. */
  currentLevel: number
  /** Result of computeFreeAntUpgradeLevels (the global free-level pool — same
   * value across all upgrades). Capped by `currentLevel` in the formula. */
  freeLevels: number
  /** antUpgradeData[antUpgrade].exemptFromCorruption. */
  exemptFromCorruption: boolean
  /** player.corruptions.used.corruptionEffects('extinction'). Used as the
   * divisor when not exempt. */
  corruptionExtinctionDivisor: number
  /** player.currentChallenge.ascension === 11. In c11 the effective level
   * collapses to `min(currentLevel, freeLevels)` (no doubling). */
  c11Active: boolean
}

/**
 * Effective ant-upgrade level. Combines purchased levels + capped free
 * levels (min(purchased, freeLevels)), then divides by corruption.
 * c11 mode caps the contribution to just `min(purchased, freeLevels)`
 * without the additive purchased term.
 */
export function calculateTrueAntLevel (input: CalculateTrueAntLevelInput): number {
  const corruptionDivisor = input.exemptFromCorruption
    ? 1
    : input.corruptionExtinctionDivisor

  if (input.c11Active) {
    return Math.min(input.currentLevel, input.freeLevels) / corruptionDivisor
  }
  return (input.currentLevel + Math.min(input.currentLevel, input.freeLevels))
    / corruptionDivisor
}
