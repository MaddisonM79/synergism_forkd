import { Decimal } from '../math/bignum'
import type { DecimalSource } from '../math/bignum'
import { CalcECC } from './challenges'

// Pure subroutines from packages/web_ui/src/Calculate.ts (and the
// getReductionValue helper from Buy.ts, which conceptually belongs with the
// cost-divisor aggregators). Each takes its inputs as precomputed numbers —
// the surrounding StatLine reductions stay in web_ui (those are essentially
// aggregators over per-line `stat()` callbacks, which still read from
// player/G state).

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

// ─── Reduction value (cost divisor `r`) ────────────────────────────────────

export interface ReductionValueInput {
  /** getRuneEffects('thrift', 'costDelay') — thrift rune cost-delay effect. */
  thriftCostDelay: number
  /** Sum of player.researches[56..60]. Divided by 200. */
  researchesSum: number
  /** player.challengecompletions[4]. Feeds CalcECC('transcend', cc4) / 200. */
  challengeCompletions4: number
  /** getAntUpgradeEffect(AntUpgrades.BuildingCostScale).buildingCostScale. */
  antBuildingCostScale: number
}

/**
 * Aggregator from packages/web_ui/src/Buy.ts. The value scales the
 * cost-step thresholds in producer / particle / boost cost formulas; the
 * already-migrated buy mechanics receive it as their `r` / `costDivisor`
 * input. Logic owns the formula now; callers (the Buy.ts shim) precompute
 * each contribution and pass it in.
 */
export function getReductionValue(input: ReductionValueInput): number {
  return 1
    + input.thriftCostDelay
    + input.researchesSum / 200
    + CalcECC('transcend', input.challengeCompletions4) / 200
    + input.antBuildingCostScale
}

// ─── Offerings aggregator ──────────────────────────────────────────────────

export interface CalculateOfferingsInput {
  /** Sum from allBaseOfferingStats. */
  baseOfferings: number
  /** Product from offeringObtainiumTimeModifiers when timeMultUsed, else 1. */
  timeMultiplier: number
  /** Product from allOfferingStats (Decimal — can exceed 1e300). */
  offeringMult: Decimal
  /** player.singularityChallenges.taxmanLastStand.enabled. */
  taxmanLastStandEnabled: boolean
  /** player.singularityChallenges.taxmanLastStand.completions (>= 2 triggers the cap). */
  taxmanLastStandCompletions: number
  /** player.offerings — used by the taxman cap. */
  currentOfferings: Decimal
}

/**
 * Final offerings value for the next reset. Combines a base floor with a
 * Decimal "max possible" product, and applies the Exalt 8 (taxmanLastStand)
 * cap when that singularity challenge has been completed at least twice:
 *
 *   if taxmanLastStand2+: min(offerings*100 + 1, max(base, mult * time))
 *   else                : max(base, mult * time)
 */
export function calculateOfferings(input: CalculateOfferingsInput): Decimal {
  const main = Decimal.max(input.baseOfferings, input.offeringMult.times(input.timeMultiplier))
  if (input.taxmanLastStandEnabled && input.taxmanLastStandCompletions >= 2) {
    return Decimal.min(input.currentOfferings.times(100).plus(1), main)
  }
  return main
}

// ─── Obtainium aggregator ──────────────────────────────────────────────────

export interface CalculateObtainiumInput {
  /** Sum from allBaseObtainiumStats. */
  baseObtainium: number
  /** Product from allObtainiumIgnoreDRStats — the "immaculate" leg ignores DR. */
  immaculate: number
  /**
   * Corruption "illiteracy" effect — applied as an exponent on baseMults.
   * (web_ui: player.corruptions.used.corruptionEffects('illiteracy'))
   */
  DR: number
  /** Product from offeringObtainiumTimeModifiers when timeMultUsed, else 1. */
  timeMultiplier: number
  /** Product from allObtainiumStats — large Decimal. */
  baseMults: Decimal
  /** player.currentChallenge.ascension === 14 — short-circuits to 0. */
  inAscensionChallenge14: boolean
  /** player.singularityChallenges.taxmanLastStand.enabled. */
  taxmanLastStandEnabled: boolean
  /** player.singularityChallenges.taxmanLastStand.completions. */
  taxmanLastStandCompletions: number
  /** player.obtainium — used by the taxman cap. */
  currentObtainium: Decimal
}

/**
 * Final obtainium value for the next reincarnation. Like calculateOfferings,
 * with the additional twist that ascension challenge 14 zeroes everything
 * out and the headline multiplier is `immaculate * baseMults^DR * time`
 * (the DR is the illiteracy-corruption damping exponent).
 */
export function calculateObtainium(input: CalculateObtainiumInput): Decimal {
  if (input.inAscensionChallenge14) return new Decimal(0)
  const total = new Decimal(input.immaculate)
    .times(Decimal.pow(input.baseMults, input.DR))
    .times(input.timeMultiplier)
  const main = Decimal.max(input.baseObtainium, total)
  if (input.taxmanLastStandEnabled && input.taxmanLastStandCompletions >= 2) {
    return Decimal.min(input.currentObtainium.times(100).plus(1), main)
  }
  return main
}

// ─── Positive salvage aggregator ───────────────────────────────────────────

export interface CalculatePositiveSalvageInput {
  /** Sum from positiveSalvageStats. */
  rawPositiveSalvage: number
  /** Product from calculatePositiveSalvageMultiplier (a small number-only multiplier). */
  positiveSalvageMultiplier: number
  /** player.singularityChallenges.taxmanLastStand.enabled. */
  taxmanLastStandEnabled: boolean
}

/**
 * Total positive salvage. Two branches:
 *
 *   taxman enabled → 100 + raw * mult / max(1, log(raw))   (log-damped, base 100 floor)
 *   otherwise      → raw * mult
 */
export function calculatePositiveSalvage(input: CalculatePositiveSalvageInput): number {
  if (input.taxmanLastStandEnabled) {
    const baseSalvage = 100
    return baseSalvage
      + (input.rawPositiveSalvage * input.positiveSalvageMultiplier)
        / Math.max(1, Math.log(input.rawPositiveSalvage))
  }
  return input.rawPositiveSalvage * input.positiveSalvageMultiplier
}
