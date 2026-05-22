// Parity test for the producer buyMax loop.
//
// Pre-migration source: packages/web_ui/src/Buy.ts at commit 1e66c1bf~1.
// The OLD function below transcribes that loop with `player.*` / `G.ordinals`
// hoisted into an explicit ProducerFamilyState parameter. getProducerCost and
// smallestInc are imported from the logic package (both were migrated earlier
// and are themselves parity-verified by their existing unit tests). This
// isolates the buyMax loop structure as the thing under test.

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import { smallestInc } from '../../src/math/smallestInc'
import {
  buyMax as newBuyMax,
  getProducerCost,
  type GetProducerCostInput,
  type ProducerIndex,
  type ProducerType
} from '../../src/mechanics/producers'
import type { ProducerFamilyState } from '../../src/state/schema'

// OLD buyMax modeled as a pure transform on ProducerFamilyState. Each
// player[posOwnedType] / player[posCostType] / player[tag] mutation maps to
// the corresponding owned/cost/resource field on the cloned state.
const POSITIONS = ['first', 'second', 'third', 'fourth', 'fifth'] as const

const readOwnedOld = (state: ProducerFamilyState, index: ProducerIndex): number => {
  const pos = POSITIONS[index - 1]
  if (pos === 'first') return state.firstOwned
  if (pos === 'second') return state.secondOwned
  if (pos === 'third') return state.thirdOwned
  if (pos === 'fourth') return state.fourthOwned
  return state.fifthOwned
}
const writeOwnedOld = (state: ProducerFamilyState, index: ProducerIndex, v: number): void => {
  const pos = POSITIONS[index - 1]
  if (pos === 'first') state.firstOwned = v
  else if (pos === 'second') state.secondOwned = v
  else if (pos === 'third') state.thirdOwned = v
  else if (pos === 'fourth') state.fourthOwned = v
  else state.fifthOwned = v
}
const writeCostOld = (state: ProducerFamilyState, index: ProducerIndex, v: Decimal): void => {
  const pos = POSITIONS[index - 1]
  if (pos === 'first') state.firstCost = v
  else if (pos === 'second') state.secondCost = v
  else if (pos === 'third') state.thirdCost = v
  else if (pos === 'fourth') state.fourthCost = v
  else state.fifthCost = v
}

const applyOldBuyMax = (
  state: ProducerFamilyState,
  index: ProducerIndex,
  type: ProducerType,
  costInput: GetProducerCostInput
): ProducerFamilyState => {
  const buymax = Math.pow(10, 15)
  const coinmax = 1e99
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

  const buyStart = readOwnedOld(next, index)
  if (buyStart >= buymax) {
    const diminishingExponent = 1 / 8

    const log10Resource = Decimal.log10(next.resource)
    const log10QuadrillionCost = Decimal.log10(getProducerCost(index, type, buymax, costInput))

    let hi = Math.floor(buymax * Math.max(1, Math.pow(log10Resource / log10QuadrillionCost, diminishingExponent)))
    let lo = buymax
    while (hi - lo > 0.5) {
      const mid = Math.floor(lo + (hi - lo) / 2)
      if (mid === lo || mid === hi) break
      if (!next.resource.gte(getProducerCost(index, type, mid, costInput))) {
        hi = mid
      } else {
        lo = mid
      }
    }
    const buyable = lo
    const thisCost = getProducerCost(index, type, buyable, costInput)

    writeOwnedOld(next, index, buyable)
    writeCostOld(next, index, thisCost)
    return next
  }

  const buydefault = buyStart + smallestInc(buyStart)
  let buyInc = 1

  let cashToBuy = getProducerCost(index, type, buyStart + buyInc, costInput)

  if (cashToBuy.exponent >= coinmax || !next.resource.gte(cashToBuy)) {
    return next
  }

  while (cashToBuy.exponent < coinmax && next.resource.gte(cashToBuy)) {
    buyInc = buyInc * 4
    cashToBuy = getProducerCost(index, type, buyStart + buyInc, costInput)
  }
  let stepdown = Math.floor(buyInc / 8)
  while (stepdown >= smallestInc(buyInc)) {
    if (getProducerCost(index, type, buyStart + buyInc - stepdown, costInput).lte(next.resource)) {
      stepdown = Math.floor(stepdown / 2)
    } else {
      buyInc = buyInc - Math.max(smallestInc(buyInc), stepdown)
    }
  }

  if (buyStart + buyInc >= buymax) {
    writeOwnedOld(next, index, buymax)
    writeCostOld(next, index, getProducerCost(index, type, buymax, costInput))
    return next
  }

  let buyFrom = Math.max(buyStart + buyInc - 6 - smallestInc(buyInc), buydefault)
  let thisCost = getProducerCost(index, type, buyFrom, costInput)
  while (buyFrom <= buyStart + buyInc && next.resource.gte(thisCost)) {
    next.resource = next.resource.sub(thisCost)
    writeOwnedOld(next, index, buyFrom)
    buyFrom = buyFrom + smallestInc(buyFrom)
    thisCost = getProducerCost(index, type, buyFrom, costInput)
    writeCostOld(next, index, thisCost)
  }

  return next
}

// ─── Helpers / fixtures ────────────────────────────────────────────────────

const baseCostInput: GetProducerCostInput = {
  costDivisor: 1,
  inTranscensionChallenge4: false,
  inReincarnationChallenge8: false,
  inReincarnationChallenge10: false,
  challengecompletions4: 0,
  challengecompletions8: 0
}

// Strict deep-state equality with tight Decimal tolerance.
const expectStatesEqual = (a: ProducerFamilyState, b: ProducerFamilyState): void => {
  const rel = 1e-12
  const closeEnough = (x: Decimal, y: Decimal): boolean => {
    if (x.eq(y)) return true
    const diff = x.minus(y).abs()
    const scale = Decimal.max(x.abs(), y.abs(), new Decimal(1))
    return diff.div(scale).lt(rel)
  }
  expect(closeEnough(a.resource, b.resource)).toBe(true)
  for (const k of ['firstOwned', 'secondOwned', 'thirdOwned', 'fourthOwned', 'fifthOwned'] as const) {
    expect(a[k]).toBe(b[k])
  }
  for (const k of ['firstCost', 'secondCost', 'thirdCost', 'fourthCost', 'fifthCost'] as const) {
    expect(closeEnough(a[k], b[k])).toBe(true)
  }
}

// ─── Parity assertions ─────────────────────────────────────────────────────

describe('parity: buyMax', () => {
  const fixtures: Array<{
    type: ProducerType
    index: ProducerIndex
    owned: [number, number, number, number, number]
    resource: Decimal
    label: string
  }> = [
    { type: 'Coin', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(0), label: 'Coin/idx1/empty/0' },
    { type: 'Coin', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(50), label: 'Coin/idx1/cant afford' },
    { type: 'Coin', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e6), label: 'Coin/idx1/normal' },
    { type: 'Coin', index: 2, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e6), label: 'Coin/idx2/normal' },
    { type: 'Coin', index: 5, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e20), label: 'Coin/idx5/big' },
    { type: 'Coin', index: 1, owned: [50, 0, 0, 0, 0], resource: new Decimal(1e10), label: 'Coin/idx1/midowned' },
    { type: 'Coin', index: 1, owned: [500, 0, 0, 0, 0], resource: new Decimal(1e50), label: 'Coin/idx1/deep' },
    { type: 'Diamonds', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e6), label: 'Diamonds/idx1' },
    { type: 'Diamonds', index: 3, owned: [10, 5, 0, 0, 0], resource: new Decimal(1e30), label: 'Diamonds/idx3/mid' },
    { type: 'Mythos', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e8), label: 'Mythos/idx1' },
    { type: 'Mythos', index: 5, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e50), label: 'Mythos/idx5' },
    { type: 'Particles', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e6), label: 'Particles/idx1' }
  ]

  it.each(fixtures)('$label', (fixture) => {
    const oldStart: ProducerFamilyState = {
      resource: new Decimal(fixture.resource),
      firstOwned: fixture.owned[0],
      firstCost: getProducerCost(1, fixture.type, fixture.owned[0] + 1, baseCostInput),
      secondOwned: fixture.owned[1],
      secondCost: getProducerCost(2, fixture.type, fixture.owned[1] + 1, baseCostInput),
      thirdOwned: fixture.owned[2],
      thirdCost: getProducerCost(3, fixture.type, fixture.owned[2] + 1, baseCostInput),
      fourthOwned: fixture.owned[3],
      fourthCost: getProducerCost(4, fixture.type, fixture.owned[3] + 1, baseCostInput),
      fifthOwned: fixture.owned[4],
      fifthCost: getProducerCost(5, fixture.type, fixture.owned[4] + 1, baseCostInput)
    }
    // Clone (defensive — both fns should not mutate input but parity test
    // demands an independent baseline).
    const oldInput: ProducerFamilyState = JSON.parse(JSON.stringify({}))
    Object.assign(oldInput, {
      resource: new Decimal(oldStart.resource),
      firstOwned: oldStart.firstOwned,
      firstCost: new Decimal(oldStart.firstCost),
      secondOwned: oldStart.secondOwned,
      secondCost: new Decimal(oldStart.secondCost),
      thirdOwned: oldStart.thirdOwned,
      thirdCost: new Decimal(oldStart.thirdCost),
      fourthOwned: oldStart.fourthOwned,
      fourthCost: new Decimal(oldStart.fourthCost),
      fifthOwned: oldStart.fifthOwned,
      fifthCost: new Decimal(oldStart.fifthCost)
    })

    const oldEnd = applyOldBuyMax(oldInput, fixture.index, fixture.type, baseCostInput)
    const { state: newEnd } = newBuyMax(oldStart, {
      index: fixture.index,
      type: fixture.type,
      costInput: baseCostInput
    })

    expectStatesEqual(newEnd, oldEnd)
  })
})

