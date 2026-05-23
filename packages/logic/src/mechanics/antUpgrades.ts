// Ant-upgrade base costs + pure effect formulas + cost solvers. Lifted from:
//   packages/web_ui/src/Features/Ants/AntUpgrades/data/data.ts (effect closures)
//   packages/web_ui/src/Features/Ants/AntUpgrades/lib/get-cost.ts
//
// The data table is indexed 0..15 to match the AntUpgrades enum in web_ui:
//   AntSpeed=0, Coins=1, Taxes=2, AcceleratorBoosts=3, Multipliers=4,
//   Offerings=5, BuildingCostScale=6, Salvage=7, FreeRunes=8, Obtainium=9,
//   AntSacrifice=10, Mortuus=11, AntELO=12, WowCubes=13, AscensionScore=14,
//   Mortuus2=15
//
// Cost shape: cost-to-reach-level-N is baseCost × 10^((N-1) × costIncreaseExponent);
// cost-of-just-level-N is the delta from level-(N-1) to level-N. Cost solvers
// mirror the legacy logic in get-cost.ts exactly.
//
// Effect functions: most are pure 1-arg formulas. AntSpeed / Coins / AntELO
// read additional player state; those are parameterized through input objects.

import { Decimal } from '../math/bignum'
import { calculateSigmoidExponential } from '../math/sigmoid'

// ─── Cost data (parallel arrays indexed 0..15) ────────────────────────────

/** Base cost (level-0 → level-1) for each upgrade. */
export const antUpgradeBaseCosts: readonly Decimal[] = [
  Decimal.fromString('100'), // 0 AntSpeed
  Decimal.fromString('100'), // 1 Coins
  Decimal.fromString('1000'), // 2 Taxes
  Decimal.fromString('1000'), // 3 AcceleratorBoosts
  Decimal.fromString('1e5'), // 4 Multipliers
  Decimal.fromString('1e6'), // 5 Offerings
  Decimal.fromString('1e11'), // 6 BuildingCostScale
  Decimal.fromString('1e15'), // 7 Salvage
  Decimal.fromString('1e20'), // 8 FreeRunes
  Decimal.fromString('1e6'), // 9 Obtainium
  Decimal.fromString('1e120'), // 10 AntSacrifice
  Decimal.fromString('1e300'), // 11 Mortuus
  Decimal.fromString('1e70'), // 12 AntELO
  Decimal.fromString('1e400'), // 13 WowCubes
  Decimal.fromString('1e300'), // 14 AscensionScore
  Decimal.fromString('1e37777') // 15 Mortuus2
] as const

/** Per-level log-10 cost increase exponent. */
export const antUpgradeCostIncreaseExponents: readonly number[] = [
  1, // AntSpeed
  1, // Coins
  1, // Taxes
  1, // AcceleratorBoosts
  2, // Multipliers
  2, // Offerings
  2, // BuildingCostScale
  3, // Salvage
  3, // FreeRunes
  2, // Obtainium
  20, // AntSacrifice
  100, // Mortuus
  4, // AntELO
  10, // WowCubes
  2, // AscensionScore
  2000 // Mortuus2
] as const

// ─── Cost solvers ─────────────────────────────────────────────────────────

export interface AntUpgradeCostInput {
  /** antUpgradeBaseCosts[upgradeIndex] — caller indexes the table. */
  baseCost: Decimal
  /** antUpgradeCostIncreaseExponents[upgradeIndex]. */
  costIncreaseExponent: number
  /** player.ants.upgrades[upgradeIndex] — current owned level. */
  currentLevel: number
}

/**
 * Cost of buying the next level. The cost-to-reach-level-N formula is
 * `baseCost × 10^((N-1) × exp)`; the delta from current to next is
 * `nextCost - lastCost` (with lastCost=0 when currentLevel=0).
 */
export function getCostNextAntUpgrade (input: AntUpgradeCostInput): Decimal {
  const nextCost = input.baseCost.times(
    Decimal.pow(10, input.currentLevel * input.costIncreaseExponent)
  )
  const lastCost = input.currentLevel > 0
    ? input.baseCost.times(
      Decimal.pow(10, (input.currentLevel - 1) * input.costIncreaseExponent)
    )
    : Decimal.fromNumber(0)
  return nextCost.sub(lastCost)
}

export interface AntUpgradeMaxPurchasableInput {
  baseCost: Decimal
  costIncreaseExponent: number
  /** player.ants.upgrades[upgradeIndex]. */
  currentLevel: number
  /** Budget to spend (player.ants.crumbs in legacy). */
  budget: Decimal
}

/**
 * Max level reachable with `budget`. Re-adds the sunk cost (cost paid for
 * current level) to the budget to compute total spendable, then solves the
 * inverse: `level = 1 + floor(log10(realBudget/baseCost) / costIncreaseExponent)`.
 * Floored at 0.
 */
export function getMaxPurchasableAntUpgrades (input: AntUpgradeMaxPurchasableInput): number {
  const sunkCost = input.currentLevel > 0
    ? input.baseCost.times(
      Decimal.pow(10, input.costIncreaseExponent * (input.currentLevel - 1))
    )
    : Decimal.fromNumber(0)
  const realBudget = input.budget.add(sunkCost)
  return Math.max(
    0,
    1 + Math.floor(Decimal.log(realBudget.div(input.baseCost), 10) / input.costIncreaseExponent)
  )
}

/**
 * Cost to buy from currentLevel up to the max affordable level given the
 * budget. Caller supplies the result of getMaxPurchasableAntUpgrades as
 * `maxBuyable` (which may be 0 if budget can't reach next level).
 */
export interface AntUpgradeMaxCostInput {
  baseCost: Decimal
  costIncreaseExponent: number
  currentLevel: number
  /** Result of getMaxPurchasableAntUpgrades for the same baseCost/exp/level/budget. */
  maxBuyable: number
}

export function getCostMaxAntUpgrades (input: AntUpgradeMaxCostInput): Decimal {
  const spent = input.currentLevel > 0
    ? Decimal.pow(10, input.costIncreaseExponent * (input.currentLevel - 1))
      .times(input.baseCost)
    : Decimal.fromNumber(0)
  const maxCost = Decimal.pow(10, input.costIncreaseExponent * (input.maxBuyable - 1))
    .times(input.baseCost)
  return maxCost.sub(spent)
}

// ─── Pure effect functions (per upgrade) ──────────────────────────────────

export interface AntSpeedAntUpgradeInput {
  /** Current effective ant-upgrade level. */
  level: number
  /** player.researches[101] — Research 5x1. */
  research101: number
  /** player.researches[162] — Research 7x12. */
  research162: number
}
export interface AntSpeedAntUpgradeEffect { antSpeed: Decimal }
export function antSpeedAntUpgradeEffect (input: AntSpeedAntUpgradeInput): AntSpeedAntUpgradeEffect {
  const baseMul = 1.1 + input.research101 / 1000 + input.research162 / 1000
  return { antSpeed: Decimal.pow(baseMul, input.level) }
}

export interface CoinsAntUpgradeInput {
  level: number
  /** player.currentChallenge.ascension — affects divisor when === 15. */
  ascensionChallenge: number
  /** player.ants.crumbs — affects multiplier (Decimal.pow(crumbs, exponent)). */
  crumbs: Decimal
}
export interface CoinsAntUpgradeEffect { crumbToCoinExp: number; coinMultiplier: Decimal }
export function coinsAntUpgradeEffect (input: CoinsAntUpgradeInput): CoinsAntUpgradeEffect {
  const n = input.level
  let divisor = 1
  if (input.ascensionChallenge === 15) {
    divisor = 100 + 9900 * (1000 + n) / (1000 + n ** 2)
  }
  const baseExponent = 999999 + calculateSigmoidExponential(49000001, n / 3000)
  const bonusExponent = 250 * n
  const exponent = (baseExponent + bonusExponent) / divisor
  const coinMult = Decimal.max(1, Decimal.pow(input.crumbs, exponent))
  return { crumbToCoinExp: exponent, coinMultiplier: coinMult }
}

export interface TaxesAntUpgradeEffect { taxReduction: number }
export function taxesAntUpgradeEffect (level: number): TaxesAntUpgradeEffect {
  return { taxReduction: 0.005 + 0.995 * Math.pow(0.99, level) }
}

export interface AcceleratorBoostsAntUpgradeEffect { acceleratorBoostMult: number }
export function acceleratorBoostsAntUpgradeEffect (level: number): AcceleratorBoostsAntUpgradeEffect {
  return { acceleratorBoostMult: calculateSigmoidExponential(20, level / 1000) }
}

export interface MultipliersAntUpgradeEffect { multiplierMult: number }
export function multipliersAntUpgradeEffect (level: number): MultipliersAntUpgradeEffect {
  return { multiplierMult: calculateSigmoidExponential(40, level / 1000) }
}

export interface OfferingsAntUpgradeEffect { offeringMult: number }
export function offeringsAntUpgradeEffect (level: number): OfferingsAntUpgradeEffect {
  return { offeringMult: Math.pow(1 + level / 10, 0.5) }
}

export interface BuildingCostScaleAntUpgradeEffect {
  buildingCostScale: number
  buildingPowerMult: number
}
export function buildingCostScaleAntUpgradeEffect (level: number): BuildingCostScaleAntUpgradeEffect {
  return {
    buildingCostScale: (3 * level) / 100,
    buildingPowerMult: 1 + level / 100
  }
}

export interface SalvageAntUpgradeEffect { salvage: number }
export function salvageAntUpgradeEffect (level: number): SalvageAntUpgradeEffect {
  return { salvage: 120 * (1 - Math.pow(0.995, level)) }
}

export interface FreeRunesAntUpgradeEffect { freeRuneLevel: number }
export function freeRunesAntUpgradeEffect (level: number): FreeRunesAntUpgradeEffect {
  return { freeRuneLevel: 3000 * (1 - Math.pow(1 - 1 / 3000, level)) }
}

export interface ObtainiumAntUpgradeEffect { obtainiumMult: number }
export function obtainiumAntUpgradeEffect (level: number): ObtainiumAntUpgradeEffect {
  return { obtainiumMult: Math.pow(1 + level / 10, 0.5) }
}

export interface AntSacrificeAntUpgradeEffect {
  antSacrificeMultiplier: number
  elo: number
}
export function antSacrificeAntUpgradeEffect (level: number): AntSacrificeAntUpgradeEffect {
  return {
    antSacrificeMultiplier: Math.pow(1 + level / 10, 0.5),
    elo: Math.round(5 * Math.min(200, level))
  }
}

export interface MortuusAntUpgradeEffect {
  talismanUnlock: boolean
  globalSpeed: number
}
export function mortuusAntUpgradeEffect (level: number): MortuusAntUpgradeEffect {
  return {
    talismanUnlock: level > 0,
    globalSpeed: 2 - Math.pow(0.99, level)
  }
}

export interface AntELOAntUpgradeInput {
  level: number
  /** player.ants.antSacrificeCount. */
  antSacrificeCount: number
  /** +getAchievementReward('antSpeed2UpgradeImprover'). */
  antSpeed2UpgradeImprover: number
}
export interface AntELOAntUpgradeEffect {
  antELO: number
  antSacrificeLimitCount: number
}
export function antELOAntUpgradeEffect (input: AntELOAntUpgradeInput): AntELOAntUpgradeEffect {
  const n = input.level
  const antSacrificeLimitCount = n + 200 * Math.min(1, n)
  const upgradeImprover = Math.min(n, input.antSpeed2UpgradeImprover)
  const effectiveSacs = Math.min(
    antSacrificeLimitCount + upgradeImprover,
    input.antSacrificeCount + upgradeImprover
  )
  return {
    antELO: effectiveSacs,
    antSacrificeLimitCount
  }
}

export interface Mortuus2AntUpgradeEffect {
  talismanLevelIncreaser: number
  talismanEffectBuff: number
  ascensionSpeed: number
}
export function mortuus2AntUpgradeEffect (level: number): Mortuus2AntUpgradeEffect {
  return {
    talismanLevelIncreaser: Math.min(1200, Math.floor(level / 2)),
    talismanEffectBuff: 1 + 0.65 * (1 - Math.pow(0.999, level)) + 0.005 * Math.min(20, level),
    ascensionSpeed: 1 + 0.5 * (1 - Math.pow(0.996, level))
  }
}

export interface AscensionScoreAntUpgradeEffect {
  cubesBanked: number
  ascensionScoreBase: number
}
export function ascensionScoreAntUpgradeEffect (level: number): AscensionScoreAntUpgradeEffect {
  return {
    ascensionScoreBase: 100000 * (1 - Math.pow(0.999, level)),
    cubesBanked: 3 * Math.min(200, level)
      + 2500 * (1 - Math.pow(1 - 1 / 2750, level))
      + 96900 * (1 - Math.pow(1 - 1 / 969000, level))
  }
}

export interface WowCubesAntUpgradeEffect { wowCubes: number }
export function wowCubesAntUpgradeEffect (level: number): WowCubesAntUpgradeEffect {
  return { wowCubes: 2 - Math.pow(0.999, level) }
}
