// Parity tests for ant-upgrade level computation. Old bodies transcribed
// verbatim from packages/web_ui/src/Features/Ants/AntUpgrades/lib/
// free-levels.ts + total-levels.ts.

import { describe, expect, it } from 'vitest'
import {
  calculateTrueAntLevel as newCalculateTrueAntLevel,
  computeFreeAntUpgradeLevels as newComputeFreeAntUpgradeLevels
} from '../../src/mechanics/antUpgradeLevels'

// ─── Old implementations ──────────────────────────────────────────────────

interface OldFreeLevelsInput {
  c9ReincarnationECC: number
  constantUpgrade6: number
  c11AscensionECC: number
  research97: number
  research98: number
  research102: number
  research132: number
  research200: number
  freeAntUpgradesAchievementReward: number
  challenge15BonusAntLevelValue: number
  c11Active: boolean
  c8Completions: number
  c9Completions: number
}

const oldComputeFreeAntUpgradeLevels = (input: OldFreeLevelsInput): number => {
  let bonusLevels = 0
  bonusLevels += input.c9ReincarnationECC
  bonusLevels += Math.round(2000 * (1 - Math.pow(0.999, input.constantUpgrade6)))
  bonusLevels += 12 * input.c11AscensionECC
  bonusLevels += 2 * input.research97
  bonusLevels += 2 * input.research98
  bonusLevels += input.research102
  bonusLevels += 2 * input.research132
  bonusLevels += Math.floor((1 / 200) * input.research200)
  bonusLevels += input.freeAntUpgradesAchievementReward
  bonusLevels *= input.challenge15BonusAntLevelValue

  if (input.c11Active) {
    bonusLevels += Math.floor(
      3 * input.c8Completions + 5 * input.c9Completions
    )
  }
  return bonusLevels
}

interface OldTrueLevelInput {
  currentLevel: number
  freeLevels: number
  exemptFromCorruption: boolean
  corruptionExtinctionDivisor: number
  c11Active: boolean
}

const oldCalculateTrueAntLevel = (input: OldTrueLevelInput): number => {
  const corruptionDivisor = input.exemptFromCorruption ? 1 : input.corruptionExtinctionDivisor
  if (input.c11Active) {
    return Math.min(input.currentLevel, input.freeLevels) / corruptionDivisor
  }
  return (input.currentLevel + Math.min(input.currentLevel, input.freeLevels)) / corruptionDivisor
}

// ─── computeFreeAntUpgradeLevels ─────────────────────────────────────────

const zeroInput: OldFreeLevelsInput = {
  c9ReincarnationECC: 0,
  constantUpgrade6: 0,
  c11AscensionECC: 0,
  research97: 0,
  research98: 0,
  research102: 0,
  research132: 0,
  research200: 0,
  freeAntUpgradesAchievementReward: 0,
  challenge15BonusAntLevelValue: 1,
  c11Active: false,
  c8Completions: 0,
  c9Completions: 0
}

describe('parity: computeFreeAntUpgradeLevels', () => {
  const cases: OldFreeLevelsInput[] = [
    zeroInput,
    // Each contribution individually
    { ...zeroInput, c9ReincarnationECC: 5 },
    { ...zeroInput, constantUpgrade6: 1000 },
    { ...zeroInput, c11AscensionECC: 10 },
    { ...zeroInput, research97: 50 },
    { ...zeroInput, research98: 25 },
    { ...zeroInput, research102: 100 },
    { ...zeroInput, research132: 50 },
    { ...zeroInput, research200: 1000 },
    { ...zeroInput, freeAntUpgradesAchievementReward: 200 },
    // Challenge 15 multiplier > 1 (the bonus is a small multiplier)
    { ...zeroInput, c9ReincarnationECC: 10, challenge15BonusAntLevelValue: 1.5 },
    // c11 active toggles tail bonus
    { ...zeroInput, c11Active: true, c8Completions: 10, c9Completions: 5 },
    // Realistic late-game-ish bundle
    {
      c9ReincarnationECC: 100,
      constantUpgrade6: 5000,
      c11AscensionECC: 30,
      research97: 100,
      research98: 100,
      research102: 100,
      research132: 100,
      research200: 100000,
      freeAntUpgradesAchievementReward: 1000,
      challenge15BonusAntLevelValue: 2.5,
      c11Active: false,
      c8Completions: 0,
      c9Completions: 0
    },
    // Same but c11 active
    {
      c9ReincarnationECC: 100,
      constantUpgrade6: 5000,
      c11AscensionECC: 30,
      research97: 100,
      research98: 100,
      research102: 100,
      research132: 100,
      research200: 100000,
      freeAntUpgradesAchievementReward: 1000,
      challenge15BonusAntLevelValue: 2.5,
      c11Active: true,
      c8Completions: 50,
      c9Completions: 25
    }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(newComputeFreeAntUpgradeLevels(input)).toBe(oldComputeFreeAntUpgradeLevels(input))
    })
  }
})

// ─── calculateTrueAntLevel ────────────────────────────────────────────────

describe('parity: calculateTrueAntLevel', () => {
  const cases: OldTrueLevelInput[] = [
    // Standard: currentLevel <= freeLevels (free levels not capped)
    {
      currentLevel: 50,
      freeLevels: 100,
      exemptFromCorruption: false,
      corruptionExtinctionDivisor: 1,
      c11Active: false
    },
    // currentLevel > freeLevels (free levels capped at currentLevel)
    {
      currentLevel: 150,
      freeLevels: 100,
      exemptFromCorruption: false,
      corruptionExtinctionDivisor: 1,
      c11Active: false
    },
    // Exempt from corruption (divisor=1 regardless)
    {
      currentLevel: 50,
      freeLevels: 100,
      exemptFromCorruption: true,
      corruptionExtinctionDivisor: 5,
      c11Active: false
    },
    // Corruption divisor applied
    {
      currentLevel: 100,
      freeLevels: 100,
      exemptFromCorruption: false,
      corruptionExtinctionDivisor: 2,
      c11Active: false
    },
    // c11 active: collapses to min(currentLevel, freeLevels)
    {
      currentLevel: 50,
      freeLevels: 100,
      exemptFromCorruption: false,
      corruptionExtinctionDivisor: 1,
      c11Active: true
    },
    // c11 + corruption
    {
      currentLevel: 100,
      freeLevels: 50,
      exemptFromCorruption: false,
      corruptionExtinctionDivisor: 4,
      c11Active: true
    },
    // Edge: currentLevel = 0
    {
      currentLevel: 0,
      freeLevels: 100,
      exemptFromCorruption: false,
      corruptionExtinctionDivisor: 1,
      c11Active: false
    }
  ]
  for (const input of cases) {
    it(JSON.stringify(input), () => {
      expect(newCalculateTrueAntLevel(input)).toBe(oldCalculateTrueAntLevel(input))
    })
  }
})
