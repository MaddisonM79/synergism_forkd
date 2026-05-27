// Parity tests for updateAllMultiplier. Old body transcribed verbatim from
// packages/web_ui/src/Synergism.ts (updateAllMultiplier, ~line 2435 pre-migration).

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import { CalcECC } from '../../src/mechanics/challenges'
import {
  updateAllMultiplier as newUpdateAllMultiplier,
  type UpdateAllMultiplierInput,
  type UpdateAllMultiplierResult
} from '../../src/mechanics/updateAllMultiplier'

const oldUpdateAllMultiplier = (
  input: UpdateAllMultiplierInput
): UpdateAllMultiplierResult => {
  let a = 0

  if (input.upgrade7 > 0) {
    a += Math.min(4, 1 + Math.floor(Decimal.log(input.fifthOwnedCoin + 1, 10)))
  }
  if (input.upgrade9 > 0) {
    a += Math.floor(input.acceleratorBought / 10)
  }
  if (input.upgrade21 > 0) a += 1
  if (input.upgrade22 > 0) a += 1
  if (input.upgrade23 > 0) a += 1
  if (input.upgrade24 > 0) a += 1
  if (input.upgrade25 > 0) a += 1
  if (input.upgrade33 > 0) a += input.totalAcceleratorBoost
  if (input.upgrade49 > 0) {
    a += Math.min(50, Math.floor(Decimal.log(input.transcendPoints.add(1), 1e10)))
  }
  if (input.upgrade68 > 0) {
    a += Math.min(2500, Math.floor((Decimal.log(input.taxdivisor, 10) * 1) / 1000))
  }
  if (input.c1Completions > 0) a += 1

  a += input.multipliersAchievement
  a += 20 * input.research94 * Math.floor(input.sumOfRuneLevels / 8)

  const freeUpgradeMultiplier = Math.min(1e100, a)

  const ecc14a = CalcECC('ascension', input.c14Completions)
  const ecc1tr = CalcECC('transcend', input.c1Completions)
  const ecc7r = CalcECC('reincarnation', input.c7Completions)

  a *= Math.pow(
    1.01,
    input.upgrade21 + input.upgrade22 + input.upgrade23 + input.upgrade24 + input.upgrade25
  )
  a *= 1 + 0.03 * input.upgrade34 + 0.02 * input.upgrade35
  a *= 1 + (1 / 5) * input.research2 * (1 + (1 / 2) * ecc14a)
  a *= 1
    + (1 / 20) * input.research11
    + (1 / 25) * input.research12
    + (1 / 40) * input.research13
    + (3 / 200) * input.research14
    + (1 / 200) * input.research15
  a *= input.multiplicativeMultipliersRune
  a *= 1 + (1 / 20) * input.research87
  a *= 1 + (1 / 100) * input.research128
  a *= 1 + (0.8 / 100) * input.research143
  a *= 1 + (0.6 / 100) * input.research158
  a *= 1 + (0.4 / 100) * input.research173
  a *= 1 + (0.2 / 100) * input.research188
  a *= 1 + (0.01 / 100) * input.research200
  a *= 1 + (0.01 / 100) * input.cubeUpgrade50
  a *= input.antMultiplierMult
  a *= input.multiplierCubeBlessing

  if (
    (input.transcensionChallenge !== 0 || input.reincarnationChallenge !== 0)
    && input.upgrade50 > 0.5
  ) {
    a *= 1.25
  }
  a = Math.pow(
    a,
    Math.min(1, (1 + input.platonicUpgrade6 / 30) * input.viscosityPower)
  )
  a += input.hepteractMultiplier
  a *= input.challenge15RewardMultiplier
  a *= input.hepteractMultiplierMult
  a = Math.floor(Math.min(1e100, a))

  if (input.viscosityCorruptionLevel >= 15) a = Math.pow(a, 0.2)
  if (input.viscosityCorruptionLevel >= 16) a = 1

  const freeMultiplier = a
  const totalMultiplier = freeMultiplier + input.multiplierBought
  const challengeOneLog = 3

  let b = 0
  b += Decimal.log(input.transcendShards.add(1), 3)
  b += input.multiplierBoostsRune
  b += 2 * ecc1tr
  b *= 1 + (11 * input.research33) / 100
  b *= 1 + (11 * input.research34) / 100
  b *= 1 + (11 * input.research35) / 100
  b *= 1 + input.research89 / 5
  b *= input.multiplierBoostsRuneBlessing

  const totalMultiplierBoost = Math.pow(Math.floor(b), 1 + ecc7r * 0.04)

  const c7 = input.c7Completions > 0.5 ? 1.25 : 1

  let multiplierPower = 2 + 0.02 * totalMultiplierBoost * c7

  if (
    input.reincarnationChallenge !== 7
    && input.reincarnationChallenge !== 10
  ) {
    if (input.transcensionChallenge === 1) multiplierPower = 1
    if (input.transcensionChallenge === 2) multiplierPower = 1.25 + 0.0012 * b * c7
  }
  multiplierPower = Math.min(1e300, multiplierPower)

  if (input.reincarnationChallenge === 7) multiplierPower = 1
  if (input.reincarnationChallenge === 10) multiplierPower = 1

  const multiplierEffect = Decimal.pow(multiplierPower, totalMultiplier)

  return {
    freeUpgradeMultiplier,
    freeMultiplier,
    totalMultiplier,
    challengeOneLog,
    totalMultiplierBoost,
    multiplierPower,
    multiplierEffect
  }
}

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

const baseInput: UpdateAllMultiplierInput = {
  upgrade7: 0,
  upgrade9: 0,
  upgrade21: 0,
  upgrade22: 0,
  upgrade23: 0,
  upgrade24: 0,
  upgrade25: 0,
  upgrade33: 0,
  upgrade34: 0,
  upgrade35: 0,
  upgrade49: 0,
  upgrade50: 0,
  upgrade68: 0,
  acceleratorBought: 0,
  multiplierBought: 0,
  fifthOwnedCoin: 0,
  c1Completions: 0,
  c7Completions: 0,
  c14Completions: 0,
  transcendPoints: new Decimal(0),
  transcendShards: new Decimal(0),
  research2: 0,
  research11: 0,
  research12: 0,
  research13: 0,
  research14: 0,
  research15: 0,
  research33: 0,
  research34: 0,
  research35: 0,
  research87: 0,
  research89: 0,
  research94: 0,
  research128: 0,
  research143: 0,
  research158: 0,
  research173: 0,
  research188: 0,
  research200: 0,
  cubeUpgrade50: 0,
  platonicUpgrade6: 0,
  transcensionChallenge: 0,
  reincarnationChallenge: 0,
  viscosityCorruptionLevel: 0,

  multipliersAchievement: 0,
  sumOfRuneLevels: 0,
  multiplicativeMultipliersRune: 1,
  multiplierBoostsRune: 0,
  multiplierBoostsRuneBlessing: 1,
  antMultiplierMult: 1,
  multiplierCubeBlessing: 1,
  hepteractMultiplier: 0,
  hepteractMultiplierMult: 1,

  totalAcceleratorBoost: 0,
  taxdivisor: new Decimal(1),
  viscosityPower: 1,
  challenge15RewardMultiplier: 1
}

interface Case {
  name: string
  input: UpdateAllMultiplierInput
}

const cases: Case[] = [
  { name: 'baseline (all zero)', input: baseInput },

  // ─── Upgrade additions ────────────────────────────────────────────────
  {
    name: 'upgrade 7 adds min(4, 1 + log10(fifthOwnedCoin+1))',
    input: { ...baseInput, upgrade7: 1, fifthOwnedCoin: 100 }
  },
  {
    name: 'upgrade 9 adds floor(acceleratorBought / 10)',
    input: { ...baseInput, upgrade9: 1, acceleratorBought: 35 }
  },
  {
    name: 'upgrades 21-25 each add +1 and feed 1.01^count',
    input: { ...baseInput, upgrade21: 1, upgrade22: 1, upgrade23: 1, upgrade24: 1, upgrade25: 1 }
  },
  {
    name: 'upgrade 33 adds totalAcceleratorBoost',
    input: { ...baseInput, upgrade33: 1, totalAcceleratorBoost: 50 }
  },
  {
    name: 'upgrade 49 adds min(50, log1e10(transcendPoints+1))',
    input: { ...baseInput, upgrade49: 1, transcendPoints: new Decimal('1e500') }
  },
  {
    name: 'upgrade 68 adds min(2500, log10(taxdivisor)/1000)',
    input: { ...baseInput, upgrade68: 1, taxdivisor: new Decimal('1e1000') }
  },
  {
    name: 'c1 completions add +1',
    input: { ...baseInput, c1Completions: 5, multipliersAchievement: 10 }
  },

  // ─── Research / cube / rune contributions ─────────────────────────────
  {
    name: 'research 94 with sumOfRuneLevels adds 20*94*floor(n/8)',
    input: { ...baseInput, research94: 5, sumOfRuneLevels: 100 }
  },
  {
    name: 'upgrade 34/35 add 0.03n + 0.02n multiplier',
    input: { ...baseInput, multipliersAchievement: 50, upgrade34: 10, upgrade35: 10 }
  },
  {
    name: 'research 2 with c14 ECC scales',
    input: { ...baseInput, multipliersAchievement: 50, research2: 5, c14Completions: 25 }
  },
  {
    name: 'researches 11-15 stack',
    input: {
      ...baseInput,
      multipliersAchievement: 50,
      research11: 10,
      research12: 10,
      research13: 10,
      research14: 10,
      research15: 10
    }
  },
  {
    name: 'multiplicativeMultipliersRune scales',
    input: { ...baseInput, multipliersAchievement: 50, multiplicativeMultipliersRune: 2.5 }
  },
  {
    name: 'researches 87/128/143/158/173/188/200 stack',
    input: {
      ...baseInput,
      multipliersAchievement: 50,
      research87: 10,
      research128: 100,
      research143: 100,
      research158: 100,
      research173: 100,
      research188: 100,
      research200: 100000
    }
  },
  {
    name: 'cubeUpgrade 50 multiplier',
    input: { ...baseInput, multipliersAchievement: 50, cubeUpgrade50: 50000 }
  },
  {
    name: 'antMultiplierMult and cube blessing scale',
    input: {
      ...baseInput,
      multipliersAchievement: 50,
      antMultiplierMult: 2,
      multiplierCubeBlessing: 3
    }
  },
  {
    name: 'upgrade 50 in a challenge gives 1.25x',
    input: {
      ...baseInput,
      multipliersAchievement: 50,
      upgrade50: 1,
      transcensionChallenge: 4
    }
  },
  {
    name: 'upgrade 50 outside a challenge does nothing',
    input: { ...baseInput, multipliersAchievement: 50, upgrade50: 1 }
  },

  // ─── Viscosity exponent + corruption gates ────────────────────────────
  {
    name: 'platonic 6 + viscosity power exponent',
    input: {
      ...baseInput,
      multipliersAchievement: 50,
      platonicUpgrade6: 10,
      viscosityPower: 0.9
    }
  },
  {
    name: 'viscosity corruption 15 applies ^0.2',
    input: { ...baseInput, multipliersAchievement: 100, viscosityCorruptionLevel: 15 }
  },
  {
    name: 'viscosity corruption 16 clamps to 1',
    input: { ...baseInput, multipliersAchievement: 100, viscosityCorruptionLevel: 16 }
  },

  // ─── Hepteract + challenge15 ──────────────────────────────────────────
  {
    name: 'hepteract multiplier adds, mult scales, challenge15 reward scales',
    input: {
      ...baseInput,
      multipliersAchievement: 50,
      hepteractMultiplier: 25,
      hepteractMultiplierMult: 1.5,
      challenge15RewardMultiplier: 2
    }
  },

  // ─── multiplierBought added to totalMultiplier ────────────────────────
  {
    name: 'multiplierBought added to totalMultiplier',
    input: { ...baseInput, multipliersAchievement: 100, multiplierBought: 50 }
  },

  // ─── b / totalMultiplierBoost / multiplierPower ───────────────────────
  {
    name: 'b accumulates from transcendShards + rune + ecc',
    input: {
      ...baseInput,
      transcendShards: new Decimal('1e20'),
      multiplierBoostsRune: 10,
      c1Completions: 25,
      multipliersAchievement: 50
    }
  },
  {
    name: 'research 33/34/35/89 + rune blessing multiplies b',
    input: {
      ...baseInput,
      transcendShards: new Decimal('1e20'),
      research33: 5,
      research34: 5,
      research35: 5,
      research89: 5,
      multiplierBoostsRuneBlessing: 1.5,
      multipliersAchievement: 50
    }
  },
  {
    name: 'c7 > 0 sets c7 multiplier to 1.25 in multiplierPower',
    input: {
      ...baseInput,
      transcendShards: new Decimal('1e20'),
      c7Completions: 25,
      multipliersAchievement: 50
    }
  },

  // ─── Transcension challenge overrides ─────────────────────────────────
  {
    name: 't-chal 1 forces multiplierPower = 1',
    input: { ...baseInput, transcensionChallenge: 1, multipliersAchievement: 50 }
  },
  {
    name: 't-chal 2 sets multiplierPower = 1.25 + 0.0012*b*c7',
    input: {
      ...baseInput,
      transcensionChallenge: 2,
      transcendShards: new Decimal('1e20'),
      c7Completions: 25,
      multipliersAchievement: 50
    }
  },

  // ─── Reincarnation challenge overrides ────────────────────────────────
  {
    name: 'r-chal 7 forces multiplierPower = 1 even with stack',
    input: {
      ...baseInput,
      reincarnationChallenge: 7,
      transcendShards: new Decimal('1e20'),
      c7Completions: 25,
      multipliersAchievement: 50
    }
  },
  {
    name: 'r-chal 10 forces multiplierPower = 1 even with stack',
    input: {
      ...baseInput,
      reincarnationChallenge: 10,
      transcendShards: new Decimal('1e20'),
      c7Completions: 25,
      multipliersAchievement: 50
    }
  },

  // ─── Combined scenarios ───────────────────────────────────────────────
  {
    name: 'big stack: upgrades + researches + hepteract + boosts + viscosity',
    input: {
      ...baseInput,
      multiplierBought: 100,
      upgrade7: 1,
      fifthOwnedCoin: 1e8,
      upgrade9: 1,
      acceleratorBought: 200,
      upgrade21: 1,
      upgrade22: 1,
      upgrade23: 1,
      upgrade24: 1,
      upgrade25: 1,
      upgrade33: 1,
      upgrade34: 5,
      upgrade35: 5,
      upgrade49: 1,
      transcendPoints: new Decimal('1e500'),
      upgrade50: 1,
      upgrade68: 1,
      research2: 5,
      research11: 10,
      research12: 10,
      research13: 10,
      research14: 10,
      research15: 10,
      research33: 5,
      research34: 5,
      research35: 5,
      research87: 10,
      research89: 5,
      research94: 5,
      research128: 100,
      research143: 100,
      research158: 100,
      research173: 100,
      research188: 100,
      research200: 100000,
      cubeUpgrade50: 50000,
      platonicUpgrade6: 5,
      transcensionChallenge: 3,
      viscosityCorruptionLevel: 12,
      sumOfRuneLevels: 200,
      multipliersAchievement: 50,
      multiplicativeMultipliersRune: 1.5,
      multiplierBoostsRune: 10,
      multiplierBoostsRuneBlessing: 1.3,
      antMultiplierMult: 2,
      multiplierCubeBlessing: 1.5,
      hepteractMultiplier: 50,
      hepteractMultiplierMult: 1.4,
      totalAcceleratorBoost: 100,
      taxdivisor: new Decimal('1e500'),
      viscosityPower: 0.85,
      challenge15RewardMultiplier: 1.6,
      transcendShards: new Decimal('1e100'),
      c1Completions: 25,
      c7Completions: 25,
      c14Completions: 25
    }
  }
]

describe('parity: updateAllMultiplier', () => {
  for (const c of cases) {
    it(c.name, () => {
      const newRes = newUpdateAllMultiplier(c.input)
      const oldRes = oldUpdateAllMultiplier(c.input)
      expect(newRes.freeUpgradeMultiplier).toBe(oldRes.freeUpgradeMultiplier)
      expect(newRes.freeMultiplier).toBe(oldRes.freeMultiplier)
      expect(newRes.totalMultiplier).toBe(oldRes.totalMultiplier)
      expect(newRes.challengeOneLog).toBe(oldRes.challengeOneLog)
      expect(newRes.totalMultiplierBoost).toBe(oldRes.totalMultiplierBoost)
      expect(newRes.multiplierPower).toBe(oldRes.multiplierPower)
      expect(decimalEq(newRes.multiplierEffect, oldRes.multiplierEffect)).toBe(true)
    })
  }
})
