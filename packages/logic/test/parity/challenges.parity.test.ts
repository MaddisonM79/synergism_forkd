// Parity tests for the pure-math helpers migrated from
// packages/web_ui/src/Challenges.ts. The OLD implementations are transcribed
// verbatim below, with the previously-implicit dependencies (G state, shop
// upgrades, player flags) lifted into explicit parameters.

import { describe, expect, it } from 'vitest'
import { Decimal } from '../../src/math/bignum'
import {
  autoAscensionChallengeSweepUnlock as newAutoAscChallengeSweepUnlock,
  CalcECC as newCalcECC,
  calculateChallengeRequirementMultiplier as newCalcReqMult,
  challenge15ScoreMultiplier as newChal15ScoreMult,
  challengeRequirement as newChallengeRequirement,
  challengeScoreDisplay as newScoreDisplay,
  type ChallengeType,
  getMaxChallenges as newGetMax,
  getNextAscensionChallenge as newNextAscChallenge,
  getNextRegularChallenge as newNextRegularChallenge
} from '../../src/mechanics/challenges'

// ─── CalcECC ───────────────────────────────────────────────────────────────

const oldCalcECC = (type: ChallengeType, completions: number): number => {
  let effective = 0
  if (type === 'transcend') {
    effective += Math.min(100, completions)
    effective += 1 / 20 * (Math.min(1000, Math.max(100, completions)) - 100)
    effective += 1 / 100 * (Math.max(1000, completions) - 1000)
    return effective
  }
  if (type === 'reincarnation') {
    effective += Math.min(25, completions)
    effective += 1 / 2 * (Math.min(75, Math.max(25, completions)) - 25)
    effective += 1 / 10 * (Math.max(75, completions) - 75)
    return effective
  }
  // ascension
  effective += Math.min(10, completions)
  effective += 1 / 2 * (Math.max(10, completions) - 10)
  return effective
}

describe('parity: CalcECC', () => {
  const types: ChallengeType[] = ['transcend', 'reincarnation', 'ascension']
  // Sample across each piecewise segment for every type, including
  // boundary values (10/25/75/100/1000) and well past the last knee.
  const completionsGrid = [0, 1, 5, 9, 10, 11, 24, 25, 26, 50, 74, 75, 76, 99, 100, 101, 500, 999, 1000, 1001, 5000, 100000]

  for (const type of types) {
    describe(type, () => {
      it.each(completionsGrid)('completions=%i', (completions) => {
        expect(newCalcECC(type, completions)).toBe(oldCalcECC(type, completions))
      })
    })
  }
})

// ─── OLD calculateChallengeRequirementMultiplier ──────────────────────────

const oldCalcReqMult = (
  type: ChallengeType,
  completions: number,
  special: number,
  hyperchallengeMultiplier: number,
  platonicUpgrade8: number,
  chal15TranscendReduction: number,
  chal15ReincarnationReduction: number,
  c9c10ScalingReduction: number,
  c9c10ScalingReduction2: number
): number => {
  let requirementMultiplier = Math.max(
    1,
    hyperchallengeMultiplier / (1 + platonicUpgrade8 / 2.5)
  )
  if (type === 'ascension') {
    requirementMultiplier = 1
  }
  switch (type) {
    case 'transcend':
      requirementMultiplier *= chal15TranscendReduction
      if (completions >= 75) {
        requirementMultiplier *= Math.pow(1 + completions, 12) / Math.pow(75, 8)
      } else {
        requirementMultiplier *= Math.pow(1 + completions, 2)
      }
      if (completions >= 1000) {
        requirementMultiplier *= 10 * Math.pow(completions / 1000, 3)
      }
      if (completions >= 9000) {
        requirementMultiplier *= 1337
      }
      if (completions >= 9001) {
        requirementMultiplier *= completions - 8999
      }
      return requirementMultiplier
    case 'reincarnation':
      if (completions >= 100 && (special === 9 || special === 10)) {
        requirementMultiplier *= Math.pow(1.05, (completions - 100) * (1 + (completions - 100) / 20))
      }
      if (completions >= 90) {
        if (special === 6) requirementMultiplier *= 100
        else if (special === 7) requirementMultiplier *= 50
        else if (special === 8) requirementMultiplier *= 10
        else requirementMultiplier *= 4
      }
      if (completions >= 80) {
        if (special === 6) requirementMultiplier *= 50
        else if (special === 7) requirementMultiplier *= 20
        else if (special === 8) requirementMultiplier *= 4
        else requirementMultiplier *= 2
      }
      if (completions >= 70) {
        if (special === 6) requirementMultiplier *= 20
        else if (special === 7) requirementMultiplier *= 10
        else if (special === 8) requirementMultiplier *= 2
        else requirementMultiplier *= 1
      }
      if (completions >= 60 && (special === 9 || special === 10)) {
        requirementMultiplier *= Math.pow(
          1000,
          (completions - 60)
            * (1 + c9c10ScalingReduction + c9c10ScalingReduction2)
            / 10
        )
      }
      if (completions >= 25) {
        requirementMultiplier *= Math.pow(1 + completions, 5) / 625
      }
      if (completions < 25) {
        requirementMultiplier *= Math.min(Math.pow(1 + completions, 2), Math.pow(1.3797, completions))
      }
      requirementMultiplier *= chal15ReincarnationReduction
      return requirementMultiplier
    case 'ascension':
      if (special !== 15) {
        if (completions >= 10) {
          requirementMultiplier *= 2 * (1 + completions) - 10
        } else {
          requirementMultiplier *= 1 + completions
        }
      } else {
        requirementMultiplier *= Math.pow(1000, completions)
      }
      return requirementMultiplier
    default:
      throw new Error('unreachable')
  }
}

const closeEnough = (a: number, b: number, rel = 1e-12): boolean => {
  if (a === b) return true
  if (!isFinite(a) || !isFinite(b)) return a === b
  if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < rel
  return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < rel
}

const decimalClose = (a: Decimal | number, b: Decimal | number, rel = 1e-12): boolean => {
  const aD = a instanceof Decimal ? a : new Decimal(a)
  const bD = b instanceof Decimal ? b : new Decimal(b)
  if (aD.eq(bD)) return true
  if (!aD.isFinite() || !bD.isFinite()) return aD.eq(bD)
  const diff = aD.sub(bD).abs()
  const denom = Decimal.max(aD.abs(), bD.abs())
  return diff.div(denom).lt(rel)
}

// ─── calculateChallengeRequirementMultiplier ──────────────────────────────

describe('parity: calculateChallengeRequirementMultiplier — transcend', () => {
  // Hits the 75/1000/9000/9001 thresholds plus the < 75 branch.
  const completionsGrid = [0, 1, 25, 50, 74, 75, 100, 500, 999, 1000, 1001, 5000, 8999, 9000, 9001, 9100]
  const hyperGrid = [1, 1.5, 5, 100]
  const plat8Grid = [0, 1, 5]
  const reductionGrid = [0.5, 1]

  for (const completions of completionsGrid) {
    for (const hyper of hyperGrid) {
      for (const plat8 of plat8Grid) {
        for (const reduction of reductionGrid) {
          it(`c=${completions} h=${hyper} p8=${plat8} red=${reduction}`, () => {
            const oldVal = oldCalcReqMult(
              'transcend',
              completions,
              0,
              hyper,
              plat8,
              reduction,
              1,
              0,
              0
            )
            const newVal = newCalcReqMult({
              type: 'transcend',
              completions,
              special: 0,
              hyperchallengeMultiplier: hyper,
              platonicUpgrade8: plat8,
              challenge15TranscendReduction: reduction,
              challenge15ReincarnationReduction: 1,
              challengeTomeC9C10ScalingReduction: 0,
              challengeTome2C9C10ScalingReduction: 0
            })
            expect(closeEnough(newVal, oldVal)).toBe(true)
          })
        }
      }
    }
  }
})

describe('parity: calculateChallengeRequirementMultiplier — reincarnation', () => {
  // Each special triggers a different multiplier path in the >=70/80/90/100 bands.
  const completionsGrid = [0, 10, 24, 25, 50, 60, 70, 80, 90, 100, 150]
  const specialGrid = [0, 6, 7, 8, 9, 10]
  const c9c10Grid = [0, -0.1, -0.5]

  for (const completions of completionsGrid) {
    for (const special of specialGrid) {
      for (const c9c10 of c9c10Grid) {
        it(`c=${completions} s=${special} c9c10=${c9c10}`, () => {
          const oldVal = oldCalcReqMult(
            'reincarnation',
            completions,
            special,
            1,
            0,
            1,
            1,
            c9c10,
            0
          )
          const newVal = newCalcReqMult({
            type: 'reincarnation',
            completions,
            special,
            hyperchallengeMultiplier: 1,
            platonicUpgrade8: 0,
            challenge15TranscendReduction: 1,
            challenge15ReincarnationReduction: 1,
            challengeTomeC9C10ScalingReduction: c9c10,
            challengeTome2C9C10ScalingReduction: 0
          })
          expect(closeEnough(newVal, oldVal)).toBe(true)
        })
      }
    }
  }
})

describe('parity: calculateChallengeRequirementMultiplier — ascension', () => {
  // Two branches: special=15 (Math.pow(1000, completions)) vs others (linear/quadratic).
  // hyperchallenge is normalized to 1 internally so we don't sweep it.
  const completionsGrid = [0, 1, 9, 10, 11, 50, 100]
  const specialGrid = [0, 11, 12, 13, 14, 15]

  for (const completions of completionsGrid) {
    for (const special of specialGrid) {
      it(`c=${completions} s=${special}`, () => {
        const oldVal = oldCalcReqMult('ascension', completions, special, 1, 0, 1, 1, 0, 0)
        const newVal = newCalcReqMult({
          type: 'ascension',
          completions,
          special,
          hyperchallengeMultiplier: 1,
          platonicUpgrade8: 0,
          challenge15TranscendReduction: 1,
          challenge15ReincarnationReduction: 1,
          challengeTomeC9C10ScalingReduction: 0,
          challengeTome2C9C10ScalingReduction: 0
        })
        expect(closeEnough(newVal, oldVal)).toBe(true)
      })
    }
  }
})

// ─── challengeRequirement (Decimal output for T/R/15, number for 11..14) ─

describe('parity: challengeRequirement', () => {
  const baseRequirements = [10, 20, 60, 100, 200, 125, 500, 7500, 2.0e8, 2.5e9]
  const cases: Array<{ challenge: number; completion: number; special?: number; c10red?: number }> = [
    { challenge: 1, completion: 0 },
    { challenge: 1, completion: 100 },
    { challenge: 5, completion: 750 },
    { challenge: 6, completion: 25, special: 6 },
    { challenge: 9, completion: 70, special: 9 },
    { challenge: 10, completion: 50, special: 10, c10red: 5 },
    { challenge: 11, completion: 0 },
    { challenge: 12, completion: 15 },
    { challenge: 15, completion: 1 },
    { challenge: 16, completion: 0 } // out-of-range → 0
  ]

  for (const c of cases) {
    it(`challenge ${c.challenge} completion ${c.completion}`, () => {
      const special = c.special ?? 0
      const c10red = c.c10red ?? 0
      const newVal = newChallengeRequirement({
        challenge: c.challenge,
        completion: c.completion,
        special,
        challengeBaseRequirement: baseRequirements[c.challenge - 1] ?? 0,
        c10RequirementReduction: c10red,
        hyperchallengeMultiplier: 1,
        platonicUpgrade8: 0,
        challenge15TranscendReduction: 1,
        challenge15ReincarnationReduction: 1,
        challengeTomeC9C10ScalingReduction: 0,
        challengeTome2C9C10ScalingReduction: 0
      })

      const mult = oldCalcReqMult(
        c.challenge <= 5 ? 'transcend' : c.challenge <= 10 ? 'reincarnation' : 'ascension',
        c.completion,
        special,
        1,
        0,
        1,
        1,
        0,
        0
      )
      let oldVal: Decimal | number
      if (c.challenge >= 1 && c.challenge <= 5) {
        oldVal = Decimal.pow(10, (baseRequirements[c.challenge - 1] ?? 0) * mult)
      } else if (c.challenge >= 6 && c.challenge <= 10) {
        oldVal = Decimal.pow(10, ((baseRequirements[c.challenge - 1] ?? 0) - c10red) * mult)
      } else if (c.challenge >= 11 && c.challenge <= 14) {
        oldVal = mult
      } else if (c.challenge === 15) {
        oldVal = Decimal.pow(10, 1 * Math.pow(10, 30) * mult)
      } else {
        oldVal = 0
      }

      if (typeof newVal === 'number' && typeof oldVal === 'number') {
        expect(closeEnough(newVal, oldVal)).toBe(true)
      } else {
        expect(decimalClose(newVal as Decimal | number, oldVal as Decimal | number)).toBe(true)
      }
    })
  }
})

// ─── challenge15ScoreMultiplier ────────────────────────────────────────────

describe('parity: challenge15ScoreMultiplier', () => {
  const cases = [
    { c15: 1, hept: 0, p15: 0 },
    { c15: 2, hept: 1e4, p15: 1 },
    { c15: 0.5, hept: 5e3, p15: 3 },
    { c15: 1.5, hept: 1e6, p15: 4 }
  ]
  for (const { c15, hept, p15 } of cases) {
    it(`c15=${c15} hept=${hept} p15=${p15}`, () => {
      const oldVal = c15 * (1 + 5 / 10000 * hept) * (1 + 0.25 * p15)
      const newVal = newChal15ScoreMult({
        c15Bonus: c15,
        challengeHepteractEffective: hept,
        platonicUpgrade15: p15
      })
      expect(closeEnough(newVal, oldVal)).toBe(true)
    })
  }
})

// ─── getMaxChallenges ──────────────────────────────────────────────────────

describe('parity: getMaxChallenges', () => {
  it('oneChallengeCap clamps every tier (and c15 still 0)', () => {
    const base = {
      oneChallengeCapEnabled: true,
      infiniteTranscendResearch: 0,
      transcendResearchForChallenge: 0,
      cubeUpgrade29: 0,
      challengeExtensionCap: 0,
      gqReincarnationCapIncrease: 0,
      singReincarnationCapIncrease: 0,
      gqAscensionCapIncrease: 0,
      singAscensionCapIncrease: 0,
      platonicUpgrade5: 0,
      platonicUpgrade10: 0,
      platonicUpgrade15: 0
    }
    expect(newGetMax({ challenge: 1, ...base })).toBe(1)
    expect(newGetMax({ challenge: 6, ...base })).toBe(1)
    expect(newGetMax({ challenge: 14, ...base })).toBe(1)
    expect(newGetMax({ challenge: 15, ...base })).toBe(0)
  })

  it('transcend tier: research 105 short-circuits to 9001', () => {
    const out = newGetMax({
      challenge: 3,
      oneChallengeCapEnabled: false,
      infiniteTranscendResearch: 1,
      transcendResearchForChallenge: 10,
      cubeUpgrade29: 0,
      challengeExtensionCap: 0,
      gqReincarnationCapIncrease: 0,
      singReincarnationCapIncrease: 0,
      gqAscensionCapIncrease: 0,
      singAscensionCapIncrease: 0,
      platonicUpgrade5: 0,
      platonicUpgrade10: 0,
      platonicUpgrade15: 0
    })
    expect(out).toBe(9001)
  })

  it('reincarnation tier sums every contributor (40 + 4*cu29 + ext + plats + gq + sing)', () => {
    const out = newGetMax({
      challenge: 7,
      oneChallengeCapEnabled: false,
      infiniteTranscendResearch: 0,
      transcendResearchForChallenge: 0,
      cubeUpgrade29: 2,
      challengeExtensionCap: 3,
      gqReincarnationCapIncrease: 7,
      singReincarnationCapIncrease: 11,
      gqAscensionCapIncrease: 0,
      singAscensionCapIncrease: 0,
      platonicUpgrade5: 1,
      platonicUpgrade10: 1,
      platonicUpgrade15: 1
    })
    // 40 + 4*2 + 3 + 10 + 10 + 30 + 7 + 11 = 119
    expect(out).toBe(119)
  })

  it('ascension tier (11..14) sums to 30 + 5 + 5 + 20 + asc contributions', () => {
    const out = newGetMax({
      challenge: 12,
      oneChallengeCapEnabled: false,
      infiniteTranscendResearch: 0,
      transcendResearchForChallenge: 0,
      cubeUpgrade29: 0,
      challengeExtensionCap: 0,
      gqReincarnationCapIncrease: 0,
      singReincarnationCapIncrease: 0,
      gqAscensionCapIncrease: 2,
      singAscensionCapIncrease: 3,
      platonicUpgrade5: 1,
      platonicUpgrade10: 1,
      platonicUpgrade15: 1
    })
    // 30 + 5 + 5 + 20 + 2 + 3 = 65
    expect(out).toBe(65)
  })
})

// ─── challengeScoreDisplay (banding for the UI) ───────────────────────────

describe('parity: challengeScoreDisplay bands', () => {
  // Transcend: 0..74 → array1, 75..749 → array2, 750..8999 → array3, 9000+ → array4
  it('transcend c=1', () => {
    expect(newScoreDisplay(1, 0)).toBe(8)
    expect(newScoreDisplay(1, 75)).toBe(10)
    expect(newScoreDisplay(1, 750)).toBe(20)
    expect(newScoreDisplay(1, 9000)).toBe(10000)
  })
  // Reincarnation: 0..24 → array1, 25..59 → array2, 60+ → array3
  it('reincarnation c=7', () => {
    expect(newScoreDisplay(7, 0)).toBe(80)
    expect(newScoreDisplay(7, 25)).toBe(120)
    expect(newScoreDisplay(7, 60)).toBe(300)
  })
  it('out of range → 0', () => {
    expect(newScoreDisplay(0, 100)).toBe(0)
    expect(newScoreDisplay(11, 100)).toBe(0)
    expect(newScoreDisplay(15, 100)).toBe(0)
  })
})

// ─── auto-ascension sweep unlock ───────────────────────────────────────────

describe('parity: autoAscensionChallengeSweepUnlock', () => {
  it('requires both singularity >= 101 AND instantChallenge2 unlocked', () => {
    expect(newAutoAscChallengeSweepUnlock(100, true)).toBe(false)
    expect(newAutoAscChallengeSweepUnlock(101, false)).toBe(false)
    expect(newAutoAscChallengeSweepUnlock(101, true)).toBe(true)
    expect(newAutoAscChallengeSweepUnlock(500, true)).toBe(true)
  })
})

// ─── traversal helpers ────────────────────────────────────────────────────

describe('parity: getNextRegularChallenge', () => {
  it('finds first eligible, wraps around', () => {
    const maxChallenges = [0, 25, 25, 25, 25, 25, 40, 40, 40, 40, 40]
    const highestCompletions = [0, 25, 25, 25, 25, 25, 40, 40, 0, 40, 40] // only c8 eligible
    const autoChallengeToggles = [false, true, true, true, true, true, true, true, true, true, true]
    expect(newNextRegularChallenge({
      startIndex: 1,
      explored: new Set(),
      maxChallenges,
      highestCompletions,
      autoChallengeToggles
    })).toBe(8)
  })

  it('returns -1 if all explored or maxed', () => {
    const maxChallenges = [0, 25, 25, 25, 25, 25, 40, 40, 40, 40, 40]
    const highestCompletions = [0, 25, 25, 25, 25, 25, 40, 40, 40, 40, 40]
    const autoChallengeToggles = [false, true, true, true, true, true, true, true, true, true, true]
    expect(newNextRegularChallenge({
      startIndex: 1,
      explored: new Set(),
      maxChallenges,
      highestCompletions,
      autoChallengeToggles
    })).toBe(-1)
  })

  it('skips toggled-off challenges', () => {
    const maxChallenges = [0, 25, 25, 25, 25, 25, 40, 40, 40, 40, 40]
    const highestCompletions = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    const autoChallengeToggles = [false, false, true, false, false, false, false, false, false, false, false]
    expect(newNextRegularChallenge({
      startIndex: 1,
      explored: new Set(),
      maxChallenges,
      highestCompletions,
      autoChallengeToggles
    })).toBe(2)
  })
})

describe('parity: getNextAscensionChallenge', () => {
  it('wraps 15 → 11', () => {
    const maxChallenges = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30, 30, 30, 30, 0]
    const highestCompletions = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30, 0, 0, 0]
    const autoChallengeToggles = [false, false, false, false, false, false, false, false, false, false, false, true, true, true, true, true]
    // Start at 12: ++ goes to 13, 13 is below max → return 13
    expect(newNextAscChallenge({
      startIndex: 12,
      maxChallenges,
      highestCompletions,
      autoChallengeToggles
    })).toBe(13)
  })

  it('c15 is always eligible if toggled on', () => {
    const maxChallenges = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30, 30, 30, 30, 0]
    const highestCompletions = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30, 30, 30, 30, 0]
    const autoChallengeToggles = [false, false, false, false, false, false, false, false, false, false, false, true, true, true, true, true]
    expect(newNextAscChallenge({
      startIndex: 14,
      maxChallenges,
      highestCompletions,
      autoChallengeToggles
    })).toBe(15)
  })
})
