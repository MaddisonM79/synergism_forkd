// Ant-sacrifice reward calculators (offering + obtainium + immortal-ELO
// gain + taxman-last-stand clamp). Lifted from:
//   packages/web_ui/src/Features/Ants/AntSacrifice/Rewards/
//     Offerings/calculate-offerings.ts
//     Obtainium/calculate-obtainium.ts
//     ELO/ImmortalELO/lib/calculate.ts
//
// The Statistics-aggregator reductions (offeringObtainiumTimeModifiers,
// antSacrificeRewardStats, ELO stat arrays) stay in web_ui because they
// reduce display-metadata closures. Logic owns the per-call arithmetic
// once the caller has those reductions in hand.

import { Decimal } from '../math/bignum'

// ─── Immortal-ELO gain ────────────────────────────────────────────────────

export interface CalculateImmortalELOGainInput {
  /** Result of calculateEffectiveAntELO (Statistics-coupled in web_ui). */
  effectiveELO: number
  /** player.ants.immortalELO. */
  immortalELO: number
}

/** Floor-clamped delta `max(0, effectiveELO − immortalELO)`. */
export function calculateImmortalELOGain (input: CalculateImmortalELOGainInput): number {
  return Math.max(0, input.effectiveELO - input.immortalELO)
}

// ─── Taxman-last-stand clamp ──────────────────────────────────────────────

export interface ApplyTaxmanLastStandClampInput {
  /** Pre-clamp final reward. */
  finalReward: Decimal
  /** Currently-held resource amount (offerings / obtainium etc.). */
  currentResource: Decimal
  /** Whether the taxmanLastStand challenge is enabled. */
  taxmanLastStandEnabled: boolean
  /** completions count for taxmanLastStand; the clamp engages at >=2. */
  taxmanLastStandCompletions: number
}

/**
 * Caps `finalReward` at `currentResource * 100 + 1` when the taxman-last-
 * stand challenge is at 2+ completions. Otherwise passes through unchanged.
 */
export function applyTaxmanLastStandClamp (input: ApplyTaxmanLastStandClampInput): Decimal {
  if (input.taxmanLastStandEnabled && input.taxmanLastStandCompletions >= 2) {
    return Decimal.min(input.currentResource.times(100).plus(1), input.finalReward)
  }
  return input.finalReward
}

// ─── Ant-sacrifice offering / obtainium ───────────────────────────────────

export interface AntSacrificeOfferingInput {
  /** calculateAntSacrificeMultiplier(). */
  antSacMult: Decimal
  /** Reborn-ELO stage modifier antSacrificeOfferingMult. */
  stageMult: number
  /** Reduce-of offeringObtainiumTimeModifiers (a × b.stat() product). */
  timeMultiplier: number
  /** calculateOfferings(false) — without the time-mult double-application. */
  offeringMult: Decimal
  /** player.offerings — current balance, for the taxman clamp. */
  currentOfferings: Decimal
  /** player.singularityChallenges.taxmanLastStand.enabled. */
  taxmanLastStandEnabled: boolean
  /** player.singularityChallenges.taxmanLastStand.completions. */
  taxmanLastStandCompletions: number
}

/**
 * Per-sacrifice offering reward:
 *   offeringMult × (1 × antSacMult × stageMult × timeMultiplier)
 * then clamp by `currentOfferings × 100 + 1` when taxman-last-stand ≥ 2.
 */
export function calculateAntSacrificeOffering (input: AntSacrificeOfferingInput): Decimal {
  const overallSacrificeMultiplier = Decimal.fromString('1')
    .times(input.antSacMult)
    .times(input.stageMult)
    .times(input.timeMultiplier)
  const finalOfferings = input.offeringMult.times(overallSacrificeMultiplier)
  return applyTaxmanLastStandClamp({
    finalReward: finalOfferings,
    currentResource: input.currentOfferings,
    taxmanLastStandEnabled: input.taxmanLastStandEnabled,
    taxmanLastStandCompletions: input.taxmanLastStandCompletions
  })
}

export interface AntSacrificeObtainiumInput {
  /** calculateAntSacrificeMultiplier(). */
  antSacMult: Decimal
  /** Reborn-ELO stage modifier antSacrificeObtainiumMult. */
  stageMult: number
  /** Reduce-of offeringObtainiumTimeModifiers (a × b.stat() product). */
  timeMultiplier: number
  /** calculateObtainium(false) — without the time-mult double-application. */
  obtainiumMult: Decimal
  /** player.obtainium — current balance, for the taxman clamp. */
  currentObtainium: Decimal
  /** player.singularityChallenges.taxmanLastStand.enabled. */
  taxmanLastStandEnabled: boolean
  /** player.singularityChallenges.taxmanLastStand.completions. */
  taxmanLastStandCompletions: number
}

/** Mirrors calculateAntSacrificeOffering for obtainium. */
export function calculateAntSacrificeObtainium (input: AntSacrificeObtainiumInput): Decimal {
  const overallSacrificeMultiplier = Decimal.fromString('1')
    .times(input.antSacMult)
    .times(input.stageMult)
    .times(input.timeMultiplier)
  const finalObtainium = input.obtainiumMult.times(overallSacrificeMultiplier)
  return applyTaxmanLastStandClamp({
    finalReward: finalObtainium,
    currentResource: input.currentObtainium,
    taxmanLastStandEnabled: input.taxmanLastStandEnabled,
    taxmanLastStandCompletions: input.taxmanLastStandCompletions
  })
}
