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
export type { ComputeAchievementPointsInput } from './mechanics/achievementPoints'
export {
  ambrosiaCountPoints,
  antMasteryPoints,
  computeAchievementPoints,
  exaltPoints,
  freeRuneLevelPoints,
  getAchievementQuarks,
  maxedUpgradeFamilyPoints,
  rebornELOPoints,
  redAmbrosiaCountPoints,
  runeLevelPoints,
  singularityCountPoints,
  talismanRarityPoints
} from './mechanics/achievementPoints'
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
export type {
  AntMasteryData,
  CanBuyAntMasteryInput,
  SelfSpeedFromMasteryInput
} from './mechanics/antMasteries'
export {
  antMasteryData,
  calculateSelfSpeedFromMastery,
  canBuyAntMastery,
  getBuyableAntMasteryLevels,
  MAX_ANT_MASTERY_LEVEL
} from './mechanics/antMasteries'
export type {
  AvailableRebornELOInput,
  RebornELOStageModifiers,
  RebornELOStageModifiersInput,
  RebornELOTranche,
  SingularityPerkELOInput,
  TotalProductionForRebornELOInput
} from './mechanics/antRebornELO'
export {
  calculateAvailableRebornELO,
  calculateLeftoverRebornELO,
  calculateRebornELOThresholds,
  calculateSingularityPerkELO,
  calculateStageRebornSpeedMult,
  calculateToNextRebornELOThreshold,
  calculateTotalProductionForRebornELO,
  perRebornELOStageModifiers,
  quarkMultiplierPerRebornELOThreshold,
  rebornELOStageModifiers,
  rebornELOThresholdTranches,
  rebornSpeedPerkLevels,
  singularityELOBonusMult,
  singularityELOBonusMultLevels,
  singularityPerkELOLevels,
  singularityRebornSpeedMultModifier
} from './mechanics/antRebornELO'
export type {
  AntSacrificeTalismanItemInput,
  TalismanCraftItem
} from './mechanics/antSacrificeRewards'
export {
  calculateAntSacrificeTalismanItem,
  talismanItemRequiredELO,
  talismanRewardMultipliers
} from './mechanics/antSacrificeRewards'
export type {
  AcceleratorBoostsAntUpgradeEffect,
  AntELOAntUpgradeEffect,
  AntELOAntUpgradeInput,
  AntSacrificeAntUpgradeEffect,
  AntSpeedAntUpgradeEffect,
  AntSpeedAntUpgradeInput,
  AntUpgradeCostInput,
  AntUpgradeMaxCostInput,
  AntUpgradeMaxPurchasableInput,
  AscensionScoreAntUpgradeEffect,
  BuildingCostScaleAntUpgradeEffect,
  CoinsAntUpgradeEffect,
  CoinsAntUpgradeInput,
  FreeRunesAntUpgradeEffect,
  MortuusAntUpgradeEffect,
  Mortuus2AntUpgradeEffect,
  MultipliersAntUpgradeEffect,
  ObtainiumAntUpgradeEffect,
  OfferingsAntUpgradeEffect,
  SalvageAntUpgradeEffect,
  TaxesAntUpgradeEffect,
  WowCubesAntUpgradeEffect
} from './mechanics/antUpgrades'
export {
  acceleratorBoostsAntUpgradeEffect,
  antELOAntUpgradeEffect,
  antSacrificeAntUpgradeEffect,
  antSpeedAntUpgradeEffect,
  antUpgradeBaseCosts,
  antUpgradeCostIncreaseExponents,
  ascensionScoreAntUpgradeEffect,
  buildingCostScaleAntUpgradeEffect,
  coinsAntUpgradeEffect,
  freeRunesAntUpgradeEffect,
  getCostMaxAntUpgrades,
  getCostNextAntUpgrade,
  getMaxPurchasableAntUpgrades,
  mortuusAntUpgradeEffect,
  mortuus2AntUpgradeEffect,
  multipliersAntUpgradeEffect,
  obtainiumAntUpgradeEffect,
  offeringsAntUpgradeEffect,
  salvageAntUpgradeEffect,
  taxesAntUpgradeEffect,
  wowCubesAntUpgradeEffect
} from './mechanics/antUpgrades'
export type { CalculateAscensionCountInput } from './mechanics/ascensions'
export { calculateAscensionCount } from './mechanics/ascensions'
export type { AmbrosiaUpgradeNames, AmbrosiaUpgradeRewards } from './mechanics/blueberryUpgrades'
export {
  ambrosiaBaseObtainium1CostFormula,
  ambrosiaBaseObtainium1Effect,
  ambrosiaBaseObtainium2CostFormula,
  ambrosiaBaseObtainium2Effect,
  ambrosiaBaseOffering1CostFormula,
  ambrosiaBaseOffering1Effect,
  ambrosiaBaseOffering2CostFormula,
  ambrosiaBaseOffering2Effect,
  ambrosiaBrickOfLeadCostFormula,
  ambrosiaBrickOfLeadEffect,
  ambrosiaCubeLuck1CostFormula,
  ambrosiaCubeLuck1Effect,
  ambrosiaCubeQuark1CostFormula,
  ambrosiaCubeQuark1Effect,
  ambrosiaCubes1CostFormula,
  ambrosiaCubes1Effect,
  ambrosiaCubes2CostFormula,
  ambrosiaCubes2Effect,
  ambrosiaCubes3CostFormula,
  ambrosiaCubes3Effect,
  ambrosiaFreeGenerationUpgradesCostFormula,
  ambrosiaFreeGenerationUpgradesEffect,
  ambrosiaFreeLuckUpgradesCostFormula,
  ambrosiaFreeLuckUpgradesEffect,
  ambrosiaFreeQuarkUpgradesCostFormula,
  ambrosiaFreeQuarkUpgradesEffect,
  ambrosiaFreeRedLuckUpgradesCostFormula,
  ambrosiaFreeRedLuckUpgradesEffect,
  ambrosiaHyperfluxCostFormula,
  ambrosiaHyperfluxEffect,
  ambrosiaInfiniteShopUpgrades1CostFormula,
  ambrosiaInfiniteShopUpgrades1Effect,
  ambrosiaInfiniteShopUpgrades2CostFormula,
  ambrosiaInfiniteShopUpgrades2Effect,
  ambrosiaLuck1CostFormula,
  ambrosiaLuck1Effect,
  ambrosiaLuck2CostFormula,
  ambrosiaLuck2Effect,
  ambrosiaLuck3CostFormula,
  ambrosiaLuck3Effect,
  ambrosiaLuck4CostFormula,
  ambrosiaLuck4Effect,
  ambrosiaLuckCube1CostFormula,
  ambrosiaLuckCube1Effect,
  ambrosiaLuckQuark1CostFormula,
  ambrosiaLuckQuark1Effect,
  ambrosiaObtainium1CostFormula,
  ambrosiaObtainium1Effect,
  ambrosiaOffering1CostFormula,
  ambrosiaOffering1Effect,
  ambrosiaPatreonCostFormula,
  ambrosiaPatreonEffect,
  ambrosiaQuarkCube1CostFormula,
  ambrosiaQuarkCube1Effect,
  ambrosiaQuarkLuck1CostFormula,
  ambrosiaQuarkLuck1Effect,
  ambrosiaQuarks1CostFormula,
  ambrosiaQuarks1Effect,
  ambrosiaQuarks2CostFormula,
  ambrosiaQuarks2Effect,
  ambrosiaQuarks3CostFormula,
  ambrosiaQuarks3Effect,
  ambrosiaRuneOOMBonusCostFormula,
  ambrosiaRuneOOMBonusEffect,
  ambrosiaSingReduction1CostFormula,
  ambrosiaSingReduction1Effect,
  ambrosiaSingReduction2CostFormula,
  ambrosiaSingReduction2Effect,
  ambrosiaTalismanBonusRuneLevelCostFormula,
  ambrosiaTalismanBonusRuneLevelEffect,
  ambrosiaTutorialCostFormula,
  ambrosiaTutorialEffect
} from './mechanics/blueberryUpgrades'
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
export type { CampaignTutorialBonus } from './mechanics/campaignTokenRewards'
export {
  campaignAmbrosiaLuckBonus,
  campaignAscensionScoreMultiplier,
  campaignBlueberrySpeedBonus,
  campaignBonusRune6,
  campaignC15Bonus,
  campaignCubeBonus,
  campaignGoldenQuarkBonus,
  campaignObtainiumBonus,
  campaignOcteractBonus,
  campaignOfferingBonus,
  campaignQuarkBonus,
  campaignTaxMultiplier,
  campaignTimeThresholdReduction,
  tutorialBonus as campaignTutorialBonus
} from './mechanics/campaignTokenRewards'
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
  CorruptionRawMultiplierInput,
  DroughtEffectInput,
  HyperchallengeEffectInput,
  IlliteracyEffectInput,
  MaxCorruptionLevelInput,
  ViscosityEffectInput
} from './mechanics/corruptions'
export {
  calculateCorruptionDifficultyScore,
  calculateCorruptionRawMultiplier,
  clipCorruptionLevel,
  corruptionScoreMults,
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
export type { GoldenQuarkUpgradeRewards, SingularityDataKeys } from './mechanics/goldenQuarkUpgrades'
export {
  advancedPackEffect,
  ascensionsEffect as gqAscensionsEffect,
  blueberriesEffect as gqBlueberriesEffect,
  cookies2Effect,
  cookies3Effect,
  cookies4Effect,
  cookies5Effect,
  cookiesEffect,
  corruptionFifteenEffect,
  corruptionFourteenEffect,
  divinePackEffect,
  expertPackEffect,
  favoriteUpgradeEffect,
  goldenQuarks1Effect,
  goldenQuarks2Effect,
  goldenQuarks3Effect,
  halfMindEffect,
  intermediatePackEffect,
  masterPackEffect,
  octeractUnlockEffect,
  offeringAutomaticEffect,
  oneMindEffect,
  platonicAlphaEffect,
  platonicDeltaEffect,
  platonicPhiEffect,
  platonicTauEffect,
  potionBuff2Effect,
  potionBuff3Effect,
  potionBuffEffect,
  singAmbrosiaGeneration2Effect as gqSingAmbrosiaGeneration2Effect,
  singAmbrosiaGeneration3Effect as gqSingAmbrosiaGeneration3Effect,
  singAmbrosiaGeneration4Effect as gqSingAmbrosiaGeneration4Effect,
  singAmbrosiaGenerationEffect as gqSingAmbrosiaGenerationEffect,
  singAmbrosiaLuck2Effect as gqSingAmbrosiaLuck2Effect,
  singAmbrosiaLuck3Effect as gqSingAmbrosiaLuck3Effect,
  singAmbrosiaLuck4Effect as gqSingAmbrosiaLuck4Effect,
  singAmbrosiaLuckEffect as gqSingAmbrosiaLuckEffect,
  singAscensionSpeed2Effect,
  singAscensionSpeedEffect,
  singBonusTokens1Effect,
  singBonusTokens2Effect,
  singBonusTokens3Effect,
  singBonusTokens4Effect,
  singChallengeExtension2Effect,
  singChallengeExtension3Effect,
  singChallengeExtensionEffect,
  singCitadel2Effect,
  singCitadelEffect,
  singCubes1Effect,
  singCubes2Effect,
  singCubes3Effect,
  singFastForward2Effect,
  singFastForwardEffect,
  singInfiniteShopUpgradesEffect as gqSingInfiniteShopUpgradesEffect,
  singObtainium1Effect,
  singObtainium2Effect,
  singObtainium3Effect,
  singOcteractGain2Effect,
  singOcteractGain3Effect,
  singOcteractGain4Effect,
  singOcteractGain5Effect,
  singOcteractGainEffect,
  singOcteractPatreonBonusEffect,
  singOfferings1Effect,
  singOfferings2Effect,
  singOfferings3Effect,
  singQuarkHepteract2Effect,
  singQuarkHepteract3Effect,
  singQuarkHepteractEffect,
  singQuarkImprover1Effect,
  singTalismanBonusRunes1Effect,
  singTalismanBonusRunes2Effect,
  singTalismanBonusRunes3Effect,
  singTalismanBonusRunes4Effect,
  starterPackEffect,
  ultimatePenEffect,
  wowPass2Effect,
  wowPass3Effect,
  wowPass4Effect,
  wowPassEffect
} from './mechanics/goldenQuarkUpgrades'
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
export type { OcteractUpgradeRewards, OcteractUpgrades } from './mechanics/octeracts'
export {
  octeractAmbrosiaGeneration2CostFormula,
  octeractAmbrosiaGeneration2Effect,
  octeractAmbrosiaGeneration3CostFormula,
  octeractAmbrosiaGeneration3Effect,
  octeractAmbrosiaGeneration4CostFormula,
  octeractAmbrosiaGeneration4Effect,
  octeractAmbrosiaGenerationCostFormula,
  octeractAmbrosiaGenerationEffect,
  octeractAmbrosiaLuck2CostFormula,
  octeractAmbrosiaLuck2Effect,
  octeractAmbrosiaLuck3CostFormula,
  octeractAmbrosiaLuck3Effect,
  octeractAmbrosiaLuck4CostFormula,
  octeractAmbrosiaLuck4Effect,
  octeractAmbrosiaLuckCostFormula,
  octeractAmbrosiaLuckEffect,
  octeractAscensions2CostFormula,
  octeractAscensions2Effect,
  octeractAscensionsCostFormula,
  octeractAscensionsEffect,
  octeractAscensionsOcteractGainCostFormula,
  octeractAscensionsOcteractGainEffect,
  octeractAutoPotionEfficiencyCostFormula,
  octeractAutoPotionEfficiencyEffect,
  octeractAutoPotionSpeedCostFormula,
  octeractAutoPotionSpeedEffect,
  octeractBlueberriesCostFormula,
  octeractBlueberriesEffect,
  octeractBonusTokens1CostFormula,
  octeractBonusTokens1Effect,
  octeractBonusTokens2CostFormula,
  octeractBonusTokens2Effect,
  octeractBonusTokens3CostFormula,
  octeractBonusTokens3Effect,
  octeractBonusTokens4CostFormula,
  octeractBonusTokens4Effect,
  octeractCorruptionCostFormula,
  octeractCorruptionEffect,
  octeractExportQuarksCostFormula,
  octeractExportQuarksEffect,
  octeractFastForwardCostFormula,
  octeractFastForwardEffect,
  octeractGain2CostFormula,
  octeractGain2Effect,
  octeractGainCostFormula,
  octeractGainEffect,
  octeractGQCostReduceCostFormula,
  octeractGQCostReduceEffect,
  octeractImprovedAscensionSpeed2CostFormula,
  octeractImprovedAscensionSpeed2Effect,
  octeractImprovedAscensionSpeedCostFormula,
  octeractImprovedAscensionSpeedEffect,
  octeractImprovedDaily2CostFormula,
  octeractImprovedDaily2Effect,
  octeractImprovedDaily3CostFormula,
  octeractImprovedDaily3Effect,
  octeractImprovedDailyCostFormula,
  octeractImprovedDailyEffect,
  octeractImprovedFree2CostFormula,
  octeractImprovedFree2Effect,
  octeractImprovedFree3CostFormula,
  octeractImprovedFree3Effect,
  octeractImprovedFree4CostFormula,
  octeractImprovedFree4Effect,
  octeractImprovedFreeCostFormula,
  octeractImprovedFreeEffect,
  octeractImprovedGlobalSpeedCostFormula,
  octeractImprovedGlobalSpeedEffect,
  octeractImprovedQuarkHeptCostFormula,
  octeractImprovedQuarkHeptEffect,
  octeractInfiniteShopUpgradesCostFormula,
  octeractInfiniteShopUpgradesEffect,
  octeractObtainium1CostFormula,
  octeractObtainium1Effect,
  octeractOfferings1CostFormula,
  octeractOfferings1Effect,
  octeractOneMindImproverCostFormula,
  octeractOneMindImproverEffect,
  octeractQuarkGain2CostFormula,
  octeractQuarkGain2Effect,
  octeractQuarkGainCostFormula,
  octeractQuarkGainEffect,
  octeractSingUpgradeCapCostFormula,
  octeractSingUpgradeCapEffect,
  octeractStarterCostFormula,
  octeractStarterEffect,
  octeractTalismanLevelCap1CostFormula,
  octeractTalismanLevelCap1Effect,
  octeractTalismanLevelCap2CostFormula,
  octeractTalismanLevelCap2Effect,
  octeractTalismanLevelCap3CostFormula,
  octeractTalismanLevelCap3Effect,
  octeractTalismanLevelCap4CostFormula,
  octeractTalismanLevelCap4Effect
} from './mechanics/octeracts'
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
export type {
  CheckPlatonicUpgradeInput,
  PlatonicResourceKey,
  PlatonicUpgradeAffordability,
  PlatonicUpgradeBaseCost,
  PlatonicUpgradePriceMultiplierInput
} from './mechanics/platonicUpgradeCosts'
export {
  checkPlatonicUpgradeAffordability,
  platonicResources,
  platonicUpgradeBaseCosts,
  platonicUpgradePriceMultiplier
} from './mechanics/platonicUpgradeCosts'
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
export type { RedAmbrosiaNames, RedAmbrosiaUpgradeRewards } from './mechanics/redAmbrosiaUpgrades'
export {
  blueberriesCostFormula,
  blueberriesEffect,
  blueberryGenerationSpeed2CostFormula,
  blueberryGenerationSpeed2Effect,
  blueberryGenerationSpeedCostFormula,
  blueberryGenerationSpeedEffect,
  conversionImprovement1CostFormula,
  conversionImprovement1Effect,
  conversionImprovement2CostFormula,
  conversionImprovement2Effect,
  conversionImprovement3CostFormula,
  conversionImprovement3Effect,
  freeCubeUpgradesCostFormula,
  freeCubeUpgradesEffect,
  freeLevelsRow2CostFormula,
  freeLevelsRow2Effect,
  freeLevelsRow3CostFormula,
  freeLevelsRow3Effect,
  freeLevelsRow4CostFormula,
  freeLevelsRow4Effect,
  freeLevelsRow5CostFormula,
  freeLevelsRow5Effect,
  freeObtainiumUpgradesCostFormula,
  freeObtainiumUpgradesEffect,
  freeOfferingUpgradesCostFormula,
  freeOfferingUpgradesEffect,
  freeSpeedUpgradesCostFormula,
  freeSpeedUpgradesEffect,
  freeTutorialLevelsCostFormula,
  freeTutorialLevelsEffect,
  infiniteShopUpgradesCostFormula,
  infiniteShopUpgradesEffect,
  redAmbrosiaAcceleratorCostFormula,
  redAmbrosiaAcceleratorEffect,
  redAmbrosiaCubeCostFormula,
  redAmbrosiaCubeEffect,
  redAmbrosiaCubeImproverCostFormula,
  redAmbrosiaCubeImproverEffect,
  redAmbrosiaFreeAccumulatorCostFormula,
  redAmbrosiaFreeAccumulatorEffect,
  redAmbrosiaObtainiumCostFormula,
  redAmbrosiaObtainiumEffect,
  redAmbrosiaOfferingCostFormula,
  redAmbrosiaOfferingEffect,
  redGenerationSpeedCostFormula,
  redGenerationSpeedEffect,
  redLuckCostFormula,
  redLuckEffect,
  regularLuck2CostFormula,
  regularLuck2Effect,
  regularLuckCostFormula,
  regularLuckEffect,
  salvageYinYangCostFormula,
  salvageYinYangEffect,
  tutorialCostFormula,
  tutorialEffect,
  viscountCostFormula,
  viscountEffect
} from './mechanics/redAmbrosiaUpgrades'
export type { RangeLevelAndCost } from './mechanics/researches'
export {
  polyBuyToLevel,
  polyCostForLevels,
  researchBaseCosts,
  researchLevelCostRanges,
  researchMaxLevels
} from './mechanics/researches'
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
export type {
  BonusRuneLevelsDuplicationInput,
  BonusRuneLevelsInfiniteAscentInput,
  BonusRuneLevelsSpeedInput,
  DuplicationRuneOOMIncreaseInput,
  FirstFiveFreeLevelsInput,
  PrismRuneOOMIncreaseInput,
  SpeedRuneOOMIncreaseInput,
  SuperiorIntellectRuneOOMIncreaseInput,
  ThriftRuneOOMIncreaseInput
} from './mechanics/runeLevelBonuses'
export {
  bonusRuneLevelsDuplication,
  bonusRuneLevelsInfiniteAscent,
  bonusRuneLevelsSpeed,
  duplicationRuneOOMIncrease,
  firstFiveFreeLevels,
  prismRuneOOMIncrease,
  speedRuneOOMIncrease,
  superiorIntellectRuneOOMIncrease,
  thriftRuneOOMIncrease
} from './mechanics/runeLevelBonuses'
export type { MaxRuneLevelPurchaseInput, MaxRuneLevelPurchaseResult } from './mechanics/runeLevels'
export {
  maxRuneLevelPurchase,
  runeEXPLeftToLevel,
  runeEXPToLevel,
  runeLevelFromEXP,
  runeOfferingsToLevel
} from './mechanics/runeLevels'
export type {
  DuplicationRuneSpiritEffects,
  PrismRuneSpiritEffects,
  SpeedRuneSpiritEffects,
  SuperiorIntellectRuneSpiritEffects,
  ThriftRuneSpiritEffects
} from './mechanics/runeSpiritEffects'
export {
  duplicationRuneSpiritEffects,
  prismRuneSpiritEffects,
  speedRuneSpiritEffects,
  superiorIntellectRuneSpiritEffects,
  thriftRuneSpiritEffects
} from './mechanics/runeSpiritEffects'
export type {
  MaxRuneUpgradePurchaseInput,
  MaxRuneUpgradePurchaseResult
} from './mechanics/runeUpgradeProgression'
export {
  maxRuneUpgradePurchase,
  runeUpgradeEXPLeftToLevel,
  runeUpgradeEXPToLevel,
  runeUpgradeLevelFromEXP
} from './mechanics/runeUpgradeProgression'
export type { ShopCostInput } from './mechanics/shopCosts'
export { shopCost } from './mechanics/shopCosts'
export type { QuarkShopUpgradeRewards, ShopPanthemaBonusLevels, ShopUpgradeNames } from './mechanics/shopUpgrades'
export {
  antSpeedEffect as shopAntSpeedEffect,
  autoWarpEffect as shopAutoWarpEffect,
  calculator2Effect as shopCalculator2Effect,
  calculator3Effect as shopCalculator3Effect,
  calculator4Effect as shopCalculator4Effect,
  calculator5Effect as shopCalculator5Effect,
  calculator6Effect as shopCalculator6Effect,
  calculator7Effect as shopCalculator7Effect,
  calculatorEffect as shopCalculatorEffect,
  cashGrab2Effect as shopCashGrab2Effect,
  cashGrabEffect as shopCashGrabEffect,
  challenge15AutoEffect as shopChallenge15AutoEffect,
  challengeExtensionEffect as shopChallengeExtensionEffect,
  challengeTome2Effect as shopChallengeTome2Effect,
  challengeTomeEffect as shopChallengeTomeEffect,
  chronometer2Effect as shopChronometer2Effect,
  chronometer3Effect as shopChronometer3Effect,
  chronometerEffect as shopChronometerEffect,
  chronometerInfinityEffect as shopChronometerInfinityEffect,
  chronometerZEffect as shopChronometerZEffect,
  constantEXEffect as shopConstantEXEffect,
  cubeToQuarkAllEffect as shopCubeToQuarkAllEffect,
  cubeToQuarkEffect as shopCubeToQuarkEffect,
  extraWarpEffect as shopExtraWarpEffect,
  hypercubeToQuarkEffect as shopHypercubeToQuarkEffect,
  improveQuarkHept2Effect as shopImproveQuarkHept2Effect,
  improveQuarkHept3Effect as shopImproveQuarkHept3Effect,
  improveQuarkHept4Effect as shopImproveQuarkHept4Effect,
  improveQuarkHept5Effect as shopImproveQuarkHept5Effect,
  improveQuarkHeptEffect as shopImproveQuarkHeptEffect,
  infiniteAscentEffect as shopInfiniteAscentEffect,
  instantChallenge2Effect as shopInstantChallenge2Effect,
  instantChallengeEffect as shopInstantChallengeEffect,
  obtainiumAutoEffect as shopObtainiumAutoEffect,
  obtainiumEX2Effect as shopObtainiumEX2Effect,
  obtainiumEX3Effect as shopObtainiumEX3Effect,
  obtainiumEXEffect as shopObtainiumEXEffect,
  obtainiumPotionEffect as shopObtainiumPotionEffect,
  offeringAutoEffect as shopOfferingAutoEffect,
  offeringEX2Effect as shopOfferingEX2Effect,
  offeringEX3Effect as shopOfferingEX3Effect,
  offeringEXEffect as shopOfferingEXEffect,
  offeringPotionEffect as shopOfferingPotionEffect,
  powderAutoEffect as shopPowderAutoEffect,
  powderEXEffect as shopPowderEXEffect,
  seasonPass2Effect as shopSeasonPass2Effect,
  seasonPass3Effect as shopSeasonPass3Effect,
  seasonPassEffect as shopSeasonPassEffect,
  seasonPassInfinityEffect as shopSeasonPassInfinityEffect,
  seasonPassLostEffect as shopSeasonPassLostEffect,
  seasonPassYEffect as shopSeasonPassYEffect,
  seasonPassZEffect as shopSeasonPassZEffect,
  shopAmbrosiaAcceleratorEffect,
  shopAmbrosiaGeneration1Effect,
  shopAmbrosiaGeneration2Effect,
  shopAmbrosiaGeneration3Effect,
  shopAmbrosiaGeneration4Effect,
  shopAmbrosiaLuck1Effect,
  shopAmbrosiaLuck2Effect,
  shopAmbrosiaLuck3Effect,
  shopAmbrosiaLuck4Effect,
  shopAmbrosiaLuckMultiplier4Effect,
  shopAmbrosiaUltraEffect,
  shopCashGrabUltraEffect,
  shopChronometerSEffect,
  shopEXUltraEffect,
  shopHorseShoeEffect,
  shopImprovedDaily2Effect,
  shopImprovedDaily3Effect,
  shopImprovedDaily4Effect,
  shopImprovedDailyEffect,
  shopInfiniteShopUpgradesEffect,
  shopOcteractAmbrosiaLuckEffect,
  shopPanthemaEffect,
  shopRedLuck1Effect,
  shopRedLuck2Effect,
  shopRedLuck3Effect,
  shopSadisticRuneEffect,
  shopSingularityPenaltyDebuffEffect,
  shopSingularityPotencyEffect,
  shopSingularitySpeedupEffect,
  shopTalismanEffect,
  tesseractToQuarkEffect as shopTesseractToQuarkEffect
} from './mechanics/shopUpgrades'
export type { SingularityChallengeDataKeys, SingularityChallengeRewards } from './mechanics/singularityChallenges'
export {
  limitedAscensionsAchievementPointValue,
  limitedAscensionsEffect,
  limitedAscensionsSingularityRequirement,
  limitedTimeAchievementPointValue,
  limitedTimeEffect,
  limitedTimeSingularityRequirement,
  noAmbrosiaUpgradesAchievementPointValue,
  noAmbrosiaUpgradesEffect,
  noAmbrosiaUpgradesSingularityRequirement,
  noOcteractsAchievementPointValue,
  noOcteractsEffect,
  noOcteractsSingularityRequirement,
  noQuarkUpgradesAchievementPointValue,
  noQuarkUpgradesEffect,
  noQuarkUpgradesSingularityRequirement,
  noSingularityUpgradesAchievementPointValue,
  noSingularityUpgradesEffect,
  noSingularityUpgradesSingularityRequirement,
  oneChallengeCapAchievementPointValue,
  oneChallengeCapEffect,
  oneChallengeCapSingularityRequirement,
  sadisticPrequelAchievementPointValue,
  sadisticPrequelEffect,
  sadisticPrequelSingularityRequirement,
  taxmanLastStandAchievementPointValue,
  taxmanLastStandEffect,
  taxmanLastStandSingularityRequirement
} from './mechanics/singularityChallenges'
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
  AffordableTalismanLevelInput,
  ComputeTalismanRarityInput,
  LevelsUntilTalismanRarityIncreaseInput
} from './mechanics/talismanLevels'
export {
  affordableTalismanLevel,
  computeTalismanRarity,
  levelsUntilTalismanRarityIncrease,
  rarityValues,
  sumOfTalismanRarities
} from './mechanics/talismanLevels'
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
