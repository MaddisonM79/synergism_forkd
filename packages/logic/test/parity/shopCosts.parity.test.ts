// Parity tests for shopCost, lifted from packages/web_ui/src/Shop.ts.
// Sweeps cover both branches: consumable / one-shot upgrades (flat price)
// vs. stacked upgrades (linear-in-level scaling).

import { describe, expect, it } from 'vitest'
import { shopCost as newShopCost, type ShopCostInput } from '../../src/mechanics/shopCosts'

// ─── Old implementation (verbatim from packages/web_ui/src/Shop.ts) ────────

const oldShopCost = (input: ShopCostInput): number => {
  if (input.isConsumable || input.maxLevel === 1) {
    return input.price
  }
  return input.price + input.priceIncrease * input.currentLevel
}

describe('parity: shopCost (consumable branch)', () => {
  // Consumables always return flat price regardless of currentLevel.
  for (const currentLevel of [0, 1, 10, 100]) {
    for (const price of [10, 100, 1000]) {
      it(`consumable price=${price} level=${currentLevel}`, () => {
        const input: ShopCostInput = {
          isConsumable: true,
          maxLevel: 100, // irrelevant for consumables
          price,
          priceIncrease: 50, // irrelevant for consumables
          currentLevel
        }
        expect(newShopCost(input)).toBe(oldShopCost(input))
        expect(newShopCost(input)).toBe(price)
      })
    }
  }
})

describe('parity: shopCost (maxLevel=1 branch)', () => {
  // One-shot upgrades also flat-price.
  for (const currentLevel of [0, 1]) {
    for (const price of [10, 1000]) {
      it(`maxLevel=1 price=${price} level=${currentLevel}`, () => {
        const input: ShopCostInput = {
          isConsumable: false,
          maxLevel: 1,
          price,
          priceIncrease: 25,
          currentLevel
        }
        expect(newShopCost(input)).toBe(oldShopCost(input))
        expect(newShopCost(input)).toBe(price)
      })
    }
  }
})

describe('parity: shopCost (stacked-upgrade branch)', () => {
  // Linear scaling: price + priceIncrease * currentLevel.
  for (const price of [10, 100]) {
    for (const priceIncrease of [5, 20, 100]) {
      for (const currentLevel of [0, 1, 5, 10, 100]) {
        it(`price=${price} incr=${priceIncrease} level=${currentLevel}`, () => {
          const input: ShopCostInput = {
            isConsumable: false,
            maxLevel: 100,
            price,
            priceIncrease,
            currentLevel
          }
          expect(newShopCost(input)).toBe(oldShopCost(input))
        })
      }
    }
  }
})
