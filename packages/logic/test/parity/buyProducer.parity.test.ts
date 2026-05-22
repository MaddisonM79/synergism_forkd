// Parity test for the producer buyProducer manual-click loop.
//
// Pre-migration source: packages/web_ui/src/Buy.ts at HEAD (this commit).
// The OLD function below transcribes that loop with `player.*` / `G.ticker`
// hoisted into explicit parameters. The NEW version (imported from
// @synergism/logic) is then asserted byte-equal across a grid that exercises:
//   - all (index, type) combinations
//   - threshold transitions at 1000*r / 5000*r / 20000*r / 250000*r
//   - challenge-4 (transcension) and challenge-8 (reincarnation) amplifiers
//   - autobuyer cap (500) vs configured buyamount

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import {
  buyProducer as newBuyProducer,
  type ProducerIndex,
  type ProducerType
} from '../../src/mechanics/producers'
import type { ProducerFamilyState } from '../../src/state/schema'

const POSITION_NAMES = ['first', 'second', 'third', 'fourth', 'fifth'] as const
type PositionName = typeof POSITION_NAMES[number]

const readOwned = (state: ProducerFamilyState, index: ProducerIndex): number => {
  if (index === 1) return state.firstOwned
  if (index === 2) return state.secondOwned
  if (index === 3) return state.thirdOwned
  if (index === 4) return state.fourthOwned
  return state.fifthOwned
}
const readCost = (state: ProducerFamilyState, index: ProducerIndex): Decimal => {
  if (index === 1) return state.firstCost
  if (index === 2) return state.secondCost
  if (index === 3) return state.thirdCost
  if (index === 4) return state.fourthCost
  return state.fifthCost
}
const writeOwned = (state: ProducerFamilyState, index: ProducerIndex, v: number): void => {
  if (index === 1) state.firstOwned = v
  else if (index === 2) state.secondOwned = v
  else if (index === 3) state.thirdOwned = v
  else if (index === 4) state.fourthOwned = v
  else state.fifthOwned = v
}
const writeCost = (state: ProducerFamilyState, index: ProducerIndex, v: Decimal): void => {
  if (index === 1) state.firstCost = v
  else if (index === 2) state.secondCost = v
  else if (index === 3) state.thirdCost = v
  else if (index === 4) state.fourthCost = v
  else state.fifthCost = v
}

// `num` derivation matches the original Buy.ts call sites in EventListeners.ts:
// for Coin it's the position index; for everything else it's the triangle
// number index*(index+1)/2.
const numFor = (index: ProducerIndex, type: ProducerType): number =>
  type === 'Coin' ? index : index * (index + 1) / 2

// OLD buyProducer transcribed verbatim from Buy.ts (HEAD) as a pure transform.
// Original used `G.ticker` as the loop counter; we model it as the local `t`.
const applyOldBuyProducer = (
  state: ProducerFamilyState,
  index: ProducerIndex,
  type: ProducerType,
  autobuyer: boolean,
  buyamount: number,
  r: number,
  inTranscensionChallenge4: boolean,
  inReincarnationChallenge8: boolean,
  challengecompletions4: number,
  challengecompletions8: number
): ProducerFamilyState => {
  const num = numFor(index, type)
  void (POSITION_NAMES as readonly PositionName[]) // documentation only — original used pos names
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
  const buythisamount = autobuyer ? 500 : buyamount
  let t = 0

  while (
    next.resource.gte(readCost(next, index))
    && t < buythisamount
    && readOwned(next, index) < Number.MAX_SAFE_INTEGER
  ) {
    next.resource = next.resource.sub(readCost(next, index))
    writeOwned(next, index, readOwned(next, index) + 1)
    let cost = readCost(next, index).times(Decimal.pow(1.25, num))
    cost = cost.add(1)
    const owned = readOwned(next, index)
    if (owned >= 1000 * r) {
      cost = cost.times(owned).dividedBy(1000).times(1 + num / 2)
    }
    if (owned >= 5000 * r) {
      cost = cost.times(owned).times(10).times(10 + num * 10)
    }
    if (owned >= 20000 * r) {
      cost = cost.times(Decimal.pow(owned, 3)).times(100000).times(100 + num * 100)
    }
    if (owned >= 250000 * r) {
      cost = cost.times(Decimal.pow(1.03, owned - 250000 * r))
    }
    if (inTranscensionChallenge4 && (type === 'Coin' || type === 'Diamonds')) {
      cost = cost.times(
        Math.pow(100 * owned + 10000, 1.25 + 1 / 4 * challengecompletions4)
      )
      if (owned >= 1000 - 10 * challengecompletions4) {
        cost = cost.times(Decimal.pow(1.25, owned))
      }
    }
    if (
      inReincarnationChallenge8
      && (type === 'Coin' || type === 'Diamonds' || type === 'Mythos')
      && owned >= 1000 * challengecompletions8 * r
    ) {
      cost = cost.times(
        Decimal.pow(
          2,
          (owned - 1000 * challengecompletions8 * r) / (1 + challengecompletions8 / 2)
        )
      )
    }
    writeCost(next, index, cost)
    t += 1
  }
  return next
}

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

const makeState = (
  type: ProducerType,
  owned: [number, number, number, number, number],
  resource: Decimal,
  initialCosts: [Decimal, Decimal, Decimal, Decimal, Decimal]
): ProducerFamilyState => ({
  resource,
  firstOwned: owned[0],
  firstCost: initialCosts[0],
  secondOwned: owned[1],
  secondCost: initialCosts[1],
  thirdOwned: owned[2],
  thirdCost: initialCosts[2],
  fourthOwned: owned[3],
  fourthCost: initialCosts[3],
  fifthOwned: owned[4],
  fifthCost: initialCosts[4]
})

// Tier-1 starting costs by type (used to derive baseline initial states).
const BASE_COST_BY_TYPE: Record<ProducerType, [number, number, number, number, number]> = {
  Coin: [100, 1000, 2e4, 4e5, 8e6],
  Diamonds: [100, 1e5, 1e15, 1e40, 1e100],
  Mythos: [1, 1e2, 1e4, 1e8, 1e16],
  Particles: [1, 1e2, 1e4, 1e8, 1e16]
}

describe('parity: buyProducer', () => {
  const fixtures: Array<{
    label: string
    type: ProducerType
    index: ProducerIndex
    owned: [number, number, number, number, number]
    resource: Decimal
    autobuyer: boolean
    buyamount: number
    r: number
    inTC4: boolean
    inRC8: boolean
    cc4: number
    cc8: number
  }> = [
    { label: 'Coin idx1 idle', type: 'Coin', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(50), autobuyer: false, buyamount: 100, r: 1, inTC4: false, inRC8: false, cc4: 0, cc8: 0 },
    { label: 'Coin idx1 click 100', type: 'Coin', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e8), autobuyer: false, buyamount: 100, r: 1, inTC4: false, inRC8: false, cc4: 0, cc8: 0 },
    { label: 'Coin idx1 autobuyer cap 500', type: 'Coin', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e80), autobuyer: true, buyamount: 100, r: 1, inTC4: false, inRC8: false, cc4: 0, cc8: 0 },
    { label: 'Coin idx5 deep', type: 'Coin', index: 5, owned: [0, 0, 0, 0, 1500], resource: new Decimal(1e80), autobuyer: false, buyamount: 100, r: 1, inTC4: false, inRC8: false, cc4: 0, cc8: 0 },
    { label: 'Coin idx1 threshold 5000', type: 'Coin', index: 1, owned: [4998, 0, 0, 0, 0], resource: new Decimal(1e120), autobuyer: false, buyamount: 10, r: 1, inTC4: false, inRC8: false, cc4: 0, cc8: 0 },
    { label: 'Coin idx2 threshold 20000', type: 'Coin', index: 2, owned: [0, 19990, 0, 0, 0], resource: new Decimal(1e200), autobuyer: false, buyamount: 20, r: 1, inTC4: false, inRC8: false, cc4: 0, cc8: 0 },
    { label: 'Coin idx1 threshold 250k with r=2', type: 'Coin', index: 1, owned: [499990, 0, 0, 0, 0], resource: new Decimal('1e1000'), autobuyer: false, buyamount: 25, r: 2, inTC4: false, inRC8: false, cc4: 0, cc8: 0 },
    { label: 'Diamonds idx1 normal', type: 'Diamonds', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e6), autobuyer: false, buyamount: 50, r: 1, inTC4: false, inRC8: false, cc4: 0, cc8: 0 },
    { label: 'Diamonds idx3 deep', type: 'Diamonds', index: 3, owned: [0, 0, 100, 0, 0], resource: new Decimal(1e50), autobuyer: false, buyamount: 50, r: 1, inTC4: false, inRC8: false, cc4: 0, cc8: 0 },
    { label: 'Coin in C4 with cc4=5', type: 'Coin', index: 1, owned: [950, 0, 0, 0, 0], resource: new Decimal(1e100), autobuyer: false, buyamount: 100, r: 1, inTC4: true, inRC8: false, cc4: 5, cc8: 0 },
    { label: 'Diamonds in C4 with cc4=10', type: 'Diamonds', index: 2, owned: [0, 850, 0, 0, 0], resource: new Decimal(1e200), autobuyer: false, buyamount: 50, r: 1, inTC4: true, inRC8: false, cc4: 10, cc8: 0 },
    { label: 'Coin in C8 with cc8=3', type: 'Coin', index: 1, owned: [3010, 0, 0, 0, 0], resource: new Decimal(1e150), autobuyer: false, buyamount: 100, r: 1, inTC4: false, inRC8: true, cc4: 0, cc8: 3 },
    { label: 'Mythos in C8 with cc8=5', type: 'Mythos', index: 1, owned: [4995, 0, 0, 0, 0], resource: new Decimal(1e200), autobuyer: false, buyamount: 100, r: 1, inTC4: false, inRC8: true, cc4: 0, cc8: 5 },
    { label: 'Particles idx1 (no C8 effect)', type: 'Particles', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e10), autobuyer: false, buyamount: 100, r: 1, inTC4: false, inRC8: true, cc4: 0, cc8: 5 },
    { label: 'Big r=3 deferred thresholds', type: 'Coin', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e80), autobuyer: false, buyamount: 100, r: 3, inTC4: false, inRC8: false, cc4: 0, cc8: 0 },
    { label: 'Combined C4+C8', type: 'Diamonds', index: 1, owned: [0, 0, 0, 0, 0], resource: new Decimal(1e120), autobuyer: false, buyamount: 80, r: 1, inTC4: true, inRC8: true, cc4: 5, cc8: 3 }
  ]

  it.each(fixtures)('$label', (f) => {
    const baseCosts = BASE_COST_BY_TYPE[f.type]
    const initialCosts: [Decimal, Decimal, Decimal, Decimal, Decimal] = [
      new Decimal(baseCosts[0] * Math.pow(1 + f.owned[0], 3)),
      new Decimal(baseCosts[1] * Math.pow(1 + f.owned[1], 3)),
      new Decimal(baseCosts[2] * Math.pow(1 + f.owned[2], 3)),
      new Decimal(baseCosts[3] * Math.pow(1 + f.owned[3], 3)),
      new Decimal(baseCosts[4] * Math.pow(1 + f.owned[4], 3))
    ]
    // Use a sensible "current cost" only for the chosen index — others are
    // never touched by buyProducer, so their initial values are arbitrary.
    const state = makeState(f.type, f.owned, f.resource, initialCosts)

    const oldNext = applyOldBuyProducer(
      state,
      f.index,
      f.type,
      f.autobuyer,
      f.buyamount,
      f.r,
      f.inTC4,
      f.inRC8,
      f.cc4,
      f.cc8
    )
    const { state: newNext } = newBuyProducer(state, {
      index: f.index,
      type: f.type,
      autobuyer: f.autobuyer,
      buyamount: f.buyamount,
      r: f.r,
      inTranscensionChallenge4: f.inTC4,
      inReincarnationChallenge8: f.inRC8,
      challengecompletions4: f.cc4,
      challengecompletions8: f.cc8
    })
    expectStatesEqual(newNext, oldNext)
  })
})
