// Per-tick global multiplier aggregator. Lifted from
// packages/web_ui/src/Synergism.ts (multipliers).
//
// Populates the 17 G.* multiplier fields the rest of the tick reads.
// Largest single migration of Phase 2 — composes ~30 pre-evaluated calls plus
// 50+ player.* / G.* reads. Web_ui pre-evaluates everything; this stays a
// pure (state, inputs) → result function.

import { Decimal } from '../math/bignum'
import { CalcECC } from './challenges'

export interface GlobalMultipliersInput {
  // ─── Direct player state — upgrades 1-60 (coin tier perks) ────────────

  /** player.upgrades[1] — gates coinOneMulti × first6CoinUp. */
  upgrade1: number
  /** player.upgrades[2] — gates coinTwoMulti × first6CoinUp. */
  upgrade2: number
  /** player.upgrades[3] — gates coinThreeMulti × first6CoinUp. */
  upgrade3: number
  /** player.upgrades[4] — gates coinFourMulti × first6CoinUp. */
  upgrade4: number
  /** player.upgrades[5] — gates coinFiveMulti × first6CoinUp. */
  upgrade5: number
  /** player.upgrades[6] — adds first6CoinUp to the main s multiplier. */
  upgrade6: number
  /** player.upgrades[10] — coinOneMulti × `2 ^ min(50, secondOwnedCoin/15)`. */
  upgrade10: number
  /** player.upgrades[12] — s × min(1e4, 1.01^prestigeCount). */
  upgrade12: number
  /** player.upgrades[13] — coinTwoMulti × min(1e50, (firstGenMythos+firstOwnMythos+1)^(4/3) * 1e22). */
  upgrade13: number
  /** player.upgrades[17] — coinFourMulti × 1e100. */
  upgrade17: number
  /** player.upgrades[18] — coinThreeMulti × min(1e125, transcendShards+1). */
  upgrade18: number
  /** player.upgrades[19] — coinTwoMulti × min(1e200, transcendPoints*1e30 + 1). */
  upgrade19: number
  /** player.upgrades[20] — s × (totalCoinOwned/4 + 1)^10. */
  upgrade20: number
  /** player.upgrades[41] — s × min(1e30, (transcendPoints+4)^0.5). */
  upgrade41: number
  /** player.upgrades[43] — s × min(1e30, 1.01^transcendCount). */
  upgrade43: number
  /** player.upgrades[48] — s × ((totalMultiplier*totalAccelerator)/1000 + 1)^8. */
  upgrade48: number
  /** player.upgrades[56] — coinOneMulti × 1e5000. */
  upgrade56: number
  /** player.upgrades[57] — coinTwoMulti × 1e7500. */
  upgrade57: number
  /** player.upgrades[58] — coinThreeMulti × 1e15000. */
  upgrade58: number
  /** player.upgrades[59] — coinFourMulti × 1e25000. */
  upgrade59: number
  /** player.upgrades[60] — coinFiveMulti × 1e35000. */
  upgrade60: number

  // ─── upgrades 36-67 (crystal / mythos tier) ───────────────────────────

  /** player.upgrades[36] — globalCrystalMultiplier × min(1e5000, prestigePoints^(1/500)). */
  upgrade36: number
  /** player.upgrades[37] — globalMythosMultiplier × log10(prestigePoints+10)^2. */
  upgrade37: number
  /** player.upgrades[42] — globalMythosMultiplier × min(1e50, (prestigePoints+1)^(1/50)/2.5 + 1). */
  upgrade42: number
  /** player.upgrades[47] — globalMythosMultiplier × 1.01^aP × (aP/5 + 1). */
  upgrade47: number
  /** player.upgrades[51] — globalMythosMultiplier × totalAcceleratorBoost^2. */
  upgrade51: number
  /** player.upgrades[52] — globalMythosMultiplier × itself^0.025 (idempotent-ish). */
  upgrade52: number
  /** player.upgrades[53] — mythosupgrade13 × min(1e1250, acceleratorEffect^(1/125)). */
  upgrade53: number
  /** player.upgrades[54] — mythosupgrade14 × min(1e2000, multiplierEffect^(1/180)). */
  upgrade54: number
  /** player.upgrades[55] — mythosupgrade15 × 1e1000 ^ min(1000, buildingPower-1). */
  upgrade55: number
  /** player.upgrades[63] — globalCrystalMultiplier × min(1e6000, (reincarnationPoints+1)^6). */
  upgrade63: number
  /** player.upgrades[64] — globalMythosMultiplier × (reincarnationPoints+1)^2. */
  upgrade64: number
  /** player.upgrades[123] — additional `1 + 0.025n` exponent on c. */
  upgrade123: number

  // ─── Researches ───────────────────────────────────────────────────────

  /** player.researches[5] — globalCrystalMultiplier × 1e4 ^ (n * (1 + ECC(asc, c14)/2)). */
  research5: number
  /** player.researches[17] — `1 + 0.001n` exponent on s for c. */
  research17: number
  /** player.researches[26] — globalCrystalMultiplier × 2.5^n. */
  research26: number
  /** player.researches[27] — globalCrystalMultiplier × 2.5^n. */
  research27: number
  /** player.researches[39] — globalCrystalMultiplier × buildingPowerMult^(1/50). */
  research39: number
  /** player.researches[40] — globalMythosMultiplier × buildingPowerMult^(1/250). */
  research40: number
  /** player.researches[139] — globalConstantMult × (1 + 0.02n). */
  research139: number
  /** player.researches[154] — globalConstantMult × (1 + 0.03n). */
  research154: number
  /** player.researches[184] — globalConstantMult × (1 + 0.05n). */
  research184: number
  /** player.researches[199] — globalConstantMult × (1 + 0.10n). */
  research199: number

  // ─── Crystal / constant / platonic upgrades ───────────────────────────

  /** player.crystalUpgrades[0] — exponent base for the achievementPoints-fueled bonus. */
  crystalUpgrade0: number
  /** player.crystalUpgrades[1] — feeds the log10(coins+1) bonus + its log2 exponent. */
  crystalUpgrade1: number
  /** player.crystalUpgrades[4] — exponent base for the c1-c5 completion sum bonus. */
  crystalUpgrade4: number
  /** player.constantUpgrades[1] — globalConstantMult ^ `1.05 + constUpgrade1Buff + 0.001*plat18`. */
  constantUpgrade1: number
  /** player.constantUpgrades[2] — bounded percentage bump fed into globalConstantMult. */
  constantUpgrade2: number
  /** player.platonicUpgrades[5] — × 2 when > 0. */
  platonicUpgrade5: number
  /** player.platonicUpgrades[10] — × 10 when > 0. */
  platonicUpgrade10: number
  /** player.platonicUpgrades[14] — a-chal 15 reincarnation-corrupted log10-coin exponent term. */
  platonicUpgrade14: number
  /** player.platonicUpgrades[15] — a-chal 15: ^1.1 on lol + globalConstantMult ×1e250 when > 0. */
  platonicUpgrade15: number
  /** player.platonicUpgrades[16] — `overfluxPowder+1 ^ 10*plat16` multiplied into globalConstantMult. */
  platonicUpgrade16: number
  /** player.platonicUpgrades[18] — adds to constUpgrade1 base + bounded contribution to constUpgrade2 percent. */
  platonicUpgrade18: number

  // ─── Resources / counters ─────────────────────────────────────────────

  /** player.coins — log10 base for crystalUpgrade1 bonus + plat14 r-corruption term. */
  coins: Decimal
  /** player.prestigePoints — feeds upgrade-36, upgrade-37, upgrade-42 multipliers. */
  prestigePoints: Decimal
  /** player.transcendPoints — feeds upgrade-19, upgrade-41 multipliers. */
  transcendPoints: Decimal
  /** player.reincarnationPoints — feeds upgrade-63, upgrade-64 multipliers. */
  reincarnationPoints: Decimal
  /** player.transcendShards — feeds upgrade-18 coinThree multiplier. */
  transcendShards: Decimal
  /** player.prestigeCount — exponent for upgrade-12. */
  prestigeCount: number
  /** player.transcendCount — exponent for upgrade-43. */
  transcendCount: number
  /** player.highestSingularityCount — gates the singularity s multiplier when > 0. */
  highestSingularityCount: number
  /** player.goldenQuarks — base for `(goldenQuarks+1)^1.5` singularity bonus. */
  goldenQuarks: number
  /** player.overfluxPowder — base for plat16 globalConstantMult bonus. */
  overfluxPowder: number

  // ─── Coin tier owned counts (for derived totals) ──────────────────────

  /** player.secondOwnedCoin — used in upgrade-10 (`2 ^ min(50, n/15)`). */
  secondOwnedCoin: number
  /** player.firstGeneratedMythos — feeds upgrade-13. */
  firstGeneratedMythos: Decimal
  /** player.firstOwnedMythos — feeds upgrade-13 + globalMythosOwned. */
  firstOwnedMythos: number
  /** player.secondOwnedMythos — feeds totalMythosOwned. */
  secondOwnedMythos: number
  /** player.thirdOwnedMythos — feeds totalMythosOwned. */
  thirdOwnedMythos: number
  /** player.fourthOwnedMythos — feeds totalMythosOwned. */
  fourthOwnedMythos: number
  /** player.fifthOwnedMythos — feeds totalMythosOwned. */
  fifthOwnedMythos: number

  // ─── Challenge state ──────────────────────────────────────────────────

  /** player.challengecompletions[1..5] — sum feeds crystalUpgrade4 exponent. */
  c1Completions: number
  c2Completions: number
  c3Completions: number
  c4Completions: number
  c5Completions: number
  /** player.challengecompletions[14] — fed through CalcECC for ecc14a. */
  c14Completions: number
  /** player.currentChallenge.reincarnation — r-chal 6/7/9 each divide s by a constant. */
  reincarnationChallenge: number
  /** player.currentChallenge.ascension — gates platonicUpgrade 5/14/15 lol exponent terms. */
  ascensionChallenge: number
  /** player.corruptions.used.recession — feeds recessionPower lookup + plat14 exponent. */
  recessionCorruptionLevel: number

  // ─── Pre-evaluated values (already-migrated callers) ──────────────────

  /** `calculateCrystalCoinMultiplier()` — multiplied into s. */
  crystalMult: Decimal
  /** `calculateBuildingPower()` — used in upgrade-55 `min(1000, buildingPower-1)`. */
  buildingPower: number
  /** `calculateBuildingPowerCoinMultiplier(buildingPower)` — multiplied into s, also feeds research-39/40. */
  buildingPowerMult: Decimal
  /** `calculateTotalCoinOwned()` — used in first6CoinUp + upgrade-20. */
  totalCoinOwned: number
  /** `getAntUpgradeEffect(AntUpgrades.Coins).coinMultiplier` — multiplied into s, also surfaced as antMultiplier output. */
  antMultiplier: Decimal
  /** `crystalUpgrade3CrystalMultiplier()` — multiplied into globalCrystalMultiplier. */
  crystalUpgrade3Multiplier: Decimal
  /** Module-level `achievementPoints` from Achievements.ts — feeds crystalUpgrade0 exponent + upgrade-47. */
  achievementPoints: number
  /** `+getAchievementReward('crystalMultiplier')` — multiplied into globalCrystalMultiplier. */
  crystalMultiplierAchievement: number
  /** `+getAchievementReward('constUpgrade1Buff')` — added to constUpgrade1 exponent base. */
  constUpgrade1BuffAchievement: number
  /** `+getAchievementReward('constUpgrade2Buff')` — bounded coefficient inside constUpgrade2. */
  constUpgrade2BuffAchievement: number
  /** `getRuneEffects('prism', 'productionLog10')` — exponent of 10 multiplied into globalCrystalMultiplier. */
  prismProductionLog10: number
  /** `getShopUpgradeEffects('constantEX', 'maxPercentIncrease')` — added into constUpgrade2 bounded percentage. */
  constantEXMaxPercentIncrease: number
  /** `ascendBuildingDR()` — exponent applied to constUpgrade2 contribution. */
  ascendBuildingDRValue: number

  // ─── G inputs (pre-extracted by web_ui) ───────────────────────────────

  /** G.multiplierEffect — multiplied into s (set by updateAllMultiplier earlier this tick). */
  multiplierEffect: Decimal
  /** G.acceleratorEffect — multiplied into s (set by updateAllTick earlier this tick) + mythosupgrade13. */
  acceleratorEffect: Decimal
  /** G.totalMultiplier — feeds upgrade-48 (×totalAccelerator/1000+1). */
  totalMultiplier: number
  /** G.totalAccelerator — feeds upgrade-48. */
  totalAccelerator: number
  /** G.totalAcceleratorBoost — exponent base for upgrade-51. */
  totalAcceleratorBoost: number
  /** G.challenge15Rewards.coinExponent.value — exponent of lol → globalCoinMultiplier. */
  challenge15CoinExponent: number
  /** G.challenge15Rewards.exponent.value — `(exponent-1)*1000` is bounded into constUpgrade2 percent. */
  challenge15ExponentValue: number
  /** G.challenge15Rewards.constantBonus.value — multiplied into globalConstantMult. */
  challenge15ConstantBonus: number
  /** G.recessionPower[player.corruptions.used.recession] — exponent applied to globalCoinMultiplier. */
  recessionPower: number
}

export interface GlobalMultipliersResult {
  globalCoinMultiplier: Decimal
  coinOneMulti: Decimal
  coinTwoMulti: Decimal
  coinThreeMulti: Decimal
  coinFourMulti: Decimal
  coinFiveMulti: Decimal
  globalCrystalMultiplier: Decimal
  globalMythosMultiplier: Decimal
  grandmasterMultiplier: Decimal
  totalMythosOwned: number
  mythosBuildingPower: number
  challengeThreeMultiplier: Decimal
  mythosupgrade13: Decimal
  mythosupgrade14: Decimal
  mythosupgrade15: Decimal
  globalConstantMult: Decimal
  /** Surfaced for parity even though it equals input.antMultiplier — legacy
   * sets G.antMultiplier inside multipliers() so the shim must too. */
  antMultiplier: Decimal
}

/**
 * Per-tick global-multiplier aggregator. Direct transcription of the legacy
 * multipliers() body with input/output substitution; preserves the exact
 * computation order so parity holds across every code path.
 */
export function computeGlobalMultipliers (input: GlobalMultipliersInput): GlobalMultipliersResult {
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
  if (input.upgrade6 > 0.5) {
    s = s.times(first6CoinUp)
  }
  if (input.upgrade12 > 0.5) {
    s = s.times(Decimal.min(1e4, Decimal.pow(1.01, input.prestigeCount)))
  }
  if (input.upgrade20 > 0.5) {
    s = s.times(Decimal.pow(input.totalCoinOwned / 4 + 1, 10))
  }
  if (input.upgrade41 > 0.5) {
    s = s.times(Decimal.min(1e30, Decimal.pow(input.transcendPoints.add(4), 1 / 2)))
  }
  if (input.upgrade43 > 0.5) {
    s = s.times(Decimal.min(1e30, Decimal.pow(1.01, input.transcendCount)))
  }
  if (input.upgrade48 > 0.5) {
    s = s.times(
      Decimal.pow((input.totalMultiplier * input.totalAccelerator) / 1000 + 1, 8)
    )
  }
  if (input.reincarnationChallenge === 6) {
    s = s.dividedBy(1e250)
  }
  if (input.reincarnationChallenge === 7) {
    s = s.dividedBy('1e1250')
  }
  if (input.reincarnationChallenge === 9) {
    s = s.dividedBy('1e2000000')
  }
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
  if (input.upgrade1 > 0.5) {
    coinOneMulti = coinOneMulti.times(first6CoinUp)
  }
  if (input.upgrade10 > 0.5) {
    coinOneMulti = coinOneMulti.times(
      Decimal.pow(2, Math.min(50, input.secondOwnedCoin / 15))
    )
  }
  if (input.upgrade56 > 0.5) {
    coinOneMulti = coinOneMulti.times('1e5000')
  }

  let coinTwoMulti = new Decimal(1)
  if (input.upgrade2 > 0.5) {
    coinTwoMulti = coinTwoMulti.times(first6CoinUp)
  }
  if (input.upgrade13 > 0.5) {
    coinTwoMulti = coinTwoMulti.times(
      Decimal.min(
        1e50,
        Decimal.pow(input.firstGeneratedMythos.add(input.firstOwnedMythos).add(1), 4 / 3)
          .times(1e22)
      )
    )
  }
  if (input.upgrade19 > 0.5) {
    coinTwoMulti = coinTwoMulti.times(
      Decimal.min(1e200, input.transcendPoints.times(1e30).add(1))
    )
  }
  if (input.upgrade57 > 0.5) {
    coinTwoMulti = coinTwoMulti.times('1e7500')
  }

  let coinThreeMulti = new Decimal(1)
  if (input.upgrade3 > 0.5) {
    coinThreeMulti = coinThreeMulti.times(first6CoinUp)
  }
  if (input.upgrade18 > 0.5) {
    coinThreeMulti = coinThreeMulti.times(
      Decimal.min(1e125, input.transcendShards.add(1))
    )
  }
  if (input.upgrade58 > 0.5) {
    coinThreeMulti = coinThreeMulti.times('1e15000')
  }

  let coinFourMulti = new Decimal(1)
  if (input.upgrade4 > 0.5) {
    coinFourMulti = coinFourMulti.times(first6CoinUp)
  }
  if (input.upgrade17 > 0.5) {
    coinFourMulti = coinFourMulti.times(1e100)
  }
  if (input.upgrade59 > 0.5) {
    coinFourMulti = coinFourMulti.times('1e25000')
  }

  let coinFiveMulti = new Decimal(1)
  if (input.upgrade5 > 0.5) {
    coinFiveMulti = coinFiveMulti.times(first6CoinUp)
  }
  if (input.upgrade60 > 0.5) {
    coinFiveMulti = coinFiveMulti.times('1e35000')
  }

  let globalCrystalMultiplier = new Decimal(1)
  globalCrystalMultiplier = globalCrystalMultiplier.times(input.crystalMultiplierAchievement)
  globalCrystalMultiplier = globalCrystalMultiplier.times(
    Decimal.pow(10, input.prismProductionLog10)
  )
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
    globalMythosMultiplier = globalMythosMultiplier.times(
      Decimal.pow(input.totalAcceleratorBoost, 2)
    )
  }
  if (input.upgrade52 > 0.5) {
    globalMythosMultiplier = globalMythosMultiplier.times(
      Decimal.pow(globalMythosMultiplier, 0.025)
    )
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
      1.05
        + input.constUpgrade1BuffAchievement
        + 0.001 * input.platonicUpgrade18,
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
  if (input.platonicUpgrade5 > 0) {
    globalConstantMult = globalConstantMult.times(2)
  }
  if (input.platonicUpgrade10 > 0) {
    globalConstantMult = globalConstantMult.times(10)
  }
  if (input.platonicUpgrade15 > 0) {
    globalConstantMult = globalConstantMult.times(1e250)
  }
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
