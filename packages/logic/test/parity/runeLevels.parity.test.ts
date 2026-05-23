// Parity tests for the rune EXP/level math cluster, lifted from
// packages/web_ui/src/Runes.ts. Each `oldXxx` transcribes the pre-migration
// body verbatim. Sweeps cover: the EXP↔level closed-form invert at
// representative levels per the in-game rune coefficients, the
// max-affordable-purchase planner across budgets that span sub-1-level,
// exactly-N-levels, and far-overshoot regimes, plus the "level 0 unowned"
// edge case.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  maxRuneLevelPurchase as newMaxPurchase,
  runeEXPLeftToLevel as newEXPLeft,
  runeEXPToLevel as newEXPToLevel,
  runeLevelFromEXP as newLevelFromEXP,
  runeOfferingsToLevel as newOfferingsToLevel
} from '../../src/mechanics/runeLevels'

// ─── Old implementations (verbatim from packages/web_ui/src/Runes.ts) ─────

const oldEXPToLevel = (costCoefficient: Decimal, level: number, levelsPerOOM: number): Decimal =>
  costCoefficient.times(Decimal.pow(10, level / levelsPerOOM).minus(1))

const oldEXPLeftToLevel = (
  costCoefficient: Decimal,
  targetLevel: number,
  levelsPerOOM: number,
  currentRuneEXP: Decimal
): Decimal => Decimal.max(0, oldEXPToLevel(costCoefficient, targetLevel, levelsPerOOM).minus(currentRuneEXP))

const oldOfferingsToLevel = (
  costCoefficient: Decimal,
  targetLevel: number,
  levelsPerOOM: number,
  currentRuneEXP: Decimal,
  runeEXPPerOffering: Decimal
): Decimal =>
  Decimal.max(
    1,
    oldEXPLeftToLevel(costCoefficient, targetLevel, levelsPerOOM, currentRuneEXP)
      .div(runeEXPPerOffering)
      .ceil()
  )

const oldLevelFromEXP = (currentRuneEXP: Decimal, costCoefficient: Decimal, levelsPerOOM: number): number =>
  Math.floor(levelsPerOOM * Decimal.log10(currentRuneEXP.div(costCoefficient).plus(1)))

interface OldMaxPurchaseInput {
  costCoefficient: Decimal
  levelsPerOOM: number
  currentLevel: number
  currentRuneEXP: Decimal
  runeEXPPerOffering: Decimal
  budget: Decimal
  isUnlocked: boolean
}

const oldMaxPurchase = (input: OldMaxPurchaseInput) => {
  if (!input.isUnlocked || input.budget.lt(0)) {
    return { levels: 0, expRequired: new Decimal(0), offerings: new Decimal(0) }
  }
  const totalEXPAvailable = input.budget.times(input.runeEXPPerOffering).add(input.currentRuneEXP)
  const maxLevel = Math.floor(input.levelsPerOOM * Decimal.log10(totalEXPAvailable.div(input.costCoefficient).plus(1)))
  const levelsGained = Math.max(0, maxLevel - input.currentLevel)
  if (levelsGained === 0) {
    const nextLevelEXP = oldEXPToLevel(input.costCoefficient, input.currentLevel + 1, input.levelsPerOOM)
    const offeringsRequired = Decimal.max(
      1,
      nextLevelEXP.minus(input.currentRuneEXP).div(input.runeEXPPerOffering).ceil()
    )
    return { levels: 1, expRequired: nextLevelEXP, offerings: offeringsRequired }
  }
  const expRequired = oldEXPToLevel(input.costCoefficient, input.currentLevel + levelsGained, input.levelsPerOOM)
  const offeringsRequired = Decimal.max(
    1,
    expRequired.minus(input.currentRuneEXP).div(input.runeEXPPerOffering).ceil()
  )
  return { levels: levelsGained, expRequired, offerings: offeringsRequired }
}

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

// In-game rune coefficients (per packages/web_ui/src/Runes.ts) chosen to
// represent the spectrum: speed=50/150, antiquities=1/100, infiniteAscent
// = whatever its costCoefficient is. We use a couple of representative
// (costCoeff, levelsPerOOM) pairs plus a wider sweep.
const runeShapes = [
  { name: 'speed', costCoefficient: new Decimal(50), levelsPerOOM: 150 },
  { name: 'small-coeff', costCoefficient: new Decimal(1), levelsPerOOM: 100 },
  { name: 'huge-coeff', costCoefficient: new Decimal('1e10'), levelsPerOOM: 200 }
]

const levelGrid = [0, 1, 10, 50, 100, 150, 500, 1000, 5000]

// ─── runeEXPToLevel ────────────────────────────────────────────────────────

describe('parity: runeEXPToLevel', () => {
  for (const shape of runeShapes) {
    for (const level of levelGrid) {
      it(`${shape.name} level=${level}`, () => {
        expect(decimalEq(
          newEXPToLevel(shape.costCoefficient, level, shape.levelsPerOOM),
          oldEXPToLevel(shape.costCoefficient, level, shape.levelsPerOOM)
        )).toBe(true)
      })
    }
  }
})

// ─── runeEXPLeftToLevel (clamps at zero when over-leveled) ─────────────────

describe('parity: runeEXPLeftToLevel', () => {
  const currentEXPGrid = [
    new Decimal(0),
    new Decimal(100),
    new Decimal('1e9'),
    new Decimal('1e100')
  ]
  for (const shape of runeShapes) {
    for (const level of levelGrid) {
      for (const currentRuneEXP of currentEXPGrid) {
        it(`${shape.name} level=${level} currentEXP=${currentRuneEXP.toString()}`, () => {
          expect(decimalEq(
            newEXPLeft(shape.costCoefficient, level, shape.levelsPerOOM, currentRuneEXP),
            oldEXPLeftToLevel(shape.costCoefficient, level, shape.levelsPerOOM, currentRuneEXP)
          )).toBe(true)
        })
      }
    }
  }
})

// ─── runeOfferingsToLevel (clamps at 1) ────────────────────────────────────

describe('parity: runeOfferingsToLevel', () => {
  const perOfferingGrid = [new Decimal(1), new Decimal(100), new Decimal('1e6')]
  for (const shape of runeShapes) {
    for (const level of [10, 100, 500]) {
      for (const currentEXP of [new Decimal(0), new Decimal('1e5')]) {
        for (const perOffering of perOfferingGrid) {
          it(`${shape.name} level=${level} EXP=${currentEXP.toString()} perOff=${perOffering.toString()}`, () => {
            expect(decimalEq(
              newOfferingsToLevel(shape.costCoefficient, level, shape.levelsPerOOM, currentEXP, perOffering),
              oldOfferingsToLevel(shape.costCoefficient, level, shape.levelsPerOOM, currentEXP, perOffering)
            )).toBe(true)
          })
        }
      }
    }
  }
})

// ─── runeLevelFromEXP (closed-form invert) ─────────────────────────────────

describe('parity: runeLevelFromEXP', () => {
  // Spread EXP values from sub-1-level (small EXP) to massive (Decimal range).
  const expGrid = [
    new Decimal(0),
    new Decimal(1),
    new Decimal(100),
    new Decimal('1e3'),
    new Decimal('1e6'),
    new Decimal('1e10'),
    new Decimal('1e30'),
    new Decimal('1e100')
  ]
  for (const shape of runeShapes) {
    for (const exp of expGrid) {
      it(`${shape.name} exp=${exp.toString()}`, () => {
        expect(newLevelFromEXP(exp, shape.costCoefficient, shape.levelsPerOOM))
          .toBe(oldLevelFromEXP(exp, shape.costCoefficient, shape.levelsPerOOM))
      })
    }
  }
})

// ─── maxRuneLevelPurchase (gate, sub-level budget, multi-level budget) ─────

describe('parity: maxRuneLevelPurchase', () => {
  const baseInputs = [
    { costCoefficient: new Decimal(50), levelsPerOOM: 150 },
    { costCoefficient: new Decimal(1), levelsPerOOM: 100 }
  ]
  // Span: locked → unlocked-but-broke, broke at high level, exact-next-level,
  // few-levels-budget, massive overshoot.
  const cases = [
    // Locked rune — must return zero result.
    {
      currentLevel: 0,
      currentRuneEXP: new Decimal(0),
      runeEXPPerOffering: new Decimal(10),
      budget: new Decimal('1e20'),
      isUnlocked: false
    },
    // Negative budget — also zero result.
    {
      currentLevel: 5,
      currentRuneEXP: new Decimal(100),
      runeEXPPerOffering: new Decimal(10),
      budget: new Decimal(-1),
      isUnlocked: true
    },
    // Can't afford even one level (small budget at high current level)
    {
      currentLevel: 100,
      currentRuneEXP: new Decimal(0),
      runeEXPPerOffering: new Decimal(1),
      budget: new Decimal(1),
      isUnlocked: true
    },
    // Some affordable levels
    {
      currentLevel: 10,
      currentRuneEXP: new Decimal(0),
      runeEXPPerOffering: new Decimal(100),
      budget: new Decimal('1e6'),
      isUnlocked: true
    },
    // Big budget — should reach many levels
    {
      currentLevel: 0,
      currentRuneEXP: new Decimal(0),
      runeEXPPerOffering: new Decimal(10),
      budget: new Decimal('1e20'),
      isUnlocked: true
    },
    // Already-accumulated EXP plus budget
    {
      currentLevel: 50,
      currentRuneEXP: new Decimal('1e8'),
      runeEXPPerOffering: new Decimal(1000),
      budget: new Decimal('1e10'),
      isUnlocked: true
    }
  ]
  for (const base of baseInputs) {
    for (const c of cases) {
      const input = { ...base, ...c }
      it(
        `coeff=${base.costCoefficient.toString()} oom=${base.levelsPerOOM} curLevel=${c.currentLevel} unlocked=${c.isUnlocked} budget=${c.budget.toString()}`,
        () => {
          const newRes = newMaxPurchase(input)
          const oldRes = oldMaxPurchase(input)
          expect(newRes.levels).toBe(oldRes.levels)
          expect(decimalEq(newRes.expRequired, oldRes.expRequired)).toBe(true)
          expect(decimalEq(newRes.offerings, oldRes.offerings)).toBe(true)
        }
      )
    }
  }
})
