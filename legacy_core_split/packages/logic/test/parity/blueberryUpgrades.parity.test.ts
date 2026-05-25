// Parity tests for the BlueberryUpgrades per-upgrade cost/effect formulas,
// lifted from packages/web_ui/src/BlueberryUpgrades.ts. Sweeps cover:
//   - costFormula: 0..maxLevel grid per upgrade (with baseCost = the legacy
//     costPerLevel value, since that's what the data-table passes in)
//   - effects: every reward key × representative level counts, with the
//     impure upgrades swept across multiple player-input values (worlds,
//     wowCubeLogSum, ambrosiaLuck, lifetime totals, gate booleans, etc.)

import { describe, expect, it } from 'vitest'
import {
  ambrosiaBaseObtainium1CostFormula as newBaseObt1Cost,
  ambrosiaBaseObtainium1Effect as newBaseObt1Effect,
  ambrosiaBaseObtainium2CostFormula as newBaseObt2Cost,
  ambrosiaBaseObtainium2Effect as newBaseObt2Effect,
  ambrosiaBaseOffering1CostFormula as newBaseOff1Cost,
  ambrosiaBaseOffering1Effect as newBaseOff1Effect,
  ambrosiaBaseOffering2CostFormula as newBaseOff2Cost,
  ambrosiaBaseOffering2Effect as newBaseOff2Effect,
  ambrosiaBrickOfLeadCostFormula as newBrickCost,
  ambrosiaBrickOfLeadEffect as newBrickEffect,
  ambrosiaCubeLuck1CostFormula as newCubeLuck1Cost,
  ambrosiaCubeLuck1Effect as newCubeLuck1Effect,
  ambrosiaCubeQuark1CostFormula as newCubeQuark1Cost,
  ambrosiaCubeQuark1Effect as newCubeQuark1Effect,
  ambrosiaCubes1CostFormula as newCubes1Cost,
  ambrosiaCubes1Effect as newCubes1Effect,
  ambrosiaCubes2CostFormula as newCubes2Cost,
  ambrosiaCubes2Effect as newCubes2Effect,
  ambrosiaCubes3CostFormula as newCubes3Cost,
  ambrosiaCubes3Effect as newCubes3Effect,
  ambrosiaFreeGenerationUpgradesCostFormula as newFreeGenCost,
  ambrosiaFreeGenerationUpgradesEffect as newFreeGenEffect,
  ambrosiaFreeLuckUpgradesCostFormula as newFreeLuckCost,
  ambrosiaFreeLuckUpgradesEffect as newFreeLuckEffect,
  ambrosiaFreeQuarkUpgradesCostFormula as newFreeQuarkCost,
  ambrosiaFreeQuarkUpgradesEffect as newFreeQuarkEffect,
  ambrosiaFreeRedLuckUpgradesCostFormula as newFreeRedLuckCost,
  ambrosiaFreeRedLuckUpgradesEffect as newFreeRedLuckEffect,
  ambrosiaHyperfluxCostFormula as newHyperfluxCost,
  ambrosiaHyperfluxEffect as newHyperfluxEffect,
  ambrosiaInfiniteShopUpgrades1CostFormula as newInfShop1Cost,
  ambrosiaInfiniteShopUpgrades1Effect as newInfShop1Effect,
  ambrosiaInfiniteShopUpgrades2CostFormula as newInfShop2Cost,
  ambrosiaInfiniteShopUpgrades2Effect as newInfShop2Effect,
  ambrosiaLuck1CostFormula as newLuck1Cost,
  ambrosiaLuck1Effect as newLuck1Effect,
  ambrosiaLuck2CostFormula as newLuck2Cost,
  ambrosiaLuck2Effect as newLuck2Effect,
  ambrosiaLuck3CostFormula as newLuck3Cost,
  ambrosiaLuck3Effect as newLuck3Effect,
  ambrosiaLuck4CostFormula as newLuck4Cost,
  ambrosiaLuck4Effect as newLuck4Effect,
  ambrosiaLuckCube1CostFormula as newLuckCube1Cost,
  ambrosiaLuckCube1Effect as newLuckCube1Effect,
  ambrosiaLuckQuark1CostFormula as newLuckQuark1Cost,
  ambrosiaLuckQuark1Effect as newLuckQuark1Effect,
  ambrosiaObtainium1CostFormula as newObt1Cost,
  ambrosiaObtainium1Effect as newObt1Effect,
  ambrosiaOffering1CostFormula as newOff1Cost,
  ambrosiaOffering1Effect as newOff1Effect,
  ambrosiaPatreonCostFormula as newPatreonCost,
  ambrosiaPatreonEffect as newPatreonEffect,
  ambrosiaQuarkCube1CostFormula as newQuarkCube1Cost,
  ambrosiaQuarkCube1Effect as newQuarkCube1Effect,
  ambrosiaQuarkLuck1CostFormula as newQuarkLuck1Cost,
  ambrosiaQuarkLuck1Effect as newQuarkLuck1Effect,
  ambrosiaQuarks1CostFormula as newQuarks1Cost,
  ambrosiaQuarks1Effect as newQuarks1Effect,
  ambrosiaQuarks2CostFormula as newQuarks2Cost,
  ambrosiaQuarks2Effect as newQuarks2Effect,
  ambrosiaQuarks3CostFormula as newQuarks3Cost,
  ambrosiaQuarks3Effect as newQuarks3Effect,
  ambrosiaRuneOOMBonusCostFormula as newRuneOOMCost,
  ambrosiaRuneOOMBonusEffect as newRuneOOMEffect,
  ambrosiaSingReduction1CostFormula as newSingRed1Cost,
  ambrosiaSingReduction1Effect as newSingRed1Effect,
  ambrosiaSingReduction2CostFormula as newSingRed2Cost,
  ambrosiaSingReduction2Effect as newSingRed2Effect,
  ambrosiaTalismanBonusRuneLevelCostFormula as newTalismanCost,
  ambrosiaTalismanBonusRuneLevelEffect as newTalismanEffect,
  ambrosiaTutorialCostFormula as newTutCost,
  ambrosiaTutorialEffect as newTutEffect
} from '../../src/mechanics/blueberryUpgrades'

// ─── Old implementations (verbatim from web_ui BlueberryUpgrades.ts) ───────

// Pre-name a couple of repeating shapes that the legacy file inlines.
const oldCubic = (level: number, baseCost: number): number => baseCost * (Math.pow(level + 1, 3) - Math.pow(level, 3))
const oldQuad = (level: number, baseCost: number): number => baseCost * (Math.pow(level + 1, 2) - Math.pow(level, 2))

const oldTutCost = oldQuad
const oldQuarks1Cost = oldCubic
const oldCubes1Cost = oldCubic
const oldLuck1Cost = oldCubic
const oldQuarkCube1Cost = oldCubic
const oldLuckCube1Cost = oldCubic
const oldCubeQuark1Cost = oldCubic
const oldLuckQuark1Cost = oldCubic
const oldCubeLuck1Cost = oldCubic
const oldQuarkLuck1Cost = oldCubic
const oldQuarks2Cost = oldQuad
const oldCubes2Cost = oldQuad
const oldLuck2Cost = oldQuad
const oldQuarks3Cost = (level: number, baseCost: number) => baseCost + 50000 * level
const oldCubes3Cost = (level: number, baseCost: number) => baseCost + 5000 * level
const oldLuck3Cost = (_level: number, baseCost: number) => baseCost
const oldLuck4Cost = (level: number, baseCost: number) => baseCost + 20000 * level
const oldPatreonCost = oldQuad
const oldObt1Cost = (level: number, baseCost: number) => baseCost * Math.pow(25, level)
const oldOff1Cost = (level: number, baseCost: number) => baseCost * Math.pow(25, level)
const oldHyperfluxCost = (level: number, baseCost: number) =>
  (baseCost + 33333 * Math.min(4, level)) * Math.max(1, Math.pow(3, level - 4))
const oldBaseOff1Cost = oldCubic
const oldBaseObt1Cost = oldCubic
const oldBaseOff2Cost = oldCubic
const oldBaseObt2Cost = oldCubic
const oldSingRed1Cost = (level: number, baseCost: number) => baseCost * Math.pow(99, level)
const oldInfShop1Cost = (_level: number, baseCost: number) => baseCost
const oldInfShop2Cost = (_level: number, baseCost: number) => baseCost
const oldSingRed2Cost = (level: number, baseCost: number) => baseCost * Math.pow(3, level)
const oldTalismanCost = oldQuad
const oldRuneOOMCost = (level: number, baseCost: number) =>
  Math.ceil(baseCost * (Math.pow(level + 1, 1.5) - Math.pow(level, 1.5)))
const oldBrickCost = oldCubic
const oldFreeLuckCost = oldQuad
const oldFreeGenCost = (level: number, baseCost: number) => baseCost * (Math.pow(10, level + 1) - Math.pow(10, level))
const oldFreeRedLuckCost = oldQuad
const oldFreeQuarkCost = oldCubic

// Effects — verbatim

const oldTutEffect = (n: number, key: string): number => {
  if (key === 'cubes') return 1 + 0.05 * n
  return 1 + 0.01 * n
}
const oldQuarks1Effect = (n: number) => 1 + 0.01 * n
const oldCubes1Effect = (n: number) => (1 + 0.05 * n) * Math.pow(1.1, Math.floor(n / 5))
const oldLuck1Effect = (n: number) => 2 * n + 12 * Math.floor(n / 10)
const oldQuarkCube1Effect = (n: number, worlds: number) => {
  const baseVal = 0.001 * n
  return 1 + baseVal * Math.floor(Math.pow(Math.log10(worlds + 1) + 1, 2))
}
const oldLuckCube1Effect = (n: number, luck: number) => 1 + 0.0005 * n * luck
const oldCubeQuark1Effect = (n: number, wowSum: number) => 1 + 0.0001 * n * (wowSum + 6)
const oldLuckQuark1Effect = (n: number, luck: number) => {
  const baseVal = 0.0001 * n
  const eff = Math.min(luck, Math.pow(1000, 0.5) * Math.pow(luck, 0.5))
  return 1 + baseVal * eff
}
const oldCubeLuck1Effect = (n: number, wowSum: number) => 0.02 * n * (wowSum + 6)
const oldQuarkLuck1Effect = (n: number, worlds: number) =>
  0.02 * n * Math.floor(Math.pow(Math.log10(worlds + 1) + 1, 2))
const oldQuarks2Effect = (n: number, q1Eff: number) => 1 + (0.01 + Math.floor(q1Eff / 10) / 1000) * n
const oldCubes2Effect = (n: number, c1Eff: number) =>
  (1 + (0.1 + 10 * (Math.floor(c1Eff / 10) / 1000)) * n) * Math.pow(1.15, Math.floor(n / 5))
const oldLuck2Effect = (n: number, l1Eff: number) => (3 + 0.3 * Math.floor(l1Eff / 10)) * n + 40 * Math.floor(n / 10)
const oldQuarks3Effect = (n: number, q2Eff: number) => {
  const q2mult = 1 + q2Eff / 100
  return 1 + 0.05 * n * q2mult
}
const oldCubes3Effect = (n: number, c2Eff: number) => {
  const c2mult = 1 + 3 * c2Eff / 100
  return (1 + 0.2 * n * c2mult) * Math.pow(1.2, Math.floor(n / 5))
}
const oldLuck3Effect = (n: number, inventory: number) => inventory * n
const oldLuck4Effect = (n: number, lifetimeRedAmbrosia: number, lifetimeAmbrosia: number) => {
  const digits = Math.ceil(Math.log10(lifetimeRedAmbrosia + 1))
    + Math.ceil(Math.log10(lifetimeAmbrosia + 1))
  return digits * n / 10000
}
const oldPatreonEffect = (n: number, qBonus: number) => 1 + (n * qBonus) / 100
const oldObt1Effect = (n: number, luck: number) => 1 + n * luck / 1000
const oldOff1Effect = (n: number, luck: number) => 1 + n * luck / 1000
const oldHyperfluxEffect = (n: number, p19: number) => Math.pow(1 + n / 100, p19)
const oldBaseOff1Effect = (n: number) => n
const oldBaseObt1Effect = (n: number) => n
const oldBaseOff2Effect = (n: number) => n
const oldBaseObt2Effect = (n: number) => n
const oldSingRed1Effect = (n: number, inside: boolean) => inside ? 0 : n
const oldInfShop1Effect = (n: number) => n
const oldInfShop2Effect = (n: number) => n
const oldSingRed2Effect = (n: number, inside: boolean) => inside ? n : 0
const oldTalismanEffect = (n: number) => n / 200
const oldRuneOOMEffect = (n: number, key: string): number => {
  if (key === 'runeOOMBonus') return n
  return n / 1000
}
const oldBrickEffect = (n: number, key: string): number => {
  if (key === 'barRequirementMult') return 1 / (1 - n / 50)
  if (key === 'additiveLuckMult') return n / 50
  return 1 - n / 100
}
const oldFreeLuckEffect = (n: number) => n
const oldFreeGenEffect = (n: number) => n
const oldFreeRedLuckEffect = (n: number) => n
const oldFreeQuarkEffect = (n: number) => n / 10

const closeEnough = (a: number | boolean, b: number | boolean): boolean => {
  if (a === b) return true
  if (typeof a === 'number' && typeof b === 'number') {
    if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
    if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < 1e-12
    return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < 1e-12
  }
  return false
}

// ─── costFormula parity ───────────────────────────────────────────────────

interface CostCase {
  name: string
  costPerLevel: number
  maxLevel: number
  newFn: (level: number, baseCost: number) => number
  oldFn: (level: number, baseCost: number) => number
}

const costCases: CostCase[] = [
  { name: 'ambrosiaTutorial', costPerLevel: 1, maxLevel: 10, newFn: newTutCost, oldFn: oldTutCost },
  { name: 'ambrosiaQuarks1', costPerLevel: 1, maxLevel: 100, newFn: newQuarks1Cost, oldFn: oldQuarks1Cost },
  { name: 'ambrosiaCubes1', costPerLevel: 1, maxLevel: 100, newFn: newCubes1Cost, oldFn: oldCubes1Cost },
  { name: 'ambrosiaLuck1', costPerLevel: 1, maxLevel: 100, newFn: newLuck1Cost, oldFn: oldLuck1Cost },
  { name: 'ambrosiaQuarkCube1', costPerLevel: 250, maxLevel: 25, newFn: newQuarkCube1Cost, oldFn: oldQuarkCube1Cost },
  { name: 'ambrosiaLuckCube1', costPerLevel: 250, maxLevel: 25, newFn: newLuckCube1Cost, oldFn: oldLuckCube1Cost },
  { name: 'ambrosiaCubeQuark1', costPerLevel: 500, maxLevel: 25, newFn: newCubeQuark1Cost, oldFn: oldCubeQuark1Cost },
  { name: 'ambrosiaLuckQuark1', costPerLevel: 500, maxLevel: 25, newFn: newLuckQuark1Cost, oldFn: oldLuckQuark1Cost },
  { name: 'ambrosiaCubeLuck1', costPerLevel: 100, maxLevel: 25, newFn: newCubeLuck1Cost, oldFn: oldCubeLuck1Cost },
  { name: 'ambrosiaQuarkLuck1', costPerLevel: 100, maxLevel: 25, newFn: newQuarkLuck1Cost, oldFn: oldQuarkLuck1Cost },
  { name: 'ambrosiaQuarks2', costPerLevel: 500, maxLevel: 100, newFn: newQuarks2Cost, oldFn: oldQuarks2Cost },
  { name: 'ambrosiaCubes2', costPerLevel: 500, maxLevel: 100, newFn: newCubes2Cost, oldFn: oldCubes2Cost },
  { name: 'ambrosiaLuck2', costPerLevel: 250, maxLevel: 100, newFn: newLuck2Cost, oldFn: oldLuck2Cost },
  { name: 'ambrosiaQuarks3', costPerLevel: 750000, maxLevel: 10, newFn: newQuarks3Cost, oldFn: oldQuarks3Cost },
  { name: 'ambrosiaCubes3', costPerLevel: 75000, maxLevel: 100, newFn: newCubes3Cost, oldFn: oldCubes3Cost },
  { name: 'ambrosiaLuck3', costPerLevel: 50000, maxLevel: 100, newFn: newLuck3Cost, oldFn: oldLuck3Cost },
  { name: 'ambrosiaLuck4', costPerLevel: 250000, maxLevel: 50, newFn: newLuck4Cost, oldFn: oldLuck4Cost },
  { name: 'ambrosiaPatreon', costPerLevel: 1, maxLevel: 1, newFn: newPatreonCost, oldFn: oldPatreonCost },
  { name: 'ambrosiaObtainium1', costPerLevel: 50000, maxLevel: 2, newFn: newObt1Cost, oldFn: oldObt1Cost },
  { name: 'ambrosiaOffering1', costPerLevel: 50000, maxLevel: 2, newFn: newOff1Cost, oldFn: oldOff1Cost },
  { name: 'ambrosiaHyperflux', costPerLevel: 33333, maxLevel: 7, newFn: newHyperfluxCost, oldFn: oldHyperfluxCost },
  { name: 'ambrosiaBaseOffering1', costPerLevel: 5, maxLevel: 40, newFn: newBaseOff1Cost, oldFn: oldBaseOff1Cost },
  { name: 'ambrosiaBaseObtainium1', costPerLevel: 40, maxLevel: 20, newFn: newBaseObt1Cost, oldFn: oldBaseObt1Cost },
  { name: 'ambrosiaBaseOffering2', costPerLevel: 20, maxLevel: 60, newFn: newBaseOff2Cost, oldFn: oldBaseOff2Cost },
  { name: 'ambrosiaBaseObtainium2', costPerLevel: 160, maxLevel: 30, newFn: newBaseObt2Cost, oldFn: oldBaseObt2Cost },
  { name: 'ambrosiaSingReduction1', costPerLevel: 100000, maxLevel: 2, newFn: newSingRed1Cost, oldFn: oldSingRed1Cost },
  {
    name: 'ambrosiaInfiniteShopUpgrades1',
    costPerLevel: 25000,
    maxLevel: 20,
    newFn: newInfShop1Cost,
    oldFn: oldInfShop1Cost
  },
  {
    name: 'ambrosiaInfiniteShopUpgrades2',
    costPerLevel: 75000,
    maxLevel: 20,
    newFn: newInfShop2Cost,
    oldFn: oldInfShop2Cost
  },
  { name: 'ambrosiaSingReduction2', costPerLevel: 1.25e7, maxLevel: 2, newFn: newSingRed2Cost, oldFn: oldSingRed2Cost },
  {
    name: 'ambrosiaTalismanBonusRuneLevel',
    costPerLevel: 100,
    maxLevel: 100,
    newFn: newTalismanCost,
    oldFn: oldTalismanCost
  },
  { name: 'ambrosiaRuneOOMBonus', costPerLevel: 2500, maxLevel: 100, newFn: newRuneOOMCost, oldFn: oldRuneOOMCost },
  { name: 'ambrosiaBrickOfLead', costPerLevel: 10, maxLevel: 25, newFn: newBrickCost, oldFn: oldBrickCost },
  {
    name: 'ambrosiaFreeLuckUpgrades',
    costPerLevel: 5000,
    maxLevel: 25,
    newFn: newFreeLuckCost,
    oldFn: oldFreeLuckCost
  },
  {
    name: 'ambrosiaFreeGenerationUpgrades',
    costPerLevel: 5000,
    maxLevel: 3,
    newFn: newFreeGenCost,
    oldFn: oldFreeGenCost
  },
  {
    name: 'ambrosiaFreeRedLuckUpgrades',
    costPerLevel: 10000,
    maxLevel: 40,
    newFn: newFreeRedLuckCost,
    oldFn: oldFreeRedLuckCost
  },
  {
    name: 'ambrosiaFreeQuarkUpgrades',
    costPerLevel: 25000,
    maxLevel: 10,
    newFn: newFreeQuarkCost,
    oldFn: oldFreeQuarkCost
  }
]

const sampleLevelsFor = (maxLevel: number): number[] => {
  const grid = new Set<number>([0, 1, maxLevel, maxLevel + 1])
  if (maxLevel >= 4) {
    grid.add(Math.floor(maxLevel / 4))
    grid.add(Math.floor(maxLevel / 2))
    grid.add(Math.floor(3 * maxLevel / 4))
  }
  return Array.from(grid).sort((a, b) => a - b)
}

describe('parity: blueberry costFormula (all 36 upgrades)', () => {
  for (const c of costCases) {
    for (const level of sampleLevelsFor(c.maxLevel)) {
      it(`${c.name} level=${level}`, () => {
        expect(closeEnough(c.newFn(level, c.costPerLevel), c.oldFn(level, c.costPerLevel))).toBe(true)
      })
    }
  }
})

// ─── effects parity (per-upgrade) ─────────────────────────────────────────

const levelGrid = [0, 1, 2, 5, 10, 25, 50, 100]
const worldsGrid = [0, 1, 100, 1e6, 1e10, 1e15]
const luckGrid = [0, 50, 500, 5000, 50_000]
const wowSumGrid = [0, 1, 5, 10, 20, 50]
const inventoryGrid = [0, 1, 3, 5, 10]
const lifetimeGrid = [0, 1, 1e6, 1e12, 1e18]
const quarkBonusGrid = [0, 1, 10, 50, 100]
const effLevelGrid = [0, 5, 10, 30, 100]
const platonic19Grid = [0, 1, 10, 100]

describe('parity: blueberry effects — pure-shape upgrades', () => {
  const pureSingleKey: { name: string; new: (n: number) => number; old: (n: number) => number }[] = [
    { name: 'ambrosiaQuarks1', new: newQuarks1Effect, old: oldQuarks1Effect },
    { name: 'ambrosiaCubes1', new: newCubes1Effect, old: oldCubes1Effect },
    { name: 'ambrosiaLuck1', new: newLuck1Effect, old: oldLuck1Effect },
    { name: 'ambrosiaBaseOffering1', new: newBaseOff1Effect, old: oldBaseOff1Effect },
    { name: 'ambrosiaBaseObtainium1', new: newBaseObt1Effect, old: oldBaseObt1Effect },
    { name: 'ambrosiaBaseOffering2', new: newBaseOff2Effect, old: oldBaseOff2Effect },
    { name: 'ambrosiaBaseObtainium2', new: newBaseObt2Effect, old: oldBaseObt2Effect },
    { name: 'ambrosiaInfiniteShopUpgrades1', new: newInfShop1Effect, old: oldInfShop1Effect },
    { name: 'ambrosiaInfiniteShopUpgrades2', new: newInfShop2Effect, old: oldInfShop2Effect },
    { name: 'ambrosiaTalismanBonusRuneLevel', new: newTalismanEffect, old: oldTalismanEffect },
    { name: 'ambrosiaFreeLuckUpgrades', new: newFreeLuckEffect, old: oldFreeLuckEffect },
    { name: 'ambrosiaFreeGenerationUpgrades', new: newFreeGenEffect, old: oldFreeGenEffect },
    { name: 'ambrosiaFreeRedLuckUpgrades', new: newFreeRedLuckEffect, old: oldFreeRedLuckEffect },
    { name: 'ambrosiaFreeQuarkUpgrades', new: newFreeQuarkEffect, old: oldFreeQuarkEffect }
  ]
  for (const c of pureSingleKey) {
    for (const n of levelGrid) {
      it(`${c.name} n=${n}`, () => {
        expect(closeEnough(c.new(n), c.old(n))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaTutorialEffect', () => {
  const keys = ['cubes', 'quarks'] as const
  for (const key of keys) {
    for (const n of levelGrid) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(newTutEffect(n, key), oldTutEffect(n, key))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaRuneOOMBonusEffect', () => {
  const keys = ['runeOOMBonus', 'infiniteAscentOOMBonus'] as const
  for (const key of keys) {
    for (const n of levelGrid) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(newRuneOOMEffect(n, key), oldRuneOOMEffect(n, key))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaBrickOfLeadEffect', () => {
  const keys = ['barRequirementMult', 'additiveLuckMult', 'singularitySpeedMult'] as const
  // Bar requirement explodes at n=50, so cap our grid at 49 for that one.
  for (const key of keys) {
    for (const n of [0, 1, 2, 5, 10, 20, 25]) {
      it(`key=${key} n=${n}`, () => {
        expect(closeEnough(newBrickEffect(n, key), oldBrickEffect(n, key))).toBe(true)
      })
    }
  }
})

// ─── Impure effects: sweep across one extra player-derived axis ───────────

describe('parity: ambrosiaQuarkCube1Effect (n × worlds)', () => {
  for (const n of levelGrid) {
    for (const w of worldsGrid) {
      it(`n=${n} worlds=${w}`, () => {
        expect(closeEnough(newQuarkCube1Effect(n, w), oldQuarkCube1Effect(n, w))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaLuckCube1Effect (n × luck)', () => {
  for (const n of levelGrid) {
    for (const l of luckGrid) {
      it(`n=${n} luck=${l}`, () => {
        expect(closeEnough(newLuckCube1Effect(n, l), oldLuckCube1Effect(n, l))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaCubeQuark1Effect (n × wowSum)', () => {
  for (const n of levelGrid) {
    for (const w of wowSumGrid) {
      it(`n=${n} wowSum=${w}`, () => {
        expect(closeEnough(newCubeQuark1Effect(n, w), oldCubeQuark1Effect(n, w))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaLuckQuark1Effect (n × luck)', () => {
  for (const n of levelGrid) {
    for (const l of luckGrid) {
      it(`n=${n} luck=${l}`, () => {
        expect(closeEnough(newLuckQuark1Effect(n, l), oldLuckQuark1Effect(n, l))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaCubeLuck1Effect (n × wowSum)', () => {
  for (const n of levelGrid) {
    for (const w of wowSumGrid) {
      it(`n=${n} wowSum=${w}`, () => {
        expect(closeEnough(newCubeLuck1Effect(n, w), oldCubeLuck1Effect(n, w))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaQuarkLuck1Effect (n × worlds)', () => {
  for (const n of levelGrid) {
    for (const w of worldsGrid) {
      it(`n=${n} worlds=${w}`, () => {
        expect(closeEnough(newQuarkLuck1Effect(n, w), oldQuarkLuck1Effect(n, w))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaQuarks2Effect (n × quarks1EffLevels)', () => {
  for (const n of levelGrid) {
    for (const e of effLevelGrid) {
      it(`n=${n} q1Eff=${e}`, () => {
        expect(closeEnough(newQuarks2Effect(n, e), oldQuarks2Effect(n, e))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaCubes2Effect (n × cubes1EffLevels)', () => {
  for (const n of levelGrid) {
    for (const e of effLevelGrid) {
      it(`n=${n} c1Eff=${e}`, () => {
        expect(closeEnough(newCubes2Effect(n, e), oldCubes2Effect(n, e))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaLuck2Effect (n × luck1EffLevels)', () => {
  for (const n of levelGrid) {
    for (const e of effLevelGrid) {
      it(`n=${n} l1Eff=${e}`, () => {
        expect(closeEnough(newLuck2Effect(n, e), oldLuck2Effect(n, e))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaQuarks3Effect (n × quarks2EffLevels)', () => {
  for (const n of [0, 1, 5, 10]) {
    for (const e of effLevelGrid) {
      it(`n=${n} q2Eff=${e}`, () => {
        expect(closeEnough(newQuarks3Effect(n, e), oldQuarks3Effect(n, e))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaCubes3Effect (n × cubes2EffLevels)', () => {
  for (const n of levelGrid) {
    for (const e of effLevelGrid) {
      it(`n=${n} c2Eff=${e}`, () => {
        expect(closeEnough(newCubes3Effect(n, e), oldCubes3Effect(n, e))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaLuck3Effect (n × inventory)', () => {
  for (const n of levelGrid) {
    for (const inv of inventoryGrid) {
      it(`n=${n} inv=${inv}`, () => {
        expect(closeEnough(newLuck3Effect(n, inv), oldLuck3Effect(n, inv))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaLuck4Effect (n × lifetimeRedAmbrosia × lifetimeAmbrosia)', () => {
  for (const n of [0, 1, 25, 50]) {
    for (const lr of lifetimeGrid) {
      for (const la of lifetimeGrid) {
        it(`n=${n} lifetimeRed=${lr} lifetimeAmb=${la}`, () => {
          expect(closeEnough(newLuck4Effect(n, lr, la), oldLuck4Effect(n, lr, la))).toBe(true)
        })
      }
    }
  }
})

describe('parity: ambrosiaPatreonEffect (n × quarkBonus)', () => {
  for (const n of [0, 1]) {
    for (const q of quarkBonusGrid) {
      it(`n=${n} qBonus=${q}`, () => {
        expect(closeEnough(newPatreonEffect(n, q), oldPatreonEffect(n, q))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaObtainium1Effect (n × luck)', () => {
  for (const n of [0, 1, 2]) {
    for (const l of luckGrid) {
      it(`n=${n} luck=${l}`, () => {
        expect(closeEnough(newObt1Effect(n, l), oldObt1Effect(n, l))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaOffering1Effect (n × luck)', () => {
  for (const n of [0, 1, 2]) {
    for (const l of luckGrid) {
      it(`n=${n} luck=${l}`, () => {
        expect(closeEnough(newOff1Effect(n, l), oldOff1Effect(n, l))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaHyperfluxEffect (n × platonicUpgrades[19])', () => {
  for (const n of [0, 1, 3, 7]) {
    for (const p of platonic19Grid) {
      it(`n=${n} p19=${p}`, () => {
        expect(closeEnough(newHyperfluxEffect(n, p), oldHyperfluxEffect(n, p))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaSingReduction1Effect (n × insideSingularityChallenge)', () => {
  for (const inside of [false, true]) {
    for (const n of [0, 1, 2]) {
      it(`n=${n} inside=${inside}`, () => {
        expect(closeEnough(newSingRed1Effect(n, inside), oldSingRed1Effect(n, inside))).toBe(true)
      })
    }
  }
})

describe('parity: ambrosiaSingReduction2Effect (n × insideSingularityChallenge)', () => {
  for (const inside of [false, true]) {
    for (const n of [0, 1, 2]) {
      it(`n=${n} inside=${inside}`, () => {
        expect(closeEnough(newSingRed2Effect(n, inside), oldSingRed2Effect(n, inside))).toBe(true)
      })
    }
  }
})
