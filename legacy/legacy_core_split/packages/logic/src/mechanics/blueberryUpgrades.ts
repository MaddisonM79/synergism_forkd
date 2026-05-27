// Per-upgrade cost-formula / effect formulas for blueberry (ambrosia)
// upgrades, lifted from packages/web_ui/src/BlueberryUpgrades.ts.
//
// Web_ui still owns the ambrosiaUpgrades data table (it has UI fields the
// logic tier can't see: i18next-bound name/description/effectsDescription
// closures, DOM-driven mobile/desktop renderers, the buy flow, the
// `extraLevelCalc` closures that read other red-ambrosia upgrades). This
// module owns the two pure-formula fields each upgrade has:
//
//   - costFormula(level, baseCost)           → number
//   - effects(n, key) [or effects(n)]        → reward field
//
// And the AmbrosiaUpgradeRewards type — pure data with no UI dependency,
// re-exported from web_ui's BlueberryUpgrades.ts so the
// `AmbrosiaUpgradeNames` alias and external imports keep compiling.
//
// Several effects read player state. Those functions take the player-
// derived value as an extra parameter; the web_ui data-table closure reads
// the player state and forwards. The same pattern as salvageYinYang in
// redAmbrosiaUpgrades.ts.

export type AmbrosiaUpgradeRewards = {
  ambrosiaTutorial: { quarks: number; cubes: number }
  ambrosiaQuarks1: { quarks: number }
  ambrosiaCubes1: { cubes: number }
  ambrosiaLuck1: { ambrosiaLuck: number }
  ambrosiaQuarkCube1: { cubes: number }
  ambrosiaLuckCube1: { cubes: number }
  ambrosiaCubeQuark1: { quarks: number }
  ambrosiaLuckQuark1: { quarks: number }
  ambrosiaCubeLuck1: { ambrosiaLuck: number }
  ambrosiaQuarkLuck1: { ambrosiaLuck: number }
  ambrosiaQuarks2: { quarks: number }
  ambrosiaCubes2: { cubes: number }
  ambrosiaLuck2: { ambrosiaLuck: number }
  ambrosiaQuarks3: { quarks: number }
  ambrosiaCubes3: { cubes: number }
  ambrosiaLuck3: { ambrosiaLuck: number }
  ambrosiaLuck4: { ambrosiaLuckPercentage: number }
  ambrosiaPatreon: { blueberryGeneration: number }
  ambrosiaObtainium1: { obtainiumMult: number }
  ambrosiaOffering1: { offeringMult: number }
  ambrosiaHyperflux: { hyperFlux: number }
  ambrosiaBaseOffering1: { offering: number }
  ambrosiaBaseObtainium1: { obtainium: number }
  ambrosiaBaseOffering2: { offering: number }
  ambrosiaBaseObtainium2: { obtainium: number }
  ambrosiaSingReduction1: { singularityReduction: number }
  ambrosiaInfiniteShopUpgrades1: { freeLevels: number }
  ambrosiaInfiniteShopUpgrades2: { freeLevels: number }
  ambrosiaSingReduction2: { singularityReduction: number }
  ambrosiaTalismanBonusRuneLevel: { talismanBonusRuneLevel: number }
  ambrosiaRuneOOMBonus: { runeOOMBonus: number; infiniteAscentOOMBonus: number }
  ambrosiaBrickOfLead: { barRequirementMult: number; additiveLuckMult: number; singularitySpeedMult: number }
  ambrosiaFreeQuarkUpgrades: { freeQuarkUpgrades: number }
  ambrosiaFreeLuckUpgrades: { freeLuckUpgrades: number }
  ambrosiaFreeGenerationUpgrades: { freeGenerationUpgrades: number }
  ambrosiaFreeRedLuckUpgrades: { freeRedLuckUpgrades: number }
}

export type AmbrosiaUpgradeNames = keyof AmbrosiaUpgradeRewards

// ─── Per-upgrade costFormula functions ─────────────────────────────────────
// All pure: signature (level, baseCost) → number. A small handful of
// repeating shapes — kept individually named for code-locality with the
// matching effect functions and so callers can import individually.

export const ambrosiaTutorialCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (Math.pow(level + 1, 2) - Math.pow(level, 2))
}

// Cubic-difference shape used by many of the row-1 / row-2 / row-3 upgrades.
const cubicDifference = (level: number, baseCost: number): number => {
  return baseCost * (Math.pow(level + 1, 3) - Math.pow(level, 3))
}

export const ambrosiaQuarks1CostFormula = cubicDifference
export const ambrosiaCubes1CostFormula = cubicDifference
export const ambrosiaLuck1CostFormula = cubicDifference
export const ambrosiaQuarkCube1CostFormula = cubicDifference
export const ambrosiaLuckCube1CostFormula = cubicDifference
export const ambrosiaCubeQuark1CostFormula = cubicDifference
export const ambrosiaLuckQuark1CostFormula = cubicDifference
export const ambrosiaCubeLuck1CostFormula = cubicDifference
export const ambrosiaQuarkLuck1CostFormula = cubicDifference

// Quadratic-difference shape used by row-2 upgrades.
const quadraticDifference = (level: number, baseCost: number): number => {
  return baseCost * (Math.pow(level + 1, 2) - Math.pow(level, 2))
}

export const ambrosiaQuarks2CostFormula = quadraticDifference
export const ambrosiaCubes2CostFormula = quadraticDifference
export const ambrosiaLuck2CostFormula = quadraticDifference

export const ambrosiaQuarks3CostFormula = (level: number, baseCost: number): number => {
  return baseCost + 50000 * level
}

export const ambrosiaCubes3CostFormula = (level: number, baseCost: number): number => {
  return baseCost + 5000 * level
}

export const ambrosiaLuck3CostFormula = (_level: number, baseCost: number): number => {
  return baseCost // Level has no effect
}

export const ambrosiaLuck4CostFormula = (level: number, baseCost: number): number => {
  return baseCost + 20000 * level
}

export const ambrosiaPatreonCostFormula = quadraticDifference

export const ambrosiaObtainium1CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(25, level)
}

export const ambrosiaOffering1CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(25, level)
}

export const ambrosiaHyperfluxCostFormula = (level: number, baseCost: number): number => {
  // Linear within first 4 levels, exponentiates past level 4. The trailing
  // Math.max guarantees the multiplier is at least 1 for the early levels.
  return (baseCost + 33333 * Math.min(4, level)) * Math.max(1, Math.pow(3, level - 4))
}

export const ambrosiaBaseOffering1CostFormula = cubicDifference
export const ambrosiaBaseObtainium1CostFormula = cubicDifference
export const ambrosiaBaseOffering2CostFormula = cubicDifference
export const ambrosiaBaseObtainium2CostFormula = cubicDifference

export const ambrosiaSingReduction1CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(99, level)
}

export const ambrosiaInfiniteShopUpgrades1CostFormula = (_level: number, baseCost: number): number => {
  return baseCost
}

export const ambrosiaInfiniteShopUpgrades2CostFormula = (_level: number, baseCost: number): number => {
  return baseCost
}

export const ambrosiaSingReduction2CostFormula = (level: number, baseCost: number): number => {
  return baseCost * Math.pow(3, level)
}

export const ambrosiaTalismanBonusRuneLevelCostFormula = quadraticDifference

export const ambrosiaRuneOOMBonusCostFormula = (level: number, baseCost: number): number => {
  return Math.ceil(baseCost * (Math.pow(level + 1, 1.5) - Math.pow(level, 1.5)))
}

export const ambrosiaBrickOfLeadCostFormula = cubicDifference
export const ambrosiaFreeLuckUpgradesCostFormula = quadraticDifference

export const ambrosiaFreeGenerationUpgradesCostFormula = (level: number, baseCost: number): number => {
  return baseCost * (Math.pow(10, level + 1) - Math.pow(10, level))
}

export const ambrosiaFreeRedLuckUpgradesCostFormula = quadraticDifference
export const ambrosiaFreeQuarkUpgradesCostFormula = cubicDifference

// ─── Per-upgrade effect functions ──────────────────────────────────────────
// Each function returns the matching field of AmbrosiaUpgradeRewards[name].
// Effects that read player state take that state as an extra parameter.

export function ambrosiaTutorialEffect<K extends keyof AmbrosiaUpgradeRewards['ambrosiaTutorial']> (
  n: number,
  key: K
): AmbrosiaUpgradeRewards['ambrosiaTutorial'][K] {
  if (key === 'cubes') {
    return (1 + 0.05 * n) as AmbrosiaUpgradeRewards['ambrosiaTutorial'][K]
  }
  // quarks
  return (1 + 0.01 * n) as AmbrosiaUpgradeRewards['ambrosiaTutorial'][K]
}

export function ambrosiaQuarks1Effect (n: number): AmbrosiaUpgradeRewards['ambrosiaQuarks1']['quarks'] {
  return 1 + 0.01 * n
}

export function ambrosiaCubes1Effect (n: number): AmbrosiaUpgradeRewards['ambrosiaCubes1']['cubes'] {
  // +5% per level, plus a +10% multiplier every 5 levels (stair-step).
  return (1 + 0.05 * n) * Math.pow(1.1, Math.floor(n / 5))
}

export function ambrosiaLuck1Effect (n: number): AmbrosiaUpgradeRewards['ambrosiaLuck1']['ambrosiaLuck'] {
  return 2 * n + 12 * Math.floor(n / 10)
}

// ambrosiaQuarkCube1: cube bonus scales with `floor((log10(worlds+1)+1)^2)`.
export function ambrosiaQuarkCube1Effect (
  n: number,
  worlds: number
): AmbrosiaUpgradeRewards['ambrosiaQuarkCube1']['cubes'] {
  const baseVal = 0.001 * n
  return 1 + baseVal * Math.floor(Math.pow(Math.log10(worlds + 1) + 1, 2))
}

// ambrosiaLuckCube1: cube bonus scales with current ambrosiaLuck.
export function ambrosiaLuckCube1Effect (
  n: number,
  ambrosiaLuck: number
): AmbrosiaUpgradeRewards['ambrosiaLuckCube1']['cubes'] {
  const baseVal = 0.0005 * n
  return 1 + baseVal * ambrosiaLuck
}

// ambrosiaCubeQuark1: quark bonus scales with `sum of floor(log10(wowX+1))`
// across all six cube tiers, plus 6. The caller pre-computes that sum.
export function ambrosiaCubeQuark1Effect (
  n: number,
  wowCubeLogSum: number
): AmbrosiaUpgradeRewards['ambrosiaCubeQuark1']['quarks'] {
  const baseVal = 0.0001 * n
  return 1 + baseVal * (wowCubeLogSum + 6)
}

// ambrosiaLuckQuark1: quark bonus scales with min(luck, sqrt(1000*luck)).
export function ambrosiaLuckQuark1Effect (
  n: number,
  ambrosiaLuck: number
): AmbrosiaUpgradeRewards['ambrosiaLuckQuark1']['quarks'] {
  const baseVal = 0.0001 * n
  const effectiveLuck = Math.min(
    ambrosiaLuck,
    Math.pow(1000, 0.5) * Math.pow(ambrosiaLuck, 0.5)
  )
  return 1 + baseVal * effectiveLuck
}

// ambrosiaCubeLuck1: luck bonus scales with the same wowCubeLogSum + 6.
export function ambrosiaCubeLuck1Effect (
  n: number,
  wowCubeLogSum: number
): AmbrosiaUpgradeRewards['ambrosiaCubeLuck1']['ambrosiaLuck'] {
  const baseVal = 0.02 * n
  return baseVal * (wowCubeLogSum + 6)
}

// ambrosiaQuarkLuck1: luck bonus scales with `floor((log10(worlds+1)+1)^2)`.
export function ambrosiaQuarkLuck1Effect (
  n: number,
  worlds: number
): AmbrosiaUpgradeRewards['ambrosiaQuarkLuck1']['ambrosiaLuck'] {
  const baseVal = 0.02 * n
  return baseVal * Math.floor(Math.pow(Math.log10(worlds + 1) + 1, 2))
}

// ambrosiaQuarks2: scaled by floor(quarks1EffectiveLevels / 10) / 1000.
export function ambrosiaQuarks2Effect (
  n: number,
  quarks1EffectiveLevels: number
): AmbrosiaUpgradeRewards['ambrosiaQuarks2']['quarks'] {
  return 1 + (0.01 + Math.floor(quarks1EffectiveLevels / 10) / 1000) * n
}

// ambrosiaCubes2: similar shape with 10× weighting on the milestone term,
// then a stair-step ×1.15 every 5 levels on top.
export function ambrosiaCubes2Effect (
  n: number,
  cubes1EffectiveLevels: number
): AmbrosiaUpgradeRewards['ambrosiaCubes2']['cubes'] {
  return (1 + (0.1 + 10 * (Math.floor(cubes1EffectiveLevels / 10) / 1000)) * n)
    * Math.pow(1.15, Math.floor(n / 5))
}

// ambrosiaLuck2: base scales with luck1 milestones plus a +40 every 10 of n.
export function ambrosiaLuck2Effect (
  n: number,
  luck1EffectiveLevels: number
): AmbrosiaUpgradeRewards['ambrosiaLuck2']['ambrosiaLuck'] {
  return (3 + 0.3 * Math.floor(luck1EffectiveLevels / 10)) * n + 40 * Math.floor(n / 10)
}

// ambrosiaQuarks3: base 5% per level multiplied by quarks2 milestone.
export function ambrosiaQuarks3Effect (
  n: number,
  quarks2EffectiveLevels: number
): AmbrosiaUpgradeRewards['ambrosiaQuarks3']['quarks'] {
  const quark2Mult = 1 + quarks2EffectiveLevels / 100
  const quark3Base = 0.05 * n
  return 1 + quark3Base * quark2Mult
}

// ambrosiaCubes3: base 20% scaled by cubes2 milestone, plus a stair-step
// ×1.2 every 5 levels.
export function ambrosiaCubes3Effect (
  n: number,
  cubes2EffectiveLevels: number
): AmbrosiaUpgradeRewards['ambrosiaCubes3']['cubes'] {
  const cube2Multi = 1 + 3 * cubes2EffectiveLevels / 100
  const cube3Base = 0.2 * n
  const cube3Exponential = Math.pow(1.2, Math.floor(n / 5))
  return (1 + cube3Base * cube2Multi) * cube3Exponential
}

// ambrosiaLuck3: linear, scaled by current blueberry inventory.
export function ambrosiaLuck3Effect (
  n: number,
  blueberryInventory: number
): AmbrosiaUpgradeRewards['ambrosiaLuck3']['ambrosiaLuck'] {
  return blueberryInventory * n
}

// ambrosiaLuck4: luck percentage scales with the sum of OOM digits of two
// lifetime totals. Caller pre-computes the OOM digits.
export function ambrosiaLuck4Effect (
  n: number,
  lifetimeRedAmbrosia: number,
  lifetimeAmbrosia: number
): AmbrosiaUpgradeRewards['ambrosiaLuck4']['ambrosiaLuckPercentage'] {
  const digits = Math.ceil(Math.log10(lifetimeRedAmbrosia + 1))
    + Math.ceil(Math.log10(lifetimeAmbrosia + 1))
  return digits * n / 10000
}

// ambrosiaPatreon: linear, scaled by current quark bonus.
export function ambrosiaPatreonEffect (
  n: number,
  quarkBonus: number
): AmbrosiaUpgradeRewards['ambrosiaPatreon']['blueberryGeneration'] {
  return 1 + (n * quarkBonus) / 100
}

// ambrosiaObtainium1: scales with current ambrosiaLuck.
export function ambrosiaObtainium1Effect (
  n: number,
  ambrosiaLuck: number
): AmbrosiaUpgradeRewards['ambrosiaObtainium1']['obtainiumMult'] {
  return 1 + n * ambrosiaLuck / 1000
}

// ambrosiaOffering1: same formula as obtainium1.
export function ambrosiaOffering1Effect (
  n: number,
  ambrosiaLuck: number
): AmbrosiaUpgradeRewards['ambrosiaOffering1']['offeringMult'] {
  return 1 + n * ambrosiaLuck / 1000
}

// ambrosiaHyperflux: exponent into platonicUpgrades[19].
export function ambrosiaHyperfluxEffect (
  n: number,
  platonicUpgrade19: number
): AmbrosiaUpgradeRewards['ambrosiaHyperflux']['hyperFlux'] {
  return Math.pow(1 + n / 100, platonicUpgrade19)
}

export function ambrosiaBaseOffering1Effect (n: number): AmbrosiaUpgradeRewards['ambrosiaBaseOffering1']['offering'] {
  return n
}

export function ambrosiaBaseObtainium1Effect (
  n: number
): AmbrosiaUpgradeRewards['ambrosiaBaseObtainium1']['obtainium'] {
  return n
}

export function ambrosiaBaseOffering2Effect (n: number): AmbrosiaUpgradeRewards['ambrosiaBaseOffering2']['offering'] {
  return n
}

export function ambrosiaBaseObtainium2Effect (
  n: number
): AmbrosiaUpgradeRewards['ambrosiaBaseObtainium2']['obtainium'] {
  return n
}

// ambrosiaSingReduction1: gated OFF while inside a singularity challenge.
export function ambrosiaSingReduction1Effect (
  n: number,
  insideSingularityChallenge: boolean
): AmbrosiaUpgradeRewards['ambrosiaSingReduction1']['singularityReduction'] {
  if (insideSingularityChallenge) {
    return 0
  }
  return n
}

export function ambrosiaInfiniteShopUpgrades1Effect (
  n: number
): AmbrosiaUpgradeRewards['ambrosiaInfiniteShopUpgrades1']['freeLevels'] {
  return n
}

export function ambrosiaInfiniteShopUpgrades2Effect (
  n: number
): AmbrosiaUpgradeRewards['ambrosiaInfiniteShopUpgrades2']['freeLevels'] {
  return n
}

// ambrosiaSingReduction2: gated ON only while inside a singularity challenge.
export function ambrosiaSingReduction2Effect (
  n: number,
  insideSingularityChallenge: boolean
): AmbrosiaUpgradeRewards['ambrosiaSingReduction2']['singularityReduction'] {
  if (insideSingularityChallenge) {
    return n
  }
  return 0
}

export function ambrosiaTalismanBonusRuneLevelEffect (
  n: number
): AmbrosiaUpgradeRewards['ambrosiaTalismanBonusRuneLevel']['talismanBonusRuneLevel'] {
  return n / 200
}

export function ambrosiaRuneOOMBonusEffect<K extends keyof AmbrosiaUpgradeRewards['ambrosiaRuneOOMBonus']> (
  n: number,
  key: K
): AmbrosiaUpgradeRewards['ambrosiaRuneOOMBonus'][K] {
  if (key === 'runeOOMBonus') {
    return n as AmbrosiaUpgradeRewards['ambrosiaRuneOOMBonus'][K]
  }
  // infiniteAscentOOMBonus
  return (n / 1000) as AmbrosiaUpgradeRewards['ambrosiaRuneOOMBonus'][K]
}

export function ambrosiaBrickOfLeadEffect<K extends keyof AmbrosiaUpgradeRewards['ambrosiaBrickOfLead']> (
  n: number,
  key: K
): AmbrosiaUpgradeRewards['ambrosiaBrickOfLead'][K] {
  if (key === 'barRequirementMult') {
    return (1 / (1 - n / 50)) as AmbrosiaUpgradeRewards['ambrosiaBrickOfLead'][K]
  } else if (key === 'additiveLuckMult') {
    return (n / 50) as AmbrosiaUpgradeRewards['ambrosiaBrickOfLead'][K]
  }
  // singularitySpeedMult
  return (1 - n / 100) as AmbrosiaUpgradeRewards['ambrosiaBrickOfLead'][K]
}

export function ambrosiaFreeLuckUpgradesEffect (
  n: number
): AmbrosiaUpgradeRewards['ambrosiaFreeLuckUpgrades']['freeLuckUpgrades'] {
  return n
}

export function ambrosiaFreeGenerationUpgradesEffect (
  n: number
): AmbrosiaUpgradeRewards['ambrosiaFreeGenerationUpgrades']['freeGenerationUpgrades'] {
  return n
}

export function ambrosiaFreeRedLuckUpgradesEffect (
  n: number
): AmbrosiaUpgradeRewards['ambrosiaFreeRedLuckUpgrades']['freeRedLuckUpgrades'] {
  return n
}

export function ambrosiaFreeQuarkUpgradesEffect (
  n: number
): AmbrosiaUpgradeRewards['ambrosiaFreeQuarkUpgrades']['freeQuarkUpgrades'] {
  return n / 10
}
