// Public API surface for @synergism/logic.
//
// Re-export the pieces that the UI tier is allowed to consume here. Anything
// not exported from this file should be considered internal to the package.

export type { CoreEvent } from './events/types'
export { calculateSigmoid, calculateSigmoidExponential } from './math/sigmoid'
export type { CalculateCubicSumDataResult, CalculateSummationNonLinearResult } from './math/summations'
export {
  calculateCubicSumData,
  calculateSummationCubic,
  calculateSummationNonLinear,
  solveQuadratic
} from './math/summations'
export type { GetAcceleratorBoostCostInput } from './mechanics/acceleratorBoosts'
export { getAcceleratorBoostCost } from './mechanics/acceleratorBoosts'
export type {
  CalculateAcceleratorMultiplierInput,
  CalculateTotalAcceleratorBoostInput,
  CalculateTotalAcceleratorBoostResult
} from './mechanics/acceleratorMultipliers'
export { calculateAcceleratorMultiplier, calculateTotalAcceleratorBoost } from './mechanics/acceleratorMultipliers'
export type { BuyAcceleratorInput, GetCostAcceleratorInput } from './mechanics/accelerators'
export { buyAccelerator, getCostAccelerator } from './mechanics/accelerators'
export { achievementLevelFromPoints, toNextAchievementLevelEXP } from './mechanics/achievementLevels'
export type {
  AmbrosiaMultInput,
  CalculateRequiredBlueberryTimeInput,
  CalculateRequiredRedAmbrosiaTimeInput
} from './mechanics/ambrosia'
export {
  calculateAmbrosiaCubeMult,
  calculateAmbrosiaGenerationOcteractUpgrade,
  calculateAmbrosiaGenerationSingularityUpgrade,
  calculateAmbrosiaLuckOcteractUpgrade,
  calculateAmbrosiaLuckSingularityUpgrade,
  calculateAmbrosiaQuarkMult,
  calculateNumberOfThresholds,
  calculateRequiredBlueberryTime,
  calculateRequiredRedAmbrosiaTime,
  calculateSingularityMilestoneBlueberries,
  calculateToNextThreshold
} from './mechanics/ambrosia'
export type { CalculateAscensionCountInput } from './mechanics/ascensions'
export { calculateAscensionCount } from './mechanics/ascensions'
export type {
  ActualAntSpeedMultInput,
  AscensionScoreBonusMultiplierInput,
  AscensionSpeedMultInput,
  CalcCorruptionStuffInput,
  CalcCorruptionStuffResult,
  CalculateAmbrosiaGenerationSpeedInput,
  CalculateAmbrosiaLuckInput,
  CalculateAntSacrificeMultiplierInput,
  CalculateAscensionScoreInput,
  CalculateAscensionScoreResult,
  CalculateAscensionSpeedExponentSpreadInput,
  CalculateCubeMultiplierWithTauInput,
  CalculateNegativeSalvageInput,
  CalculateNegativeSalvageMultiplierInput,
  CalculateObtainiumDecimalInput,
  CalculateObtainiumInput,
  CalculateOfferingsInput,
  CalculatePositiveSalvageInput,
  CalculatePositiveSalvageMultiplierInput,
  CalculateTotalCoinOwnedInput,
  CalculateTotalSalvageInput,
  GlobalSpeedMultInput,
  ReductionValueInput
} from './mechanics/calculate'
export {
  CalcCorruptionStuff,
  calculateActualAntSpeedMult,
  calculateAllCubeMultiplier,
  calculateAmbrosiaAdditiveLuckMult,
  calculateAmbrosiaGenerationSpeed,
  calculateAmbrosiaGenerationSpeedRaw,
  calculateAmbrosiaLuck,
  calculateAmbrosiaLuckRaw,
  calculateAntSacrificeMultiplier,
  calculateAscensionScore,
  calculateAscensionSpeedExponentSpread,
  calculateAscensionSpeedMult,
  calculateBaseObtainium,
  calculateBaseOfferings,
  calculateBlueberryInventory,
  calculateCubeMultiplier,
  calculateCubeMultiplierWithTau,
  calculateFreeShopInfinityUpgrades,
  calculateGlobalSpeedDREnabledMult,
  calculateGlobalSpeedDRIgnoreMult,
  calculateGlobalSpeedMult,
  calculateGoldenQuarkCost,
  calculateGoldenQuarks,
  calculateHepteractMultiplier,
  calculateHypercubeMultiplier,
  calculateLuckConversion,
  calculateNegativeSalvage,
  calculateNegativeSalvageMultiplier,
  calculateObtainium,
  calculateObtainiumDecimal,
  calculateObtainiumDRIgnoreMult,
  calculateOcteractMultiplier,
  calculateOfferings,
  calculateOfferingsDecimal,
  calculatePlatonic7UpgradePower,
  calculatePlatonicMultiplier,
  calculatePositiveSalvage,
  calculatePositiveSalvageMultiplier,
  calculatePowderConversion,
  calculateQuarkMultiplier,
  calculateRawAntSpeedMult,
  calculateRawAscensionSpeedMult,
  calculateRawNegativeSalvage,
  calculateRawPositiveSalvage,
  calculateRedAmbrosiaGenerationSpeed,
  calculateRedAmbrosiaLuck,
  calculateSalvageRuneEXPMultiplier,
  calculateTesseractMultiplier,
  calculateTotalCoinOwned,
  calculateTotalSalvage,
  computeAscensionScoreBonusMultiplier,
  getReductionValue
} from './mechanics/calculate'
export type {
  Challenge15ScoreMultiplierInput,
  ChallengeRequirementInput,
  ChallengeRequirementMultiplierInput,
  ChallengeType,
  GetMaxChallengesInput,
  GetNextAscensionChallengeInput,
  GetNextRegularChallengeInput
} from './mechanics/challenges'
export {
  autoAscensionChallengeSweepUnlock,
  CalcECC,
  calculateChallengeRequirementMultiplier,
  challenge15ScoreMultiplier,
  challengeRequirement,
  challengeScoreDisplay,
  getMaxChallenges,
  getNextAscensionChallenge,
  getNextRegularChallenge
} from './mechanics/challenges'
export type {
  CalculateCoinProductionInput,
  CalculateCoinProductionResult,
  PerCoinTierInput
} from './mechanics/coinProduction'
export { calculateCoinProduction } from './mechanics/coinProduction'
export type {
  DroughtEffectInput,
  HyperchallengeEffectInput,
  IlliteracyEffectInput,
  MaxCorruptionLevelInput,
  ViscosityEffectInput
} from './mechanics/corruptions'
export {
  droughtEffect,
  hyperchallengeEffect,
  illiteracyEffect,
  maxCorruptionLevel,
  viscosityEffect
} from './mechanics/corruptions'
export type { BuyCrystalUpgradesInput } from './mechanics/crystalUpgrades'
export { buyCrystalUpgrades } from './mechanics/crystalUpgrades'
export {
  calculateAcceleratorCubeBlessing,
  calculateAntELOCubeBlessing,
  calculateAntSacrificeCubeBlessing,
  calculateAntSpeedCubeBlessing,
  calculateGlobalSpeedCubeBlessing,
  calculateMultiplierCubeBlessing,
  calculateObtainiumCubeBlessing,
  calculateOfferingCubeBlessing,
  calculateRuneEffectivenessCubeBlessing,
  calculateSalvageCubeBlessing
} from './mechanics/cubes/cubeBlessings'
export type {
  AbyssHepteractEffects,
  AcceleratorBoostHepteractEffects,
  AcceleratorHepteractEffects,
  ChallengeHepteractEffects,
  ChronosHepteractEffects,
  HyperrealismHepteractEffects,
  MultiplierHepteractEffects,
  QuarkHepteractEffects
} from './mechanics/cubes/hepteracts'
export {
  abyssHepteractEffects,
  acceleratorBoostHepteractEffects,
  acceleratorHepteractEffects,
  challengeHepteractEffects,
  chronosHepteractEffects,
  hyperrealismHepteractEffects,
  multiplierHepteractEffects,
  quarkHepteractEffects
} from './mechanics/cubes/hepteracts'
export {
  calculateAcceleratorHypercubeBlessing,
  calculateAntELOHypercubeBlessing,
  calculateAntSacrificeHypercubeBlessing,
  calculateAntSpeedHypercubeBlessing,
  calculateGlobalSpeedHypercubeBlessing,
  calculateMultiplierHypercubeBlessing,
  calculateObtainiumHypercubeBlessing,
  calculateOfferingHypercubeBlessing,
  calculateRuneEffectivenessHypercubeBlessing,
  calculateSalvageHypercubeBlessing
} from './mechanics/cubes/hypercubeBlessings'
export {
  calculateAscensionScorePlatonicBlessing,
  calculateCubeMultiplierPlatonicBlessing,
  calculateGlobalSpeedPlatonicBlessing,
  calculateHypercubeBlessingMultiplierPlatonicBlessing,
  calculateHypercubeMultiplierPlatonicBlessing,
  calculatePlatonicMultiplierPlatonicBlessing,
  calculateTaxPlatonicBlessing,
  calculateTesseractMultiplierPlatonicBlessing
} from './mechanics/cubes/platonicBlessings'
export {
  calculateAcceleratorTesseractBlessing,
  calculateAntELOTesseractBlessing,
  calculateAntSacrificeTesseractBlessing,
  calculateAntSpeedTesseractBlessing,
  calculateGlobalSpeedTesseractBlessing,
  calculateMultiplierTesseractBlessing,
  calculateObtainiumTesseractBlessing,
  calculateOfferingTesseractBlessing,
  calculateRuneEffectivenessTesseractBlessing,
  calculateSalvageTesseractBlessing
} from './mechanics/cubes/tesseractBlessings'
export type { GetCubeCostInput, GetCubeCostResult, GetCubeMaxInput } from './mechanics/cubeUpgrades'
export { getCubeCost, getCubeMax, getCubeUpgradeBaseCost } from './mechanics/cubeUpgrades'
export type {
  CalculateExalt3PenaltyInput,
  CalculateExalt4EffectiveSingularityMultiplierInput
} from './mechanics/exaltPenalties'
export {
  calculateExalt3AscensionLimit,
  calculateExalt3Penalty,
  calculateExalt4EffectiveSingularityMultiplier,
  calculateExalt6Penalty,
  calculateExalt6PenaltyPerSecond,
  calculateExalt6TimeLimit
} from './mechanics/exaltPenalties'
export type { GQUpgradeCostTNLInput, GQUpgradeSpecialCostForm } from './mechanics/gqUpgradeCost'
export { gqUpgradeCostTNL } from './mechanics/gqUpgradeCost'
export type { ActualGQUpgradeTotalLevelsInput, GqUpgradeMaxLevelInput } from './mechanics/gqUpgradeLevels'
export {
  actualGQUpgradeTotalLevels,
  computeGQUpgradeMaxLevel,
  gqFreeLevelMultiplier,
  gqUpgradeFreeLevelSoftcap
} from './mechanics/gqUpgradeLevels'
export type { HepteractEffectiveInput } from './mechanics/hepteractValues'
export { hepteractCap, hepteractEffective, hepteractFinalCap } from './mechanics/hepteractValues'
export type { LevelMilestoneData, LevelMilestoneKey, SalvageChallengeBuffInput } from './mechanics/levelMilestones'
export {
  achievementTalismanEnhancementEffect,
  duplicationRuneMilestoneEffect,
  getLevelMilestone,
  levelMilestones,
  prismRuneMilestoneEffect,
  runeAutobuyImproverEffect,
  salvageChallengeBuffEffect,
  siRuneMilestoneEffect,
  speedRuneMilestoneEffect,
  thriftRuneMilestoneEffect
} from './mechanics/levelMilestones'
export type { LevelRewardData, LevelRewardKey } from './mechanics/levelRewards'
export {
  ambrosiaLuckEffect,
  antsEffect,
  getLevelReward,
  levelRewards,
  obtainiumEffect,
  offeringsEffect,
  quarksEffect,
  redAmbrosiaLuckEffect,
  salvageEffect,
  wowCubesEffect,
  wowHepteractCubesEffect,
  wowHyperCubesEffect,
  wowOcteractsEffect,
  wowPlatonicCubesEffect,
  wowTesseractsEffect
} from './mechanics/levelRewards'
export type { BuyMultiplierInput, GetCostMultiplierInput } from './mechanics/multipliers'
export { buyMultiplier, getCostMultiplier } from './mechanics/multipliers'
export type {
  CalculateTotalOcteractCubeBonusInput,
  CalculateTotalOcteractObtainiumBonusInput,
  CalculateTotalOcteractOfferingBonusInput,
  CalculateTotalOcteractQuarkBonusInput
} from './mechanics/octeractBonuses'
export {
  calculateTotalOcteractCubeBonus,
  calculateTotalOcteractObtainiumBonus,
  calculateTotalOcteractOfferingBonus,
  calculateTotalOcteractQuarkBonus
} from './mechanics/octeractBonuses'
export type { ActualOcteractUpgradeTotalLevelsInput } from './mechanics/octeractUpgradeLevels'
export {
  actualOcteractUpgradeTotalLevels,
  octeractFreeLevelMultiplier,
  octeractFreeLevelSoftcap
} from './mechanics/octeractUpgradeLevels'
export type { CalculateCubeQuarkMultiplierInput } from './mechanics/overfluxBonuses'
export {
  calculateCubeMultFromPowder,
  calculateCubeQuarkMultiplier,
  calculateQuarkMultFromPowder
} from './mechanics/overfluxBonuses'
export type {
  BuyParticleBuildingInput,
  GetParticleCostInput,
  ParticleBuildingIndex
} from './mechanics/particleBuildings'
export { buyParticleBuilding, getParticleCost } from './mechanics/particleBuildings'
export type { CalculatePotionValueInput, PotionBonusResult } from './mechanics/potionBonuses'
export {
  calculateObtainiumPotionBaseObtainium,
  calculateOfferingPotionBaseOfferings,
  calculatePotionValue
} from './mechanics/potionBonuses'
export type {
  BuyMaxInput,
  BuyProducerInput,
  GetProducerCostInput,
  ProducerIndex,
  ProducerType
} from './mechanics/producers'
export { buyMax, buyProducer, getProducerCost } from './mechanics/producers'
export type { QuarkHandlerInput, QuarkHandlerResult } from './mechanics/quarks'
export { quarkHandler } from './mechanics/quarks'
export type {
  CalculateCookieUpgrade29LuckInput,
  CalculateRedAmbrosiaCubesInput,
  CalculateRedAmbrosiaResourceInput
} from './mechanics/redAmbrosiaBonuses'
export {
  calculateCookieUpgrade29Luck,
  calculateRedAmbrosiaCubes,
  calculateRedAmbrosiaObtainium,
  calculateRedAmbrosiaOffering
} from './mechanics/redAmbrosiaBonuses'
export type {
  DuplicationRuneBlessingEffects,
  PrismRuneBlessingEffects,
  SpeedRuneBlessingEffects,
  SuperiorIntellectRuneBlessingEffects,
  ThriftRuneBlessingEffects
} from './mechanics/runeBlessingEffects'
export {
  duplicationRuneBlessingEffects,
  prismRuneBlessingEffects,
  speedRuneBlessingEffects,
  superiorIntellectRuneBlessingEffects,
  thriftRuneBlessingEffects
} from './mechanics/runeBlessingEffects'
export type {
  AntiquitiesRuneInput,
  AntiquitiesRuneKey,
  DuplicationRuneKey,
  FiniteDescentRuneKey,
  HorseShoeRuneKey,
  InfiniteAscentRuneInput,
  InfiniteAscentRuneKey,
  PrismRuneKey,
  SpeedRuneKey,
  SuperiorIntellectRuneKey,
  ThriftRuneKey,
  TopHatRuneKey
} from './mechanics/runeEffects'
export {
  antiquitiesRuneEffects,
  duplicationRuneEffects,
  finiteDescentRuneEffects,
  horseShoeRuneEffects,
  infiniteAscentRuneEffects,
  prismRuneEffects,
  speedRuneEffects,
  superiorIntellectRuneEffects,
  thriftRuneEffects,
  topHatRuneEffects
} from './mechanics/runeEffects'
export type { UniversalRuneEXPMultInput } from './mechanics/runeEXPMultiplier'
export { universalRuneEXPMult } from './mechanics/runeEXPMultiplier'
export type { MaxRuneLevelPurchaseInput, MaxRuneLevelPurchaseResult } from './mechanics/runeLevels'
export {
  maxRuneLevelPurchase,
  runeEXPLeftToLevel,
  runeEXPToLevel,
  runeLevelFromEXP,
  runeOfferingsToLevel
} from './mechanics/runeLevels'
export type { ShopCostInput } from './mechanics/shopCosts'
export { shopCost } from './mechanics/shopCosts'
export type {
  CalculateNextSpikeInput,
  GoldenQuarkCostResult,
  MaxSingularityLookaheadInput
} from './mechanics/singularityHelpers'
export { calculateNextSpike, goldenQuarkCost, maxSingularityLookahead } from './mechanics/singularityHelpers'
export type { CalculateBaseGoldenQuarksInput } from './mechanics/singularityMilestones'
export {
  calculateBaseGoldenQuarks,
  calculateDilatedFiveLeafBonus,
  calculateImmaculateAlchemyBonus,
  calculateSingularityAmbrosiaLuckMilestoneBonus,
  calculateSingularityQuarkMilestoneMultiplier,
  derpsmithCornucopiaBonus,
  inheritanceTokens,
  singularityBonusTokenMult,
  sumOfExaltCompletions
} from './mechanics/singularityMilestones'
export type {
  CalculateEffectiveSingularitiesInput,
  CalculateSingularityDebuffInput,
  SingularityDebuff
} from './mechanics/singularityPenalties'
export { calculateEffectiveSingularities, calculateSingularityDebuff } from './mechanics/singularityPenalties'
export type { TalismanCraftCosts, TalismanCraftItems } from './mechanics/talismanCosts'
export { exponentialCostProgression, regularCostProgression } from './mechanics/talismanCosts'
export type {
  AchievementTalismanEffects,
  ChronosTalismanEffects,
  CookieGrandmaTalismanEffects,
  ExemptionTalismanEffects,
  HorseShoeTalismanEffects,
  MetaphysicsTalismanEffects,
  MidasTalismanEffects,
  MortuusTalismanEffects,
  PlasticTalismanEffects,
  PolymathTalismanEffects,
  WowSquareTalismanEffects
} from './mechanics/talismanEffects'
export {
  achievementTalismanEffects,
  chronosTalismanEffects,
  cookieGrandmaTalismanEffects,
  exemptionTalismanEffects,
  horseShoeTalismanEffects,
  metaphysicsTalismanEffects,
  midasTalismanEffects,
  mortuusTalismanEffects,
  plasticTalismanEffects,
  polymathTalismanEffects,
  wowSquareTalismanEffects
} from './mechanics/talismanEffects'
export type { CalculateTaxInput, CalculateTaxResult } from './mechanics/tax'
export { calculateTax } from './mechanics/tax'
export type {
  BuyTesseractBuildingInput,
  GetTesseractCostInput,
  TesseractBuildingIndex,
  TesseractBuildings
} from './mechanics/tesseractBuildings'
export { buyTesseractBuilding, calculateTessBuildingsInBudget, getTesseractCost } from './mechanics/tesseractBuildings'
export type { BuyUpgradeInput, UpgradeTier } from './mechanics/upgrades'
export { buyUpgrades } from './mechanics/upgrades'
export type {
  AcceleratorState,
  AscendBuildingState,
  BuyAmount,
  CrystalUpgradesState,
  CubeBlessings,
  HypercubeBlessings,
  MultiplierState,
  ParticleBuildingsState,
  PlatonicBlessings,
  ProducerFamilyState,
  TesseractBlessings,
  TesseractBuildingsState,
  UpgradesState
} from './state/schema'
