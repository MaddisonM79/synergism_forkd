// Per-upgrade cost-formula / effect formulas for red-ambrosia upgrades,
// lifted from packages/web_ui/src/RedAmbrosiaUpgrades.ts.
//
// Web_ui still owns the redAmbrosiaUpgrades data table (it has UI fields
// the logic tier can't see: i18next-bound name/description/effectsDescription
// closures, the DOM-driven mobile/desktop renderers, the buy flow). This
// module owns the two pure-formula fields each upgrade has:
//
//   - costFormula(level, baseCost)           → number
//   - effects(n, key) [or effects(n)]        → reward field
//
// And the RedAmbrosiaUpgradeRewards type — pure data with no UI dependency,
// re-exported from web_ui's RedAmbrosiaUpgrades.ts so the `RedAmbrosiaNames`
// alias and external imports (Synergism.ts, types/Synergism.ts) keep
// compiling.
//
// One impure entry: salvageYinYang's effect reads
// `player.singularityChallenges.taxmanLastStand.enabled`. The logic version
// takes the gate as a third parameter; the web_ui data-table closure reads
// the player state and forwards.

export type RedAmbrosiaUpgradeRewards = {
  tutorial: { cubeMult: number; obtainiumMult: number; offeringMult: number }
  conversionImprovement1: { conversionImprovement: number }
  conversionImprovement2: { conversionImprovement: number }
  conversionImprovement3: { conversionImprovement: number }
  freeTutorialLevels: { freeLevels: number }
  freeLevelsRow2: { freeLevels: number }
  freeLevelsRow3: { freeLevels: number }
  freeLevelsRow4: { freeLevels: number }
  freeLevelsRow5: { freeLevels: number }
  blueberryGenerationSpeed: { blueberryGenerationSpeed: number }
  regularLuck: { ambrosiaLuck: number }
  redGenerationSpeed: { redAmbrosiaGenerationSpeed: number }
  redLuck: { redAmbrosiaLuck: number }
  redAmbrosiaCube: { unlockedRedAmbrosiaCube: boolean }
  redAmbrosiaObtainium: { unlockRedAmbrosiaObtainium: boolean }
  redAmbrosiaOffering: { unlockRedAmbrosiaOffering: boolean }
  redAmbrosiaCubeImprover: { extraExponent: number }
  viscount: { roleUnlock: boolean; quarkBonus: number; luckBonus: number; redLuckBonus: number }
  infiniteShopUpgrades: { freeLevels: number }
  redAmbrosiaAccelerator: { ambrosiaTimePerRedAmbrosia: number }
  regularLuck2: { ambrosiaLuck: number }
  blueberryGenerationSpeed2: { blueberryGenerationSpeed: number }
  salvageYinYang: { positiveSalvage: number; negativeSalvage: number }
  blueberries: { blueberries: number }
  redAmbrosiaFreeAccumulator: { freeAccumulatorLevels: number; freeAccumulatorLevelCapIncrease: number }
  freeOfferingUpgrades: { levels: number }
  freeObtainiumUpgrades: { levels: number }
  freeCubeUpgrades: { levels: number }
  freeSpeedUpgrades: { levels: number }
}

export type RedAmbrosiaNames = keyof RedAmbrosiaUpgradeRewards

// ─── Constant cost-table arrays ────────────────────────────────────────────
// Five upgrades use level-indexed lookup tables instead of formulas. Kept
// at module scope (hoisted constants — matches the web_ui legacy file).

const blueberryCostValues = [100_000, 1_400_000, 3_000_000, 3_250_000, 3_500_000]
const redAmbrosiaFreeAccumulatorValues = [100, 400, 1_000, 3_000, 10_000, 25_000, 75_000, 150_000, 400_000, 1_000_000]
const freeOfferingUpgradesValues = [1_000, 3_000, 9_000, 27_000, 81_000]
const freeObtainiumUpgradesValues = [1_500, 4_500, 13_500, 40_500, 121_500]
const freeCubeUpgradesValues = [10_000, 30_000, 90_000, 270_000, 810_000]
const freeSpeedUpgradesValues = [15_000, 45_000, 135_000, 405_000, 1_215_000]

// ─── Per-upgrade costFormula functions ─────────────────────────────────────
// Signature: (level: number, baseCost: number) => number.
// Most fall into a handful of shapes; named individually for code-locality
// with the matching effect functions and so callers can import individually.

export const tutorialCostFormula = (_level: number, baseCost: number): number => {
  return baseCost // Level has no effect.
}

export const conversionImprovement1CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(2, level)
}

export const conversionImprovement2CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(4, level)
}

export const conversionImprovement3CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(10, level)
}

export const freeTutorialLevelsCostFormula = (level: number, baseCost: number): number => {
  return baseCost + level
}

export const freeLevelsRow2CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(2, level)
}

export const freeLevelsRow3CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(2, level)
}

export const freeLevelsRow4CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(2, level)
}

export const freeLevelsRow5CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(2, level)
}

export const blueberryGenerationSpeedCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (level + 1)
}

export const regularLuckCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (level + 1)
}

export const redGenerationSpeedCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (level + 1)
}

export const redLuckCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (level + 1)
}

export const redAmbrosiaCubeCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (level + 1)
}

export const redAmbrosiaObtainiumCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (level + 1)
}

export const redAmbrosiaOfferingCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (level + 1)
}

export const redAmbrosiaCubeImproverCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (level + 1)
}

export const viscountCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (level + 1)
}

export const infiniteShopUpgradesCostFormula = (level: number, baseCost: number): number => {
  return baseCost + 100 * level
}

export const redAmbrosiaAcceleratorCostFormula = (_level: number, baseCost: number): number => {
  return baseCost
}

export const regularLuck2CostFormula = (_level: number, baseCost: number): number => {
  return baseCost
}

export const blueberryGenerationSpeed2CostFormula = (_level: number, baseCost: number): number => {
  return baseCost
}

export const salvageYinYangCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (level + 1)
}

export const blueberriesCostFormula = (level: number, _baseCost: number): number => {
  return blueberryCostValues[level] ?? 0
}

export const redAmbrosiaFreeAccumulatorCostFormula = (level: number, _baseCost: number): number => {
  return redAmbrosiaFreeAccumulatorValues[level] ?? 0
}

export const freeOfferingUpgradesCostFormula = (level: number, _baseCost: number): number => {
  return freeOfferingUpgradesValues[level] ?? 0
}

export const freeObtainiumUpgradesCostFormula = (level: number, _baseCost: number): number => {
  return freeObtainiumUpgradesValues[level] ?? 0
}

export const freeCubeUpgradesCostFormula = (level: number, _baseCost: number): number => {
  return freeCubeUpgradesValues[level] ?? 0
}

export const freeSpeedUpgradesCostFormula = (level: number, _baseCost: number): number => {
  return freeSpeedUpgradesValues[level] ?? 0
}

// ─── Per-upgrade effect functions ──────────────────────────────────────────
// Each function returns the matching field of RedAmbrosiaUpgradeRewards[name].
// Single-key upgrades collapse to (n) => value. Multi-key upgrades dispatch
// on `key` with the same if/else cascade as the legacy code, including the
// trailing `else` that returns the last-key value without checking — caller
// is expected to only pass valid keys.

export function tutorialEffect<K extends keyof RedAmbrosiaUpgradeRewards['tutorial']> (
  n: number,
  _key?: K
): RedAmbrosiaUpgradeRewards['tutorial'][K] {
  // All three reward keys (cubeMult, obtainiumMult, offeringMult) share the
  // same scaling — legacy ignores the key argument entirely.
  return Math.pow(1.01, n) as RedAmbrosiaUpgradeRewards['tutorial'][K]
}

export function conversionImprovement1Effect (
  n: number
): RedAmbrosiaUpgradeRewards['conversionImprovement1']['conversionImprovement'] {
  return -n
}

export function conversionImprovement2Effect (
  n: number
): RedAmbrosiaUpgradeRewards['conversionImprovement2']['conversionImprovement'] {
  return -n
}

export function conversionImprovement3Effect (
  n: number
): RedAmbrosiaUpgradeRewards['conversionImprovement3']['conversionImprovement'] {
  return -n
}

export function freeTutorialLevelsEffect (n: number): RedAmbrosiaUpgradeRewards['freeTutorialLevels']['freeLevels'] {
  return n
}

export function freeLevelsRow2Effect (n: number): RedAmbrosiaUpgradeRewards['freeLevelsRow2']['freeLevels'] {
  return n
}

export function freeLevelsRow3Effect (n: number): RedAmbrosiaUpgradeRewards['freeLevelsRow3']['freeLevels'] {
  return n
}

export function freeLevelsRow4Effect (n: number): RedAmbrosiaUpgradeRewards['freeLevelsRow4']['freeLevels'] {
  return n
}

export function freeLevelsRow5Effect (n: number): RedAmbrosiaUpgradeRewards['freeLevelsRow5']['freeLevels'] {
  return n
}

export function blueberryGenerationSpeedEffect (
  n: number
): RedAmbrosiaUpgradeRewards['blueberryGenerationSpeed']['blueberryGenerationSpeed'] {
  return 1 + n / 500
}

export function regularLuckEffect (n: number): RedAmbrosiaUpgradeRewards['regularLuck']['ambrosiaLuck'] {
  return 2 * n
}

export function redGenerationSpeedEffect (
  n: number
): RedAmbrosiaUpgradeRewards['redGenerationSpeed']['redAmbrosiaGenerationSpeed'] {
  return 1 + 3 * n / 1000
}

export function redLuckEffect (n: number): RedAmbrosiaUpgradeRewards['redLuck']['redAmbrosiaLuck'] {
  return n
}

export function redAmbrosiaCubeEffect (
  n: number
): RedAmbrosiaUpgradeRewards['redAmbrosiaCube']['unlockedRedAmbrosiaCube'] {
  return n > 0
}

export function redAmbrosiaObtainiumEffect (
  n: number
): RedAmbrosiaUpgradeRewards['redAmbrosiaObtainium']['unlockRedAmbrosiaObtainium'] {
  return n > 0
}

export function redAmbrosiaOfferingEffect (
  n: number
): RedAmbrosiaUpgradeRewards['redAmbrosiaOffering']['unlockRedAmbrosiaOffering'] {
  return n > 0
}

export function redAmbrosiaCubeImproverEffect (
  n: number
): RedAmbrosiaUpgradeRewards['redAmbrosiaCubeImprover']['extraExponent'] {
  return 0.01 * n
}

export function viscountEffect<K extends keyof RedAmbrosiaUpgradeRewards['viscount']> (
  n: number,
  key: K
): RedAmbrosiaUpgradeRewards['viscount'][K] {
  if (key === 'roleUnlock') {
    return (n > 0) as RedAmbrosiaUpgradeRewards['viscount'][K]
  } else if (key === 'quarkBonus') {
    return (1 + 0.1 * n) as RedAmbrosiaUpgradeRewards['viscount'][K]
  } else if (key === 'luckBonus') {
    return (125 * n) as RedAmbrosiaUpgradeRewards['viscount'][K]
  }
  // redLuckBonus
  return (25 * n) as RedAmbrosiaUpgradeRewards['viscount'][K]
}

export function infiniteShopUpgradesEffect (
  n: number
): RedAmbrosiaUpgradeRewards['infiniteShopUpgrades']['freeLevels'] {
  return n
}

export function redAmbrosiaAcceleratorEffect (
  n: number
): RedAmbrosiaUpgradeRewards['redAmbrosiaAccelerator']['ambrosiaTimePerRedAmbrosia'] {
  return 0.02 * n + 1 * +(n > 0)
}

export function regularLuck2Effect (n: number): RedAmbrosiaUpgradeRewards['regularLuck2']['ambrosiaLuck'] {
  return 2 * n
}

export function blueberryGenerationSpeed2Effect (
  n: number
): RedAmbrosiaUpgradeRewards['blueberryGenerationSpeed2']['blueberryGenerationSpeed'] {
  return 1 + n / 1000
}

// salvageYinYang's effect is gated by the taxmanLastStand singularity
// challenge — both reward keys return 0 when the challenge is enabled,
// otherwise return their normal value. Gate is passed in as the third
// parameter since the logic tier cannot read `player`.
export function salvageYinYangEffect<K extends keyof RedAmbrosiaUpgradeRewards['salvageYinYang']> (
  n: number,
  key: K,
  taxmanLastStandEnabled: boolean
): RedAmbrosiaUpgradeRewards['salvageYinYang'][K] {
  if (key === 'positiveSalvage') {
    if (taxmanLastStandEnabled) {
      return 0 as RedAmbrosiaUpgradeRewards['salvageYinYang'][K]
    }
    return (10 * n) as RedAmbrosiaUpgradeRewards['salvageYinYang'][K]
  }
  // negativeSalvage
  if (taxmanLastStandEnabled) {
    return 0 as RedAmbrosiaUpgradeRewards['salvageYinYang'][K]
  }
  return (-10 * n) as RedAmbrosiaUpgradeRewards['salvageYinYang'][K]
}

export function blueberriesEffect (n: number): RedAmbrosiaUpgradeRewards['blueberries']['blueberries'] {
  return n
}

export function redAmbrosiaFreeAccumulatorEffect<
  K extends keyof RedAmbrosiaUpgradeRewards['redAmbrosiaFreeAccumulator']
> (
  n: number,
  key: K
): RedAmbrosiaUpgradeRewards['redAmbrosiaFreeAccumulator'][K] {
  if (key === 'freeAccumulatorLevels') {
    return (n / 1000 + 0.01 * +(n > 0)) as RedAmbrosiaUpgradeRewards['redAmbrosiaFreeAccumulator'][K]
  }
  // freeAccumulatorLevelCapIncrease
  return (0.1 * n) as RedAmbrosiaUpgradeRewards['redAmbrosiaFreeAccumulator'][K]
}

export function freeOfferingUpgradesEffect (n: number): RedAmbrosiaUpgradeRewards['freeOfferingUpgrades']['levels'] {
  return n
}

export function freeObtainiumUpgradesEffect (n: number): RedAmbrosiaUpgradeRewards['freeObtainiumUpgrades']['levels'] {
  return n
}

export function freeCubeUpgradesEffect (n: number): RedAmbrosiaUpgradeRewards['freeCubeUpgrades']['levels'] {
  return n
}

export function freeSpeedUpgradesEffect (n: number): RedAmbrosiaUpgradeRewards['freeSpeedUpgrades']['levels'] {
  return n
}
