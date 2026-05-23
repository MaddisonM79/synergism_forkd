// Parity test for the platonic-upgrade cost table + price multiplier +
// affordability check. Old bodies transcribed verbatim from
// packages/web_ui/src/Platonic.ts (lines 39-319). The table parity is
// sampled across each tier (priceMult: 2, undefined, 10, 500, 200, etc.).
// The price multiplier and affordability check sweeps exercise:
//   - currentLevel = 0 (unscaled) vs maxLevel - 1 (max scaled)
//   - undefined priceMult ⇒ multiplier stays 1
//   - autoMode true vs false (obtainium/offerings exemption)
//   - abyssals = 0 short-circuit vs nonzero check

import { describe, expect, it } from 'vitest'
import {
  checkPlatonicUpgradeAffordability as newCheck,
  platonicResources,
  platonicUpgradeBaseCosts as newCosts,
  platonicUpgradePriceMultiplier as newPriceMult
} from '../../src/mechanics/platonicUpgradeCosts'

// ─── Old implementation (verbatim from Platonic.ts) ───────────────────────

interface OldPlatBaseCost {
  obtainium: number
  offerings: number
  cubes: number
  tesseracts: number
  hypercubes: number
  platonics: number
  abyssals: number
  maxLevel: number
  priceMult?: number
}

// Sampled entries (1, 5, 8, 15, 16, 18, 20) — covers scaled, unscaled,
// abyssal-1, abyssal-2, abyssal-64, and the max-everything index 20.
const oldCostsSample: Record<number, OldPlatBaseCost> = {
  1: {
    obtainium: 1,
    offerings: 1e45,
    cubes: 1e13,
    tesseracts: 1e6,
    hypercubes: 1e5,
    platonics: 1e4,
    abyssals: 0,
    maxLevel: 300,
    priceMult: 2
  },
  5: {
    obtainium: 1,
    offerings: 1e59,
    cubes: 1e14,
    tesseracts: 1e9,
    hypercubes: 1e8,
    platonics: 1e7,
    abyssals: 0,
    maxLevel: 1
  },
  8: {
    obtainium: 1,
    offerings: 1e64,
    cubes: 4e15,
    tesseracts: 4e9,
    hypercubes: 4e8,
    platonics: 3e7,
    abyssals: 0,
    maxLevel: 5
  },
  15: {
    obtainium: 1,
    offerings: 1e80,
    cubes: 1e23,
    tesseracts: 1e15,
    hypercubes: 1e14,
    platonics: 1e12,
    abyssals: 1,
    maxLevel: 1
  },
  16: {
    obtainium: 1,
    offerings: 1e110,
    cubes: 0,
    tesseracts: 0,
    hypercubes: 2.5e15,
    platonics: 0,
    abyssals: 0,
    maxLevel: 100,
    priceMult: 10
  },
  18: {
    obtainium: 1,
    offerings: 1e116,
    cubes: 0,
    tesseracts: 0,
    hypercubes: 1e19,
    platonics: 0,
    abyssals: 4,
    maxLevel: 40,
    priceMult: 500
  },
  20: {
    obtainium: 1,
    offerings: 1e130,
    cubes: 1e45,
    tesseracts: 1e28,
    hypercubes: 1e25,
    platonics: 1e25,
    abyssals: Math.pow(2, 30) - 1,
    maxLevel: 1
  }
}

const oldPriceMult = (priceMult: number | undefined, currentLevel: number, maxLevel: number, debuff: number): number => {
  let m = 1
  if (priceMult) {
    m = Math.pow(priceMult, Math.pow(currentLevel / (maxLevel - 1), 1.25))
  }
  return m * debuff
}

interface OldCheckInput {
  index: number
  currentLevel: number
  priceMultiplier: number
  autoMode: boolean
  currentResources: Record<string, number>
  abyssalBalance: number
}

const oldPlatonicResources = [
  'obtainium',
  'offerings',
  'cubes',
  'tesseracts',
  'hypercubes',
  'platonics',
  'abyssals'
] as const

const oldCheck = (input: OldCheckInput, baseCost: OldPlatBaseCost): Record<string, boolean> => {
  let checksum = 0
  const checks: Record<string, boolean> = {
    obtainium: false,
    offerings: false,
    cubes: false,
    tesseracts: false,
    hypercubes: false,
    platonics: false,
    abyssals: false,
    canBuy: false
  }

  for (let i = 0; i < oldPlatonicResources.length - 1; i++) {
    const key = oldPlatonicResources[i]
    if (input.autoMode && (key === 'obtainium' || key === 'offerings')) {
      checksum++
      checks[key] = true
    } else if (
      Math.floor((baseCost as unknown as Record<string, number>)[key] * input.priceMultiplier)
        <= input.currentResources[key]
    ) {
      checksum++
      checks[key] = true
    }
  }

  if (
    input.abyssalBalance >= Math.floor(baseCost.abyssals * input.priceMultiplier)
    || baseCost.abyssals === 0
  ) {
    checksum++
    checks.abyssals = true
  }

  if (checksum === oldPlatonicResources.length && input.currentLevel < baseCost.maxLevel) {
    checks.canBuy = true
  }
  return checks
}

// ─── Table parity ──────────────────────────────────────────────────────────

describe('parity: platonicUpgradeBaseCosts table', () => {
  it('contains 20 entries (1..20)', () => {
    expect(Object.keys(newCosts).map(Number).sort((a, b) => a - b))
      .toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20])
  })

  for (const index of [1, 5, 8, 15, 16, 18, 20] as const) {
    it(`index ${index} matches verbatim sample`, () => {
      expect(newCosts[index]).toEqual(oldCostsSample[index])
    })
  }

  it('platonicResources matches legacy iteration order', () => {
    expect(platonicResources).toEqual(oldPlatonicResources)
  })
})

// ─── Price multiplier parity ──────────────────────────────────────────────

describe('parity: platonicUpgradePriceMultiplier', () => {
  const cases = [
    // Unscaled (priceMult undefined) — only debuff applies.
    { priceMult: undefined, currentLevel: 0, maxLevel: 1, singularityDebuff: 1 },
    { priceMult: undefined, currentLevel: 0, maxLevel: 1, singularityDebuff: 5.5 },
    { priceMult: undefined, currentLevel: 50, maxLevel: 100, singularityDebuff: 2 },
    // priceMult = 2 (most scaled upgrades) — level=0 ⇒ multiplier 1×debuff.
    { priceMult: 2, currentLevel: 0, maxLevel: 300, singularityDebuff: 1 },
    { priceMult: 2, currentLevel: 100, maxLevel: 300, singularityDebuff: 1 },
    { priceMult: 2, currentLevel: 299, maxLevel: 300, singularityDebuff: 1 }, // near max
    { priceMult: 2, currentLevel: 150, maxLevel: 300, singularityDebuff: 3.5 },
    // priceMult = 10 (#16, #17).
    { priceMult: 10, currentLevel: 0, maxLevel: 100, singularityDebuff: 1 },
    { priceMult: 10, currentLevel: 50, maxLevel: 100, singularityDebuff: 1 },
    { priceMult: 10, currentLevel: 99, maxLevel: 100, singularityDebuff: 2 },
    // priceMult = 500 (#18) — steep scaling.
    { priceMult: 500, currentLevel: 0, maxLevel: 40, singularityDebuff: 1 },
    { priceMult: 500, currentLevel: 20, maxLevel: 40, singularityDebuff: 1 },
    { priceMult: 500, currentLevel: 39, maxLevel: 40, singularityDebuff: 1 },
    // priceMult = 200 (#19).
    { priceMult: 200, currentLevel: 25, maxLevel: 50, singularityDebuff: 1 }
  ]
  for (const c of cases) {
    it(JSON.stringify(c), () => {
      expect(newPriceMult(c)).toBe(oldPriceMult(c.priceMult, c.currentLevel, c.maxLevel, c.singularityDebuff))
    })
  }
})

// ─── Affordability check parity ───────────────────────────────────────────

describe('parity: checkPlatonicUpgradeAffordability', () => {
  // Sample test matrix: for each indexed cost, try a few player-resource states.
  const rich = {
    obtainium: 1e200,
    offerings: 1e200,
    cubes: 1e200,
    tesseracts: 1e200,
    hypercubes: 1e200,
    platonics: 1e200
  }
  const poor = {
    obtainium: 0,
    offerings: 0,
    cubes: 0,
    tesseracts: 0,
    hypercubes: 0,
    platonics: 0
  }
  const mid = {
    obtainium: 1e10,
    offerings: 1e50,
    cubes: 1e14,
    tesseracts: 1e7,
    hypercubes: 1e6,
    platonics: 1e5
  }
  const richAbyssal = 1e10
  const poorAbyssal = 0

  const scenarios = [
    // (index, currentLevel, priceMult, autoMode, resources, abyssals)
    { index: 1, currentLevel: 0, debuff: 1, autoMode: false, resources: rich, abyssals: richAbyssal, name: 'index=1 unscaled rich auto-off' },
    { index: 1, currentLevel: 0, debuff: 1, autoMode: false, resources: poor, abyssals: poorAbyssal, name: 'index=1 poor' },
    { index: 1, currentLevel: 100, debuff: 1, autoMode: false, resources: mid, abyssals: poorAbyssal, name: 'index=1 scaled mid' },
    { index: 1, currentLevel: 100, debuff: 1, autoMode: true, resources: { ...mid, obtainium: 0, offerings: 0 }, abyssals: poorAbyssal, name: 'index=1 auto-on zeros-obt-off' },
    { index: 5, currentLevel: 0, debuff: 1, autoMode: false, resources: rich, abyssals: richAbyssal, name: 'index=5 single-buy rich' },
    { index: 5, currentLevel: 1, debuff: 1, autoMode: false, resources: rich, abyssals: richAbyssal, name: 'index=5 already maxed (level=1, max=1)' },
    { index: 15, currentLevel: 0, debuff: 1, autoMode: false, resources: rich, abyssals: 0, name: 'index=15 abyssal=1 zero balance' },
    { index: 15, currentLevel: 0, debuff: 1, autoMode: false, resources: rich, abyssals: 1, name: 'index=15 abyssal=1 exact balance' },
    { index: 16, currentLevel: 50, debuff: 2, autoMode: false, resources: rich, abyssals: richAbyssal, name: 'index=16 scaled+debuff' },
    { index: 18, currentLevel: 39, debuff: 1, autoMode: false, resources: rich, abyssals: 1e10, name: 'index=18 near-max steep scaling' },
    { index: 20, currentLevel: 0, debuff: 1, autoMode: false, resources: { ...rich, cubes: 0 }, abyssals: richAbyssal, name: 'index=20 partial fail (cubes)' }
  ]

  for (const s of scenarios) {
    it(s.name, () => {
      const baseCost = newCosts[s.index]
      const priceMultiplier = newPriceMult({
        priceMult: baseCost.priceMult,
        currentLevel: s.currentLevel,
        maxLevel: baseCost.maxLevel,
        singularityDebuff: s.debuff
      })
      const newResult = newCheck({
        index: s.index,
        currentLevel: s.currentLevel,
        priceMultiplier,
        autoMode: s.autoMode,
        currentResources: s.resources,
        abyssalBalance: s.abyssals
      })
      const oldResult = oldCheck({
        index: s.index,
        currentLevel: s.currentLevel,
        priceMultiplier,
        autoMode: s.autoMode,
        currentResources: { ...s.resources, abyssals: s.abyssals },
        abyssalBalance: s.abyssals
      }, oldCostsSample[s.index as 1 | 5 | 8 | 15 | 16 | 18 | 20])
      expect(newResult).toEqual(oldResult)
    })
  }
})
