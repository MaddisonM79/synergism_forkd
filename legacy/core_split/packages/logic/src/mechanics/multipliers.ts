import { Decimal } from '../math/bignum'
import { smallestInc } from '../math/smallestInc'
import type { CoreEvent } from '../events/types'
import type { BuyAmount, MultiplierState } from '../state/schema'

export interface BuyMultiplierInput {
  /** True when the autobuyer is driving — bypasses the per-click coinbuyamount cap. */
  autobuyer: boolean
  /** Per-click purchase cap selected in the UI. */
  coinbuyamount: BuyAmount
  /** G.costDivisor at call time. */
  costDivisor: number
  /** CalcECC('transcend', player.challengecompletions[4]). */
  transcendECC: number
  /** player.currentChallenge.transcension === 4 */
  inTranscensionChallenge4: boolean
  /** player.currentChallenge.reincarnation === 8 */
  inReincarnationChallenge8: boolean
}

export type GetCostMultiplierInput = Pick<
  BuyMultiplierInput,
  'costDivisor' | 'transcendECC' | 'inTranscensionChallenge4' | 'inReincarnationChallenge8'
>

const BUYMAX = Math.pow(10, 15)

// Cost to purchase the `buyingTo`-th multiplier. Port of getCostMultiplier
// from packages/web_ui/src/Buy.ts. Same shape as getCostAccelerator with
// different curve constants (originalCost=1e4 vs 500, base growth 10^...
// instead of 4^..., factorial threshold 75 vs 125 with 2x ECC weighting).
export function getCostMultiplier(
  buyingTo: number,
  input: GetCostMultiplierInput
): Decimal {
  --buyingTo

  const originalCost = 1e4
  let cost = new Decimal(originalCost)
  cost = cost.times(Decimal.pow(10, buyingTo / input.costDivisor))

  const transcendBreak = 2 * input.transcendECC
  if (buyingTo > (75 + transcendBreak)) {
    const num = buyingTo - 75 - transcendBreak
    const factorialBit = new Decimal(num).factorial()
    const powBit = Decimal.pow(10, num)
    cost = cost.times(factorialBit.times(powBit))
  }

  if (buyingTo > (2000 + transcendBreak)) {
    const sumNum = buyingTo - 2000 - transcendBreak
    const sumBit = sumNum * (sumNum + 1) / 2
    cost = cost.times(Decimal.pow(2, sumBit))
  }

  if (input.inTranscensionChallenge4) {
    const sumBit = buyingTo * (buyingTo + 1) / 2
    cost = cost.times(Decimal.pow(10, sumBit))
  }

  if (input.inReincarnationChallenge8) {
    const sumBit = buyingTo * (buyingTo + 1) / 2
    cost = cost.times(Decimal.pow(1e50, sumBit))
  }

  if (buyingTo > BUYMAX) {
    const diminishingExponent = 1 / 8
    const quadrillionCost = getCostMultiplier(BUYMAX, input)
    const newCost = quadrillionCost.pow(Math.pow(buyingTo / BUYMAX, 1 / diminishingExponent))
    const newExtra = newCost.exponent - Math.floor(newCost.exponent)
    newCost.exponent = Math.floor(newCost.exponent)
    newCost.mantissa *= Math.pow(10, newExtra)
    newCost.normalize()
    return Decimal.max(cost, newCost)
  }
  return cost
}

// Buy as many multipliers as possible. Mirror of buyAccelerator — same
// two-path structure (high-end binary search vs normal bracket/refine/walk),
// different field names and cost curve. Past buymax the binary-search path
// snaps state without subtracting coins, intentionally matching the original.
export function buyMultiplier(
  state: MultiplierState,
  input: BuyMultiplierInput
): { state: MultiplierState; events: CoreEvent[] } {
  const events: CoreEvent[] = []
  const next: MultiplierState = {
    multiplierBought: state.multiplierBought,
    multiplierCost: new Decimal(state.multiplierCost),
    coins: new Decimal(state.coins),
    prestigenomultiplier: state.prestigenomultiplier,
    transcendnomultiplier: state.transcendnomultiplier,
    reincarnatenomultiplier: state.reincarnatenomultiplier
  }
  const startingCoins = new Decimal(state.coins)
  const buyStart = next.multiplierBought

  if (buyStart >= BUYMAX) {
    const diminishingExponent = 1 / 8
    const log10Resource = Decimal.log10(next.coins)
    const log10QuadrillionCost = Decimal.log10(getCostMultiplier(BUYMAX, input))

    let hi = Math.floor(
      BUYMAX * Math.max(1, Math.pow(log10Resource / log10QuadrillionCost, diminishingExponent))
    )
    let lo = BUYMAX
    while (hi - lo > 0.5) {
      const mid = Math.floor(lo + (hi - lo) / 2)
      if (mid === lo || mid === hi) break
      if (!next.coins.gte(getCostMultiplier(mid, input))) {
        hi = mid
      } else {
        lo = mid
      }
    }
    const buyable = lo
    next.multiplierBought = buyable
    next.multiplierCost = getCostMultiplier(buyable, input)
    if (next.multiplierBought > 0) {
      next.prestigenomultiplier = false
      next.transcendnomultiplier = false
      next.reincarnatenomultiplier = false
    }
    if (next.multiplierBought > buyStart) {
      events.push({
        kind: 'multipliers-purchased',
        before: buyStart,
        after: next.multiplierBought,
        spent: startingCoins.sub(next.coins)
      })
    }
    return { state: next, events }
  }

  const buydefault = buyStart + smallestInc(buyStart)
  let buyTo = buydefault

  let cashToBuy = getCostMultiplier(buyTo, input)
  while (next.coins.gte(cashToBuy)) {
    buyTo = buyTo * 4
    cashToBuy = getCostMultiplier(buyTo, input)
  }
  let stepdown = Math.floor(buyTo / 8)
  while (stepdown >= smallestInc(buyTo)) {
    if (getCostMultiplier(buyTo - stepdown, input).lte(next.coins)) {
      stepdown = Math.floor(stepdown / 2)
    } else {
      buyTo = buyTo - Math.max(smallestInc(buyTo), stepdown)
    }
  }

  if (!input.autobuyer) {
    if (next.multiplierBought + input.coinbuyamount < buyTo) {
      buyTo = next.multiplierBought + input.coinbuyamount
    }
  }

  let buyFrom = Math.max(buyTo - 6 - smallestInc(buyTo), buydefault)
  let thisCost = getCostMultiplier(buyFrom, input)
  while (buyFrom <= buyTo && next.coins.gte(thisCost)) {
    if (buyFrom >= BUYMAX) buyFrom = BUYMAX
    next.coins = next.coins.sub(thisCost)
    next.multiplierBought = buyFrom
    buyFrom = buyFrom + smallestInc(buyFrom)
    thisCost = getCostMultiplier(buyFrom, input)
    next.multiplierCost = thisCost
    if (buyFrom >= BUYMAX) break
  }

  if (next.multiplierBought > 0) {
    next.prestigenomultiplier = false
    next.transcendnomultiplier = false
    next.reincarnatenomultiplier = false
  }

  if (next.multiplierBought > buyStart) {
    events.push({
      kind: 'multipliers-purchased',
      before: buyStart,
      after: next.multiplierBought,
      spent: startingCoins.sub(next.coins)
    })
  }

  return { state: next, events }
}
