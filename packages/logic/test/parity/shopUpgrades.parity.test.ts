// Parity tests for the shopUpgrades per-upgrade effect formulas, lifted
// from packages/web_ui/src/Shop.ts. Sweeps cover:
//   - every pure single-key effect across the level grid
//   - multi-key effects across every reward key × level grid
//   - 16 impure effects swept across their extra player-input axes

import { describe, expect, it } from 'vitest'
import * as logic from '../../src/mechanics/shopUpgrades'
import type { ShopPanthemaBonusLevels } from '../../src/mechanics/shopUpgrades'

// ─── Old implementations (verbatim from web_ui Shop.ts) ────────────────────

const oldOfferingPotion = (_n: number) => 7200
const oldObtainiumPotion = (_n: number) => 7200
const oldExMult = (n: number) => (1 + 0.06 * n) * Math.pow(1.08, Math.floor(n / 10))
const oldCubeQuarkConv = (n: number) => n >= 1 ? 1.5 + 0.5 * (1 - Math.pow(0.9, n - 1)) : 1

const oldOfferingAuto = (n: number, key: string): number | boolean => {
  if (key === 'autoRune') return n > 0
  return 1 + 0.01 * n
}
const oldObtainiumAuto = (n: number, key: string): number | boolean => {
  if (key === 'autoResearch') return n > 0
  return 1 - 0.001 * n
}
const oldInstantChallenge = (n: number, key: string): number | boolean => {
  if (key === 'unlocked') return n > 0
  return 10 * n
}
const oldInstantChallenge2 = (n: number, key: string, highestSing: number): number | boolean => {
  if (key === 'unlocked') return n > 0
  return n * highestSing
}
const oldChallengeTome = (n: number, key: string) => key === 'c10RequirementReduction' ? 2e7 * n : -n / 100
const oldShopTalisman = (n: number, pcoin1: boolean) => n > 0 || pcoin1
const oldInfiniteAscent = (n: number, pcoin2: boolean) => n > 0 || pcoin2
const oldSeasonPassZ = (n: number, sc: number) => 1 + 0.01 * n * sc
const oldChronometerZ = (n: number, sc: number) => 1 + 0.001 * n * sc
const oldOfferingEX2 = (n: number, sc: number) => 1 + 0.01 * n * sc
const oldObtainiumEX2 = (n: number, sc: number) => 1 + 0.01 * n * sc
const oldShopOcteractAmbLuck = (n: number, wowOct: number) => n * (1 + Math.floor(Math.max(0, Math.log10(wowOct))))
const oldShopAmbrosiaUltra = (n: number, exalts: number) => 2 * n * exalts
const oldShopInfiniteShopUpgrades = (n: number, exalts: number) => Math.floor(0.01 * n * exalts)
const oldShopAmbrosiaAccelerator = (n: number, ex5: number) => 1 - 0.006 * n * ex5
const oldShopEXUltra = (n: number, lifetimeAmb: number) => 1 + Math.min(125 * n, lifetimeAmb / 1000) / 1000
const oldShopChronometerS = (n: number, sc: number) => Math.pow(1.01, n * Math.max(0, sc - 200))
const oldShopCashGrabUltra = (n: number, key: string, lifetimeAmb: number) => {
  const ratio = Math.min(1, Math.cbrt(lifetimeAmb / 1e7))
  if (key === 'ambrosiaGenerationMult') return 1 + 0.15 * n * ratio
  if (key === 'cubesMult') return 1 + 1.2 * n * ratio
  return 1 + 0.08 * n * ratio
}
const oldShopHorseShoe = (n: number, key: string, horseShoeLevel: number) => {
  if (key === 'bonusHorseLevels') return 3 * n
  return 1 - Math.min(300, horseShoeLevel * n) / 1000
}

const oldCalculator = (n: number, key: string): number | boolean => {
  if (key === 'autoAnswer') return n > 0
  if (key === 'addQuarkMult') return 1 + 0.14 * n
  return n === 5
}
const oldCalculator2 = (n: number, key: string) => key === 'addCodeCapacity' ? 2 * n : (n === 12 ? 1.25 : 1)
const oldCalculator3 = (n: number, key: string) => key === 'addRewardVarianceMultiplier' ? 1 - n / 10 : 60 * n
const oldCalculator4 = (n: number, key: string) => key === 'addCodeIntervalMult' ? 1 - n / 25 : (n === 10 ? 32 : 0)
const oldCalculator5 = (n: number, key: string) =>
  key === 'importGQTimerAdd' ? 6 * n : (Math.floor(n / 10) + (n === 100 ? 6 : 0))
const oldCalculator6 = (n: number, key: string) => key === 'octeractTimerAdd' ? n : (n === 100 ? 24 : 0)
const oldCalculator7 = (n: number, key: string) => key === 'blueberryTimerAdd' ? n : (n === 50 ? 48 : 0)

const oldOfferingEX3 = (n: number, key: string) => key === 'offeringMult' ? Math.pow(1.012, n) : Math.floor(n / 25)
const oldObtainiumEX3 = (n: number, key: string) =>
  key === 'obtainiumMult' ? Math.pow(1.012, n) : Math.pow(1.06, Math.floor(n / 25))
const oldChronometerInfinity = (n: number, key: string) =>
  key === 'ascensionSpeedMult' ? Math.pow(1.006, n) : 0.001 * Math.floor(n / 40)
const oldSeasonPassInfinity = (n: number, key: string) =>
  key === 'globalCubeMult' ? Math.pow(1.012, n) : Math.pow(1.012, n * 1.25)

const oldShopImprovedDaily2 = (n: number, key: string) => key === 'freeSingularityUpgrades' ? n : 1 + 0.2 * n
const oldShopImprovedDaily3 = (n: number, key: string) => key === 'freeSingularityUpgrades' ? n : 1 + 0.15 * n
const oldShopImprovedDaily4 = (n: number, key: string) => key === 'freeSingularityUpgrades' ? n : 1 + 1 * n

const oldShopRedLuckBody = (n: number, key: string, mult: number) =>
  key === 'redLuck' ? mult * n : -0.01 * Math.floor(n / 20)

const oldShopPanthema = (n: number, key: string, b: ShopPanthemaBonusLevels): number => {
  const infinityBoost = 1 + 0.01 * n * b.infinityUpgrades
  if (key === 'infinityMetaBoost') return infinityBoost
  if (key === 'offeringMult') return 1 + 0.01 * n * b.offering * infinityBoost
  if (key === 'obtainiumMult') return 1 + 0.01 * n * b.obtainium * infinityBoost
  if (key === 'cubeMult') return 1 + 0.005 * n * b.cubes * infinityBoost
  if (key === 'ascensionSpeedMult') return 1 + 0.005 * n * b.speed * infinityBoost
  if (key === 'quarkMult') return 1 + 0.001 * n * b.quark * infinityBoost
  if (key === 'ambrosiaGenerationMult') return 1 + 0.001 * n * b.ambrosiaGeneration * infinityBoost
  if (key === 'ambrosiaLuck') return 0.2 * n * b.ambrosiaLuck * infinityBoost
  if (key === 'redLuck') return 0.05 * n * b.redAmbrosiaLuck * infinityBoost
  throw new TypeError(`unknown effect ${key}`)
}

const closeEnough = (a: number | boolean, b: number | boolean): boolean => {
  if (a === b) return true
  if (typeof a === 'number' && typeof b === 'number') {
    if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b
    if (Math.abs(a) < 1 && Math.abs(b) < 1) return Math.abs(a - b) < 1e-12
    return Math.abs(a - b) / Math.max(Math.abs(a), Math.abs(b)) < 1e-10
  }
  return false
}

const levelGrid = [0, 1, 5, 10, 25, 50, 100, 250, 1000]

// ─── Pure single-key effects ──────────────────────────────────────────────

describe('parity: shop pure single-arg effects', () => {
  const cases: { name: string; new: (n: number) => number | boolean; old: (n: number) => number | boolean }[] = [
    { name: 'offeringPotion', new: logic.offeringPotionEffect, old: oldOfferingPotion },
    { name: 'obtainiumPotion', new: logic.obtainiumPotionEffect, old: oldObtainiumPotion },
    { name: 'offeringEX', new: logic.offeringEXEffect, old: oldExMult },
    { name: 'obtainiumEX', new: logic.obtainiumEXEffect, old: oldExMult },
    { name: 'antSpeed', new: logic.antSpeedEffect, old: (n) => 4 * n },
    { name: 'cashGrab', new: logic.cashGrabEffect, old: (n) => 1 + 0.01 * n },
    { name: 'cashGrab2', new: logic.cashGrab2Effect, old: (n) => 1 + 0.005 * n },
    { name: 'shopSadisticRune', new: logic.shopSadisticRuneEffect, old: (n) => n > 0 },
    { name: 'challengeExtension', new: logic.challengeExtensionEffect, old: (n) => 2 * n },
    { name: 'challenge15Auto', new: logic.challenge15AutoEffect, old: (n) => n > 0 },
    { name: 'seasonPass', new: logic.seasonPassEffect, old: (n) => 1 + 0.0225 * n },
    { name: 'seasonPass2', new: logic.seasonPass2Effect, old: (n) => 1 + 0.015 * n },
    { name: 'seasonPass3', new: logic.seasonPass3Effect, old: (n) => 1 + 0.015 * n },
    { name: 'seasonPassY', new: logic.seasonPassYEffect, old: (n) => 1 + 0.0075 * n },
    { name: 'seasonPassLost', new: logic.seasonPassLostEffect, old: (n) => 1 + 0.001 * n },
    { name: 'chronometer', new: logic.chronometerEffect, old: (n) => 1 + 0.012 * n },
    { name: 'chronometer2', new: logic.chronometer2Effect, old: (n) => 1 + 0.006 * n },
    { name: 'chronometer3', new: logic.chronometer3Effect, old: (n) => 1 + 0.015 * n },
    { name: 'cubeToQuark', new: logic.cubeToQuarkEffect, old: oldCubeQuarkConv },
    { name: 'tesseractToQuark', new: logic.tesseractToQuarkEffect, old: oldCubeQuarkConv },
    { name: 'hypercubeToQuark', new: logic.hypercubeToQuarkEffect, old: oldCubeQuarkConv },
    { name: 'cubeToQuarkAll', new: logic.cubeToQuarkAllEffect, old: (n) => 1 + 0.002 * n },
    { name: 'constantEX', new: logic.constantEXEffect, old: (n) => n },
    { name: 'powderEX', new: logic.powderEXEffect, old: (n) => 1 + 0.02 * n },
    { name: 'powderAuto', new: logic.powderAutoEffect, old: (n) => 0.01 * n },
    { name: 'autoWarp', new: logic.autoWarpEffect, old: (n) => n > 0 },
    { name: 'extraWarp', new: logic.extraWarpEffect, old: (n) => n },
    { name: 'improveQuarkHept', new: logic.improveQuarkHeptEffect, old: (n) => 0.01 * n },
    { name: 'improveQuarkHept2', new: logic.improveQuarkHept2Effect, old: (n) => 0.01 * n },
    { name: 'improveQuarkHept3', new: logic.improveQuarkHept3Effect, old: (n) => 0.01 * n },
    { name: 'improveQuarkHept4', new: logic.improveQuarkHept4Effect, old: (n) => 0.01 * n },
    { name: 'improveQuarkHept5', new: logic.improveQuarkHept5Effect, old: (n) => 0.0001 * n },
    { name: 'shopImprovedDaily', new: logic.shopImprovedDailyEffect, old: (n) => 1 + 0.05 * n },
    { name: 'shopAmbrosiaGeneration1', new: logic.shopAmbrosiaGeneration1Effect, old: (n) => 1 + 0.01 * n },
    { name: 'shopAmbrosiaGeneration2', new: logic.shopAmbrosiaGeneration2Effect, old: (n) => 1 + 0.01 * n },
    { name: 'shopAmbrosiaGeneration3', new: logic.shopAmbrosiaGeneration3Effect, old: (n) => 1 + 0.01 * n },
    { name: 'shopAmbrosiaGeneration4', new: logic.shopAmbrosiaGeneration4Effect, old: (n) => 1 + 0.001 * n },
    { name: 'shopAmbrosiaLuck1', new: logic.shopAmbrosiaLuck1Effect, old: (n) => 2 * n },
    { name: 'shopAmbrosiaLuck2', new: logic.shopAmbrosiaLuck2Effect, old: (n) => 2 * n },
    { name: 'shopAmbrosiaLuck3', new: logic.shopAmbrosiaLuck3Effect, old: (n) => 2 * n },
    { name: 'shopAmbrosiaLuck4', new: logic.shopAmbrosiaLuck4Effect, old: (n) => 0.6 * n },
    { name: 'shopAmbrosiaLuckMultiplier4', new: logic.shopAmbrosiaLuckMultiplier4Effect, old: (n) => 0.01 * n },
    { name: 'shopSingularitySpeedup', new: logic.shopSingularitySpeedupEffect, old: (n) => n > 0 ? 50 : 1 },
    { name: 'shopSingularityPotency', new: logic.shopSingularityPotencyEffect, old: (n) => n > 0 ? 3.66 : 1 },
    { name: 'shopSingularityPenaltyDebuff', new: logic.shopSingularityPenaltyDebuffEffect, old: (n) => n }
  ]
  for (const c of cases) {
    for (const n of levelGrid) {
      it(`${c.name} n=${n}`, () => {
        expect(closeEnough(c.new(n), c.old(n))).toBe(true)
      })
    }
  }
})

// ─── Multi-key effects ─────────────────────────────────────────────────────

const multiKeyCases: {
  name: string
  new: (n: number, key: string) => number | boolean
  old: (n: number, key: string) => number | boolean
  keys: readonly string[]
  levels: number[]
}[] = [
  {
    name: 'offeringAuto',
    new: (n, k) => logic.offeringAutoEffect(n, k as 'autoRune' | 'autoRuneSpeedMult'),
    old: oldOfferingAuto,
    keys: ['autoRune', 'autoRuneSpeedMult'],
    levels: levelGrid
  },
  {
    name: 'obtainiumAuto',
    new: (n, k) => logic.obtainiumAutoEffect(n, k as 'autoResearch' | 'researchCostMult'),
    old: oldObtainiumAuto,
    keys: ['autoResearch', 'researchCostMult'],
    levels: levelGrid
  },
  {
    name: 'instantChallenge',
    new: (n, k) => logic.instantChallengeEffect(n, k as 'unlocked' | 'extraCompPerTick'),
    old: oldInstantChallenge,
    keys: ['unlocked', 'extraCompPerTick'],
    levels: levelGrid
  },
  {
    name: 'challengeTome',
    new: (n, k) => logic.challengeTomeEffect(n, k as 'c10RequirementReduction' | 'c9c10ScalingReduction'),
    old: oldChallengeTome,
    keys: ['c10RequirementReduction', 'c9c10ScalingReduction'],
    levels: levelGrid
  },
  {
    name: 'challengeTome2',
    new: (n, k) => logic.challengeTome2Effect(n, k as 'c10RequirementReduction' | 'c9c10ScalingReduction'),
    old: oldChallengeTome,
    keys: ['c10RequirementReduction', 'c9c10ScalingReduction'],
    levels: levelGrid
  },
  {
    name: 'calculator',
    new: (n, k) => logic.calculatorEffect(n, k as 'addQuarkMult' | 'autoAnswer' | 'autoFill'),
    old: oldCalculator,
    keys: ['addQuarkMult', 'autoAnswer', 'autoFill'],
    levels: [0, 1, 5, 10]
  },
  {
    name: 'calculator2',
    new: (n, k) => logic.calculator2Effect(n, k as 'addCodeCapacity' | 'addQuarkMult'),
    old: oldCalculator2,
    keys: ['addCodeCapacity', 'addQuarkMult'],
    levels: [0, 1, 11, 12, 13]
  },
  {
    name: 'calculator3',
    new: (n, k) => logic.calculator3Effect(n, k as 'addRewardVarianceMultiplier' | 'ascensionTimerAdd'),
    old: oldCalculator3,
    keys: ['addRewardVarianceMultiplier', 'ascensionTimerAdd'],
    levels: [0, 1, 5, 10]
  },
  {
    name: 'calculator4',
    new: (n, k) => logic.calculator4Effect(n, k as 'addCodeIntervalMult' | 'addCodeCapacity'),
    old: oldCalculator4,
    keys: ['addCodeIntervalMult', 'addCodeCapacity'],
    levels: [0, 1, 9, 10, 11]
  },
  {
    name: 'calculator5',
    new: (n, k) => logic.calculator5Effect(n, k as 'importGQTimerAdd' | 'addCodeCapacity'),
    old: oldCalculator5,
    keys: ['importGQTimerAdd', 'addCodeCapacity'],
    levels: [0, 1, 50, 99, 100]
  },
  {
    name: 'calculator6',
    new: (n, k) => logic.calculator6Effect(n, k as 'octeractTimerAdd' | 'addCodeCapacity'),
    old: oldCalculator6,
    keys: ['octeractTimerAdd', 'addCodeCapacity'],
    levels: [0, 1, 50, 99, 100]
  },
  {
    name: 'calculator7',
    new: (n, k) => logic.calculator7Effect(n, k as 'blueberryTimerAdd' | 'addCodeCapacity'),
    old: oldCalculator7,
    keys: ['blueberryTimerAdd', 'addCodeCapacity'],
    levels: [0, 1, 25, 49, 50]
  },
  {
    name: 'offeringEX3',
    new: (n, k) => logic.offeringEX3Effect(n, k as 'offeringMult' | 'baseOfferings'),
    old: oldOfferingEX3,
    keys: ['offeringMult', 'baseOfferings'],
    levels: levelGrid
  },
  {
    name: 'obtainiumEX3',
    new: (n, k) => logic.obtainiumEX3Effect(n, k as 'obtainiumMult' | 'immaculateObtainiuMult'),
    old: oldObtainiumEX3,
    keys: ['obtainiumMult', 'immaculateObtainiuMult'],
    levels: levelGrid
  },
  {
    name: 'chronometerInfinity',
    new: (n, k) => logic.chronometerInfinityEffect(n, k as 'ascensionSpeedMult' | 'exponentSpread'),
    old: oldChronometerInfinity,
    keys: ['ascensionSpeedMult', 'exponentSpread'],
    levels: levelGrid
  },
  {
    name: 'seasonPassInfinity',
    new: (n, k) => logic.seasonPassInfinityEffect(n, k as 'globalCubeMult' | 'wowOcteractMult'),
    old: oldSeasonPassInfinity,
    keys: ['globalCubeMult', 'wowOcteractMult'],
    levels: levelGrid
  },
  {
    name: 'shopImprovedDaily2',
    new: (n, k) => logic.shopImprovedDaily2Effect(n, k as 'freeSingularityUpgrades' | 'dailyCodeGoldenQuarkMult'),
    old: oldShopImprovedDaily2,
    keys: ['freeSingularityUpgrades', 'dailyCodeGoldenQuarkMult'],
    levels: levelGrid
  },
  {
    name: 'shopImprovedDaily3',
    new: (n, k) => logic.shopImprovedDaily3Effect(n, k as 'freeSingularityUpgrades' | 'dailyCodeGoldenQuarkMult'),
    old: oldShopImprovedDaily3,
    keys: ['freeSingularityUpgrades', 'dailyCodeGoldenQuarkMult'],
    levels: levelGrid
  },
  {
    name: 'shopImprovedDaily4',
    new: (n, k) => logic.shopImprovedDaily4Effect(n, k as 'freeSingularityUpgrades' | 'dailyCodeGoldenQuarkMult'),
    old: oldShopImprovedDaily4,
    keys: ['freeSingularityUpgrades', 'dailyCodeGoldenQuarkMult'],
    levels: levelGrid
  },
  {
    name: 'shopRedLuck1',
    new: (n, k) => logic.shopRedLuck1Effect(n, k as 'redLuck' | 'luckConversionRatio'),
    old: (n, k) => oldShopRedLuckBody(n, k, 0.05),
    keys: ['redLuck', 'luckConversionRatio'],
    levels: levelGrid
  },
  {
    name: 'shopRedLuck2',
    new: (n, k) => logic.shopRedLuck2Effect(n, k as 'redLuck' | 'luckConversionRatio'),
    old: (n, k) => oldShopRedLuckBody(n, k, 0.075),
    keys: ['redLuck', 'luckConversionRatio'],
    levels: levelGrid
  },
  {
    name: 'shopRedLuck3',
    new: (n, k) => logic.shopRedLuck3Effect(n, k as 'redLuck' | 'luckConversionRatio'),
    old: (n, k) => oldShopRedLuckBody(n, k, 0.1),
    keys: ['redLuck', 'luckConversionRatio'],
    levels: levelGrid
  }
]

describe('parity: shop multi-key effects', () => {
  for (const c of multiKeyCases) {
    for (const key of c.keys) {
      for (const n of c.levels) {
        it(`${c.name} key=${key} n=${n}`, () => {
          expect(closeEnough(c.new(n, key), c.old(n, key))).toBe(true)
        })
      }
    }
  }
})

// ─── Impure single-arg effects (n × extra) ────────────────────────────────

const singularityCountGrid = [0, 1, 10, 100, 1000]
const wowOcteractsGrid = [0, 1, 1e3, 1e10, 1e100]
const exaltCompletionsGrid = [0, 1, 50, 500]
const lifetimeAmbGrid = [0, 1, 1e3, 1e7, 1e10, 1e15]

describe('parity: shopTalismanEffect (n × pcoin1)', () => {
  for (const pcoin1 of [false, true]) {
    for (const n of [0, 1]) {
      it(`n=${n} pcoin1=${pcoin1}`, () => {
        expect(closeEnough(logic.shopTalismanEffect(n, pcoin1), oldShopTalisman(n, pcoin1))).toBe(true)
      })
    }
  }
})

describe('parity: infiniteAscentEffect (n × pcoin2)', () => {
  for (const pcoin2 of [false, true]) {
    for (const n of [0, 1]) {
      it(`n=${n} pcoin2=${pcoin2}`, () => {
        expect(closeEnough(logic.infiniteAscentEffect(n, pcoin2), oldInfiniteAscent(n, pcoin2))).toBe(true)
      })
    }
  }
})

describe('parity: seasonPassZEffect (n × singularityCount)', () => {
  for (const n of levelGrid) {
    for (const sc of singularityCountGrid) {
      it(`n=${n} sc=${sc}`, () => {
        expect(closeEnough(logic.seasonPassZEffect(n, sc), oldSeasonPassZ(n, sc))).toBe(true)
      })
    }
  }
})

describe('parity: chronometerZEffect (n × singularityCount)', () => {
  for (const n of levelGrid) {
    for (const sc of singularityCountGrid) {
      it(`n=${n} sc=${sc}`, () => {
        expect(closeEnough(logic.chronometerZEffect(n, sc), oldChronometerZ(n, sc))).toBe(true)
      })
    }
  }
})

describe('parity: offeringEX2Effect (n × singularityCount)', () => {
  for (const n of levelGrid) {
    for (const sc of singularityCountGrid) {
      it(`n=${n} sc=${sc}`, () => {
        expect(closeEnough(logic.offeringEX2Effect(n, sc), oldOfferingEX2(n, sc))).toBe(true)
      })
    }
  }
})

describe('parity: obtainiumEX2Effect (n × singularityCount)', () => {
  for (const n of levelGrid) {
    for (const sc of singularityCountGrid) {
      it(`n=${n} sc=${sc}`, () => {
        expect(closeEnough(logic.obtainiumEX2Effect(n, sc), oldObtainiumEX2(n, sc))).toBe(true)
      })
    }
  }
})

describe('parity: instantChallenge2Effect (n × key × highestSingularityCount)', () => {
  for (const key of ['unlocked', 'extraCompPerTick'] as const) {
    for (const n of [0, 1, 5]) {
      for (const hs of singularityCountGrid) {
        it(`n=${n} key=${key} hs=${hs}`, () => {
          expect(closeEnough(
            logic.instantChallenge2Effect(n, key, hs),
            oldInstantChallenge2(n, key, hs)
          )).toBe(true)
        })
      }
    }
  }
})

describe('parity: shopOcteractAmbrosiaLuckEffect (n × wowOcteracts)', () => {
  for (const n of levelGrid) {
    for (const wo of wowOcteractsGrid) {
      it(`n=${n} wowOct=${wo}`, () => {
        expect(closeEnough(
          logic.shopOcteractAmbrosiaLuckEffect(n, wo),
          oldShopOcteractAmbLuck(n, wo)
        )).toBe(true)
      })
    }
  }
})

describe('parity: shopAmbrosiaUltraEffect (n × exalts)', () => {
  for (const n of levelGrid) {
    for (const e of exaltCompletionsGrid) {
      it(`n=${n} exalts=${e}`, () => {
        expect(closeEnough(logic.shopAmbrosiaUltraEffect(n, e), oldShopAmbrosiaUltra(n, e))).toBe(true)
      })
    }
  }
})

describe('parity: shopInfiniteShopUpgradesEffect (n × exalts)', () => {
  for (const n of levelGrid) {
    for (const e of exaltCompletionsGrid) {
      it(`n=${n} exalts=${e}`, () => {
        expect(closeEnough(
          logic.shopInfiniteShopUpgradesEffect(n, e),
          oldShopInfiniteShopUpgrades(n, e)
        )).toBe(true)
      })
    }
  }
})

describe('parity: shopAmbrosiaAcceleratorEffect (n × ex5Completions)', () => {
  for (const n of levelGrid) {
    for (const e5 of [0, 1, 5, 10]) {
      it(`n=${n} ex5=${e5}`, () => {
        expect(closeEnough(
          logic.shopAmbrosiaAcceleratorEffect(n, e5),
          oldShopAmbrosiaAccelerator(n, e5)
        )).toBe(true)
      })
    }
  }
})

describe('parity: shopEXUltraEffect (n × lifetimeAmbrosia)', () => {
  for (const n of levelGrid) {
    for (const la of lifetimeAmbGrid) {
      it(`n=${n} lifetimeAmb=${la}`, () => {
        expect(closeEnough(logic.shopEXUltraEffect(n, la), oldShopEXUltra(n, la))).toBe(true)
      })
    }
  }
})

describe('parity: shopChronometerSEffect (n × singularityCount)', () => {
  for (const n of [0, 1, 5, 10]) {
    for (const sc of [0, 50, 200, 201, 300, 1000]) {
      it(`n=${n} sc=${sc}`, () => {
        expect(closeEnough(logic.shopChronometerSEffect(n, sc), oldShopChronometerS(n, sc))).toBe(true)
      })
    }
  }
})

describe('parity: shopCashGrabUltraEffect (n × key × lifetimeAmbrosia)', () => {
  const keys = ['ambrosiaGenerationMult', 'cubesMult', 'quarkMult'] as const
  for (const key of keys) {
    for (const n of [0, 1, 5, 10]) {
      for (const la of lifetimeAmbGrid) {
        it(`n=${n} key=${key} lifetimeAmb=${la}`, () => {
          expect(closeEnough(
            logic.shopCashGrabUltraEffect(n, key, la),
            oldShopCashGrabUltra(n, key, la)
          )).toBe(true)
        })
      }
    }
  }
})

describe('parity: shopHorseShoeEffect (n × key × horseShoeRuneLevel)', () => {
  const keys = ['bonusHorseLevels', 'singularityPenaltyMult'] as const
  for (const key of keys) {
    for (const n of [0, 1, 5, 10, 100]) {
      for (const hsl of [0, 10, 100, 300, 1000]) {
        it(`n=${n} key=${key} hsl=${hsl}`, () => {
          expect(closeEnough(
            logic.shopHorseShoeEffect(n, key, hsl),
            oldShopHorseShoe(n, key, hsl)
          )).toBe(true)
        })
      }
    }
  }
})

describe('parity: shopPanthemaEffect (n × key × bonusLevels)', () => {
  const allKeys = [
    'offeringMult',
    'obtainiumMult',
    'cubeMult',
    'quarkMult',
    'ascensionSpeedMult',
    'ambrosiaGenerationMult',
    'ambrosiaLuck',
    'redLuck',
    'infinityMetaBoost'
  ] as const
  const bonusSamples: ShopPanthemaBonusLevels[] = [
    {
      offering: 0,
      obtainium: 0,
      cubes: 0,
      speed: 0,
      quark: 0,
      ambrosiaLuck: 0,
      redAmbrosiaLuck: 0,
      ambrosiaGeneration: 0,
      infinityUpgrades: 0
    },
    {
      offering: 1,
      obtainium: 1,
      cubes: 1,
      speed: 1,
      quark: 1,
      ambrosiaLuck: 1,
      redAmbrosiaLuck: 1,
      ambrosiaGeneration: 1,
      infinityUpgrades: 1
    },
    {
      offering: 10,
      obtainium: 20,
      cubes: 30,
      speed: 5,
      quark: 100,
      ambrosiaLuck: 50,
      redAmbrosiaLuck: 25,
      ambrosiaGeneration: 40,
      infinityUpgrades: 10
    }
  ]
  for (const key of allKeys) {
    for (const n of [0, 1, 5, 25]) {
      for (let bi = 0; bi < bonusSamples.length; bi++) {
        const b = bonusSamples[bi]
        it(`n=${n} key=${key} bonusSample=${bi}`, () => {
          expect(closeEnough(
            logic.shopPanthemaEffect(n, key, b),
            oldShopPanthema(n, key, b)
          )).toBe(true)
        })
      }
    }
  }
})
