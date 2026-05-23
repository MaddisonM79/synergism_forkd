// Tax exponent and divisor formula, lifted from the second half of
// packages/web_ui/src/Tax.ts. Pure given the full bag of player + effect
// inputs; the web_ui side does the input gathering and writes the result
// fields back to G.
//
// The function returns the full output bundle plus a `shouldAwardOvertaxed`
// flag — that one's a side-effect (calls awardUngroupedAchievement in
// web_ui) which we surface as a boolean so logic stays free of UI calls.
//
// Output formulas:
//   maxexponent      = floor(275 / (log10(1.01) * exponent)) - 1 + flatMaxIncrease
//   taxdivisor       = 1.01 ^ (divisorExponent * exponent)
//   taxdivisorcheck  = 1.01 ^ (checkExponent * exponent)
// where divisorExponent / checkExponent are quadratics in their own
// exponent bases (which incorporate the produceTotal log and the flat
// max-exponent increase from ant upgrades).

import { Decimal } from '../math/bignum'
import { CalcECC } from './challenges'

export interface CalculateTaxInput {
  // ─── Challenge / completion state ─────────────────────────────────────

  /** player.currentChallenge.reincarnation === 6 — base exp = 3*(1+c6/25)^2. */
  inReinc6: boolean
  /** player.currentChallenge.reincarnation === 9 — base exp = 0.005. */
  inReinc9: boolean
  /** player.currentChallenge.ascension === 15 — base exp = 0.000005. */
  inAscension15: boolean
  /** player.currentChallenge.ascension === 13 — apply C13 exp multiplier. */
  inAscension13: boolean
  /** player.challengecompletions[6] — feeds reinc6 base + the 1.075 divisor. */
  c6Completions: number
  /** player.challengecompletions[13] — feeds the C13 exp multiplier. */
  c13Completions: number

  // ─── c13effcompletions inputs ─────────────────────────────────────────

  /**
   * `sumContents(player.challengecompletions)` — total completion count.
   * c13effcompletions subtracts the high-tier challenge contributions and
   * the singularity-15/20 bonuses.
   */
  totalChallengeCompletions: number
  c11Completions: number
  c12Completions: number
  c14Completions: number
  c15Completions: number
  /** player.singularityCount — feeds the -4 / -1 cuts at 15 / 20. */
  singularityCount: number

  // ─── Research / cube / platonic reductions ────────────────────────────

  /** player.researches[51]. exponent *= 1 - 0.06 * n. */
  research51: number
  /** player.researches[52..55]. Each: exponent *= 1 - 0.05 * n. */
  research52: number
  research53: number
  research54: number
  research55: number
  /** player.researches[159]. Feeds 0.98^(3/5 * log10(rareFragments+1) * n). */
  research159: number
  /** player.researches[200]. exponent *= 1 - 0.666 * n / 100000. */
  research200: number
  /** player.cubeUpgrades[50]. exponent *= 1 - 0.666 * n / 100000. */
  cubeUpgrade50: number
  /** player.platonicUpgrades[5]. Adds 0.1 * n to the ascendShards exponent. */
  platonicUpgrade5: number
  /** player.platonicUpgrades[10]. Adds 0.2 * n to the ascendShards exponent. */
  platonicUpgrade10: number
  /** calculateTaxPlatonicBlessing(). Added to the ascendShards exponent. */
  taxPlatonicBlessing: number
  /** player.upgrades[121]. When > 0, halves the exponent. */
  upgrade121: number
  /** player.upgrades[125]. Feeds the ascendShards exponent with c10 scaling. */
  upgrade125: number
  /** player.challengecompletions[10]. Scales upgrade[125]'s contribution. */
  c10Completions: number

  // ─── Singularity / late-game ──────────────────────────────────────────

  /** player.highestSingularityCount. When ≥ 281, halves the exponent. */
  highestSingularityCount: number
  /** player.singularityChallenges.taxmanLastStand.enabled. */
  taxmanLastStandEnabled: boolean
  /** player.unlocks.ascensions — adds ×4 inside taxmanLastStand. */
  ascensionsUnlocked: boolean
  /** player.highestchallengecompletions[14] — adds ×5 inside taxmanLastStand when > 0. */
  highestC14Completions: number

  // ─── Pre-evaluated effect values (sourced by web_ui) ──────────────────

  /** +getAchievementReward('taxReduction') — the `+` coerces a boolean to 0/1. */
  taxReductionAchievement: number
  /** getRuneEffects('duplication', 'taxReduction'). */
  duplicationRuneTaxReduction: number
  /** getRuneEffects('thrift', 'taxReduction'). */
  thriftRuneTaxReduction: number
  /** getAntUpgradeEffect(AntUpgrades.Taxes).taxReduction. */
  antTaxReduction: number
  /** getTalismanEffects('exemption').taxReduction. */
  exemptionTalismanTaxReduction: number
  /** G.challenge15Rewards.taxes.value. */
  challenge15TaxesReward: number
  /** player.campaigns.taxMultiplier. */
  campaignTaxMultiplier: number

  // ─── Decimal inputs (for log10) ───────────────────────────────────────

  /** player.ascendShards — log10 feeds the divisor in the exponent chain. */
  ascendShards: Decimal
  /** player.rareFragments — log10 feeds research[159]'s 0.98^... term. */
  rareFragments: Decimal
  /**
   * getAntUpgradeEffect(AntUpgrades.Coins).coinMultiplier — log10 of this
   * is added to flatMaxExponentIncrease (Fortunae Formicidae is tax-exempt).
   */
  fortunaeFormicidaeCoinMultiplier: Decimal
  /**
   * calculateBuildingPowerCoinMultiplier() — also log10'd into the
   * flatMaxExponentIncrease.
   */
  buildingPowerCoinMultiplier: Decimal

  /**
   * G.produceTotal — sum of pre-clamp tier outputs. Its log10 feeds the
   * exponentForDivisor (clamped to [0, maxexponent]).
   */
  produceTotal: Decimal
}

export interface CalculateTaxResult {
  /** The final tax exponent — floored at 1e-300 to dodge an overflow bug. */
  exponent: number
  /** Max exponent the player can reach — floored value with the flat increase. */
  maxexponent: number
  /** The taxdivisor that scales coin production downward at high counts. */
  taxdivisor: Decimal
  /** Sibling check value — used by web_ui to detect "you're about to hit the cap". */
  taxdivisorcheck: Decimal
  /**
   * True when the overtaxed achievement should be awarded — i.e. the player
   * is in C13, has at least 1 c13eff completion, and their max-exponent gap
   * is at most 99999. Web_ui calls awardUngroupedAchievement on this flag.
   */
  shouldAwardOvertaxed: boolean
}

// Hardcoded numerator of the maxexponent formula. Comes from the
// underlying log-base-1.01 conversion of the 275-coin "ten billion to the
// max-exponent power" target.
const MAX_EXPONENT_NUMERATOR = 275
// 1.01 is the base of every tax-divisor power. Pulled out so the two
// log10(1.01) calls in maxexponent / taxdivisor share the same constant.
const TAX_BASE = 1.01

// Computes c13effcompletions — the count of challenge completions that
// "really matter" inside C13. Filters out the high-tier (11-15) challenge
// completions and the sing-15/20 bonuses (which xander apparently exploited).
function computeC13EffCompletions (input: CalculateTaxInput): number {
  return Math.max(
    0,
    input.totalChallengeCompletions
      - input.c11Completions
      - input.c12Completions
      - input.c13Completions
      - input.c14Completions
      - input.c15Completions
      - ((input.singularityCount >= 15) ? 4 : 0)
      - ((input.singularityCount >= 20) ? 1 : 0)
  )
}

// Picks the base `exp` value, applying challenge-specific overrides in
// the same precedence order as the legacy code: reinc6 → reinc9 → asc15.
// (Each later check overwrites the prior; if none fire, base is 1.)
function computeBaseExp (input: CalculateTaxInput): number {
  let exp = 1
  if (input.inReinc6) {
    exp = 3 * Math.pow(1 + input.c6Completions / 25, 2)
  }
  if (input.inReinc9) {
    exp = 0.005
  }
  if (input.inAscension15) {
    exp = 0.000005
  }
  return exp
}

/**
 * Computes the tax exponent, max exponent, and the two taxdivisor values.
 *
 * Branching summary:
 *   - Base exp picks one of {1, reinc6 formula, 0.005, 0.000005}
 *   - C13 multiplies by 400 * (1 + c13/6) * 1.05^c13effcompletions
 *   - C6 completions divide by 1.075
 *   - Research / talisman / rune / cube reductions stack multiplicatively
 *   - ascendShards log10 raised to (1 + c10/300*upgrade125 + 0.1*plat5 +
 *     0.2*plat10 + taxPlatonicBlessing) divides
 *   - rareFragments + research159 add a 0.98^... factor
 *   - 281+ singularity + upgrade121 each halve
 *   - taxmanLastStand stacks ×4 (asc unlocked) and ×5 (c14 done)
 *   - Final clamp at 1e-300
 *
 * Then maxexponent = floor(275 / (log10(1.01) * exponent)) - 1 + flatIncrease,
 * where flatIncrease = log10(fortunaeFormicidae) + log10(buildingPower).
 *
 * exponentForDivisor clamps log10(produceTotal+1) to [0, maxexponent] then
 * subtracts flatIncrease. exponentForWarning is just maxexponent - flatIncrease.
 * Both go through (1/550 * x^2) before becoming the taxdivisor exponent.
 */
export function calculateTax (input: CalculateTaxInput): CalculateTaxResult {
  const c13eff = computeC13EffCompletions(input)

  let exp = computeBaseExp(input)

  if (input.inAscension13) {
    exp *= 400 * (1 + 1 / 6 * input.c13Completions)
    exp *= Math.pow(1.05, c13eff)
  }
  if (input.c6Completions > 0) {
    exp /= 1.075
  }

  let exponent = 1
  exponent *= exp
  exponent *= 1 - 0.06 * input.research51
  exponent *= 1 - 0.05 * input.research52
  exponent *= 1 - 0.05 * input.research53
  exponent *= 1 - 0.05 * input.research54
  exponent *= 1 - 0.05 * input.research55
  exponent *= input.taxReductionAchievement
  exponent *= Math.pow(0.965, CalcECC('reincarnation', input.c6Completions))
  exponent *= input.duplicationRuneTaxReduction
  exponent *= input.thriftRuneTaxReduction
  exponent *= input.antTaxReduction
  // ascendShards log10 raised to a sum of platonic/upgrade contributions
  exponent *= 1
    / Math.pow(
      1 + Decimal.log(input.ascendShards.add(1), 10),
      1 + 1 / 300 * input.c10Completions * input.upgrade125 + 0.1 * input.platonicUpgrade5
        + 0.2 * input.platonicUpgrade10 + input.taxPlatonicBlessing
    )
  exponent *= 1 + input.exemptionTalismanTaxReduction
  exponent *= Math.pow(0.98, 3 / 5 * Decimal.log(input.rareFragments.add(1), 10) * input.research159)
  exponent *= Math.pow(0.966, CalcECC('ascension', input.c13Completions))
  exponent *= 1 - 0.666 * input.research200 / 100000
  exponent *= 1 - 0.666 * input.cubeUpgrade50 / 100000
  exponent *= input.challenge15TaxesReward
  exponent *= input.campaignTaxMultiplier
  if (input.upgrade121 > 0) {
    exponent *= 0.5
  }
  if (input.highestSingularityCount >= 281) {
    exponent *= 0.5
  }
  if (input.taxmanLastStandEnabled) {
    if (input.ascensionsUnlocked) {
      exponent *= 4
    }
    if (input.highestC14Completions > 0) {
      exponent *= 5
    }
  }

  // Overflow guard — exponent of zero would NaN every downstream pow.
  if (exponent < 1e-300) {
    exponent = 1e-300
  }

  const flatMaxExponentIncrease = Decimal.log(input.fortunaeFormicidaeCoinMultiplier, 10)
    + Decimal.log(input.buildingPowerCoinMultiplier, 10)

  const maxexponent = Math.floor(MAX_EXPONENT_NUMERATOR / (Decimal.log(TAX_BASE, 10) * exponent)) - 1
    + flatMaxExponentIncrease

  const exponentForDivisor = Math.max(
    0,
    Math.min(maxexponent, Math.floor(Decimal.log(input.produceTotal.add(1), 10))) - flatMaxExponentIncrease
  )
  const exponentForWarning = Math.max(0, maxexponent - flatMaxExponentIncrease)

  const divisorExponent = 1 / 550 * Math.pow(exponentForDivisor, 2)
  const checkExponent = 1 / 550 * Math.pow(exponentForWarning, 2)

  const taxdivisor = Decimal.pow(TAX_BASE, divisorExponent * exponent)
  const taxdivisorcheck = Decimal.pow(TAX_BASE, checkExponent * exponent)

  // Overtaxed achievement: C13 active + at least one effective completion
  // + max-exponent gap below 100000.
  const shouldAwardOvertaxed = input.inAscension13
    && (maxexponent - flatMaxExponentIncrease) <= 99999
    && c13eff >= 1

  return { exponent, maxexponent, taxdivisor, taxdivisorcheck, shouldAwardOvertaxed }
}
