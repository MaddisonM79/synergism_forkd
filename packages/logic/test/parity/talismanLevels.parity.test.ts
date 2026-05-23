// Parity tests for the talisman rarity / level math migrated from
// packages/web_ui/src/Talismans.ts. The OLD implementations are transcribed
// below; the originals read `talismans[t].*` and `player.*` directly, the
// migration takes those as named-field inputs.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import type { TalismanCraftCosts, TalismanCraftItems } from '../../src/mechanics/talismanCosts'
import {
  affordableTalismanLevel as newAffordable,
  type AffordableTalismanLevelInput,
  computeTalismanRarity as newComputeRarity,
  type ComputeTalismanRarityInput,
  levelsUntilTalismanRarityIncrease as newLevelsUntilRarity,
  type LevelsUntilTalismanRarityIncreaseInput,
  rarityValues,
  sumOfTalismanRarities as newSumRarities
} from '../../src/mechanics/talismanLevels'

// ─── rarityValues ──────────────────────────────────────────────────────────

describe('parity: rarityValues', () => {
  it('matches the legacy table values', () => {
    const oldTable: Record<number, number> = {
      0: 0,
      1: 1,
      2: 1.2,
      3: 1.5,
      4: 1.8,
      5: 2.1,
      6: 2.5,
      7: 3,
      8: 3.25,
      9: 3.5,
      10: 4
    }
    for (const k of Object.keys(oldTable).map(Number)) {
      expect(rarityValues[k]).toBe(oldTable[k])
    }
  })
})

// ─── computeTalismanRarity ─────────────────────────────────────────────────

const oldComputeRarity = (input: ComputeTalismanRarityInput): number => {
  if (!input.isUnlocked) return 0
  const levelRatio = input.level / input.maxLevel
  let extraRarity = 0
  if (levelRatio >= 1) {
    if (levelRatio >= 2) extraRarity += 1
    if (levelRatio >= 4) extraRarity += 1
    if (levelRatio >= 8) extraRarity += 1
  }
  return 1 + Math.min(6, Math.floor(6 * levelRatio)) + extraRarity
}

describe('parity: computeTalismanRarity', () => {
  const cases: ComputeTalismanRarityInput[] = [
    // Locked
    { isUnlocked: false, level: 0, maxLevel: 180 },
    { isUnlocked: false, level: 180, maxLevel: 180 },
    { isUnlocked: false, level: 1440, maxLevel: 180 },
    // Unlocked, below tier thresholds (rarity 1 territory: levelRatio < 1/6)
    { isUnlocked: true, level: 0, maxLevel: 180 },
    { isUnlocked: true, level: 5, maxLevel: 180 },
    { isUnlocked: true, level: 29, maxLevel: 180 }, // 29/180 < 1/6 → 1
    // Rarity tier knees on a 180-cap talisman
    { isUnlocked: true, level: 30, maxLevel: 180 }, // 30/180 = 1/6 → 2
    { isUnlocked: true, level: 60, maxLevel: 180 }, // 60/180 = 2/6 → 3
    { isUnlocked: true, level: 90, maxLevel: 180 }, // 90/180 = 3/6 → 4
    { isUnlocked: true, level: 120, maxLevel: 180 }, // 120/180 = 4/6 → 5
    { isUnlocked: true, level: 150, maxLevel: 180 }, // 150/180 = 5/6 → 6
    { isUnlocked: true, level: 180, maxLevel: 180 }, // level = max, ratio = 1 → 7
    // 2x / 4x / 8x extra-rarity thresholds
    { isUnlocked: true, level: 359, maxLevel: 180 }, // < 2x → 7
    { isUnlocked: true, level: 360, maxLevel: 180 }, // = 2x → 8
    { isUnlocked: true, level: 719, maxLevel: 180 }, // < 4x → 8
    { isUnlocked: true, level: 720, maxLevel: 180 }, // = 4x → 9
    { isUnlocked: true, level: 1439, maxLevel: 180 }, // < 8x → 9
    { isUnlocked: true, level: 1440, maxLevel: 180 }, // = 8x → 10
    { isUnlocked: true, level: 5000, maxLevel: 180 }, // far past 8x → 10
    // Different maxLevel base (some talismans have different caps)
    { isUnlocked: true, level: 90, maxLevel: 90 }, // ratio = 1 → 7
    { isUnlocked: true, level: 45, maxLevel: 90 } // ratio = 0.5 = 3/6 → 4
  ]
  for (const [idx, c] of cases.entries()) {
    it(`case ${idx} (unlocked=${c.isUnlocked} level=${c.level}/${c.maxLevel})`, () => {
      expect(newComputeRarity(c)).toBe(oldComputeRarity(c))
    })
  }
})

// ─── levelsUntilTalismanRarityIncrease ─────────────────────────────────────

const oldLevelsUntilRarity = (input: LevelsUntilTalismanRarityIncreaseInput): number => {
  if (input.level >= input.maxLevel) {
    return input.levelCap - input.level
  }
  const levelReq = Math.ceil(input.maxLevel * input.currentRarity / 6)
  return levelReq - input.level
}

describe('parity: levelsUntilTalismanRarityIncrease', () => {
  const cases: LevelsUntilTalismanRarityIncreaseInput[] = [
    // Below maxLevel — uses ceil(maxLevel * rarity / 6) threshold
    { level: 0, maxLevel: 180, currentRarity: 1, levelCap: 200 }, // ceil(30) - 0 = 30
    { level: 5, maxLevel: 180, currentRarity: 1, levelCap: 200 }, // ceil(30) - 5 = 25
    { level: 30, maxLevel: 180, currentRarity: 2, levelCap: 200 }, // ceil(60) - 30 = 30
    { level: 60, maxLevel: 180, currentRarity: 3, levelCap: 200 }, // ceil(90) - 60 = 30
    { level: 90, maxLevel: 180, currentRarity: 4, levelCap: 200 }, // ceil(120) - 90 = 30
    { level: 120, maxLevel: 180, currentRarity: 5, levelCap: 200 }, // ceil(150) - 120 = 30
    { level: 150, maxLevel: 180, currentRarity: 6, levelCap: 200 }, // ceil(180) - 150 = 30
    // Non-divisible maxLevel for ceil() coverage
    { level: 0, maxLevel: 100, currentRarity: 1, levelCap: 120 }, // ceil(16.67) - 0 = 17
    { level: 17, maxLevel: 100, currentRarity: 2, levelCap: 120 }, // ceil(33.33) - 17 = 17
    // Level >= maxLevel: levelCap - level branch
    { level: 180, maxLevel: 180, currentRarity: 7, levelCap: 200 }, // 200 - 180 = 20
    { level: 195, maxLevel: 180, currentRarity: 7, levelCap: 200 }, // 200 - 195 = 5
    { level: 200, maxLevel: 180, currentRarity: 8, levelCap: 200 }, // 0
    // levelCap === maxLevel (no levelCapIncrease)
    { level: 180, maxLevel: 180, currentRarity: 7, levelCap: 180 } // 0
  ]
  for (const [idx, c] of cases.entries()) {
    it(`case ${idx} (level=${c.level} maxLevel=${c.maxLevel} rarity=${c.currentRarity} cap=${c.levelCap})`, () => {
      expect(newLevelsUntilRarity(c)).toBe(oldLevelsUntilRarity(c))
    })
  }
})

// ─── affordableTalismanLevel ───────────────────────────────────────────────

const oldAffordable = (input: AffordableTalismanLevelInput): boolean => {
  for (const item in input.costs) {
    if (input.costs[item as TalismanCraftItems].gt(input.budget[item as TalismanCraftItems].times(input.bufferMult))) {
      return false
    }
  }
  return true
}

const allZeroBudget = (): Record<TalismanCraftItems, Decimal> => ({
  shard: new Decimal(0),
  commonFragment: new Decimal(0),
  uncommonFragment: new Decimal(0),
  rareFragment: new Decimal(0),
  epicFragment: new Decimal(0),
  legendaryFragment: new Decimal(0),
  mythicalFragment: new Decimal(0)
})

const fullCostMap = (n: number): TalismanCraftCosts => ({
  shard: new Decimal(n),
  commonFragment: new Decimal(n),
  uncommonFragment: new Decimal(n),
  rareFragment: new Decimal(n),
  epicFragment: new Decimal(n),
  legendaryFragment: new Decimal(n),
  mythicalFragment: new Decimal(n)
})

describe('parity: affordableTalismanLevel', () => {
  const cases: Array<{ name: string, input: AffordableTalismanLevelInput }> = [
    // Zero cost: always affordable
    {
      name: 'zero cost / zero budget',
      input: { costs: fullCostMap(0), budget: allZeroBudget(), bufferMult: 1 }
    },
    // All affordable
    {
      name: 'budget > cost',
      input: {
        costs: fullCostMap(100),
        budget: fullCostMap(1000),
        bufferMult: 1
      }
    },
    // Exactly affordable: cost equals budget * bufferMult
    {
      name: 'cost exactly equal to budget',
      input: {
        costs: fullCostMap(100),
        budget: fullCostMap(100),
        bufferMult: 1
      }
    },
    // One item over budget
    {
      name: 'shard over budget',
      input: {
        costs: { ...fullCostMap(50), shard: new Decimal(200) },
        budget: fullCostMap(100),
        bufferMult: 1
      }
    },
    // bufferMult = 1.0001 lets a tiny imprecision through
    {
      name: 'loading buffer covers tiny imprecision',
      input: {
        costs: fullCostMap(100.005),
        budget: fullCostMap(100),
        bufferMult: 1.0001
      }
    },
    // bufferMult = 1.0001 still rejects significant over-cost
    {
      name: 'loading buffer rejects 1% over',
      input: {
        costs: fullCostMap(101),
        budget: fullCostMap(100),
        bufferMult: 1.0001
      }
    },
    // Tier-locked costs (zero in upper tiers, only shard cost set)
    {
      name: 'only shard cost, upper tiers zero',
      input: {
        costs: { ...allZeroBudget(), shard: new Decimal(50) },
        budget: { ...allZeroBudget(), shard: new Decimal(100) },
        bufferMult: 1
      }
    }
  ]
  for (const { name, input } of cases) {
    it(name, () => {
      expect(newAffordable(input)).toBe(oldAffordable(input))
    })
  }
})

// ─── sumOfTalismanRarities ─────────────────────────────────────────────────

const oldSumRarities = (rarities: readonly number[]): number => {
  let sum = 0
  for (const r of rarities) sum += r
  return sum
}

describe('parity: sumOfTalismanRarities', () => {
  const cases: readonly number[][] = [
    [],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1], // 10 unlocked at rarity 1 → 10
    [7, 7, 7, 7, 7, 7, 7, 7, 7, 7], // all maxed-out → 70
    [10, 10, 10, 10, 10, 10, 10, 10, 10, 10], // all overcapped → 100
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] // mixed
  ]
  for (const [idx, c] of cases.entries()) {
    it(`case ${idx} (n=${c.length})`, () => {
      expect(newSumRarities(c)).toBe(oldSumRarities(c))
    })
  }
})
