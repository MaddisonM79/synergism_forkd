// Corruption math, migrated from packages/web_ui/src/Corruptions.ts. The
// CorruptionLoadout / CorruptionSaves classes and the UI loadout table stay
// in web_ui — they touch DOM, Notification, and the i18next loadout-export
// flow. Logic owns the per-corruption multipliers and the cap formula.

// ─── Cap on per-corruption level ───────────────────────────────────────────

export interface MaxCorruptionLevelInput {
  /** player.challengecompletions[11]. +5 to cap when any completion exists. */
  challenge11Completions: number
  /** player.challengecompletions[12]. +2 to cap when any. */
  challenge12Completions: number
  /** player.challengecompletions[13]. +2 to cap when any. */
  challenge13Completions: number
  /** player.challengecompletions[14]. +2 to cap when any. */
  challenge14Completions: number
  /** player.platonicUpgrades[5]. +1 when any. */
  platonicUpgrade5: number
  /** player.platonicUpgrades[10]. +1 when any. */
  platonicUpgrade10: number
  /**
   * getGQUpgradeEffect('platonicTau', 'unlocked'). Floor of 13 — applied
   * AFTER the challenge/platonic adds, BEFORE corruptionFourteen.
   */
  platonicTauUnlocked: boolean
  /**
   * getGQUpgradeEffect('corruptionFourteen', 'unlocked'). +1 to the final
   * cap (after the platonicTau floor).
   */
  corruptionFourteenUnlocked: boolean
  /**
   * getOcteractUpgradeEffect('octeractCorruption', 'corruptionLevelCapIncrease').
   * Added to the final cap.
   */
  octeractCorruptionCapIncrease: number
}

/**
 * Maximum corruption level players can set on any single corruption. Sum of
 * challenge / platonic / GQ / octeract contributions, with a platonicTau
 * floor of 13 if that upgrade is unlocked.
 */
export function maxCorruptionLevel(input: MaxCorruptionLevelInput): number {
  let max = 0
  if (input.challenge11Completions > 0) max += 5
  if (input.challenge12Completions > 0) max += 2
  if (input.challenge13Completions > 0) max += 2
  if (input.challenge14Completions > 0) max += 2
  if (input.platonicUpgrade5 > 0) max += 1
  if (input.platonicUpgrade10 > 0) max += 1

  if (input.platonicTauUnlocked) {
    max = Math.max(13, max)
  }

  if (input.corruptionFourteenUnlocked) {
    max += 1
  }
  max += input.octeractCorruptionCapIncrease

  return max
}

// ─── Per-corruption effect calculators ─────────────────────────────────────
//
// Each takes the looked-up `basePower` from web_ui's G.<corruption>Power
// table (web_ui owns the data arrays). Returns the multiplicative effect
// applied to the matching system — production for viscosity/illiteracy,
// salvage for drought, challenge requirements for hyperchallenge.

export interface ViscosityEffectInput {
  /** G.viscosityPower[level] — the level-indexed base exponent. */
  basePower: number
  /** player.platonicUpgrades[6]. Multiplies base by (1 + n / 30). */
  platonicUpgrade6: number
}

/**
 * Viscosity production exponent. Clamped to ≤ 1 — buffs can only soften the
 * corruption, never reverse it.
 */
export function viscosityEffect(input: ViscosityEffectInput): number {
  return Math.min(input.basePower * (1 + input.platonicUpgrade6 / 30), 1)
}

export interface DroughtEffectInput {
  /** G.droughtSalvage[level]. */
  baseSalvage: number
  /** player.platonicUpgrades[13]. When > 0, halves the salvage reduction. */
  platonicUpgrade13: number
}

/**
 * Drought salvage reduction multiplier. Platonic 13 halves the reduction.
 */
export function droughtEffect(input: DroughtEffectInput): number {
  return input.platonicUpgrade13 > 0
    ? input.baseSalvage * 0.5
    : input.baseSalvage
}

export interface IlliteracyEffectInput {
  /** G.illiteracyPower[level]. */
  basePower: number
  /** player.platonicUpgrades[9]. */
  platonicUpgrade9: number
  /**
   * `player.obtainium.gte(1)` AND `Decimal.log10(player.obtainium)` — the
   * obtainium-based boost only applies when obtainium ≥ 1. Pass:
   *   - `null` if obtainium < 1 (boost path skipped)
   *   - the log10 value (capped at 100 by the caller or here) otherwise
   * Letting null in keeps the Decimal dependency on the wrapper side.
   */
  obtainiumLog10OrNull: number | null
}

/**
 * Illiteracy production exponent. When obtainium ≥ 1, gets bumped by
 * `1 + (platonic9 / 100) * min(100, log10(obtainium))`. Clamped to ≤ 1.
 */
export function illiteracyEffect(input: IlliteracyEffectInput): number {
  const multiplier = input.obtainiumLog10OrNull === null
    ? 1
    : 1 + (1 / 100) * input.platonicUpgrade9 * Math.min(100, input.obtainiumLog10OrNull)
  return Math.min(input.basePower * multiplier, 1)
}

export interface HyperchallengeEffectInput {
  /** G.hyperchallengeMultiplier[level]. */
  baseEffect: number
  /** player.platonicUpgrades[8]. Divides base by (1 + 2/5 * n). */
  platonicUpgrade8: number
}

/**
 * Hyperchallenge requirement multiplier. Floored at 1 — platonic-8 can soften
 * the corruption but never make challenges easier than baseline.
 */
export function hyperchallengeEffect(input: HyperchallengeEffectInput): number {
  const divisor = 1 + 2 / 5 * input.platonicUpgrade8
  return Math.max(1, input.baseEffect / divisor)
}
