// Parity tests for the SingularityChallenges per-challenge formulas, lifted
// from packages/web_ui/src/SingularityChallenges.ts. Sweeps cover:
//   - singularityRequirement: every piecewise boundary per challenge
//     (the linear ones at 0/1/10 + max; the piecewise ones around their
//     specific knees at 2/5/6, 9/10, 14, etc.)
//   - achievementPointValue: all linear, sample at 0/1/max
//   - effect: every reward key × representative completion counts
//     (including the "is unlocked at N" boolean boundaries)

import { describe, expect, it } from 'vitest'
import {
  limitedAscensionsAchievementPointValue as newLimAscAP,
  limitedAscensionsEffect as newLimAscEffect,
  limitedAscensionsSingularityRequirement as newLimAscSR,
  limitedTimeAchievementPointValue as newLimTimeAP,
  limitedTimeEffect as newLimTimeEffect,
  limitedTimeSingularityRequirement as newLimTimeSR,
  noAmbrosiaUpgradesAchievementPointValue as newNoAmbAP,
  noAmbrosiaUpgradesEffect as newNoAmbEffect,
  noAmbrosiaUpgradesSingularityRequirement as newNoAmbSR,
  noOcteractsAchievementPointValue as newNoOctAP,
  noOcteractsEffect as newNoOctEffect,
  noOcteractsSingularityRequirement as newNoOctSR,
  noQuarkUpgradesAchievementPointValue as newNoQuarkAP,
  noQuarkUpgradesEffect as newNoQuarkEffect,
  noQuarkUpgradesSingularityRequirement as newNoQuarkSR,
  noSingularityUpgradesAchievementPointValue as newNoSingAP,
  noSingularityUpgradesEffect as newNoSingEffect,
  noSingularityUpgradesSingularityRequirement as newNoSingSR,
  oneChallengeCapAchievementPointValue as newOneChalAP,
  oneChallengeCapEffect as newOneChalEffect,
  oneChallengeCapSingularityRequirement as newOneChalSR,
  sadisticPrequelAchievementPointValue as newSadAP,
  sadisticPrequelEffect as newSadEffect,
  sadisticPrequelSingularityRequirement as newSadSR,
  taxmanLastStandAchievementPointValue as newTaxAP,
  taxmanLastStandEffect as newTaxEffect,
  taxmanLastStandSingularityRequirement as newTaxSR
} from '../../src/mechanics/singularityChallenges'

// ─── Old implementations (verbatim from web_ui SingularityChallenges.ts) ───

const oldNoSingSR = (baseReq: number, completions: number) =>
  baseReq + 16 * completions + 8 * (completions >= 9 ? 1 : 0)

const oldOneChalSR = (baseReq: number, completions: number) =>
  baseReq + 19 * completions - 2 * (completions >= 14 ? 1 : 0)

const oldNoOctSR = (baseReq: number, completions: number) => {
  if (completions < 10) return baseReq + 13 * completions
  return baseReq + 13 * 9 + 10 * (completions - 9)
}

const oldLimAscSR = (baseReq: number, completions: number) => baseReq + 27 * completions

const oldNoAmbSR = (baseReq: number, completions: number) => {
  if (completions < 10) return baseReq + 12 * completions
  return baseReq + 12 * 9 + 4 * (completions - 9)
}

const oldNoQuarkSR = (baseReq: number, completions: number) => {
  if (completions > 5) return baseReq + 185 + 8 * (completions - 6)
  if (completions > 2) return baseReq + 70 + 9 * (completions - 6)
  return baseReq + 15 * completions
}

const oldLimTimeSR = (baseReq: number, completions: number) => {
  if (completions > 9) return 277 + 2 * (completions - 10)
  return baseReq + 8 * completions
}

const oldSadSR = (baseReq: number, completions: number) => baseReq + 8 * completions
const oldTaxSR = (baseReq: number, completions: number) => baseReq + 4 * completions

const oldNoSingEffect = (n: number, key: string): number | boolean => {
  if (key === 'cubes') return 1 + n
  if (key === 'goldenQuarks') return 1 + 0.12 * +(n > 0)
  if (key === 'blueberries') return +(n > 0)
  if (key === 'shopUpgrade') return n >= 10
  if (key === 'additiveLuckMult') return n >= 15 ? 0.05 : 0
  return n >= 15 // shopUpgrade2
}

const oldOneChalEffect = (n: number, key: string): number | boolean => {
  if (key === 'corrScoreIncrease') return 0.05 * n
  if (key === 'blueberrySpeedMult') return 1 + n / 60
  if (key === 'capIncrease') return 3 * +(n > 0)
  if (key === 'freeCorruptionLevel') return +(n >= 12)
  if (key === 'shopUpgrade') return n >= 12
  if (key === 'reinCapIncrease2') return 7 * +(n >= 15)
  return 2 * +(n >= 15) // ascCapIncrease2
}

const oldNoOctEffect = (n: number, key: string): number | boolean => {
  if (key === 'octeractPow') return n <= 10 ? 0.02 * n : 0.2 + (n - 10) / 100
  if (key === 'offeringBonus') return n > 0
  if (key === 'obtainiumBonus') return n >= 10
  return n >= 10 // shopUpgrade
}

const oldLimAscEffect = (n: number, key: string): number | boolean => {
  if (key === 'ascensionSpeedMult') return 1 + 0.25 * n / 100
  if (key === 'hepteractCap') return n > 0
  if (key === 'shopUpgrade') return n >= 8
  return n >= 10 // shopUpgrade2
}

const oldNoAmbEffect = (n: number, key: string): number | boolean => {
  if (key === 'bonusAmbrosia') return +(n > 0)
  if (key === 'blueberries') return Math.floor(n / 5) + +(n > 0)
  if (key === 'additiveLuckMult') return n / 200
  if (key === 'ambrosiaLuck') return 20 * n
  if (key === 'redLuck') return 4 * n
  if (key === 'blueberrySpeedMult') return 1 + n / 25
  if (key === 'redSpeedMult') return 1 + 2 * n / 100
  if (key === 'shopUpgrade') return n >= 8
  return n >= 10 // shopUpgrade2
}

const oldNoQuarkEffect = (n: number, key: string): number | boolean => {
  if (key === 'freeObtainiumLevels') return n
  if (key === 'freeOfferingLevels') return n
  if (key === 'freeSpeedLevels') return n
  if (key === 'freeCubeLevels') return n
  if (key === 'freeQuarkLevel') return n >= 5 ? 1 : 0
  if (key === 'freeInfinityLevels') return n
  if (key === 'shopUpgrade') return n >= 1
  return n >= 10 // topHatUnlock
}

const oldLimTimeEffect = (n: number, key: string): number | boolean => {
  if (key === 'preserveQuarks') return +(n > 0)
  if (key === 'quarkMult') return 1 + 0.02 * n
  if (key === 'globalSpeed') return 1 + 0.12 * n
  if (key === 'ascensionSpeed') return 1 + 0.12 * n
  if (key === 'barRequirementMultiplier') return 1 - 0.02 * n
  if (key === 'shopUpgrade') return n >= 5
  return n >= 10 // shopUpgrade2
}

const oldSadEffect = (n: number, key: string): number | boolean => {
  if (key === 'extraFree') return 50 * +(n > 0)
  if (key === 'quarkMult') return 1 + 0.06 * n
  if (key === 'freeUpgradeMult') return 1 + 0.06 * n
  if (key === 'shopUpgrade') return n >= 5
  if (key === 'shopUpgrade2') return n >= 10
  return n >= 15 // shopUpgrade3
}

const oldTaxEffect = (n: number, key: string): number | boolean => {
  if (key === 'horseShoeUnlock') return n > 0
  if (key === 'shopUpgrade') return n >= 5
  if (key === 'talismanUnlock') return n >= 10
  if (key === 'talismanFreeLevel') return 25 * n
  if (key === 'talismanRuneEffect') return 0.03 * n
  if (key === 'antiquityOOM') return 1 / 50 * n / 10
  return 1 / 20 * n / 10 // horseShoeOOM
}

const closeEnough = (a: number | boolean, b: number | boolean): boolean => {
  if (a === b) return true
  if (typeof a === 'number' && typeof b === 'number') {
    if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < 1e-12
    return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < 1e-12
  }
  return false
}

// Completion grid covering 0, 1, every documented knee (2, 5, 8, 9, 10, 12,
// 14, 15), and max+1 for each challenge.
const completionsGrid = [0, 1, 2, 3, 5, 6, 8, 9, 10, 11, 12, 14, 15, 16]

// ─── singularityRequirement (per-challenge piecewise knees) ────────────────

describe('parity: singularityRequirement (all 9 challenges)', () => {
  const cases: { name: string; baseReq: number; newFn: typeof oldNoSingSR; oldFn: typeof oldNoSingSR }[] = [
    { name: 'noSingularityUpgrades', baseReq: 1, newFn: newNoSingSR, oldFn: oldNoSingSR },
    { name: 'oneChallengeCap', baseReq: 10, newFn: newOneChalSR, oldFn: oldOneChalSR },
    { name: 'noOcteracts', baseReq: 75, newFn: newNoOctSR, oldFn: oldNoOctSR },
    { name: 'limitedAscensions', baseReq: 7, newFn: newLimAscSR, oldFn: oldLimAscSR },
    { name: 'noAmbrosiaUpgrades', baseReq: 150, newFn: newNoAmbSR, oldFn: oldNoAmbSR },
    { name: 'noQuarkUpgrades', baseReq: 20, newFn: newNoQuarkSR, oldFn: oldNoQuarkSR },
    { name: 'limitedTime', baseReq: 203, newFn: newLimTimeSR, oldFn: oldLimTimeSR },
    { name: 'sadisticPrequel', baseReq: 120, newFn: newSadSR, oldFn: oldSadSR },
    { name: 'taxmanLastStand', baseReq: 240, newFn: newTaxSR, oldFn: oldTaxSR }
  ]
  for (const c of cases) {
    for (const completions of completionsGrid) {
      it(`${c.name} comp=${completions}`, () => {
        expect(c.newFn(c.baseReq, completions)).toBe(c.oldFn(c.baseReq, completions))
      })
    }
  }
})

// ─── achievementPointValue (all linear) ────────────────────────────────────

describe('parity: achievementPointValue (all 9 challenges)', () => {
  const cases: { name: string; newFn: (n: number) => number; oldFn: (n: number) => number }[] = [
    { name: 'noSingularityUpgrades', newFn: newNoSingAP, oldFn: (n) => 15 * n },
    { name: 'oneChallengeCap', newFn: newOneChalAP, oldFn: (n) => 15 * n },
    { name: 'noOcteracts', newFn: newNoOctAP, oldFn: (n) => 20 * n },
    { name: 'limitedAscensions', newFn: newLimAscAP, oldFn: (n) => 30 * n },
    { name: 'noAmbrosiaUpgrades', newFn: newNoAmbAP, oldFn: (n) => 25 * n },
    { name: 'noQuarkUpgrades', newFn: newNoQuarkAP, oldFn: (n) => 20 * n },
    { name: 'limitedTime', newFn: newLimTimeAP, oldFn: (n) => 30 * n },
    { name: 'sadisticPrequel', newFn: newSadAP, oldFn: (n) => 40 * n },
    { name: 'taxmanLastStand', newFn: newTaxAP, oldFn: (n) => 50 * n }
  ]
  for (const c of cases) {
    for (const n of [0, 1, 5, 10, 15]) {
      it(`${c.name} n=${n}`, () => {
        expect(c.newFn(n)).toBe(c.oldFn(n))
      })
    }
  }
})

// ─── effect (per-challenge × per-reward-key) ──────────────────────────────

// Helper to run a parity assertion across the cartesian product of completions
// × keys for one challenge.
const runEffectParity = <K extends string>(
  challengeName: string,
  newFn: (n: number, key: K) => number | boolean,
  oldFn: (n: number, key: string) => number | boolean,
  keys: K[]
) => {
  describe(`parity: ${challengeName}Effect`, () => {
    for (const key of keys) {
      for (const completions of completionsGrid) {
        it(`key=${key} comp=${completions}`, () => {
          const next = newFn(completions, key)
          const old = oldFn(completions, key)
          expect(closeEnough(next, old)).toBe(true)
        })
      }
    }
  })
}

runEffectParity('noSingularityUpgrades', newNoSingEffect, oldNoSingEffect, [
  'cubes',
  'goldenQuarks',
  'blueberries',
  'shopUpgrade',
  'additiveLuckMult',
  'shopUpgrade2'
])

runEffectParity('oneChallengeCap', newOneChalEffect, oldOneChalEffect, [
  'corrScoreIncrease',
  'blueberrySpeedMult',
  'capIncrease',
  'freeCorruptionLevel',
  'shopUpgrade',
  'reinCapIncrease2',
  'ascCapIncrease2'
])

runEffectParity('noOcteracts', newNoOctEffect, oldNoOctEffect, [
  'octeractPow',
  'offeringBonus',
  'obtainiumBonus',
  'shopUpgrade'
])

runEffectParity('limitedAscensions', newLimAscEffect, oldLimAscEffect, [
  'ascensionSpeedMult',
  'hepteractCap',
  'shopUpgrade',
  'shopUpgrade2'
])

runEffectParity('noAmbrosiaUpgrades', newNoAmbEffect, oldNoAmbEffect, [
  'bonusAmbrosia',
  'blueberries',
  'additiveLuckMult',
  'ambrosiaLuck',
  'redLuck',
  'blueberrySpeedMult',
  'redSpeedMult',
  'shopUpgrade',
  'shopUpgrade2'
])

runEffectParity('noQuarkUpgrades', newNoQuarkEffect, oldNoQuarkEffect, [
  'freeObtainiumLevels',
  'freeOfferingLevels',
  'freeSpeedLevels',
  'freeCubeLevels',
  'freeQuarkLevel',
  'freeInfinityLevels',
  'shopUpgrade',
  'topHatUnlock'
])

runEffectParity('limitedTime', newLimTimeEffect, oldLimTimeEffect, [
  'preserveQuarks',
  'quarkMult',
  'globalSpeed',
  'ascensionSpeed',
  'barRequirementMultiplier',
  'shopUpgrade',
  'shopUpgrade2'
])

runEffectParity('sadisticPrequel', newSadEffect, oldSadEffect, [
  'extraFree',
  'quarkMult',
  'freeUpgradeMult',
  'shopUpgrade',
  'shopUpgrade2',
  'shopUpgrade3'
])

runEffectParity('taxmanLastStand', newTaxEffect, oldTaxEffect, [
  'horseShoeUnlock',
  'shopUpgrade',
  'talismanUnlock',
  'talismanFreeLevel',
  'talismanRuneEffect',
  'antiquityOOM',
  'horseShoeOOM'
])
