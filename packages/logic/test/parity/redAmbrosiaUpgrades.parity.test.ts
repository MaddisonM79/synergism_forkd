// Parity tests for the RedAmbrosiaUpgrades per-upgrade cost/effect formulas,
// lifted from packages/web_ui/src/RedAmbrosiaUpgrades.ts. Sweeps cover:
//   - costFormula: 0..maxLevel grid per upgrade (with baseCost = the
//     legacy costPerLevel value, since that's the only baseCost the data
//     table ever passes in)
//   - effects: every reward key × representative level counts (including
//     the 0/1 boundaries for boolean-returning upgrades and the tax-gated
//     branches for salvageYinYang)

import { describe, expect, it } from 'vitest'
import {
  blueberriesCostFormula as newBlueberriesCost,
  blueberriesEffect as newBlueberriesEffect,
  blueberryGenerationSpeed2CostFormula as newBbGen2Cost,
  blueberryGenerationSpeed2Effect as newBbGen2Effect,
  blueberryGenerationSpeedCostFormula as newBbGenCost,
  blueberryGenerationSpeedEffect as newBbGenEffect,
  conversionImprovement1CostFormula as newConv1Cost,
  conversionImprovement1Effect as newConv1Effect,
  conversionImprovement2CostFormula as newConv2Cost,
  conversionImprovement2Effect as newConv2Effect,
  conversionImprovement3CostFormula as newConv3Cost,
  conversionImprovement3Effect as newConv3Effect,
  freeCubeUpgradesCostFormula as newFreeCubeCost,
  freeCubeUpgradesEffect as newFreeCubeEffect,
  freeLevelsRow2CostFormula as newFreeRow2Cost,
  freeLevelsRow2Effect as newFreeRow2Effect,
  freeLevelsRow3CostFormula as newFreeRow3Cost,
  freeLevelsRow3Effect as newFreeRow3Effect,
  freeLevelsRow4CostFormula as newFreeRow4Cost,
  freeLevelsRow4Effect as newFreeRow4Effect,
  freeLevelsRow5CostFormula as newFreeRow5Cost,
  freeLevelsRow5Effect as newFreeRow5Effect,
  freeObtainiumUpgradesCostFormula as newFreeObtCost,
  freeObtainiumUpgradesEffect as newFreeObtEffect,
  freeOfferingUpgradesCostFormula as newFreeOffCost,
  freeOfferingUpgradesEffect as newFreeOffEffect,
  freeSpeedUpgradesCostFormula as newFreeSpeedCost,
  freeSpeedUpgradesEffect as newFreeSpeedEffect,
  freeTutorialLevelsCostFormula as newFreeTutCost,
  freeTutorialLevelsEffect as newFreeTutEffect,
  infiniteShopUpgradesCostFormula as newInfShopCost,
  infiniteShopUpgradesEffect as newInfShopEffect,
  redAmbrosiaAcceleratorCostFormula as newRaaCost,
  redAmbrosiaAcceleratorEffect as newRaaEffect,
  redAmbrosiaCubeCostFormula as newRacCost,
  redAmbrosiaCubeEffect as newRacEffect,
  redAmbrosiaCubeImproverCostFormula as newRaciCost,
  redAmbrosiaCubeImproverEffect as newRaciEffect,
  redAmbrosiaFreeAccumulatorCostFormula as newRafaCost,
  redAmbrosiaFreeAccumulatorEffect as newRafaEffect,
  redAmbrosiaObtainiumCostFormula as newRaoCost,
  redAmbrosiaObtainiumEffect as newRaoEffect,
  redAmbrosiaOfferingCostFormula as newRaofCost,
  redAmbrosiaOfferingEffect as newRaofEffect,
  redGenerationSpeedCostFormula as newRedGenCost,
  redGenerationSpeedEffect as newRedGenEffect,
  redLuckCostFormula as newRedLuckCost,
  redLuckEffect as newRedLuckEffect,
  regularLuck2CostFormula as newRegLuck2Cost,
  regularLuck2Effect as newRegLuck2Effect,
  regularLuckCostFormula as newRegLuckCost,
  regularLuckEffect as newRegLuckEffect,
  salvageYinYangCostFormula as newSyyCost,
  salvageYinYangEffect as newSyyEffect,
  tutorialCostFormula as newTutCost,
  tutorialEffect as newTutEffect,
  viscountCostFormula as newViscCost,
  viscountEffect as newViscEffect
} from '../../src/mechanics/redAmbrosiaUpgrades'

// ─── Old implementations (verbatim from web_ui RedAmbrosiaUpgrades.ts) ─────

const blueberryCostValues = [100_000, 1_400_000, 3_000_000, 3_250_000, 3_500_000]
const redAmbrosiaFreeAccumulatorValues = [100, 400, 1_000, 3_000, 10_000, 25_000, 75_000, 150_000, 400_000, 1_000_000]
const freeOfferingUpgradesValues = [1_000, 3_000, 9_000, 27_000, 81_000]
const freeObtainiumUpgradesValues = [1_500, 4_500, 13_500, 40_500, 121_500]
const freeCubeUpgradesValues = [10_000, 30_000, 90_000, 270_000, 810_000]
const freeSpeedUpgradesValues = [15_000, 45_000, 135_000, 405_000, 1_215_000]

// Cost formulas (level, baseCost) → number

const oldTutCost = (_level: number, baseCost: number) => baseCost
const oldConv1Cost = (level: number, baseCost: number) => baseCost * Math.pow(2, level)
const oldConv2Cost = (level: number, baseCost: number) => baseCost * Math.pow(4, level)
const oldConv3Cost = (level: number, baseCost: number) => baseCost * Math.pow(10, level)
const oldFreeTutCost = (level: number, baseCost: number) => baseCost + level
const oldFreeRow2Cost = (level: number, baseCost: number) => baseCost * Math.pow(2, level)
const oldFreeRow3Cost = (level: number, baseCost: number) => baseCost * Math.pow(2, level)
const oldFreeRow4Cost = (level: number, baseCost: number) => baseCost * Math.pow(2, level)
const oldFreeRow5Cost = (level: number, baseCost: number) => baseCost * Math.pow(2, level)
const oldBbGenCost = (level: number, baseCost: number) => baseCost * (level + 1)
const oldRegLuckCost = (level: number, baseCost: number) => baseCost * (level + 1)
const oldRedGenCost = (level: number, baseCost: number) => baseCost * (level + 1)
const oldRedLuckCost = (level: number, baseCost: number) => baseCost * (level + 1)
const oldRacCost = (level: number, baseCost: number) => baseCost * (level + 1)
const oldRaoCost = (level: number, baseCost: number) => baseCost * (level + 1)
const oldRaofCost = (level: number, baseCost: number) => baseCost * (level + 1)
const oldRaciCost = (level: number, baseCost: number) => baseCost * (level + 1)
const oldViscCost = (level: number, baseCost: number) => baseCost * (level + 1)
const oldInfShopCost = (level: number, baseCost: number) => baseCost + 100 * level
const oldRaaCost = (_level: number, baseCost: number) => baseCost
const oldRegLuck2Cost = (_level: number, baseCost: number) => baseCost
const oldBbGen2Cost = (_level: number, baseCost: number) => baseCost
const oldSyyCost = (level: number, baseCost: number) => baseCost * (level + 1)
const oldBlueberriesCost = (level: number, _baseCost: number) => blueberryCostValues[level] ?? 0
const oldRafaCost = (level: number, _baseCost: number) => redAmbrosiaFreeAccumulatorValues[level] ?? 0
const oldFreeOffCost = (level: number, _baseCost: number) => freeOfferingUpgradesValues[level] ?? 0
const oldFreeObtCost = (level: number, _baseCost: number) => freeObtainiumUpgradesValues[level] ?? 0
const oldFreeCubeCost = (level: number, _baseCost: number) => freeCubeUpgradesValues[level] ?? 0
const oldFreeSpeedCost = (level: number, _baseCost: number) => freeSpeedUpgradesValues[level] ?? 0

// Effect functions — each returns the matching reward type. Cast through
// number|boolean here since the unit-test side doesn't care about the
// strong type, only the value.

const oldTutEffect = (n: number) => Math.pow(1.01, n)
const oldConv1Effect = (n: number) => -n
const oldConv2Effect = (n: number) => -n
const oldConv3Effect = (n: number) => -n
const oldFreeTutEffect = (n: number) => n
const oldFreeRow2Effect = (n: number) => n
const oldFreeRow3Effect = (n: number) => n
const oldFreeRow4Effect = (n: number) => n
const oldFreeRow5Effect = (n: number) => n
const oldBbGenEffect = (n: number) => 1 + n / 500
const oldRegLuckEffect = (n: number) => 2 * n
const oldRedGenEffect = (n: number) => 1 + 3 * n / 1000
const oldRedLuckEffect = (n: number) => n
const oldRacEffect = (n: number) => n > 0
const oldRaoEffect = (n: number) => n > 0
const oldRaofEffect = (n: number) => n > 0
const oldRaciEffect = (n: number) => 0.01 * n
const oldInfShopEffect = (n: number) => n
const oldRaaEffect = (n: number) => 0.02 * n + 1 * +(n > 0)
const oldRegLuck2Effect = (n: number) => 2 * n
const oldBbGen2Effect = (n: number) => 1 + n / 1000
const oldBlueberriesEffect = (n: number) => n
const oldFreeOffEffect = (n: number) => n
const oldFreeObtEffect = (n: number) => n
const oldFreeCubeEffect = (n: number) => n
const oldFreeSpeedEffect = (n: number) => n

const oldViscEffect = (n: number, key: string): number | boolean => {
  if (key === 'roleUnlock') return n > 0
  if (key === 'quarkBonus') return 1 + 0.1 * n
  if (key === 'luckBonus') return 125 * n
  return 25 * n // redLuckBonus
}

const oldRafaEffect = (n: number, key: string): number => {
  if (key === 'freeAccumulatorLevels') return n / 1000 + 0.01 * +(n > 0)
  return 0.1 * n // freeAccumulatorLevelCapIncrease
}

const oldSyyEffect = (n: number, key: string, taxmanLastStandEnabled: boolean): number => {
  if (key === 'positiveSalvage') {
    if (taxmanLastStandEnabled) return 0
    return 10 * n
  }
  if (taxmanLastStandEnabled) return 0
  return -10 * n
}

const closeEnough = (a: number | boolean, b: number | boolean): boolean => {
  if (a === b) return true
  if (typeof a === 'number' && typeof b === 'number') {
    if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < 1e-12
    return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < 1e-12
  }
  return false
}

// ─── costFormula parity (per upgrade × level grid) ────────────────────────
//
// Tuple: name, costPerLevel (legacy baseCost), maxLevel, new fn, old fn.
// maxLevel is the legacy cap — we sweep 0..maxLevel inclusive.

interface CostCase {
  name: string
  costPerLevel: number
  maxLevel: number
  newFn: (level: number, baseCost: number) => number
  oldFn: (level: number, baseCost: number) => number
}

const costCases: CostCase[] = [
  { name: 'tutorial', costPerLevel: 1, maxLevel: 100, newFn: newTutCost, oldFn: oldTutCost },
  { name: 'conversionImprovement1', costPerLevel: 5, maxLevel: 5, newFn: newConv1Cost, oldFn: oldConv1Cost },
  { name: 'conversionImprovement2', costPerLevel: 200, maxLevel: 3, newFn: newConv2Cost, oldFn: oldConv2Cost },
  { name: 'conversionImprovement3', costPerLevel: 10000, maxLevel: 2, newFn: newConv3Cost, oldFn: oldConv3Cost },
  { name: 'freeTutorialLevels', costPerLevel: 1, maxLevel: 5, newFn: newFreeTutCost, oldFn: oldFreeTutCost },
  { name: 'freeLevelsRow2', costPerLevel: 10, maxLevel: 5, newFn: newFreeRow2Cost, oldFn: oldFreeRow2Cost },
  { name: 'freeLevelsRow3', costPerLevel: 250, maxLevel: 5, newFn: newFreeRow3Cost, oldFn: oldFreeRow3Cost },
  { name: 'freeLevelsRow4', costPerLevel: 5000, maxLevel: 5, newFn: newFreeRow4Cost, oldFn: oldFreeRow4Cost },
  { name: 'freeLevelsRow5', costPerLevel: 50000, maxLevel: 5, newFn: newFreeRow5Cost, oldFn: oldFreeRow5Cost },
  { name: 'blueberryGenerationSpeed', costPerLevel: 1, maxLevel: 100, newFn: newBbGenCost, oldFn: oldBbGenCost },
  { name: 'regularLuck', costPerLevel: 1, maxLevel: 100, newFn: newRegLuckCost, oldFn: oldRegLuckCost },
  { name: 'redGenerationSpeed', costPerLevel: 12, maxLevel: 100, newFn: newRedGenCost, oldFn: oldRedGenCost },
  { name: 'redLuck', costPerLevel: 4, maxLevel: 100, newFn: newRedLuckCost, oldFn: oldRedLuckCost },
  { name: 'redAmbrosiaCube', costPerLevel: 500, maxLevel: 1, newFn: newRacCost, oldFn: oldRacCost },
  { name: 'redAmbrosiaObtainium', costPerLevel: 1250, maxLevel: 1, newFn: newRaoCost, oldFn: oldRaoCost },
  { name: 'redAmbrosiaOffering', costPerLevel: 4000, maxLevel: 1, newFn: newRaofCost, oldFn: oldRaofCost },
  { name: 'redAmbrosiaCubeImprover', costPerLevel: 100, maxLevel: 20, newFn: newRaciCost, oldFn: oldRaciCost },
  { name: 'viscount', costPerLevel: 99999, maxLevel: 1, newFn: newViscCost, oldFn: oldViscCost },
  { name: 'infiniteShopUpgrades', costPerLevel: 200, maxLevel: 40, newFn: newInfShopCost, oldFn: oldInfShopCost },
  { name: 'redAmbrosiaAccelerator', costPerLevel: 1000, maxLevel: 100, newFn: newRaaCost, oldFn: oldRaaCost },
  { name: 'regularLuck2', costPerLevel: 8000, maxLevel: 250, newFn: newRegLuck2Cost, oldFn: oldRegLuck2Cost },
  { name: 'blueberryGenerationSpeed2', costPerLevel: 8000, maxLevel: 250, newFn: newBbGen2Cost, oldFn: oldBbGen2Cost },
  { name: 'salvageYinYang', costPerLevel: 200, maxLevel: 100, newFn: newSyyCost, oldFn: oldSyyCost },
  { name: 'blueberries', costPerLevel: 1e5, maxLevel: 5, newFn: newBlueberriesCost, oldFn: oldBlueberriesCost },
  { name: 'redAmbrosiaFreeAccumulator', costPerLevel: 1, maxLevel: 10, newFn: newRafaCost, oldFn: oldRafaCost },
  { name: 'freeOfferingUpgrades', costPerLevel: 1, maxLevel: 5, newFn: newFreeOffCost, oldFn: oldFreeOffCost },
  { name: 'freeObtainiumUpgrades', costPerLevel: 1, maxLevel: 5, newFn: newFreeObtCost, oldFn: oldFreeObtCost },
  { name: 'freeCubeUpgrades', costPerLevel: 1, maxLevel: 5, newFn: newFreeCubeCost, oldFn: oldFreeCubeCost },
  { name: 'freeSpeedUpgrades', costPerLevel: 1, maxLevel: 5, newFn: newFreeSpeedCost, oldFn: oldFreeSpeedCost }
]

// Sample at: 0, 1, every-25%, max, max+1 (to verify level array-lookup
// guard with ?? 0 also matches). Capped at 16 distinct values per case
// to keep the test count reasonable while covering every formula shape.
const sampleLevelsFor = (maxLevel: number): number[] => {
  const grid = new Set<number>([0, 1, maxLevel, maxLevel + 1])
  if (maxLevel >= 4) {
    grid.add(Math.floor(maxLevel / 4))
    grid.add(Math.floor(maxLevel / 2))
    grid.add(Math.floor(3 * maxLevel / 4))
  }
  return Array.from(grid).sort((a, b) => a - b)
}

describe('parity: redAmbrosia costFormula (all 29 upgrades)', () => {
  for (const c of costCases) {
    for (const level of sampleLevelsFor(c.maxLevel)) {
      it(`${c.name} level=${level}`, () => {
        const next = c.newFn(level, c.costPerLevel)
        const old = c.oldFn(level, c.costPerLevel)
        expect(closeEnough(next, old)).toBe(true)
      })
    }
  }
})

// ─── effect parity (per upgrade × per-reward-key) ─────────────────────────

const levelGrid = [0, 1, 5, 10, 15, 50, 100, 250]

describe('parity: redAmbrosia effects — single-key arrow upgrades', () => {
  const singleKeyCases: {
    name: string
    newFn: (n: number) => number | boolean
    oldFn: (n: number) => number | boolean
  }[] = [
    { name: 'tutorial', newFn: (n) => newTutEffect(n), oldFn: oldTutEffect },
    { name: 'conversionImprovement1', newFn: newConv1Effect, oldFn: oldConv1Effect },
    { name: 'conversionImprovement2', newFn: newConv2Effect, oldFn: oldConv2Effect },
    { name: 'conversionImprovement3', newFn: newConv3Effect, oldFn: oldConv3Effect },
    { name: 'freeTutorialLevels', newFn: newFreeTutEffect, oldFn: oldFreeTutEffect },
    { name: 'freeLevelsRow2', newFn: newFreeRow2Effect, oldFn: oldFreeRow2Effect },
    { name: 'freeLevelsRow3', newFn: newFreeRow3Effect, oldFn: oldFreeRow3Effect },
    { name: 'freeLevelsRow4', newFn: newFreeRow4Effect, oldFn: oldFreeRow4Effect },
    { name: 'freeLevelsRow5', newFn: newFreeRow5Effect, oldFn: oldFreeRow5Effect },
    { name: 'blueberryGenerationSpeed', newFn: newBbGenEffect, oldFn: oldBbGenEffect },
    { name: 'regularLuck', newFn: newRegLuckEffect, oldFn: oldRegLuckEffect },
    { name: 'redGenerationSpeed', newFn: newRedGenEffect, oldFn: oldRedGenEffect },
    { name: 'redLuck', newFn: newRedLuckEffect, oldFn: oldRedLuckEffect },
    { name: 'redAmbrosiaCube', newFn: newRacEffect, oldFn: oldRacEffect },
    { name: 'redAmbrosiaObtainium', newFn: newRaoEffect, oldFn: oldRaoEffect },
    { name: 'redAmbrosiaOffering', newFn: newRaofEffect, oldFn: oldRaofEffect },
    { name: 'redAmbrosiaCubeImprover', newFn: newRaciEffect, oldFn: oldRaciEffect },
    { name: 'infiniteShopUpgrades', newFn: newInfShopEffect, oldFn: oldInfShopEffect },
    { name: 'redAmbrosiaAccelerator', newFn: newRaaEffect, oldFn: oldRaaEffect },
    { name: 'regularLuck2', newFn: newRegLuck2Effect, oldFn: oldRegLuck2Effect },
    { name: 'blueberryGenerationSpeed2', newFn: newBbGen2Effect, oldFn: oldBbGen2Effect },
    { name: 'blueberries', newFn: newBlueberriesEffect, oldFn: oldBlueberriesEffect },
    { name: 'freeOfferingUpgrades', newFn: newFreeOffEffect, oldFn: oldFreeOffEffect },
    { name: 'freeObtainiumUpgrades', newFn: newFreeObtEffect, oldFn: oldFreeObtEffect },
    { name: 'freeCubeUpgrades', newFn: newFreeCubeEffect, oldFn: oldFreeCubeEffect },
    { name: 'freeSpeedUpgrades', newFn: newFreeSpeedEffect, oldFn: oldFreeSpeedEffect }
  ]
  for (const c of singleKeyCases) {
    for (const n of levelGrid) {
      it(`${c.name} n=${n}`, () => {
        expect(closeEnough(c.newFn(n), c.oldFn(n))).toBe(true)
      })
    }
  }
})

describe('parity: viscountEffect', () => {
  const keys = ['roleUnlock', 'quarkBonus', 'luckBonus', 'redLuckBonus'] as const
  for (const key of keys) {
    for (const n of levelGrid) {
      it(`key=${key} n=${n}`, () => {
        const next = newViscEffect(n, key)
        const old = oldViscEffect(n, key)
        expect(closeEnough(next, old)).toBe(true)
      })
    }
  }
})

describe('parity: redAmbrosiaFreeAccumulatorEffect', () => {
  const keys = ['freeAccumulatorLevels', 'freeAccumulatorLevelCapIncrease'] as const
  for (const key of keys) {
    for (const n of levelGrid) {
      it(`key=${key} n=${n}`, () => {
        const next = newRafaEffect(n, key)
        const old = oldRafaEffect(n, key)
        expect(closeEnough(next, old)).toBe(true)
      })
    }
  }
})

// salvageYinYang has the taxmanLastStand gate — sweep both gate states.
describe('parity: salvageYinYangEffect (gated by taxmanLastStand)', () => {
  const keys = ['positiveSalvage', 'negativeSalvage'] as const
  for (const taxmanLastStandEnabled of [false, true]) {
    for (const key of keys) {
      for (const n of levelGrid) {
        it(`key=${key} n=${n} taxman=${taxmanLastStandEnabled}`, () => {
          const next = newSyyEffect(n, key, taxmanLastStandEnabled)
          const old = oldSyyEffect(n, key, taxmanLastStandEnabled)
          expect(closeEnough(next, old)).toBe(true)
        })
      }
    }
  }
})
