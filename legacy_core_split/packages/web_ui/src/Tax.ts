import {
  calculateCoinProduction as logicCalculateCoinProduction,
  calculateTax as logicCalculateTax
} from '@synergism/logic'
import { calculateBuildingPowerCoinMultiplier, player } from './Synergism'
import { sumContents } from './Utility'
import { Globals as G } from './Variables'

import { awardUngroupedAchievement, getAchievementReward } from './Achievements'
import { getAntUpgradeEffect } from './Features/Ants/AntUpgrades/lib/upgrade-effects'
import { AntUpgrades } from './Features/Ants/AntUpgrades/structs/structs'
import { calculateTaxPlatonicBlessing } from './PlatonicCubes'
import { getRuneEffects } from './Runes'
import { getTalismanEffects } from './Talismans'

// Thin shim over @synergism/logic. Sources every coin-tier / globalCoinMulti
// input from player and G, calls into logic for the production aggregation
// AND the tax exponent / divisor formula, then writes the results back to G
// and fires the overtaxed achievement if the returned flag says to.
export const calculatetax = () => {
  // Per-tier coin production — pure aggregation over five tiers + clamping.
  const production = logicCalculateCoinProduction({
    first: {
      generated: player.firstGeneratedCoin,
      owned: player.firstOwnedCoin,
      coinMulti: G.coinOneMulti,
      produceCoin: player.firstProduceCoin
    },
    second: {
      generated: player.secondGeneratedCoin,
      owned: player.secondOwnedCoin,
      coinMulti: G.coinTwoMulti,
      produceCoin: player.secondProduceCoin
    },
    third: {
      generated: player.thirdGeneratedCoin,
      owned: player.thirdOwnedCoin,
      coinMulti: G.coinThreeMulti,
      produceCoin: player.thirdProduceCoin
    },
    fourth: {
      generated: player.fourthGeneratedCoin,
      owned: player.fourthOwnedCoin,
      coinMulti: G.coinFourMulti,
      produceCoin: player.fourthProduceCoin
    },
    fifth: {
      generated: player.fifthGeneratedCoin,
      owned: player.fifthOwnedCoin,
      coinMulti: G.coinFiveMulti,
      produceCoin: player.fifthProduceCoin
    },
    globalCoinMultiplier: G.globalCoinMultiplier
  })

  G.produceFirst = production.first
  G.produceSecond = production.second
  G.produceThird = production.third
  G.produceFourth = production.fourth
  G.produceFifth = production.fifth
  G.produceTotal = production.total
  G.producePerSecond = production.perSecond

  // Tax exponent / divisor / overtaxed-achievement flag.
  const tax = logicCalculateTax({
    inReinc6: player.currentChallenge.reincarnation === 6,
    inReinc9: player.currentChallenge.reincarnation === 9,
    inAscension15: player.currentChallenge.ascension === 15,
    inAscension13: player.currentChallenge.ascension === 13,
    c6Completions: player.challengecompletions[6],
    c13Completions: player.challengecompletions[13],

    totalChallengeCompletions: sumContents(player.challengecompletions),
    c11Completions: player.challengecompletions[11],
    c12Completions: player.challengecompletions[12],
    c14Completions: player.challengecompletions[14],
    c15Completions: player.challengecompletions[15],
    singularityCount: player.singularityCount,

    research51: player.researches[51],
    research52: player.researches[52],
    research53: player.researches[53],
    research54: player.researches[54],
    research55: player.researches[55],
    research159: player.researches[159],
    research200: player.researches[200],
    cubeUpgrade50: player.cubeUpgrades[50],
    platonicUpgrade5: player.platonicUpgrades[5],
    platonicUpgrade10: player.platonicUpgrades[10],
    taxPlatonicBlessing: calculateTaxPlatonicBlessing(),
    upgrade121: player.upgrades[121],
    upgrade125: player.upgrades[125],
    c10Completions: player.challengecompletions[10],

    highestSingularityCount: player.highestSingularityCount,
    taxmanLastStandEnabled: player.singularityChallenges.taxmanLastStand.enabled,
    ascensionsUnlocked: player.unlocks.ascensions,
    highestC14Completions: player.highestchallengecompletions[14],

    taxReductionAchievement: +getAchievementReward('taxReduction'),
    duplicationRuneTaxReduction: getRuneEffects('duplication', 'taxReduction'),
    thriftRuneTaxReduction: getRuneEffects('thrift', 'taxReduction'),
    antTaxReduction: getAntUpgradeEffect(AntUpgrades.Taxes).taxReduction,
    exemptionTalismanTaxReduction: getTalismanEffects('exemption').taxReduction,
    challenge15TaxesReward: G.challenge15Rewards.taxes.value,
    campaignTaxMultiplier: player.campaigns.taxMultiplier,

    ascendShards: player.ascendShards,
    rareFragments: player.rareFragments,
    fortunaeFormicidaeCoinMultiplier: getAntUpgradeEffect(AntUpgrades.Coins).coinMultiplier,
    buildingPowerCoinMultiplier: calculateBuildingPowerCoinMultiplier(),

    produceTotal: production.total
  })

  G.maxexponent = tax.maxexponent
  G.taxdivisor = tax.taxdivisor
  G.taxdivisorcheck = tax.taxdivisorcheck

  // Side-effect: overtaxed achievement — logic returns the gate condition,
  // we fire the achievement call here so logic stays free of UI hooks.
  if (tax.shouldAwardOvertaxed) {
    awardUngroupedAchievement('overtaxed')
  }
}
