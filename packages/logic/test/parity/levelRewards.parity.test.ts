// Parity tests for the level-reward effect formulas, lifted from
// packages/web_ui/src/Levels.ts (the `synergismLevelRewards.<key>.effect`
// fields). Each `oldXxx` transcribes the pre-migration body verbatim
// (including the misspelled `salavePerLevel` variable from web_ui).
// Sweeps cross every documented threshold per reward — band boundaries
// at 100/200 (salvage, ants), the /20 step (quarks), the 100-level wow-cube
// kink, the cube-tier minLevel boundaries, and the late-game post-209/229/259
// luck thresholds.

import { describe, expect, it } from 'vitest'
import {
  ambrosiaLuckEffect as newAmbrosiaLuck,
  antsEffect as newAnts,
  getLevelReward as newGetLevelReward,
  type LevelRewardKey,
  levelRewards as newLevelRewards,
  obtainiumEffect as newObtainium,
  offeringsEffect as newOfferings,
  quarksEffect as newQuarks,
  redAmbrosiaLuckEffect as newRedAmbrosiaLuck,
  salvageEffect as newSalvage,
  wowCubesEffect as newWowCubes,
  wowHepteractCubesEffect as newWowHept,
  wowHyperCubesEffect as newWowHyper,
  wowOcteractsEffect as newWowOct,
  wowPlatonicCubesEffect as newWowPlat,
  wowTesseractsEffect as newWowTess
} from '../../src/mechanics/levelRewards'

// ─── Old implementations (verbatim from packages/web_ui/src/Levels.ts) ─────

const oldSalvage = (lv: number) => {
  let salvage = 0
  let salavePerLevel = 1
  let remainingLevels = lv
  while (remainingLevels >= 100) {
    salvage += salavePerLevel * 100
    remainingLevels -= 100
    salavePerLevel += 1
  }
  salvage += salavePerLevel * remainingLevels
  return salvage
}

const oldQuarks = (lv: number) => Math.pow(1.01, Math.floor(lv / 20))
const oldOfferings = (lv: number) => Math.pow(1.01, lv) * Math.pow(1.02, Math.max(0, lv - 100))
const oldObtainium = (lv: number) => Math.pow(1.01, lv - 15) * Math.pow(1.02, Math.max(0, lv - 100))

const oldAnts = (lv: number) => {
  const first100Levels = Math.min(71, lv - 59) * 25
  const next100Levels = Math.max(0, Math.min(100, lv - 100)) * 50
  const remainingLevels = Math.max(0, lv - 200) * 100
  return first100Levels + next100Levels + remainingLevels
}

const oldWowCubes = (lv: number) => (1 + (lv - 60) / 20) * Math.pow(1.07, Math.floor(lv / 10) - 6)
const oldWowTess = (lv: number) => (1 + (lv - 80) / 20) * Math.pow(1.07, Math.floor(lv / 10) - 8)
const oldWowHyper = (lv: number) => (1 + (lv - 100) / 20) * Math.pow(1.07, Math.floor(lv / 10) - 10)
const oldWowPlat = (lv: number) => (1 + (lv - 120) / 20) * Math.pow(1.07, Math.floor(lv / 10) - 12)
const oldWowHept = (lv: number) => (1 + (lv - 150) / 20) * Math.pow(1.07, Math.floor(lv / 10) - 15)
const oldWowOct = (lv: number) => (1 + (lv - 209) / 20) * Math.pow(1.02, lv - 209)
const oldAmbrosiaLuck = (lv: number) => 4 * (lv - 229)
const oldRedAmbrosiaLuck = (lv: number) => lv - 259

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// Sweeps across every documented threshold:
//   - 0 (defaultValue boundary for salvage/offerings)
//   - 15 / 20 (obtainium / quarks minLevel)
//   - 19 / 20 / 21 / 39 / 40 (quarks /20 step boundaries)
//   - 60 / 70 (ants / wowCubes minLevel)
//   - 80 / 90 / 99 / 100 / 101 (wowTesseracts minLevel, ants band)
//   - 110 / 120 / 130 / 140 / 150 / 170 (each cube tier minLevel)
//   - 199 / 200 / 201 (ants 100→remaining band)
//   - 209 / 210 / 229 / 230 / 259 / 260 (wowOcteracts / ambrosia / redAmbrosia)
//   - 300 / 500 (long-band scaling sanity)
const levelGrid = [
  0,
  1,
  10,
  14,
  15,
  16,
  19,
  20,
  21,
  39,
  40,
  41,
  59,
  60,
  61,
  69,
  70,
  71,
  79,
  80,
  81,
  89,
  90,
  91,
  99,
  100,
  101,
  109,
  110,
  111,
  119,
  120,
  121,
  139,
  140,
  141,
  149,
  150,
  151,
  169,
  170,
  171,
  199,
  200,
  201,
  208,
  209,
  210,
  211,
  228,
  229,
  230,
  231,
  258,
  259,
  260,
  261,
  300,
  500
]

// ─── Tests ─────────────────────────────────────────────────────────────────

describe('parity: salvageEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(newSalvage(lv)).toBe(oldSalvage(lv)))
})

describe('parity: quarksEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newQuarks(lv), oldQuarks(lv))).toBe(true))
})

describe('parity: offeringsEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newOfferings(lv), oldOfferings(lv))).toBe(true))
})

describe('parity: obtainiumEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newObtainium(lv), oldObtainium(lv))).toBe(true))
})

describe('parity: antsEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(newAnts(lv)).toBe(oldAnts(lv)))
})

describe('parity: wowCubesEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newWowCubes(lv), oldWowCubes(lv))).toBe(true))
})

describe('parity: wowTesseractsEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newWowTess(lv), oldWowTess(lv))).toBe(true))
})

describe('parity: wowHyperCubesEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newWowHyper(lv), oldWowHyper(lv))).toBe(true))
})

describe('parity: wowPlatonicCubesEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newWowPlat(lv), oldWowPlat(lv))).toBe(true))
})

describe('parity: wowHepteractCubesEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newWowHept(lv), oldWowHept(lv))).toBe(true))
})

describe('parity: wowOcteractsEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(closeEnough(newWowOct(lv), oldWowOct(lv))).toBe(true))
})

describe('parity: ambrosiaLuckEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(newAmbrosiaLuck(lv)).toBe(oldAmbrosiaLuck(lv)))
})

describe('parity: redAmbrosiaLuckEffect', () => {
  it.each(levelGrid)('lv=%i', (lv) => expect(newRedAmbrosiaLuck(lv)).toBe(oldRedAmbrosiaLuck(lv)))
})

// ─── getLevelReward + threshold gate ───────────────────────────────────────

// Tests that getLevelReward returns defaultValue below minLevel and the
// formula's value at/above minLevel — the threshold behaviour that callers
// rely on (Statistics.ts gates several stat lines off these).
const allRewardKeys: LevelRewardKey[] = [
  'salvage',
  'quarks',
  'offerings',
  'obtainium',
  'ants',
  'wowCubes',
  'wowTesseracts',
  'wowHyperCubes',
  'wowPlatonicCubes',
  'wowHepteractCubes',
  'wowOcteracts',
  'ambrosiaLuck',
  'redAmbrosiaLuck'
]

describe('getLevelReward: returns defaultValue below minLevel', () => {
  for (const key of allRewardKeys) {
    const data = newLevelRewards[key]
    if (data.minLevel <= 0) continue
    it(`${key} at minLevel-1 = defaultValue (${data.defaultValue})`, () => {
      expect(newGetLevelReward(key, data.minLevel - 1)).toBe(data.defaultValue)
    })
  }
})

describe('getLevelReward: returns effect at minLevel', () => {
  for (const key of allRewardKeys) {
    const data = newLevelRewards[key]
    it(`${key} at minLevel = effect(minLevel)`, () => {
      expect(newGetLevelReward(key, data.minLevel)).toBe(data.effect(data.minLevel))
    })
  }
})
