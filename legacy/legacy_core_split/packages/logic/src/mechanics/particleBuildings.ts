import type { DecimalSource } from '../math/bignum'
import { Decimal } from '../math/bignum'
import { smallestInc } from '../math/smallestInc'
import type { CoreEvent } from '../events/types'
import type { BuyAmount, ParticleBuildingsState } from '../state/schema'

// Particle buildings: five positions purchased with reincarnationPoints. The
// cost curve is independent from the producer family (separate base list and a
// quadratic-in-exponent growth above a challenge-gated threshold), so it lives
// in its own module rather than alongside getProducerCost.

export type ParticleBuildingIndex = 1 | 2 | 3 | 4 | 5

// Base costs by position. Same constants as web_ui's
// mythosAndParticleBuildingCosts; mythos buildings reuse them via a different
// codepath that hasn't migrated yet.
const ORIGINAL_COSTS = [1, 1e2, 1e4, 1e8, 1e16] as const

// Parallel arrays for indexed state access. The state slice mirrors the
// player object's per-position field names; these tables let us translate
// (index 1..5) → field name without a 5-arm switch every call.
const OWNED_KEYS = [
  'firstOwnedParticles',
  'secondOwnedParticles',
  'thirdOwnedParticles',
  'fourthOwnedParticles',
  'fifthOwnedParticles'
] as const satisfies readonly (keyof ParticleBuildingsState)[]

const COST_KEYS = [
  'firstCostParticles',
  'secondCostParticles',
  'thirdCostParticles',
  'fourthCostParticles',
  'fifthCostParticles'
] as const satisfies readonly (keyof ParticleBuildingsState)[]

type OwnedKey = (typeof OWNED_KEYS)[number]
type CostKey = (typeof COST_KEYS)[number]

const BUYMAX = Math.pow(10, 15)

export interface GetParticleCostInput {
  /** Which of the five particle buildings (1..5). Picks the base cost. */
  index: ParticleBuildingIndex
  /** player.currentChallenge.ascension === 15 — flips the DR threshold 325000 → 1000. */
  inAscensionChallenge15: boolean
}

export interface BuyParticleBuildingInput extends GetParticleCostInput {
  /** True when the autobuyer is driving — bypasses the particlebuyamount cap. */
  autobuyer: boolean
  /** Per-click purchase cap selected in the UI. */
  particlebuyamount: BuyAmount
}

// Internal cost helper. Takes originalCost separately so the recursive
// quadrillion-snap path can pass it through without an extra index lookup.
function getCostInternal(
  originalCost: DecimalSource,
  buyingTo: number,
  inAscensionChallenge15: boolean
): Decimal {
  --buyingTo
  const base = new Decimal(originalCost)
  let cost = base.times(Decimal.pow(2, buyingTo))

  const DR = inAscensionChallenge15 ? 1000 : 325000

  if (buyingTo > DR) {
    cost = cost.times(Decimal.pow(1.001, (buyingTo - DR) * ((buyingTo - DR + 1) / 2)))
  }

  if (buyingTo > BUYMAX) {
    const diminishingExponent = 1 / 8
    const quadrillionCost = getCostInternal(base, BUYMAX, inAscensionChallenge15)
    const newCost = quadrillionCost.pow(Math.pow(buyingTo / BUYMAX, 1 / diminishingExponent))
    // Re-normalize after the in-place exponent/mantissa rewrite.
    const newExtra = newCost.exponent - Math.floor(newCost.exponent)
    newCost.exponent = Math.floor(newCost.exponent)
    newCost.mantissa *= Math.pow(10, newExtra)
    newCost.normalize()
    return Decimal.max(cost, newCost)
  }
  return cost
}

export function getParticleCost(
  buyingTo: number,
  input: GetParticleCostInput
): Decimal {
  const originalCost = ORIGINAL_COSTS[input.index - 1]
  return getCostInternal(originalCost, buyingTo, input.inAscensionChallenge15)
}

// Buy as many of the selected particle building as possible. Same two-path
// structure as buyMultiplier / buyAccelerator: high-end binary search above
// BUYMAX snaps the count without subtracting the resource; the normal path
// brackets the affordable count and walks the last few steps subtracting
// per-purchase.
export function buyParticleBuilding(
  state: ParticleBuildingsState,
  input: BuyParticleBuildingInput
): { state: ParticleBuildingsState; events: CoreEvent[] } {
  const events: CoreEvent[] = []
  const next: ParticleBuildingsState = {
    reincarnationPoints: new Decimal(state.reincarnationPoints),
    firstOwnedParticles: state.firstOwnedParticles,
    firstCostParticles: new Decimal(state.firstCostParticles),
    secondOwnedParticles: state.secondOwnedParticles,
    secondCostParticles: new Decimal(state.secondCostParticles),
    thirdOwnedParticles: state.thirdOwnedParticles,
    thirdCostParticles: new Decimal(state.thirdCostParticles),
    fourthOwnedParticles: state.fourthOwnedParticles,
    fourthCostParticles: new Decimal(state.fourthCostParticles),
    fifthOwnedParticles: state.fifthOwnedParticles,
    fifthCostParticles: new Decimal(state.fifthCostParticles)
  }
  const startingPoints = new Decimal(state.reincarnationPoints)
  const ownedKey: OwnedKey = OWNED_KEYS[input.index - 1]
  const costKey: CostKey = COST_KEYS[input.index - 1]
  const originalCost = ORIGINAL_COSTS[input.index - 1]
  const costInput: GetParticleCostInput = {
    index: input.index,
    inAscensionChallenge15: input.inAscensionChallenge15
  }

  const buyStart = next[ownedKey]

  if (buyStart >= BUYMAX) {
    const diminishingExponent = 1 / 8
    const log10Resource = Decimal.log10(next.reincarnationPoints)
    const log10QuadrillionCost = Decimal.log10(getCostInternal(originalCost, BUYMAX, input.inAscensionChallenge15))

    let hi = Math.floor(BUYMAX * Math.max(1, Math.pow(log10Resource / log10QuadrillionCost, diminishingExponent)))
    let lo = BUYMAX
    while (hi - lo > 0.5) {
      const mid = Math.floor(lo + (hi - lo) / 2)
      if (mid === lo || mid === hi) break
      if (!next.reincarnationPoints.gte(getParticleCost(mid, costInput))) {
        hi = mid
      } else {
        lo = mid
      }
    }
    const buyable = lo
    const thisCost = getParticleCost(buyable, costInput)

    next[ownedKey] = buyable
    next[costKey] = thisCost

    if (buyable > buyStart) {
      events.push({
        kind: 'particle-buildings-purchased',
        index: input.index,
        before: buyStart,
        after: buyable,
        spent: startingPoints.sub(next.reincarnationPoints)
      })
    }
    return { state: next, events }
  }

  // Start buying at the current amount bought + 1.
  const buydefault = buyStart + smallestInc(buyStart)
  let buyTo = buydefault

  let cashToBuy = getParticleCost(buyTo, costInput)
  while (next.reincarnationPoints.gte(cashToBuy)) {
    // Multiply target by 4 until cost just exceeds the available budget.
    buyTo = buyTo * 4
    cashToBuy = getParticleCost(buyTo, costInput)
  }
  let stepdown = Math.floor(buyTo / 8)
  while (stepdown >= smallestInc(buyTo)) {
    if (getParticleCost(buyTo - stepdown, costInput).lte(next.reincarnationPoints)) {
      stepdown = Math.floor(stepdown / 2)
    } else {
      buyTo = buyTo - Math.max(smallestInc(buyTo), stepdown)
    }
  }

  if (!input.autobuyer) {
    if (input.particlebuyamount + buyStart < buyTo) {
      buyTo = buyStart + input.particlebuyamount + smallestInc(buyStart + input.particlebuyamount)
    }
  }

  // Walk down 7 steps below the bracket top, then walk back up subtracting per-purchase.
  let buyFrom = Math.max(buyTo - 6 - smallestInc(buyTo), buydefault)
  let thisCost = getParticleCost(buyFrom, costInput)
  while (buyFrom <= buyTo && next.reincarnationPoints.gte(thisCost)) {
    next.reincarnationPoints = next.reincarnationPoints.sub(thisCost)
    next[ownedKey] = buyFrom
    buyFrom = buyFrom + smallestInc(buyFrom)
    thisCost = getParticleCost(buyFrom, costInput)
    next[costKey] = thisCost
  }

  if (next[ownedKey] > buyStart) {
    events.push({
      kind: 'particle-buildings-purchased',
      index: input.index,
      before: buyStart,
      after: next[ownedKey],
      spent: startingPoints.sub(next.reincarnationPoints)
    })
  }

  return { state: next, events }
}
