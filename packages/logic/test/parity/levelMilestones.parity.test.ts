// Parity tests for the level-milestone effect formulas, lifted from
// packages/web_ui/src/Levels.ts (the `synergismLevelMilestones.<key>.effect`
// fields). Each `oldXxx` transcribes the pre-migration body verbatim.
// Sweeps cover every documented levelReq boundary (5/6/7/9/12/15/20/40/60/65/
// 80/100/130/160/180/225) plus mid-band values for the per-level scaling
// rune/talisman milestones. The eight challenge-state combinations of
// `salvageChallengeBuff` get their own enumerated test.

import { describe, expect, it } from 'vitest'
import {
  achievementTalismanEnhancementEffect as newAchTalisman,
  duplicationRuneMilestoneEffect as newDuplication,
  getLevelMilestone as newGetLevelMilestone,
  type LevelMilestoneKey,
  levelMilestones as newLevelMilestones,
  prismRuneMilestoneEffect as newPrism,
  runeAutobuyImproverEffect as newRuneAuto,
  salvageChallengeBuffEffect as newSalvageBuff,
  siRuneMilestoneEffect as newSI,
  speedRuneMilestoneEffect as newSpeed,
  thriftRuneMilestoneEffect as newThrift
} from '../../src/mechanics/levelMilestones'

// ─── Old implementations (verbatim from packages/web_ui/src/Levels.ts) ─────

const oldSpeedRune = (achievementLevel: number) => 0.5 * (achievementLevel - 19)
const oldDuplicationRune = (achievementLevel: number) => 0.4 * (achievementLevel - 39)
const oldPrismRune = (achievementLevel: number) => 0.3 * (achievementLevel - 59)
const oldThriftRune = (achievementLevel: number) => 0.2 * (achievementLevel - 79)
const oldSIRune = (achievementLevel: number) => 0.1 * (achievementLevel - 99)
const oldRuneAutobuyImprover = (achievementLevel: number) => 1.1 + 0.01 * (achievementLevel - 130)
const oldAchievementTalismanEnhancement = (achievementLevel: number) => achievementLevel

const oldSalvageChallengeBuff = (
  inAnyChallenge: boolean,
  inAscension15: boolean,
  insideSingularityChallenge: boolean
): number => {
  let baseVal = 25
  if (inAnyChallenge) {
    baseVal *= 2
  }
  if (inAscension15) {
    baseVal *= 2
  }
  if (insideSingularityChallenge) {
    baseVal *= 3
  }
  return baseVal
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// Sweeps the achievement-level range covering every documented threshold:
// 5/6/7/9/12/15/20/40/60/65/80/100/130/160/180/225 and a few mid-band.
const levelGrid = [
  0,
  1,
  4,
  5,
  6,
  7,
  8,
  9,
  10,
  11,
  12,
  13,
  14,
  15,
  19,
  20,
  21,
  39,
  40,
  41,
  59,
  60,
  64,
  65,
  79,
  80,
  99,
  100,
  129,
  130,
  131,
  159,
  160,
  179,
  180,
  224,
  225,
  226,
  300
]

const allPureMilestoneKeys: LevelMilestoneKey[] = [
  'offeringTimerScaling',
  'autoPrestige',
  'speedRune',
  'duplicationRune',
  'prismRune',
  'thriftRune',
  'SIRune',
  'tier1CrystalAutobuy',
  'tier2CrystalAutobuy',
  'tier3CrystalAutobuy',
  'tier4CrystalAutobuy',
  'tier5CrystalAutobuy',
  'achievementTalismanUnlock',
  'runeAutobuyImprover',
  'achievementTalismanEnhancement',
  'antSpeed2Autobuyer',
  'wowCubesAutobuyer',
  'ascensionScoreAutobuyer',
  'mortuus2Autobuyer'
]

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: speedRuneMilestoneEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newSpeed(lv), oldSpeedRune(lv))).toBe(true))
})

describe('parity: duplicationRuneMilestoneEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newDuplication(lv), oldDuplicationRune(lv))).toBe(true))
})

describe('parity: prismRuneMilestoneEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newPrism(lv), oldPrismRune(lv))).toBe(true))
})

describe('parity: thriftRuneMilestoneEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newThrift(lv), oldThriftRune(lv))).toBe(true))
})

describe('parity: siRuneMilestoneEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newSI(lv), oldSIRune(lv))).toBe(true))
})

describe('parity: runeAutobuyImproverEffect', () => {
  it.each(levelGrid)(
    'lv=%i',
    (lv) => expect(closeEnough(newRuneAuto(lv), oldRuneAutobuyImprover(lv))).toBe(true)
  )
})

describe('parity: achievementTalismanEnhancementEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(newAchTalisman(lv)).toBe(oldAchievementTalismanEnhancement(lv)))
})

// ─── salvageChallengeBuff: every state combination ─────────────────────────

describe('parity: salvageChallengeBuffEffect (every state combo)', () => {
  const booleanGrid = [true, false]
  for (const inAnyChallenge of booleanGrid) {
    for (const inAscension15 of booleanGrid) {
      for (const insideSingularityChallenge of booleanGrid) {
        it(`inAnyChallenge=${inAnyChallenge} inAscension15=${inAscension15} insideSing=${insideSingularityChallenge}`, () => {
          const next = newSalvageBuff({ inAnyChallenge, inAscension15, insideSingularityChallenge })
          const old = oldSalvageChallengeBuff(inAnyChallenge, inAscension15, insideSingularityChallenge)
          expect(next).toBe(old)
        })
      }
    }
  }
})

// ─── getLevelMilestone: threshold gate ─────────────────────────────────────

describe('getLevelMilestone: returns defaultValue below levelReq', () => {
  for (const key of allPureMilestoneKeys) {
    const data = newLevelMilestones[key]
    if (data.levelReq <= 0) continue
    it(`${key} at levelReq-1 = defaultValue (${data.defaultValue})`, () => {
      expect(newGetLevelMilestone(key, data.levelReq - 1)).toBe(data.defaultValue)
    })
  }
})

describe('getLevelMilestone: returns effect at levelReq', () => {
  for (const key of allPureMilestoneKeys) {
    const data = newLevelMilestones[key]
    it(`${key} at levelReq = effect(levelReq)`, () => {
      expect(newGetLevelMilestone(key, data.levelReq)).toBe(data.effect(data.levelReq))
    })
  }
})
