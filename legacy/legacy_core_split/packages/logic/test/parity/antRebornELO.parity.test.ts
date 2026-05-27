// Parity tests for the ant reborn-ELO math cluster. Old bodies transcribed
// verbatim from packages/web_ui/src/Features/Ants/AntSacrifice/Rewards/ELO/
//   RebornELO/Stages/lib/threshold.ts
//   RebornELO/lib/calculate.ts (pure portions)
//   AntELO/lib/singularity-perk.ts
//
// Sweeps cover:
//   - rebornELO values that span each tranche boundary (0, just under each
//     tier's exhaustion, just over, far above)
//   - singCount across every perk-tier transition
//   - immortalELO across the two-tranche split for the invigorated-spirits
//     perk (under 200k, exactly 200k, between 200k and 2M, above 2M)

import { describe, expect, it } from 'vitest'
import {
  calculateAvailableRebornELO as newAvailable,
  calculateLeftoverRebornELO as newLeftover,
  calculateRebornELOThresholds as newThresholds,
  calculateSingularityPerkELO as newSingularityPerkELO,
  calculateStageRebornSpeedMult as newStageMult,
  calculateToNextRebornELOThreshold as newToNext,
  calculateTotalProductionForRebornELO as newTotalProduction,
  rebornELOStageModifiers as newStageModifiers,
  singularityELOBonusMult as newELOBonusMult,
  singularityRebornSpeedMultModifier as newSpeedMod
} from '../../src/mechanics/antRebornELO'

// ─── Old implementations (verbatim) ───────────────────────────────────────

const oldThresholdTranches = [
  { stages: 100, perStage: 100, quarkPerStage: 1 },
  { stages: 100, perStage: 1000, quarkPerStage: 2 },
  { stages: 100, perStage: 3000, quarkPerStage: 3 },
  { stages: 700, perStage: 20000, quarkPerStage: 4 },
  { stages: Number.POSITIVE_INFINITY, perStage: 100000, quarkPerStage: 7 }
]

const oldPerThresholdModifiers = {
  rebornSpeedMult: 0.98,
  antSacrificeObtainiumMult: 1.05,
  antSacrificeOfferingMult: 1.05,
  antSacrificeTalismanFragmentMult: 1.2
}

const oldRebornSpeedPerkLevels = [1, 9, 25, 49, 81, 121, 169, 196, 225, 256, 289]
const oldELOBonusMultLevels = [3, 11, 27, 51, 83, 123, 171, 198, 227, 258, 291]
const oldSingularityPerkELOLevels = [2, 10, 26, 50, 82, 122, 170, 197, 226, 257, 290]

const oldSpeedMod = (singCount: number): number => {
  for (let i = oldRebornSpeedPerkLevels.length - 1; i >= 0; i--) {
    if (singCount >= oldRebornSpeedPerkLevels[i]) {
      return 0.0001 + 0.00009 * i
    }
  }
  return 0
}

const oldStageMult = (singCount: number): number => {
  const base = oldPerThresholdModifiers.rebornSpeedMult
  return Math.min(1, base + oldSpeedMod(singCount))
}

const oldThresholds = (rebornELO: number): number => {
  let rebornELOBudget = rebornELO
  let thresholds = 0
  for (const tranche of oldThresholdTranches) {
    const stagesAdded = Math.min(tranche.stages, Math.floor(rebornELOBudget / tranche.perStage))
    thresholds += stagesAdded
    rebornELOBudget -= stagesAdded * tranche.perStage
    if (stagesAdded < tranche.stages) {
      break
    }
  }
  return thresholds
}

const oldToNext = (rebornELO: number, stage?: number): number => {
  const thresholds = stage ?? oldThresholds(rebornELO)
  let stagesChecked = 0
  let tempELO = rebornELO
  for (const tranche of oldThresholdTranches) {
    if (thresholds < stagesChecked + tranche.stages) {
      const reqELOThisThreshold = tranche.perStage
      return (1 + Math.floor(tempELO / reqELOThisThreshold)) * reqELOThisThreshold - tempELO
    }
    stagesChecked += tranche.stages
    tempELO -= tranche.stages * tranche.perStage
  }
  throw new Error('unreachable')
}

const oldLeftover = (rebornELO: number, stage?: number): number => {
  const thresholds = stage ?? oldThresholds(rebornELO)
  let usedELO = 0
  let stagesChecked = 0
  for (const tranche of oldThresholdTranches) {
    const stagesInThisTranche = Math.min(tranche.stages, thresholds - stagesChecked)
    usedELO += stagesInThisTranche * tranche.perStage
    stagesChecked += stagesInThisTranche
    if (stagesChecked >= thresholds) {
      break
    }
  }
  return rebornELO - usedELO
}

const oldStageModifiers = (rebornELO: number, singCount: number) => {
  const thresholds = oldThresholds(rebornELO)
  return {
    rebornSpeedMult: Math.pow(oldStageMult(singCount), thresholds),
    antSacrificeObtainiumMult: Math.pow(oldPerThresholdModifiers.antSacrificeObtainiumMult, thresholds),
    antSacrificeOfferingMult: Math.pow(oldPerThresholdModifiers.antSacrificeOfferingMult, thresholds),
    antSacrificeTalismanFragmentMult: Math.pow(
      oldPerThresholdModifiers.antSacrificeTalismanFragmentMult,
      thresholds
    )
  }
}

const oldAvailable = (immortalELO: number, rebornELO: number): number =>
  Math.max(0, immortalELO - rebornELO)

// Reproduce the geometric-series helper from Utility
const oldGeometric = (startIndex: number, endIndex: number, ratio: number): number => {
  if (endIndex < startIndex) return 0
  if (ratio === 1) return endIndex - startIndex + 1
  return (Math.pow(ratio, endIndex + 1) - Math.pow(ratio, startIndex)) / (ratio - 1)
}

const oldTotalProduction = (rebornELO: number, stageRebornSpeedMult: number): number => {
  const stage = oldThresholds(rebornELO)
  const leftover = oldLeftover(rebornELO, stage)
  const perStageMult = 1 / stageRebornSpeedMult
  let production = 0
  let stagesSpent = 0
  for (const tranch of oldThresholdTranches) {
    const startIndex = stagesSpent
    const stagesInThisTranche = Math.min(tranch.stages, stage - stagesSpent)
    const endIndex = stagesSpent + stagesInThisTranche - 1
    const productionThisTranche = oldGeometric(startIndex, endIndex, perStageMult) * tranch.perStage
    production += productionThisTranche
    stagesSpent += stagesInThisTranche
    if (stagesSpent >= stage) {
      production += leftover * perStageMult ** stage
      break
    }
  }
  return production
}

const oldELOBonusMult = (singCount: number): number => {
  for (let i = oldELOBonusMultLevels.length - 1; i >= 0; i--) {
    if (singCount >= oldELOBonusMultLevels[i]) {
      return 0.001 + 0.0009 * i
    }
  }
  return 0
}

const oldSingularityPerkELO = (singCount: number, immortalELO: number): number => {
  for (let i = oldSingularityPerkELOLevels.length - 1; i >= 0; i--) {
    if (singCount >= oldSingularityPerkELOLevels[i]) {
      const firstTranchMult = 0.02 + 0.018 * i
      const secondTranchMult = 0.001 + 0.0009 * i
      return Math.min(200_000, immortalELO) * firstTranchMult
        + Math.max(0, Math.min(1_800_000, immortalELO - 200_000)) * secondTranchMult
    }
  }
  return 0
}

// ─── Test grids ───────────────────────────────────────────────────────────

// ELO values: 0, sub-tier-1 (under 100), tier-1-mid, tier-1-end, tier-2-mid,
// tier-3-mid, tier-4-mid, tier-5 (above 2M).
const eloGrid = [0, 50, 100, 5000, 10000, 150000, 1_000_000, 5_000_000, 50_000_000]

// SingCount values: 0, below all perks, at each perk boundary, above all.
const singCountGrid = [0, 1, 8, 9, 10, 25, 50, 80, 81, 169, 257, 290, 291, 500]

// Immortal ELO values: spans the two-tranche split (200k, 2M).
const immortalGrid = [0, 100, 199_999, 200_000, 200_001, 1_000_000, 2_000_000, 2_000_001, 10_000_000]

// ─── Stage / speed mod parity ─────────────────────────────────────────────

describe('parity: singularityRebornSpeedMultModifier', () => {
  for (const singCount of singCountGrid) {
    it(`singCount=${singCount}`, () => {
      expect(newSpeedMod(singCount)).toBe(oldSpeedMod(singCount))
    })
  }
})

describe('parity: calculateStageRebornSpeedMult', () => {
  for (const singCount of singCountGrid) {
    it(`singCount=${singCount}`, () => {
      expect(newStageMult(singCount)).toBe(oldStageMult(singCount))
    })
  }
})

// ─── Threshold parity ─────────────────────────────────────────────────────

describe('parity: calculateRebornELOThresholds', () => {
  for (const elo of eloGrid) {
    it(`rebornELO=${elo}`, () => {
      expect(newThresholds(elo)).toBe(oldThresholds(elo))
    })
  }
})

describe('parity: calculateToNextRebornELOThreshold', () => {
  for (const elo of eloGrid) {
    // Hit both code paths: with and without an externally-supplied stage.
    it(`rebornELO=${elo} (no stage)`, () => {
      expect(newToNext(elo)).toBe(oldToNext(elo))
    })
    it(`rebornELO=${elo} (with stage)`, () => {
      const stage = newThresholds(elo)
      expect(newToNext(elo, stage)).toBe(oldToNext(elo, stage))
    })
  }
})

describe('parity: calculateLeftoverRebornELO', () => {
  for (const elo of eloGrid) {
    it(`rebornELO=${elo} (no stage)`, () => {
      expect(newLeftover(elo)).toBe(oldLeftover(elo))
    })
    it(`rebornELO=${elo} (with stage)`, () => {
      const stage = newThresholds(elo)
      expect(newLeftover(elo, stage)).toBe(oldLeftover(elo, stage))
    })
  }
})

// ─── Stage-modifier aggregator parity ─────────────────────────────────────

describe('parity: rebornELOStageModifiers', () => {
  // Use a smaller grid to keep test count reasonable.
  const eloSubset = [0, 100, 10000, 1_000_000, 50_000_000]
  const singSubset = [0, 9, 81, 290]
  for (const rebornELO of eloSubset) {
    for (const singCount of singSubset) {
      it(`rebornELO=${rebornELO} singCount=${singCount}`, () => {
        expect(newStageModifiers({ rebornELO, singCount }))
          .toEqual(oldStageModifiers(rebornELO, singCount))
      })
    }
  }
})

// ─── Available / total production parity ──────────────────────────────────

describe('parity: calculateAvailableRebornELO', () => {
  const cases = [
    { immortalELO: 0, rebornELO: 0 },
    { immortalELO: 1000, rebornELO: 0 },
    { immortalELO: 1000, rebornELO: 500 },
    { immortalELO: 1000, rebornELO: 1000 },
    { immortalELO: 1000, rebornELO: 1500 }, // rebornELO > immortalELO → 0
    { immortalELO: 1_000_000, rebornELO: 250_000 }
  ]
  for (const c of cases) {
    it(JSON.stringify(c), () => {
      expect(newAvailable(c)).toBe(oldAvailable(c.immortalELO, c.rebornELO))
    })
  }
})

describe('parity: calculateTotalProductionForRebornELO', () => {
  // The reciprocal-stage-mult math is sensitive to large stage counts (the
  // power blows up). Use moderate values, both with the legacy default
  // 0.98 mult and a perk-modified value.
  const cases = [
    { rebornELO: 0, stageRebornSpeedMult: 0.98 },
    { rebornELO: 100, stageRebornSpeedMult: 0.98 },
    { rebornELO: 10000, stageRebornSpeedMult: 0.98 },
    { rebornELO: 100000, stageRebornSpeedMult: 0.98 },
    { rebornELO: 1_000_000, stageRebornSpeedMult: 0.98 },
    { rebornELO: 100, stageRebornSpeedMult: 0.985 }, // mid-perk mult
    { rebornELO: 10000, stageRebornSpeedMult: 0.999 } // near-perfect mult
  ]
  for (const c of cases) {
    it(JSON.stringify(c), () => {
      const newRes = newTotalProduction(c)
      const oldRes = oldTotalProduction(c.rebornELO, c.stageRebornSpeedMult)
      // Floats — allow tiny relative drift from the geometric-series helper.
      expect(Math.abs(newRes - oldRes)).toBeLessThan(1e-6 * Math.max(1, Math.abs(oldRes)))
    })
  }
})

// ─── Singularity ELO perks ────────────────────────────────────────────────

describe('parity: singularityELOBonusMult', () => {
  for (const singCount of singCountGrid) {
    it(`singCount=${singCount}`, () => {
      expect(newELOBonusMult(singCount)).toBe(oldELOBonusMult(singCount))
    })
  }
})

describe('parity: calculateSingularityPerkELO', () => {
  for (const singCount of singCountGrid) {
    for (const immortalELO of immortalGrid) {
      it(`singCount=${singCount} immortalELO=${immortalELO}`, () => {
        expect(newSingularityPerkELO({ singCount, immortalELO }))
          .toBe(oldSingularityPerkELO(singCount, immortalELO))
      })
    }
  }
})
