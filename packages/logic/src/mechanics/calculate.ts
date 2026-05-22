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

// ─── Salvage support: multipliers, raw sums, total ─────────────────────────

export interface CalculatePositiveSalvageMultiplierInput {
  /** posSalvagePerkSings.filter(x => x <= player.highestSingularityCount).length — count of unlocked perk thresholds. */
  positiveSalvagePerkUnlockedCount: number
  /** getTalismanEffects('achievement').positiveSalvageMult. */
  talismanAchievementPositiveSalvageMult: number
}

export function calculatePositiveSalvageMultiplier(input: CalculatePositiveSalvageMultiplierInput): number {
  return 1
    + input.positiveSalvagePerkUnlockedCount / 100
    + input.talismanAchievementPositiveSalvageMult
}

export interface CalculateNegativeSalvageMultiplierInput {
  /** negSalvagePerkSings.filter(x => x <= player.highestSingularityCount).length. */
  negativeSalvagePerkUnlockedCount: number
  /** getTalismanEffects('achievement').negativeSalvageMult. */
  talismanAchievementNegativeSalvageMult: number
}

export function calculateNegativeSalvageMultiplier(input: CalculateNegativeSalvageMultiplierInput): number {
  return 1
    - input.negativeSalvagePerkUnlockedCount / 100
    + input.talismanAchievementNegativeSalvageMult
}

export interface CalculateNegativeSalvageInput {
  rawNegativeSalvage: number
  negativeSalvageMultiplier: number
}
export function calculateNegativeSalvage(input: CalculateNegativeSalvageInput): number {
  return input.rawNegativeSalvage * input.negativeSalvageMultiplier
}

export interface CalculateTotalSalvageInput {
  positiveSalvage: number
  negativeSalvage: number
}
export function calculateTotalSalvage(input: CalculateTotalSalvageInput): number {
  return input.positiveSalvage + input.negativeSalvage
}

/**
 * Each point of total salvage shifts the rune-EXP multiplier by ~7.6%
 * (10^(1/30) ≈ 1.079). 30 points = 10x.
 */
export function calculateSalvageRuneEXPMultiplier(salvage: number): Decimal {
  return Decimal.pow(10, salvage / 30)
}

// ─── Ambrosia ──────────────────────────────────────────────────────────────

export interface CalculateAmbrosiaLuckInput {
  rawLuck: number
  multiplier: number
}
export function calculateAmbrosiaLuck(input: CalculateAmbrosiaLuckInput): number {
  return input.rawLuck * input.multiplier
}

export interface CalculateAmbrosiaGenerationSpeedInput {
  rawSpeed: number
  blueberries: number
}
export function calculateAmbrosiaGenerationSpeed(input: CalculateAmbrosiaGenerationSpeedInput): number {
  return input.rawSpeed * input.blueberries
}

// ─── Cube multiplier with tau exponent ─────────────────────────────────────

export interface CalculateCubeMultiplierWithTauInput {
  /** Base cube multiplier (web_ui: calculateCubeMultiplier()). */
  base: number
  /** getGQUpgradeEffect('platonicTau', 'tauPower'). */
  tauPower: number
}
export function calculateCubeMultiplierWithTau(input: CalculateCubeMultiplierWithTauInput): number {
  return Math.pow(input.base, input.tauPower)
}

// ─── Platonic-7 DR power (used inside calculateGlobalSpeedMult below 1) ────

export function calculatePlatonic7UpgradePower(platonicUpgrade7: number): number {
  return 1 - platonicUpgrade7 / 30
}

// ─── Ascension speed exponent spread ───────────────────────────────────────

export interface CalculateAscensionSpeedExponentSpreadInput {
  /** Sum of three GQ/shop upgrade contributions. */
  singAscensionSpeedExponentSpread: number
  singAscensionSpeed2ExponentSpread: number
  chronometerInfinityExponentSpread: number
}
export function calculateAscensionSpeedExponentSpread(input: CalculateAscensionSpeedExponentSpreadInput): number {
  return input.singAscensionSpeedExponentSpread
    + input.singAscensionSpeed2ExponentSpread
    + input.chronometerInfinityExponentSpread
}

// ─── StatLine reducers (named for documentation, trivial inside) ──────────
//
// The Statistics.ts StatLine arrays each compute their own per-line values
// in web_ui. The aggregator math — product or sum — moves here. Each named
// function makes its semantic explicit; the implementation is `reduce`.

export const calculateAllCubeMultiplier = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculateCubeMultiplier = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculateTesseractMultiplier = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculateHypercubeMultiplier = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculatePlatonicMultiplier = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculateHepteractMultiplier = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculateOcteractMultiplier = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)

export const calculateBaseOfferings = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a + b, 0)
export const calculateBaseObtainium = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a + b, 0)

export const calculateOfferingsDecimal = (stats: readonly DecimalSource[]): Decimal =>
  stats.reduce<Decimal>((acc, v) => acc.times(v), new Decimal(1))

/**
 * calculateObtainiumDecimal in web_ui multiplies the per-stat product by the
 * already-precomputed obtainium cube blessing. Keep the same shape here:
 * caller supplies the cubeBlessing value alongside the stat list.
 */
export interface CalculateObtainiumDecimalInput {
  stats: readonly DecimalSource[]
  obtainiumCubeBlessing: DecimalSource
}
export const calculateObtainiumDecimal = (input: CalculateObtainiumDecimalInput): Decimal =>
  input.stats.reduce<Decimal>((acc, v) => acc.times(v), new Decimal(1)).times(input.obtainiumCubeBlessing)

export const calculateObtainiumDRIgnoreMult = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)

export const calculateQuarkMultiplier = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)

/** Like calculateObtainiumDecimal: trailing factor is the antSacrifice cube blessing (Decimal). */
export interface CalculateAntSacrificeMultiplierInput {
  stats: readonly DecimalSource[]
  antSacrificeCubeBlessing: DecimalSource
}
export const calculateAntSacrificeMultiplier = (input: CalculateAntSacrificeMultiplierInput): Decimal =>
  input.stats.reduce<Decimal>((acc, v) => acc.times(v), new Decimal(1))
    .times(input.antSacrificeCubeBlessing)

export const calculateGlobalSpeedDRIgnoreMult = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculateGlobalSpeedDREnabledMult = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculateRawAscensionSpeedMult = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)

export const calculateAmbrosiaAdditiveLuckMult = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a + b, 0)
export const calculateAmbrosiaLuckRaw = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a + b, 0)
export const calculateBlueberryInventory = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a + b, 0)
export const calculateAmbrosiaGenerationSpeedRaw = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)

export const calculatePowderConversion = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculateGoldenQuarks = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculateGoldenQuarkCost = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculateLuckConversion = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a + b, 0)
export const calculateRedAmbrosiaLuck = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a + b, 0)
export const calculateRedAmbrosiaGenerationSpeed = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a * b, 1)
export const calculateFreeShopInfinityUpgrades = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a + b, 0)

export const calculateRawAntSpeedMult = (stats: readonly DecimalSource[]): Decimal =>
  stats.reduce<Decimal>((acc, v) => acc.times(v), new Decimal(1))

export const calculateRawPositiveSalvage = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a + b, 0)
export const calculateRawNegativeSalvage = (stats: readonly number[]): number =>
  stats.reduce((a, b) => a + b, 0)

// ─── Misc helpers ──────────────────────────────────────────────────────────

export interface CalculateTotalCoinOwnedInput {
  firstOwnedCoin: number
  secondOwnedCoin: number
  thirdOwnedCoin: number
  fourthOwnedCoin: number
  fifthOwnedCoin: number
}
export function calculateTotalCoinOwned(input: CalculateTotalCoinOwnedInput): number {
  return input.firstOwnedCoin
    + input.secondOwnedCoin
    + input.thirdOwnedCoin
    + input.fourthOwnedCoin
    + input.fifthOwnedCoin
}
