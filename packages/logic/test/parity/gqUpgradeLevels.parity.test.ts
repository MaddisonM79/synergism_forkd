// Parity tests for gqUpgradeLevels — `gqFreeLevelMultiplier`,
// `gqUpgradeFreeLevelSoftcap`, `computeGQUpgradeMaxLevel`, and the gated
// `actualGQUpgradeTotalLevels`, all lifted from
// packages/web_ui/src/singularity.ts.
//
// Sweeps cover: the softcap kink at level == baseRealFreeLevels (where
// `min(level, baseRealFree) + sqrt(max(0, baseRealFree - level))` transitions
// from the linear branch to the sqrt branch), every overclock-perks
// threshold for the max-level cap, all four challenge gates and the
// platonicDelta-specific gate, the improvedFree polynomial path with several
// exponents.

import { describe, expect, it } from 'vitest'
import {
  actualGQUpgradeTotalLevels as newActualLevels,
  computeGQUpgradeMaxLevel as newMaxLevel,
  gqFreeLevelMultiplier as newFreeLevelMult,
  gqUpgradeFreeLevelSoftcap as newFreeLevelSoftcap
} from '../../src/mechanics/gqUpgradeLevels'

// ─── Old implementations (verbatim from packages/web_ui/src/singularity.ts) ─

// Same array web_ui used. Sorted ascending; the loop bails at the first unmet
// threshold.
const oldOverclockPerks = [50, 60, 75, 100, 125, 150, 175, 200, 225, 250]

const oldFreeLevelMult = (shopFreeUpgradeMult: number, cubeUpgrade75: number): number =>
  shopFreeUpgradeMult + 0.3 / 100 * cubeUpgrade75

const oldFreeLevelSoftcap = (freeLevel: number, level: number, freeLevelMult: number): number => {
  const baseRealFreeLevels = freeLevelMult * freeLevel
  return Math.min(level, baseRealFreeLevels) + Math.sqrt(Math.max(0, baseRealFreeLevels - level))
}

interface OldMaxLevelInput {
  canExceedCap: boolean
  maxLevel: number
  highestSingularityCount: number
  octeractSingUpgradeCapIncrease: number
}

const oldMaxLevel = (input: OldMaxLevelInput): number => {
  if (!input.canExceedCap) {
    return input.maxLevel
  }
  let cap = input.maxLevel
  for (const perk of oldOverclockPerks) {
    if (input.highestSingularityCount >= perk) {
      cap += 1
    } else {
      break
    }
  }
  cap += input.octeractSingUpgradeCapIncrease
  return cap
}

interface OldActualLevelsInput {
  level: number
  freeLevel: number
  qualityOfLife: boolean
  isPlatonicDelta: boolean
  inNoSingularityUpgrades: boolean
  inSadisticPrequel: boolean
  inLimitedAscensions: boolean
  inLimitedTime: boolean
  freeLevelMult: number
  improvedFreeUnlocked: boolean
  improvedFreeExponent: number
}

const oldActualLevels = (input: OldActualLevelsInput): number => {
  if ((input.inNoSingularityUpgrades || input.inSadisticPrequel) && !input.qualityOfLife) {
    return 0
  }
  if (
    (input.inLimitedAscensions || input.inLimitedTime || input.inSadisticPrequel)
    && input.isPlatonicDelta
  ) {
    return 0
  }
  const actualFreeLevels = oldFreeLevelSoftcap(input.freeLevel, input.level, input.freeLevelMult)
  const linearLevels = input.level + actualFreeLevels
  let polynomialLevels = 0
  if (input.improvedFreeUnlocked) {
    polynomialLevels = Math.pow(input.level * actualFreeLevels, input.improvedFreeExponent)
  }
  return Math.max(linearLevels, polynomialLevels)
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

// ─── gqFreeLevelMultiplier ─────────────────────────────────────────────────

describe('parity: gqFreeLevelMultiplier', () => {
  const shopGrid = [0, 0.5, 1, 1.5, 2]
  const cubeGrid = [0, 1, 10, 100, 1000]
  for (const shop of shopGrid) {
    it.each(cubeGrid)(`shop=${shop} cube75=%i`, (cube) => {
      expect(closeEnough(newFreeLevelMult(shop, cube), oldFreeLevelMult(shop, cube))).toBe(true)
    })
  }
})

// ─── gqUpgradeFreeLevelSoftcap (linear-to-sqrt boundary) ───────────────────

describe('parity: gqUpgradeFreeLevelSoftcap (every regime)', () => {
  // Pick freeLevel × mult combinations and sweep level around the kink at
  // baseRealFreeLevels = freeLevel * mult — sub, equal, super.
  const cases = [
    { freeLevel: 0, mult: 1 },
    { freeLevel: 10, mult: 1 },
    { freeLevel: 100, mult: 1.5 },
    { freeLevel: 50, mult: 2 },
    { freeLevel: 1000, mult: 1 }
  ]
  for (const { freeLevel, mult } of cases) {
    const base = freeLevel * mult
    const levelSweep = [
      0,
      Math.floor(base / 2),
      Math.max(0, Math.floor(base) - 1),
      Math.floor(base),
      Math.ceil(base),
      Math.floor(base) + 1,
      Math.floor(base * 2),
      Math.floor(base * 10)
    ]
    for (const level of levelSweep) {
      it(`freeLevel=${freeLevel} mult=${mult} level=${level}`, () => {
        expect(closeEnough(newFreeLevelSoftcap(freeLevel, level, mult), oldFreeLevelSoftcap(freeLevel, level, mult)))
          .toBe(true)
      })
    }
  }
})

// ─── computeGQUpgradeMaxLevel (each overclock-perks threshold) ─────────────

describe('parity: computeGQUpgradeMaxLevel', () => {
  // Sweep every threshold in the overclock-perks array in both directions
  // (perk-1, perk, perk+1) plus a few values past the last perk.
  const singCountGrid = [
    0,
    49,
    50,
    51,
    59,
    60,
    61,
    74,
    75,
    76,
    99,
    100,
    101,
    124,
    125,
    149,
    150,
    174,
    175,
    199,
    200,
    224,
    225,
    249,
    250,
    251,
    500
  ]
  const capIncreaseGrid = [0, 1, 5]
  const baseMaxLevelGrid = [0, 10, 100, -1] // -1 sentinel covered too
  for (const canExceedCap of [true, false]) {
    for (const maxLevel of baseMaxLevelGrid) {
      for (const capIncrease of capIncreaseGrid) {
        it.each(singCountGrid)(
          `canExceed=${canExceedCap} maxLevel=${maxLevel} capInc=${capIncrease} sing=%i`,
          (sing) => {
            const input = {
              canExceedCap,
              maxLevel,
              highestSingularityCount: sing,
              octeractSingUpgradeCapIncrease: capIncrease
            }
            expect(newMaxLevel(input)).toBe(oldMaxLevel(input))
          }
        )
      }
    }
  }
})

// ─── actualGQUpgradeTotalLevels (gates + improvedFree path) ────────────────

describe('parity: actualGQUpgradeTotalLevels (gate combinations, gates active)', () => {
  // Pick a representative non-zero level / freeLevel and sweep the four
  // challenge gates × the two upgrade flags. Verifies all eight gating
  // truth-table cases produce the same gating decisions.
  const level = 25
  const freeLevel = 10
  const freeLevelMult = 1.5
  for (const inNoSingularityUpgrades of [true, false]) {
    for (const inSadisticPrequel of [true, false]) {
      for (const inLimitedAscensions of [true, false]) {
        for (const inLimitedTime of [true, false]) {
          for (const qualityOfLife of [true, false]) {
            for (const isPlatonicDelta of [true, false]) {
              const input = {
                level,
                freeLevel,
                qualityOfLife,
                isPlatonicDelta,
                inNoSingularityUpgrades,
                inSadisticPrequel,
                inLimitedAscensions,
                inLimitedTime,
                freeLevelMult,
                improvedFreeUnlocked: false,
                improvedFreeExponent: 0
              }
              it(`gates: noSU=${inNoSingularityUpgrades} sadistic=${inSadisticPrequel} limAsc=${inLimitedAscensions} limTime=${inLimitedTime} qol=${qualityOfLife} plat=${isPlatonicDelta}`, () => {
                expect(closeEnough(newActualLevels(input), oldActualLevels(input))).toBe(true)
              })
            }
          }
        }
      }
    }
  }
})

describe('parity: actualGQUpgradeTotalLevels (improvedFree polynomial path)', () => {
  // No gates active. Sweep across exponent values and softcap regimes to
  // make sure the max(linear, polynomial) decision matches.
  const exponentGrid = [0.5, 0.8, 1.0, 1.05, 1.1, 1.2]
  const cases = [
    { level: 0, freeLevel: 0, mult: 1 },
    { level: 5, freeLevel: 10, mult: 1 },
    { level: 10, freeLevel: 10, mult: 1 }, // exactly at kink
    { level: 50, freeLevel: 10, mult: 1 },
    { level: 100, freeLevel: 100, mult: 1.5 },
    { level: 1000, freeLevel: 500, mult: 2 }
  ]
  for (const { level, freeLevel, mult } of cases) {
    for (const exponent of exponentGrid) {
      for (const improvedFreeUnlocked of [true, false]) {
        const input = {
          level,
          freeLevel,
          qualityOfLife: false,
          isPlatonicDelta: false,
          inNoSingularityUpgrades: false,
          inSadisticPrequel: false,
          inLimitedAscensions: false,
          inLimitedTime: false,
          freeLevelMult: mult,
          improvedFreeUnlocked,
          improvedFreeExponent: exponent
        }
        it(`lv=${level} free=${freeLevel} mult=${mult} exp=${exponent} improvedFree=${improvedFreeUnlocked}`, () => {
          expect(closeEnough(newActualLevels(input), oldActualLevels(input))).toBe(true)
        })
      }
    }
  }
})
