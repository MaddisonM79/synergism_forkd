import { Decimal } from '../math/bignum'
import type { DecimalSource } from '../math/bignum'

// Pure subroutines from packages/web_ui/src/Calculate.ts. Each takes its
// inputs as precomputed numbers — the surrounding StatLine reductions stay in
// web_ui (those are essentially aggregators over per-line `stat()` callbacks,
// which still read from player/G state).

// ─── Global speed multiplier ───────────────────────────────────────────────

export interface GlobalSpeedMultInput {
  /**
   * Product of the DR-enabled multiplier StatLines (web_ui:
   *   allGlobalSpeedStats.reduce((a, b) => a * b.stat(), 1)
   * ). DR branches apply only to this leg.
   */
  normalMult: number
  /**
   * Product of the DR-ignored multiplier StatLines (web_ui:
   *   allGlobalSpeedIgnoreDRStats.reduce((a, b) => a * b.stat(), 1)
   * ). Multiplied straight through.
   */
  immaculateMult: number
  /**
   * Platonic upgrade 7 exponent power — calculatePlatonic7UpgradePower() in
   * web_ui, = 1 - player.platonicUpgrades[7] / 30. Only used in the
   * normalMult < 1 branch.
   */
  drPower: number
}

/**
 * Combines two precomputed multiplier legs with diminishing-returns
 * thresholds on the normal leg:
 *   - normalMult > 100   → sqrt(normalMult) * 10
 *   - normalMult < 1     → normalMult ^ drPower
 *   - otherwise          → unchanged
 *
 * Returns the product of the (possibly-transformed) normal leg and the
 * immaculate leg. The "verySlow" / "veryFast" achievement awards stay in the
 * web_ui shim — they're side effects, not part of the multiplier computation.
 */
export function calculateGlobalSpeedMult(input: GlobalSpeedMultInput): number {
  let normalMult = input.normalMult
  if (normalMult > 100) {
    normalMult = Math.pow(normalMult, 0.5) * 10
  } else if (normalMult < 1) {
    normalMult = Math.pow(normalMult, input.drPower)
  }
  return normalMult * input.immaculateMult
}

// ─── Ascension speed multiplier ────────────────────────────────────────────

export interface AscensionSpeedMultInput {
  /**
   * Product of the ascension-speed StatLines (web_ui:
   *   allAscensionSpeedStats.reduce((a, b) => a * b.stat(), 1)
   * ).
   */
  base: number
  /**
   * Sum of three GQ / shop upgrade contributions (web_ui:
   *   calculateAscensionSpeedExponentSpread()
   * ). Applied symmetrically around 1 — speeds get faster, slows get slower.
   */
  exponentSpread: number
}

/**
 * Applies an exponent-spread transformation to the precomputed base
 * ascension-speed multiplier:
 *   - base < 1  → base ^ (1 - spread)   (slower runs get more punishing)
 *   - base >= 1 → base ^ (1 + spread)   (faster runs get more rewarding)
 */
export function calculateAscensionSpeedMult(input: AscensionSpeedMultInput): number {
  return input.base < 1
    ? Math.pow(input.base, 1 - input.exponentSpread)
    : Math.pow(input.base, 1 + input.exponentSpread)
}

// ─── Ant speed (with ascension-challenge penalties) ────────────────────────

export interface ActualAntSpeedMultInput {
  /**
   * Product of the antSpeedStats StatLines (web_ui's
   *   statLineDecimalMultiplication(antSpeedStats)
   * ).
   */
  base: DecimalSource
  /**
   * player.currentChallenge.ascension. Picks the exponent penalty:
   *   12 → 0.75   13 → 0.23   14 → 0.20   15 → 0.50   else → 1
   */
  ascensionChallenge: number
  /**
   * player.platonicUpgrades[10]. When > 0 AND ascensionChallenge === 15, the
   * exponent is multiplied by 1.25 — partial mitigation of the C15 penalty.
   */
  platonicUpgrade10: number
}

/**
 * Raises the precomputed Decimal base by an exponent that depends on the
 * current ascension challenge. The penalties (and the platonic-10 mitigation
 * for C15) are preserved verbatim from web_ui.
 */
export function calculateActualAntSpeedMult(input: ActualAntSpeedMultInput): Decimal {
  let exponent = 1
  if (input.ascensionChallenge === 12) exponent = 0.75
  else if (input.ascensionChallenge === 13) exponent = 0.23
  else if (input.ascensionChallenge === 14) exponent = 0.2
  else if (input.ascensionChallenge === 15) exponent = 0.5

  if (input.platonicUpgrade10 > 0 && input.ascensionChallenge === 15) {
    exponent *= 1.25
  }

  return Decimal.pow(input.base, exponent)
}
