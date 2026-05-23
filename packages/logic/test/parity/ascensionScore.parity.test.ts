// Parity tests for the ascension-score and corruption-stuff transforms
// migrated from packages/web_ui/src/Calculate.ts. The OLD implementations
// are transcribed verbatim below with all `player.*` / `G.*` reads lifted
// into explicit parameters.

import { describe, expect, it } from 'vitest'
import {
  CalcCorruptionStuff as newCalcCorruption,
  calculateAscensionScore as newCalcAscensionScore,
  computeAscensionScoreBonusMultiplier as newComputeBonus
} from '../../src/mechanics/calculate'

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (!isFinite(a) || !isFinite(b)) return a === b
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// ─── computeAscensionScoreBonusMultiplier ─────────────────────────────────

interface OldBonusInputs {
  challenge15ScoreReward: number
  platonicBlessingMult: number
  campaignAscensionScoreMult: number
  finiteDescentAscensionScore: number
  cubeUpgrade21: number
  cubeUpgrade31: number
  cubeUpgrade41: number
  ascensionScoreAchievementReward: number
  masterPackAscensionScoreMult: number
  eventBuff: number // 0 if no event active
}

const oldComputeBonus = (i: OldBonusInputs): number => {
  let multiplier = 1
  multiplier *= i.challenge15ScoreReward
  multiplier *= i.platonicBlessingMult
  multiplier *= i.campaignAscensionScoreMult
  multiplier *= i.finiteDescentAscensionScore
  if (i.cubeUpgrade21 > 0) multiplier *= 1 + 0.05 * i.cubeUpgrade21
  if (i.cubeUpgrade31 > 0) multiplier *= 1 + 0.05 * i.cubeUpgrade31
  if (i.cubeUpgrade41 > 0) multiplier *= 1 + 0.05 * i.cubeUpgrade41
  multiplier *= i.ascensionScoreAchievementReward
  multiplier *= i.masterPackAscensionScoreMult
  // The web_ui code guards on `G.isEvent` before adding; here the caller
  // passes 0 when no event is active.
  multiplier *= 1 + i.eventBuff
  return multiplier
}

describe('parity: computeAscensionScoreBonusMultiplier', () => {
  const cases: OldBonusInputs[] = [
    // Identity-ish baseline
    {
      challenge15ScoreReward: 1,
      platonicBlessingMult: 1,
      campaignAscensionScoreMult: 1,
      finiteDescentAscensionScore: 1,
      cubeUpgrade21: 0,
      cubeUpgrade31: 0,
      cubeUpgrade41: 0,
      ascensionScoreAchievementReward: 1,
      masterPackAscensionScoreMult: 1,
      eventBuff: 0
    },
    // Cube upgrades trigger 1 + 0.05*n branch
    {
      challenge15ScoreReward: 2,
      platonicBlessingMult: 1.5,
      campaignAscensionScoreMult: 1.2,
      finiteDescentAscensionScore: 1.1,
      cubeUpgrade21: 10,
      cubeUpgrade31: 5,
      cubeUpgrade41: 2,
      ascensionScoreAchievementReward: 1.5,
      masterPackAscensionScoreMult: 1.25,
      eventBuff: 0
    },
    // Event active
    {
      challenge15ScoreReward: 1,
      platonicBlessingMult: 1,
      campaignAscensionScoreMult: 1,
      finiteDescentAscensionScore: 1,
      cubeUpgrade21: 0,
      cubeUpgrade31: 0,
      cubeUpgrade41: 0,
      ascensionScoreAchievementReward: 1,
      masterPackAscensionScoreMult: 1,
      eventBuff: 0.5
    },
    // All bumps + event
    {
      challenge15ScoreReward: 3,
      platonicBlessingMult: 2,
      campaignAscensionScoreMult: 1.5,
      finiteDescentAscensionScore: 1.3,
      cubeUpgrade21: 20,
      cubeUpgrade31: 15,
      cubeUpgrade41: 10,
      ascensionScoreAchievementReward: 2,
      masterPackAscensionScoreMult: 1.5,
      eventBuff: 0.25
    }
  ]

  for (const [i, input] of cases.entries()) {
    it(`case ${i}`, () => {
      expect(closeEnough(newComputeBonus(input), oldComputeBonus(input))).toBe(true)
    })
  }
})

// ─── calculateAscensionScore ──────────────────────────────────────────────

interface OldAscScoreInputs {
  highestChallengeCompletions: readonly number[]
  cubeUpgrade56: number
  cubeUpgrade39: number
  platonicUpgrade5: number
  platonicUpgrade10: number
  corruptionMultiplier: number
  antUpgradeAscensionScoreBase: number
  expertPackAscensionScoreMult: number
  bonusMultiplier: number
}

const oldChallengeScoreArrays2 = [0, 10, 12, 15, 20, 30, 80, 120, 180, 300, 450]
const oldChallengeScoreArrays3 = [0, 20, 30, 50, 100, 200, 250, 300, 400, 500, 750]
const oldChallengeScoreArrays4 = [0, 10000, 10000, 10000, 10000, 10000, 2000, 3000, 4000, 5000, 7500]

const oldCalcAscensionScore = (input: OldAscScoreInputs) => {
  let baseScore = 0
  const challengeScoreArrays1 = [0, 8, 10, 12, 15, 20, 60, 80, 120, 180, 300]
  challengeScoreArrays1[1] += input.cubeUpgrade56
  challengeScoreArrays1[2] += input.cubeUpgrade56
  challengeScoreArrays1[3] += input.cubeUpgrade56

  for (let i = 1; i <= 10; i++) {
    const c = input.highestChallengeCompletions[i]
    baseScore += challengeScoreArrays1[i] * c
    if (i <= 5 && c >= 75) {
      baseScore += oldChallengeScoreArrays2[i] * (c - 75)
      if (c >= 750) baseScore += oldChallengeScoreArrays3[i] * (c - 750)
      if (c >= 9000) baseScore += oldChallengeScoreArrays4[i] * (c - 9000)
    }
    if (i <= 10 && i > 5 && c >= 25) {
      baseScore += oldChallengeScoreArrays2[i] * (c - 25)
      if (c >= 60) baseScore += oldChallengeScoreArrays3[i] * (c - 60)
    }
  }

  baseScore += input.antUpgradeAscensionScoreBase

  baseScore *= Math.pow(
    1.03 + 0.005 * input.cubeUpgrade39 + 0.0025 * (input.platonicUpgrade5 + input.platonicUpgrade10),
    input.highestChallengeCompletions[10]
  )

  let effectiveScore = baseScore * input.corruptionMultiplier * input.bonusMultiplier
  if (effectiveScore > 1e23) effectiveScore = Math.pow(effectiveScore, 0.5) * Math.pow(1e23, 0.5)
  effectiveScore *= input.expertPackAscensionScoreMult

  return {
    baseScore,
    corruptionMultiplier: input.corruptionMultiplier,
    bonusMultiplier: input.bonusMultiplier,
    effectiveScore
  }
}

describe('parity: calculateAscensionScore', () => {
  // Each case exercises a different combination of thresholds:
  // - completions below every band (only baseScoreArray contributes)
  // - exactly at the 75/750/9000 transcend boundaries
  // - exactly at the 25/60 reincarnation boundaries
  // - past the 1e23 effectiveScore softcap
  const cases: OldAscScoreInputs[] = [
    // Baseline: zeroes everywhere
    {
      highestChallengeCompletions: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
      cubeUpgrade56: 0,
      cubeUpgrade39: 0,
      platonicUpgrade5: 0,
      platonicUpgrade10: 0,
      corruptionMultiplier: 1,
      antUpgradeAscensionScoreBase: 0,
      expertPackAscensionScoreMult: 1,
      bonusMultiplier: 1
    },
    // Below 75, transcend baseline only
    {
      highestChallengeCompletions: [0, 50, 50, 50, 50, 50, 0, 0, 0, 0, 0],
      cubeUpgrade56: 0,
      cubeUpgrade39: 0,
      platonicUpgrade5: 0,
      platonicUpgrade10: 0,
      corruptionMultiplier: 1,
      antUpgradeAscensionScoreBase: 0,
      expertPackAscensionScoreMult: 1,
      bonusMultiplier: 1
    },
    // Cross 75 threshold for transcend, 25 for reincarnation
    {
      highestChallengeCompletions: [0, 100, 100, 100, 100, 100, 50, 50, 50, 50, 50],
      cubeUpgrade56: 0,
      cubeUpgrade39: 0,
      platonicUpgrade5: 0,
      platonicUpgrade10: 0,
      corruptionMultiplier: 1,
      antUpgradeAscensionScoreBase: 100,
      expertPackAscensionScoreMult: 1,
      bonusMultiplier: 1
    },
    // Cross 750 transcend + 60 reincarnation thresholds, cubeUpgrade56 active
    {
      highestChallengeCompletions: [0, 800, 800, 800, 100, 100, 80, 80, 80, 80, 80],
      cubeUpgrade56: 5,
      cubeUpgrade39: 0,
      platonicUpgrade5: 0,
      platonicUpgrade10: 0,
      corruptionMultiplier: 1.5,
      antUpgradeAscensionScoreBase: 0,
      expertPackAscensionScoreMult: 1.2,
      bonusMultiplier: 1.1
    },
    // Cross 9000 transcend threshold, max C10 exponent contributors
    {
      highestChallengeCompletions: [0, 9100, 9100, 100, 100, 100, 80, 80, 80, 80, 50],
      cubeUpgrade56: 10,
      cubeUpgrade39: 5,
      platonicUpgrade5: 1,
      platonicUpgrade10: 1,
      corruptionMultiplier: 2,
      antUpgradeAscensionScoreBase: 1000,
      expertPackAscensionScoreMult: 1.5,
      bonusMultiplier: 1.3
    },
    // Force effectiveScore past 1e23 softcap
    {
      highestChallengeCompletions: [0, 9100, 9100, 9100, 9100, 9100, 100, 100, 100, 100, 500],
      cubeUpgrade56: 10,
      cubeUpgrade39: 5,
      platonicUpgrade5: 1,
      platonicUpgrade10: 1,
      corruptionMultiplier: 1e10,
      antUpgradeAscensionScoreBase: 0,
      expertPackAscensionScoreMult: 2,
      bonusMultiplier: 1e10
    }
  ]

  for (const [i, input] of cases.entries()) {
    it(`case ${i}`, () => {
      const newRes = newCalcAscensionScore(input)
      const oldRes = oldCalcAscensionScore(input)
      expect(closeEnough(newRes.baseScore, oldRes.baseScore)).toBe(true)
      expect(closeEnough(newRes.corruptionMultiplier, oldRes.corruptionMultiplier)).toBe(true)
      expect(closeEnough(newRes.bonusMultiplier, oldRes.bonusMultiplier)).toBe(true)
      expect(closeEnough(newRes.effectiveScore, oldRes.effectiveScore)).toBe(true)
    })
  }
})

// ─── CalcCorruptionStuff ──────────────────────────────────────────────────

describe('parity: CalcCorruptionStuff', () => {
  const cases = [
    // Below every threshold: only cubes/tess
    {
      label: 'below all thresholds',
      effectiveScore: 1e3,
      cubeMult: 5,
      tessMult: 2,
      hyperMult: 3,
      platMult: 4,
      heptMult: 5,
      hepteractsUnlocked: 0,
      singularityCount: 0
    },
    // Cross 1e5 — tess base bumps to 1.5
    {
      label: 'tess base bumps',
      effectiveScore: 2e5,
      cubeMult: 10,
      tessMult: 2,
      hyperMult: 1,
      platMult: 1,
      heptMult: 1,
      hepteractsUnlocked: 0,
      singularityCount: 0
    },
    // Cross 1e9 — hypercubes unlock
    {
      label: 'hyper unlocks',
      effectiveScore: 5e9,
      cubeMult: 100,
      tessMult: 50,
      hyperMult: 25,
      platMult: 1,
      heptMult: 1,
      hepteractsUnlocked: 0,
      singularityCount: 0
    },
    // Cross 2.666e12 — platonic unlocks
    {
      label: 'platonic unlocks',
      effectiveScore: 1e13,
      cubeMult: 1e5,
      tessMult: 1e4,
      hyperMult: 1e3,
      platMult: 100,
      heptMult: 1,
      hepteractsUnlocked: 0,
      singularityCount: 0
    },
    // Cross 1.666e17 + hepteracts unlocked
    {
      label: 'hepteract unlocks',
      effectiveScore: 5e17,
      cubeMult: 1e8,
      tessMult: 1e7,
      hyperMult: 1e6,
      platMult: 1e4,
      heptMult: 100,
      hepteractsUnlocked: 1,
      singularityCount: 0
    },
    // Hepteract threshold met but flag locked
    {
      label: 'hept threshold but flag locked',
      effectiveScore: 5e17,
      cubeMult: 1,
      tessMult: 1,
      hyperMult: 1,
      platMult: 1,
      heptMult: 100,
      hepteractsUnlocked: 0,
      singularityCount: 0
    },
    // singularityCount floor for tesseracts
    {
      label: 'sing count > tess gain',
      effectiveScore: 1e6,
      cubeMult: 1,
      tessMult: 5,
      hyperMult: 1,
      platMult: 1,
      heptMult: 1,
      hepteractsUnlocked: 0,
      singularityCount: 50
    },
    // 1e300 ceiling clamps — use 1e308 (the largest finite double-ish exponent)
    // to push the floored values past 1e300 without overflowing to Infinity.
    {
      label: '1e300 clamps',
      effectiveScore: 1e30,
      cubeMult: 1e308,
      tessMult: 1e308,
      hyperMult: 1e308,
      platMult: 1e308,
      heptMult: 1e308,
      hepteractsUnlocked: 1,
      singularityCount: 0
    }
  ]

  for (const c of cases) {
    it(c.label, () => {
      const scores = {
        baseScore: c.effectiveScore / 2,
        corruptionMultiplier: 1,
        bonusMultiplier: 1,
        effectiveScore: c.effectiveScore
      }
      const newRes = newCalcCorruption({
        scores,
        cubeMultiplier: c.cubeMult,
        tesseractMultiplier: c.tessMult,
        hypercubeMultiplier: c.hyperMult,
        platonicMultiplier: c.platMult,
        hepteractMultiplier: c.heptMult,
        hepteractsUnlocked: c.hepteractsUnlocked,
        singularityCount: c.singularityCount
      })

      // OLD transcription
      const cubeGain = c.cubeMult
      let tesseractGain = 1
      if (c.effectiveScore >= 100000) tesseractGain += 0.5
      tesseractGain *= c.tessMult
      let hypercubeGain = c.effectiveScore >= 1e9 ? 1 : 0
      hypercubeGain *= c.hyperMult
      let platonicGain = c.effectiveScore >= 2.666e12 ? 1 : 0
      platonicGain *= c.platMult
      let hepteractGain = c.hepteractsUnlocked && c.effectiveScore >= 1.666e17 ? 1 : 0
      hepteractGain *= c.heptMult
      const oldRes = {
        wowCubes: Math.min(1e300, Math.floor(cubeGain)),
        wowTesseracts: Math.min(1e300, Math.max(c.singularityCount, Math.floor(tesseractGain))),
        wowHypercubes: Math.min(1e300, Math.floor(hypercubeGain)),
        wowPlatonicCubes: Math.min(1e300, Math.floor(platonicGain)),
        wowHepteracts: Math.min(1e300, Math.floor(hepteractGain)),
        baseScore: Math.floor(scores.baseScore),
        bonusMultiplier: scores.bonusMultiplier,
        corruptionMultiplier: scores.corruptionMultiplier,
        effectiveScore: Math.floor(c.effectiveScore)
      }

      expect(newRes.wowCubes).toBe(oldRes.wowCubes)
      expect(newRes.wowTesseracts).toBe(oldRes.wowTesseracts)
      expect(newRes.wowHypercubes).toBe(oldRes.wowHypercubes)
      expect(newRes.wowPlatonicCubes).toBe(oldRes.wowPlatonicCubes)
      expect(newRes.wowHepteracts).toBe(oldRes.wowHepteracts)
      expect(newRes.baseScore).toBe(oldRes.baseScore)
      expect(newRes.bonusMultiplier).toBe(oldRes.bonusMultiplier)
      expect(newRes.corruptionMultiplier).toBe(oldRes.corruptionMultiplier)
      expect(newRes.effectiveScore).toBe(oldRes.effectiveScore)
    })
  }
})
