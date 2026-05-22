import type { CoreEvent } from '../events/types'
import type { AscendBuildingState, TesseractBuildingsState } from '../state/schema'

// Tesseract (ascension-tier) buildings: five tiers purchased with
// wowTesseracts. The cost of the nth building of tier i is
//   tesseractBuildingCosts[i-1] * n^3.
// Cumulative cost to own n buildings is
//   tesseractBuildingCosts[i-1] * (n * (n+1) / 2)^2.
// All resources here are plain numbers (Decimal isn't needed — buying caps out
// long before 1e308). The web_ui WowTesseracts wrapper class stays in web_ui;
// the shim passes Number(player.wowTesseracts) across the boundary.

export type TesseractBuildingIndex = 1 | 2 | 3 | 4 | 5

// Array of five owned-counts. `null` marks a building as not-to-be-bought
// (used by callers that want to allocate budget across a subset of tiers).
export type TesseractBuildings = [
  number | null,
  number | null,
  number | null,
  number | null,
  number | null
]

const TESSERACT_BUILDING_COSTS = [1, 10, 100, 1000, 10000] as const

// Access a specific tier's slice. Explicit if-chain (not switch) so the final
// return is unconditional — TypeScript narrows to AscendBuildingState without
// needing a default/exhaustiveness branch.
function getAscendBuilding(
  state: TesseractBuildingsState,
  index: TesseractBuildingIndex
): AscendBuildingState {
  if (index === 1) return state.ascendBuilding1
  if (index === 2) return state.ascendBuilding2
  if (index === 3) return state.ascendBuilding3
  if (index === 4) return state.ascendBuilding4
  return state.ascendBuilding5
}

// ─── calculateTessBuildingsInBudget ────────────────────────────────────────

// Ported verbatim from packages/web_ui/src/Buy.ts — already pure. Internal
// helper used by the binary search inside calculateTessBuildingsInBudget.
function buyTessBuildingsToCheapestPrice(
  ownedBuildings: TesseractBuildings,
  cheapestPrice: number
): [number, TesseractBuildings] {
  const buyToBuildings = ownedBuildings.map((currentlyOwned, index) => {
    if (currentlyOwned === null) {
      return null
    }
    // thisPrice >= cheapestPrice = TESSERACT_BUILDING_COSTS[index] * (buyTo+1)^3
    // buyTo = cuberoot(cheapestPrice / cost[index]) - 1; round UP so the next
    // building's price strictly exceeds cheapestPrice.
    const buyTo = Math.ceil(Math.pow(cheapestPrice / TESSERACT_BUILDING_COSTS[index], 1 / 3) - 1)
    // cheapestPrice may be below the building's current price — clamp to what
    // we already own.
    return Math.max(currentlyOwned, buyTo)
  }) as TesseractBuildings

  let price = 0
  for (let i = 0; i < ownedBuildings.length; i++) {
    const buyFrom = ownedBuildings[i]
    const buyTo = buyToBuildings[i]
    if (buyFrom === null || buyTo === null) {
      continue
    }
    price += TESSERACT_BUILDING_COSTS[i]
      * (Math.pow(buyTo * (buyTo + 1) / 2, 2) - Math.pow(buyFrom * (buyFrom + 1) / 2, 2))
  }

  return [price, buyToBuildings]
}

/**
 * Calculate the result of repeatedly buying the cheapest tesseract building,
 * given an initial list of owned buildings and a budget.
 *
 * Pure: only depends on inputs and TESSERACT_BUILDING_COSTS.
 *
 * Documented anchor cases (from the original implementation):
 *   calculateTessBuildingsInBudget([0,0,0,0,0], 100)         -> [3,1,0,0,0]
 *   calculateTessBuildingsInBudget([null,0,0,0,0], 100)      -> [null,2,0,0,0]
 *   calculateTessBuildingsInBudget([3,1,0,0,0], 64+80-1)     -> [4,1,0,0,0]
 *   calculateTessBuildingsInBudget([3,1,0,0,0], 64+80)       -> [4,2,0,0,0]
 *   calculateTessBuildingsInBudget([9,100,100,0,100], 1000)  -> [9,100,100,1,100]
 *   calculateTessBuildingsInBudget([9,100,100,0,100], 2000)  -> [10,100,100,1,100]
 *   calculateTessBuildingsInBudget([0,0,0,0,0], 1e46)        -> runs in <1s
 *
 * @param ownedBuildings Current counts, with null marking tiers to skip.
 * @param budget Tesseracts available to spend.
 */
export function calculateTessBuildingsInBudget(
  ownedBuildings: TesseractBuildings,
  budget: number
): TesseractBuildings {
  // Cheapest current next-building price. If null, every tier is opted-out.
  let minCurrentPrice: number | null = null
  for (let i = 0; i < ownedBuildings.length; i++) {
    const owned = ownedBuildings[i]
    if (owned === null) {
      continue
    }
    const price = TESSERACT_BUILDING_COSTS[i] * Math.pow(owned + 1, 3)
    if (minCurrentPrice === null || price < minCurrentPrice) {
      minCurrentPrice = price
    }
  }

  if (minCurrentPrice === null || minCurrentPrice > budget) {
    return ownedBuildings
  }

  // Binary search for the maximum "cheapest price" the budget can reach. See
  // the original commentary in web_ui — the math relies on the fact that
  // f(cheapestPrice) = cumulative cost to buy until all next-prices are
  // >= cheapestPrice is monotone in cheapestPrice.
  let lo = minCurrentPrice
  let hi = lo * 2
  while (buyTessBuildingsToCheapestPrice(ownedBuildings, hi)[0] <= budget) {
    lo = hi
    hi *= 2
  }
  while (hi - lo > 0.5) {
    const mid = lo + (hi - lo) / 2
    // Floating-point edge: mid can equal lo or hi even when hi > lo. Break to
    // avoid an infinite loop.
    if (mid === lo || mid === hi) {
      break
    }
    if (buyTessBuildingsToCheapestPrice(ownedBuildings, mid)[0] <= budget) {
      lo = mid
    } else {
      hi = mid
    }
  }

  const [cost, buildings] = buyTessBuildingsToCheapestPrice(ownedBuildings, lo)

  // Edge case: when 2..5 tiers share the cheapest price and we can only
  // afford a subset of them. Binary search hands back a state where one more
  // building is still affordable; clean it up by greedily buying the cheapest
  // a handful of times.
  let remainingBudget = budget - cost
  const currentPrices = buildings.map((num, index) => {
    if (num === null) {
      return null
    }
    return TESSERACT_BUILDING_COSTS[index] * Math.pow(num + 1, 3)
  })

  for (let iteration = 1; iteration <= 5; iteration++) {
    let minimum: { price: number; index: number } | null = null
    for (let index = 0; index < currentPrices.length; index++) {
      const price = currentPrices[index]
      if (price === null) {
        continue
      }
      // <= over < to prefer higher tiers when prices tie.
      if (minimum === null || price <= minimum.price) {
        minimum = { price, index }
      }
    }
    if (minimum !== null && minimum.price <= remainingBudget) {
      remainingBudget -= minimum.price
      // buildings[minimum.index] is guaranteed non-null at this point.
      buildings[minimum.index]!++
      currentPrices[minimum.index] = TESSERACT_BUILDING_COSTS[minimum.index]
        * Math.pow(buildings[minimum.index]! + 1, 3)
    } else {
      break
    }
  }

  return buildings
}

// ─── getTesseractCost ──────────────────────────────────────────────────────

export interface GetTesseractCostInput {
  /** Number of buildings to attempt to buy. */
  amount: number
  /** Limit the purchase to what wowTesseracts can afford (default true). */
  checkCanAfford?: boolean
  /** Override starting count. Defaults to state's current owned for this tier. */
  buyFrom?: number
}

/**
 * Compute the new owned-count and tesseracts spent for a tier purchase.
 * Returns [newOwned, costSpent].
 */
export function getTesseractCost(
  index: TesseractBuildingIndex,
  input: GetTesseractCostInput,
  state: TesseractBuildingsState
): [number, number] {
  const intCost = TESSERACT_BUILDING_COSTS[index - 1]
  const buyFrom = input.buyFrom ?? getAscendBuilding(state, index).owned
  const subCost = intCost * Math.pow(buyFrom * (buyFrom + 1) / 2, 2)
  const checkCanAfford = input.checkCanAfford ?? true

  let actualBuy: number
  if (checkCanAfford) {
    // Inverse of cumulative cost: solve cost(buyTo) = wowTesseracts + subCost.
    // cost(n) = intCost * (n(n+1)/2)^2  →  n = (-1 + sqrt(1 + 8 * sqrt(C/intCost))) / 2
    const buyTo = Math.floor(
      -1 / 2 + 1 / 2 * Math.pow(1 + 8 * Math.pow((state.wowTesseracts + subCost) / intCost, 1 / 2), 1 / 2)
    )
    actualBuy = Math.min(buyTo, buyFrom + input.amount)
  } else {
    actualBuy = buyFrom + input.amount
  }
  const actualCost = intCost * Math.pow(actualBuy * (actualBuy + 1) / 2, 2) - subCost
  return [actualBuy, actualCost]
}

// ─── buyTesseractBuilding ──────────────────────────────────────────────────

export interface BuyTesseractBuildingInput {
  /** Which tier to buy. */
  index: TesseractBuildingIndex
  /** How many to buy (caller usually passes player.tesseractbuyamount). */
  amount: number
}

/**
 * Buy as many of the selected tesseract building as the budget allows, up to
 * `amount`. Returns the new state slice and a purchase event when the count
 * changes.
 */
export function buyTesseractBuilding(
  state: TesseractBuildingsState,
  input: BuyTesseractBuildingInput
): { state: TesseractBuildingsState; events: CoreEvent[] } {
  const events: CoreEvent[] = []
  const next: TesseractBuildingsState = {
    wowTesseracts: state.wowTesseracts,
    ascendBuilding1: { ...state.ascendBuilding1 },
    ascendBuilding2: { ...state.ascendBuilding2 },
    ascendBuilding3: { ...state.ascendBuilding3 },
    ascendBuilding4: { ...state.ascendBuilding4 },
    ascendBuilding5: { ...state.ascendBuilding5 }
  }
  const intCost = TESSERACT_BUILDING_COSTS[input.index - 1]
  const target = getAscendBuilding(next, input.index)
  const buyStart = target.owned
  const [buyTo, actualCost] = getTesseractCost(input.index, { amount: input.amount }, next)

  target.owned = buyTo
  next.wowTesseracts = Math.max(0, next.wowTesseracts - actualCost)
  target.cost = intCost * Math.pow(1 + buyTo, 3)

  if (buyTo > buyStart) {
    events.push({
      kind: 'tesseract-buildings-purchased',
      index: input.index,
      before: buyStart,
      after: buyTo,
      spent: state.wowTesseracts - next.wowTesseracts
    })
  }

  return { state: next, events }
}
