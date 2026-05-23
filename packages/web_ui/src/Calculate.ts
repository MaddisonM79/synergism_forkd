import {
  CalcCorruptionStuff as logicCalcCorruptionStuff,
  calculateAcceleratorMultiplier as logicCalcAcceleratorMultiplier,
  calculateActualAntSpeedMult as logicCalcActualAntSpeedMult,
  calculateAllCubeMultiplier as logicCalcAllCubeMultiplier,
  calculateAmbrosiaAdditiveLuckMult as logicCalcAmbrosiaAdditiveLuckMult,
  calculateAscensionScore as logicCalcAscensionScore,
  calculateAmbrosiaCubeMult as logicCalcAmbrosiaCubeMult,
  calculateAmbrosiaGenerationOcteractUpgrade as logicCalcAmbrosiaGenerationOcteractUpgrade,
  calculateAmbrosiaGenerationSingularityUpgrade as logicCalcAmbrosiaGenerationSingularityUpgrade,
  calculateAmbrosiaGenerationSpeed as logicCalcAmbrosiaGenerationSpeed,
  calculateAmbrosiaGenerationSpeedRaw as logicCalcAmbrosiaGenerationSpeedRaw,
  calculateAmbrosiaLuck as logicCalcAmbrosiaLuck,
  calculateAmbrosiaLuckOcteractUpgrade as logicCalcAmbrosiaLuckOcteractUpgrade,
  calculateAmbrosiaLuckRaw as logicCalcAmbrosiaLuckRaw,
  calculateAmbrosiaLuckSingularityUpgrade as logicCalcAmbrosiaLuckSingularityUpgrade,
  calculateAmbrosiaQuarkMult as logicCalcAmbrosiaQuarkMult,
  calculateAntSacrificeMultiplier as logicCalcAntSacrificeMultiplier,
  calculateAscensionCount as logicCalcAscensionCount,
  calculateAscensionSpeedExponentSpread as logicCalcAscensionSpeedExponentSpread,
  calculateAscensionSpeedMult as logicCalcAscensionSpeedMult,
  calculateBaseGoldenQuarks as logicCalcBaseGoldenQuarks,
  calculateBaseObtainium as logicCalcBaseObtainium,
  calculateBaseOfferings as logicCalcBaseOfferings,
  calculateBlueberryInventory as logicCalcBlueberryInventory,
  calculateCookieUpgrade29Luck as logicCalcCookieUpgrade29Luck,
  calculateCubeMultFromPowder as logicCalcCubeMultFromPowder,
  calculateCubeQuarkMultiplier as logicCalcCubeQuarkMultiplier,
  calculateCubicSumData as logicCalcCubicSumData,
  calculateCubeMultiplier as logicCalcCubeMultiplier,
  calculateCubeMultiplierWithTau as logicCalcCubeMultiplierWithTau,
  calculateDilatedFiveLeafBonus as logicCalcDilatedFiveLeafBonus,
  calculateExalt3AscensionLimit as logicCalcExalt3AscensionLimit,
  calculateExalt3Penalty as logicCalcExalt3Penalty,
  calculateExalt4EffectiveSingularityMultiplier as logicCalcExalt4EffSingMult,
  calculateExalt6Penalty as logicCalcExalt6Penalty,
  calculateExalt6PenaltyPerSecond as logicCalcExalt6PenaltyPerSecond,
  calculateExalt6TimeLimit as logicCalcExalt6TimeLimit,
  calculateFreeShopInfinityUpgrades as logicCalcFreeShopInfinityUpgrades,
  calculateGlobalSpeedDREnabledMult as logicCalcGlobalSpeedDREnabledMult,
  calculateGlobalSpeedDRIgnoreMult as logicCalcGlobalSpeedDRIgnoreMult,
  calculateGlobalSpeedMult as logicCalcGlobalSpeedMult,
  calculateGoldenQuarkCost as logicCalcGoldenQuarkCost,
  calculateGoldenQuarks as logicCalcGoldenQuarks,
  calculateHepteractMultiplier as logicCalcHepteractMultiplier,
  calculateHypercubeMultiplier as logicCalcHypercubeMultiplier,
  calculateImmaculateAlchemyBonus as logicCalcImmaculateAlchemyBonus,
  calculateLuckConversion as logicCalcLuckConversion,
  calculateNumberOfThresholds as logicCalcNumberOfThresholds,
  calculateObtainiumPotionBaseObtainium as logicCalcObtainiumPotionBaseObtainium,
  calculateOfferingPotionBaseOfferings as logicCalcOfferingPotionBaseOfferings,
  calculatePotionValue as logicCalcPotionValue,
  calculateQuarkMultFromPowder as logicCalcQuarkMultFromPowder,
  calculateNegativeSalvage as logicCalcNegativeSalvage,
  calculateNegativeSalvageMultiplier as logicCalcNegativeSalvageMultiplier,
  calculateObtainium as logicCalcObtainium,
  calculateObtainiumDecimal as logicCalcObtainiumDecimal,
  calculateObtainiumDRIgnoreMult as logicCalcObtainiumDRIgnoreMult,
  calculateOcteractMultiplier as logicCalcOcteractMultiplier,
  calculateOfferings as logicCalcOfferings,
  calculateOfferingsDecimal as logicCalcOfferingsDecimal,
  calculatePlatonic7UpgradePower as logicCalcPlatonic7UpgradePower,
  calculatePlatonicMultiplier as logicCalcPlatonicMultiplier,
  calculatePositiveSalvage as logicCalcPositiveSalvage,
  calculatePositiveSalvageMultiplier as logicCalcPositiveSalvageMultiplier,
  calculatePowderConversion as logicCalcPowderConversion,
  calculateQuarkMultiplier as logicCalcQuarkMultiplier,
  calculateRawAntSpeedMult as logicCalcRawAntSpeedMult,
  calculateRawAscensionSpeedMult as logicCalcRawAscensionSpeedMult,
  calculateRawNegativeSalvage as logicCalcRawNegativeSalvage,
  calculateRawPositiveSalvage as logicCalcRawPositiveSalvage,
  calculateRedAmbrosiaCubes as logicCalcRedAmbrosiaCubes,
  calculateRedAmbrosiaGenerationSpeed as logicCalcRedAmbrosiaGenerationSpeed,
  calculateRedAmbrosiaLuck as logicCalcRedAmbrosiaLuck,
  calculateRedAmbrosiaObtainium as logicCalcRedAmbrosiaObtainium,
  calculateRedAmbrosiaOffering as logicCalcRedAmbrosiaOffering,
  calculateRequiredBlueberryTime as logicCalcRequiredBlueberryTime,
  calculateRequiredRedAmbrosiaTime as logicCalcRequiredRedAmbrosiaTime,
  calculateSalvageRuneEXPMultiplier as logicCalcSalvageRuneEXPMultiplier,
  calculateSigmoid as logicCalcSigmoid,
  calculateSigmoidExponential as logicCalcSigmoidExponential,
  calculateSingularityAmbrosiaLuckMilestoneBonus as logicCalcSingAmbrosiaLuckBonus,
  calculateSingularityMilestoneBlueberries as logicCalcSingularityMilestoneBlueberries,
  calculateSingularityQuarkMilestoneMultiplier as logicCalcSingQuarkMilestoneMult,
  calculateSummationNonLinear as logicCalcSummationNonLinear,
  calculateTesseractMultiplier as logicCalcTesseractMultiplier,
  calculateTotalAcceleratorBoost as logicCalcTotalAcceleratorBoost,
  calculateTotalCoinOwned as logicCalcTotalCoinOwned,
  calculateTotalOcteractCubeBonus as logicCalcTotalOcteractCubeBonus,
  calculateTotalOcteractObtainiumBonus as logicCalcTotalOcteractObtainiumBonus,
  calculateTotalOcteractOfferingBonus as logicCalcTotalOcteractOfferingBonus,
  calculateTotalOcteractQuarkBonus as logicCalcTotalOcteractQuarkBonus,
  calculateToNextThreshold as logicCalcToNextThreshold,
  calculateTotalSalvage as logicCalcTotalSalvage,
  computeAscensionScoreBonusMultiplier as logicComputeAscScoreBonusMult,
  derpsmithCornucopiaBonus as logicDerpsmithCornucopiaBonus,
  inheritanceTokens as logicInheritanceTokens,
  singularityBonusTokenMult as logicSingularityBonusTokenMult,
  sumOfExaltCompletions as logicSumOfExaltCompletions
} from '@synergism/logic'
import Decimal from 'break_infinity.js'
import i18next from 'i18next'
import { awardUngroupedAchievement, getAchievementReward } from './Achievements'
import { getAmbrosiaUpgradeEffects } from './BlueberryUpgrades'
import { DOMCacheGetOrSet } from './Cache/DOM'
import { calculateAntSacrificeCubeBlessing, calculateObtainiumCubeBlessing } from './Cubes'
import { BuffType, calculateEventSourceBuff } from './Event'
import { generateAntsAndCrumbs } from './Features/Ants/AntProducers/lib/generate-ant-producers'
import { resetPlayerRebornELODaily } from './Features/Ants/AntSacrifice/Rewards/ELO/RebornELO/player/reset'
import { thresholdModifiers } from './Features/Ants/AntSacrifice/Rewards/ELO/RebornELO/Stages/lib/threshold'
import { calculateAntSacrificeObtainium } from './Features/Ants/AntSacrifice/Rewards/Obtainium/calculate-obtainium'
import { getAntUpgradeEffect } from './Features/Ants/AntUpgrades/lib/upgrade-effects'
import { AntUpgrades } from './Features/Ants/AntUpgrades/structs/structs'
import { addTimers, automaticTools } from './Helper'
import { hepteractEffective } from './Hepteracts'
import { disableHotkeys, enableHotkeys } from './Hotkeys'
import { getLevelMilestone } from './Levels'
import { getOcteractUpgradeEffect } from './Octeracts'
import { calculateAscensionScorePlatonicBlessing } from './PlatonicCubes'
import { PCoinUpgradeEffects } from './PseudoCoinUpgrades'
import { quarkHandler } from './Quark'
import { getRedAmbrosiaUpgradeEffects } from './RedAmbrosiaUpgrades'
import { updatePrestigeCount, updateReincarnationCount, updateTranscensionCount } from './Reset'
import { getRuneEffects, sumOfRuneLevels } from './Runes'
import { getShopUpgradeEffects } from './Shop'
import { getGQUpgradeEffect } from './singularity'
import { getSingularityChallengeEffect } from './SingularityChallenges'
import {
  allAdditiveLuckMultStats,
  allAmbrosiaBlueberryStats,
  allAmbrosiaGenerationSpeedStats,
  allAmbrosiaLuckStats,
  allAscensionSpeedStats,
  allBaseObtainiumStats,
  allBaseOfferingStats,
  allCubeStats,
  allGlobalSpeedIgnoreDRStats,
  allGlobalSpeedStats,
  allGoldenQuarkMultiplierStats,
  allGoldenQuarkPurchaseCostStats,
  allHepteractCubeStats,
  allHypercubeStats,
  allLuckConversionStats,
  allObtainiumIgnoreDRStats,
  allObtainiumStats,
  allOcteractCubeStats,
  allOfferingStats,
  allPlatonicCubeStats,
  allPowderMultiplierStats,
  allQuarkStats,
  allRedAmbrosiaGenerationSpeedStats,
  allRedAmbrosiaLuckStats,
  allShopTablets,
  allTesseractStats,
  allWowCubeStats,
  antSacrificeRewardStats,
  antSpeedStats,
  ascensionCountMultStats,
  negativeSalvageStats,
  offeringObtainiumTimeModifiers,
  positiveSalvageStats
} from './Statistics'
import { format, getTimePinnedToLoadDate, player, resourceGain, saveSynergy, updateAll } from './Synergism'
import { getTalismanEffects, toggleTalismanBuy, updateTalismanInventory } from './Talismans'
import { clearInterval, setInterval } from './Timers'
import { Alert, Prompt } from './UpdateHTML'
import { Globals as G } from './Variables'

const posSalvagePerkSings = [230, 245, 260, 275, 290]
const negSalvagePerkSings = [75, 85, 105, 125, 155, 185, 215, 245, 260, 275]

export const calculateAllCubeMultiplier = () => logicCalcAllCubeMultiplier(allCubeStats.map(s => s.stat()))
export const calculateCubeMultiplier = () => logicCalcCubeMultiplier(allWowCubeStats.map(s => s.stat()))

export const calculateCubeMultiplierWithTau = () => logicCalcCubeMultiplierWithTau({
  base: calculateCubeMultiplier(),
  tauPower: getGQUpgradeEffect('platonicTau', 'tauPower')
})

export const calculateTesseractMultiplier = () => logicCalcTesseractMultiplier(allTesseractStats.map(s => s.stat()))
export const calculateHypercubeMultiplier = () => logicCalcHypercubeMultiplier(allHypercubeStats.map(s => s.stat()))
export const calculatePlatonicMultiplier = () => logicCalcPlatonicMultiplier(allPlatonicCubeStats.map(s => s.stat()))
export const calculateHepteractMultiplier = () => logicCalcHepteractMultiplier(allHepteractCubeStats.map(s => s.stat()))
export const calculateOcteractMultiplier = () => logicCalcOcteractMultiplier(allOcteractCubeStats.map(s => s.stat()))

// 'Decimal' is used for calculating stats that can exceed the 1e300 cap.
export const calculateOfferingsDecimal = () => logicCalcOfferingsDecimal(allOfferingStats.map(s => s.stat()))
export const calculateBaseOfferings = () => logicCalcBaseOfferings(allBaseOfferingStats.map(s => s.stat()))

export const calculateOfferings = (timeMultUsed = true) => {
  const timeMultiplier = timeMultUsed
    ? offeringObtainiumTimeModifiers(player.prestigecounter, getLevelMilestone('offeringTimerScaling') === 1).reduce(
      (a, b) => a * b.stat(),
      1
    )
    : 1

  return logicCalcOfferings({
    baseOfferings: calculateBaseOfferings(),
    timeMultiplier,
    offeringMult: calculateOfferingsDecimal(),
    taxmanLastStandEnabled: player.singularityChallenges.taxmanLastStand.enabled,
    taxmanLastStandCompletions: player.singularityChallenges.taxmanLastStand.completions,
    currentOfferings: player.offerings
  })
}

// 'Decimal' is used for calculating stats that can exceed the 1e300 cap.
export const calculateObtainiumDecimal = () => logicCalcObtainiumDecimal({
  stats: allObtainiumStats.map(s => s.stat()),
  obtainiumCubeBlessing: calculateObtainiumCubeBlessing()
})

export const calculateBaseObtainium = () => logicCalcBaseObtainium(allBaseObtainiumStats.map(s => s.stat()))
export const calculateObtainiumDRIgnoreMult = () =>
  logicCalcObtainiumDRIgnoreMult(allObtainiumIgnoreDRStats.map(s => s.stat()))

/**
 * @param timeMultUsed Default true. If false, gives multiplier as if time multiplier was 1
 * @param logMultOnly Default false. If true, returns the log10 of the obtainium multiplier, possibly greater than 300.
 * @returns
 */
export const calculateObtainium = (timeMultUsed = true) => {
  const timeMultiplier = timeMultUsed
    ? offeringObtainiumTimeModifiers(player.reincarnationcounter, player.reincarnationCount >= 5)
      .reduce((a, b) => a * b.stat(), 1)
    : 1

  return logicCalcObtainium({
    baseObtainium: calculateBaseObtainium(),
    immaculate: calculateObtainiumDRIgnoreMult(),
    DR: player.corruptions.used.corruptionEffects('illiteracy'),
    timeMultiplier,
    baseMults: calculateObtainiumDecimal(),
    inAscensionChallenge14: player.currentChallenge.ascension === 14,
    taxmanLastStandEnabled: player.singularityChallenges.taxmanLastStand.enabled,
    taxmanLastStandCompletions: player.singularityChallenges.taxmanLastStand.completions,
    currentObtainium: player.obtainium
  })
}

export const calculatePotionValue = (resetTime: number, resourceMult: Decimal, baseResource: number) =>
  logicCalcPotionValue({
    resetTime,
    resourceMult,
    baseResource,
    halfMindUnlocked: getGQUpgradeEffect('halfMind', 'unlocked'),
    globalSpeedMult: calculateGlobalSpeedMult(),
    resetTimeThreshold: resetTimeThreshold(),
    potionMultipliers: getGQUpgradeEffect('potionBuff', 'potionPowerMult')
      * getGQUpgradeEffect('potionBuff2', 'potionPowerMult')
      * getGQUpgradeEffect('potionBuff3', 'potionPowerMult')
      * getOcteractUpgradeEffect('octeractAutoPotionEfficiency', 'potionPowerMult')
  })

export const calculateResearchAutomaticObtainium = (deltaTime: number) => {
  if (player.currentChallenge.ascension === 14) {
    return new Decimal('0')
  }

  const multiplier = 0.5 * player.researches[61]
    + 0.1 * player.researches[62]
    + 0.8 * player.cubeUpgrades[3]

  if (multiplier === 0) {
    return new Decimal('0')
  }

  const useTimer = false
  const resourceMult = calculateObtainium(useTimer)
  const globalSpeedMult = calculateGlobalSpeedMult()
  const resetTimeDivisor = resetTimeThreshold()
  const timePenaltyMult = Math.min(1, player.reincarnationcounter / resetTimeDivisor)

  const baseObtainium = calculateBaseObtainium()
  const nonBaseValue = resourceMult.times(globalSpeedMult).times(timePenaltyMult)
  let nonBaseAntValue = new Decimal(0)
  if (player.cubeUpgrades[47] > 0) {
    const stageMod = thresholdModifiers().antSacrificeObtainiumMult
    const antMult = calculateAntSacrificeObtainium(stageMod, useTimer)
    const antTimePenaltyMult = Math.min(1, player.antSacrificeTimer / resetTimeDivisor)
    nonBaseAntValue = antMult.times(globalSpeedMult).times(antTimePenaltyMult)
  }

  return Decimal.max(baseObtainium, Decimal.max(nonBaseValue, nonBaseAntValue)).times(deltaTime).div(resetTimeDivisor)
    .times(multiplier)
}

export const calculateQuarkMultiplier = () => logicCalcQuarkMultiplier(allQuarkStats.map(s => s.stat()))

export const calculateAntSacrificeMultiplier = () => logicCalcAntSacrificeMultiplier({
  stats: antSacrificeRewardStats.map(s => s.stat()),
  antSacrificeCubeBlessing: calculateAntSacrificeCubeBlessing()
})

export const calculateGlobalSpeedDRIgnoreMult = () =>
  logicCalcGlobalSpeedDRIgnoreMult(allGlobalSpeedIgnoreDRStats.map(s => s.stat()))
export const calculateGlobalSpeedDREnabledMult = () =>
  logicCalcGlobalSpeedDREnabledMult(allGlobalSpeedStats.map(s => s.stat()))

export const calculateGlobalSpeedMult = () => {
  const totalTimeMultiplier = logicCalcGlobalSpeedMult({
    normalMult: calculateGlobalSpeedDREnabledMult(),
    immaculateMult: calculateGlobalSpeedDRIgnoreMult(),
    drPower: calculatePlatonic7UpgradePower()
  })

  // Achievement awards stay in web_ui — side effects, not part of the
  // multiplier computation.
  // One second in 100 years
  if (totalTimeMultiplier < 1 / (3600 * 24 * 365 * 100)) {
    awardUngroupedAchievement('verySlow')
  }
  // One hour in a second
  if (totalTimeMultiplier > 3600) {
    awardUngroupedAchievement('veryFast')
  }

  return totalTimeMultiplier
}

export const calculateRawAscensionSpeedMult = () =>
  logicCalcRawAscensionSpeedMult(allAscensionSpeedStats.map(s => s.stat()))

export const calculateAscensionSpeedMult = () => {
  return logicCalcAscensionSpeedMult({
    base: calculateRawAscensionSpeedMult(),
    exponentSpread: calculateAscensionSpeedExponentSpread()
  })
}

export const calculateAmbrosiaAdditiveLuckMult = () =>
  logicCalcAmbrosiaAdditiveLuckMult(allAdditiveLuckMultStats.map(s => s.stat()))
export const calculateAmbrosiaLuckRaw = () =>
  logicCalcAmbrosiaLuckRaw(allAmbrosiaLuckStats.map(s => s.stat()))

export const calculateAmbrosiaLuck = () => logicCalcAmbrosiaLuck({
  rawLuck: calculateAmbrosiaLuckRaw(),
  multiplier: calculateAmbrosiaAdditiveLuckMult()
})

export const calculateBlueberryInventory = () =>
  logicCalcBlueberryInventory(allAmbrosiaBlueberryStats.map(s => s.stat()))
export const calculateAmbrosiaGenerationSpeedRaw = () =>
  logicCalcAmbrosiaGenerationSpeedRaw(allAmbrosiaGenerationSpeedStats.map(s => s.stat()))

export const calculateAmbrosiaGenerationSpeed = () => logicCalcAmbrosiaGenerationSpeed({
  rawSpeed: calculateAmbrosiaGenerationSpeedRaw(),
  blueberries: calculateBlueberryInventory()
})

export const calculatePowderConversion = () =>
  logicCalcPowderConversion(allPowderMultiplierStats.map(s => s.stat()))
export const calculateGoldenQuarks = () =>
  logicCalcGoldenQuarks(allGoldenQuarkMultiplierStats.map(s => s.stat()))
export const calculateGoldenQuarkCost = () =>
  logicCalcGoldenQuarkCost(allGoldenQuarkPurchaseCostStats.map(s => s.stat()))
export const calculateLuckConversion = () =>
  logicCalcLuckConversion(allLuckConversionStats.map(s => s.stat()))
export const calculateRedAmbrosiaLuck = () =>
  logicCalcRedAmbrosiaLuck(allRedAmbrosiaLuckStats.map(s => s.stat()))
export const calculateRedAmbrosiaGenerationSpeed = () =>
  logicCalcRedAmbrosiaGenerationSpeed(allRedAmbrosiaGenerationSpeedStats.map(s => s.stat()))
export const calculateFreeShopInfinityUpgrades = () =>
  logicCalcFreeShopInfinityUpgrades(allShopTablets.map(s => s.stat()))

export const calculateTotalCoinOwned = () => logicCalcTotalCoinOwned({
  firstOwnedCoin: player.firstOwnedCoin,
  secondOwnedCoin: player.secondOwnedCoin,
  thirdOwnedCoin: player.thirdOwnedCoin,
  fourthOwnedCoin: player.fourthOwnedCoin,
  fifthOwnedCoin: player.fifthOwnedCoin
})

export const calculateTotalAcceleratorBoost = () => {
  const { freeAcceleratorBoost, totalAcceleratorBoost } = logicCalcTotalAcceleratorBoost({
    upgrade26: player.upgrades[26],
    upgrade31: player.upgrades[31],
    totalCoinOwned: calculateTotalCoinOwned(),
    achievementAccelBoosts: +getAchievementReward('accelBoosts'),
    research93: player.researches[93],
    sumOfRuneLevels: sumOfRuneLevels(),
    research3: player.researches[3],
    challengeCompletions14: player.challengecompletions[14],
    research16: player.researches[16],
    research17: player.researches[17],
    research88: player.researches[88],
    antBuildingAcceleratorBoostMult: getAntUpgradeEffect(AntUpgrades.AcceleratorBoosts).acceleratorBoostMult,
    research127: player.researches[127],
    research142: player.researches[142],
    research157: player.researches[157],
    research172: player.researches[172],
    research187: player.researches[187],
    research200: player.researches[200],
    cubeUpgrade50: player.cubeUpgrades[50],
    hepteractEffectiveAcceleratorBoost: hepteractEffective('acceleratorBoost'),
    upgrade73: player.upgrades[73],
    inReincarnationChallenge: player.currentChallenge.reincarnation !== 0,
    acceleratorBoostBought: player.acceleratorBoostBought
  })
  G.freeAcceleratorBoost = freeAcceleratorBoost
  G.totalAcceleratorBoost = totalAcceleratorBoost
}

export const calculateAcceleratorMultiplier = () => {
  G.acceleratorMultiplier = logicCalcAcceleratorMultiplier({
    research1: player.researches[1],
    challengeCompletions14: player.challengecompletions[14],
    research6: player.researches[6],
    research7: player.researches[7],
    research8: player.researches[8],
    research9: player.researches[9],
    research10: player.researches[10],
    research86: player.researches[86],
    research126: player.researches[126],
    research141: player.researches[141],
    research156: player.researches[156],
    research171: player.researches[171],
    research186: player.researches[186],
    research200: player.researches[200],
    cubeUpgrade50: player.cubeUpgrades[50],
    upgrade21: player.upgrades[21],
    upgrade22: player.upgrades[22],
    upgrade23: player.upgrades[23],
    upgrade24: player.upgrades[24],
    upgrade25: player.upgrades[25],
    upgrade50: player.upgrades[50],
    inTranscensionOrReincarnationChallenge: player.currentChallenge.transcension !== 0
      || player.currentChallenge.reincarnation !== 0
  })
}

export const calculatePositiveSalvageMultiplier = () => logicCalcPositiveSalvageMultiplier({
  positiveSalvagePerkUnlockedCount: posSalvagePerkSings.filter(x => x <= player.highestSingularityCount).length,
  talismanAchievementPositiveSalvageMult: getTalismanEffects('achievement').positiveSalvageMult
})

export const calculateRawPositiveSalvage = () => logicCalcRawPositiveSalvage(positiveSalvageStats.map(s => s.stat()))

export const calculatePositiveSalvage = () => {
  return logicCalcPositiveSalvage({
    rawPositiveSalvage: calculateRawPositiveSalvage(),
    positiveSalvageMultiplier: calculatePositiveSalvageMultiplier(),
    taxmanLastStandEnabled: player.singularityChallenges.taxmanLastStand.enabled
  })
}

export const calculateNegativeSalvageMultiplier = () => logicCalcNegativeSalvageMultiplier({
  negativeSalvagePerkUnlockedCount: negSalvagePerkSings.filter(x => x <= player.highestSingularityCount).length,
  talismanAchievementNegativeSalvageMult: getTalismanEffects('achievement').negativeSalvageMult
})

export const calculateRawNegativeSalvage = () => logicCalcRawNegativeSalvage(negativeSalvageStats.map(s => s.stat()))

export const calculateNegativeSalvage = () => logicCalcNegativeSalvage({
  rawNegativeSalvage: calculateRawNegativeSalvage(),
  negativeSalvageMultiplier: calculateNegativeSalvageMultiplier()
})

export const calculateTotalSalvage = () => logicCalcTotalSalvage({
  positiveSalvage: calculatePositiveSalvage(),
  negativeSalvage: calculateNegativeSalvage()
})

export const calculateSalvageRuneEXPMultiplier = (salvageVal?: number): Decimal => {
  const salvage = salvageVal ?? calculateTotalSalvage()
  return logicCalcSalvageRuneEXPMultiplier(salvage)
}

export const calculateRawAntSpeedMult = () =>
  logicCalcRawAntSpeedMult(antSpeedStats.map(s => s.stat()))

export const calculateActualAntSpeedMult = () => {
  return logicCalcActualAntSpeedMult({
    base: calculateRawAntSpeedMult(),
    ascensionChallenge: player.currentChallenge.ascension,
    platonicUpgrade10: player.platonicUpgrades[10]
  })
}

export const timeWarp = async () => {
  const time = await Prompt(i18next.t('calculate.timePrompt'))
  const timeUse = Number(time)
  if (Number.isNaN(timeUse) || timeUse <= 0) {
    return Alert(i18next.t('calculate.timePromptError'))
  }

  DOMCacheGetOrSet('offlineContainer').style.display = 'flex'
  DOMCacheGetOrSet('offlineBlur').style.display = ''
  calculateOffline(timeUse)
}

/**
 * @param forceTime The number of SECONDS to warp. Why the fuck is it in seconds?
 */
export const calculateOffline = (forceTime = 0, fromTips = false) => {
  disableHotkeys()

  G.timeWarp = true

  // Variable Declarations i guess
  const maximumTimer = !fromTips
    ? (86400 * 3
      + 7200 * 2 * player.researches[31]
      + 7200 * 2 * player.researches[32])
      * PCoinUpgradeEffects.OFFLINE_TIMER_CAP_BUFF
    : 1e100 // If someone exceeds this, we will be very rich aha!

  const updatedTime = Date.now()
  const timeAdd = Math.min(
    maximumTimer,
    Math.max(forceTime, (updatedTime - player.offlinetick) / 1000)
  )
  const timeTick = timeAdd / 200
  let resourceTicks = 200

  DOMCacheGetOrSet('offlineTimer').textContent = i18next.t(
    'calculate.offlineTimer',
    { value: format(timeAdd, 0) }
  )

  // May 11, 2021: I've revamped calculations for this significantly. Note to May 11 Platonic: Fuck off -May 15 Platonic
  // Some one-time tick things that are relatively important
  toggleTalismanBuy(player.buyTalismanShardPercent)
  updateTalismanInventory()

  const offlineDialog = player.offlinetick > 0

  player.offlinetick = player.offlinetick < 1.5e12 ? Date.now() : player.offlinetick

  G.timeMultiplier = calculateGlobalSpeedMult()
  const obtainiumGain = calculateResearchAutomaticObtainium(timeAdd)

  const resetAdd = {
    prestige: (player.prestigeCount > 0) ? timeAdd / Math.max(0.25, player.fastestprestige) : 0,
    offering: Math.floor(timeAdd),
    transcension: (player.transcendCount > 0) ? timeAdd / Math.max(0.25, player.fastesttranscend) : 0,
    reincarnation: (player.reincarnationCount > 0) ? timeAdd / Math.max(0.25, player.fastestreincarnate) : 0,
    obtainium: obtainiumGain.times(timeAdd).times(G.timeMultiplier)
  }

  const resetAddDisplay = {
    prestige: player.prestigeCount,
    transcension: player.transcendCount,
    reincarnation: player.reincarnationCount
  }

  const timerAdd = {
    prestige: timeAdd * G.timeMultiplier,
    transcension: timeAdd * G.timeMultiplier,
    reincarnation: timeAdd * G.timeMultiplier,
    ants: timeAdd * G.timeMultiplier,
    antsReal: timeAdd,
    ascension: player.ascensionCounter, // Calculate this after the fact
    quarks: quarkHandler().gain, // Calculate this after the fact
    ambrosia: player.lifetimeAmbrosia,
    redAmbrosia: player.lifetimeRedAmbrosia,
    ambrosiaPoints: timeAdd * calculateAmbrosiaGenerationSpeed(),
    redAmbrosiaPoints: timeAdd * calculateRedAmbrosiaGenerationSpeed()
  }

  addTimers('ascension', timeAdd)
  addTimers('quarks', timeAdd)
  addTimers('goldenQuarks', timeAdd)
  addTimers('singularity', timeAdd)
  addTimers('octeracts', timeTick)
  addTimers('ambrosia', timeAdd)
  addTimers('redAmbrosia', timeAdd)

  updatePrestigeCount(resetAdd.prestige)
  updateTranscensionCount(resetAdd.transcension)
  updateReincarnationCount(resetAdd.reincarnation)

  timerAdd.ascension = player.ascensionCounter - timerAdd.ascension
  timerAdd.quarks = quarkHandler().gain - timerAdd.quarks
  timerAdd.ambrosia = player.lifetimeAmbrosia - timerAdd.ambrosia
  timerAdd.redAmbrosia = player.lifetimeRedAmbrosia - timerAdd.redAmbrosia

  resetAddDisplay.prestige = player.prestigeCount - resetAddDisplay.prestige
  resetAddDisplay.transcension = player.transcendCount - resetAddDisplay.transcension
  resetAddDisplay.reincarnation = player.reincarnationCount - resetAddDisplay.reincarnation

  // 200 simulated all ticks [July 12, 2021]
  const runOffline = setInterval(() => {
    G.timeMultiplier = calculateGlobalSpeedMult()
    calculateObtainium()

    // Reset Stuff lmao!
    addTimers('prestige', timeTick)
    addTimers('transcension', timeTick)
    addTimers('reincarnation', timeTick)
    addTimers('octeracts', timeTick)

    resourceGain(timeTick * G.timeMultiplier)
    generateAntsAndCrumbs(timeTick)

    // Auto Obtainium Stuff
    if (player.researches[61] > 0 && player.currentChallenge.ascension !== 14) {
      automaticTools('addObtainium', timeTick)
    }

    // Auto Ant Sacrifice Stuff
    if (getAchievementReward('antSacrificeUnlock')) {
      automaticTools('antSacrifice', timeTick)
    }

    // Auto Offerings
    automaticTools('addOfferings', timeTick)
    // Auto Rune Sacrifice Stuff
    if (getShopUpgradeEffects('offeringAuto', 'autoRune') && player.autoSacrificeToggle) {
      automaticTools('runeSacrifice', timeTick)
    }

    if (resourceTicks % 5 === 1) {
      // 196, 191, ... , 6, 1 ticks remaining
      updateAll()
    }

    resourceTicks -= 1
    // Misc functions
    if (resourceTicks < 1) {
      clearInterval(runOffline)
      G.timeWarp = false
    }
  }, 0)

  DOMCacheGetOrSet('offlinePrestigeCount').innerHTML = i18next.t(
    'offlineProgress.prestigeCount',
    {
      value: format(resetAddDisplay.prestige, 0, true)
    }
  )
  DOMCacheGetOrSet('offlinePrestigeTimer').innerHTML = i18next.t(
    'offlineProgress.currentPrestigeTimer',
    {
      value: format(timerAdd.prestige, 2, false)
    }
  )
  DOMCacheGetOrSet('offlineOfferingCount').innerHTML = i18next.t(
    'offlineProgress.offeringsGenerated',
    {
      value: format(resetAdd.offering, 0, true)
    }
  )
  DOMCacheGetOrSet('offlineTranscensionCount').innerHTML = i18next.t(
    'offlineProgress.transcensionCount',
    {
      value: format(resetAddDisplay.transcension, 0, true)
    }
  )
  DOMCacheGetOrSet('offlineTranscensionTimer').innerHTML = i18next.t(
    'offlineProgress.currentTranscensionCounter',
    {
      value: format(timerAdd.transcension, 2, false)
    }
  )
  DOMCacheGetOrSet('offlineReincarnationCount').innerHTML = i18next.t(
    'offlineProgress.reincarnationCount',
    {
      value: format(resetAddDisplay.reincarnation, 0, true)
    }
  )
  DOMCacheGetOrSet('offlineReincarnationTimer').innerHTML = i18next.t(
    'offlineProgress.currentReincarnationTimer',
    {
      value: format(timerAdd.reincarnation, 2, false)
    }
  )
  DOMCacheGetOrSet('offlineObtainiumCount').innerHTML = i18next.t(
    'offlineProgress.obtainiumGenerated',
    {
      value: format(resetAdd.obtainium, 0, true)
    }
  )
  DOMCacheGetOrSet('offlineAntTimer').innerHTML = i18next.t(
    'offlineProgress.ingameAntSacTimer',
    {
      value: format(timerAdd.ants, 2, false)
    }
  )
  DOMCacheGetOrSet('offlineRealAntTimer').innerHTML = i18next.t(
    'offlineProgress.realAntSacTimer',
    {
      value: format(timerAdd.antsReal, 2, true)
    }
  )
  DOMCacheGetOrSet('offlineAscensionTimer').innerHTML = i18next.t(
    'offlineProgress.currentAscensionTimer',
    {
      value: format(timerAdd.ascension, 2, true)
    }
  )
  DOMCacheGetOrSet('offlineQuarkCount').innerHTML = i18next.t(
    'offlineProgress.exportQuarks',
    {
      value: format(timerAdd.quarks, 0, true)
    }
  )
  DOMCacheGetOrSet('offlineAmbrosiaCount').innerHTML = i18next.t(
    'offlineProgress.ambrosia',
    {
      value: format(timerAdd.ambrosia, 0, true),
      value2: format(timerAdd.ambrosiaPoints, 0, true)
    }
  )
  DOMCacheGetOrSet('offlineRedAmbrosiaCount').innerHTML = i18next.t(
    'offlineProgress.redAmbrosia',
    {
      value: format(timerAdd.redAmbrosia, 0, true),
      value2: format(timerAdd.redAmbrosiaPoints, 0, true)
    }
  )

  DOMCacheGetOrSet('progressbardescription').textContent = i18next.t(
    'calculate.offlineEarnings'
  )

  player.offlinetick = updatedTime

  saveSynergy()

  updateTalismanInventory()
  calculateObtainium()

  // allow aesthetic offline progress
  if (offlineDialog) {
    const el = DOMCacheGetOrSet('notification')
    el.classList.add('slide-out')
    el.classList.remove('slide-in')
    document.body.classList.remove('scrollbar')
    document.body.classList.add('loading')
    DOMCacheGetOrSet('offlineContainer').style.display = 'flex'
    DOMCacheGetOrSet('transparentBG').style.display = 'block'
  } else {
    exitOffline()
  }
}

export const exitOffline = () => {
  document.body.classList.remove('loading')
  document.body.classList.add('scrollbar')
  DOMCacheGetOrSet('transparentBG').style.display = 'none'
  DOMCacheGetOrSet('offlineContainer').style.display = 'none'
  DOMCacheGetOrSet('offlineBlur').style.display = 'none'
  enableHotkeys()
}

export const calculateSigmoid = (
  constant: number,
  factor: number,
  divisor: number
) => logicCalcSigmoid(constant, factor, divisor)

export const calculateSigmoidExponential = (
  constant: number,
  coefficient: number
) => logicCalcSigmoidExponential(constant, coefficient)

export const calculateTotalOcteractCubeBonus = () =>
  logicCalcTotalOcteractCubeBonus({
    exalt4Enabled: player.singularityChallenges.noOcteracts.enabled,
    totalWowOcteracts: player.totalWowOcteracts,
    octeractPow: getSingularityChallengeEffect('noOcteracts', 'octeractPow')
  })

export const calculateTotalOcteractQuarkBonus = () =>
  logicCalcTotalOcteractQuarkBonus({
    exalt4Enabled: player.singularityChallenges.noOcteracts.enabled,
    totalWowOcteracts: player.totalWowOcteracts
  })

export const calculateTotalOcteractOfferingBonus = () =>
  logicCalcTotalOcteractOfferingBonus({
    offeringBonusEnabled: getSingularityChallengeEffect('noOcteracts', 'offeringBonus'),
    cubeBonus: calculateTotalOcteractCubeBonus()
  })

export const calculateTotalOcteractObtainiumBonus = () =>
  logicCalcTotalOcteractObtainiumBonus({
    obtainiumBonusEnabled: getSingularityChallengeEffect('noOcteracts', 'obtainiumBonus'),
    cubeBonus: calculateTotalOcteractCubeBonus()
  })

export const calculateSingularityQuarkMilestoneMultiplier = () =>
  logicCalcSingQuarkMilestoneMult(player.singularityCount)

export const calculateSummationNonLinear = (
  baseLevel: number,
  baseCost: number,
  resourceAvailable: number,
  diffPerLevel: number,
  buyAmount: number
): { levelCanBuy: number; cost: number } =>
  logicCalcSummationNonLinear(baseLevel, baseCost, resourceAvailable, diffPerLevel, buyAmount)

const cubicSumErrorMessageByCode: Record<string, string> = {
  SUMMATIONS_QUADRATIC_IMPROPER: 'calculate.quadraticImproperError',
  SUMMATIONS_QUADRATIC_DETERMINANT: 'calculate.quadraticDeterminantError',
  SUMMATIONS_CUBIC_SUM_NEGATIVE: 'calculate.cubicSumNegativeError'
}

export const calculateCubicSumData = (
  initialLevel: number,
  baseCost: number,
  amountToSpend: number,
  maxLevel: number
) => {
  try {
    return logicCalcCubicSumData(initialLevel, baseCost, amountToSpend, maxLevel)
  } catch (err) {
    if (err instanceof Error && err.message in cubicSumErrorMessageByCode) {
      throw new Error(String(i18next.t(cubicSumErrorMessageByCode[err.message])))
    }
    throw err
  }
}

// IDEA: Rework this shit to be friendly for Stats for Nerds
/* May 25, 2021 - Platonic
    Reorganize this function to make sense, because right now it aint
    What I did was use the separation of cube gain method on other cube types, and made some methods their
    own function (specifically: calc of effective score and other global multipliers) to make it easy.
*/

const computeAscensionScoreBonusMultiplier = () =>
  logicComputeAscScoreBonusMult({
    challenge15ScoreReward: G.challenge15Rewards.score.value,
    platonicBlessingMult: calculateAscensionScorePlatonicBlessing(),
    campaignAscensionScoreMult: player.campaigns.ascensionScoreMultiplier,
    finiteDescentAscensionScore: getRuneEffects('finiteDescent', 'ascensionScore'),
    cubeUpgrade21: player.cubeUpgrades[21],
    cubeUpgrade31: player.cubeUpgrades[31],
    cubeUpgrade41: player.cubeUpgrades[41],
    ascensionScoreAchievementReward: +getAchievementReward('ascensionScore'),
    masterPackAscensionScoreMult: getGQUpgradeEffect('masterPack', 'ascensionScoreMult'),
    eventBuff: G.isEvent ? calculateEventBuff(BuffType.AscensionScore) : 0
  })

export const calculateAscensionScore = () =>
  logicCalcAscensionScore({
    highestChallengeCompletions: player.highestchallengecompletions,
    cubeUpgrade56: player.cubeUpgrades[56],
    cubeUpgrade39: player.cubeUpgrades[39],
    platonicUpgrade5: player.platonicUpgrades[5],
    platonicUpgrade10: player.platonicUpgrades[10],
    corruptionMultiplier: player.corruptions.used.totalCorruptionAscensionMultiplier,
    antUpgradeAscensionScoreBase: getAntUpgradeEffect(AntUpgrades.AscensionScore).ascensionScoreBase,
    expertPackAscensionScoreMult: getGQUpgradeEffect('expertPack', 'ascensionScoreMult'),
    bonusMultiplier: computeAscensionScoreBonusMultiplier()
  })

export const CalcCorruptionStuff = () =>
  logicCalcCorruptionStuff({
    scores: calculateAscensionScore(),
    cubeMultiplier: calculateCubeMultiplierWithTau(),
    tesseractMultiplier: calculateTesseractMultiplier(),
    hypercubeMultiplier: calculateHypercubeMultiplier(),
    platonicMultiplier: calculatePlatonicMultiplier(),
    hepteractMultiplier: calculateHepteractMultiplier(),
    hepteractsUnlocked: G.challenge15Rewards.hepteractsUnlocked.value,
    singularityCount: player.singularityCount
  })

export const calculateAscensionCount = () =>
  logicCalcAscensionCount({
    limitedAscensionsEnabled: player.singularityChallenges.limitedAscensions.enabled,
    ascensionCountMults: ascensionCountMultStats.map(s => s.stat())
  })

export const calculateCubeQuarkMultiplier = () =>
  logicCalcCubeQuarkMultiplier({
    overfluxOrbs: player.overfluxOrbs,
    highestSingularityCount: player.highestSingularityCount,
    cubeToQuarkAllMult: getShopUpgradeEffects('cubeToQuarkAll', 'quarkMult'),
    autoWarpCheck: player.autoWarpCheck,
    dailyPowderResetUses: player.dailyPowderResetUses
  })

export const calculateCubeMultFromPowder = () => logicCalcCubeMultFromPowder(player.overfluxPowder)

export const calculateQuarkMultFromPowder = () => logicCalcQuarkMultFromPowder(player.overfluxPowder)

export const calculateBaseGoldenQuarks = (singularity: number) =>
  logicCalcBaseGoldenQuarks({
    singularity,
    quarksThisSingularity: player.quarksThisSingularity,
    highestSingularityCount: player.highestSingularityCount
  })

export const calculateSingularityAmbrosiaLuckMilestoneBonus = () =>
  logicCalcSingAmbrosiaLuckBonus(player.highestSingularityCount)

export const calculateAmbrosiaGenerationSingularityUpgrade = () =>
  logicCalcAmbrosiaGenerationSingularityUpgrade([
    getGQUpgradeEffect('singAmbrosiaGeneration', 'ambrosiaBarSpeedMult'),
    getGQUpgradeEffect('singAmbrosiaGeneration2', 'ambrosiaBarSpeedMult'),
    getGQUpgradeEffect('singAmbrosiaGeneration3', 'ambrosiaBarSpeedMult'),
    getGQUpgradeEffect('singAmbrosiaGeneration4', 'ambrosiaBarSpeedMult')
  ])

export const calculateAmbrosiaLuckSingularityUpgrade = () =>
  logicCalcAmbrosiaLuckSingularityUpgrade([
    getGQUpgradeEffect('singAmbrosiaLuck', 'ambrosiaLuck'),
    getGQUpgradeEffect('singAmbrosiaLuck2', 'ambrosiaLuck'),
    getGQUpgradeEffect('singAmbrosiaLuck3', 'ambrosiaLuck'),
    getGQUpgradeEffect('singAmbrosiaLuck4', 'ambrosiaLuck')
  ])

export const calculateAmbrosiaGenerationOcteractUpgrade = () =>
  logicCalcAmbrosiaGenerationOcteractUpgrade([
    getOcteractUpgradeEffect('octeractAmbrosiaGeneration', 'ambrosiaBarSpeedMult'),
    getOcteractUpgradeEffect('octeractAmbrosiaGeneration2', 'ambrosiaBarSpeedMult'),
    getOcteractUpgradeEffect('octeractAmbrosiaGeneration3', 'ambrosiaBarSpeedMult'),
    getOcteractUpgradeEffect('octeractAmbrosiaGeneration4', 'ambrosiaBarSpeedMult')
  ])

export const calculateAmbrosiaLuckOcteractUpgrade = () =>
  logicCalcAmbrosiaLuckOcteractUpgrade([
    getOcteractUpgradeEffect('octeractAmbrosiaLuck', 'ambrosiaLuck'),
    getOcteractUpgradeEffect('octeractAmbrosiaLuck2', 'ambrosiaLuck'),
    getOcteractUpgradeEffect('octeractAmbrosiaLuck3', 'ambrosiaLuck'),
    getOcteractUpgradeEffect('octeractAmbrosiaLuck4', 'ambrosiaLuck')
  ])

export const calculateNumberOfThresholds = () => logicCalcNumberOfThresholds(player.lifetimeAmbrosia)

export const calculateToNextThreshold = () => logicCalcToNextThreshold(player.lifetimeAmbrosia)

export const calculateRequiredBlueberryTime = () =>
  logicCalcRequiredBlueberryTime({
    timePerAmbrosia: G.TIME_PER_AMBROSIA,
    lifetimeAmbrosia: player.lifetimeAmbrosia,
    acceleratorMult: getShopUpgradeEffects('shopAmbrosiaAccelerator', 'ambrosiaPointRequirementMult'),
    brickOfLeadMult: getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead', 'barRequirementMult')
  })

export const calculateRequiredRedAmbrosiaTime = () =>
  logicCalcRequiredRedAmbrosiaTime({
    timePerRedAmbrosia: G.TIME_PER_RED_AMBROSIA,
    lifetimeRedAmbrosia: player.lifetimeRedAmbrosia,
    barRequirementMultiplier: getSingularityChallengeEffect('limitedTime', 'barRequirementMultiplier')
  })

export const calculateSingularityMilestoneBlueberries = () =>
  logicCalcSingularityMilestoneBlueberries(player.highestSingularityCount)

export const calculateAmbrosiaCubeMult = () =>
  logicCalcAmbrosiaCubeMult({
    noAmbrosiaUpgradesEnabled: player.singularityChallenges.noAmbrosiaUpgrades.enabled,
    lifetimeAmbrosia: player.lifetimeAmbrosia
  })

export const calculateAmbrosiaQuarkMult = () =>
  logicCalcAmbrosiaQuarkMult({
    noAmbrosiaUpgradesEnabled: player.singularityChallenges.noAmbrosiaUpgrades.enabled,
    lifetimeAmbrosia: player.lifetimeAmbrosia
  })

export const calculateExalt3AscensionLimit = (comps: number) => logicCalcExalt3AscensionLimit(comps)

export const calculateExalt3Penalty = () =>
  logicCalcExalt3Penalty({
    limitedAscensionsEnabled: player.singularityChallenges.limitedAscensions.enabled,
    limitedAscensionsCompletions: player.singularityChallenges.limitedAscensions.completions,
    ascensionCount: player.ascensionCount
  })

export const calculateExalt4EffectiveSingularityMultiplier = (comps: number, force: boolean) =>
  logicCalcExalt4EffSingMult({
    comps,
    force,
    inExalt4: player.singularityChallenges.noOcteracts.enabled
  })

export const calculateExalt6TimeLimit = (comps: number) => logicCalcExalt6TimeLimit(comps)

export const calculateExalt6PenaltyPerSecond = (comps: number) => logicCalcExalt6PenaltyPerSecond(comps)

export const calculateExalt6Penalty = (comps: number, time: number) => logicCalcExalt6Penalty(comps, time)

export const calculateDilatedFiveLeafBonus = () =>
  logicCalcDilatedFiveLeafBonus(player.highestSingularityCount)

export const dailyResetCheck = () => {
  if (!player.dayCheck) {
    return
  }
  const now = new Date(getTimePinnedToLoadDate())
  const day = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const h = now.getHours()
  const m = now.getMinutes()
  const s = now.getSeconds()
  player.dayTimer = 60 * 60 * 24 - 60 * 60 * h - 60 * m - s

  // Daily is not reset even if it is set to a past time.
  // If the daily is not reset, the data may have been set to a future time.
  if (day.getTime() - 3600000 > player.dayCheck.getTime()) {
    player.dayCheck = day

    forcedDailyReset(true)
    player.dailyPowderResetUses = 1 + getShopUpgradeEffects('extraWarp', 'additionalWarps')
    player.dailyCodeUsed = false

    DOMCacheGetOrSet('cubeQuarksOpenRequirement').style.display = 'block'
    if (player.challengecompletions[11] > 0) {
      DOMCacheGetOrSet('tesseractQuarksOpenRequirement').style.display = 'block'
    }
    if (player.challengecompletions[13] > 0) {
      DOMCacheGetOrSet('hypercubeQuarksOpenRequirement').style.display = 'block'
    }
    if (player.challengecompletions[14] > 0) {
      DOMCacheGetOrSet('platonicCubeQuarksOpenRequirement').style.display = 'block'
    }
  }
}

/**
 * Resets Cube Counts and stuff. NOTE: It is intentional it does not award powder or expire orbs.
 */
export const forcedDailyReset = (rewards = false) => {
  player.cubeQuarkDaily = 0
  player.tesseractQuarkDaily = 0
  player.hypercubeQuarkDaily = 0
  player.platonicCubeQuarkDaily = 0
  player.cubeOpenedDaily = 0
  player.tesseractOpenedDaily = 0
  player.hypercubeOpenedDaily = 0
  player.platonicCubeOpenedDaily = 0
  resetPlayerRebornELODaily()

  if (rewards) {
    player.overfluxPowder += player.overfluxOrbs * calculatePowderConversion()
    player.overfluxOrbs = G.challenge15Rewards.freeOrbs.value
  }
}

export const calculateEventBuff = (buff: BuffType) => {
  // if (!G.isEvent) {
  //  return 0
  // }
  return calculateEventSourceBuff(buff)
}

export const derpsmithCornucopiaBonus = () =>
  logicDerpsmithCornucopiaBonus(player.highestSingularityCount)

export const calculateImmaculateAlchemyBonus = () =>
  logicCalcImmaculateAlchemyBonus(player.singularityCount)

export const sumOfExaltCompletions = () =>
  logicSumOfExaltCompletions(Object.values(player.singularityChallenges).map(c => c.completions))

export const inheritanceTokens = () =>
  logicInheritanceTokens(player.highestSingularityCount)

export const singularityBonusTokenMult = () =>
  logicSingularityBonusTokenMult(player.highestSingularityCount)

export const resetTimeThreshold = () => {
  const base = 10
  let reduction = 0

  reduction += player.campaigns.timeThresholdReduction

  return base - reduction
}

const calculatePlatonic7UpgradePower = () => logicCalcPlatonic7UpgradePower(player.platonicUpgrades[7])

export const calculateOfferingPotionBaseOfferings = () =>
  logicCalcOfferingPotionBaseOfferings(player.shopPotionsConsumed.offering)

export const calculateObtainiumPotionBaseObtainium = () =>
  logicCalcObtainiumPotionBaseObtainium(player.shopPotionsConsumed.obtainium)

export const calculateAscensionSpeedExponentSpread = () => logicCalcAscensionSpeedExponentSpread({
  singAscensionSpeedExponentSpread: getGQUpgradeEffect('singAscensionSpeed', 'exponentSpread'),
  singAscensionSpeed2ExponentSpread: getGQUpgradeEffect('singAscensionSpeed2', 'exponentSpread'),
  chronometerInfinityExponentSpread: getShopUpgradeEffects('chronometerInfinity', 'exponentSpread')
})

export const calculateCookieUpgrade29Luck = () =>
  logicCalcCookieUpgrade29Luck({
    cubeUpgrade79: player.cubeUpgrades[79],
    lifetimeRedAmbrosia: player.lifetimeRedAmbrosia
  })

export const calculateRedAmbrosiaCubes = () =>
  logicCalcRedAmbrosiaCubes({
    unlocked: getRedAmbrosiaUpgradeEffects('redAmbrosiaCube', 'unlockedRedAmbrosiaCube'),
    lifetimeRedAmbrosia: player.lifetimeRedAmbrosia,
    extraExponent: getRedAmbrosiaUpgradeEffects('redAmbrosiaCubeImprover', 'extraExponent')
  })

export const calculateRedAmbrosiaObtainium = () =>
  logicCalcRedAmbrosiaObtainium({
    unlocked: getRedAmbrosiaUpgradeEffects('redAmbrosiaObtainium', 'unlockRedAmbrosiaObtainium'),
    lifetimeRedAmbrosia: player.lifetimeRedAmbrosia
  })

export const calculateRedAmbrosiaOffering = () =>
  logicCalcRedAmbrosiaOffering({
    unlocked: getRedAmbrosiaUpgradeEffects('redAmbrosiaOffering', 'unlockRedAmbrosiaOffering'),
    lifetimeRedAmbrosia: player.lifetimeRedAmbrosia
  })
