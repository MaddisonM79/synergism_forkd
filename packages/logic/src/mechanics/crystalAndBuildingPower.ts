// Crystal + building-power production formulas. Lifted from
// packages/web_ui/src/Synergism.ts lines 2661-2745.
//
// All 8 functions are pure given pre-extracted player / antUpgrade / rune /
// challenge inputs. The two "Multiplier" functions accept an optional
// pre-computed base (power / exponent / base) so callers can avoid double
// evaluation when they already have it.

import { Decimal } from '../math/bignum'

// ─── Building power ───────────────────────────────────────────────────────

export interface CalculateBuildingPowerInput {
  /** CalcECC('reincarnation', player.challengecompletions[8]) — ×0.25 contribution. */
  c8ReincarnationECC: number
  /** player.reincarnationShards — additive atom-bonus contribution. */
  reincarnationShards: Decimal
  /** player.researches[36] — ×1/20 contribution to the additive base. */
  research36: number
  /** player.researches[37] — ×1/40 contribution. */
  research37: number
  /** player.researches[38] — ×1/40 contribution. */
  research38: number
  /** getAntUpgradeEffect(AntUpgrades.BuildingCostScale).buildingPowerMult. */
  buildingCostScaleAntUpgradeBuildingPowerMult: number
  /** player.cubeUpgrades[12] — exponent bump of ×0.09. */
  cubeUpgrade12: number
  /** player.cubeUpgrades[36] — exponent bump of ×0.05. */
  cubeUpgrade36: number
  /** Whether player.currentChallenge.reincarnation === 7 — collapses to `1 + 0.05 * power`. */
  inReincarnationChallenge7: boolean
}

/**
 * Per-building power scalar. Aggregates atom bonus (log of reincarnationShards),
 * challenge-8 reincarnation completions, the three research multipliers,
 * the building-cost-scale ant-upgrade mult, the two cube-upgrade exponent
 * bumps, then the challenge-7 final-fold case.
 */
export function calculateBuildingPower (input: CalculateBuildingPowerInput): number {
  const challenge8Bonus = 0.25 * input.c8ReincarnationECC

  let power = 1
  // Atom bonus
  power += (1 - Math.pow(2, -1 / 160))
    * Decimal.log(input.reincarnationShards.add(1), 10)
  // Challenge 8 reward
  power += challenge8Bonus

  // Researches
  power *= 1 + (1 / 20) * input.research36
  power *= 1 + (1 / 40) * input.research37
  power *= 1 + (1 / 40) * input.research38

  // Ant
  power *= input.buildingCostScaleAntUpgradeBuildingPowerMult

  // Cube upgrades raise the base to a power
  power = Math.pow(power, 1 + input.cubeUpgrade12 * 0.09)
  power = Math.pow(power, 1 + input.cubeUpgrade36 * 0.05)

  // Challenge 7 — collapses to a much smaller power
  if (input.inReincarnationChallenge7) {
    power = 1 + 0.05 * power
  }

  return power
}

/**
 * Coin-side building-power multiplier: `buildingPower^totalOwnedCoin`.
 * Caller passes pre-computed `buildingPower` and `totalOwnedCoin`.
 */
export function calculateBuildingPowerCoinMultiplier (buildingPower: number, totalOwnedCoin: number): Decimal {
  return Decimal.pow(buildingPower, totalOwnedCoin)
}

// ─── Crystal exponent ─────────────────────────────────────────────────────

export interface CrystalUpgrade4MaxExponentInput {
  /** player.researches[129] — ×0.05 × log_4(commonFragments+1). */
  research129: number
  /** player.commonFragments — log base 4. */
  commonFragments: Decimal
  /** getRuneSpiritEffect('prism').crystalCaps — additive contribution. */
  prismSpiritCrystalCaps: number
}

/** Cap on crystal-upgrade-3's exponent contribution. */
export function crystalUpgrade4MaxExponent (input: CrystalUpgrade4MaxExponentInput): number {
  let exponent = 10
  exponent += 0.05 * input.research129 * Decimal.log(input.commonFragments.add(1), 4)
  exponent += input.prismSpiritCrystalCaps
  return exponent
}

export interface CalculateCrystalExponentInput {
  /** Result of crystalUpgrade4MaxExponent — the cap. */
  crystalUpgrade3MaxExponent: number
  /** player.crystalUpgrades[3] — drives the ×(1 - 0.995^N) approach to the cap. */
  crystalUpgrade3: number
  /** CalcECC('transcend', player.challengecompletions[3]) — ×0.04 contribution. */
  c3TranscendECC: number
  /** player.researches[28] — Research 2x3, ×0.08 contribution. */
  research28: number
  /** player.researches[29] — Research 2x4, ×0.08 contribution. */
  research29: number
  /** player.researches[30] — Research 2x5, ×0.04 contribution. */
  research30: number
  /** player.cubeUpgrades[17] — Cube 2x7, ×8 contribution. */
  cubeUpgrade17: number
}

/**
 * Crystal exponent for the prestige-shards production formula. Base 1/3
 * plus capped crystal-upgrade-3 contribution, challenge-3 ECC, three
 * research lines, and cube-upgrade-17 (×8 contribution per level).
 */
export function calculateCrystalExponent (input: CalculateCrystalExponentInput): number {
  let exponent = 1 / 3
  exponent += input.crystalUpgrade3MaxExponent * (1 - Math.pow(0.995, input.crystalUpgrade3))
  exponent += 0.04 * input.c3TranscendECC
  exponent += 0.08 * input.research28
  exponent += 0.08 * input.research29
  exponent += 0.04 * input.research30
  exponent += 8 * input.cubeUpgrade17
  return exponent
}

/**
 * Coin-side crystal multiplier: `(prestigeShards + 1)^crystalExponent`.
 */
export function calculateCrystalCoinMultiplier (prestigeShards: Decimal, crystalExponent: number): Decimal {
  return Decimal.pow(prestigeShards.add(1), crystalExponent)
}

// ─── Crystal upgrade 3 base ───────────────────────────────────────────────

export interface CrystalUpgrade3MaxBaseInput {
  /** player.upgrades[122] — ×1 contribution. */
  upgrade122: number
  /** player.researches[129] — ×0.001 × log_4(commonFragments+1). */
  research129: number
  /** player.commonFragments — log base 4. */
  commonFragments: Decimal
}

/** Cap on crystal-upgrade-3's base. */
export function crystalUpgrade3MaxBase (input: CrystalUpgrade3MaxBaseInput): number {
  let maxBase = 2
  maxBase += input.upgrade122
  maxBase += 0.001 * input.research129 * Decimal.log(input.commonFragments.add(1), 4)
  return maxBase
}

export interface CrystalUpgrade3BaseInput {
  /** Result of crystalUpgrade3MaxBase. */
  maxBase: number
  /** player.crystalUpgrades[2] — drives the ×(1 - 0.999^N) approach to maxBase. */
  crystalUpgrade2: number
}

/** Effective base for crystal-upgrade-3's contribution to crystal production. */
export function crystalUpgrade3Base (input: CrystalUpgrade3BaseInput): number {
  return 1 + (input.maxBase - 1) * (1 - Math.pow(0.999, input.crystalUpgrade2))
}

export interface CrystalUpgrade3CrystalMultiplierInput {
  /** Result of crystalUpgrade3Base. */
  base: number
  /** Sum of player.first/second/third/fourth/fifthOwnedDiamonds — diamond producer count. */
  crystalProducersOwned: number
}

/** Crystal-side multiplier: `base^crystalProducersOwned`. */
export function crystalUpgrade3CrystalMultiplier (input: CrystalUpgrade3CrystalMultiplierInput): Decimal {
  return Decimal.pow(input.base, input.crystalProducersOwned)
}
