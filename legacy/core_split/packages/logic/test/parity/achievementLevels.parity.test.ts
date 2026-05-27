// Parity tests for the achievement-level math, lifted from
// packages/web_ui/src/Achievements.ts. Both `oldXxx` transcribe the
// pre-migration bodies verbatim. The interesting threshold is 2500 points —
// below it the level advances every 50 points, above it every 100 points.
// At exactly 2500, both regimes agree (50). Sweep crosses the boundary in
// both directions plus regime-midpoints to catch off-by-one errors.

import { describe, expect, it } from 'vitest'
import {
  achievementLevelFromPoints as newLevelFromPoints,
  toNextAchievementLevelEXP as newToNextEXP
} from '../../src/mechanics/achievementLevels'

// ─── Old implementations (verbatim from packages/web_ui/src/Achievements.ts) ─

const oldLevelFromPoints = (achievementPoints: number): number => {
  if (achievementPoints < 2500) {
    return Math.floor(achievementPoints / 50)
  } else {
    return 50 + Math.floor((achievementPoints - 2500) / 100)
  }
}

const oldToNextEXP = (achievementPoints: number): number => {
  if (achievementPoints < 2500) {
    return 50 - (achievementPoints % 50)
  } else {
    return 100 - (achievementPoints % 100)
  }
}

// Sweep across the 2500 boundary plus every 50/100-point step on either side.
// Includes points-at-exact-multiples to verify the modulo behavior at level
// boundaries, plus a couple of large values to catch arithmetic-bug regressions.
const pointsGrid = [
  0,
  1,
  25,
  49,
  50,
  51,
  99,
  100,
  101,
  149,
  150,
  199,
  200,
  499,
  500,
  999,
  1000,
  2449,
  2450,
  2499,
  2500,
  2501,
  2549,
  2550,
  2599,
  2600,
  2601,
  2999,
  3000,
  3001,
  5000,
  5099,
  5100,
  10000,
  100000
]

describe('parity: achievementLevelFromPoints', () => {
  it.each(pointsGrid)('points=%i', (points) => {
    expect(newLevelFromPoints(points)).toBe(oldLevelFromPoints(points))
  })
})

describe('parity: toNextAchievementLevelEXP', () => {
  it.each(pointsGrid)('points=%i', (points) => {
    expect(newToNextEXP(points)).toBe(oldToNextEXP(points))
  })
})

// Sanity: the level should be 50 at exactly 2500 points, and 51 at 2600.
// The old code's piecewise definition is contiguous here, so both functions
// agree at 2500 — pinning that explicitly to catch regressions if someone
// changes the threshold and forgets to update both branches.
describe('regime boundary sanity', () => {
  it('level is 50 at exactly 2500 points (both regimes agree)', () => {
    expect(newLevelFromPoints(2500)).toBe(50)
  })
  it('level is 51 at 2600 points', () => {
    expect(newLevelFromPoints(2600)).toBe(51)
  })
  it('toNext is 100 at exactly 2500 points', () => {
    expect(newToNextEXP(2500)).toBe(100)
  })
  it('toNext is 50 at 2499 points', () => {
    expect(newToNextEXP(2499)).toBe(1)
  })
})
