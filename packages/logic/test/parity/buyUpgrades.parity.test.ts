// Parity test for buyUpgrades.
//
// Pre-migration source: packages/web_ui/src/Buy.ts at HEAD (this commit).
// The OLD function is transcribed below as a pure transform with these
// dependencies hoisted into explicit inputs:
//   - `Upgrade` enum  → `tier: 'coin' | 'prestige' | 'transcend' | 'reincarnation'`
//   - `G.upgradeCosts[pos]` → `costExponent` (the log10 cost)
//   - `upgradeRequirements[pos]` → `requirementExists` (preserves the original
//      "out-of-bounds guard" — the OLD code checked the FUNCTION existed, not
//      whether it returned true; all current entries are () => true so this is
//      effectively a bounds check)
//
// The UI side effect `upgradeupdate(pos, state)` is omitted — the parity test
// only validates state transitions, not UI hooks.

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import { buyUpgrades as newBuyUpgrades } from '../../src/mechanics/upgrades'
import type { UpgradesState } from '../../src/state/schema'

type UpgradeTier = 'coin' | 'prestige' | 'transcend' | 'reincarnation'

const RESOURCE_BY_TIER: Record<UpgradeTier, keyof UpgradesState> = {
  coin: 'coins',
  prestige: 'prestigePoints',
  transcend: 'transcendPoints',
  reincarnation: 'reincarnationPoints'
}

const applyOldBuyUpgrades = (
  state: UpgradesState,
  tier: UpgradeTier,
  pos: number,
  costExponent: number,
  requirementExists: boolean
): UpgradesState => {
  const next: UpgradesState = {
    coins: new Decimal(state.coins),
    prestigePoints: new Decimal(state.prestigePoints),
    transcendPoints: new Decimal(state.transcendPoints),
    reincarnationPoints: new Decimal(state.reincarnationPoints),
    upgrades: [...state.upgrades],
    prestigenocoinupgrades: state.prestigenocoinupgrades,
    transcendnocoinupgrades: state.transcendnocoinupgrades,
    transcendnocoinorprestigeupgrades: state.transcendnocoinorprestigeupgrades,
    reincarnatenocoinupgrades: state.reincarnatenocoinupgrades,
    reincarnatenocoinorprestigeupgrades: state.reincarnatenocoinorprestigeupgrades,
    reincarnatenocoinprestigeortranscendupgrades: state.reincarnatenocoinprestigeortranscendupgrades,
    reincarnatenocoinprestigetranscendorgeneratorupgrades: state.reincarnatenocoinprestigetranscendorgeneratorupgrades
  }
  if (!requirementExists) return next

  const currencyKey = RESOURCE_BY_TIER[tier] as 'coins' | 'prestigePoints' | 'transcendPoints' | 'reincarnationPoints'
  const cost = Decimal.pow(10, costExponent)
  if (next[currencyKey].gte(cost) && next.upgrades[pos] === 0) {
    next[currencyKey] = next[currencyKey].sub(cost)
    next.upgrades[pos] = 1
    // (upgradeupdate(pos, state) UI side effect omitted from parity model)
  }

  if (tier === 'transcend') {
    next.reincarnatenocoinprestigeortranscendupgrades = false
    next.reincarnatenocoinprestigetranscendorgeneratorupgrades = false
  }
  if (tier === 'prestige') {
    next.transcendnocoinorprestigeupgrades = false
    next.reincarnatenocoinorprestigeupgrades = false
    next.reincarnatenocoinprestigeortranscendupgrades = false
    next.reincarnatenocoinprestigetranscendorgeneratorupgrades = false
  }
  if (tier === 'coin') {
    next.prestigenocoinupgrades = false
    next.transcendnocoinupgrades = false
    next.transcendnocoinorprestigeupgrades = false
    next.reincarnatenocoinupgrades = false
    next.reincarnatenocoinorprestigeupgrades = false
    next.reincarnatenocoinprestigeortranscendupgrades = false
    next.reincarnatenocoinprestigetranscendorgeneratorupgrades = false
  }
  return next
}

const makeState = (overrides: Partial<UpgradesState> = {}): UpgradesState => ({
  coins: new Decimal(1e6),
  prestigePoints: new Decimal(1e6),
  transcendPoints: new Decimal(1e6),
  reincarnationPoints: new Decimal(1e6),
  // 141 slots — index 0 reserved per the upgradeRequirements comment.
  upgrades: Array.from({ length: 141 }, () => 0),
  prestigenocoinupgrades: true,
  transcendnocoinupgrades: true,
  transcendnocoinorprestigeupgrades: true,
  reincarnatenocoinupgrades: true,
  reincarnatenocoinorprestigeupgrades: true,
  reincarnatenocoinprestigeortranscendupgrades: true,
  reincarnatenocoinprestigetranscendorgeneratorupgrades: true,
  ...overrides
})

const expectStatesEqual = (a: UpgradesState, b: UpgradesState): void => {
  expect(a.coins.eq(b.coins)).toBe(true)
  expect(a.prestigePoints.eq(b.prestigePoints)).toBe(true)
  expect(a.transcendPoints.eq(b.transcendPoints)).toBe(true)
  expect(a.reincarnationPoints.eq(b.reincarnationPoints)).toBe(true)
  expect(a.upgrades).toEqual(b.upgrades)
  expect(a.prestigenocoinupgrades).toBe(b.prestigenocoinupgrades)
  expect(a.transcendnocoinupgrades).toBe(b.transcendnocoinupgrades)
  expect(a.transcendnocoinorprestigeupgrades).toBe(b.transcendnocoinorprestigeupgrades)
  expect(a.reincarnatenocoinupgrades).toBe(b.reincarnatenocoinupgrades)
  expect(a.reincarnatenocoinorprestigeupgrades).toBe(b.reincarnatenocoinorprestigeupgrades)
  expect(a.reincarnatenocoinprestigeortranscendupgrades).toBe(b.reincarnatenocoinprestigeortranscendupgrades)
  expect(a.reincarnatenocoinprestigetranscendorgeneratorupgrades).toBe(
    b.reincarnatenocoinprestigetranscendorgeneratorupgrades
  )
}

describe('parity: buyUpgrades', () => {
  const fixtures: Array<{
    label: string
    tier: UpgradeTier
    pos: number
    costExponent: number
    requirementExists: boolean
    setup?: (s: UpgradesState) => UpgradesState
  }> = [
    { label: 'coin tier successful buy', tier: 'coin', pos: 1, costExponent: 6, requirementExists: true },
    { label: 'coin tier insufficient funds (no buy, flags still flip)', tier: 'coin', pos: 1, costExponent: 6, requirementExists: true, setup: (s) => ({ ...s, coins: new Decimal(10) }) },
    { label: 'coin tier already owned', tier: 'coin', pos: 1, costExponent: 6, requirementExists: true, setup: (s) => { const upgrades = [...s.upgrades]; upgrades[1] = 1; return { ...s, upgrades } } },
    { label: 'coin tier requirement missing (early return)', tier: 'coin', pos: 1, costExponent: 6, requirementExists: false },
    { label: 'prestige tier successful buy', tier: 'prestige', pos: 22, costExponent: 15, requirementExists: true, setup: (s) => ({ ...s, prestigePoints: new Decimal(1e20) }) },
    { label: 'transcend tier successful buy', tier: 'transcend', pos: 42, costExponent: 2, requirementExists: true },
    { label: 'reincarnation tier successful buy (no flag flips)', tier: 'reincarnation', pos: 62, costExponent: 1, requirementExists: true },
    { label: 'coin tier — flag flips even when buy fails', tier: 'coin', pos: 5, costExponent: 12, requirementExists: true, setup: (s) => ({ ...s, coins: new Decimal(0) }) },
    { label: 'transcend tier — exact cost', tier: 'transcend', pos: 50, costExponent: 5, requirementExists: true, setup: (s) => ({ ...s, transcendPoints: new Decimal(1e5) }) },
    { label: 'coin tier pos=0 (zero cost, 10^0 = 1)', tier: 'coin', pos: 0, costExponent: 0, requirementExists: true },
    { label: 'prestige tier already owned mid-array', tier: 'prestige', pos: 38, costExponent: 8, requirementExists: true, setup: (s) => { const upgrades = [...s.upgrades]; upgrades[38] = 1; return { ...s, upgrades, prestigePoints: new Decimal(1e20) } } }
  ]

  it.each(fixtures)('$label', (f) => {
    const start = f.setup ? f.setup(makeState()) : makeState()
    const oldNext = applyOldBuyUpgrades(start, f.tier, f.pos, f.costExponent, f.requirementExists)
    const { state: newNext } = newBuyUpgrades(start, {
      tier: f.tier,
      pos: f.pos,
      costExponent: f.costExponent,
      requirementExists: f.requirementExists
    })
    expectStatesEqual(newNext, oldNext)
  })
})
