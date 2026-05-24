// Parity tests for the rune-blessing / rune-spirit EXP/level cluster and the
// `maxRuneUpgradePurchase` planner. Old bodies transcribed verbatim from
// packages/web_ui/src/RuneBlessings.ts (lines 192-313) and
// packages/web_ui/src/RuneSpirits.ts (lines 197-308). Sweeps cover the four
// in-game cost shapes (blessing speed/SI × spirit speed/SI), bracketed
// across negative-budget, locked-by-budget, exact-next-level, multi-level,
// and big-EXP+budget scenarios. Both `upperLimit` values and both
// `minOfferingsFloor` strategies are exercised.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  maxRuneUpgradePurchase as newMaxPurchase,
  runeUpgradeEXPLeftToLevel as newEXPLeft,
  runeUpgradeEXPToLevel as newEXPToLevel,
  runeUpgradeLevelFromEXP as newLevelFromEXP
} from '../../src/mechanics/runeUpgradeProgression'

// ─── Old implementations ──────────────────────────────────────────────────

const oldEXPToLevel = (costCoefficient: Decimal, level: number, levelsPerOOM: number): Decimal =>
  costCoefficient.times(Decimal.pow(10, level / levelsPerOOM).minus(1))

const oldEXPLeftToLevel = (
  costCoefficient: Decimal,
  targetLevel: number,
  levelsPerOOM: number,
  currentRuneEXP: Decimal
): Decimal => Decimal.max(0, oldEXPToLevel(costCoefficient, targetLevel, levelsPerOOM).minus(currentRuneEXP))

// Old "level from EXP, with float-bump check". Returns the level that
// web_ui's updateLevelsFromEXP would set.
const oldLevelWithBump = (
  currentRuneEXP: Decimal,
  costCoefficient: Decimal,
  levelsPerOOM: number
): number => {
  const levels = Math.floor(levelsPerOOM * Decimal.log10(currentRuneEXP.div(costCoefficient).plus(1)))
  if (oldEXPLeftToLevel(costCoefficient, levels + 1, levelsPerOOM, currentRuneEXP).eq(0)) {
    return levels + 1
  }
  return levels
}

interface OldMaxPurchaseInput {
  costCoefficient: Decimal
  levelsPerOOM: number
  currentLevel: number
  currentRuneEXP: Decimal
  runeEXPPerOffering: Decimal
  budget: Decimal
  upperLimit: number
  minOfferingsFloor: Decimal
}

const oldMaxPurchase = (input: OldMaxPurchaseInput) => {
  if (input.budget.lt(0)) {
    return { levels: 0, expRequired: new Decimal(0), offerings: new Decimal(0) }
  }
  const totalEXPAvailable = input.budget.times(input.runeEXPPerOffering).add(input.currentRuneEXP)
  const maxLevel = Math.floor(
    input.levelsPerOOM * Decimal.log10(totalEXPAvailable.div(input.costCoefficient).plus(1))
  )
  const levelsGained = Math.min(input.upperLimit, Math.max(0, maxLevel - input.currentLevel))

  if (levelsGained === 0) {
    const nextLevelEXP = oldEXPToLevel(input.costCoefficient, input.currentLevel + 1, input.levelsPerOOM)
    const offeringsRequired = Decimal.max(
      input.minOfferingsFloor,
      nextLevelEXP.minus(input.currentRuneEXP).div(input.runeEXPPerOffering).ceil()
    )
    return { levels: 1, expRequired: nextLevelEXP, offerings: offeringsRequired }
  }

  const expRequired = oldEXPToLevel(input.costCoefficient, input.currentLevel + levelsGained, input.levelsPerOOM)
  const offeringsRequired = Decimal.max(
    input.minOfferingsFloor,
    expRequired.minus(input.currentRuneEXP).div(input.runeEXPPerOffering).ceil()
  )
  return { levels: levelsGained, expRequired, offerings: offeringsRequired }
}

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

// ─── Shape grid: 2 blessings + 2 spirits from packages/web_ui ──────────────

const shapes = [
  { name: 'speedBlessing', costCoefficient: new Decimal(1e6), levelsPerOOM: 4 },
  { name: 'siBlessing', costCoefficient: new Decimal(1e15), levelsPerOOM: 4 },
  { name: 'speedSpirit', costCoefficient: new Decimal(1e45), levelsPerOOM: 2 },
  { name: 'siSpirit', costCoefficient: new Decimal(1e85), levelsPerOOM: 2 }
]

const levelGrid = [0, 1, 5, 10, 25, 50, 100, 200, 1000]

// ─── runeUpgradeEXPToLevel ────────────────────────────────────────────────

describe('parity: runeUpgradeEXPToLevel', () => {
  for (const shape of shapes) {
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

// ─── runeUpgradeEXPLeftToLevel ────────────────────────────────────────────

describe('parity: runeUpgradeEXPLeftToLevel', () => {
  const currentEXPGrid = [
    new Decimal(0),
    new Decimal('1e6'),
    new Decimal('1e30'),
    new Decimal('1e100')
  ]
  for (const shape of shapes) {
    for (const level of [0, 5, 25, 100, 1000]) {
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

// ─── runeUpgradeLevelFromEXP (with float-bump check) ───────────────────────

describe('parity: runeUpgradeLevelFromEXP', () => {
  const expGrid = [
    new Decimal(0),
    new Decimal(1),
    new Decimal('1e6'),
    new Decimal('1e15'),
    new Decimal('1e45'),
    new Decimal('1e85'),
    new Decimal('1e100')
  ]
  for (const shape of shapes) {
    for (const exp of expGrid) {
      it(`${shape.name} exp=${exp.toString()}`, () => {
        const { levels, needsFloatBump } = newLevelFromEXP(exp, shape.costCoefficient, shape.levelsPerOOM)
        const effectiveLevel = needsFloatBump ? levels + 1 : levels
        expect(effectiveLevel).toBe(oldLevelWithBump(exp, shape.costCoefficient, shape.levelsPerOOM))
      })
    }
  }

  // Boundary cases: EXP set to exactly the EXP-for-level-N value should
  // trigger the float-bump path.
  describe('boundary: exactly-on-level EXP triggers float bump', () => {
    for (const shape of shapes) {
      for (const targetLevel of [1, 5, 25]) {
        it(`${shape.name} EXP = EXPforLevel(${targetLevel})`, () => {
          const exactEXP = oldEXPToLevel(shape.costCoefficient, targetLevel, shape.levelsPerOOM)
          const { levels, needsFloatBump } = newLevelFromEXP(exactEXP, shape.costCoefficient, shape.levelsPerOOM)
          const effectiveLevel = needsFloatBump ? levels + 1 : levels
          expect(effectiveLevel).toBe(oldLevelWithBump(exactEXP, shape.costCoefficient, shape.levelsPerOOM))
        })
      }
    }
  })
})

// ─── maxRuneUpgradePurchase ────────────────────────────────────────────────

describe('parity: maxRuneUpgradePurchase', () => {
  // Span: negative budget, can't afford one, exact-next-level, multi-level,
  // upper-limit-clamped, big-EXP+budget.
  const cases = [
    // Negative budget — zero result short-circuit.
    {
      label: 'negative-budget',
      currentLevel: 5,
      currentRuneEXP: new Decimal(0),
      runeEXPPerOffering: new Decimal(10),
      budget: new Decimal(-1)
    },
    // Tiny budget at high current level — can't afford one.
    {
      label: 'cant-afford-one',
      currentLevel: 50,
      currentRuneEXP: new Decimal(0),
      runeEXPPerOffering: new Decimal(1),
      budget: new Decimal(1)
    },
    // A few affordable levels.
    {
      label: 'few-levels',
      currentLevel: 5,
      currentRuneEXP: new Decimal(0),
      runeEXPPerOffering: new Decimal(100),
      budget: new Decimal('1e10')
    },
    // Big budget — would reach many levels if upperLimit didn't cap.
    {
      label: 'big-budget-capped',
      currentLevel: 0,
      currentRuneEXP: new Decimal(0),
      runeEXPPerOffering: new Decimal(10),
      budget: new Decimal('1e120')
    },
    // Already-accumulated EXP plus budget.
    {
      label: 'EXP-plus-budget',
      currentLevel: 10,
      currentRuneEXP: new Decimal('1e30'),
      runeEXPPerOffering: new Decimal(1000),
      budget: new Decimal('1e50')
    },
    // Zero budget at level 0 — falls to the cost-to-next-level branch.
    {
      label: 'zero-budget',
      currentLevel: 0,
      currentRuneEXP: new Decimal(0),
      runeEXPPerOffering: new Decimal(1),
      budget: new Decimal(0)
    }
  ]
  const upperLimits = [1, 10]
  const minOfferingsFloors = [
    { name: 'spirit-floor-1', floor: new Decimal(1) },
    // Blessing-style dynamic floor; uses one of the sample currentEXP values
    // to mirror the per-call recompute. The exact value doesn't matter for
    // parity (both old and new use the same input), but exercising a
    // nontrivial Decimal validates the Decimal.max comparison.
    { name: 'blessing-style-floor', floor: new Decimal('1e3') }
  ]

  for (const shape of shapes) {
    for (const c of cases) {
      for (const upperLimit of upperLimits) {
        for (const floor of minOfferingsFloors) {
          const input = {
            costCoefficient: shape.costCoefficient,
            levelsPerOOM: shape.levelsPerOOM,
            currentLevel: c.currentLevel,
            currentRuneEXP: c.currentRuneEXP,
            runeEXPPerOffering: c.runeEXPPerOffering,
            budget: c.budget,
            upperLimit,
            minOfferingsFloor: floor.floor
          }
          it(`${shape.name} ${c.label} upperLimit=${upperLimit} floor=${floor.name}`, () => {
            const newRes = newMaxPurchase(input)
            const oldRes = oldMaxPurchase(input)
            expect(newRes.levels).toBe(oldRes.levels)
            expect(decimalEq(newRes.expRequired, oldRes.expRequired)).toBe(true)
            expect(decimalEq(newRes.offerings, oldRes.offerings)).toBe(true)
          })
        }
      }
    }
  }
})
