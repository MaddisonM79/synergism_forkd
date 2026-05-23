// Parity tests for the Octeracts per-upgrade cost/effect formulas, lifted
// from packages/web_ui/src/Octeracts.ts. Sweeps cover:
//   - costFormula: representative level grid per upgrade (with baseCost set
//     to the legacy costPerLevel value)
//   - effects: every reward key × representative level counts, with impure
//     upgrades swept across their extra player-input axes (singularityCount,
//     ascensionCount, sibling quarkGain level, hepteract quark BAL)

import { describe, expect, it } from 'vitest'
import {
  octeractAmbrosiaGeneration2CostFormula as newAmbGen2Cost,
  octeractAmbrosiaGeneration2Effect as newAmbGen2Effect,
  octeractAmbrosiaGeneration3CostFormula as newAmbGen3Cost,
  octeractAmbrosiaGeneration3Effect as newAmbGen3Effect,
  octeractAmbrosiaGeneration4CostFormula as newAmbGen4Cost,
  octeractAmbrosiaGeneration4Effect as newAmbGen4Effect,
  octeractAmbrosiaGenerationCostFormula as newAmbGenCost,
  octeractAmbrosiaGenerationEffect as newAmbGenEffect,
  octeractAmbrosiaLuck2CostFormula as newAmbLuck2Cost,
  octeractAmbrosiaLuck2Effect as newAmbLuck2Effect,
  octeractAmbrosiaLuck3CostFormula as newAmbLuck3Cost,
  octeractAmbrosiaLuck3Effect as newAmbLuck3Effect,
  octeractAmbrosiaLuck4CostFormula as newAmbLuck4Cost,
  octeractAmbrosiaLuck4Effect as newAmbLuck4Effect,
  octeractAmbrosiaLuckCostFormula as newAmbLuckCost,
  octeractAmbrosiaLuckEffect as newAmbLuckEffect,
  octeractAscensions2CostFormula as newAscensions2Cost,
  octeractAscensions2Effect as newAscensions2Effect,
  octeractAscensionsCostFormula as newAscensionsCost,
  octeractAscensionsEffect as newAscensionsEffect,
  octeractAscensionsOcteractGainCostFormula as newAscOctGainCost,
  octeractAscensionsOcteractGainEffect as newAscOctGainEffect,
  octeractAutoPotionEfficiencyCostFormula as newAutoPotionEffCost,
  octeractAutoPotionEfficiencyEffect as newAutoPotionEffEffect,
  octeractAutoPotionSpeedCostFormula as newAutoPotionSpeedCost,
  octeractAutoPotionSpeedEffect as newAutoPotionSpeedEffect,
  octeractBlueberriesCostFormula as newBlueberriesCost,
  octeractBlueberriesEffect as newBlueberriesEffect,
  octeractBonusTokens1CostFormula as newBonusTokens1Cost,
  octeractBonusTokens1Effect as newBonusTokens1Effect,
  octeractBonusTokens2CostFormula as newBonusTokens2Cost,
  octeractBonusTokens2Effect as newBonusTokens2Effect,
  octeractBonusTokens3CostFormula as newBonusTokens3Cost,
  octeractBonusTokens3Effect as newBonusTokens3Effect,
  octeractBonusTokens4CostFormula as newBonusTokens4Cost,
  octeractBonusTokens4Effect as newBonusTokens4Effect,
  octeractCorruptionCostFormula as newCorruptionCost,
  octeractCorruptionEffect as newCorruptionEffect,
  octeractExportQuarksCostFormula as newExportQuarksCost,
  octeractExportQuarksEffect as newExportQuarksEffect,
  octeractFastForwardCostFormula as newFastForwardCost,
  octeractFastForwardEffect as newFastForwardEffect,
  octeractGain2CostFormula as newGain2Cost,
  octeractGain2Effect as newGain2Effect,
  octeractGainCostFormula as newGainCost,
  octeractGainEffect as newGainEffect,
  octeractGQCostReduceCostFormula as newGQCostReduceCost,
  octeractGQCostReduceEffect as newGQCostReduceEffect,
  octeractImprovedAscensionSpeed2CostFormula as newImprAscSpeed2Cost,
  octeractImprovedAscensionSpeed2Effect as newImprAscSpeed2Effect,
  octeractImprovedAscensionSpeedCostFormula as newImprAscSpeedCost,
  octeractImprovedAscensionSpeedEffect as newImprAscSpeedEffect,
  octeractImprovedDaily2CostFormula as newImprDaily2Cost,
  octeractImprovedDaily2Effect as newImprDaily2Effect,
  octeractImprovedDaily3CostFormula as newImprDaily3Cost,
  octeractImprovedDaily3Effect as newImprDaily3Effect,
  octeractImprovedDailyCostFormula as newImprDailyCost,
  octeractImprovedDailyEffect as newImprDailyEffect,
  octeractImprovedFree2CostFormula as newImprFree2Cost,
  octeractImprovedFree2Effect as newImprFree2Effect,
  octeractImprovedFree3CostFormula as newImprFree3Cost,
  octeractImprovedFree3Effect as newImprFree3Effect,
  octeractImprovedFree4CostFormula as newImprFree4Cost,
  octeractImprovedFree4Effect as newImprFree4Effect,
  octeractImprovedFreeCostFormula as newImprFreeCost,
  octeractImprovedFreeEffect as newImprFreeEffect,
  octeractImprovedGlobalSpeedCostFormula as newImprGlobalSpeedCost,
  octeractImprovedGlobalSpeedEffect as newImprGlobalSpeedEffect,
  octeractImprovedQuarkHeptCostFormula as newImprQuarkHeptCost,
  octeractImprovedQuarkHeptEffect as newImprQuarkHeptEffect,
  octeractInfiniteShopUpgradesCostFormula as newInfShopCost,
  octeractInfiniteShopUpgradesEffect as newInfShopEffect,
  octeractObtainium1CostFormula as newObtainium1Cost,
  octeractObtainium1Effect as newObtainium1Effect,
  octeractOfferings1CostFormula as newOfferings1Cost,
  octeractOfferings1Effect as newOfferings1Effect,
  octeractOneMindImproverCostFormula as newOneMindCost,
  octeractOneMindImproverEffect as newOneMindEffect,
  octeractQuarkGain2CostFormula as newQuarkGain2Cost,
  octeractQuarkGain2Effect as newQuarkGain2Effect,
  octeractQuarkGainCostFormula as newQuarkGainCost,
  octeractQuarkGainEffect as newQuarkGainEffect,
  octeractSingUpgradeCapCostFormula as newSingUpgradeCapCost,
  octeractSingUpgradeCapEffect as newSingUpgradeCapEffect,
  octeractStarterCostFormula as newStarterCost,
  octeractStarterEffect as newStarterEffect,
  octeractTalismanLevelCap1CostFormula as newTalismanCap1Cost,
  octeractTalismanLevelCap1Effect as newTalismanCap1Effect,
  octeractTalismanLevelCap2CostFormula as newTalismanCap2Cost,
  octeractTalismanLevelCap2Effect as newTalismanCap2Effect,
  octeractTalismanLevelCap3CostFormula as newTalismanCap3Cost,
  octeractTalismanLevelCap3Effect as newTalismanCap3Effect,
  octeractTalismanLevelCap4CostFormula as newTalismanCap4Cost,
  octeractTalismanLevelCap4Effect as newTalismanCap4Effect
} from '../../src/mechanics/octeracts'

// ─── Old implementations (verbatim from web_ui Octeracts.ts) ──────────────

const octeractBlueberryCostArr = [1, 1e3, 1e9, 1e27, 1e81, 1e111]

// Cost formulas

const oldStarterCost = (level: number, baseCost: number) => baseCost * (level + 1)
const oldGainCost = (level: number, baseCost: number) => baseCost * (Math.pow(level + 1, 6) - Math.pow(level, 6))
const oldGain2Cost = (level: number, baseCost: number) => baseCost * Math.pow(10, Math.pow(level, 0.5) / 3)
const oldQuarkGainCost = (level: number, baseCost: number) => {
  if (level < 1000) {
    return baseCost * (Math.pow(level + 1, 7) - Math.pow(level, 7))
  }
  const fasterMult = (level >= 10000) ? Math.pow(10, (level - 10000) / 250) : 1
  const fasterMult2 = (level >= 15000) ? Math.pow(10, (level - 15000) / 250) : 1
  return baseCost * (Math.pow(1001, 7) - Math.pow(1000, 7)) * Math.pow(10, level / 1000) * fasterMult * fasterMult2
}
const oldQuarkGain2Cost = (level: number, baseCost: number) => baseCost * Math.pow(1e20, level)
const oldCorruptionCost = (level: number, baseCost: number) => baseCost * Math.pow(10, level * 10)
const oldGQCostReduceCost = (level: number, baseCost: number) => baseCost * Math.pow(2, level)
const oldExportQuarksCost = (level: number, baseCost: number) => baseCost * Math.pow(level + 1, 3)
const oldImprDailyCost = (level: number, baseCost: number) => baseCost * Math.pow(1.6, level)
const oldImprDaily2Cost = (level: number, baseCost: number) => baseCost * Math.pow(2, level)
const oldImprDaily3Cost = (level: number, baseCost: number) => baseCost * Math.pow(20, level)
const oldImprQuarkHeptCost = (level: number, baseCost: number) => baseCost * Math.pow(1e3, level)
const oldImprGlobalSpeedCost = (level: number, baseCost: number) => baseCost * Math.pow(level + 1, 3)
const oldImprAscSpeedCost = (level: number, baseCost: number) => baseCost * Math.pow(1e9, level / 100)
const oldImprAscSpeed2Cost = (level: number, baseCost: number) => baseCost * Math.pow(1e12, level / 250)
const oldImprFreeCost = (level: number, baseCost: number) => baseCost * Math.pow(level + 1, 3)
const oldImprFree2Cost = (level: number, baseCost: number) => baseCost * Math.pow(level + 1, 3)
const oldImprFree3Cost = (level: number, baseCost: number) => baseCost * Math.pow(level + 1, 3)
const oldImprFree4Cost = (level: number, baseCost: number) => baseCost * Math.pow(1e20, level / 40)
const oldSingUpgradeCapCost = (level: number, baseCost: number) => baseCost * Math.pow(1e3, level)
const oldOfferings1Cost = (level: number, baseCost: number) => {
  if (level < 25) return baseCost * Math.pow(level + 1, 5)
  return baseCost * 1e15 * Math.pow(10, level / 25 - 1)
}
const oldObtainium1Cost = oldOfferings1Cost
const oldAscensionsCost = (level: number, baseCost: number) => baseCost * Math.pow(level + 1, 3)
const oldAscensions2Cost = (level: number, baseCost: number) => baseCost * Math.pow(10, Math.pow(level, 0.5) / 3)
const oldAscOctGainCost = (level: number, baseCost: number) => baseCost * Math.pow(40, level)
const oldFastForwardCost = (level: number, baseCost: number) => baseCost * Math.pow(1e8, level)
const oldAutoPotionSpeedCost = (level: number, baseCost: number) => baseCost * Math.pow(10, level)
const oldAutoPotionEffCost = (level: number, baseCost: number) => baseCost * Math.pow(10, level)
const oldOneMindCost = (level: number, baseCost: number) => {
  const fasterMult = (level >= 10) ? Math.pow(1e3, level - 10) : 1
  return baseCost * Math.pow(1e5, level) * fasterMult
}
const oldAmbLuckCost = (level: number, baseCost: number) => {
  const useLevel = level + 1
  return baseCost * (Math.pow(10, useLevel) - Math.pow(10, useLevel - 1))
}
const oldAmbLuck2Cost = (level: number, baseCost: number) => baseCost * (Math.pow(level + 1, 6) - Math.pow(level, 6))
const oldAmbLuck3Cost = (level: number, baseCost: number) => baseCost * (Math.pow(level + 1, 8) - Math.pow(level, 8))
const oldAmbLuck4Cost = (level: number, baseCost: number) => {
  const useLevel = level + 1
  return baseCost * (Math.pow(3, useLevel) - Math.pow(3, useLevel - 1))
}
const oldAmbGenCost = oldAmbLuckCost
const oldAmbGen2Cost = oldAmbLuck2Cost
const oldAmbGen3Cost = oldAmbLuck3Cost
const oldAmbGen4Cost = oldAmbLuck4Cost
const oldBonusTokens1Cost = (level: number, baseCost: number) => baseCost * Math.pow(1e2, level)
const oldBonusTokens2Cost = (level: number, baseCost: number) => baseCost * Math.pow(1e8, level)
const oldBonusTokens3Cost = (level: number, baseCost: number) => baseCost * Math.pow(1e10, level)
const oldBonusTokens4Cost = (level: number, baseCost: number) => baseCost * Math.pow(4, level)
const oldBlueberriesCost = (level: number, _baseCost: number) => {
  if (level === 6) return 0
  return octeractBlueberryCostArr[level]
}
const oldInfShopCost = (level: number, baseCost: number) => baseCost * Math.pow(16, level)
const oldTalismanCap1Cost = (level: number, baseCost: number) => baseCost * Math.pow(level + 1, 5)
const oldTalismanCap2Cost = (level: number, baseCost: number) => baseCost * Math.pow(level + 1, 10)
const oldTalismanCap3Cost = (level: number, baseCost: number) => baseCost * Math.pow(level + 1, 20)
const oldTalismanCap4Cost = (level: number, baseCost: number) => baseCost * Math.pow(10, level)

// Effects

const oldStarterEffect = (n: number, key: string): number => {
  if (key === 'quarkMult') return 1 + 0.25 * n
  if (key === 'antSpeedMult') return 1 + 99999 * n
  return 1 + 0.4 * n
}
const oldGainEffect = (n: number) => 1 + 0.01 * n
const oldGain2Effect = (n: number) => 1 + 0.01 * n
const oldQuarkGainEffect = (n: number) => 1 + 0.011 * n
const oldQuarkGain2Effect = (n: number, quarkGainLevel: number, hepteractQuarkBAL: number) =>
  1 + (1 / 10000) * Math.floor(quarkGainLevel / 111) * n
    * Math.floor(1 + Math.log10(Math.max(1, hepteractQuarkBAL)))
const oldCorruptionEffect = (n: number) => n
const oldGQCostReduceEffect = (n: number) => 1 - n / 100
const oldExportQuarksEffect = (n: number) => 4 * n / 10 + 1
const oldImprDailyEffect = (n: number) => n
const oldImprDaily2Effect = (n: number) => 1 + 0.01 * n
const oldImprDaily3Effect = (n: number, key: string): number => {
  if (key === 'goldenQuarkMult') return 1 + 0.005 * n
  return n
}
const oldImprQuarkHeptEffect = (n: number) => n / 100
const oldImprGlobalSpeedEffect = (n: number, singularityCount: number) => 1 + n * singularityCount / 100
const oldImprAscSpeedEffect = (n: number, singularityCount: number) => 1 + n * singularityCount / 2000
const oldImprAscSpeed2Effect = (n: number, singularityCount: number) => 1 + n * singularityCount / 2000
const oldImprFreeEffect = (n: number, key: string): number | boolean => {
  if (key === 'unlocked') return n > 0
  return 0.6 * n
}
const oldImprFree2Effect = (n: number) => 0.05 * n
const oldImprFree3Effect = (n: number) => 0.05 * n
const oldImprFree4Effect = (n: number) => 0.001 * n + ((n > 0) ? 0.01 : 0)
const oldSingUpgradeCapEffect = (n: number) => n
const oldOfferings1Effect = (n: number) => 1 + 0.01 * n
const oldObtainium1Effect = (n: number) => 1 + 0.01 * n
const oldAscensionsEffect = (n: number) => (1 + n / 100) * (1 + 2 * Math.floor(n / 10) / 100)
const oldAscensions2Effect = oldAscensionsEffect
const oldAscOctGainEffect = (n: number, ascensionCount: number) =>
  Math.pow(1 + n / 100, 1 + Math.floor(Math.log10(1 + ascensionCount)))
const oldFastForwardEffect = (n: number) => n
const oldAutoPotionSpeedEffect = (n: number) => 1 + 4 * n / 100
const oldAutoPotionEffEffect = (n: number) => 1 + 2 * n / 100
const oldOneMindEffect = (n: number) => 0.55 + n / 150
const oldAmbLuckEffect = (n: number) => 4 * n
const oldAmbLuck2Effect = (n: number) => 2 * n
const oldAmbLuck3Effect = (n: number) => 3 * n
const oldAmbLuck4Effect = (n: number) => 5 * n
const oldAmbGenEffect = (n: number) => 1 + n / 100
const oldAmbGen2Effect = (n: number) => 1 + n / 100
const oldAmbGen3Effect = (n: number) => 1 + n / 100
const oldAmbGen4Effect = (n: number) => 1 + 2 * n / 100
const oldBonusTokens1Effect = (n: number) => n
const oldBonusTokens2Effect = (n: number) => 1 + n / 100
const oldBonusTokens3Effect = (n: number) => n
const oldBonusTokens4Effect = (n: number) => 2 * n
const oldBlueberriesEffect = (n: number) => n
const oldInfShopEffect = (n: number) => n
const oldTalismanCap1Effect = (n: number) => n
const oldTalismanCap2Effect = (n: number) => n
const oldTalismanCap3Effect = (n: number) => n
const oldTalismanCap4Effect = (n: number) => n

const closeEnough = (a: number | boolean, b: number | boolean): boolean => {
  if (a === b) return true
  if (typeof a === 'number' && typeof b === 'number') {
    if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
    if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < 1e-12
    return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < 1e-10
  }
  return false
}

// ─── costFormula parity ──────────────────────────────────────────────────

interface CostCase {
  name: string
  costPerLevel: number
  levels: number[]
  newFn: (level: number, baseCost: number) => number
  oldFn: (level: number, baseCost: number) => number
}

// Hand-pick representative levels per upgrade. For uncapped ones we sample
// a sparse high-level grid. For QuarkGain we include both knees (1000,
// 10000, 15000).
const costCases: CostCase[] = [
  { name: 'octeractStarter', costPerLevel: 1e-15, levels: [0, 1], newFn: newStarterCost, oldFn: oldStarterCost },
  {
    name: 'octeractGain',
    costPerLevel: 1e-8,
    levels: [0, 1, 100, 1000, 1e6, 1e8],
    newFn: newGainCost,
    oldFn: oldGainCost
  },
  { name: 'octeractGain2', costPerLevel: 1e10, levels: [0, 1, 100, 1000], newFn: newGain2Cost, oldFn: oldGain2Cost },
  {
    name: 'octeractQuarkGain',
    costPerLevel: 1e-7,
    levels: [0, 1, 100, 999, 1000, 1001, 5000, 9999, 10000, 10001, 14999, 15000, 15001, 20000],
    newFn: newQuarkGainCost,
    oldFn: oldQuarkGainCost
  },
  {
    name: 'octeractQuarkGain2',
    costPerLevel: 1e22,
    levels: [0, 1, 5],
    newFn: newQuarkGain2Cost,
    oldFn: oldQuarkGain2Cost
  },
  {
    name: 'octeractCorruption',
    costPerLevel: 10,
    levels: [0, 1, 2],
    newFn: newCorruptionCost,
    oldFn: oldCorruptionCost
  },
  {
    name: 'octeractGQCostReduce',
    costPerLevel: 1e-9,
    levels: [0, 1, 25, 50],
    newFn: newGQCostReduceCost,
    oldFn: oldGQCostReduceCost
  },
  {
    name: 'octeractExportQuarks',
    costPerLevel: 1,
    levels: [0, 1, 50, 100],
    newFn: newExportQuarksCost,
    oldFn: oldExportQuarksCost
  },
  {
    name: 'octeractImprovedDaily',
    costPerLevel: 1e-3,
    levels: [0, 1, 25, 50],
    newFn: newImprDailyCost,
    oldFn: oldImprDailyCost
  },
  {
    name: 'octeractImprovedDaily2',
    costPerLevel: 1e-2,
    levels: [0, 1, 25, 50],
    newFn: newImprDaily2Cost,
    oldFn: oldImprDaily2Cost
  },
  {
    name: 'octeractImprovedDaily3',
    costPerLevel: 1e20,
    levels: [0, 1, 5],
    newFn: newImprDaily3Cost,
    oldFn: oldImprDaily3Cost
  },
  {
    name: 'octeractImprovedQuarkHept',
    costPerLevel: 0.1,
    levels: [0, 1, 12, 25],
    newFn: newImprQuarkHeptCost,
    oldFn: oldImprQuarkHeptCost
  },
  {
    name: 'octeractImprovedGlobalSpeed',
    costPerLevel: 1e-5,
    levels: [0, 1, 500, 1000],
    newFn: newImprGlobalSpeedCost,
    oldFn: oldImprGlobalSpeedCost
  },
  {
    name: 'octeractImprovedAscensionSpeed',
    costPerLevel: 100,
    levels: [0, 1, 50, 100],
    newFn: newImprAscSpeedCost,
    oldFn: oldImprAscSpeedCost
  },
  {
    name: 'octeractImprovedAscensionSpeed2',
    costPerLevel: 1e5,
    levels: [0, 1, 125, 250],
    newFn: newImprAscSpeed2Cost,
    oldFn: oldImprAscSpeed2Cost
  },
  { name: 'octeractImprovedFree', costPerLevel: 100, levels: [0, 1], newFn: newImprFreeCost, oldFn: oldImprFreeCost },
  {
    name: 'octeractImprovedFree2',
    costPerLevel: 1e7,
    levels: [0, 1],
    newFn: newImprFree2Cost,
    oldFn: oldImprFree2Cost
  },
  {
    name: 'octeractImprovedFree3',
    costPerLevel: 1e17,
    levels: [0, 1],
    newFn: newImprFree3Cost,
    oldFn: oldImprFree3Cost
  },
  {
    name: 'octeractImprovedFree4',
    costPerLevel: 1e20,
    levels: [0, 1, 20, 40],
    newFn: newImprFree4Cost,
    oldFn: oldImprFree4Cost
  },
  {
    name: 'octeractSingUpgradeCap',
    costPerLevel: 1e10,
    levels: [0, 1, 5, 10],
    newFn: newSingUpgradeCapCost,
    oldFn: oldSingUpgradeCapCost
  },
  {
    name: 'octeractOfferings1',
    costPerLevel: 1e-15,
    levels: [0, 1, 24, 25, 26, 50, 100],
    newFn: newOfferings1Cost,
    oldFn: oldOfferings1Cost
  },
  {
    name: 'octeractObtainium1',
    costPerLevel: 1e-15,
    levels: [0, 1, 24, 25, 26, 50, 100],
    newFn: newObtainium1Cost,
    oldFn: oldObtainium1Cost
  },
  {
    name: 'octeractAscensions',
    costPerLevel: 1,
    levels: [0, 1, 1000, 1e5, 1e6],
    newFn: newAscensionsCost,
    oldFn: oldAscensionsCost
  },
  {
    name: 'octeractAscensions2',
    costPerLevel: 1e12,
    levels: [0, 1, 100, 1000],
    newFn: newAscensions2Cost,
    oldFn: oldAscensions2Cost
  },
  {
    name: 'octeractAscensionsOcteractGain',
    costPerLevel: 1000,
    levels: [0, 1, 5, 10],
    newFn: newAscOctGainCost,
    oldFn: oldAscOctGainCost
  },
  {
    name: 'octeractFastForward',
    costPerLevel: 1e8,
    levels: [0, 1, 2],
    newFn: newFastForwardCost,
    oldFn: oldFastForwardCost
  },
  {
    name: 'octeractAutoPotionSpeed',
    costPerLevel: 1e-10,
    levels: [0, 1, 10, 100],
    newFn: newAutoPotionSpeedCost,
    oldFn: oldAutoPotionSpeedCost
  },
  {
    name: 'octeractAutoPotionEfficiency',
    costPerLevel: 1e-10 * Math.pow(10, 0.5),
    levels: [0, 1, 50, 100],
    newFn: newAutoPotionEffCost,
    oldFn: oldAutoPotionEffCost
  },
  {
    name: 'octeractOneMindImprover',
    costPerLevel: 1e25,
    levels: [0, 1, 9, 10, 11, 20],
    newFn: newOneMindCost,
    oldFn: oldOneMindCost
  },
  {
    name: 'octeractAmbrosiaLuck',
    costPerLevel: 1e60 / 9,
    levels: [0, 1, 5],
    newFn: newAmbLuckCost,
    oldFn: oldAmbLuckCost
  },
  {
    name: 'octeractAmbrosiaLuck2',
    costPerLevel: 1,
    levels: [0, 1, 15, 30],
    newFn: newAmbLuck2Cost,
    oldFn: oldAmbLuck2Cost
  },
  {
    name: 'octeractAmbrosiaLuck3',
    costPerLevel: 1e30,
    levels: [0, 1, 15, 30],
    newFn: newAmbLuck3Cost,
    oldFn: oldAmbLuck3Cost
  },
  {
    name: 'octeractAmbrosiaLuck4',
    costPerLevel: 1e70 / 2,
    levels: [0, 1, 25, 50],
    newFn: newAmbLuck4Cost,
    oldFn: oldAmbLuck4Cost
  },
  {
    name: 'octeractAmbrosiaGeneration',
    costPerLevel: 1e60 / 9,
    levels: [0, 1, 5],
    newFn: newAmbGenCost,
    oldFn: oldAmbGenCost
  },
  {
    name: 'octeractAmbrosiaGeneration2',
    costPerLevel: 1,
    levels: [0, 1, 10, 20],
    newFn: newAmbGen2Cost,
    oldFn: oldAmbGen2Cost
  },
  {
    name: 'octeractAmbrosiaGeneration3',
    costPerLevel: 1e30,
    levels: [0, 1, 17, 35],
    newFn: newAmbGen3Cost,
    oldFn: oldAmbGen3Cost
  },
  {
    name: 'octeractAmbrosiaGeneration4',
    costPerLevel: 1e70 / 2,
    levels: [0, 1, 25, 50],
    newFn: newAmbGen4Cost,
    oldFn: oldAmbGen4Cost
  },
  {
    name: 'octeractBonusTokens1',
    costPerLevel: 1e-5,
    levels: [0, 1, 5, 10],
    newFn: newBonusTokens1Cost,
    oldFn: oldBonusTokens1Cost
  },
  {
    name: 'octeractBonusTokens2',
    costPerLevel: 1e8,
    levels: [0, 1, 5],
    newFn: newBonusTokens2Cost,
    oldFn: oldBonusTokens2Cost
  },
  {
    name: 'octeractBonusTokens3',
    costPerLevel: 1e40,
    levels: [0, 1, 5],
    newFn: newBonusTokens3Cost,
    oldFn: oldBonusTokens3Cost
  },
  {
    name: 'octeractBonusTokens4',
    costPerLevel: 1e75,
    levels: [0, 1, 25, 50],
    newFn: newBonusTokens4Cost,
    oldFn: oldBonusTokens4Cost
  },
  {
    name: 'octeractBlueberries',
    costPerLevel: 1,
    levels: [0, 1, 2, 3, 4, 5, 6, 7],
    newFn: newBlueberriesCost,
    oldFn: oldBlueberriesCost
  },
  {
    name: 'octeractInfiniteShopUpgrades',
    costPerLevel: 1e30,
    levels: [0, 1, 40, 80],
    newFn: newInfShopCost,
    oldFn: oldInfShopCost
  },
  {
    name: 'octeractTalismanLevelCap1',
    costPerLevel: 1e-5,
    levels: [0, 1, 12, 25],
    newFn: newTalismanCap1Cost,
    oldFn: oldTalismanCap1Cost
  },
  {
    name: 'octeractTalismanLevelCap2',
    costPerLevel: 1e10,
    levels: [0, 1, 17, 35],
    newFn: newTalismanCap2Cost,
    oldFn: oldTalismanCap2Cost
  },
  {
    name: 'octeractTalismanLevelCap3',
    costPerLevel: 1e20,
    levels: [0, 1, 20, 40],
    newFn: newTalismanCap3Cost,
    oldFn: oldTalismanCap3Cost
  },
  {
    name: 'octeractTalismanLevelCap4',
    costPerLevel: 1e40,
    levels: [0, 1, 10, 50],
    newFn: newTalismanCap4Cost,
    oldFn: oldTalismanCap4Cost
  }
]

describe('parity: octeract costFormula (all 46 upgrades)', () => {
  for (const c of costCases) {
    for (const level of c.levels) {
      it(`${c.name} level=${level}`, () => {
        const next = c.newFn(level, c.costPerLevel)
        const old = c.oldFn(level, c.costPerLevel)
        expect(closeEnough(next, old)).toBe(true)
      })
    }
  }
})

// ─── effects parity (pure upgrades) ────────────────────────────────────────

const levelGrid = [0, 1, 5, 10, 25, 50, 100, 250, 1000]

describe('parity: octeract effects — pure single-key upgrades', () => {
  const cases: { name: string; new: (n: number) => number | boolean; old: (n: number) => number | boolean }[] = [
    { name: 'octeractGain', new: newGainEffect, old: oldGainEffect },
    { name: 'octeractGain2', new: newGain2Effect, old: oldGain2Effect },
    { name: 'octeractQuarkGain', new: newQuarkGainEffect, old: oldQuarkGainEffect },
    { name: 'octeractCorruption', new: newCorruptionEffect, old: oldCorruptionEffect },
    { name: 'octeractGQCostReduce', new: newGQCostReduceEffect, old: oldGQCostReduceEffect },
    { name: 'octeractExportQuarks', new: newExportQuarksEffect, old: oldExportQuarksEffect },
    { name: 'octeractImprovedDaily', new: newImprDailyEffect, old: oldImprDailyEffect },
    { name: 'octeractImprovedDaily2', new: newImprDaily2Effect, old: oldImprDaily2Effect },
    { name: 'octeractImprovedQuarkHept', new: newImprQuarkHeptEffect, old: oldImprQuarkHeptEffect },
    { name: 'octeractImprovedFree2', new: newImprFree2Effect, old: oldImprFree2Effect },
    { name: 'octeractImprovedFree3', new: newImprFree3Effect, old: oldImprFree3Effect },
    { name: 'octeractImprovedFree4', new: newImprFree4Effect, old: oldImprFree4Effect },
    { name: 'octeractSingUpgradeCap', new: newSingUpgradeCapEffect, old: oldSingUpgradeCapEffect },
    { name: 'octeractOfferings1', new: newOfferings1Effect, old: oldOfferings1Effect },
    { name: 'octeractObtainium1', new: newObtainium1Effect, old: oldObtainium1Effect },
    { name: 'octeractAscensions', new: newAscensionsEffect, old: oldAscensionsEffect },
    { name: 'octeractAscensions2', new: newAscensions2Effect, old: oldAscensions2Effect },
    { name: 'octeractFastForward', new: newFastForwardEffect, old: oldFastForwardEffect },
    { name: 'octeractAutoPotionSpeed', new: newAutoPotionSpeedEffect, old: oldAutoPotionSpeedEffect },
    { name: 'octeractAutoPotionEfficiency', new: newAutoPotionEffEffect, old: oldAutoPotionEffEffect },
    { name: 'octeractOneMindImprover', new: newOneMindEffect, old: oldOneMindEffect },
    { name: 'octeractAmbrosiaLuck', new: newAmbLuckEffect, old: oldAmbLuckEffect },
    { name: 'octeractAmbrosiaLuck2', new: newAmbLuck2Effect, old: oldAmbLuck2Effect },
    { name: 'octeractAmbrosiaLuck3', new: newAmbLuck3Effect, old: oldAmbLuck3Effect },
    { name: 'octeractAmbrosiaLuck4', new: newAmbLuck4Effect, old: oldAmbLuck4Effect },
    { name: 'octeractAmbrosiaGeneration', new: newAmbGenEffect, old: oldAmbGenEffect },
    { name: 'octeractAmbrosiaGeneration2', new: newAmbGen2Effect, old: oldAmbGen2Effect },
    { name: 'octeractAmbrosiaGeneration3', new: newAmbGen3Effect, old: oldAmbGen3Effect },
    { name: 'octeractAmbrosiaGeneration4', new: newAmbGen4Effect, old: oldAmbGen4Effect },
    { name: 'octeractBonusTokens1', new: newBonusTokens1Effect, old: oldBonusTokens1Effect },
    { name: 'octeractBonusTokens2', new: newBonusTokens2Effect, old: oldBonusTokens2Effect },
    { name: 'octeractBonusTokens3', new: newBonusTokens3Effect, old: oldBonusTokens3Effect },
    { name: 'octeractBonusTokens4', new: newBonusTokens4Effect, old: oldBonusTokens4Effect },
    { name: 'octeractBlueberries', new: newBlueberriesEffect, old: oldBlueberriesEffect },
    { name: 'octeractInfiniteShopUpgrades', new: newInfShopEffect, old: oldInfShopEffect },
    { name: 'octeractTalismanLevelCap1', new: newTalismanCap1Effect, old: oldTalismanCap1Effect },
    { name: 'octeractTalismanLevelCap2', new: newTalismanCap2Effect, old: oldTalismanCap2Effect },
    { name: 'octeractTalismanLevelCap3', new: newTalismanCap3Effect, old: oldTalismanCap3Effect },
    { name: 'octeractTalismanLevelCap4', new: newTalismanCap4Effect, old: oldTalismanCap4Effect }
  ]
  for (const c of cases) {
    for (const n of levelGrid) {
      it(`${c.name} n=${n}`, () => {
        expect(closeEnough(c.new(n), c.old(n))).toBe(true)
      })
    }
  }
})

describe('parity: octeractStarterEffect', () => {
  const keys = ['quarkMult', 'antSpeedMult', 'octeractMult'] as const
  for (const key of keys) {
    for (const n of [0, 1]) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(newStarterEffect(n, key), oldStarterEffect(n, key))).toBe(true)
      })
    }
  }
})

describe('parity: octeractImprovedDaily3Effect', () => {
  const keys = ['extraGoldenQuarks', 'goldenQuarkMult'] as const
  for (const key of keys) {
    for (const n of levelGrid) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(newImprDaily3Effect(n, key), oldImprDaily3Effect(n, key))).toBe(true)
      })
    }
  }
})

describe('parity: octeractImprovedFreeEffect', () => {
  const keys = ['unlocked', 'freeLevelPower'] as const
  for (const key of keys) {
    for (const n of [0, 1]) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(newImprFreeEffect(n, key), oldImprFreeEffect(n, key))).toBe(true)
      })
    }
  }
})

// ─── effects parity (impure upgrades — extra player-input axis) ────────────

const singularityCountGrid = [0, 1, 10, 100, 1000]
const ascensionCountGrid = [0, 1, 1e3, 1e6, 1e12, 1e18]
const quarkGainLevelGrid = [0, 50, 111, 222, 555, 1110, 20000]
const hepteractQuarkBALGrid = [0, 1, 100, 1e6, 1e12, 1e20]

describe('parity: octeractQuarkGain2Effect (n × quarkGainLevel × hepteractQuarkBAL)', () => {
  for (const n of [0, 1, 5]) {
    for (const qLevel of quarkGainLevelGrid) {
      for (const bal of hepteractQuarkBALGrid) {
        it(`n=${n} qLevel=${qLevel} bal=${bal}`, () => {
          expect(closeEnough(
            newQuarkGain2Effect(n, qLevel, bal),
            oldQuarkGain2Effect(n, qLevel, bal)
          )).toBe(true)
        })
      }
    }
  }
})

describe('parity: octeractImprovedGlobalSpeedEffect (n × singularityCount)', () => {
  for (const n of levelGrid) {
    for (const sc of singularityCountGrid) {
      it(`n=${n} sc=${sc}`, () => {
        expect(closeEnough(newImprGlobalSpeedEffect(n, sc), oldImprGlobalSpeedEffect(n, sc))).toBe(true)
      })
    }
  }
})

describe('parity: octeractImprovedAscensionSpeedEffect (n × singularityCount)', () => {
  for (const n of [0, 1, 50, 100]) {
    for (const sc of singularityCountGrid) {
      it(`n=${n} sc=${sc}`, () => {
        expect(closeEnough(newImprAscSpeedEffect(n, sc), oldImprAscSpeedEffect(n, sc))).toBe(true)
      })
    }
  }
})

describe('parity: octeractImprovedAscensionSpeed2Effect (n × singularityCount)', () => {
  for (const n of [0, 1, 125, 250]) {
    for (const sc of singularityCountGrid) {
      it(`n=${n} sc=${sc}`, () => {
        expect(closeEnough(newImprAscSpeed2Effect(n, sc), oldImprAscSpeed2Effect(n, sc))).toBe(true)
      })
    }
  }
})

describe('parity: octeractAscensionsOcteractGainEffect (n × ascensionCount)', () => {
  for (const n of [0, 1, 5, 10]) {
    for (const ac of ascensionCountGrid) {
      it(`n=${n} ac=${ac}`, () => {
        expect(closeEnough(newAscOctGainEffect(n, ac), oldAscOctGainEffect(n, ac))).toBe(true)
      })
    }
  }
})
