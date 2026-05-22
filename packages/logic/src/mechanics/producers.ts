import type { DecimalSource } from '../math/bignum'
import { Decimal } from '../math/bignum'

// Producer family: Coin / Diamond / Mythos / Particle buildings. Each
// family has 5 positions (first..fifth) and a cost curve parameterized by
// position-derived `num`. Logic owns the pure cost formula; buy loops
// stay in packages/web_ui until they're migrated next.

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
