// Parity tests for ant-upgrade effects + cost solvers. Old bodies
// transcribed verbatim from
//   packages/web_ui/src/Features/Ants/AntUpgrades/data/data.ts (effects)
//   packages/web_ui/src/Features/Ants/AntUpgrades/lib/get-cost.ts (solvers)
//
// Sweep covers each effect across representative levels including the
// piecewise transitions (Mortuus level=0 vs 1, AntSacrifice cap at 200,
// AntELO cap at min(antSacLimit, antSacrificeCount), AscensionScore three-
// term decay points).

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  acceleratorBoostsAntUpgradeEffect as newAcceleratorBoosts,
  antELOAntUpgradeEffect as newAntELO,
  antSacrificeAntUpgradeEffect as newAntSacrifice,
  antSpeedAntUpgradeEffect as newAntSpeed,
  antUpgradeBaseCosts as newBaseCosts,
  antUpgradeCostIncreaseExponents as newExponents,
  ascensionScoreAntUpgradeEffect as newAscensionScore,
  buildingCostScaleAntUpgradeEffect as newBuildingCostScale,
  coinsAntUpgradeEffect as newCoins,
  freeRunesAntUpgradeEffect as newFreeRunes,
  getCostMaxAntUpgrades as newCostMax,
  getCostNextAntUpgrade as newCostNext,
  getMaxPurchasableAntUpgrades as newMaxPurchasable,
  mortuusAntUpgradeEffect as newMortuus,
  mortuus2AntUpgradeEffect as newMortuus2,
  multipliersAntUpgradeEffect as newMultipliers,
  obtainiumAntUpgradeEffect as newObtainium,
  offeringsAntUpgradeEffect as newOfferings,
  salvageAntUpgradeEffect as newSalvage,
  taxesAntUpgradeEffect as newTaxes,
  wowCubesAntUpgradeEffect as newWowCubes
} from '../../src/mechanics/antUpgrades'
import { calculateSigmoidExponential } from '../../src/math/sigmoid'

// ─── Old implementations ──────────────────────────────────────────────────

const oldBaseCosts: Decimal[] = [
  Decimal.fromString('100'),
  Decimal.fromString('100'),
  Decimal.fromString('1000'),
  Decimal.fromString('1000'),
  Decimal.fromString('1e5'),
  Decimal.fromString('1e6'),
  Decimal.fromString('1e11'),
  Decimal.fromString('1e15'),
  Decimal.fromString('1e20'),
  Decimal.fromString('1e6'),
  Decimal.fromString('1e120'),
  Decimal.fromString('1e300'),
  Decimal.fromString('1e70'),
  Decimal.fromString('1e400'),
  Decimal.fromString('1e300'),
  Decimal.fromString('1e37777')
]

const oldExponents = [1, 1, 1, 1, 2, 2, 2, 3, 3, 2, 20, 100, 4, 10, 2, 2000]

const oldAntSpeed = (n: number, r101: number, r162: number) => {
  const baseMul = 1.1 + r101 / 1000 + r162 / 1000
  return { antSpeed: Decimal.pow(baseMul, n) }
}

const oldCoins = (n: number, ascensionChallenge: number, crumbs: Decimal) => {
  let divisor = 1
  if (ascensionChallenge === 15) {
    divisor = 100 + 9900 * (1000 + n) / (1000 + n ** 2)
  }
  const baseExponent = 999999 + calculateSigmoidExponential(49000001, n / 3000)
  const bonusExponent = 250 * n
  const exponent = (baseExponent + bonusExponent) / divisor
  const coinMult = Decimal.max(1, Decimal.pow(crumbs, exponent))
  return { crumbToCoinExp: exponent, coinMultiplier: coinMult }
}

const oldTaxes = (n: number) => ({ taxReduction: 0.005 + 0.995 * Math.pow(0.99, n) })
const oldAcceleratorBoosts = (n: number) => ({ acceleratorBoostMult: calculateSigmoidExponential(20, n / 1000) })
const oldMultipliers = (n: number) => ({ multiplierMult: calculateSigmoidExponential(40, n / 1000) })
const oldOfferings = (n: number) => ({ offeringMult: Math.pow(1 + n / 10, 0.5) })
const oldBuildingCostScale = (n: number) => ({
  buildingCostScale: (3 * n) / 100,
  buildingPowerMult: 1 + n / 100
})
const oldSalvage = (n: number) => ({ salvage: 120 * (1 - Math.pow(0.995, n)) })
const oldFreeRunes = (n: number) => ({ freeRuneLevel: 3000 * (1 - Math.pow(1 - 1 / 3000, n)) })
const oldObtainium = (n: number) => ({ obtainiumMult: Math.pow(1 + n / 10, 0.5) })
const oldAntSacrifice = (n: number) => ({
  antSacrificeMultiplier: Math.pow(1 + n / 10, 0.5),
  elo: Math.round(5 * Math.min(200, n))
})
const oldMortuus = (n: number) => ({
  talismanUnlock: n > 0,
  globalSpeed: 2 - Math.pow(0.99, n)
})
const oldAntELO = (n: number, antSacrificeCount: number, upgradeImproverInput: number) => {
  const antSacrificeLimitCount = n + 200 * Math.min(1, n)
  const upgradeImprover = Math.min(n, upgradeImproverInput)
  const effectiveSacs = Math.min(
    antSacrificeLimitCount + upgradeImprover,
    antSacrificeCount + upgradeImprover
  )
  return { antELO: effectiveSacs, antSacrificeLimitCount }
}
const oldMortuus2 = (n: number) => ({
  talismanLevelIncreaser: Math.min(1200, Math.floor(n / 2)),
  talismanEffectBuff: 1 + 0.65 * (1 - Math.pow(0.999, n)) + 0.005 * Math.min(20, n),
  ascensionSpeed: 1 + 0.5 * (1 - Math.pow(0.996, n))
})
const oldAscensionScore = (n: number) => ({
  ascensionScoreBase: 100000 * (1 - Math.pow(0.999, n)),
  cubesBanked: 3 * Math.min(200, n)
    + 2500 * (1 - Math.pow(1 - 1 / 2750, n))
    + 96900 * (1 - Math.pow(1 - 1 / 969000, n))
})
const oldWowCubes = (n: number) => ({ wowCubes: 2 - Math.pow(0.999, n) })

const oldCostNext = (baseCost: Decimal, exp: number, currentLevel: number) => {
  const nextCost = baseCost.times(Decimal.pow(10, currentLevel * exp))
  const lastCost = currentLevel > 0
    ? baseCost.times(Decimal.pow(10, (currentLevel - 1) * exp))
    : Decimal.fromNumber(0)
  return nextCost.sub(lastCost)
}

const oldMaxPurchasable = (baseCost: Decimal, exp: number, currentLevel: number, budget: Decimal) => {
  const sunkCost = currentLevel > 0
    ? baseCost.times(Decimal.pow(10, exp * (currentLevel - 1)))
    : Decimal.fromNumber(0)
  const realBudget = budget.add(sunkCost)
  return Math.max(0, 1 + Math.floor(Decimal.log(realBudget.div(baseCost), 10) / exp))
}

const oldCostMax = (baseCost: Decimal, exp: number, currentLevel: number, maxBuyable: number) => {
  const spent = currentLevel > 0
    ? Decimal.pow(10, exp * (currentLevel - 1)).times(baseCost)
    : Decimal.fromNumber(0)
  const maxCost = Decimal.pow(10, exp * (maxBuyable - 1)).times(baseCost)
  return maxCost.sub(spent)
}

// ─── Data table parity ────────────────────────────────────────────────────

describe('parity: antUpgradeBaseCosts table', () => {
  it('has 16 entries', () => {
    expect(newBaseCosts.length).toBe(16)
  })
  for (let i = 0; i < 16; i++) {
    it(`baseCost[${i}] matches`, () => {
      expect(newBaseCosts[i].eq(oldBaseCosts[i])).toBe(true)
    })
    it(`costIncreaseExponent[${i}] matches`, () => {
      expect(newExponents[i]).toBe(oldExponents[i])
    })
  }
})

// ─── Pure effect functions ────────────────────────────────────────────────

const levelGrid = [0, 1, 10, 100, 1000, 5000]

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)
const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

describe('parity: AntSpeed (3-arg, reads research)', () => {
  const cases = [
    { n: 0, r101: 0, r162: 0 },
    { n: 100, r101: 0, r162: 0 },
    { n: 100, r101: 200, r162: 100 },
    { n: 1000, r101: 200, r162: 100 }
  ]
  for (const c of cases) {
    it(JSON.stringify(c), () => {
      const newRes = newAntSpeed({ level: c.n, research101: c.r101, research162: c.r162 })
      const oldRes = oldAntSpeed(c.n, c.r101, c.r162)
      expect(decimalEq(newRes.antSpeed, oldRes.antSpeed)).toBe(true)
    })
  }
})

describe('parity: Coins (reads ascensionChallenge + crumbs)', () => {
  const cases = [
    { n: 0, asc: 0, crumbs: new Decimal(1) },
    { n: 100, asc: 0, crumbs: new Decimal('1e10') },
    { n: 100, asc: 15, crumbs: new Decimal('1e10') }, // c15 divisor path
    { n: 1000, asc: 15, crumbs: new Decimal('1e1000') }
  ]
  for (const c of cases) {
    it(JSON.stringify({ n: c.n, asc: c.asc, crumbs: c.crumbs.toString() }), () => {
      const newRes = newCoins({ level: c.n, ascensionChallenge: c.asc, crumbs: c.crumbs })
      const oldRes = oldCoins(c.n, c.asc, c.crumbs)
      expect(closeEnough(newRes.crumbToCoinExp, oldRes.crumbToCoinExp)).toBe(true)
      expect(decimalEq(newRes.coinMultiplier, oldRes.coinMultiplier)).toBe(true)
    })
  }
})

describe('parity: Taxes', () => {
  for (const n of levelGrid) {
    it(`level=${n}`, () => {
      expect(closeEnough(newTaxes(n).taxReduction, oldTaxes(n).taxReduction)).toBe(true)
    })
  }
})

describe('parity: AcceleratorBoosts', () => {
  for (const n of levelGrid) {
    it(`level=${n}`, () => {
      expect(closeEnough(
        newAcceleratorBoosts(n).acceleratorBoostMult,
        oldAcceleratorBoosts(n).acceleratorBoostMult
      )).toBe(true)
    })
  }
})

describe('parity: Multipliers', () => {
  for (const n of levelGrid) {
    it(`level=${n}`, () => {
      expect(closeEnough(newMultipliers(n).multiplierMult, oldMultipliers(n).multiplierMult)).toBe(true)
    })
  }
})

describe('parity: Offerings', () => {
  for (const n of levelGrid) {
    it(`level=${n}`, () => {
      expect(closeEnough(newOfferings(n).offeringMult, oldOfferings(n).offeringMult)).toBe(true)
    })
  }
})

describe('parity: BuildingCostScale', () => {
  for (const n of levelGrid) {
    it(`level=${n}`, () => {
      const newRes = newBuildingCostScale(n)
      const oldRes = oldBuildingCostScale(n)
      expect(closeEnough(newRes.buildingCostScale, oldRes.buildingCostScale)).toBe(true)
      expect(closeEnough(newRes.buildingPowerMult, oldRes.buildingPowerMult)).toBe(true)
    })
  }
})

describe('parity: Salvage', () => {
  for (const n of levelGrid) {
    it(`level=${n}`, () => {
      expect(closeEnough(newSalvage(n).salvage, oldSalvage(n).salvage)).toBe(true)
    })
  }
})

describe('parity: FreeRunes', () => {
  for (const n of levelGrid) {
    it(`level=${n}`, () => {
      expect(closeEnough(newFreeRunes(n).freeRuneLevel, oldFreeRunes(n).freeRuneLevel)).toBe(true)
    })
  }
})

describe('parity: Obtainium', () => {
  for (const n of levelGrid) {
    it(`level=${n}`, () => {
      expect(closeEnough(newObtainium(n).obtainiumMult, oldObtainium(n).obtainiumMult)).toBe(true)
    })
  }
})

describe('parity: AntSacrifice (200-cap on elo)', () => {
  const grid = [...levelGrid, 199, 200, 201, 1000]
  for (const n of grid) {
    it(`level=${n}`, () => {
      const newRes = newAntSacrifice(n)
      const oldRes = oldAntSacrifice(n)
      expect(closeEnough(newRes.antSacrificeMultiplier, oldRes.antSacrificeMultiplier)).toBe(true)
      expect(newRes.elo).toBe(oldRes.elo)
    })
  }
})

describe('parity: Mortuus (level=0 special)', () => {
  for (const n of [0, 1, 100, 1000]) {
    it(`level=${n}`, () => {
      const newRes = newMortuus(n)
      const oldRes = oldMortuus(n)
      expect(newRes.talismanUnlock).toBe(oldRes.talismanUnlock)
      expect(closeEnough(newRes.globalSpeed, oldRes.globalSpeed)).toBe(true)
    })
  }
})

describe('parity: AntELO (reads antSacrificeCount + improver)', () => {
  const cases = [
    { n: 0, antSacrificeCount: 0, improver: 0 },
    { n: 100, antSacrificeCount: 50, improver: 0 },
    { n: 200, antSacrificeCount: 1000, improver: 30 }, // limit > sacrifices
    { n: 5, antSacrificeCount: 1, improver: 10 } // improver caps at level
  ]
  for (const c of cases) {
    it(JSON.stringify(c), () => {
      const newRes = newAntELO({
        level: c.n,
        antSacrificeCount: c.antSacrificeCount,
        antSpeed2UpgradeImprover: c.improver
      })
      const oldRes = oldAntELO(c.n, c.antSacrificeCount, c.improver)
      expect(newRes.antELO).toBe(oldRes.antELO)
      expect(newRes.antSacrificeLimitCount).toBe(oldRes.antSacrificeLimitCount)
    })
  }
})

describe('parity: Mortuus2 (talisman cap at 1200, +0.005 capped at 20)', () => {
  for (const n of [0, 1, 19, 20, 21, 100, 2400, 2401, 10000]) {
    it(`level=${n}`, () => {
      const newRes = newMortuus2(n)
      const oldRes = oldMortuus2(n)
      expect(newRes.talismanLevelIncreaser).toBe(oldRes.talismanLevelIncreaser)
      expect(closeEnough(newRes.talismanEffectBuff, oldRes.talismanEffectBuff)).toBe(true)
      expect(closeEnough(newRes.ascensionSpeed, oldRes.ascensionSpeed)).toBe(true)
    })
  }
})

describe('parity: AscensionScore (3-term decay; cap at 200 on first term)', () => {
  for (const n of [0, 1, 100, 200, 201, 1000, 100000]) {
    it(`level=${n}`, () => {
      const newRes = newAscensionScore(n)
      const oldRes = oldAscensionScore(n)
      expect(closeEnough(newRes.ascensionScoreBase, oldRes.ascensionScoreBase)).toBe(true)
      expect(closeEnough(newRes.cubesBanked, oldRes.cubesBanked)).toBe(true)
    })
  }
})

describe('parity: WowCubes', () => {
  for (const n of levelGrid) {
    it(`level=${n}`, () => {
      expect(closeEnough(newWowCubes(n).wowCubes, oldWowCubes(n).wowCubes)).toBe(true)
    })
  }
})

// ─── Cost solvers ─────────────────────────────────────────────────────────

describe('parity: getCostNextAntUpgrade', () => {
  // Exercise across all 16 upgrades at level 0, mid, high.
  const levels = [0, 1, 10, 100]
  for (let i = 0; i < 16; i++) {
    for (const lvl of levels) {
      it(`upgrade=${i} currentLevel=${lvl}`, () => {
        const newRes = newCostNext({
          baseCost: newBaseCosts[i],
          costIncreaseExponent: newExponents[i],
          currentLevel: lvl
        })
        const oldRes = oldCostNext(oldBaseCosts[i], oldExponents[i], lvl)
        expect(decimalEq(newRes, oldRes)).toBe(true)
      })
    }
  }
})

describe('parity: getMaxPurchasableAntUpgrades', () => {
  const budgets = [new Decimal(0), new Decimal('1e10'), new Decimal('1e100'), new Decimal('1e1000')]
  for (let i = 0; i < 16; i++) {
    for (const budget of budgets) {
      it(`upgrade=${i} budget=${budget.toString()} currentLevel=0`, () => {
        const newRes = newMaxPurchasable({
          baseCost: newBaseCosts[i],
          costIncreaseExponent: newExponents[i],
          currentLevel: 0,
          budget
        })
        const oldRes = oldMaxPurchasable(oldBaseCosts[i], oldExponents[i], 0, budget)
        expect(newRes).toBe(oldRes)
      })
      it(`upgrade=${i} budget=${budget.toString()} currentLevel=10`, () => {
        const newRes = newMaxPurchasable({
          baseCost: newBaseCosts[i],
          costIncreaseExponent: newExponents[i],
          currentLevel: 10,
          budget
        })
        const oldRes = oldMaxPurchasable(oldBaseCosts[i], oldExponents[i], 10, budget)
        expect(newRes).toBe(oldRes)
      })
    }
  }
})

describe('parity: getCostMaxAntUpgrades', () => {
  // Sweep a few representative cases per upgrade
  const cases = [
    { currentLevel: 0, maxBuyable: 1 },
    { currentLevel: 0, maxBuyable: 10 },
    { currentLevel: 5, maxBuyable: 10 },
    { currentLevel: 10, maxBuyable: 100 }
  ]
  for (let i = 0; i < 16; i++) {
    for (const c of cases) {
      it(`upgrade=${i} ${JSON.stringify(c)}`, () => {
        const newRes = newCostMax({
          baseCost: newBaseCosts[i],
          costIncreaseExponent: newExponents[i],
          currentLevel: c.currentLevel,
          maxBuyable: c.maxBuyable
        })
        const oldRes = oldCostMax(oldBaseCosts[i], oldExponents[i], c.currentLevel, c.maxBuyable)
        expect(decimalEq(newRes, oldRes)).toBe(true)
      })
    }
  }
})
