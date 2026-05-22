import { describe, expect, it } from 'vitest'
import { Decimal } from '../src/math/bignum'
import {
  buyMax,
  getProducerCost,
  type BuyMaxInput,
  type GetProducerCostInput,
  type ProducerIndex,
  type ProducerType
} from '../src/mechanics/producers'
import type { ProducerFamilyState } from '../src/state/schema'

const baseCostInput: GetProducerCostInput = {
  costDivisor: 1,
  inTranscensionChallenge4: false,
  inReincarnationChallenge8: false,
  inReincarnationChallenge10: false,
  challengecompletions4: 0,
  challengecompletions8: 0
}

const makeState = (overrides: Partial<ProducerFamilyState> = {}): ProducerFamilyState => ({
  resource: new Decimal(0),
  firstOwned: 0,
  firstCost: getProducerCost(1, 'Coin', 1, baseCostInput),
  secondOwned: 0,
  secondCost: getProducerCost(2, 'Coin', 1, baseCostInput),
  thirdOwned: 0,
  thirdCost: getProducerCost(3, 'Coin', 1, baseCostInput),
  fourthOwned: 0,
  fourthCost: getProducerCost(4, 'Coin', 1, baseCostInput),
  fifthOwned: 0,
  fifthCost: getProducerCost(5, 'Coin', 1, baseCostInput),
  ...overrides
})

const makeInput = (overrides: Partial<BuyMaxInput> = {}): BuyMaxInput => ({
  index: 1,
  type: 'Coin',
  costInput: baseCostInput,
  ...overrides
})

describe('buyMax', () => {
  it('is a no-op when the resource is zero', () => {
    const state = makeState()
    const { state: next, events } = buyMax(state, makeInput())
    expect(next.firstOwned).toBe(0)
    expect(next.resource.eq(state.resource)).toBe(true)
    expect(events).toEqual([])
  })

  it('purchases at least one producer when the resource covers the first cost', () => {
    // First-Coin cost is 100; give enough to buy a handful.
    const state = makeState({ resource: new Decimal(1e6) })
    const { state: next, events } = buyMax(state, makeInput())
    expect(next.firstOwned).toBeGreaterThan(0)
    expect(next.resource.lt(state.resource)).toBe(true)
    expect(events).toHaveLength(1)
    expect(events[0]?.kind).toBe('producers-purchased')
  })

  it('updates only the targeted position (index 3 buys third-coin)', () => {
    const state = makeState({ resource: new Decimal(1e15) })
    const { state: next } = buyMax(state, makeInput({ index: 3 }))
    expect(next.thirdOwned).toBeGreaterThan(0)
    expect(next.firstOwned).toBe(0)
    expect(next.secondOwned).toBe(0)
    expect(next.fourthOwned).toBe(0)
    expect(next.fifthOwned).toBe(0)
    expect(next.thirdCost.gt(state.thirdCost)).toBe(true)
    // Untouched-position costs preserved.
    expect(next.firstCost.eq(state.firstCost)).toBe(true)
    expect(next.fifthCost.eq(state.fifthCost)).toBe(true)
  })

  it('handles every producer type independently', () => {
    for (const type of ['Coin', 'Diamonds', 'Mythos', 'Particles'] as ProducerType[]) {
      // Diamonds tier-1 cost is 100; Mythos tier-1 is 1; Particles tier-1 is 1.
      // Pick a budget that comfortably affords several at any type.
      const state = makeState({
        resource: new Decimal(1e20),
        firstCost: getProducerCost(1, type, 1, baseCostInput)
      })
      const { state: next } = buyMax(state, makeInput({ type }))
      expect(next.firstOwned).toBeGreaterThan(0)
    }
  })

  it('emits an event whose spent matches the resource delta', () => {
    const state = makeState({ resource: new Decimal(1e8) })
    const { state: next, events } = buyMax(state, makeInput())
    const spent = state.resource.sub(next.resource)
    expect(events[0]?.kind).toBe('producers-purchased')
    if (events[0]?.kind === 'producers-purchased') {
      expect(events[0].spent.eq(spent)).toBe(true)
      expect(events[0].before).toBe(0)
      expect(events[0].after).toBe(next.firstOwned)
      expect(events[0].index).toBe(1)
      expect(events[0].type).toBe('Coin')
    }
  })

  it('does not mutate the input state', () => {
    const state = makeState({ resource: new Decimal(1e6) })
    const snapshot = {
      resourceMantissa: state.resource.mantissa,
      resourceExponent: state.resource.exponent,
      firstOwned: state.firstOwned,
      firstCostMantissa: state.firstCost.mantissa,
      firstCostExponent: state.firstCost.exponent,
      thirdOwned: state.thirdOwned
    }
    buyMax(state, makeInput())
    expect(state.resource.mantissa).toBe(snapshot.resourceMantissa)
    expect(state.resource.exponent).toBe(snapshot.resourceExponent)
    expect(state.firstOwned).toBe(snapshot.firstOwned)
    expect(state.firstCost.mantissa).toBe(snapshot.firstCostMantissa)
    expect(state.firstCost.exponent).toBe(snapshot.firstCostExponent)
    expect(state.thirdOwned).toBe(snapshot.thirdOwned)
  })

  it('returns events typed as the discriminated union', () => {
    const state = makeState({ resource: new Decimal(1e6) })
    const { events } = buyMax(state, makeInput())
    for (const ev of events) {
      switch (ev.kind) {
        case 'producers-purchased':
          expect(typeof ev.before).toBe('number')
          expect(typeof ev.after).toBe('number')
          expect(typeof ev.index).toBe('number')
          expect(typeof ev.type).toBe('string')
          expect(ev.spent).toBeInstanceOf(Decimal)
          break
      }
    }
  })

  it('walks the affordability bracket — final cost matches getProducerCost(after+1)', () => {
    const state = makeState({ resource: new Decimal(1e10) })
    const { state: next } = buyMax(state, makeInput())
    // After the walk, firstCost should equal the cost to buy ONE MORE.
    const expectedNextCost = getProducerCost(1, 'Coin', next.firstOwned + 1, baseCostInput)
    expect(next.firstCost.eq(expectedNextCost)).toBe(true)
  })

  it('respects the resource budget — never overspends', () => {
    const budget = new Decimal(1e8)
    const state = makeState({ resource: budget })
    const { state: next } = buyMax(state, makeInput())
    // Resource never goes negative.
    expect(next.resource.gte(0)).toBe(true)
    // And spent (= startingResource - final) <= budget.
    const spent = budget.sub(next.resource)
    expect(spent.lte(budget)).toBe(true)
  })

  it.each<ProducerIndex>([1, 2, 3, 4, 5])('each position index buys its own slot (%s)', (index) => {
    const state = makeState({ resource: new Decimal(1e20) })
    const { state: next } = buyMax(state, makeInput({ index }))
    const owned = [next.firstOwned, next.secondOwned, next.thirdOwned, next.fourthOwned, next.fifthOwned]
    expect(owned[index - 1]).toBeGreaterThan(0)
    for (let i = 0; i < 5; i++) {
      if (i !== index - 1) expect(owned[i]).toBe(0)
    }
  })
})
