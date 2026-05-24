// Parity tests for resetTimeThreshold + calculateResearchAutomaticObtainium.
// Old bodies transcribed verbatim from packages/web_ui/src/Calculate.ts.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import {
  calculateResearchAutomaticObtainium as newResearchAuto,
  resetTimeThreshold as newResetTimeThreshold
} from '../../src/mechanics/resetTimeAndAutoObtainium'

const oldResetTimeThreshold = (campaignTimeThresholdReduction: number): number =>
  10 - campaignTimeThresholdReduction

interface OldResearchAutoInput {
  deltaTime: number
  ascensionChallenge: number
  research61: number
  research62: number
  cubeUpgrade3: number
  cubeUpgrade47: number
  resourceMult: Decimal
  globalSpeedMult: number
  resetTimeDivisor: number
  reincarnationcounter: number
  baseObtainium: number
  antSacrificeObtainium: Decimal
  antSacrificeStageMult: number
  antSacrificeTimer: number
}

const oldResearchAuto = (input: OldResearchAutoInput): Decimal => {
  if (input.ascensionChallenge === 14) return Decimal.fromString('0')
  const multiplier = 0.5 * input.research61 + 0.1 * input.research62 + 0.8 * input.cubeUpgrade3
  if (multiplier === 0) return Decimal.fromString('0')

  const timePenaltyMult = Math.min(1, input.reincarnationcounter / input.resetTimeDivisor)
  const nonBaseValue = input.resourceMult.times(input.globalSpeedMult).times(timePenaltyMult)

  let nonBaseAntValue = Decimal.fromString('0')
  if (input.cubeUpgrade47 > 0) {
    const antTimePenaltyMult = Math.min(1, input.antSacrificeTimer / input.resetTimeDivisor)
    nonBaseAntValue = input.antSacrificeObtainium.times(input.globalSpeedMult).times(antTimePenaltyMult)
  }

  return Decimal.max(input.baseObtainium, Decimal.max(nonBaseValue, nonBaseAntValue))
    .times(input.deltaTime)
    .div(input.resetTimeDivisor)
    .times(multiplier)
}

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

// ─── resetTimeThreshold ──────────────────────────────────────────────────

describe('parity: resetTimeThreshold', () => {
  const cases = [0, 1, 5, 9, 9.5]
  for (const reduction of cases) {
    it(`reduction=${reduction}`, () => {
      expect(newResetTimeThreshold({ campaignTimeThresholdReduction: reduction }))
        .toBe(oldResetTimeThreshold(reduction))
    })
  }
})

// ─── calculateResearchAutomaticObtainium ────────────────────────────────

describe('parity: calculateResearchAutomaticObtainium', () => {
  const baseInput: OldResearchAutoInput = {
    deltaTime: 1,
    ascensionChallenge: 0,
    research61: 0,
    research62: 0,
    cubeUpgrade3: 0,
    cubeUpgrade47: 0,
    resourceMult: new Decimal('1e10'),
    globalSpeedMult: 1,
    resetTimeDivisor: 10,
    reincarnationcounter: 5,
    baseObtainium: 100,
    antSacrificeObtainium: new Decimal('1e6'),
    antSacrificeStageMult: 1,
    antSacrificeTimer: 5
  }

  const cases: OldResearchAutoInput[] = [
    // Challenge 14 — returns 0 immediately
    { ...baseInput, ascensionChallenge: 14, research61: 10, cubeUpgrade3: 1 },
    // multiplier === 0 — returns 0
    baseInput,
    // Only researches enabled (no cube upgrade 47 → no ant branch)
    { ...baseInput, research61: 10, research62: 5, cubeUpgrade3: 1 },
    // baseObtainium > nonBaseValue (use a large but finite number)
    { ...baseInput, research61: 10, baseObtainium: 1e30 },
    // nonBaseValue > baseObtainium (resourceMult dominates)
    { ...baseInput, research61: 10, resourceMult: new Decimal('1e100') },
    // ant branch active (cubeUpgrade47 > 0)
    {
      ...baseInput,
      research61: 10,
      cubeUpgrade47: 1,
      antSacrificeObtainium: new Decimal('1e200'),
      antSacrificeStageMult: 1.5
    },
    // Time penalty < 1 (counter < divisor)
    { ...baseInput, research61: 10, reincarnationcounter: 2, resetTimeDivisor: 10 },
    // Time penalty saturates (counter >> divisor)
    { ...baseInput, research61: 10, reincarnationcounter: 100, resetTimeDivisor: 10 }
  ]

  for (const input of cases) {
    it(JSON.stringify({
      ...input,
      resourceMult: input.resourceMult.toString(),
      antSacrificeObtainium: input.antSacrificeObtainium.toString()
    }), () => {
      const newRes = newResearchAuto(input)
      const oldRes = oldResearchAuto(input)
      expect(decimalEq(newRes, oldRes)).toBe(true)
    })
  }
})
