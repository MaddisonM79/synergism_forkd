// Total-octeract bonuses lifted from packages/web_ui/src/Calculate.ts. Four
// thematically-related functions gated by the noOcteracts (Exalt 4)
// singularity challenge. All bodies are transcribed verbatim — only the
// player / getSingularityChallengeEffect lookups are hoisted out into input
// fields that the web_ui shim precomputes.
//
// Offering & Obtainium bonuses are derived from the cube bonus, so the
// caller precomputes that and passes it in (matches the rest of the package's
// "callers aggregate, logic transforms" convention).

// ─── Total octeract cube bonus ─────────────────────────────────────────────

export interface CalculateTotalOcteractCubeBonusInput {
  /** player.singularityChallenges.noOcteracts.enabled — Exalt 4 gate. */
  exalt4Enabled: boolean
  /** player.totalWowOcteracts — lifetime octeract earnings. */
  totalWowOcteracts: number
  /**
   * getSingularityChallengeEffect('noOcteracts', 'octeractPow') — additive
   * exponent boost on the log10 branch. Base power is 2 + octeractPow.
   */
  octeractPow: number
}

/**
 * Linear ramp from 1 to 3 across 0–1000 octeracts (with a small-value
 * threshold to avoid noise just above 1), then a log10 power curve above
 * 1000: `3 * (log10(N) - 2) ^ (2 + octeractPow)`. Returns 1 when inside
 * Exalt 4 (bonus is suppressed by the challenge).
 */
export function calculateTotalOcteractCubeBonus(input: CalculateTotalOcteractCubeBonusInput): number {
  if (input.exalt4Enabled) {
    return 1
  }
  if (input.totalWowOcteracts < 1000) {
    const bonus = 1 + (2 / 1000) * input.totalWowOcteracts
    return bonus > 1.00001 ? bonus : 1
  }
  const power = 2 + input.octeractPow
  return 3 * Math.pow(Math.log10(input.totalWowOcteracts) - 2, power)
}

// ─── Total octeract quark bonus ────────────────────────────────────────────

export interface CalculateTotalOcteractQuarkBonusInput {
  /** player.singularityChallenges.noOcteracts.enabled — Exalt 4 gate. */
  exalt4Enabled: boolean
  /** player.totalWowOcteracts — lifetime octeract earnings. */
  totalWowOcteracts: number
}

/**
 * Linear ramp from 1 to 1.20 across 0–1000 octeracts (with the same
 * small-value tolerance as the cube bonus), then linear in `log10(N) - 2`
 * above 1000: `1.1 + 0.1 * (log10(N) - 2)`. Returns 1 in Exalt 4.
 */
export function calculateTotalOcteractQuarkBonus(input: CalculateTotalOcteractQuarkBonusInput): number {
  if (input.exalt4Enabled) {
    return 1
  }
  if (input.totalWowOcteracts < 1000) {
    const bonus = 1 + (0.2 / 1000) * input.totalWowOcteracts
    return bonus > 1.00001 ? bonus : 1
  }
  return 1.1 + 0.1 * (Math.log10(input.totalWowOcteracts) - 2)
}

// ─── Total octeract offering bonus ─────────────────────────────────────────

export interface CalculateTotalOcteractOfferingBonusInput {
  /**
   * Truthy when `getSingularityChallengeEffect('noOcteracts', 'offeringBonus')`
   * is unlocked (Exalt 4 has been completed enough to grant the offering
   * bonus). Falsy → returns 1.
   */
  offeringBonusEnabled: boolean
  /**
   * Precomputed cube bonus (caller invokes `calculateTotalOcteractCubeBonus`
   * first). Raised to the 1.25 power.
   */
  cubeBonus: number
}

/**
 * `cubeBonus ^ 1.25` once the offering reward of Exalt 4 has been unlocked;
 * otherwise 1.
 */
export function calculateTotalOcteractOfferingBonus(
  input: CalculateTotalOcteractOfferingBonusInput
): number {
  if (!input.offeringBonusEnabled) {
    return 1
  }
  return Math.pow(input.cubeBonus, 1.25)
}

// ─── Total octeract obtainium bonus ────────────────────────────────────────

export interface CalculateTotalOcteractObtainiumBonusInput {
  /**
   * Truthy when `getSingularityChallengeEffect('noOcteracts', 'obtainiumBonus')`
   * is unlocked. Falsy → returns 1.
   */
  obtainiumBonusEnabled: boolean
  /** Precomputed cube bonus, raised to the 1.25 power. */
  cubeBonus: number
}

/**
 * `cubeBonus ^ 1.25` once the obtainium reward of Exalt 4 has been unlocked;
 * otherwise 1. Same formula as the offering bonus, gated by a different
 * challenge effect.
 */
export function calculateTotalOcteractObtainiumBonus(
  input: CalculateTotalOcteractObtainiumBonusInput
): number {
  if (!input.obtainiumBonusEnabled) {
    return 1
  }
  return Math.pow(input.cubeBonus, 1.25)
}
