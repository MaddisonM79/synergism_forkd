// Reset-time-threshold + automatic-research-obtainium calculator. Lifted from
//   packages/web_ui/src/Calculate.ts (resetTimeThreshold + calculateResearchAutomaticObtainium)
//
// resetTimeThreshold is a 2-input affine function. calculateResearchAutomaticObtainium
// is the offline obtainium gain from research idle production; the caller pre-
// extracts every input including the already-computed obtainium / base-obtainium
// / ant-sacrifice-obtainium / global-speed multipliers from Calculate.ts logic.

import { Decimal } from '../math/bignum'

export interface ResetTimeThresholdInput {
  /** player.campaigns.timeThresholdReduction — subtracted from the base 10. */
  campaignTimeThresholdReduction: number
}

/** Reset-time threshold (in seconds): `10 − campaignTimeThresholdReduction`. */
export function resetTimeThreshold (input: ResetTimeThresholdInput): number {
  return 10 - input.campaignTimeThresholdReduction
}

export interface ResearchAutomaticObtainiumInput {
  /** Tick delta time in seconds. */
  deltaTime: number
  /** player.currentChallenge.ascension — short-circuits to 0 when === 14. */
  ascensionChallenge: number
  /** player.researches[61] — ×0.5 contribution to the multiplier. */
  research61: number
  /** player.researches[62] — ×0.1 contribution. */
  research62: number
  /** player.cubeUpgrades[3] — ×0.8 contribution. */
  cubeUpgrade3: number
  /** player.cubeUpgrades[47] — non-zero enables the ant-sacrifice branch. */
  cubeUpgrade47: number
  /** Pre-evaluated calculateObtainium(false). */
  resourceMult: Decimal
  /** Pre-evaluated calculateGlobalSpeedMult(). */
  globalSpeedMult: number
  /** Pre-evaluated resetTimeThreshold() — used as the time-penalty divisor. */
  resetTimeDivisor: number
  /** player.reincarnationcounter — capped at resetTimeDivisor. */
  reincarnationcounter: number
  /** Pre-evaluated calculateBaseObtainium() — returns number, will be wrapped
   * by Decimal.max which accepts mixed types. */
  baseObtainium: number
  /** Pre-evaluated calculateAntSacrificeObtainium(antSacrificeStageMult, false). */
  antSacrificeObtainium: Decimal
  /** Pre-evaluated thresholdModifiers().antSacrificeObtainiumMult — only
   * used if cubeUpgrade47 > 0 (no need to compute when disabled). */
  antSacrificeStageMult: number
  /** player.antSacrificeTimer — capped at resetTimeDivisor in ant branch. */
  antSacrificeTimer: number
}

/**
 * Per-tick automatic obtainium from research idle gain. Returns 0 in
 * challenge 14 OR when the per-upgrade multiplier is 0 (no enabling
 * researches / cube upgrades).
 *
 * Compares three obtainium sources (base / current-resource × time-penalty /
 * ant-sacrifice × time-penalty when cubeUpgrade47 > 0) and takes the max,
 * scaled by deltaTime / resetTimeDivisor × multiplier.
 */
export function calculateResearchAutomaticObtainium (input: ResearchAutomaticObtainiumInput): Decimal {
  if (input.ascensionChallenge === 14) {
    return Decimal.fromString('0')
  }

  const multiplier = 0.5 * input.research61
    + 0.1 * input.research62
    + 0.8 * input.cubeUpgrade3

  if (multiplier === 0) {
    return Decimal.fromString('0')
  }

  const timePenaltyMult = Math.min(1, input.reincarnationcounter / input.resetTimeDivisor)
  const nonBaseValue = input.resourceMult.times(input.globalSpeedMult).times(timePenaltyMult)

  let nonBaseAntValue = Decimal.fromString('0')
  if (input.cubeUpgrade47 > 0) {
    const antTimePenaltyMult = Math.min(1, input.antSacrificeTimer / input.resetTimeDivisor)
    nonBaseAntValue = input.antSacrificeObtainium.times(input.globalSpeedMult).times(antTimePenaltyMult)
  }

  return Decimal.max(input.baseObtainium, Decimal.max(nonBaseValue, nonBaseAntValue))
    .times(input.deltaTime)
    .div(input.resetTimeDivisor)
    .times(multiplier)
}
