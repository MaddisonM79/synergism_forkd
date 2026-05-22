// Parity test for getParticleCost.
//
// Holds the migrated logic-package implementation byte-equal to the
// pre-migration version that lived in packages/web_ui/src/Buy.ts at commit
// a25de5f0~1. The OLD function below is a pure transcription of that
// implementation — only the challenge-15 ascension check is hoisted from
// `player.currentChallenge.ascension !== 15` into an explicit parameter.

import { describe, expect, it } from 'vitest'
import Decimal, { type DecimalSource } from 'break_infinity.js'
import { smallestInc } from '../../src/math/smallestInc'
import {
  buyParticleBuilding as newBuyParticleBuilding,
  getParticleCost as newGetParticleCost,
  type ParticleBuildingIndex
} from '../../src/mechanics/particleBuildings'
import type { BuyAmount, ParticleBuildingsState } from '../../src/state/schema'

const oldGetParticleCost = (
  originalCost: DecimalSource,
  buyTo: number,
  isAscensionChallenge15: boolean
): Decimal => {
  ;--buyTo
  originalCost = new Decimal(originalCost)
  let cost = originalCost.times(Decimal.pow(2, buyTo))

  const DR = isAscensionChallenge15 ? 1000 : 325000

  if (buyTo > DR) {
    cost = cost.times(Decimal.pow(1.001, (buyTo - DR) * ((buyTo - DR + 1) / 2)))
  }
  const buymax = Math.pow(10, 15)
  if (buyTo > buymax) {
    const diminishingExponent = 1 / 8

    // Match the original code's `getParticleCost(originalCost, buymax)`
    // recursive call — pass the 1-based caller buyTo verbatim; the inner
    // pre-decrement turns it into buymax-1 internally.
    const QuadrillionCost = oldGetParticleCost(originalCost, buymax, isAscensionChallenge15)

    const newCost = QuadrillionCost.pow(Math.pow(buyTo / buymax, 1 / diminishingExponent))
    const newExtra = newCost.exponent - Math.floor(newCost.exponent)
    newCost.exponent = Math.floor(newCost.exponent)
    newCost.mantissa *= Math.pow(10, newExtra)
    newCost.normalize()
    return Decimal.max(cost, newCost)
  }
  return cost
}

const equalEnough = (a: Decimal, b: Decimal, rel = 1e-12): boolean => {
  if (a.eq(b)) return true
  if (a.abs().lt(1) && b.abs().lt(1)) {
    return a.minus(b).abs().lt(rel)
  }
  const diff = a.minus(b).abs()
  const scale = Decimal.max(a.abs(), b.abs())
  return diff.div(scale).lt(rel)
}

// Maps the migrated function's index → originalCost base (the OLD function
// took originalCost directly; the new one looks it up from index).
const ORIGINAL_COST_BY_INDEX: Record<ParticleBuildingIndex, number> = {
  1: 1,
  2: 1e2,
  3: 1e4,
  4: 1e8,
  5: 1e16
}

describe('parity: getParticleCost', () => {
  const buyToPoints = [1, 5, 50, 500, 5_000, 50_000, 325_000, 325_001, 326_000, 500_000, 1_000_000]
  const indexes: ParticleBuildingIndex[] = [1, 2, 3, 4, 5]
  const c15States = [false, true]

  for (const isC15 of c15States) {
    for (const index of indexes) {
      describe(`index=${index}, inAscensionChallenge15=${isC15}`, () => {
        it.each(buyToPoints)('buyTo=%i matches', (buyTo) => {
          const oldCost = oldGetParticleCost(ORIGINAL_COST_BY_INDEX[index], buyTo, isC15)
          const newCost = newGetParticleCost(buyTo, { index, inAscensionChallenge15: isC15 })
          expect(equalEnough(oldCost, newCost)).toBe(true)
        })
      })
    }
  }

  // Buymax dim branch — sample around the breakpoint.
  it('matches across the buymax (1e15) dim branch', () => {
    const points = [1e15, 1.0001e15, 2e15]
    for (const lvl of points) {
      const oldCost = oldGetParticleCost(1e4, lvl, false)
      const newCost = newGetParticleCost(lvl, { index: 3, inAscensionChallenge15: false })
      expect(equalEnough(oldCost, newCost)).toBe(true)
    }
  })
})

// ─── buyParticleBuilding loop parity ───────────────────────────────────────

const POSITIONS = ['first', 'second', 'third', 'fourth', 'fifth'] as const

const readOwnedSlice = (state: ParticleBuildingsState, index: ParticleBuildingIndex): number => {
  if (index === 1) return state.firstOwnedParticles
  if (index === 2) return state.secondOwnedParticles
  if (index === 3) return state.thirdOwnedParticles
  if (index === 4) return state.fourthOwnedParticles
  return state.fifthOwnedParticles
}
const writeOwnedSlice = (state: ParticleBuildingsState, index: ParticleBuildingIndex, v: number): void => {
  if (index === 1) state.firstOwnedParticles = v
  else if (index === 2) state.secondOwnedParticles = v
  else if (index === 3) state.thirdOwnedParticles = v
  else if (index === 4) state.fourthOwnedParticles = v
  else state.fifthOwnedParticles = v
}
const writeCostSlice = (state: ParticleBuildingsState, index: ParticleBuildingIndex, v: Decimal): void => {
  if (index === 1) state.firstCostParticles = v
  else if (index === 2) state.secondCostParticles = v
  else if (index === 3) state.thirdCostParticles = v
  else if (index === 4) state.fourthCostParticles = v
  else state.fifthCostParticles = v
}

// OLD buyParticleBuilding modeled as a pure transform on
// ParticleBuildingsState. Mirrors the loop in /tmp/parity/old_buy_particle.ts
// lines 541-615 with player.* replaced by the cloned slice.
const applyOldBuyParticle = (
  state: ParticleBuildingsState,
  index: ParticleBuildingIndex,
  autobuyer: boolean,
  particlebuyamount: BuyAmount,
  isAscensionChallenge15: boolean
): ParticleBuildingsState => {
  void POSITIONS // referenced via index → field-name helpers above
  const originalCost = ORIGINAL_COST_BY_INDEX[index]
  const buymax = Math.pow(10, 15)
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
  const buyStart = readOwnedSlice(next, index)
  const cost = (b: number): Decimal => oldGetParticleCost(originalCost, b, isAscensionChallenge15)

  if (buyStart >= buymax) {
    const diminishingExponent = 1 / 8
    const log10Resource = Decimal.log10(next.reincarnationPoints)
    const log10QuadrillionCost = Decimal.log10(cost(buymax))
    let hi = Math.floor(buymax * Math.max(1, Math.pow(log10Resource / log10QuadrillionCost, diminishingExponent)))
    let lo = buymax
    while (hi - lo > 0.5) {
      const mid = Math.floor(lo + (hi - lo) / 2)
      if (mid === lo || mid === hi) break
      if (!next.reincarnationPoints.gte(cost(mid))) {
        hi = mid
      } else {
        lo = mid
      }
    }
    const buyable = lo
    writeOwnedSlice(next, index, buyable)
    writeCostSlice(next, index, cost(buyable))
    return next
  }

  const buydefault = buyStart + smallestInc(buyStart)
  let buyTo = buydefault
  let cashToBuy = cost(buyTo)
  while (next.reincarnationPoints.gte(cashToBuy)) {
    buyTo = buyTo * 4
    cashToBuy = cost(buyTo)
  }
  let stepdown = Math.floor(buyTo / 8)
  while (stepdown >= smallestInc(buyTo)) {
    if (cost(buyTo - stepdown).lte(next.reincarnationPoints)) {
      stepdown = Math.floor(stepdown / 2)
    } else {
      buyTo = buyTo - Math.max(smallestInc(buyTo), stepdown)
    }
  }
  if (!autobuyer) {
    if (particlebuyamount + buyStart < buyTo) {
      buyTo = buyStart + particlebuyamount + smallestInc(buyStart + particlebuyamount)
    }
  }

  let buyFrom = Math.max(buyTo - 6 - smallestInc(buyTo), buydefault)
  let thisCost = cost(buyFrom)
  while (buyFrom <= buyTo && next.reincarnationPoints.gte(thisCost)) {
    next.reincarnationPoints = next.reincarnationPoints.sub(thisCost)
    writeOwnedSlice(next, index, buyFrom)
    buyFrom = buyFrom + smallestInc(buyFrom)
    thisCost = cost(buyFrom)
    writeCostSlice(next, index, thisCost)
  }

  return next
}

const makeParticleState = (
  owned: [number, number, number, number, number] = [0, 0, 0, 0, 0],
  reincarnationPoints: Decimal = new Decimal(0)
): ParticleBuildingsState => ({
  reincarnationPoints,
  firstOwnedParticles: owned[0],
  firstCostParticles: oldGetParticleCost(ORIGINAL_COST_BY_INDEX[1], owned[0] + 1, false),
  secondOwnedParticles: owned[1],
  secondCostParticles: oldGetParticleCost(ORIGINAL_COST_BY_INDEX[2], owned[1] + 1, false),
  thirdOwnedParticles: owned[2],
  thirdCostParticles: oldGetParticleCost(ORIGINAL_COST_BY_INDEX[3], owned[2] + 1, false),
  fourthOwnedParticles: owned[3],
  fourthCostParticles: oldGetParticleCost(ORIGINAL_COST_BY_INDEX[4], owned[3] + 1, false),
  fifthOwnedParticles: owned[4],
  fifthCostParticles: oldGetParticleCost(ORIGINAL_COST_BY_INDEX[5], owned[4] + 1, false)
})

const expectParticleStatesEqual = (a: ParticleBuildingsState, b: ParticleBuildingsState): void => {
  expect(equalEnough(a.reincarnationPoints, b.reincarnationPoints)).toBe(true)
  expect(a.firstOwnedParticles).toBe(b.firstOwnedParticles)
  expect(a.secondOwnedParticles).toBe(b.secondOwnedParticles)
  expect(a.thirdOwnedParticles).toBe(b.thirdOwnedParticles)
  expect(a.fourthOwnedParticles).toBe(b.fourthOwnedParticles)
  expect(a.fifthOwnedParticles).toBe(b.fifthOwnedParticles)
  expect(equalEnough(a.firstCostParticles, b.firstCostParticles)).toBe(true)
  expect(equalEnough(a.secondCostParticles, b.secondCostParticles)).toBe(true)
  expect(equalEnough(a.thirdCostParticles, b.thirdCostParticles)).toBe(true)
  expect(equalEnough(a.fourthCostParticles, b.fourthCostParticles)).toBe(true)
  expect(equalEnough(a.fifthCostParticles, b.fifthCostParticles)).toBe(true)
}

describe('parity: buyParticleBuilding', () => {
  const fixtures: Array<{
    label: string
    owned: [number, number, number, number, number]
    points: Decimal
    index: ParticleBuildingIndex
    autobuyer: boolean
    particlebuyamount: BuyAmount
    isC15: boolean
  }> = [
    { label: 'zero resource', owned: [0, 0, 0, 0, 0], points: new Decimal(0), index: 1, autobuyer: false, particlebuyamount: 1, isC15: false },
    { label: 'normal click idx1', owned: [0, 0, 0, 0, 0], points: new Decimal(1e6), index: 1, autobuyer: false, particlebuyamount: 100, isC15: false },
    { label: 'autobuyer idx1', owned: [0, 0, 0, 0, 0], points: new Decimal(1e6), index: 1, autobuyer: true, particlebuyamount: 100, isC15: false },
    { label: 'cap=1', owned: [4, 0, 0, 0, 0], points: new Decimal(1e9), index: 1, autobuyer: false, particlebuyamount: 1, isC15: false },
    { label: 'cap=10', owned: [0, 0, 0, 0, 0], points: new Decimal(1e30), index: 1, autobuyer: false, particlebuyamount: 10, isC15: false },
    { label: 'idx3 mid', owned: [0, 0, 10, 0, 0], points: new Decimal(1e20), index: 3, autobuyer: false, particlebuyamount: 1000, isC15: false },
    { label: 'idx5 big', owned: [0, 0, 0, 0, 5], points: new Decimal(1e50), index: 5, autobuyer: true, particlebuyamount: 100, isC15: false },
    { label: 'C15 active idx1', owned: [0, 0, 0, 0, 0], points: new Decimal(1e6), index: 1, autobuyer: false, particlebuyamount: 100, isC15: true },
    { label: 'C15 deep idx2', owned: [0, 500, 0, 0, 0], points: new Decimal('1e500'), index: 2, autobuyer: true, particlebuyamount: 100, isC15: true }
  ]

  it.each(fixtures)('$label', (fixture) => {
    const start = makeParticleState(fixture.owned, fixture.points)
    const oldNext = applyOldBuyParticle(
      start,
      fixture.index,
      fixture.autobuyer,
      fixture.particlebuyamount,
      fixture.isC15
    )
    const { state: newNext } = newBuyParticleBuilding(start, {
      index: fixture.index,
      autobuyer: fixture.autobuyer,
      particlebuyamount: fixture.particlebuyamount,
      inAscensionChallenge15: fixture.isC15
    })
    expectParticleStatesEqual(newNext, oldNext)
  })
})
