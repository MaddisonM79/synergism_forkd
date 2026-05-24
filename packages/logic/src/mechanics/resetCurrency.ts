// Per-tick prestige / transcend / reincarnation point-gain calculator. Lifted
// from packages/web_ui/src/Synergism.ts (resetCurrency).
//
// Three independent point formulas with shared challenge-overrides:
//   - Transcension Challenge 5 forces prestigePow to 0.01.
//   - Reincarnation Challenge 10 forces prestigePow to 1e-4 and
//     transcendPow to 0.001.
//   - Ascension Challenge 12 zeroes reincarnationPointGain.
//
// Caller pre-evaluates CalcECC('transcend', challengecompletions[5]) as
// `ecc5`, the deflation table lookup as `deflationMultiplier`, the
// particleGain achievement reward as `particleGainReward`, and passes the
// G.acceleratorEffect Decimal through directly.

import { Decimal } from '../math/bignum'

export interface ResetCurrencyInput {
  /** Pre-evaluated `CalcECC('transcend', player.challengecompletions[5])` —
   * contributes `+ ecc5 / 100` to the base prestigePow of 0.5. */
  ecc5: number
  /** player.currentChallenge.transcension — when === 5, forces prestigePow
   * to 0.01 and disables the upgrade-16 multiplier. */
  transcensionChallenge: number
  /** player.currentChallenge.reincarnation — when === 10, forces prestigePow
   * to 1e-4, transcendPow to 0.001, and disables both upgrade multipliers.
   * When non-zero, also re-exponents reincarnationPointGain by 0.01. */
  reincarnationChallenge: number
  /** player.currentChallenge.ascension — when === 12, zeroes
   * reincarnationPointGain after all other calculations. */
  ascensionChallenge: number
  /** Pre-evaluated `G.deflationMultiplier[player.corruptions.used.deflation]`
   * — multiplies prestigePow, and used as the upgrade-16 acceleratorEffect
   * exponent scaler (1/3 * deflationMultiplier). */
  deflationMultiplier: number
  /** player.coinsThisPrestige — base for the prestige-point formula
   * `floor((coinsThisPrestige / 1e12) ^ prestigePow)`. */
  coinsThisPrestige: Decimal
  /** player.coinsThisTranscension — base for the transcend-point formula
   * `floor((coinsThisTranscension / 1e100) ^ transcendPow)`. */
  coinsThisTranscension: Decimal
  /** player.transcendShards — base for the reincarnation-point formula
   * `floor((transcendShards / 1e300) ^ 0.01)`. */
  transcendShards: Decimal
  /** player.upgrades[16] — when > 0.5 and outside t-chal 5 / r-chal 10,
   * multiplies prestigePointGain by min(10^1e33, acceleratorEffect ^
   * (deflationMultiplier / 3)). */
  upgrade16: number
  /** player.upgrades[44] — when > 0.5 and outside t-chal 5 / r-chal 10,
   * multiplies transcendPointGain by min(1e6, 1.01 ^ transcendCount). */
  upgrade44: number
  /** player.upgrades[65] — when > 0.5, multiplies reincarnationPointGain by 5. */
  upgrade65: number
  /** player.transcendCount — exponent base for the upgrade-44 multiplier. */
  transcendCount: number
  /** G.acceleratorEffect — Decimal base for the upgrade-16 prestige multiplier. */
  acceleratorEffect: Decimal
  /** Pre-evaluated `+getAchievementReward('particleGain')` — multiplied into
   * reincarnationPointGain (unconditional). */
  particleGainReward: number
}

export interface ResetCurrencyResult {
  prestigePointGain: Decimal
  transcendPointGain: Decimal
  reincarnationPointGain: Decimal
}

/**
 * Compute the three per-tick reset-currency point-gain values. Pure: no
 * mutation of the input; returns a fresh result the caller writes back
 * to `G.prestigePointGain` / `G.transcendPointGain` / `G.reincarnationPointGain`.
 */
export function resetCurrency (input: ResetCurrencyInput): ResetCurrencyResult {
  let prestigePow = 0.5 + input.ecc5 / 100
  let transcendPow = 0.03

  if (input.transcensionChallenge === 5) {
    prestigePow = 0.01
  }
  if (input.reincarnationChallenge === 10) {
    prestigePow = 1e-4
    transcendPow = 0.001
  }
  prestigePow *= input.deflationMultiplier

  let prestigePointGain = Decimal.floor(
    Decimal.pow(input.coinsThisPrestige.dividedBy(1e12), prestigePow)
  )
  if (
    input.upgrade16 > 0.5
    && input.transcensionChallenge !== 5
    && input.reincarnationChallenge !== 10
  ) {
    prestigePointGain = prestigePointGain.times(
      Decimal.min(
        Decimal.pow(10, 1e33),
        Decimal.pow(
          input.acceleratorEffect,
          (1 / 3) * input.deflationMultiplier
        )
      )
    )
  }

  let transcendPointGain = Decimal.floor(
    Decimal.pow(input.coinsThisTranscension.dividedBy(1e100), transcendPow)
  )
  if (
    input.upgrade44 > 0.5
    && input.transcensionChallenge !== 5
    && input.reincarnationChallenge !== 10
  ) {
    transcendPointGain = transcendPointGain.times(
      Decimal.min(1e6, Decimal.pow(1.01, input.transcendCount))
    )
  }

  let reincarnationPointGain = Decimal.floor(
    Decimal.pow(input.transcendShards.dividedBy(1e300), 0.01)
  )
  if (input.reincarnationChallenge !== 0) {
    reincarnationPointGain = Decimal.pow(reincarnationPointGain, 0.01)
  }
  reincarnationPointGain = reincarnationPointGain.times(input.particleGainReward)
  if (input.upgrade65 > 0.5) {
    reincarnationPointGain = reincarnationPointGain.times(5)
  }
  if (input.ascensionChallenge === 12) {
    reincarnationPointGain = new Decimal('0')
  }

  return { prestigePointGain, transcendPointGain, reincarnationPointGain }
}
