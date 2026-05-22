// Per-rune effect formulas. Each function maps (effective rune level n, effect
// key) → effect value. These are the pure cores of the `runes.<rune>.effects`
// fields in packages/web_ui/src/Runes.ts. The surrounding plumbing —
// EXP/level state, effective-level computation, free-level aggregation,
// unlock checks, UI descriptions — stays in web_ui.
//
// Two runes (infiniteAscent and antiquities) read singularity state for one
// of their keys; those values are hoisted into a small `input` argument and
// the shim recomputes them per call.

// ─── speed ─────────────────────────────────────────────────────────────────

export type SpeedRuneKey = 'acceleratorPower' | 'multiplicativeAccelerators' | 'globalSpeed'

export function speedRuneEffects(n: number, key: SpeedRuneKey): number {
  if (key === 'acceleratorPower') return 0.0002 * n
  if (key === 'multiplicativeAccelerators') return 1 + n / 400
  return 2 - Math.exp(-Math.cbrt(n) / 100) // globalSpeed
}

// ─── duplication ───────────────────────────────────────────────────────────

export type DuplicationRuneKey = 'multiplierBoosts' | 'multiplicativeMultipliers' | 'taxReduction'

export function duplicationRuneEffects(n: number, key: DuplicationRuneKey): number {
  if (key === 'multiplierBoosts') return n / 5
  if (key === 'multiplicativeMultipliers') return 1 + n / 400
  return 0.001 + .999 * Math.exp(-Math.cbrt(n) / 5) // taxReduction
}

// ─── prism ─────────────────────────────────────────────────────────────────

export type PrismRuneKey = 'productionLog10' | 'costDivisorLog10'

export function prismRuneEffects(n: number, key: PrismRuneKey): number {
  if (key === 'productionLog10') {
    return Math.max(0, 2 * Math.log10(1 + n / 2) + (n / 2) * Math.log10(2) - Math.log10(256))
  }
  return Math.floor(n / 10) // costDivisorLog10
}

// ─── thrift ────────────────────────────────────────────────────────────────

export type ThriftRuneKey = 'costDelay' | 'salvage' | 'taxReduction'

export function thriftRuneEffects(n: number, key: ThriftRuneKey): number {
  if (key === 'costDelay') return Math.min(1e15, n / 125)
  if (key === 'salvage') return 2.5 * Math.log(1 + n / 10)
  return 0.01 + 0.99 * Math.exp(-Math.cbrt(n) / 10) // taxReduction
}

// ─── superiorIntellect ─────────────────────────────────────────────────────

export type SuperiorIntellectRuneKey = 'offeringMult' | 'obtainiumMult' | 'antSpeed'

export function superiorIntellectRuneEffects(n: number, key: SuperiorIntellectRuneKey): number {
  if (key === 'offeringMult') return 1 + n / 2000
  if (key === 'obtainiumMult') return 1 + n / 200
  return Math.pow(1 + n / 500, 2) // antSpeed
}

// ─── infiniteAscent ────────────────────────────────────────────────────────

export type InfiniteAscentRuneKey = 'quarkMult' | 'cubeMult' | 'salvage'

export interface InfiniteAscentRuneInput {
  /**
   * Number of salvage-perk thresholds the player has unlocked. In web_ui this
   * is `salvagePerkLevels.filter(x => x <= player.highestSingularityCount).length`
   * where `salvagePerkLevels = [30, 40, 61, 81, 111, 131, 161, 191, 236, 260]`.
   * The perk-levels table is a UI config, so callers compute the count.
   */
  salvagePerkUnlockedCount: number
}

export function infiniteAscentRuneEffects(
  n: number,
  key: InfiniteAscentRuneKey,
  input: InfiniteAscentRuneInput
): number {
  if (key === 'quarkMult') return 1 + n / 500 + (n > 0 ? 0.1 : 0)
  if (key === 'cubeMult') return 1 + n / 100
  return n * 0.025 * input.salvagePerkUnlockedCount // salvage
}

// ─── antiquities ───────────────────────────────────────────────────────────

export type AntiquitiesRuneKey =
  | 'addCodeCooldownReduction'
  | 'offeringLog10'
  | 'obtainiumLog10'
  | 'cubeBonus'

export interface AntiquitiesRuneInput {
  /** player.singularityCount — feeds the cubeBonus exponent. */
  singularityCount: number
}

export function antiquitiesRuneEffects(
  n: number,
  key: AntiquitiesRuneKey,
  input: AntiquitiesRuneInput
): number {
  if (key === 'addCodeCooldownReduction') return n > 0 ? 0.8 - 0.3 * (n - 1) / (n + 10) : 1
  if (key === 'offeringLog10') return Math.round(300 * (1 - Math.pow(1 - 1 / 300, n)))
  if (key === 'obtainiumLog10') return Math.round(300 * (1 - Math.pow(1 - 1 / 300, n)))
  return (n > 0) ? Math.pow(1.01, Math.min(5, n) * input.singularityCount) : 1 // cubeBonus
}

// ─── horseShoe ─────────────────────────────────────────────────────────────

export type HorseShoeRuneKey = 'ambrosiaLuck' | 'redLuck' | 'redLuckConversion'

export function horseShoeRuneEffects(n: number, key: HorseShoeRuneKey): number {
  if (key === 'ambrosiaLuck') return n
  if (key === 'redLuck') return n / 5
  return -0.5 * n / (n + 50) // redLuckConversion
}

// ─── finiteDescent ─────────────────────────────────────────────────────────

export type FiniteDescentRuneKey = 'ascensionScore' | 'corruptionFreeLevels' | 'infiniteAscentFreeLevel'

export function finiteDescentRuneEffects(n: number, key: FiniteDescentRuneKey): number {
  if (key === 'ascensionScore') return n >= 1 ? 1.04 + 0.96 * (n - 1) / (n + 25) : 1
  if (key === 'corruptionFreeLevels') return n >= 1 ? 0.01 + 0.14 * (n - 1) / (n + 16) : 0
  return Math.floor(n / 2) // infiniteAscentFreeLevel
}

// ─── topHat ────────────────────────────────────────────────────────────────

export type TopHatRuneKey =
  | 'freeOfferingLevels'
  | 'freeObtainiumLevels'
  | 'freeCubeLevels'
  | 'freeSpeedLevels'
  | 'freeInfinityLevels'

export function topHatRuneEffects(n: number, key: TopHatRuneKey): number {
  if (key === 'freeOfferingLevels') return Math.round(200 * (1 - Math.pow(0.995, n))) / 10
  if (key === 'freeObtainiumLevels') return Math.round(200 * (1 - Math.pow(0.995, n))) / 10
  if (key === 'freeCubeLevels') return Math.round(150 * (1 - Math.pow(0.997, n))) / 10
  if (key === 'freeSpeedLevels') return Math.round(150 * (1 - Math.pow(0.997, n))) / 10
  return Math.round(100 * (1 - Math.pow(0.999, n))) / 10 // freeInfinityLevels
}
