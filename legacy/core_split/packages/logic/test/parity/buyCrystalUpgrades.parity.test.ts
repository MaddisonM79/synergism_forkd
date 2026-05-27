// Parity test for buyCrystalUpgrades + its internal calculateCrystalBuy.
//
// Pre-migration source: packages/web_ui/src/Buy.ts at HEAD (this commit).
// The OLD pair is transcribed below as a pure transform with these
// dependencies hoisted into explicit inputs:
//   - getRuneEffects('prism', 'costDivisorLog10') → prismCostDivisorLog10
//   - G.crystalUpgradesCost[i-1]                  → crystalUpgradesCost
//   - G.crystalUpgradeCostIncrement[i-1]          → crystalUpgradeCostIncrement
//   - player.upgrades[73]                         → upgrade73
//   - player.currentChallenge.reincarnation !== 0 → inAnyReincarnationChallenge
//
// UI side effect `crystalupgradedescriptions(i)` is omitted — the parity test
// only validates state transitions, not UI hooks.

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import { buyCrystalUpgrades as newBuyCrystalUpgrades } from '../../src/mechanics/crystalUpgrades'
import type { CrystalUpgradesState } from '../../src/state/schema'

interface OldInput {
  i: number
  auto: boolean
  prismCostDivisorLog10: number
  crystalUpgradesCost: number
  crystalUpgradeCostIncrement: number
  upgrade73: number
  inAnyReincarnationChallenge: boolean
}

const oldCalculateCrystalBuy = (
  prestigeShards: Decimal,
  prismCostDivisorLog10: number,
  crystalUpgradesCost: number,
  crystalUpgradeCostIncrement: number
): number => {
  const exponent = Decimal.log(prestigeShards.add(1), 10)
  const toBuy = Math.floor(
    Math.pow(
      Math.max(
        0,
        2 * (exponent + prismCostDivisorLog10 - crystalUpgradesCost) / crystalUpgradeCostIncrement + 1 / 4
      ),
      1 / 2
    )
      + 1 / 2
  )
  return toBuy
}

const applyOldBuyCrystalUpgrades = (
  state: CrystalUpgradesState,
  input: OldInput
): CrystalUpgradesState => {
  const next: CrystalUpgradesState = {
    prestigeShards: new Decimal(state.prestigeShards),
    crystalUpgrades: [...state.crystalUpgrades]
  }
  const u = input.i - 1

  let c = 0
  if (input.upgrade73 > 0.5 && input.inAnyReincarnationChallenge) {
    c += 10
  }

  const toBuy = oldCalculateCrystalBuy(
    next.prestigeShards,
    input.prismCostDivisorLog10,
    input.crystalUpgradesCost,
    input.crystalUpgradeCostIncrement
  )

  if (toBuy + c > next.crystalUpgrades[u]) {
    next.crystalUpgrades[u] = 100 / 100 * (toBuy + c) // preserved verbatim — legacy scaling no-op
    if (toBuy > 0 && !input.auto) {
      next.prestigeShards = next.prestigeShards.sub(
        Decimal.pow(
          10,
          input.crystalUpgradesCost - input.prismCostDivisorLog10
            + input.crystalUpgradeCostIncrement * (1 / 2 * Math.pow(toBuy - 1 / 2, 2) - 1 / 8)
        )
      )
      next.prestigeShards = next.prestigeShards.max(0)
    }
  }

  return next
}

const makeState = (overrides: Partial<CrystalUpgradesState> = {}): CrystalUpgradesState => ({
  prestigeShards: new Decimal(1e20),
  crystalUpgrades: Array.from({ length: 5 }, () => 0),
  ...overrides
})

const expectStatesEqual = (a: CrystalUpgradesState, b: CrystalUpgradesState): void => {
  // prestigeShards: equal within tight tolerance (the formula uses log/pow
  // round-trips that can introduce sub-ULP noise).
  const rel = 1e-9
  const diff = a.prestigeShards.minus(b.prestigeShards).abs()
  const scale = Decimal.max(a.prestigeShards.abs(), b.prestigeShards.abs(), new Decimal(1))
  expect(diff.div(scale).lt(rel)).toBe(true)
  expect(a.crystalUpgrades).toEqual(b.crystalUpgrades)
}

// Realistic crystal upgrade cost constants from packages/web_ui/src/Variables.ts:
//   crystalUpgradesCost:       [7,  15, 20, 40,   1000]
//   crystalUpgradeCostIncrement: [10, 11, 12, 13,   100]
// Sampling a couple of them in the grid.
describe('parity: buyCrystalUpgrades', () => {
  const fixtures: Array<
    {
      label: string
      shards: Decimal
      currentLevel: number
      i: number
      auto: boolean
      prism: number
      cost: number
      incr: number
      upg73: number
      inRC: boolean
    }
  > = [
    { label: 'idx1 manual zero level, fresh', shards: new Decimal(1e20), currentLevel: 0, i: 1, auto: false, prism: 0, cost: 7, incr: 10, upg73: 0, inRC: false },
    { label: 'idx1 auto path (no spend)', shards: new Decimal(1e20), currentLevel: 0, i: 1, auto: true, prism: 0, cost: 7, incr: 10, upg73: 0, inRC: false },
    { label: 'idx1 manual with prism reduction', shards: new Decimal(1e20), currentLevel: 0, i: 1, auto: false, prism: 5, cost: 7, incr: 10, upg73: 0, inRC: false },
    { label: 'idx2 manual', shards: new Decimal(1e15), currentLevel: 0, i: 2, auto: false, prism: 0, cost: 15, incr: 11, upg73: 0, inRC: false },
    { label: 'idx3 high level start', shards: new Decimal(1e30), currentLevel: 50, i: 3, auto: false, prism: 0, cost: 20, incr: 12, upg73: 0, inRC: false },
    { label: 'idx1 with c=10 (upg73 + reincarnation challenge)', shards: new Decimal(1e20), currentLevel: 5, i: 1, auto: false, prism: 0, cost: 7, incr: 10, upg73: 1, inRC: true },
    { label: 'idx1 with upg73 but NOT in challenge (c=0)', shards: new Decimal(1e20), currentLevel: 5, i: 1, auto: false, prism: 0, cost: 7, incr: 10, upg73: 1, inRC: false },
    { label: 'idx1 in challenge but no upg73 (c=0)', shards: new Decimal(1e20), currentLevel: 5, i: 1, auto: false, prism: 0, cost: 7, incr: 10, upg73: 0, inRC: true },
    { label: 'idx1 already past toBuy+c (no buy)', shards: new Decimal(1e6), currentLevel: 100, i: 1, auto: false, prism: 0, cost: 7, incr: 10, upg73: 0, inRC: false },
    { label: 'idx1 zero shards (toBuy=0, no buy)', shards: new Decimal(0), currentLevel: 0, i: 1, auto: false, prism: 0, cost: 7, incr: 10, upg73: 0, inRC: false },
    { label: 'idx5 deep curve (cost=1000 incr=100)', shards: new Decimal('1e6000'), currentLevel: 0, i: 5, auto: false, prism: 50, cost: 1000, incr: 100, upg73: 0, inRC: false },
    { label: 'idx5 auto deep curve', shards: new Decimal('1e6000'), currentLevel: 0, i: 5, auto: true, prism: 50, cost: 1000, incr: 100, upg73: 0, inRC: false }
  ]

  it.each(fixtures)('$label', (f) => {
    const start = makeState({
      prestigeShards: f.shards,
      crystalUpgrades: (() => {
        const arr = Array.from({ length: 5 }, () => 0)
        arr[f.i - 1] = f.currentLevel
        return arr
      })()
    })
    const oldNext = applyOldBuyCrystalUpgrades(start, {
      i: f.i,
      auto: f.auto,
      prismCostDivisorLog10: f.prism,
      crystalUpgradesCost: f.cost,
      crystalUpgradeCostIncrement: f.incr,
      upgrade73: f.upg73,
      inAnyReincarnationChallenge: f.inRC
    })
    const { state: newNext } = newBuyCrystalUpgrades(start, {
      i: f.i,
      auto: f.auto,
      prismCostDivisorLog10: f.prism,
      crystalUpgradesCost: f.cost,
      crystalUpgradeCostIncrement: f.incr,
      upgrade73: f.upg73,
      inAnyReincarnationChallenge: f.inRC
    })
    expectStatesEqual(newNext, oldNext)
  })
})
