// Parity tests for the ant-producer cluster. Old bodies transcribed verbatim
// from packages/web_ui/src/Features/Ants/AntProducers/data/data.ts (pure
// fields) and lib/get-cost.ts + lib/calculate-production.ts.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  antProducerData as newProducerData,
  calculateBaseAntsToBeGenerated as newBaseGen,
  getCostMaxAntProducers as newCostMax,
  getCostNextAntProducer as newCostNext,
  getMaxPurchasableAntProducers as newMaxPurchasable
} from '../../src/mechanics/antProducers'

// ─── Old data table ───────────────────────────────────────────────────────

const oldProducerData = [
  { baseCost: Decimal.fromString('1'), costIncrease: 3, baseProduction: Decimal.fromNumber(0.01) },
  { baseCost: Decimal.fromString('10'), costIncrease: 10, baseProduction: Decimal.fromNumber(1.5e-4) },
  { baseCost: Decimal.fromString('1e5'), costIncrease: 1e2, baseProduction: Decimal.fromNumber(5e-6) },
  { baseCost: Decimal.fromString('1e12'), costIncrease: 1e4, baseProduction: Decimal.fromNumber(3e-5) },
  { baseCost: Decimal.fromString('1e145'), costIncrease: 1e8, baseProduction: Decimal.fromNumber(1e-30) },
  { baseCost: Decimal.fromString('1e700'), costIncrease: 1e16, baseProduction: Decimal.fromNumber(1e-90) },
  { baseCost: Decimal.fromString('1e5000'), costIncrease: 1e32, baseProduction: Decimal.fromString('1e-600') },
  { baseCost: Decimal.fromString('1e25000'), costIncrease: 1e64, baseProduction: Decimal.fromString('1e-3500') },
  {
    baseCost: Decimal.fromString('1e1000000'),
    costIncrease: 1e128,
    baseProduction: Decimal.fromString('1e-110000')
  }
]

// ─── Old solvers ──────────────────────────────────────────────────────────

const oldCostNext = (baseCost: Decimal, costIncrease: number, purchased: number) => {
  const nextCost = baseCost.times(Decimal.pow(costIncrease, purchased))
  const lastCost = purchased > 0
    ? baseCost.times(Decimal.pow(costIncrease, purchased - 1))
    : Decimal.fromString('0')
  return nextCost.sub(lastCost)
}

const oldMaxPurchasable = (baseCost: Decimal, costIncrease: number, purchased: number, budget: Decimal) => {
  const sunkCost = purchased > 0
    ? baseCost.times(Decimal.pow(costIncrease, purchased - 1))
    : Decimal.fromString('0')
  const realBudget = budget.add(sunkCost)
  return Math.max(0, 1 + Math.floor(Decimal.log(realBudget.div(baseCost), costIncrease)))
}

const oldCostMax = (baseCost: Decimal, costIncrease: number, purchased: number, maxBuyable: number) => {
  const spent = purchased > 0
    ? Decimal.pow(costIncrease, purchased - 1).times(baseCost)
    : Decimal.fromString('0')
  const maxCost = Decimal.pow(costIncrease, maxBuyable - 1).times(baseCost)
  return maxCost.sub(spent)
}

const oldBaseGen = (generated: Decimal, purchased: number, baseProduction: Decimal, selfSpeedMult: Decimal, antSpeedMult: Decimal) => {
  return generated.add(purchased).times(baseProduction).times(selfSpeedMult).times(antSpeedMult)
}

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

// ─── Data table ───────────────────────────────────────────────────────────

describe('parity: antProducerData', () => {
  it('has 9 entries', () => {
    expect(newProducerData.length).toBe(9)
  })
  for (let i = 0; i < 9; i++) {
    it(`producer ${i} baseCost matches`, () => {
      expect(newProducerData[i].baseCost.eq(oldProducerData[i].baseCost)).toBe(true)
    })
    it(`producer ${i} costIncrease matches`, () => {
      expect(newProducerData[i].costIncrease).toBe(oldProducerData[i].costIncrease)
    })
    it(`producer ${i} baseProduction matches`, () => {
      expect(newProducerData[i].baseProduction.eq(oldProducerData[i].baseProduction)).toBe(true)
    })
  }

  // produces chain: Breeders(1)→Workers(0), MetaBreeders(2)→Breeders(1), etc.
  it('produces chain is correct', () => {
    expect(newProducerData[0].produces).toBeUndefined()
    expect(newProducerData[1].produces).toBe(0)
    expect(newProducerData[2].produces).toBe(1)
    expect(newProducerData[3].produces).toBe(2)
    expect(newProducerData[4].produces).toBe(3)
    expect(newProducerData[5].produces).toBe(4)
    expect(newProducerData[6].produces).toBe(5)
    expect(newProducerData[7].produces).toBe(6)
    expect(newProducerData[8].produces).toBe(7)
  })
})

// ─── Cost solvers ─────────────────────────────────────────────────────────

describe('parity: getCostNextAntProducer', () => {
  const purchasedValues = [0, 1, 5, 25, 100]
  for (let i = 0; i < 9; i++) {
    const { baseCost, costIncrease } = newProducerData[i]
    for (const purchased of purchasedValues) {
      it(`producer ${i} purchased=${purchased}`, () => {
        const newRes = newCostNext({ baseCost, costIncrease, purchased })
        const oldRes = oldCostNext(baseCost, costIncrease, purchased)
        expect(decimalEq(newRes, oldRes)).toBe(true)
      })
    }
  }
})

describe('parity: getMaxPurchasableAntProducers', () => {
  const budgets = [new Decimal('0'), new Decimal('1e10'), new Decimal('1e100'), new Decimal('1e10000')]
  for (let i = 0; i < 9; i++) {
    const { baseCost, costIncrease } = newProducerData[i]
    for (const budget of budgets) {
      for (const purchased of [0, 10]) {
        it(`producer ${i} purchased=${purchased} budget=${budget.toString()}`, () => {
          const newRes = newMaxPurchasable({ baseCost, costIncrease, purchased, budget })
          const oldRes = oldMaxPurchasable(baseCost, costIncrease, purchased, budget)
          expect(newRes).toBe(oldRes)
        })
      }
    }
  }
})

describe('parity: getCostMaxAntProducers', () => {
  const cases = [
    { purchased: 0, maxBuyable: 1 },
    { purchased: 0, maxBuyable: 10 },
    { purchased: 5, maxBuyable: 10 },
    { purchased: 10, maxBuyable: 25 },
    { purchased: 25, maxBuyable: 100 }
  ]
  for (let i = 0; i < 9; i++) {
    const { baseCost, costIncrease } = newProducerData[i]
    for (const c of cases) {
      it(`producer ${i} ${JSON.stringify(c)}`, () => {
        const newRes = newCostMax({ baseCost, costIncrease, purchased: c.purchased, maxBuyable: c.maxBuyable })
        const oldRes = oldCostMax(baseCost, costIncrease, c.purchased, c.maxBuyable)
        expect(decimalEq(newRes, oldRes)).toBe(true)
      })
    }
  }
})

// ─── Base production ──────────────────────────────────────────────────────

describe('parity: calculateBaseAntsToBeGenerated', () => {
  const speedMults = [Decimal.fromString('1'), Decimal.fromString('10'), Decimal.fromString('1e10')]
  const antSpeedMults = [Decimal.fromString('1'), Decimal.fromString('100')]
  const generatedValues = [Decimal.fromString('0'), Decimal.fromString('1e6')]
  const purchasedValues = [0, 50, 500]

  for (let i = 0; i < 9; i++) {
    const { baseProduction } = newProducerData[i]
    for (const generated of generatedValues) {
      for (const purchased of purchasedValues) {
        for (const selfSpeedMult of speedMults) {
          for (const antSpeedMult of antSpeedMults) {
            it(`producer ${i} gen=${generated.toString()} pur=${purchased} self=${selfSpeedMult.toString()} ant=${antSpeedMult.toString()}`, () => {
              const newRes = newBaseGen({
                generated,
                purchased,
                baseProduction,
                selfSpeedMult,
                antSpeedMult
              })
              const oldRes = oldBaseGen(generated, purchased, baseProduction, selfSpeedMult, antSpeedMult)
              expect(decimalEq(newRes, oldRes)).toBe(true)
            })
          }
        }
      }
    }
  }
})
