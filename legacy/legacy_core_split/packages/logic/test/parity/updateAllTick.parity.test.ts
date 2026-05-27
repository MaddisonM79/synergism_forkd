// Parity tests for updateAllTick. Old body transcribed verbatim from
// packages/web_ui/src/Synergism.ts (updateAllTick, ~line 2372 pre-migration).
// Same input/output shape as the new function so the comparison is direct.

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import { CalcECC } from '../../src/mechanics/challenges'
import {
  updateAllTick as newUpdateAllTick,
  type UpdateAllTickInput,
  type UpdateAllTickResult
} from '../../src/mechanics/updateAllTick'

const oldUpdateAllTick = (
  input: UpdateAllTickInput,
  totalMultiplier: number
): UpdateAllTickResult => {
  let a = 0

  const totalAcceleratorInit = input.acceleratorBought
  const costDivisor = 1

  if (input.upgrade8 !== 0) {
    a += Math.floor(input.multiplierBought / 7)
  }
  if (input.upgrade21 !== 0) a += 5
  if (input.upgrade22 !== 0) a += 4
  if (input.upgrade23 !== 0) a += 3
  if (input.upgrade24 !== 0) a += 2
  if (input.upgrade25 !== 0) a += 1
  if (input.upgrade32 !== 0) {
    a += Math.min(500, Math.floor(Decimal.log(input.prestigePoints.add(1), 1e25)))
  }
  if (input.upgrade45 !== 0) {
    a += Math.min(2500, Math.floor(Decimal.log(input.transcendShards.add(1), 10)))
  }
  a += input.acceleratorsAchievement

  const ecc2tr = CalcECC('transcend', input.c2Completions)
  const ecc7r = CalcECC('reincarnation', input.c7Completions)

  a += 5 * ecc2tr
  const freeUpgradeAccelerator = a

  a += input.totalAcceleratorBoost
    * (5
      + 2 * input.research18
      + 2 * input.research19
      + 3 * input.research20
      + input.acceleratorCubeBlessing)

  if (input.prestigeUnlocked) {
    a *= input.multiplicativeAcceleratorsRune
  }

  a *= input.acceleratorMultiplier
  a = Math.pow(
    a,
    Math.min(1, (1 + input.platonicUpgrade6 / 30) * input.viscosityPower)
  )
  a += input.hepteractAccelerators
  a *= input.challenge15RewardAccelerator
  a *= input.hepteractAcceleratorMult
  a = Math.floor(Math.min(1e100, a))

  if (input.viscosityCorruptionLevel >= 15) a = Math.pow(a, 0.2)
  if (input.viscosityCorruptionLevel >= 16) a = 1

  const freeAccelerator = a
  const totalAccelerator = totalAcceleratorInit + freeAccelerator

  const tuSevenMulti = input.upgrade46 > 0.5 ? 1.05 : 1

  let acceleratorPower = Math.pow(
    1.1
      + input.acceleratorPowerRune
      + 1 / 400 * ecc2tr
      + input.acceleratorPowerAchievement
      + tuSevenMulti * (input.totalAcceleratorBoost / 100) * (1 + ecc2tr / 20),
    1 + 0.04 * ecc7r
  )

  if (
    input.reincarnationChallenge !== 7
    && input.reincarnationChallenge !== 10
  ) {
    if (input.transcensionChallenge === 1) {
      acceleratorPower *= 25 / (50 + input.c1Completions)
      acceleratorPower += 0.55
      acceleratorPower = Math.max(1, acceleratorPower)
    }
    if (input.transcensionChallenge === 2) acceleratorPower = 1
    if (input.transcensionChallenge === 3) acceleratorPower = 1 + acceleratorPower / 2
  }
  acceleratorPower = Math.min(1e300, acceleratorPower)
  if (input.reincarnationChallenge === 7) acceleratorPower = 1
  if (input.reincarnationChallenge === 10) acceleratorPower = 1

  let acceleratorEffect: Decimal
  if (input.transcensionChallenge !== 1) {
    acceleratorEffect = Decimal.pow(acceleratorPower, totalAccelerator)
  } else {
    acceleratorEffect = Decimal.pow(acceleratorPower, totalAccelerator + totalMultiplier)
  }
  const acceleratorEffectDisplay = new Decimal(acceleratorPower * 100 - 100)
  if (input.reincarnationChallenge === 10) {
    acceleratorEffect = new Decimal(1)
  }

  let generatorPower = new Decimal(1)
  if (input.upgrade11 > 0.5 && input.reincarnationChallenge !== 7) {
    generatorPower = Decimal.pow(1.02, totalAccelerator)
  }

  return {
    totalAccelerator,
    costDivisor,
    freeUpgradeAccelerator,
    freeAccelerator,
    tuSevenMulti,
    acceleratorPower,
    acceleratorEffect,
    acceleratorEffectDisplay,
    generatorPower
  }
}

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

const baseInput: UpdateAllTickInput = {
  acceleratorBought: 0,
  multiplierBought: 0,
  upgrade8: 0,
  upgrade11: 0,
  upgrade21: 0,
  upgrade22: 0,
  upgrade23: 0,
  upgrade24: 0,
  upgrade25: 0,
  upgrade32: 0,
  upgrade45: 0,
  upgrade46: 0,
  prestigePoints: new Decimal(0),
  transcendShards: new Decimal(0),
  c1Completions: 0,
  c2Completions: 0,
  c7Completions: 0,
  transcensionChallenge: 0,
  reincarnationChallenge: 0,
  research18: 0,
  research19: 0,
  research20: 0,
  platonicUpgrade6: 0,
  prestigeUnlocked: false,
  viscosityCorruptionLevel: 0,

  acceleratorsAchievement: 0,
  acceleratorPowerAchievement: 0,
  multiplicativeAcceleratorsRune: 1,
  acceleratorPowerRune: 0,
  acceleratorCubeBlessing: 0,
  hepteractAccelerators: 0,
  hepteractAcceleratorMult: 1,

  totalAcceleratorBoost: 0,
  acceleratorMultiplier: 1,
  viscosityPower: 1,
  challenge15RewardAccelerator: 1
}

interface Case {
  name: string
  input: UpdateAllTickInput
  totalMultiplier: number
}

const cases: Case[] = [
  { name: 'baseline (all zero)', input: baseInput, totalMultiplier: 0 },

  // ─── Upgrade additions ────────────────────────────────────────────────
  {
    name: 'upgrade 8 adds floor(multiplierBought / 7)',
    input: { ...baseInput, upgrade8: 1, multiplierBought: 100 },
    totalMultiplier: 0
  },
  {
    name: 'upgrades 21-25 add 5/4/3/2/1',
    input: { ...baseInput, upgrade21: 1, upgrade22: 1, upgrade23: 1, upgrade24: 1, upgrade25: 1 },
    totalMultiplier: 0
  },
  {
    name: 'upgrade 32 adds min(500, log1e25(prestigePoints+1))',
    input: { ...baseInput, upgrade32: 1, prestigePoints: new Decimal('1e100') },
    totalMultiplier: 0
  },
  {
    name: 'upgrade 32 saturates at 500 for huge prestigePoints',
    input: { ...baseInput, upgrade32: 1, prestigePoints: new Decimal('1e20000') },
    totalMultiplier: 0
  },
  {
    name: 'upgrade 45 adds min(2500, log10(transcendShards+1))',
    input: { ...baseInput, upgrade45: 1, transcendShards: new Decimal('1e1000') },
    totalMultiplier: 0
  },
  {
    name: 'upgrade 45 saturates at 2500 for huge transcendShards',
    input: { ...baseInput, upgrade45: 1, transcendShards: new Decimal('1e3000') },
    totalMultiplier: 0
  },
  {
    name: 'upgrade 46 sets tuSevenMulti to 1.05',
    input: { ...baseInput, upgrade46: 1 },
    totalMultiplier: 0
  },
  {
    name: 'upgrade 11 enables generatorPower formula',
    input: { ...baseInput, upgrade11: 1, acceleratorBought: 50 },
    totalMultiplier: 0
  },

  // ─── ECC contributions ────────────────────────────────────────────────
  {
    name: 'c2 completions raise freeUpgradeAccelerator via ECC + acceleratorPower',
    input: { ...baseInput, c2Completions: 30, totalAcceleratorBoost: 100 },
    totalMultiplier: 0
  },
  {
    name: 'c7 completions raise acceleratorPower exponent',
    input: { ...baseInput, c7Completions: 25, totalAcceleratorBoost: 50 },
    totalMultiplier: 0
  },

  // ─── Boost / multiplier path ──────────────────────────────────────────
  {
    name: 'totalAcceleratorBoost + research stack',
    input: {
      ...baseInput,
      totalAcceleratorBoost: 100,
      research18: 10,
      research19: 10,
      research20: 10,
      acceleratorCubeBlessing: 5
    },
    totalMultiplier: 0
  },
  {
    name: 'prestige unlocked applies multiplicative rune',
    input: { ...baseInput, totalAcceleratorBoost: 100, prestigeUnlocked: true, multiplicativeAcceleratorsRune: 1.5 },
    totalMultiplier: 0
  },
  {
    name: 'acceleratorMultiplier scales a',
    input: { ...baseInput, totalAcceleratorBoost: 100, acceleratorMultiplier: 2.5 },
    totalMultiplier: 0
  },
  {
    name: 'platonic 6 + viscosity power raise exponent',
    input: {
      ...baseInput,
      totalAcceleratorBoost: 100,
      platonicUpgrade6: 10,
      viscosityPower: 0.9
    },
    totalMultiplier: 0
  },

  // ─── Hepteract + challenge15 ──────────────────────────────────────────
  {
    name: 'hepteract accelerators add + multiplier scales',
    input: {
      ...baseInput,
      totalAcceleratorBoost: 100,
      hepteractAccelerators: 50,
      hepteractAcceleratorMult: 2,
      challenge15RewardAccelerator: 1.5
    },
    totalMultiplier: 0
  },

  // ─── Corruption viscosity gates ───────────────────────────────────────
  {
    name: 'viscosity 15 applies ^0.2 to a',
    input: { ...baseInput, totalAcceleratorBoost: 100, viscosityCorruptionLevel: 15 },
    totalMultiplier: 0
  },
  {
    name: 'viscosity 16 clamps a to 1',
    input: { ...baseInput, totalAcceleratorBoost: 100, viscosityCorruptionLevel: 16 },
    totalMultiplier: 0
  },

  // ─── Transcension challenge overrides ─────────────────────────────────
  {
    name: 't-chal 1 overrides acceleratorPower (25/(50+c1))',
    input: { ...baseInput, transcensionChallenge: 1, c1Completions: 25, totalAcceleratorBoost: 50 },
    totalMultiplier: 100
  },
  {
    name: 't-chal 2 forces acceleratorPower = 1',
    input: { ...baseInput, transcensionChallenge: 2, totalAcceleratorBoost: 100 },
    totalMultiplier: 0
  },
  {
    name: 't-chal 3 halves acceleratorPower then adds 1',
    input: { ...baseInput, transcensionChallenge: 3, totalAcceleratorBoost: 100 },
    totalMultiplier: 0
  },

  // ─── Reincarnation challenge overrides ────────────────────────────────
  {
    name: 'r-chal 7 forces acceleratorPower = 1 and disables generatorPower',
    input: { ...baseInput, reincarnationChallenge: 7, upgrade11: 1, totalAcceleratorBoost: 50, acceleratorBought: 50 },
    totalMultiplier: 0
  },
  {
    name: 'r-chal 10 forces acceleratorPower = 1 and acceleratorEffect = 1',
    input: { ...baseInput, reincarnationChallenge: 10, totalAcceleratorBoost: 50, acceleratorBought: 50 },
    totalMultiplier: 0
  },

  // ─── Combined scenarios ───────────────────────────────────────────────
  {
    name: 'big stack: upgrades + boost + cube + rune + viscosity',
    input: {
      ...baseInput,
      acceleratorBought: 100,
      upgrade8: 1,
      multiplierBought: 35,
      upgrade21: 1,
      upgrade32: 1,
      prestigePoints: new Decimal('1e150'),
      upgrade45: 1,
      transcendShards: new Decimal('1e500'),
      upgrade46: 1,
      upgrade11: 1,
      research18: 5,
      research19: 5,
      research20: 3,
      platonicUpgrade6: 5,
      prestigeUnlocked: true,
      viscosityCorruptionLevel: 12,
      acceleratorsAchievement: 25,
      acceleratorPowerAchievement: 0.1,
      multiplicativeAcceleratorsRune: 1.3,
      acceleratorPowerRune: 0.05,
      acceleratorCubeBlessing: 3,
      hepteractAccelerators: 100,
      hepteractAcceleratorMult: 1.5,
      totalAcceleratorBoost: 200,
      acceleratorMultiplier: 2.2,
      viscosityPower: 0.85,
      challenge15RewardAccelerator: 1.4,
      c1Completions: 25,
      c2Completions: 25,
      c7Completions: 20
    },
    totalMultiplier: 500
  }
]

describe('parity: updateAllTick', () => {
  for (const c of cases) {
    it(c.name, () => {
      const newRes = newUpdateAllTick(c.input, c.totalMultiplier)
      const oldRes = oldUpdateAllTick(c.input, c.totalMultiplier)
      expect(newRes.totalAccelerator).toBe(oldRes.totalAccelerator)
      expect(newRes.costDivisor).toBe(oldRes.costDivisor)
      expect(newRes.freeUpgradeAccelerator).toBe(oldRes.freeUpgradeAccelerator)
      expect(newRes.freeAccelerator).toBe(oldRes.freeAccelerator)
      expect(newRes.tuSevenMulti).toBe(oldRes.tuSevenMulti)
      expect(newRes.acceleratorPower).toBe(oldRes.acceleratorPower)
      expect(decimalEq(newRes.acceleratorEffect, oldRes.acceleratorEffect)).toBe(true)
      expect(decimalEq(newRes.acceleratorEffectDisplay, oldRes.acceleratorEffectDisplay)).toBe(true)
      expect(decimalEq(newRes.generatorPower, oldRes.generatorPower)).toBe(true)
    })
  }
})
