// Per-upgrade cost-formula / effect formulas for octeract upgrades, lifted
// from packages/web_ui/src/Octeracts.ts.
//
// Web_ui still owns the octeractUpgrades data table (it has UI fields the
// logic tier can't see: i18next-bound name/description/effectDescription
// closures, the DOM-driven modal renderer, the buy flow). This module owns
// the two pure-formula fields each upgrade has:
//
//   - costFormula(level, baseCost)           → number
//   - effect(n, key) [or effect(n)]          → reward field
//
// And the OcteractUpgradeRewards type — pure data with no UI dependency,
// re-exported from web_ui's Octeracts.ts so the `OcteractUpgrades` alias
// and external imports keep compiling.
//
// Five effects read mutable state outside the logic tier:
//   - octeractImprovedGlobalSpeed / ImprovedAscensionSpeed / ImprovedAscensionSpeed2:
//     need player.singularityCount
//   - octeractAscensionsOcteractGain: needs player.ascensionCount
//   - octeractQuarkGain2: needs the level of the octeractQuarkGain upgrade
//     (sibling lookup) and the BAL of the quark hepteract
// Each of those takes the extra value as a parameter; the web_ui data-table
// closure reads it from player/state and forwards.

export type OcteractUpgradeRewards = {
  octeractStarter: {
    quarkMult: number
    antSpeedMult: number
    octeractMult: number
  }
  octeractGain: { octeractMult: number }
  octeractGain2: { octeractMult: number }
  octeractQuarkGain: { quarkMult: number }
  octeractQuarkGain2: { quarkMult: number }
  octeractCorruption: { corruptionLevelCapIncrease: number }
  octeractGQCostReduce: { goldenQuarkCostMult: number }
  octeractExportQuarks: { exportQuarkMult: number }
  octeractImprovedDaily: { extraGoldenQuarks: number }
  octeractImprovedDaily2: { goldenQuarkMult: number }
  octeractImprovedDaily3: {
    extraGoldenQuarks: number
    goldenQuarkMult: number
  }
  octeractImprovedQuarkHept: { quarkHeptExponent: number }
  octeractImprovedGlobalSpeed: { globalSpeedMult: number }
  octeractImprovedAscensionSpeed: { ascensionSpeedMult: number }
  octeractImprovedAscensionSpeed2: { ascensionSpeedMult: number }
  octeractImprovedFree: {
    unlocked: boolean
    freeLevelPower: number
  }
  octeractImprovedFree2: { freeLevelPowerIncrease: number }
  octeractImprovedFree3: { freeLevelPowerIncrease: number }
  octeractImprovedFree4: { freeLevelPowerIncrease: number }
  octeractSingUpgradeCap: { goldenQuarkUpgradeCapIncrease: number }
  octeractOfferings1: { offeringMult: number }
  octeractObtainium1: { obtainiumMult: number }
  octeractAscensions: { ascensionCountMult: number }
  octeractAscensions2: { ascensionCountMult: number }
  octeractAscensionsOcteractGain: { octeractMult: number }
  octeractFastForward: { lookahead: number }
  octeractAutoPotionSpeed: { autoPotionSpeedMult: number }
  octeractAutoPotionEfficiency: { potionPowerMult: number }
  octeractOneMindImprover: { ascendSpeedExponent: number }
  octeractAmbrosiaLuck: { ambrosiaLuck: number }
  octeractAmbrosiaLuck2: { ambrosiaLuck: number }
  octeractAmbrosiaLuck3: { ambrosiaLuck: number }
  octeractAmbrosiaLuck4: { ambrosiaLuck: number }
  octeractAmbrosiaGeneration: { ambrosiaBarSpeedMult: number }
  octeractAmbrosiaGeneration2: { ambrosiaBarSpeedMult: number }
  octeractAmbrosiaGeneration3: { ambrosiaBarSpeedMult: number }
  octeractAmbrosiaGeneration4: { ambrosiaBarSpeedMult: number }
  octeractBonusTokens1: { lastCompletionBonusTokens: number }
  octeractBonusTokens2: { tokenMultiplier: number }
  octeractBonusTokens3: { firstCompletionBonusTokens: number }
  octeractBonusTokens4: { initialTokenBonus: number }
  octeractBlueberries: { blueberries: number }
  octeractInfiniteShopUpgrades: { infinityVouchers: number }
  octeractTalismanLevelCap1: { talismanLevelCapIncrease: number }
  octeractTalismanLevelCap2: { talismanLevelCapIncrease: number }
  octeractTalismanLevelCap3: { talismanLevelCapIncrease: number }
  octeractTalismanLevelCap4: { talismanLevelCapIncrease: number }
}

export type OcteractUpgrades = keyof OcteractUpgradeRewards

// Cost-lookup table for octeractBlueberries — fixed sequence of costs per
// level. The legacy file inlines this as `octeractBlueberryCostArr`.
const octeractBlueberryCostArr = [1, 1e3, 1e9, 1e27, 1e81, 1e111]

// ─── Per-upgrade costFormula functions ─────────────────────────────────────

export const octeractStarterCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (level + 1)
}

export const octeractGainCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (Math.pow(level + 1, 6) - Math.pow(level, 6))
}

export const octeractGain2CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(10, Math.pow(level, 0.5) / 3)
}

// octeractQuarkGain: linear-difference of (level+1)^7 below 1000, then a
// log-blow-up past with two extra fasterMult kickers at 10k and 15k. The
// (1001^7 - 1000^7) prefix is a constant — keeps the curve continuous at
// the knee. Verbatim from legacy.
export const octeractQuarkGainCostFormula = (level: number, baseCost: number): number => {
  if (level < 1000) {
    return baseCost * (Math.pow(level + 1, 7) - Math.pow(level, 7))
  }
  const fasterMult = (level >= 10000) ? Math.pow(10, (level - 10000) / 250) : 1
  const fasterMult2 = (level >= 15000) ? Math.pow(10, (level - 15000) / 250) : 1
  return baseCost * (Math.pow(1001, 7) - Math.pow(1000, 7)) * Math.pow(10, level / 1000) * fasterMult * fasterMult2
}

export const octeractQuarkGain2CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(1e20, level)
}

export const octeractCorruptionCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(10, level * 10)
}

export const octeractGQCostReduceCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(2, level)
}

export const octeractExportQuarksCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(level + 1, 3)
}

export const octeractImprovedDailyCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(1.6, level)
}

export const octeractImprovedDaily2CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(2, level)
}

export const octeractImprovedDaily3CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(20, level)
}

export const octeractImprovedQuarkHeptCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(1e3, level)
}

export const octeractImprovedGlobalSpeedCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(level + 1, 3)
}

export const octeractImprovedAscensionSpeedCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(1e9, level / 100)
}

export const octeractImprovedAscensionSpeed2CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(1e12, level / 250)
}

export const octeractImprovedFreeCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(level + 1, 3)
}

export const octeractImprovedFree2CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(level + 1, 3)
}

export const octeractImprovedFree3CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(level + 1, 3)
}

export const octeractImprovedFree4CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(1e20, level / 40)
}

export const octeractSingUpgradeCapCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(1e3, level)
}

// octeractOfferings1 / octeractObtainium1 share the same piecewise: a
// quintic below 25 levels, then a fixed prefix × 10^(level/25 - 1) past.
const octeractOfferingsObtainiumCost = (level: number, baseCost: number): number => {
  if (level < 25) {
    return baseCost * Math.pow(level + 1, 5)
  }
  return baseCost * 1e15 * Math.pow(10, level / 25 - 1)
}

export const octeractOfferings1CostFormula = octeractOfferingsObtainiumCost
export const octeractObtainium1CostFormula = octeractOfferingsObtainiumCost

export const octeractAscensionsCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(level + 1, 3)
}

export const octeractAscensions2CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(10, Math.pow(level, 0.5) / 3)
}

export const octeractAscensionsOcteractGainCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(40, level)
}

export const octeractFastForwardCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(1e8, level)
}

export const octeractAutoPotionSpeedCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(10, level)
}

export const octeractAutoPotionEfficiencyCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(10, level)
}

// octeractOneMindImprover: 1e5^level below 10, then an extra ×1e3 per level
// past 10. Both kickers compound.
export const octeractOneMindImproverCostFormula = (level: number, baseCost: number): number => {
  const fasterMult = (level >= 10) ? Math.pow(1e3, level - 10) : 1
  return baseCost * Math.pow(1e5, level) * fasterMult
}

// Difference-of-powers shapes reused across multiple ambrosia upgrades.
const tenPowerDiff = (level: number, baseCost: number): number => {
  const useLevel = level + 1
  return baseCost * (Math.pow(10, useLevel) - Math.pow(10, useLevel - 1))
}

const sixthPowerDiff = (level: number, baseCost: number): number => {
  return baseCost * (Math.pow(level + 1, 6) - Math.pow(level, 6))
}

const eighthPowerDiff = (level: number, baseCost: number): number => {
  return baseCost * (Math.pow(level + 1, 8) - Math.pow(level, 8))
}

const threePowerDiff = (level: number, baseCost: number): number => {
  const useLevel = level + 1
  return baseCost * (Math.pow(3, useLevel) - Math.pow(3, useLevel - 1))
}

export const octeractAmbrosiaLuckCostFormula = tenPowerDiff
export const octeractAmbrosiaLuck2CostFormula = sixthPowerDiff
export const octeractAmbrosiaLuck3CostFormula = eighthPowerDiff
export const octeractAmbrosiaLuck4CostFormula = threePowerDiff
export const octeractAmbrosiaGenerationCostFormula = tenPowerDiff
export const octeractAmbrosiaGeneration2CostFormula = sixthPowerDiff
export const octeractAmbrosiaGeneration3CostFormula = eighthPowerDiff
export const octeractAmbrosiaGeneration4CostFormula = threePowerDiff

export const octeractBonusTokens1CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(1e2, level)
}

export const octeractBonusTokens2CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(1e8, level)
}

export const octeractBonusTokens3CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(1e10, level)
}

export const octeractBonusTokens4CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(4, level)
}

// octeractBlueberries uses a fixed cost-lookup table — `baseCost` is unused.
// Legacy returns 0 once the cap of 6 is reached; we preserve that and the
// out-of-range default (`undefined` falls through to the lookup).
export const octeractBlueberriesCostFormula = (level: number, _baseCost: number): number => {
  if (level === 6) {
    return 0
  }
  return octeractBlueberryCostArr[level]
}

export const octeractInfiniteShopUpgradesCostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(16, level)
}

export const octeractTalismanLevelCap1CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(level + 1, 5)
}

export const octeractTalismanLevelCap2CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(level + 1, 10)
}

export const octeractTalismanLevelCap3CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(level + 1, 20)
}

export const octeractTalismanLevelCap4CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(10, level)
}

// ─── Per-upgrade effect functions ──────────────────────────────────────────

export function octeractStarterEffect<K extends keyof OcteractUpgradeRewards['octeractStarter']> (
  n: number,
  key: K
): OcteractUpgradeRewards['octeractStarter'][K] {
  if (key === 'quarkMult') {
    return (1 + 0.25 * n) as OcteractUpgradeRewards['octeractStarter'][K]
  }
  if (key === 'antSpeedMult') {
    return (1 + 99999 * n) as OcteractUpgradeRewards['octeractStarter'][K]
  }
  // octeractMult
  return (1 + 0.4 * n) as OcteractUpgradeRewards['octeractStarter'][K]
}

export function octeractGainEffect (n: number): OcteractUpgradeRewards['octeractGain']['octeractMult'] {
  return 1 + 0.01 * n
}

export function octeractGain2Effect (n: number): OcteractUpgradeRewards['octeractGain2']['octeractMult'] {
  return 1 + 0.01 * n
}

export function octeractQuarkGainEffect (n: number): OcteractUpgradeRewards['octeractQuarkGain']['quarkMult'] {
  return 1 + 0.011 * n
}

// octeractQuarkGain2: quark bonus scales with floor(quarkGainLevels / 111)
// and floor(1 + log10(max(1, hepteractQuarkBAL))). Both are sibling-state
// lookups in the legacy code — the logic version takes them as params.
export function octeractQuarkGain2Effect (
  n: number,
  quarkGainLevel: number,
  hepteractQuarkBAL: number
): OcteractUpgradeRewards['octeractQuarkGain2']['quarkMult'] {
  return 1
    + (1 / 10000) * Math.floor(quarkGainLevel / 111)
      * n
      * Math.floor(1 + Math.log10(Math.max(1, hepteractQuarkBAL)))
}

export function octeractCorruptionEffect (
  n: number
): OcteractUpgradeRewards['octeractCorruption']['corruptionLevelCapIncrease'] {
  return n
}

export function octeractGQCostReduceEffect (
  n: number
): OcteractUpgradeRewards['octeractGQCostReduce']['goldenQuarkCostMult'] {
  return 1 - n / 100
}

export function octeractExportQuarksEffect (
  n: number
): OcteractUpgradeRewards['octeractExportQuarks']['exportQuarkMult'] {
  return 4 * n / 10 + 1
}

export function octeractImprovedDailyEffect (
  n: number
): OcteractUpgradeRewards['octeractImprovedDaily']['extraGoldenQuarks'] {
  return n
}

export function octeractImprovedDaily2Effect (
  n: number
): OcteractUpgradeRewards['octeractImprovedDaily2']['goldenQuarkMult'] {
  return 1 + 0.01 * n
}

export function octeractImprovedDaily3Effect<K extends keyof OcteractUpgradeRewards['octeractImprovedDaily3']> (
  n: number,
  key: K
): OcteractUpgradeRewards['octeractImprovedDaily3'][K] {
  if (key === 'goldenQuarkMult') {
    return (1 + 0.005 * n) as OcteractUpgradeRewards['octeractImprovedDaily3'][K]
  }
  // extraGoldenQuarks
  return n as OcteractUpgradeRewards['octeractImprovedDaily3'][K]
}

export function octeractImprovedQuarkHeptEffect (
  n: number
): OcteractUpgradeRewards['octeractImprovedQuarkHept']['quarkHeptExponent'] {
  return n / 100
}

// Speed upgrades all scale with player.singularityCount.
export function octeractImprovedGlobalSpeedEffect (
  n: number,
  singularityCount: number
): OcteractUpgradeRewards['octeractImprovedGlobalSpeed']['globalSpeedMult'] {
  return 1 + n * singularityCount / 100
}

export function octeractImprovedAscensionSpeedEffect (
  n: number,
  singularityCount: number
): OcteractUpgradeRewards['octeractImprovedAscensionSpeed']['ascensionSpeedMult'] {
  return 1 + n * singularityCount / 2000
}

export function octeractImprovedAscensionSpeed2Effect (
  n: number,
  singularityCount: number
): OcteractUpgradeRewards['octeractImprovedAscensionSpeed2']['ascensionSpeedMult'] {
  return 1 + n * singularityCount / 2000
}

export function octeractImprovedFreeEffect<K extends keyof OcteractUpgradeRewards['octeractImprovedFree']> (
  n: number,
  key: K
): OcteractUpgradeRewards['octeractImprovedFree'][K] {
  if (key === 'unlocked') {
    return (n > 0) as OcteractUpgradeRewards['octeractImprovedFree'][K]
  }
  // freeLevelPower
  return (0.6 * n) as OcteractUpgradeRewards['octeractImprovedFree'][K]
}

export function octeractImprovedFree2Effect (
  n: number
): OcteractUpgradeRewards['octeractImprovedFree2']['freeLevelPowerIncrease'] {
  return 0.05 * n
}

export function octeractImprovedFree3Effect (
  n: number
): OcteractUpgradeRewards['octeractImprovedFree3']['freeLevelPowerIncrease'] {
  return 0.05 * n
}

// octeractImprovedFree4: linear scaling plus a +0.01 floor bump on first
// level. Verbatim from legacy.
export function octeractImprovedFree4Effect (
  n: number
): OcteractUpgradeRewards['octeractImprovedFree4']['freeLevelPowerIncrease'] {
  return 0.001 * n + ((n > 0) ? 0.01 : 0)
}

export function octeractSingUpgradeCapEffect (
  n: number
): OcteractUpgradeRewards['octeractSingUpgradeCap']['goldenQuarkUpgradeCapIncrease'] {
  return n
}

export function octeractOfferings1Effect (n: number): OcteractUpgradeRewards['octeractOfferings1']['offeringMult'] {
  return 1 + 0.01 * n
}

export function octeractObtainium1Effect (n: number): OcteractUpgradeRewards['octeractObtainium1']['obtainiumMult'] {
  return 1 + 0.01 * n
}

// octeractAscensions / octeractAscensions2 share the same effect: a +1% per
// level plus a +2% bump every 10 levels.
const ascensionCountMultEffect = (n: number): number => (1 + n / 100) * (1 + 2 * Math.floor(n / 10) / 100)

export function octeractAscensionsEffect (
  n: number
): OcteractUpgradeRewards['octeractAscensions']['ascensionCountMult'] {
  return ascensionCountMultEffect(n)
}

export function octeractAscensions2Effect (
  n: number
): OcteractUpgradeRewards['octeractAscensions2']['ascensionCountMult'] {
  return ascensionCountMultEffect(n)
}

// octeractAscensionsOcteractGain: exponent into 1 + log10(1 + ascensionCount).
export function octeractAscensionsOcteractGainEffect (
  n: number,
  ascensionCount: number
): OcteractUpgradeRewards['octeractAscensionsOcteractGain']['octeractMult'] {
  return Math.pow(1 + n / 100, 1 + Math.floor(Math.log10(1 + ascensionCount)))
}

export function octeractFastForwardEffect (n: number): OcteractUpgradeRewards['octeractFastForward']['lookahead'] {
  return n
}

export function octeractAutoPotionSpeedEffect (
  n: number
): OcteractUpgradeRewards['octeractAutoPotionSpeed']['autoPotionSpeedMult'] {
  return 1 + 4 * n / 100
}

export function octeractAutoPotionEfficiencyEffect (
  n: number
): OcteractUpgradeRewards['octeractAutoPotionEfficiency']['potionPowerMult'] {
  return 1 + 2 * n / 100
}

export function octeractOneMindImproverEffect (
  n: number
): OcteractUpgradeRewards['octeractOneMindImprover']['ascendSpeedExponent'] {
  return 0.55 + n / 150
}

export function octeractAmbrosiaLuckEffect (n: number): OcteractUpgradeRewards['octeractAmbrosiaLuck']['ambrosiaLuck'] {
  return 4 * n
}

export function octeractAmbrosiaLuck2Effect (
  n: number
): OcteractUpgradeRewards['octeractAmbrosiaLuck2']['ambrosiaLuck'] {
  return 2 * n
}

export function octeractAmbrosiaLuck3Effect (
  n: number
): OcteractUpgradeRewards['octeractAmbrosiaLuck3']['ambrosiaLuck'] {
  return 3 * n
}

export function octeractAmbrosiaLuck4Effect (
  n: number
): OcteractUpgradeRewards['octeractAmbrosiaLuck4']['ambrosiaLuck'] {
  return 5 * n
}

// All four AmbrosiaGeneration share +1%/level except #4 which doubles it.
export function octeractAmbrosiaGenerationEffect (
  n: number
): OcteractUpgradeRewards['octeractAmbrosiaGeneration']['ambrosiaBarSpeedMult'] {
  return 1 + n / 100
}

export function octeractAmbrosiaGeneration2Effect (
  n: number
): OcteractUpgradeRewards['octeractAmbrosiaGeneration2']['ambrosiaBarSpeedMult'] {
  return 1 + n / 100
}

export function octeractAmbrosiaGeneration3Effect (
  n: number
): OcteractUpgradeRewards['octeractAmbrosiaGeneration3']['ambrosiaBarSpeedMult'] {
  return 1 + n / 100
}

export function octeractAmbrosiaGeneration4Effect (
  n: number
): OcteractUpgradeRewards['octeractAmbrosiaGeneration4']['ambrosiaBarSpeedMult'] {
  return 1 + 2 * n / 100
}

export function octeractBonusTokens1Effect (
  n: number
): OcteractUpgradeRewards['octeractBonusTokens1']['lastCompletionBonusTokens'] {
  return n
}

export function octeractBonusTokens2Effect (
  n: number
): OcteractUpgradeRewards['octeractBonusTokens2']['tokenMultiplier'] {
  return 1 + n / 100
}

export function octeractBonusTokens3Effect (
  n: number
): OcteractUpgradeRewards['octeractBonusTokens3']['firstCompletionBonusTokens'] {
  return n
}

export function octeractBonusTokens4Effect (
  n: number
): OcteractUpgradeRewards['octeractBonusTokens4']['initialTokenBonus'] {
  return 2 * n
}

export function octeractBlueberriesEffect (n: number): OcteractUpgradeRewards['octeractBlueberries']['blueberries'] {
  return n
}

export function octeractInfiniteShopUpgradesEffect (
  n: number
): OcteractUpgradeRewards['octeractInfiniteShopUpgrades']['infinityVouchers'] {
  return n
}

export function octeractTalismanLevelCap1Effect (
  n: number
): OcteractUpgradeRewards['octeractTalismanLevelCap1']['talismanLevelCapIncrease'] {
  return n
}

export function octeractTalismanLevelCap2Effect (
  n: number
): OcteractUpgradeRewards['octeractTalismanLevelCap2']['talismanLevelCapIncrease'] {
  return n
}

export function octeractTalismanLevelCap3Effect (
  n: number
): OcteractUpgradeRewards['octeractTalismanLevelCap3']['talismanLevelCapIncrease'] {
  return n
}

export function octeractTalismanLevelCap4Effect (
  n: number
): OcteractUpgradeRewards['octeractTalismanLevelCap4']['talismanLevelCapIncrease'] {
  return n
}
