import { Decimal } from '../math/bignum'
import { smallestInc } from '../math/smallestInc'
import type { CoreEvent } from '../events/types'
import type { AcceleratorState, BuyAmount } from '../state/schema'

export interface BuyAcceleratorInput {
  /** True when the autobuyer is driving — bypasses the per-click coinbuyamount cap. */
  autobuyer: boolean
  /** Per-click purchase cap selected in the UI. */
  coinbuyamount: BuyAmount
  /** G.costDivisor at call time (computed in the UI tick from runes/researches/ant upgrades). */
  costDivisor: number
  /** CalcECC('transcend', player.challengecompletions[4]) — Eternal Challenge transcend completions. */
  transcendECC: number
  /** player.currentChallenge.transcension === 4 */
  inTranscensionChallenge4: boolean
  /** player.currentChallenge.reincarnation === 8 */
  inReincarnationChallenge8: boolean
}

export type GetCostAcceleratorInput = Pick<
  BuyAcceleratorInput,
  'costDivisor' | 'transcendECC' | 'inTranscensionChallenge4' | 'inReincarnationChallenge8'
>

const BUYMAX = Math.pow(10, 15)

// Cost to purchase the `buyingTo`-th accelerator. Port of getCostAccelerator
// from packages/web_ui/src/Buy.ts (originally a const, with all `player.*` /
// `G.*` reads hoisted into the explicit `input` parameter for portability).
export function getCostAccelerator(
  buyingTo: number,
  input: GetCostAcceleratorInput
): Decimal {
  --buyingTo

  const originalCost = 500
  let cost = new Decimal(originalCost)

  cost = cost.times(Decimal.pow(4 / input.costDivisor, buyingTo))

  const transcendBreak = 5 * input.transcendECC
  if (buyingTo > (125 + transcendBreak)) {
    const num = buyingTo - 125 - transcendBreak
    const factorialBit = new Decimal(num).factorial()
    const multBit = Decimal.pow(4, num)
    cost = cost.times(multBit.times(factorialBit))
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
    const quadrillionCost = getCostAccelerator(BUYMAX, input)
    const newCost = quadrillionCost.pow(Math.pow(buyingTo / BUYMAX, 1 / diminishingExponent))
    // break_infinity's pow returned a fresh instance; safe to massage in-place.
    const newExtra = newCost.exponent - Math.floor(newCost.exponent)
    newCost.exponent = Math.floor(newCost.exponent)
    newCost.mantissa *= Math.pow(10, newExtra)
    newCost.normalize()
    return Decimal.max(cost, newCost)
  }
  return cost
}

// Buy as many accelerators as possible given the current coin balance and
// per-click cap. Port of buyAccelerator from packages/web_ui/src/Buy.ts:72,
// with all `player.*` / `G.*` reads hoisted into the explicit input.
//
// Two paths:
//   - High-end (acceleratorBought >= 1e15): binary-search for the largest
//     affordable count and snap the state to it (no per-step coin subtraction
//     — the cost function in that range diminishes so aggressively that
//     post-hoc cost accounting matches the buy).
//   - Normal: bracket the target with a 4x doubling search, refine with
//     stepdown, then walk forward subtracting cost each step.
export function buyAccelerator(
  state: AcceleratorState,
  input: BuyAcceleratorInput
): { state: AcceleratorState; events: CoreEvent[] } {
  const events: CoreEvent[] = []
  const next: AcceleratorState = {
    acceleratorBought: state.acceleratorBought,
    acceleratorCost: new Decimal(state.acceleratorCost),
    coins: new Decimal(state.coins),
    prestigenoaccelerator: state.prestigenoaccelerator,
    transcendnoaccelerator: state.transcendnoaccelerator,
    reincarnatenoaccelerator: state.reincarnatenoaccelerator
  }
  const startingCoins = new Decimal(state.coins)
  const buyStart = next.acceleratorBought

  // High-end binary search path
  if (buyStart >= BUYMAX) {
    const diminishingExponent = 1 / 8
    const log10Resource = Decimal.log10(next.coins)
    const log10QuadrillionCost = Decimal.log10(getCostAccelerator(BUYMAX, input))

    let hi = Math.floor(
      BUYMAX * Math.max(1, Math.pow(log10Resource / log10QuadrillionCost, diminishingExponent))
    )
    let lo = BUYMAX
    while (hi - lo > 0.5) {
      const mid = Math.floor(lo + (hi - lo) / 2)
      if (mid === lo || mid === hi) break
      if (!next.coins.gte(getCostAccelerator(mid, input))) {
        hi = mid
      } else {
        lo = mid
      }
    }
    const buyable = lo
    next.acceleratorBought = buyable
    next.acceleratorCost = getCostAccelerator(buyable, input)
    if (next.acceleratorBought > 0) {
      next.prestigenoaccelerator = false
      next.transcendnoaccelerator = false
      next.reincarnatenoaccelerator = false
    }
    if (next.acceleratorBought > buyStart) {
      events.push({
        kind: 'accelerators-purchased',
        before: buyStart,
        after: next.acceleratorBought,
        spent: startingCoins.sub(next.coins)
      })
    }
    return { state: next, events }
  }

  // Normal path: bracket with 4x doubling, refine with stepdown, walk forward.
  const buydefault = buyStart + smallestInc(buyStart)
  let buyTo = buydefault

  let cashToBuy = getCostAccelerator(buyTo, input)
  while (next.coins.gte(cashToBuy)) {
    buyTo = buyTo * 4
    cashToBuy = getCostAccelerator(buyTo, input)
  }
  let stepdown = Math.floor(buyTo / 8)
  while (stepdown >= smallestInc(buyTo)) {
    if (getCostAccelerator(buyTo - stepdown, input).lte(next.coins)) {
      stepdown = Math.floor(stepdown / 2)
    } else {
      buyTo = buyTo - Math.max(smallestInc(buyTo), stepdown)
    }
  }

  // Per-click cap (only when not autobuying).
  if (!input.autobuyer) {
    if (next.acceleratorBought + input.coinbuyamount < buyTo) {
      buyTo = next.acceleratorBought + input.coinbuyamount
    }
  }

  let buyFrom = Math.max(buyTo - 6 - smallestInc(buyTo), buydefault)
  let thisCost = getCostAccelerator(buyFrom, input)
  while (buyFrom <= buyTo && next.coins.gte(thisCost)) {
    if (buyFrom >= BUYMAX) buyFrom = BUYMAX
    next.coins = next.coins.sub(thisCost)
    next.acceleratorBought = buyFrom
    buyFrom = buyFrom + smallestInc(buyFrom)
    thisCost = getCostAccelerator(buyFrom, input)
    next.acceleratorCost = thisCost
    if (buyFrom >= BUYMAX) break
  }

  if (next.acceleratorBought > 0) {
    next.prestigenoaccelerator = false
    next.transcendnoaccelerator = false
    next.reincarnatenoaccelerator = false
  }

  if (next.acceleratorBought > buyStart) {
    events.push({
      kind: 'accelerators-purchased',
      before: buyStart,
      after: next.acceleratorBought,
      spent: startingCoins.sub(next.coins)
    })
  }

  return { state: next, events }
}
