// Singularity-penalty math, lifted from packages/web_ui/src/singularity.ts.
//
// `calculateEffectiveSingularities` is the post-multiplier singularity count
// used as the basis for every per-system debuff. `calculateSingularityDebuff`
// switches on the system tag and produces the actual multiplier (or
// subtractive amount, for Salvage / Ant ELO). The web_ui side stays
// responsible for sourcing shop / ambrosia / antiquities / rune state and
// for the i18n display strings — this module is pure math.

import { calculateExalt4EffectiveSingularityMultiplier } from './exaltPenalties'

export type SingularityDebuff =
  | 'Offering'
  | 'Obtainium'
  | 'Salvage'
  | 'Global Speed'
  | 'Researches'
  | 'Ant ELO'
  | 'Ascension Speed'
  | 'Cubes'
  | 'Cube Upgrades'
  | 'Platonic Costs'
  | 'Hepteract Costs'

export interface CalculateEffectiveSingularitiesInput {
  /** The raw singularity count being evaluated. Callers usually pass the
   *  constitutive count (singularityCount - reductions), not the raw value. */
  singularityCount: number
  /** player.singularityChallenges.noOcteracts.completions — feeds Exalt 4. */
  noOcteractsCompletions: number
  /** player.singularityChallenges.noOcteracts.enabled — gates Exalt 4. */
  inExalt4: boolean
  /** player.singularityChallenges.taxmanLastStand.enabled — gates the cube
   *  root of the final value when the suppress-platonic-15 condition is met. */
  taxmanLastStandEnabled: boolean
  /** player.singularityChallenges.taxmanLastStand.completions — must be ≥ 8
   *  alongside an unowned platonic 15 for the ^(3/2) override to kick in. */
  taxmanLastStandCompletions: number
  /** player.platonicUpgrades[15] — when > 0, suppresses the taxman override. */
  platonicUpgrade15: number
}

/**
 * Effective singularity count after stacking the staircase of milestone
 * multipliers (×1.5 past 10, ×2.5 past 25, etc.). The Exalt 4 multiplier and
 * the taxman-last-stand ^(3/2) override are both applied here.
 */
export function calculateEffectiveSingularities (
  input: CalculateEffectiveSingularitiesInput
): number {
  const singularityCount = input.singularityCount
  let effectiveSingularities = singularityCount
  effectiveSingularities *= Math.min(4.75, (0.75 * singularityCount) / 10 + 1)

  effectiveSingularities *= calculateExalt4EffectiveSingularityMultiplier({
    comps: input.noOcteractsCompletions,
    force: false,
    inExalt4: input.inExalt4
  })

  if (singularityCount > 10) {
    effectiveSingularities *= 1.5
    effectiveSingularities *= Math.min(
      4,
      (1.25 * singularityCount) / 10 - 0.25
    )
  }
  if (singularityCount > 25) {
    effectiveSingularities *= 2.5
    effectiveSingularities *= Math.min(6, (1.5 * singularityCount) / 25 - 0.5)
  }
  if (singularityCount > 36) {
    effectiveSingularities *= 4
    effectiveSingularities *= Math.min(5, singularityCount / 18 - 1)
    effectiveSingularities *= Math.pow(
      1.1,
      Math.min(singularityCount - 36, 64)
    )
  }
  if (singularityCount > 50) {
    effectiveSingularities *= 5
    effectiveSingularities *= Math.min(8, (2 * singularityCount) / 50 - 1)
    effectiveSingularities *= Math.pow(
      1.1,
      Math.min(singularityCount - 50, 50)
    )
  }
  if (singularityCount > 100) {
    effectiveSingularities *= 2
    effectiveSingularities *= singularityCount / 25
    effectiveSingularities *= Math.pow(1.1, singularityCount - 100)
  }
  if (singularityCount > 150) {
    effectiveSingularities *= 2
    effectiveSingularities *= Math.pow(1.05, singularityCount - 150)
  }
  if (singularityCount > 200) {
    effectiveSingularities *= 1.5
    effectiveSingularities *= Math.pow(1.275, singularityCount - 200)
  }
  if (singularityCount > 215) {
    effectiveSingularities *= 1.25
    effectiveSingularities *= Math.pow(1.2, singularityCount - 215)
  }
  if (singularityCount > 230) {
    effectiveSingularities *= 2
  }
  if (singularityCount > 269) {
    effectiveSingularities *= 3
    effectiveSingularities *= Math.pow(3, singularityCount - 269)
  }

  if (
    input.taxmanLastStandEnabled
    && input.taxmanLastStandCompletions >= 8
    && input.platonicUpgrade15 === 0
  ) {
    effectiveSingularities = Math.pow(effectiveSingularities, 3 / 2)
  }

  return effectiveSingularities
}

export interface CalculateSingularityDebuffInput {
  /** Which system the debuff applies to. Determines the formula branch. */
  debuff: SingularityDebuff
  /** The raw singularity count being evaluated (usually player.singularityCount). */
  singularityCount: number
  /** runes.antiquities.level > 0 — when true, all penalties drop to their
   *  no-penalty value (0 for Salvage / Ant ELO, 1 otherwise). */
  antiquitiesRuneActive: boolean
  /**
   * Sum of `shopSingularityPenaltyDebuff.singularityPenaltyReducers` and the
   * appropriate ambrosia singularity-reduction (1 outside sing challenges,
   * 2 inside). Subtracted from singularityCount to form the constitutive
   * count used by every formula branch.
   */
  singularityReductions: number
  /**
   * getShopUpgradeEffects('shopHorseShoe', 'singularityPenaltyMult') — applied
   * to all multiplicative branches as `baseDebuffMultiplier`. Not applied to
   * Salvage / Ant ELO (those are subtractive).
   */
  horseShoeMult: number
  /** Pass-through to calculateEffectiveSingularities. */
  noOcteractsCompletions: number
  /** Pass-through to calculateEffectiveSingularities. */
  inExalt4: boolean
  /** Pass-through to calculateEffectiveSingularities. */
  taxmanLastStandEnabled: boolean
  /** Pass-through to calculateEffectiveSingularities. */
  taxmanLastStandCompletions: number
  /** Pass-through to calculateEffectiveSingularities. */
  platonicUpgrade15: number
}

/**
 * Per-system singularity penalty.
 *
 * - Returns 0 (Salvage / Ant ELO) or 1 (everything else) when the singularity
 *   count is zero OR the antiquities rune is active OR the constitutive count
 *   (singularityCount - reductions) is below 1.
 * - Otherwise switches on `debuff` to pick a formula. Salvage and Ant ELO
 *   return subtractive amounts (negative of the magnitude); the rest return
 *   multiplicative penalties.
 */
export function calculateSingularityDebuff (
  input: CalculateSingularityDebuffInput
): number {
  if (input.singularityCount === 0 || input.antiquitiesRuneActive) {
    return (input.debuff === 'Salvage' || input.debuff === 'Ant ELO') ? 0 : 1
  }

  const constitutiveSingularityCount = input.singularityCount - input.singularityReductions
  if (constitutiveSingularityCount < 1) {
    return 1
  }

  const effectiveSingularities = calculateEffectiveSingularities({
    singularityCount: constitutiveSingularityCount,
    noOcteractsCompletions: input.noOcteractsCompletions,
    inExalt4: input.inExalt4,
    taxmanLastStandEnabled: input.taxmanLastStandEnabled,
    taxmanLastStandCompletions: input.taxmanLastStandCompletions,
    platonicUpgrade15: input.platonicUpgrade15
  })

  const baseDebuffMultiplier = input.horseShoeMult

  if (input.debuff === 'Offering') {
    const extraMult = Math.pow(1.02, constitutiveSingularityCount)
    return extraMult * baseDebuffMultiplier * (constitutiveSingularityCount < 150
      ? 3 * (Math.sqrt(effectiveSingularities) + 1)
      : Math.pow(effectiveSingularities, 2 / 3) / 400)
  } else if (input.debuff === 'Salvage') {
    return -(4 * constitutiveSingularityCount
      + 4 * Math.max(0, constitutiveSingularityCount - 100)
      + 4 * Math.max(0, constitutiveSingularityCount - 200)
      + 3 * Math.max(0, constitutiveSingularityCount - 250)
      + 3 * Math.max(0, constitutiveSingularityCount - 270)
      + 2 * Math.max(0, constitutiveSingularityCount - 280))
  } else if (input.debuff === 'Ant ELO') {
    return -Math.min(1, 0.001 * constitutiveSingularityCount)
  } else if (input.debuff === 'Global Speed') {
    return baseDebuffMultiplier * (1 + Math.sqrt(effectiveSingularities) / 4)
  } else if (input.debuff === 'Obtainium') {
    const extraMult = Math.pow(1.02, constitutiveSingularityCount)
    return extraMult * baseDebuffMultiplier * (constitutiveSingularityCount < 150
      ? 3 * (Math.sqrt(effectiveSingularities) + 1)
      : Math.pow(effectiveSingularities, 2 / 3) / 400)
  } else if (input.debuff === 'Researches') {
    return baseDebuffMultiplier * (1 + Math.sqrt(effectiveSingularities) / 2)
  } else if (input.debuff === 'Ascension Speed') {
    return baseDebuffMultiplier * (constitutiveSingularityCount < 150
      ? 1 + Math.sqrt(effectiveSingularities) / 5
      : 1 + Math.pow(effectiveSingularities, 0.75) / 10000)
  } else if (input.debuff === 'Cubes') {
    const extraMult = constitutiveSingularityCount > 100
      ? 2 * Math.pow(1.03, constitutiveSingularityCount - 100)
      : 2
    return baseDebuffMultiplier * (constitutiveSingularityCount < 150
      ? 3 * (1 + (Math.sqrt(effectiveSingularities) * extraMult) / 4)
      : 1 + (Math.pow(effectiveSingularities, 0.75) * extraMult) / 1000)
  } else if (input.debuff === 'Platonic Costs') {
    return baseDebuffMultiplier * (constitutiveSingularityCount > 36
      ? 1 + Math.pow(effectiveSingularities, 3 / 10) / 12
      : 1)
  } else if (input.debuff === 'Hepteract Costs') {
    return baseDebuffMultiplier * (constitutiveSingularityCount > 50
      ? 1 + Math.pow(effectiveSingularities, 11 / 50) / 25
      : 1)
  } else {
    return baseDebuffMultiplier * Math.cbrt(effectiveSingularities + 1)
  }
}
