// Parity tests for the achievement-points pure math migrated from
// packages/web_ui/src/Achievements.ts. The OLD implementations are
// transcribed below; the originals read `player.*` or closed over the
// `cached` parameter the dispatch map fed them — the migration takes the
// already-extracted player inputs as named arguments.

import { describe, expect, it } from 'vitest'
import {
  ambrosiaCountPoints as newAmbrosia,
  antMasteryPoints as newAntMastery,
  type ComputeAchievementPointsInput,
  computeAchievementPoints as newComputePoints,
  exaltPoints as newExalt,
  freeRuneLevelPoints as newFreeRune,
  getAchievementQuarks as newAchQuarks,
  maxedUpgradeFamilyPoints as newMaxedFamily,
  rebornELOPoints as newRebornELO,
  redAmbrosiaCountPoints as newRedAmbrosia,
  runeLevelPoints as newRuneLevel,
  singularityCountPoints as newSingCount,
  talismanRarityPoints as newTalismanRarity
} from '../../src/mechanics/achievementPoints'

// ─── runeLevelPoints ───────────────────────────────────────────────────────

const oldRuneLevel = (cached: number): number => {
  return Math.min(200, Math.floor(cached / 1000)) + Math.min(400, Math.floor(cached / 2500))
    + Math.min(400, Math.floor(cached / 12500))
}

describe('parity: runeLevelPoints', () => {
  const cases = [0, 1, 999, 1000, 2499, 2500, 12499, 12500, 100_000, 200_000, 500_000, 5_000_000, 10_000_000]
  for (const cached of cases) {
    it(`cached=${cached}`, () => {
      expect(newRuneLevel(cached)).toBe(oldRuneLevel(cached))
    })
  }
})

// ─── freeRuneLevelPoints ───────────────────────────────────────────────────

const oldFreeRune = (cached: number): number => {
  return Math.min(100, Math.floor(cached / 250)) + Math.min(200, Math.floor(cached / 750))
    + Math.min(200, Math.floor(cached / 2500))
}

describe('parity: freeRuneLevelPoints', () => {
  const cases = [0, 249, 250, 749, 750, 2499, 2500, 25_000, 50_000, 150_000, 500_000, 1_000_000]
  for (const cached of cases) {
    it(`cached=${cached}`, () => {
      expect(newFreeRune(cached)).toBe(oldFreeRune(cached))
    })
  }
})

// ─── antMasteryPoints ──────────────────────────────────────────────────────

const oldAntMastery = (masteries: readonly number[]): number => {
  let pointValue = 0
  for (const m of masteries) {
    pointValue += 3 * m
    if (m >= 12) {
      pointValue += 4
    }
  }
  return pointValue
}

describe('parity: antMasteryPoints', () => {
  const cases: number[][] = [
    [],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], // 36
    [11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11], // 12 * 33 = 396, no +4s
    [12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12], // 12 * (36 + 4) = 480
    [13, 14, 15, 12, 11, 10, 5, 6, 7, 8, 9, 0], // mixed
    [100] // single big
  ]
  for (const [idx, masteries] of cases.entries()) {
    it(`case ${idx} (n=${masteries.length})`, () => {
      expect(newAntMastery(masteries)).toBe(oldAntMastery(masteries))
    })
  }
})

// ─── rebornELOPoints ───────────────────────────────────────────────────────

const oldRebornELO = (leaderboardELO: number): number => {
  return Math.min(100, Math.floor(leaderboardELO / 100))
    + Math.min(150, Math.floor(leaderboardELO / 1000))
    + Math.min(150, Math.floor(leaderboardELO / 9000))
    + Math.min(200, Math.floor(leaderboardELO / 75000))
    + Math.min(400, Math.floor(leaderboardELO / 150000))
}

describe('parity: rebornELOPoints', () => {
  const cases = [
    0,
    99,
    100,
    999,
    1000,
    8999,
    9000,
    74_999,
    75_000,
    149_999,
    150_000,
    1_000_000,
    10_000_000,
    100_000_000
  ]
  for (const elo of cases) {
    it(`elo=${elo}`, () => {
      expect(newRebornELO(elo)).toBe(oldRebornELO(elo))
    })
  }
})

// ─── singularityCountPoints ────────────────────────────────────────────────

const oldSingCount = (h: number): number => {
  return 9 * h + 3 * Math.max(0, h - 100) + 3 * Math.max(0, h - 200)
}

describe('parity: singularityCountPoints', () => {
  const cases = [0, 1, 50, 99, 100, 101, 150, 199, 200, 201, 250, 300, 500, 1000]
  for (const h of cases) {
    it(`highest=${h}`, () => {
      expect(newSingCount(h)).toBe(oldSingCount(h))
    })
  }
})

// ─── ambrosiaCountPoints ───────────────────────────────────────────────────

const oldAmbrosia = (cached: number): number => {
  return Math.min(200, Math.floor(cached / 100))
    + Math.min(200, Math.floor(cached / 10000))
    + Math.min(400, Math.floor(400 * Math.sqrt(cached / 1e8)))
}

describe('parity: ambrosiaCountPoints', () => {
  const cases = [0, 99, 100, 9999, 10_000, 99_999, 100_000, 1e6, 1e7, 1e8, 1e9, 1e10, 1e12]
  for (const cached of cases) {
    it(`cached=${cached}`, () => {
      expect(newAmbrosia(cached)).toBe(oldAmbrosia(cached))
    })
  }
})

// ─── redAmbrosiaCountPoints ────────────────────────────────────────────────

const oldRedAmbrosia = (cached: number): number => {
  return Math.min(200, Math.floor(cached / 25))
    + Math.min(200, Math.floor(cached / 2500))
    + Math.min(400, Math.floor(400 * cached / 5e6))
    + Math.min(200, Math.floor(200 * cached / 1.25e7))
}

describe('parity: redAmbrosiaCountPoints', () => {
  const cases = [0, 24, 25, 2499, 2500, 250_000, 5e6, 1.25e7, 5e7, 1e8, 1e9]
  for (const cached of cases) {
    it(`cached=${cached}`, () => {
      expect(newRedAmbrosia(cached)).toBe(oldRedAmbrosia(cached))
    })
  }
})

// ─── talismanRarityPoints ──────────────────────────────────────────────────

describe('parity: talismanRarityPoints', () => {
  const cases = [0, 1, 7, 50, 100, 1000]
  for (const cached of cases) {
    it(`cached=${cached}`, () => {
      expect(newTalismanRarity(cached)).toBe(5 * cached)
    })
  }
})

// ─── exaltPoints ───────────────────────────────────────────────────────────

const oldExalt = (rewardAPs: readonly number[]): number => {
  let pointValue = 0
  for (const ap of rewardAPs) pointValue += ap
  return pointValue
}

describe('parity: exaltPoints', () => {
  const cases: number[][] = [
    [],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [10, 20, 30, 40, 50, 60, 70, 80, 90], // 450
    [100, 100, 100, 100, 100, 100, 100, 100, 100], // 900
    [25, 0, 50, 75, 0, 100, 125, 0, 25] // partial completion
  ]
  for (const [idx, c] of cases.entries()) {
    it(`case ${idx}`, () => {
      expect(newExalt(c)).toBe(oldExalt(c))
    })
  }
})

// ─── maxedUpgradeFamilyPoints ──────────────────────────────────────────────

describe('parity: maxedUpgradeFamilyPoints', () => {
  const cases: Array<[number, number]> = [
    [0, 5],
    [10, 5], // GQ side: 50
    [80, 5], // 400
    [0, 8],
    [50, 8], // octeract: 400
    [0, 10],
    [20, 10], // red ambrosia: 200
    [100, 10] // 1000
  ]
  for (const [count, points] of cases) {
    it(`count=${count} pts=${points}`, () => {
      expect(newMaxedFamily(count, points)).toBe(count * points)
    })
  }
})

// ─── getAchievementQuarks ──────────────────────────────────────────────────

const oldAchQuarks = (globalQuarkMultiplier: number): number => {
  let actualMultiplier = globalQuarkMultiplier
  if (actualMultiplier > 100) {
    actualMultiplier = Math.pow(100, 0.6) * Math.pow(actualMultiplier, 0.4)
  }
  return Math.floor(5 * actualMultiplier)
}

describe('parity: getAchievementQuarks', () => {
  const cases = [1, 5, 10, 50, 99.99, 100, 100.01, 500, 1000, 10_000, 1e6, 1e9]
  for (const mult of cases) {
    it(`mult=${mult}`, () => {
      expect(newAchQuarks(mult)).toBe(oldAchQuarks(mult))
    })
  }
})

// ─── computeAchievementPoints ──────────────────────────────────────────────
//
// The old aggregator walks the achievements array plus the progressive map.
// The new signature takes pre-extracted parallel arrays.

const oldComputePoints = (input: ComputeAchievementPointsInput): number => {
  let points = 0
  for (let i = 0; i < input.pointValues.length; i++) {
    if (input.savedAchievements[i]) points += input.pointValues[i]
  }
  for (const a of input.progressivePointsAwarded) points += a
  return points
}

describe('parity: computeAchievementPoints', () => {
  const cases: Array<{ name: string, input: ComputeAchievementPointsInput }> = [
    {
      name: 'empty',
      input: { pointValues: [], savedAchievements: [], progressivePointsAwarded: [] }
    },
    {
      name: 'all locked',
      input: {
        pointValues: [5, 10, 15, 20, 25],
        savedAchievements: [0, 0, 0, 0, 0],
        progressivePointsAwarded: []
      }
    },
    {
      name: 'all unlocked',
      input: {
        pointValues: [5, 10, 15, 20, 25], // sum 75
        savedAchievements: [1, 1, 1, 1, 1],
        progressivePointsAwarded: []
      }
    },
    {
      name: 'partial + progressives',
      input: {
        pointValues: [5, 10, 15, 20, 25],
        savedAchievements: [1, 0, 1, 0, 1], // 5 + 15 + 25 = 45
        progressivePointsAwarded: [100, 200, 50] // 350
      }
    },
    {
      name: 'savedAchievements truthy non-1 still counts',
      input: {
        pointValues: [5, 10, 15],
        savedAchievements: [2, 0, 5],
        progressivePointsAwarded: []
      }
    },
    {
      name: 'realistic-shaped (subset)',
      input: {
        pointValues: Array.from({ length: 100 }, (_, i) => 5 + i % 30),
        savedAchievements: Array.from({ length: 100 }, (_, i) => (i % 3 === 0 ? 1 : 0)),
        progressivePointsAwarded: [800, 1000, 360, 1000, 3600, 500, 800, 1000]
      }
    }
  ]
  for (const { name, input } of cases) {
    it(name, () => {
      expect(newComputePoints(input)).toBe(oldComputePoints(input))
    })
  }
})
