import type { DecimalSource } from '../math/bignum'
import { Decimal } from '../math/bignum'
import { smallestInc } from '../math/smallestInc'
import type { CoreEvent } from '../events/types'
import type { ProducerFamilyState } from '../state/schema'

// Producer family: Coin / Diamond / Mythos / Particle buildings. Each
// family has 5 positions (first..fifth) and a cost curve parameterized by
// position-derived `num`. Logic owns the pure cost formula AND the buyMax
// purchase loop; the manual-click buyProducer loop and the click-handler
// surface remain in packages/web_ui pending migration.

export type ProducerType = 'Coin' | 'Diamonds' | 'Mythos' | 'Particles'
export type ProducerIndex = 1 | 2 | 3 | 4 | 5

export interface GetProducerCostInput {
  /** G.costDivisor at call time (= getReductionValue() in the UI). */
  costDivisor: number
  /** player.currentChallenge.transcension === 4 */
  inTranscensionChallenge4: boolean
  /** player.currentChallenge.reincarnation === 8 */
  inReincarnationChallenge8: boolean
  /** player.currentChallenge.reincarnation === 10 */
  inReincarnationChallenge10: boolean
  /** player.challengecompletions[4] */
  challengecompletions4: number
  /** player.challengecompletions[8] */
  challengecompletions8: number
}

const BUYMAX = Math.pow(10, 15)

// Stirling-approximation factorial helpers. Operating on the exponent of a
// log10 representation avoids constructing a full Decimal per factorial —
// the producer-cost formula calls these in hot loops.
const mantissaFactorialPartExtra = Math.log10(2 * Math.PI)
const exponentFactorialPartExtra = Math.log10(Math.E)

const factorialByExponent = (fact: number): number => {
  if (++fact === 0) {
    return 0
  }
  return ((Math.log10(fact * Math.sqrt(fact * Math.sinh(1 / fact) + 1 / (810 * Math.pow(fact, 6))))
    - exponentFactorialPartExtra) * fact) + ((mantissaFactorialPartExtra - Math.log10(fact)) / 2)
}

const fact100exponent = Math.log10(9.332621544394e+157)

// 16 digits of precision threshold for adding 1s into the cost mantissa.
// log10(1.25) * n = log10(x) + 16 => xn ~= 188.582 => x ~= 188.582/n.
// Below this threshold the +1 corrections matter; above it they round off.
const precision16_loss_addition_of_ones = 188.582

// Precomputed log10 table for the values the cost formula references in
// hot paths. Generated once at module load — every entry is constant.
const known_log10s = (() => {
  const needed: number[] = [1.03, 1.25]
  const nums = [1, 2, 3, 4, 5, 6, 10, 15]
  for (const num of nums) {
    needed.push(100 + (100 * num))
    needed.push(10 + (10 * num))
  }
  // Reincarnation-challenge-8 completion amounts span integer halves up to ~500.
  const chalcompletions = 1000
  for (let i = 0; i < chalcompletions; ++i) {
    needed.push(1 + (i / 2))
  }
  const obj: Record<number, number> = {}
  for (const need of needed) {
    obj[need] ??= Math.log10(need)
  }
  return obj
})()

const coinBuildingCosts = [100, 1000, 2e4, 4e5, 8e6] as const
const diamondBuildingCosts = [100, 1e5, 1e15, 1e40, 1e100] as const
const mythosAndParticleBuildingCosts = [1, 1e2, 1e4, 1e8, 1e16] as const

const getOriginalCostAndNum = (
  index: ProducerIndex,
  type: ProducerType
): readonly [DecimalSource, number] => {
  const originalCostArray = type === 'Coin'
    ? coinBuildingCosts
    : type === 'Diamonds'
    ? diamondBuildingCosts
    : mythosAndParticleBuildingCosts
  const num = type === 'Coin' ? index : index * (index + 1) / 2
  const originalCost = originalCostArray[index - 1]
  return [originalCost, num] as const
}

const getCostInternal = (
  originalCost: DecimalSource,
  buyingTo: number,
  type: ProducerType,
  num: number,
  input: GetProducerCostInput
): Decimal => {
  const r = input.costDivisor
  // Off-by-one: formula is 0-indexed, callers pass 1-indexed.
  --buyingTo
  const cost = new Decimal(originalCost)
  // Accounts for the cumulative `* 1.25^num` buyingTo times.
  let mlog10125 = num * buyingTo
  // Accounts for the +1 corrections — only mattering below the precision floor.
  if (buyingTo < precision16_loss_addition_of_ones / num) {
    cost.mantissa += buyingTo / Math.pow(10, cost.exponent)
  }
  let fastFactMultBuyTo = 0
  let fr = Math.floor(r * 1000)
  if (buyingTo >= r * 1000) {
    ++fastFactMultBuyTo
    cost.exponent -= factorialByExponent(fr)
    cost.exponent += (-3 + Math.log10(1 + (num / 2))) * (buyingTo - fr)
  }

  fr = Math.floor(r * 5000)
  if (buyingTo >= r * 5000) {
    ++fastFactMultBuyTo
    cost.exponent -= factorialByExponent(fr)
    cost.exponent += ((known_log10s[10 + num * 10] + 1) * (buyingTo - fr - 1)) + 1
  }

  fr = Math.floor(r * 20000)
  if (buyingTo >= r * 20000) {
    fastFactMultBuyTo += 3
    cost.exponent -= factorialByExponent(fr) * 3
    cost.exponent += (known_log10s[100 + (100 * num)] + 5) * (buyingTo - fr)
  }

  fr = Math.floor(r * 250000)
  if (buyingTo >= r * 250000) {
    // 1.03^x * 1.03^y = 1.03^(x+y) — sum the power as a triangle number.
    cost.exponent += Math.log10(1.03) * (buyingTo - fr) * ((buyingTo - fr + 1) / 2)
  }
  // Apply the factorial corrections accumulated across the r-bracket regions.
  cost.exponent += factorialByExponent(buyingTo) * fastFactMultBuyTo

  // Challenge-driven mantissa amplifiers — Coin / Diamonds in C4 transcension
  // and C10 reincarnation, separately accumulated.
  let fastFactMultBuyTo100 = 0
  if (input.inTranscensionChallenge4 && (type === 'Coin' || type === 'Diamonds')) {
    ++fastFactMultBuyTo100
    if (buyingTo >= (1000 - (10 * input.challengecompletions4))) {
      mlog10125 += buyingTo * (buyingTo + 1) / 2
    }
  }
  if (input.inReincarnationChallenge10 && (type === 'Coin' || type === 'Diamonds')) {
    ++fastFactMultBuyTo100
    if (buyingTo >= (r * 25000)) {
      mlog10125 += buyingTo * (buyingTo + 1) / 2
    }
  }
  cost.exponent += fastFactMultBuyTo100
    * ((factorialByExponent(buyingTo + 100) - fact100exponent + (2 * buyingTo))
      * (1.25 + (input.challengecompletions4 / 4)))
  cost.exponent += known_log10s[1.25] * mlog10125

  // Reincarnation Challenge 8 — affects Coin / Diamonds / Mythos at high counts.
  fr = Math.floor(r * 1000 * input.challengecompletions8)
  if (
    input.inReincarnationChallenge8
    && (type === 'Coin' || type === 'Diamonds' || type === 'Mythos')
    && buyingTo >= (1000 * input.challengecompletions8 * r)
  ) {
    cost.exponent +=
      ((known_log10s[2] * ((buyingTo - fr + 1) / 2)) - known_log10s[1 + (input.challengecompletions8 / 2)])
      * (buyingTo - fr)
  }

  // Re-normalize the mantissa-exponent split after all the in-place writes.
  const extra = cost.exponent - Math.floor(cost.exponent)
  cost.exponent = Math.floor(cost.exponent)
  cost.mantissa *= Math.pow(10, extra)
  cost.normalize()

  if (buyingTo > BUYMAX) {
    const diminishingExponent = 1 / 8
    // Off-by-one in the recursion is intentional: BUYMAX here is the
    // pre-decrement value, the recursive call decrements again.
    const quadrillionCost = getCostInternal(originalCost, BUYMAX, type, num, input)
    const newCost = quadrillionCost.pow(Math.pow(buyingTo / BUYMAX, 1 / diminishingExponent))
    const newExtra = newCost.exponent - Math.floor(newCost.exponent)
    newCost.exponent = Math.floor(newCost.exponent)
    newCost.mantissa *= Math.pow(10, newExtra)
    newCost.normalize()
    return Decimal.max(cost, newCost)
  }
  return cost
}

// Public entry point. Looks up the (originalCost, num) pair for the given
// (index, type) and dispatches to the internal cost formula.
export function getProducerCost(
  index: ProducerIndex,
  type: ProducerType,
  buyingTo: number,
  input: GetProducerCostInput
): Decimal {
  const [originalCost, num] = getOriginalCostAndNum(index, type)
  return getCostInternal(originalCost, buyingTo, type, num, input)
}

// ─── buyMax ────────────────────────────────────────────────────────────────

export interface BuyMaxInput {
  index: ProducerIndex
  type: ProducerType
  /** Inputs threaded into getProducerCost for every cost query in the loop. */
  costInput: GetProducerCostInput
}

// Position-keyed accessors. Switch (or an if-chain) is required because
// ProducerFamilyState's field types vary (number vs Decimal) — keyed lookup
// via a string table can't narrow.
function readOwned(state: ProducerFamilyState, index: ProducerIndex): number {
  if (index === 1) return state.firstOwned
  if (index === 2) return state.secondOwned
  if (index === 3) return state.thirdOwned
  if (index === 4) return state.fourthOwned
  return state.fifthOwned
}
function readCost(state: ProducerFamilyState, index: ProducerIndex): Decimal {
  if (index === 1) return state.firstCost
  if (index === 2) return state.secondCost
  if (index === 3) return state.thirdCost
  if (index === 4) return state.fourthCost
  return state.fifthCost
}
function writeOwned(state: ProducerFamilyState, index: ProducerIndex, value: number): void {
  if (index === 1) state.firstOwned = value
  else if (index === 2) state.secondOwned = value
  else if (index === 3) state.thirdOwned = value
  else if (index === 4) state.fourthOwned = value
  else state.fifthOwned = value
}
function writeCost(state: ProducerFamilyState, index: ProducerIndex, value: Decimal): void {
  if (index === 1) state.firstCost = value
  else if (index === 2) state.secondCost = value
  else if (index === 3) state.thirdCost = value
  else if (index === 4) state.fourthCost = value
  else state.fifthCost = value
}

// Coin/exponent ceiling guard. Mirrors the original buyMax's `coinmax = 1e99`
// degenerate-case check — once the next cost's exponent crosses this we bail
// rather than continue doubling buyInc into infinity.
const COIN_EXPONENT_CEILING = 1e99
const BUYMAX_PRODUCER = Math.pow(10, 15)

/**
 * Buy as many of the selected producer (5 positions × 4 families) as the
 * available resource allows. Same two-path structure as buyMultiplier /
 * buyAccelerator: high-end binary search above BUYMAX_PRODUCER snaps the
 * count without subtracting the resource; the normal path brackets the
 * affordable count and walks the last few steps subtracting per-purchase.
 */
export function buyMax(
  state: ProducerFamilyState,
  input: BuyMaxInput
): { state: ProducerFamilyState; events: CoreEvent[] } {
  const events: CoreEvent[] = []
  const next: ProducerFamilyState = {
    resource: new Decimal(state.resource),
    firstOwned: state.firstOwned,
    firstCost: new Decimal(state.firstCost),
    secondOwned: state.secondOwned,
    secondCost: new Decimal(state.secondCost),
    thirdOwned: state.thirdOwned,
    thirdCost: new Decimal(state.thirdCost),
    fourthOwned: state.fourthOwned,
    fourthCost: new Decimal(state.fourthCost),
    fifthOwned: state.fifthOwned,
    fifthCost: new Decimal(state.fifthCost)
  }
  const startingResource = new Decimal(state.resource)
  const buyStart = readOwned(next, input.index)

  const cost = (buyingTo: number): Decimal => getProducerCost(input.index, input.type, buyingTo, input.costInput)

  if (buyStart >= BUYMAX_PRODUCER) {
    const diminishingExponent = 1 / 8
    const log10Resource = Decimal.log10(next.resource)
    const log10QuadrillionCost = Decimal.log10(cost(BUYMAX_PRODUCER))

    let hi = Math.floor(
      BUYMAX_PRODUCER * Math.max(1, Math.pow(log10Resource / log10QuadrillionCost, diminishingExponent))
    )
    let lo = BUYMAX_PRODUCER
    while (hi - lo > 0.5) {
      const mid = Math.floor(lo + (hi - lo) / 2)
      if (mid === lo || mid === hi) break
      if (!next.resource.gte(cost(mid))) {
        hi = mid
      } else {
        lo = mid
      }
    }
    const buyable = lo
    writeOwned(next, input.index, buyable)
    writeCost(next, input.index, cost(buyable))
    if (buyable > buyStart) {
      events.push({
        kind: 'producers-purchased',
        type: input.type,
        index: input.index,
        before: buyStart,
        after: buyable,
        spent: startingResource.sub(next.resource)
      })
    }
    return { state: next, events }
  }

  // Normal path: exponential bracket, then refine, then walk the tail.
  const buydefault = buyStart + smallestInc(buyStart)
  let buyInc = 1

  let cashToBuy = cost(buyStart + buyInc)

  // Degenerate case: cost already past the exponent ceiling or unaffordable.
  if (cashToBuy.exponent >= COIN_EXPONENT_CEILING || !next.resource.gte(cashToBuy)) {
    return { state: next, events }
  }

  while (cashToBuy.exponent < COIN_EXPONENT_CEILING && next.resource.gte(cashToBuy)) {
    // Multiply target by 4 until cost just exceeds the available budget.
    buyInc = buyInc * 4
    cashToBuy = cost(buyStart + buyInc)
  }
  let stepdown = Math.floor(buyInc / 8)
  while (stepdown >= smallestInc(buyInc)) {
    if (cost(buyStart + buyInc - stepdown).lte(next.resource)) {
      stepdown = Math.floor(stepdown / 2)
    } else {
      buyInc = buyInc - Math.max(smallestInc(buyInc), stepdown)
    }
  }

  // Snap to BUYMAX cap before the walk. The original commentary calls this
  // the "infamous autobuyer bug" fix — past BUYMAX_PRODUCER we just write the
  // snapped state and stop.
  if (buyStart + buyInc >= BUYMAX_PRODUCER) {
    writeOwned(next, input.index, BUYMAX_PRODUCER)
    writeCost(next, input.index, cost(BUYMAX_PRODUCER))
    events.push({
      kind: 'producers-purchased',
      type: input.type,
      index: input.index,
      before: buyStart,
      after: BUYMAX_PRODUCER,
      spent: startingResource.sub(next.resource)
    })
    return { state: next, events }
  }

  let buyFrom = Math.max(buyStart + buyInc - 6 - smallestInc(buyInc), buydefault)
  let thisCost = cost(buyFrom)
  while (buyFrom <= buyStart + buyInc && next.resource.gte(thisCost)) {
    next.resource = next.resource.sub(thisCost)
    writeOwned(next, input.index, buyFrom)
    buyFrom = buyFrom + smallestInc(buyFrom)
    thisCost = cost(buyFrom)
    writeCost(next, input.index, thisCost)
  }

  if (readOwned(next, input.index) > buyStart) {
    events.push({
      kind: 'producers-purchased',
      type: input.type,
      index: input.index,
      before: buyStart,
      after: readOwned(next, input.index),
      spent: startingResource.sub(next.resource)
    })
  }

  return { state: next, events }
}

// ─── buyProducer (manual-click loop) ───────────────────────────────────────

export interface BuyProducerInput {
  index: ProducerIndex
  type: ProducerType
  /** True when the autobuyer is driving — caps the loop at 500 iterations. */
  autobuyer: boolean
  /** Per-click cap from player.{coin,crystal,mythos,particle}buyamount. */
  buyamount: number
  /**
   * Reduction value — `getReductionValue()` in web_ui. Shifts the per-step
   * exponent thresholds (1000*r, 5000*r, 20000*r, 250000*r) and the
   * challenge-8 amplifier threshold. Combine of:
   *   1 + getRuneEffects('thrift', 'costDelay')
   *     + (researches[56..60] sum)/200
   *     + CalcECC('transcend', cc4)/200
   *     + getAntUpgradeEffect(AntUpgrades.BuildingCostScale).buildingCostScale
   */
  r: number
  inTranscensionChallenge4: boolean
  inReincarnationChallenge8: boolean
  challengecompletions4: number
  challengecompletions8: number
}

// `num` derivation: Coin uses the position index directly, every other family
// uses the triangle number index*(index+1)/2. Mirrors the call-site convention
// in EventListeners.ts and getOriginalCostAndNum above.
function numFor(index: ProducerIndex, type: ProducerType): number {
  return type === 'Coin' ? index : index * (index + 1) / 2
}

/**
 * Manual-click producer purchase loop. Buys one producer per iteration,
 * subtracts current cost, then applies the per-iteration cost multiplier
 * ladder (×1.25^num, +1 mantissa adjustment, threshold amplifiers at
 * 1000/5000/20000/250000 *r, challenge-4 transcension, challenge-8
 * reincarnation). Loop caps at `buyamount` (or 500 when the autobuyer is
 * driving).
 */
export function buyProducer(
  state: ProducerFamilyState,
  input: BuyProducerInput
): { state: ProducerFamilyState; events: CoreEvent[] } {
  const events: CoreEvent[] = []
  const next: ProducerFamilyState = {
    resource: new Decimal(state.resource),
    firstOwned: state.firstOwned,
    firstCost: new Decimal(state.firstCost),
    secondOwned: state.secondOwned,
    secondCost: new Decimal(state.secondCost),
    thirdOwned: state.thirdOwned,
    thirdCost: new Decimal(state.thirdCost),
    fourthOwned: state.fourthOwned,
    fourthCost: new Decimal(state.fourthCost),
    fifthOwned: state.fifthOwned,
    fifthCost: new Decimal(state.fifthCost)
  }
  const startingResource = new Decimal(state.resource)
  const buyStart = readOwned(next, input.index)
  const num = numFor(input.index, input.type)
  const buythisamount = input.autobuyer ? 500 : input.buyamount

  let t = 0
  while (
    next.resource.gte(readCost(next, input.index))
    && t < buythisamount
    && readOwned(next, input.index) < Number.MAX_SAFE_INTEGER
  ) {
    next.resource = next.resource.sub(readCost(next, input.index))
    writeOwned(next, input.index, readOwned(next, input.index) + 1)
    let cost = readCost(next, input.index).times(Decimal.pow(1.25, num))
    cost = cost.add(1)
    const owned = readOwned(next, input.index)

    // Per-step exponent threshold ladder. Each rung adds a one-off cost
    // multiplier once the cumulative count crosses the (threshold * r) mark.
    if (owned >= 1000 * input.r) {
      cost = cost.times(owned).dividedBy(1000).times(1 + num / 2)
    }
    if (owned >= 5000 * input.r) {
      cost = cost.times(owned).times(10).times(10 + num * 10)
    }
    if (owned >= 20000 * input.r) {
      cost = cost.times(Decimal.pow(owned, 3)).times(100000).times(100 + num * 100)
    }
    if (owned >= 250000 * input.r) {
      cost = cost.times(Decimal.pow(1.03, owned - 250000 * input.r))
    }

    // Challenge-4 (transcension) — amplifies Coin / Diamonds.
    if (input.inTranscensionChallenge4 && (input.type === 'Coin' || input.type === 'Diamonds')) {
      cost = cost.times(
        Math.pow(100 * owned + 10000, 1.25 + 1 / 4 * input.challengecompletions4)
      )
      if (owned >= 1000 - 10 * input.challengecompletions4) {
        cost = cost.times(Decimal.pow(1.25, owned))
      }
    }

    // Challenge-8 (reincarnation) — amplifies Coin / Diamonds / Mythos at high counts.
    if (
      input.inReincarnationChallenge8
      && (input.type === 'Coin' || input.type === 'Diamonds' || input.type === 'Mythos')
      && owned >= 1000 * input.challengecompletions8 * input.r
    ) {
      cost = cost.times(
        Decimal.pow(
          2,
          (owned - 1000 * input.challengecompletions8 * input.r)
            / (1 + input.challengecompletions8 / 2)
        )
      )
    }

    writeCost(next, input.index, cost)
    t += 1
  }

  if (readOwned(next, input.index) > buyStart) {
    events.push({
      kind: 'producers-purchased',
      type: input.type,
      index: input.index,
      before: buyStart,
      after: readOwned(next, input.index),
      spent: startingResource.sub(next.resource)
    })
  }

  return { state: next, events }
}
