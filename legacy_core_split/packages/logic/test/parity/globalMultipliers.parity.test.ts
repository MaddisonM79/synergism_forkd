// Parity tests for computeGlobalMultipliers. Old body transcribed verbatim from
// packages/web_ui/src/Synergism.ts (multipliers, ~line 2572 pre-migration).

import Decimal from 'break_infinity.js'
import { describe, expect, it } from 'vitest'
import { CalcECC } from '../../src/mechanics/challenges'
import {
  computeGlobalMultipliers as newComputeGlobalMultipliers,
  type GlobalMultipliersInput,
  type GlobalMultipliersResult
} from '../../src/mechanics/globalMultipliers'

const oldComputeGlobalMultipliers = (input: GlobalMultipliersInput): GlobalMultipliersResult => {
  let s = new Decimal(1)
  let c = new Decimal(1)

  s = s.times(input.multiplierEffect)
  s = s.times(input.acceleratorEffect)
  s = s.times(input.crystalMult)
  s = s.times(input.buildingPowerMult)
  s = s.times(input.antMultiplier)

  const first6CoinUp = new Decimal(input.totalCoinOwned + 1).times(
    Decimal.min(1e30, Decimal.pow(1.008, input.totalCoinOwned))
  )

  if (input.highestSingularityCount > 0) {
    s = s.times(
      Math.pow(input.goldenQuarks + 1, 1.5)
        * Math.pow(input.highestSingularityCount + 1, 2)
    )
  }
  if (input.upgrade6 > 0.5) s = s.times(first6CoinUp)
  if (input.upgrade12 > 0.5) s = s.times(Decimal.min(1e4, Decimal.pow(1.01, input.prestigeCount)))
  if (input.upgrade20 > 0.5) s = s.times(Decimal.pow(input.totalCoinOwned / 4 + 1, 10))
  if (input.upgrade41 > 0.5) {
    s = s.times(Decimal.min(1e30, Decimal.pow(input.transcendPoints.add(4), 1 / 2)))
  }
  if (input.upgrade43 > 0.5) {
    s = s.times(Decimal.min(1e30, Decimal.pow(1.01, input.transcendCount)))
  }
  if (input.upgrade48 > 0.5) {
    s = s.times(Decimal.pow((input.totalMultiplier * input.totalAccelerator) / 1000 + 1, 8))
  }
  if (input.reincarnationChallenge === 6) s = s.dividedBy(1e250)
  if (input.reincarnationChallenge === 7) s = s.dividedBy('1e1250')
  if (input.reincarnationChallenge === 9) s = s.dividedBy('1e2000000')
  c = Decimal.pow(s, 1 + 0.001 * input.research17)
  let lol = Decimal.pow(c, 1 + 0.025 * input.upgrade123)
  if (input.ascensionChallenge === 15 && input.platonicUpgrade5 > 0) {
    lol = Decimal.pow(lol, 1.1)
  }
  if (input.ascensionChallenge === 15 && input.platonicUpgrade14 > 0) {
    lol = Decimal.pow(
      lol,
      1
        + ((1 / 20)
            * input.recessionCorruptionLevel
            * Decimal.log(input.coins.add(1), 10))
          / (1e7 + Decimal.log(input.coins.add(1), 10))
    )
  }
  if (input.ascensionChallenge === 15 && input.platonicUpgrade15 > 0) {
    lol = Decimal.pow(lol, 1.1)
  }
  lol = Decimal.pow(lol, input.challenge15CoinExponent)
  let globalCoinMultiplier = lol
  globalCoinMultiplier = Decimal.pow(globalCoinMultiplier, input.recessionPower)

  let coinOneMulti = new Decimal(1)
  if (input.upgrade1 > 0.5) coinOneMulti = coinOneMulti.times(first6CoinUp)
  if (input.upgrade10 > 0.5) {
    coinOneMulti = coinOneMulti.times(Decimal.pow(2, Math.min(50, input.secondOwnedCoin / 15)))
  }
  if (input.upgrade56 > 0.5) coinOneMulti = coinOneMulti.times('1e5000')

  let coinTwoMulti = new Decimal(1)
  if (input.upgrade2 > 0.5) coinTwoMulti = coinTwoMulti.times(first6CoinUp)
  if (input.upgrade13 > 0.5) {
    coinTwoMulti = coinTwoMulti.times(
      Decimal.min(
        1e50,
        Decimal.pow(input.firstGeneratedMythos.add(input.firstOwnedMythos).add(1), 4 / 3).times(1e22)
      )
    )
  }
  if (input.upgrade19 > 0.5) {
    coinTwoMulti = coinTwoMulti.times(Decimal.min(1e200, input.transcendPoints.times(1e30).add(1)))
  }
  if (input.upgrade57 > 0.5) coinTwoMulti = coinTwoMulti.times('1e7500')

  let coinThreeMulti = new Decimal(1)
  if (input.upgrade3 > 0.5) coinThreeMulti = coinThreeMulti.times(first6CoinUp)
  if (input.upgrade18 > 0.5) {
    coinThreeMulti = coinThreeMulti.times(Decimal.min(1e125, input.transcendShards.add(1)))
  }
  if (input.upgrade58 > 0.5) coinThreeMulti = coinThreeMulti.times('1e15000')

  let coinFourMulti = new Decimal(1)
  if (input.upgrade4 > 0.5) coinFourMulti = coinFourMulti.times(first6CoinUp)
  if (input.upgrade17 > 0.5) coinFourMulti = coinFourMulti.times(1e100)
  if (input.upgrade59 > 0.5) coinFourMulti = coinFourMulti.times('1e25000')

  let coinFiveMulti = new Decimal(1)
  if (input.upgrade5 > 0.5) coinFiveMulti = coinFiveMulti.times(first6CoinUp)
  if (input.upgrade60 > 0.5) coinFiveMulti = coinFiveMulti.times('1e35000')

  let globalCrystalMultiplier = new Decimal(1)
  globalCrystalMultiplier = globalCrystalMultiplier.times(input.crystalMultiplierAchievement)
  globalCrystalMultiplier = globalCrystalMultiplier.times(Decimal.pow(10, input.prismProductionLog10))
  if (input.upgrade36 > 0.5) {
    globalCrystalMultiplier = globalCrystalMultiplier.times(
      Decimal.min('1e5000', Decimal.pow(input.prestigePoints, 1 / 500))
    )
  }
  if (input.upgrade63 > 0.5) {
    globalCrystalMultiplier = globalCrystalMultiplier.times(
      Decimal.min('1e6000', Decimal.pow(input.reincarnationPoints.add(1), 6))
    )
  }
  if (input.research39 > 0.5) {
    globalCrystalMultiplier = globalCrystalMultiplier.times(
      Decimal.pow(input.buildingPowerMult, 1 / 50)
    )
  }
  globalCrystalMultiplier = globalCrystalMultiplier.times(
    Decimal.pow(1 + 0.01 * input.crystalUpgrade0, input.achievementPoints)
  )
  globalCrystalMultiplier = globalCrystalMultiplier.times(
    Decimal.pow(
      1 + input.crystalUpgrade1 * Decimal.log(input.coins.add(1), 10) / 100,
      2 + Math.log2(input.crystalUpgrade1 + 1)
    )
  )
  globalCrystalMultiplier = globalCrystalMultiplier.times(input.crystalUpgrade3Multiplier)
  globalCrystalMultiplier = globalCrystalMultiplier.times(
    Decimal.pow(
      1 + 0.05 * input.crystalUpgrade4,
      input.c1Completions
        + input.c2Completions
        + input.c3Completions
        + input.c4Completions
        + input.c5Completions
    )
  )
  globalCrystalMultiplier = globalCrystalMultiplier.times(
    Decimal.pow(10, CalcECC('transcend', input.c5Completions))
  )
  globalCrystalMultiplier = globalCrystalMultiplier.times(
    Decimal.pow(
      1e4,
      input.research5 * (1 + (1 / 2) * CalcECC('ascension', input.c14Completions))
    )
  )
  globalCrystalMultiplier = globalCrystalMultiplier.times(Decimal.pow(2.5, input.research26))
  globalCrystalMultiplier = globalCrystalMultiplier.times(Decimal.pow(2.5, input.research27))

  let globalMythosMultiplier = new Decimal(1)

  if (input.upgrade37 > 0.5) {
    globalMythosMultiplier = globalMythosMultiplier.times(
      Decimal.pow(Decimal.log(input.prestigePoints.add(10), 10), 2)
    )
  }
  if (input.upgrade42 > 0.5) {
    globalMythosMultiplier = globalMythosMultiplier.times(
      Decimal.min(
        1e50,
        Decimal.pow(input.prestigePoints.add(1), 1 / 50).dividedBy(2.5).add(1)
      )
    )
  }
  if (input.upgrade47 > 0.5) {
    globalMythosMultiplier = globalMythosMultiplier
      .times(Decimal.pow(1.01, input.achievementPoints))
      .times(input.achievementPoints / 5 + 1)
  }
  if (input.upgrade51 > 0.5) {
    globalMythosMultiplier = globalMythosMultiplier.times(Decimal.pow(input.totalAcceleratorBoost, 2))
  }
  if (input.upgrade52 > 0.5) {
    globalMythosMultiplier = globalMythosMultiplier.times(Decimal.pow(globalMythosMultiplier, 0.025))
  }
  if (input.upgrade64 > 0.5) {
    globalMythosMultiplier = globalMythosMultiplier.times(
      Decimal.pow(input.reincarnationPoints.add(1), 2)
    )
  }
  if (input.research40 > 0.5) {
    globalMythosMultiplier = globalMythosMultiplier.times(
      Decimal.pow(input.buildingPowerMult, 1 / 250)
    )
  }

  let grandmasterMultiplier = new Decimal(1)
  const totalMythosOwned = input.firstOwnedMythos
    + input.secondOwnedMythos
    + input.thirdOwnedMythos
    + input.fourthOwnedMythos
    + input.fifthOwnedMythos

  const mythosBuildingPower = 1 + CalcECC('transcend', input.c3Completions) / 200
  const challengeThreeMultiplier = Decimal.pow(mythosBuildingPower, totalMythosOwned)

  grandmasterMultiplier = grandmasterMultiplier.times(challengeThreeMultiplier)

  let mythosupgrade13 = new Decimal(1)
  let mythosupgrade14 = new Decimal(1)
  let mythosupgrade15 = new Decimal(1)
  if (input.upgrade53 === 1) {
    mythosupgrade13 = mythosupgrade13.times(
      Decimal.min('1e1250', Decimal.pow(input.acceleratorEffect, 1 / 125))
    )
  }
  if (input.upgrade54 === 1) {
    mythosupgrade14 = mythosupgrade14.times(
      Decimal.min('1e2000', Decimal.pow(input.multiplierEffect, 1 / 180))
    )
  }
  if (input.upgrade55 === 1) {
    mythosupgrade15 = mythosupgrade15.times(
      Decimal.pow('1e1000', Math.min(1000, input.buildingPower - 1))
    )
  }

  let globalConstantMult = new Decimal('1')
  globalConstantMult = globalConstantMult.times(
    Decimal.pow(
      1.05 + input.constUpgrade1BuffAchievement + 0.001 * input.platonicUpgrade18,
      input.constantUpgrade1
    )
  )
  globalConstantMult = globalConstantMult.times(
    Decimal.pow(
      1
        + 0.001
          * Math.min(
            100
              + 1000 * input.constUpgrade2BuffAchievement
              + 10 * input.constantEXMaxPercentIncrease
              + 1000 * (input.challenge15ExponentValue - 1)
              + 3 * input.platonicUpgrade18,
            input.constantUpgrade2
          ),
      input.ascendBuildingDRValue
    )
  )
  globalConstantMult = globalConstantMult.times(1 + (2 / 100) * input.research139)
  globalConstantMult = globalConstantMult.times(1 + (3 / 100) * input.research154)
  globalConstantMult = globalConstantMult.times(1 + (5 / 100) * input.research184)
  globalConstantMult = globalConstantMult.times(1 + (10 / 100) * input.research199)
  globalConstantMult = globalConstantMult.times(input.challenge15ConstantBonus)
  if (input.platonicUpgrade5 > 0) globalConstantMult = globalConstantMult.times(2)
  if (input.platonicUpgrade10 > 0) globalConstantMult = globalConstantMult.times(10)
  if (input.platonicUpgrade15 > 0) globalConstantMult = globalConstantMult.times(1e250)
  globalConstantMult = globalConstantMult.times(
    Decimal.pow(input.overfluxPowder + 1, 10 * input.platonicUpgrade16)
  )

  return {
    globalCoinMultiplier,
    coinOneMulti,
    coinTwoMulti,
    coinThreeMulti,
    coinFourMulti,
    coinFiveMulti,
    globalCrystalMultiplier,
    globalMythosMultiplier,
    grandmasterMultiplier,
    totalMythosOwned,
    mythosBuildingPower,
    challengeThreeMultiplier,
    mythosupgrade13,
    mythosupgrade14,
    mythosupgrade15,
    globalConstantMult,
    antMultiplier: input.antMultiplier
  }
}

const decimalEq = (a: Decimal, b: Decimal): boolean => a.eq(b)

const baseInput: GlobalMultipliersInput = {
  upgrade1: 0, upgrade2: 0, upgrade3: 0, upgrade4: 0, upgrade5: 0, upgrade6: 0,
  upgrade10: 0, upgrade12: 0, upgrade13: 0, upgrade17: 0, upgrade18: 0,
  upgrade19: 0, upgrade20: 0, upgrade41: 0, upgrade43: 0, upgrade48: 0,
  upgrade56: 0, upgrade57: 0, upgrade58: 0, upgrade59: 0, upgrade60: 0,
  upgrade36: 0, upgrade37: 0, upgrade42: 0, upgrade47: 0, upgrade51: 0,
  upgrade52: 0, upgrade53: 0, upgrade54: 0, upgrade55: 0, upgrade63: 0,
  upgrade64: 0, upgrade123: 0,
  research5: 0, research17: 0, research26: 0, research27: 0, research39: 0,
  research40: 0, research139: 0, research154: 0, research184: 0, research199: 0,
  crystalUpgrade0: 0, crystalUpgrade1: 0, crystalUpgrade4: 0,
  constantUpgrade1: 0, constantUpgrade2: 0,
  platonicUpgrade5: 0, platonicUpgrade10: 0, platonicUpgrade14: 0,
  platonicUpgrade15: 0, platonicUpgrade16: 0, platonicUpgrade18: 0,
  coins: new Decimal(1e20),
  prestigePoints: new Decimal(1e30),
  transcendPoints: new Decimal(1e100),
  reincarnationPoints: new Decimal(1e10),
  transcendShards: new Decimal(1e300),
  prestigeCount: 0,
  transcendCount: 0,
  highestSingularityCount: 0,
  goldenQuarks: 0,
  overfluxPowder: 0,
  secondOwnedCoin: 0,
  firstGeneratedMythos: new Decimal(0),
  firstOwnedMythos: 0,
  secondOwnedMythos: 0,
  thirdOwnedMythos: 0,
  fourthOwnedMythos: 0,
  fifthOwnedMythos: 0,
  c1Completions: 0,
  c2Completions: 0,
  c3Completions: 0,
  c4Completions: 0,
  c5Completions: 0,
  c14Completions: 0,
  reincarnationChallenge: 0,
  ascensionChallenge: 0,
  recessionCorruptionLevel: 0,
  crystalMult: new Decimal(1),
  buildingPower: 1,
  buildingPowerMult: new Decimal(1),
  totalCoinOwned: 0,
  antMultiplier: new Decimal(1),
  crystalUpgrade3Multiplier: new Decimal(1),
  achievementPoints: 0,
  crystalMultiplierAchievement: 1,
  constUpgrade1BuffAchievement: 0,
  constUpgrade2BuffAchievement: 0,
  prismProductionLog10: 0,
  constantEXMaxPercentIncrease: 0,
  ascendBuildingDRValue: 1,
  multiplierEffect: new Decimal(1),
  acceleratorEffect: new Decimal(1),
  totalMultiplier: 0,
  totalAccelerator: 0,
  totalAcceleratorBoost: 0,
  challenge15CoinExponent: 1,
  challenge15ExponentValue: 1,
  challenge15ConstantBonus: 1,
  recessionPower: 1
}

const cases: Array<{ name: string, input: GlobalMultipliersInput }> = [
  { name: 'baseline (all neutral)', input: baseInput },

  // ─── s multiplier branches ────────────────────────────────────────────
  {
    name: 'singularity s bonus (highestSingularityCount > 0)',
    input: { ...baseInput, highestSingularityCount: 50, goldenQuarks: 1e6 }
  },
  {
    name: 'upgrade 6 adds first6CoinUp',
    input: { ...baseInput, upgrade6: 1, totalCoinOwned: 100 }
  },
  {
    name: 'upgrade 12 with prestigeCount',
    input: { ...baseInput, upgrade12: 1, prestigeCount: 50 }
  },
  {
    name: 'upgrade 20 with totalCoinOwned',
    input: { ...baseInput, upgrade20: 1, totalCoinOwned: 1000 }
  },
  {
    name: 'upgrade 41/43 transcendPoints/Count',
    input: { ...baseInput, upgrade41: 1, upgrade43: 1, transcendCount: 100 }
  },
  {
    name: 'upgrade 48 with totalMultiplier × totalAccelerator',
    input: { ...baseInput, upgrade48: 1, totalMultiplier: 1000, totalAccelerator: 500 }
  },

  // ─── Reincarnation challenge divisors ─────────────────────────────────
  { name: 'r-chal 6 divides s by 1e250', input: { ...baseInput, reincarnationChallenge: 6 } },
  { name: 'r-chal 7 divides s by 1e1250', input: { ...baseInput, reincarnationChallenge: 7 } },
  { name: 'r-chal 9 divides s by 1e2000000', input: { ...baseInput, reincarnationChallenge: 9 } },

  // ─── research17 / upgrade123 / a-chal 15 platonics ───────────────────
  {
    name: 'research 17 applies exponent on s',
    input: { ...baseInput, research17: 50, upgrade6: 1, totalCoinOwned: 100 }
  },
  {
    name: 'upgrade 123 applies exponent on c',
    input: { ...baseInput, upgrade123: 10, upgrade6: 1, totalCoinOwned: 100 }
  },
  {
    name: 'a-chal 15 + platonic 5 → ^1.1',
    input: { ...baseInput, ascensionChallenge: 15, platonicUpgrade5: 1 }
  },
  {
    name: 'a-chal 15 + platonic 14 with recession + log coins',
    input: {
      ...baseInput,
      ascensionChallenge: 15,
      platonicUpgrade14: 1,
      recessionCorruptionLevel: 5,
      coins: new Decimal(1e50)
    }
  },
  {
    name: 'a-chal 15 + platonic 15 → ^1.1',
    input: { ...baseInput, ascensionChallenge: 15, platonicUpgrade15: 1 }
  },
  {
    name: 'challenge15CoinExponent applies to lol',
    input: { ...baseInput, challenge15CoinExponent: 1.05 }
  },
  {
    name: 'recessionPower applies to globalCoinMultiplier',
    input: { ...baseInput, recessionPower: 0.9 }
  },

  // ─── Coin tier multipliers (each gates) ───────────────────────────────
  {
    name: 'coinOneMulti: upgrades 1/10/56 all on',
    input: { ...baseInput, upgrade1: 1, upgrade10: 1, upgrade56: 1, secondOwnedCoin: 200, totalCoinOwned: 100 }
  },
  {
    name: 'coinTwoMulti: upgrades 2/13/19/57 all on',
    input: {
      ...baseInput,
      upgrade2: 1, upgrade13: 1, upgrade19: 1, upgrade57: 1,
      firstGeneratedMythos: new Decimal(1e5), firstOwnedMythos: 100, totalCoinOwned: 100
    }
  },
  {
    name: 'coinThreeMulti: upgrades 3/18/58 all on',
    input: { ...baseInput, upgrade3: 1, upgrade18: 1, upgrade58: 1, totalCoinOwned: 100 }
  },
  {
    name: 'coinFourMulti: upgrades 4/17/59 all on',
    input: { ...baseInput, upgrade4: 1, upgrade17: 1, upgrade59: 1, totalCoinOwned: 100 }
  },
  {
    name: 'coinFiveMulti: upgrades 5/60 all on',
    input: { ...baseInput, upgrade5: 1, upgrade60: 1, totalCoinOwned: 100 }
  },

  // ─── globalCrystalMultiplier contributions ────────────────────────────
  {
    name: 'crystal: achievement + prism + upgrade36/63 + research39',
    input: {
      ...baseInput,
      crystalMultiplierAchievement: 2.5,
      prismProductionLog10: 5,
      upgrade36: 1, upgrade63: 1, research39: 1,
      buildingPowerMult: new Decimal(1e100)
    }
  },
  {
    name: 'crystal: crystalUpgrade 0/1/4 + c1-c5 completions',
    input: {
      ...baseInput,
      crystalUpgrade0: 100, crystalUpgrade1: 50, crystalUpgrade4: 50,
      c1Completions: 25, c2Completions: 25, c3Completions: 25, c4Completions: 25, c5Completions: 25,
      achievementPoints: 100,
      crystalUpgrade3Multiplier: new Decimal(1e10)
    }
  },
  {
    name: 'crystal: research5 with c14 ECC + research26/27',
    input: { ...baseInput, research5: 10, c14Completions: 25, research26: 5, research27: 5 }
  },

  // ─── globalMythosMultiplier contributions ─────────────────────────────
  {
    name: 'mythos: upgrades 37/42/47/51/52/64 + research 40',
    input: {
      ...baseInput,
      upgrade37: 1, upgrade42: 1, upgrade47: 1, upgrade51: 1, upgrade52: 1, upgrade64: 1, research40: 1,
      achievementPoints: 100,
      totalAcceleratorBoost: 50,
      buildingPowerMult: new Decimal(1e50)
    }
  },

  // ─── grandmaster + challengeThreeMultiplier ───────────────────────────
  {
    name: 'totalMythosOwned + c3 ECC scales challengeThreeMultiplier',
    input: {
      ...baseInput,
      firstOwnedMythos: 50, secondOwnedMythos: 50, thirdOwnedMythos: 50,
      fourthOwnedMythos: 50, fifthOwnedMythos: 50,
      c3Completions: 25
    }
  },

  // ─── mythosupgrade13/14/15 ────────────────────────────────────────────
  {
    name: 'mythosupgrade13/14/15 all active',
    input: {
      ...baseInput,
      upgrade53: 1, upgrade54: 1, upgrade55: 1,
      acceleratorEffect: new Decimal(1e100),
      multiplierEffect: new Decimal(1e150),
      buildingPower: 50
    }
  },

  // ─── globalConstantMult contributions ─────────────────────────────────
  {
    name: 'constant: upgrade 1 base exponent with platonic18 + achievement',
    input: {
      ...baseInput,
      constantUpgrade1: 100,
      platonicUpgrade18: 10,
      constUpgrade1BuffAchievement: 0.05
    }
  },
  {
    name: 'constant: upgrade 2 with bounded percent + ascendBuildingDR',
    input: {
      ...baseInput,
      constantUpgrade2: 200,
      constUpgrade2BuffAchievement: 0.05,
      constantEXMaxPercentIncrease: 5,
      challenge15ExponentValue: 1.05,
      platonicUpgrade18: 5,
      ascendBuildingDRValue: 0.5
    }
  },
  {
    name: 'constant: researches 139/154/184/199 + challenge15ConstantBonus',
    input: {
      ...baseInput,
      research139: 10, research154: 10, research184: 10, research199: 10,
      challenge15ConstantBonus: 1.5
    }
  },
  {
    name: 'constant: platonic 5/10/15 multiply by 2/10/1e250',
    input: { ...baseInput, platonicUpgrade5: 1, platonicUpgrade10: 1, platonicUpgrade15: 1 }
  },
  {
    name: 'constant: platonic16 with overfluxPowder',
    input: { ...baseInput, platonicUpgrade16: 5, overfluxPowder: 100 }
  },

  // ─── antMultiplier passthrough ────────────────────────────────────────
  {
    name: 'antMultiplier passthrough surfaces unchanged',
    input: { ...baseInput, antMultiplier: new Decimal(1e25) }
  },

  // ─── Big combined stack ───────────────────────────────────────────────
  {
    name: 'big stack: many flags + late-game state',
    input: {
      ...baseInput,
      upgrade1: 1, upgrade2: 1, upgrade3: 1, upgrade4: 1, upgrade5: 1,
      upgrade6: 1, upgrade10: 1, upgrade12: 1, upgrade13: 1, upgrade17: 1,
      upgrade18: 1, upgrade19: 1, upgrade20: 1, upgrade41: 1, upgrade43: 1,
      upgrade48: 1, upgrade36: 1, upgrade37: 1, upgrade42: 1, upgrade47: 1,
      upgrade51: 1, upgrade53: 1, upgrade54: 1, upgrade55: 1, upgrade63: 1,
      upgrade64: 1, upgrade123: 5,
      research5: 5, research17: 25, research26: 3, research27: 3, research39: 1,
      research40: 1, research139: 5, research154: 5, research184: 5, research199: 5,
      crystalUpgrade0: 50, crystalUpgrade1: 25, crystalUpgrade4: 25,
      constantUpgrade1: 50, constantUpgrade2: 100,
      platonicUpgrade5: 1, platonicUpgrade14: 1, platonicUpgrade16: 3, platonicUpgrade18: 5,
      coins: new Decimal('1e1000'),
      prestigePoints: new Decimal('1e500'),
      transcendPoints: new Decimal('1e1000'),
      reincarnationPoints: new Decimal('1e500'),
      transcendShards: new Decimal('1e1000'),
      prestigeCount: 100,
      transcendCount: 100,
      highestSingularityCount: 100,
      goldenQuarks: 1e10,
      overfluxPowder: 1000,
      secondOwnedCoin: 1000,
      firstGeneratedMythos: new Decimal(1e10),
      firstOwnedMythos: 500, secondOwnedMythos: 500, thirdOwnedMythos: 500,
      fourthOwnedMythos: 500, fifthOwnedMythos: 500,
      c1Completions: 25, c2Completions: 25, c3Completions: 25, c4Completions: 25, c5Completions: 25,
      c14Completions: 25,
      ascensionChallenge: 15,
      recessionCorruptionLevel: 8,
      crystalMult: new Decimal(1e50),
      buildingPower: 100,
      buildingPowerMult: new Decimal(1e80),
      totalCoinOwned: 5000,
      antMultiplier: new Decimal(1e30),
      crystalUpgrade3Multiplier: new Decimal(1e20),
      achievementPoints: 250,
      crystalMultiplierAchievement: 4,
      constUpgrade1BuffAchievement: 0.05,
      constUpgrade2BuffAchievement: 0.02,
      prismProductionLog10: 10,
      constantEXMaxPercentIncrease: 5,
      ascendBuildingDRValue: 0.7,
      multiplierEffect: new Decimal(1e200),
      acceleratorEffect: new Decimal(1e150),
      totalMultiplier: 5000,
      totalAccelerator: 10000,
      totalAcceleratorBoost: 500,
      challenge15CoinExponent: 1.1,
      challenge15ExponentValue: 1.05,
      challenge15ConstantBonus: 1.5,
      recessionPower: 0.92
    }
  }
]

describe('parity: computeGlobalMultipliers', () => {
  for (const c of cases) {
    it(c.name, () => {
      const newRes = newComputeGlobalMultipliers(c.input)
      const oldRes = oldComputeGlobalMultipliers(c.input)
      expect(decimalEq(newRes.globalCoinMultiplier, oldRes.globalCoinMultiplier)).toBe(true)
      expect(decimalEq(newRes.coinOneMulti, oldRes.coinOneMulti)).toBe(true)
      expect(decimalEq(newRes.coinTwoMulti, oldRes.coinTwoMulti)).toBe(true)
      expect(decimalEq(newRes.coinThreeMulti, oldRes.coinThreeMulti)).toBe(true)
      expect(decimalEq(newRes.coinFourMulti, oldRes.coinFourMulti)).toBe(true)
      expect(decimalEq(newRes.coinFiveMulti, oldRes.coinFiveMulti)).toBe(true)
      expect(decimalEq(newRes.globalCrystalMultiplier, oldRes.globalCrystalMultiplier)).toBe(true)
      expect(decimalEq(newRes.globalMythosMultiplier, oldRes.globalMythosMultiplier)).toBe(true)
      expect(decimalEq(newRes.grandmasterMultiplier, oldRes.grandmasterMultiplier)).toBe(true)
      expect(newRes.totalMythosOwned).toBe(oldRes.totalMythosOwned)
      expect(newRes.mythosBuildingPower).toBe(oldRes.mythosBuildingPower)
      expect(decimalEq(newRes.challengeThreeMultiplier, oldRes.challengeThreeMultiplier)).toBe(true)
      expect(decimalEq(newRes.mythosupgrade13, oldRes.mythosupgrade13)).toBe(true)
      expect(decimalEq(newRes.mythosupgrade14, oldRes.mythosupgrade14)).toBe(true)
      expect(decimalEq(newRes.mythosupgrade15, oldRes.mythosupgrade15)).toBe(true)
      expect(decimalEq(newRes.globalConstantMult, oldRes.globalConstantMult)).toBe(true)
      expect(decimalEq(newRes.antMultiplier, oldRes.antMultiplier)).toBe(true)
    })
  }
})
