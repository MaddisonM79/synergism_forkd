// Parity test for the leaderboard + quark additions to antRebornELO.ts.
// Old bodies transcribed from:
//   packages/web_ui/src/Features/Ants/AntSacrifice/Rewards/ELO/RebornELO/
//     QuarkCorner/lib/leaderboard-update.ts (LEADERBOARD_WEIGHTS)
//     QuarkCorner/lib/calculate-leaderboard.ts
//     QuarkCorner/lib/calculate-quarks.ts (per-tranche loop + quarksFromELOMult)

import { describe, expect, it } from 'vitest'
import {
  baseQuarksFromRebornELOStages as newBaseQuarks,
  calculateLeaderboardValue as newLeaderboardValue,
  calculateRebornELOThresholds,
  LEADERBOARD_WEIGHTS as newWeights,
  quarksFromELOMult as newQuarksFromELOMult,
  quarkMultiplierPerRebornELOThreshold,
  rebornELOThresholdTranches
} from '../../src/mechanics/antRebornELO'

// ─── Old implementations ──────────────────────────────────────────────────

const oldWeights = [1, 0.8, 0.6, 0.4, 0.2]

const oldLeaderboardValue = (leaderboard: Array<{ elo: number; sacrificeId: number }>): number => {
  let total = 0
  for (let i = 0; i < Math.min(leaderboard.length, oldWeights.length); i++) {
    total += leaderboard[i].elo * oldWeights[i]
  }
  return Math.floor(total)
}

const oldQuarksFromELOMult = (lifetimeTotalELOValue: number): number => {
  const numStages = calculateRebornELOThresholds(lifetimeTotalELOValue)
  return 2 - Math.pow(0.8, numStages / 100)
}

const oldBaseQuarks = (numStages: number) => {
  let baseQuarks = 0
  let remaining = numStages
  const usedNumberStagesForMult = Math.min(numStages, 1000)
  const stageMult = Math.pow(quarkMultiplierPerRebornELOThreshold, usedNumberStagesForMult)
  for (const tranch of rebornELOThresholdTranches) {
    const stagesInThisTranche = Math.min(tranch.stages, remaining)
    baseQuarks += stagesInThisTranche * tranch.quarkPerStage
    remaining -= stagesInThisTranche
    if (remaining <= 0) break
  }
  return { baseQuarks, stageMult }
}

// ─── Tests ────────────────────────────────────────────────────────────────

describe('parity: LEADERBOARD_WEIGHTS', () => {
  it('matches legacy', () => {
    expect([...newWeights]).toEqual(oldWeights)
  })
})

describe('parity: calculateLeaderboardValue', () => {
  const cases = [
    [],
    [{ elo: 0, sacrificeId: 1 }],
    [{ elo: 1000, sacrificeId: 1 }],
    [
      { elo: 1000, sacrificeId: 1 },
      { elo: 500, sacrificeId: 2 }
    ],
    [
      { elo: 10000, sacrificeId: 1 },
      { elo: 5000, sacrificeId: 2 },
      { elo: 2500, sacrificeId: 3 },
      { elo: 1000, sacrificeId: 4 },
      { elo: 500, sacrificeId: 5 }
    ],
    // More entries than weights — extras are ignored
    [
      { elo: 10000, sacrificeId: 1 },
      { elo: 5000, sacrificeId: 2 },
      { elo: 2500, sacrificeId: 3 },
      { elo: 1000, sacrificeId: 4 },
      { elo: 500, sacrificeId: 5 },
      { elo: 100, sacrificeId: 6 },
      { elo: 50, sacrificeId: 7 }
    ]
  ]
  for (const leaderboard of cases) {
    it(`length=${leaderboard.length}`, () => {
      expect(newLeaderboardValue(leaderboard)).toBe(oldLeaderboardValue(leaderboard))
    })
  }
})

describe('parity: quarksFromELOMult', () => {
  // Use varied ELO totals across tranche boundaries
  const elos = [0, 50, 100, 1_000, 100_000, 1_000_000, 50_000_000, 1_000_000_000]
  for (const elo of elos) {
    it(`elo=${elo}`, () => {
      expect(newQuarksFromELOMult(elo)).toBe(oldQuarksFromELOMult(elo))
    })
  }
})

describe('parity: baseQuarksFromRebornELOStages', () => {
  const stages = [0, 1, 50, 100, 101, 150, 200, 300, 500, 1000, 1001, 2000]
  for (const numStages of stages) {
    it(`numStages=${numStages}`, () => {
      const newRes = newBaseQuarks(numStages)
      const oldRes = oldBaseQuarks(numStages)
      expect(newRes.baseQuarks).toBe(oldRes.baseQuarks)
      expect(newRes.stageMult).toBe(oldRes.stageMult)
    })
  }
})
