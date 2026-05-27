// Parity tests for tesseract buildings.
//
// Pre-migration source: packages/web_ui/src/Buy.ts at commit 3574df00~1.
// Three OLD functions are transcribed here as pure helpers (player/G refs
// replaced with explicit parameters) and compared to the migrated versions in
// packages/logic/src/mechanics/tesseractBuildings.ts across a grid.

import { describe, expect, it } from 'vitest'
import {
  buyTesseractBuilding as newBuyTesseractBuilding,
  calculateTessBuildingsInBudget as newCalculateTessBuildingsInBudget,
  getTesseractCost as newGetTesseractCost,
  type TesseractBuildingIndex,
  type TesseractBuildings
} from '../../src/mechanics/tesseractBuildings'
import type { TesseractBuildingsState } from '../../src/state/schema'

// ─── OLD reference impls ───────────────────────────────────────────────────

const tesseractBuildingCostsOld = [1, 10, 100, 1000, 10000] as const

const oldBuyTessBuildingsToCheapestPrice = (
  ownedBuildings: TesseractBuildings,
  cheapestPrice: number
): [number, TesseractBuildings] => {
  const buyToBuildings = ownedBuildings.map((currentlyOwned, index) => {
    if (currentlyOwned === null) {
      return null
    }
    const buyTo = Math.ceil(Math.pow(cheapestPrice / tesseractBuildingCostsOld[index], 1 / 3) - 1)
    return Math.max(currentlyOwned, buyTo)
  }) as TesseractBuildings

  let price = 0
  for (let i = 0; i < ownedBuildings.length; i++) {
    const buyFrom = ownedBuildings[i]
    const buyTo = buyToBuildings[i]
    if (buyFrom === null || buyTo === null) continue
    price += tesseractBuildingCostsOld[i]
      * (Math.pow(buyTo * (buyTo + 1) / 2, 2) - Math.pow(buyFrom * (buyFrom + 1) / 2, 2))
  }

  return [price, buyToBuildings]
}

const oldCalculateTessBuildingsInBudget = (
  ownedBuildings: TesseractBuildings,
  budget: number
): TesseractBuildings => {
  let minCurrentPrice: number | null = null
  for (let i = 0; i < ownedBuildings.length; i++) {
    const owned = ownedBuildings[i]
    if (owned === null) continue
    const price = tesseractBuildingCostsOld[i] * Math.pow(owned + 1, 3)
    if (minCurrentPrice === null || price < minCurrentPrice) {
      minCurrentPrice = price
    }
  }
  if (minCurrentPrice === null || minCurrentPrice > budget) return ownedBuildings

  let lo = minCurrentPrice
  let hi = lo * 2
  while (oldBuyTessBuildingsToCheapestPrice(ownedBuildings, hi)[0] <= budget) {
    lo = hi
    hi *= 2
  }
  while (hi - lo > 0.5) {
    const mid = lo + (hi - lo) / 2
    if (mid === lo || mid === hi) break
    if (oldBuyTessBuildingsToCheapestPrice(ownedBuildings, mid)[0] <= budget) {
      lo = mid
    } else {
      hi = mid
    }
  }

  const [cost, buildings] = oldBuyTessBuildingsToCheapestPrice(ownedBuildings, lo)

  let remainingBudget = budget - cost
  const currentPrices = buildings.map((num, index) => {
    if (num === null) return null
    return tesseractBuildingCostsOld[index] * Math.pow(num + 1, 3)
  })

  for (let iteration = 1; iteration <= 5; iteration++) {
    let minimum: { price: number; index: number } | null = null
    for (let index = 0; index < currentPrices.length; index++) {
      const price = currentPrices[index]
      if (price === null) continue
      if (minimum === null || price <= minimum.price) {
        minimum = { price, index }
      }
    }
    if (minimum !== null && minimum.price <= remainingBudget) {
      remainingBudget -= minimum.price
      buildings[minimum.index]!++
      currentPrices[minimum.index] = tesseractBuildingCostsOld[minimum.index]
        * Math.pow(buildings[minimum.index]! + 1, 3)
    } else {
      break
    }
  }

  return buildings
}

// OLD getTesseractCost — hoists player.wowTesseracts and the chosen building's
// `owned` count into explicit parameters.
const oldGetTesseractCost = (
  index: TesseractBuildingIndex,
  amount: number,
  wowTesseracts: number,
  buyFromOwned: number
): [number, number] => {
  const intCost = tesseractBuildingCostsOld[index - 1]
  const subCost = intCost * Math.pow(buyFromOwned * (buyFromOwned + 1) / 2, 2)

  const buyTo = Math.floor(
    -1 / 2 + 1 / 2 * Math.pow(1 + 8 * Math.pow((wowTesseracts + subCost) / intCost, 1 / 2), 1 / 2)
  )
  const actualBuy = Math.min(buyTo, buyFromOwned + amount)
  const actualCost = intCost * Math.pow(actualBuy * (actualBuy + 1) / 2, 2) - subCost
  return [actualBuy, actualCost]
}

// OLD buyTesseractBuilding modeled as a pure state transformation. The
// original mutated player[ascendBuildingN].owned/.cost and called
// player.wowTesseracts.sub(actualCost) which mutated the WowTesseracts wrapper
// in place clamping at 0. Here we model the same clamping.
interface FlatSliceForTier {
  owned: number
  cost: number
}
const applyOldBuyTesseract = (
  state: TesseractBuildingsState,
  index: TesseractBuildingIndex,
  amount: number
): TesseractBuildingsState => {
  const intCost = tesseractBuildingCostsOld[index - 1]
  const next: TesseractBuildingsState = {
    wowTesseracts: state.wowTesseracts,
    ascendBuilding1: { ...state.ascendBuilding1 },
    ascendBuilding2: { ...state.ascendBuilding2 },
    ascendBuilding3: { ...state.ascendBuilding3 },
    ascendBuilding4: { ...state.ascendBuilding4 },
    ascendBuilding5: { ...state.ascendBuilding5 }
  }
  const slice: FlatSliceForTier = next[`ascendBuilding${index}` as const]
  const [buyTo, actualCost] = oldGetTesseractCost(index, amount, next.wowTesseracts, slice.owned)
  slice.owned = buyTo
  next.wowTesseracts = Math.max(0, next.wowTesseracts - actualCost)
  slice.cost = intCost * Math.pow(1 + buyTo, 3)
  return next
}

// ─── Helpers / fixtures ────────────────────────────────────────────────────

const makeState = (
  owned: [number, number, number, number, number] = [0, 0, 0, 0, 0],
  wowTesseracts = 0
): TesseractBuildingsState => ({
  wowTesseracts,
  ascendBuilding1: { owned: owned[0], cost: tesseractBuildingCostsOld[0] * Math.pow(1 + owned[0], 3) },
  ascendBuilding2: { owned: owned[1], cost: tesseractBuildingCostsOld[1] * Math.pow(1 + owned[1], 3) },
  ascendBuilding3: { owned: owned[2], cost: tesseractBuildingCostsOld[2] * Math.pow(1 + owned[2], 3) },
  ascendBuilding4: { owned: owned[3], cost: tesseractBuildingCostsOld[3] * Math.pow(1 + owned[3], 3) },
  ascendBuilding5: { owned: owned[4], cost: tesseractBuildingCostsOld[4] * Math.pow(1 + owned[4], 3) }
})

// ─── Parity assertions ─────────────────────────────────────────────────────

describe('parity: calculateTessBuildingsInBudget', () => {
  const fixtures: Array<{ owned: TesseractBuildings; budget: number }> = [
    { owned: [0, 0, 0, 0, 0], budget: 0 },
    { owned: [0, 0, 0, 0, 0], budget: 100 },
    { owned: [0, 0, 0, 0, 0], budget: 1_000_000 },
    { owned: [null, 0, 0, 0, 0], budget: 100 },
    { owned: [3, 1, 0, 0, 0], budget: 64 + 80 - 1 },
    { owned: [3, 1, 0, 0, 0], budget: 64 + 80 },
    { owned: [9, 100, 100, 0, 100], budget: 1000 },
    { owned: [9, 100, 100, 0, 100], budget: 2000 },
    { owned: [5, 5, 5, 5, 5], budget: 0 },
    { owned: [5, 5, 5, 5, 5], budget: 63 },
    { owned: [null, null, null, null, 0], budget: 1e9 },
    { owned: [10, 10, 10, 10, 10], budget: 1e12 },
    { owned: [100, 100, 100, 100, 100], budget: 1e30 }
  ]

  it.each(fixtures)('owned=$owned budget=$budget matches', ({ owned, budget }) => {
    const oldResult = oldCalculateTessBuildingsInBudget([...owned] as TesseractBuildings, budget)
    const newResult = newCalculateTessBuildingsInBudget([...owned] as TesseractBuildings, budget)
    expect(newResult).toEqual(oldResult)
  })
})

describe('parity: getTesseractCost', () => {
  const fixtures: Array<{ index: TesseractBuildingIndex; amount: number; wow: number; buyFrom: number }> = [
    { index: 1, amount: 1, wow: 0, buyFrom: 0 },
    { index: 1, amount: 10, wow: 1e6, buyFrom: 0 },
    { index: 1, amount: 100, wow: 1e9, buyFrom: 5 },
    { index: 1, amount: 1000, wow: 100, buyFrom: 0 }, // affordability cap
    { index: 2, amount: 5, wow: 1e6, buyFrom: 0 },
    { index: 3, amount: 50, wow: 1e9, buyFrom: 10 },
    { index: 4, amount: 20, wow: 1e12, buyFrom: 0 },
    { index: 5, amount: 2, wow: 1e6, buyFrom: 0 },
    { index: 5, amount: 100, wow: 1e30, buyFrom: 50 }
  ]

  it.each(fixtures)('index=$index amount=$amount wow=$wow buyFrom=$buyFrom matches', ({ index, amount, wow, buyFrom }) => {
    const oldResult = oldGetTesseractCost(index, amount, wow, buyFrom)
    const state = makeState([0, 0, 0, 0, 0], wow)
    // Use buyFrom override to mirror the OLD overload semantics.
    const newResult = newGetTesseractCost(index, { amount, buyFrom }, state)
    expect(newResult).toEqual(oldResult)
  })
})

describe('parity: buyTesseractBuilding', () => {
  const fixtures: Array<{ owned: [number, number, number, number, number]; wow: number; index: TesseractBuildingIndex; amount: number }> = [
    { owned: [0, 0, 0, 0, 0], wow: 0, index: 1, amount: 10 },
    { owned: [0, 0, 0, 0, 0], wow: 1e6, index: 1, amount: 10 },
    { owned: [0, 0, 0, 0, 0], wow: 100, index: 1, amount: 1000 }, // cap by affordability
    { owned: [5, 0, 0, 0, 0], wow: 1e9, index: 1, amount: 5 },
    { owned: [10, 5, 3, 0, 0], wow: 1e15, index: 3, amount: 50 },
    { owned: [0, 0, 0, 0, 0], wow: 1e18, index: 5, amount: 100 },
    { owned: [50, 50, 50, 50, 50], wow: 1e30, index: 5, amount: 25 }
  ]

  it.each(fixtures)('owned=$owned wow=$wow index=$index amount=$amount matches', (fixture) => {
    const state = makeState(fixture.owned, fixture.wow)
    const oldNext = applyOldBuyTesseract(state, fixture.index, fixture.amount)
    const { state: newNext } = newBuyTesseractBuilding(state, { index: fixture.index, amount: fixture.amount })
    expect(newNext.wowTesseracts).toBe(oldNext.wowTesseracts)
    for (const i of [1, 2, 3, 4, 5] as TesseractBuildingIndex[]) {
      const key = `ascendBuilding${i}` as const
      expect(newNext[key].owned).toBe(oldNext[key].owned)
      expect(newNext[key].cost).toBe(oldNext[key].cost)
    }
  })
})
