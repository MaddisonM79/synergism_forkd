import {
  actualOcteractUpgradeTotalLevels as logicActualOcteractUpgradeTotalLevels,
  octeractAmbrosiaGeneration2CostFormula as logicOctAmbGen2Cost,
  octeractAmbrosiaGeneration2Effect as logicOctAmbGen2Effect,
  octeractAmbrosiaGeneration3CostFormula as logicOctAmbGen3Cost,
  octeractAmbrosiaGeneration3Effect as logicOctAmbGen3Effect,
  octeractAmbrosiaGeneration4CostFormula as logicOctAmbGen4Cost,
  octeractAmbrosiaGeneration4Effect as logicOctAmbGen4Effect,
  octeractAmbrosiaGenerationCostFormula as logicOctAmbGenCost,
  octeractAmbrosiaGenerationEffect as logicOctAmbGenEffect,
  octeractAmbrosiaLuck2CostFormula as logicOctAmbLuck2Cost,
  octeractAmbrosiaLuck2Effect as logicOctAmbLuck2Effect,
  octeractAmbrosiaLuck3CostFormula as logicOctAmbLuck3Cost,
  octeractAmbrosiaLuck3Effect as logicOctAmbLuck3Effect,
  octeractAmbrosiaLuck4CostFormula as logicOctAmbLuck4Cost,
  octeractAmbrosiaLuck4Effect as logicOctAmbLuck4Effect,
  octeractAmbrosiaLuckCostFormula as logicOctAmbLuckCost,
  octeractAmbrosiaLuckEffect as logicOctAmbLuckEffect,
  octeractAscensions2CostFormula as logicOctAscensions2Cost,
  octeractAscensions2Effect as logicOctAscensions2Effect,
  octeractAscensionsCostFormula as logicOctAscensionsCost,
  octeractAscensionsEffect as logicOctAscensionsEffect,
  octeractAscensionsOcteractGainCostFormula as logicOctAscOctGainCost,
  octeractAscensionsOcteractGainEffect as logicOctAscOctGainEffect,
  octeractAutoPotionEfficiencyCostFormula as logicOctAutoPotionEffCost,
  octeractAutoPotionEfficiencyEffect as logicOctAutoPotionEffEffect,
  octeractAutoPotionSpeedCostFormula as logicOctAutoPotionSpeedCost,
  octeractAutoPotionSpeedEffect as logicOctAutoPotionSpeedEffect,
  octeractBlueberriesCostFormula as logicOctBlueberriesCost,
  octeractBlueberriesEffect as logicOctBlueberriesEffect,
  octeractBonusTokens1CostFormula as logicOctBonusTokens1Cost,
  octeractBonusTokens1Effect as logicOctBonusTokens1Effect,
  octeractBonusTokens2CostFormula as logicOctBonusTokens2Cost,
  octeractBonusTokens2Effect as logicOctBonusTokens2Effect,
  octeractBonusTokens3CostFormula as logicOctBonusTokens3Cost,
  octeractBonusTokens3Effect as logicOctBonusTokens3Effect,
  octeractBonusTokens4CostFormula as logicOctBonusTokens4Cost,
  octeractBonusTokens4Effect as logicOctBonusTokens4Effect,
  octeractCorruptionCostFormula as logicOctCorruptionCost,
  octeractCorruptionEffect as logicOctCorruptionEffect,
  octeractExportQuarksCostFormula as logicOctExportQuarksCost,
  octeractExportQuarksEffect as logicOctExportQuarksEffect,
  octeractFastForwardCostFormula as logicOctFastForwardCost,
  octeractFastForwardEffect as logicOctFastForwardEffect,
  octeractFreeLevelMultiplier as logicOcteractFreeLevelMultiplier,
  octeractFreeLevelSoftcap as logicOcteractFreeLevelSoftcap,
  octeractGain2CostFormula as logicOctGain2Cost,
  octeractGain2Effect as logicOctGain2Effect,
  octeractGainCostFormula as logicOctGainCost,
  octeractGainEffect as logicOctGainEffect,
  octeractGQCostReduceCostFormula as logicOctGQCostReduceCost,
  octeractGQCostReduceEffect as logicOctGQCostReduceEffect,
  octeractImprovedAscensionSpeed2CostFormula as logicOctImprAscSpeed2Cost,
  octeractImprovedAscensionSpeed2Effect as logicOctImprAscSpeed2Effect,
  octeractImprovedAscensionSpeedCostFormula as logicOctImprAscSpeedCost,
  octeractImprovedAscensionSpeedEffect as logicOctImprAscSpeedEffect,
  octeractImprovedDaily2CostFormula as logicOctImprDaily2Cost,
  octeractImprovedDaily2Effect as logicOctImprDaily2Effect,
  octeractImprovedDaily3CostFormula as logicOctImprDaily3Cost,
  octeractImprovedDaily3Effect as logicOctImprDaily3Effect,
  octeractImprovedDailyCostFormula as logicOctImprDailyCost,
  octeractImprovedDailyEffect as logicOctImprDailyEffect,
  octeractImprovedFree2CostFormula as logicOctImprFree2Cost,
  octeractImprovedFree2Effect as logicOctImprFree2Effect,
  octeractImprovedFree3CostFormula as logicOctImprFree3Cost,
  octeractImprovedFree3Effect as logicOctImprFree3Effect,
  octeractImprovedFree4CostFormula as logicOctImprFree4Cost,
  octeractImprovedFree4Effect as logicOctImprFree4Effect,
  octeractImprovedFreeCostFormula as logicOctImprFreeCost,
  octeractImprovedFreeEffect as logicOctImprFreeEffect,
  octeractImprovedGlobalSpeedCostFormula as logicOctImprGlobalSpeedCost,
  octeractImprovedGlobalSpeedEffect as logicOctImprGlobalSpeedEffect,
  octeractImprovedQuarkHeptCostFormula as logicOctImprQuarkHeptCost,
  octeractImprovedQuarkHeptEffect as logicOctImprQuarkHeptEffect,
  octeractInfiniteShopUpgradesCostFormula as logicOctInfShopCost,
  octeractInfiniteShopUpgradesEffect as logicOctInfShopEffect,
  octeractObtainium1CostFormula as logicOctObtainium1Cost,
  octeractObtainium1Effect as logicOctObtainium1Effect,
  octeractOfferings1CostFormula as logicOctOfferings1Cost,
  octeractOfferings1Effect as logicOctOfferings1Effect,
  octeractOneMindImproverCostFormula as logicOctOneMindCost,
  octeractOneMindImproverEffect as logicOctOneMindEffect,
  octeractQuarkGain2CostFormula as logicOctQuarkGain2Cost,
  octeractQuarkGain2Effect as logicOctQuarkGain2Effect,
  octeractQuarkGainCostFormula as logicOctQuarkGainCost,
  octeractQuarkGainEffect as logicOctQuarkGainEffect,
  octeractSingUpgradeCapCostFormula as logicOctSingUpgradeCapCost,
  octeractSingUpgradeCapEffect as logicOctSingUpgradeCapEffect,
  octeractStarterCostFormula as logicOctStarterCost,
  octeractStarterEffect as logicOctStarterEffect,
  octeractTalismanLevelCap1CostFormula as logicOctTalismanCap1Cost,
  octeractTalismanLevelCap1Effect as logicOctTalismanCap1Effect,
  octeractTalismanLevelCap2CostFormula as logicOctTalismanCap2Cost,
  octeractTalismanLevelCap2Effect as logicOctTalismanCap2Effect,
  octeractTalismanLevelCap3CostFormula as logicOctTalismanCap3Cost,
  octeractTalismanLevelCap3Effect as logicOctTalismanCap3Effect,
  octeractTalismanLevelCap4CostFormula as logicOctTalismanCap4Cost,
  octeractTalismanLevelCap4Effect as logicOctTalismanCap4Effect,
  type OcteractUpgradeRewards as LogicOcteractUpgradeRewards,
  type OcteractUpgrades as LogicOcteractUpgrades
} from '@synergism/logic'
import i18next from 'i18next'
import { DOMCacheGetOrSet } from './Cache/DOM'
import { calculateOcteractMultiplier } from './Calculate'
import { updateMaxTokens, updateTokens } from './Campaign'
import { hepteracts } from './Hepteracts'
import { format, formatAsPercentIncrease, formatTimeShort, player } from './Synergism'
import { Alert, Prompt } from './UpdateHTML'
import { isMobile } from './Utility'

// Re-exported from @synergism/logic so existing call sites that import these
// types from this module keep compiling unchanged.
type OcteractUpgradeRewards = LogicOcteractUpgradeRewards
export type OcteractUpgrades = LogicOcteractUpgrades

interface OcteractUpgrade<T extends OcteractUpgrades, K extends keyof OcteractUpgradeRewards[T]> {
  level: number
  freeLevel: number
  octeractsInvested: number
  maxLevel: number
  qualityOfLife: boolean
  costPerLevel: number
  costFormula(this: void, level: number, baseCost: number): number
  effect(n: number, key: K): OcteractUpgradeRewards[T][K]
  effectDescription(n: number): string
  name(): string
  description(): string
}

export const octeractUpgrades: {
  [K in OcteractUpgrades]: OcteractUpgrade<K, keyof OcteractUpgradeRewards[K]>
} = {
  octeractStarter: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    maxLevel: 1,
    costPerLevel: 1e-15,
    qualityOfLife: false,
    costFormula: logicOctStarterCost,
    effect: logicOctStarterEffect,
    effectDescription: (n: number) => {
      if (n > 0) {
        const quarkMult = getOcteractUpgradeEffect('octeractStarter', 'quarkMult')
        const octeractMult = getOcteractUpgradeEffect('octeractStarter', 'octeractMult')
        const antSpeedMult = getOcteractUpgradeEffect('octeractStarter', 'antSpeedMult')
        return i18next.t('octeract.data.octeractStarter.effectEnabled', {
          amount: formatAsPercentIncrease(quarkMult, 0),
          amount2: formatAsPercentIncrease(octeractMult, 0),
          amount3: format(antSpeedMult, 0, true)
        })
      } else {
        return i18next.t('octeract.data.octeractStarter.effectDisabled')
      }
    },
    name: () => i18next.t('octeract.data.octeractStarter.name'),
    description: () => i18next.t('octeract.data.octeractStarter.description')
  },
  octeractGain: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    maxLevel: 1e8,
    costPerLevel: 1e-8,
    qualityOfLife: false,
    costFormula: logicOctGainCost,
    effect: logicOctGainEffect,
    effectDescription: function(_n: number) {
      const effectValue = getOcteractUpgradeEffect('octeractGain', 'octeractMult')
      return i18next.t('octeract.data.octeractGain.effect', { n: formatAsPercentIncrease(effectValue, 2) })
    },
    name: () => i18next.t('octeract.data.octeractGain.name'),
    description: () => i18next.t('octeract.data.octeractGain.description')
  },
  octeractGain2: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctGain2Cost,
    maxLevel: -1,
    costPerLevel: 1e10,
    qualityOfLife: false,
    effect: logicOctGain2Effect,
    effectDescription: function(_n: number) {
      const effectValue = getOcteractUpgradeEffect('octeractGain2', 'octeractMult')
      return i18next.t('octeract.data.octeractGain2.effect', { n: formatAsPercentIncrease(effectValue, 2) })
    },
    name: () => i18next.t('octeract.data.octeractGain2.name'),
    description: () => i18next.t('octeract.data.octeractGain2.description')
  },
  octeractQuarkGain: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctQuarkGainCost,
    maxLevel: 20000,
    costPerLevel: 1e-7,
    qualityOfLife: false,
    effect: logicOctQuarkGainEffect,
    effectDescription: function(_n: number) {
      const effectValue = getOcteractUpgradeEffect('octeractQuarkGain', 'quarkMult')
      return i18next.t('octeract.data.octeractQuarkGain.effect', { n: formatAsPercentIncrease(effectValue, 2) })
    },
    name: () => i18next.t('octeract.data.octeractQuarkGain.name'),
    description: () => i18next.t('octeract.data.octeractQuarkGain.description')
  },
  octeractQuarkGain2: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctQuarkGain2Cost,
    maxLevel: 5,
    costPerLevel: 1e22,
    qualityOfLife: false,
    effect: (n: number) => logicOctQuarkGain2Effect(n, octeractUpgrades.octeractQuarkGain.level, hepteracts.quark.BAL),
    effectDescription: (n: number) => {
      if (n > 0) {
        const quarkMult = getOcteractUpgradeEffect('octeractQuarkGain2', 'quarkMult')
        const quarkGain1Levels = octeractUpgrades.octeractQuarkGain.level
        const digits = Math.floor(1 + Math.log10(Math.max(1, hepteracts.quark.BAL)))
        return i18next.t('octeract.data.octeractQuarkGain2.effectEnabled', {
          amount: formatAsPercentIncrease(quarkMult, 2),
          amount2: formatAsPercentIncrease(1 + n / 10000 * Math.floor(quarkGain1Levels / 111), 2),
          amount3: format(digits, 0, true)
        })
      } else {
        return i18next.t('octeract.data.octeractQuarkGain2.effectDisabled')
      }
    },
    name: () => i18next.t('octeract.data.octeractQuarkGain2.name'),
    description: () => i18next.t('octeract.data.octeractQuarkGain2.description')
  },
  octeractCorruption: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctCorruptionCost,
    maxLevel: 2,
    costPerLevel: 10,
    qualityOfLife: false,
    effect: logicOctCorruptionEffect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractCorruption.effect', { n }),
    name: () => i18next.t('octeract.data.octeractCorruption.name'),
    description: () => i18next.t('octeract.data.octeractCorruption.description')
  },
  octeractGQCostReduce: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctGQCostReduceCost,
    maxLevel: 50,
    costPerLevel: 1e-9,
    qualityOfLife: false,
    effect: logicOctGQCostReduceEffect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractGQCostReduce.effect', { n }),
    name: () => i18next.t('octeract.data.octeractGQCostReduce.name'),
    description: () => i18next.t('octeract.data.octeractGQCostReduce.description')
  },
  octeractExportQuarks: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctExportQuarksCost,
    maxLevel: 100,
    costPerLevel: 1,
    qualityOfLife: false,
    effect: logicOctExportQuarksEffect,
    effectDescription: (n: number) =>
      i18next.t('octeract.data.octeractExportQuarks.effect', { n: format(40 * n, 0, true) }),
    name: () => i18next.t('octeract.data.octeractExportQuarks.name'),
    description: () => i18next.t('octeract.data.octeractExportQuarks.description')
  },
  octeractImprovedDaily: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctImprDailyCost,
    maxLevel: 50,
    costPerLevel: 1e-3,
    effect: logicOctImprDailyEffect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractImprovedDaily.effect', { n }),
    name: () => i18next.t('octeract.data.octeractImprovedDaily.name'),
    description: () => i18next.t('octeract.data.octeractImprovedDaily.description'),
    qualityOfLife: true
  },
  octeractImprovedDaily2: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctImprDaily2Cost,
    maxLevel: 50,
    costPerLevel: 1e-2,
    effect: logicOctImprDaily2Effect,
    effectDescription: function(_n: number) {
      const goldenQuarkMult = getOcteractUpgradeEffect('octeractImprovedDaily2', 'goldenQuarkMult')
      return i18next.t('octeract.data.octeractImprovedDaily2.effect', {
        n: formatAsPercentIncrease(goldenQuarkMult, 0)
      })
    },
    name: () => i18next.t('octeract.data.octeractImprovedDaily2.name'),
    description: () => i18next.t('octeract.data.octeractImprovedDaily2.description'),
    qualityOfLife: true
  },
  octeractImprovedDaily3: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctImprDaily3Cost,
    maxLevel: -1,
    costPerLevel: 1e20,
    effect: logicOctImprDaily3Effect,
    effectDescription: (n: number) =>
      i18next.t('octeract.data.octeractImprovedDaily3.effect', { n: `${n} +${0.5 * n}%` }),
    name: () => i18next.t('octeract.data.octeractImprovedDaily3.name'),
    description: () => i18next.t('octeract.data.octeractImprovedDaily3.description'),
    qualityOfLife: true
  },
  octeractImprovedQuarkHept: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctImprQuarkHeptCost,
    maxLevel: 25,
    costPerLevel: 1 / 10,
    effect: logicOctImprQuarkHeptEffect,
    effectDescription: function(_n: number) {
      const quarkHeptExponent = getOcteractUpgradeEffect('octeractImprovedQuarkHept', 'quarkHeptExponent')
      return i18next.t('octeract.data.octeractImprovedQuarkHept.effect', { n: format(quarkHeptExponent, 2, true) })
    },
    name: () => i18next.t('octeract.data.octeractImprovedQuarkHept.name'),
    description: () => i18next.t('octeract.data.octeractImprovedQuarkHept.description'),
    qualityOfLife: false
  },
  octeractImprovedGlobalSpeed: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctImprGlobalSpeedCost,
    maxLevel: 1000,
    costPerLevel: 1e-5,
    effect: (n: number) => logicOctImprGlobalSpeedEffect(n, player.singularityCount),
    effectDescription: (n: number) => {
      const globalSpeedMult = getOcteractUpgradeEffect('octeractImprovedGlobalSpeed', 'globalSpeedMult')
      return i18next.t('octeract.data.octeractImprovedGlobalSpeed.effect', {
        n: format(n, 0, true),
        mult: formatAsPercentIncrease(globalSpeedMult, 0)
      })
    },
    name: () => i18next.t('octeract.data.octeractImprovedGlobalSpeed.name'),
    description: () => i18next.t('octeract.data.octeractImprovedGlobalSpeed.description'),
    qualityOfLife: false
  },
  octeractImprovedAscensionSpeed: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctImprAscSpeedCost,
    maxLevel: 100,
    costPerLevel: 100,
    effect: (n: number) => logicOctImprAscSpeedEffect(n, player.singularityCount),
    effectDescription: (n: number) => {
      const ascensionSpeedMult = getOcteractUpgradeEffect('octeractImprovedAscensionSpeed', 'ascensionSpeedMult')
      return i18next.t('octeract.data.octeractImprovedAscensionSpeed.effect', {
        n: format(n / 20, 2, true),
        mult: formatAsPercentIncrease(ascensionSpeedMult, 2)
      })
    },
    name: () => i18next.t('octeract.data.octeractImprovedAscensionSpeed.name'),
    description: () => i18next.t('octeract.data.octeractImprovedAscensionSpeed.description'),
    qualityOfLife: false
  },
  octeractImprovedAscensionSpeed2: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctImprAscSpeed2Cost,
    maxLevel: 250,
    costPerLevel: 1e5,
    effect: (n: number) => logicOctImprAscSpeed2Effect(n, player.singularityCount),
    effectDescription: (n: number) => {
      const ascensionSpeedMult = getOcteractUpgradeEffect('octeractImprovedAscensionSpeed2', 'ascensionSpeedMult')
      return i18next.t('octeract.data.octeractImprovedAscensionSpeed2.effect', {
        n: format(n / 20, 2, true),
        mult: formatAsPercentIncrease(ascensionSpeedMult, 2)
      })
    },
    name: () => i18next.t('octeract.data.octeractImprovedAscensionSpeed2.name'),
    description: () => i18next.t('octeract.data.octeractImprovedAscensionSpeed2.description'),
    qualityOfLife: false
  },
  octeractImprovedFree: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctImprFreeCost,
    maxLevel: 1,
    costPerLevel: 100,
    effect: logicOctImprFreeEffect,
    effectDescription: (n: number) => {
      if (n > 0) {
        return i18next.t('octeract.data.octeractImprovedFree.effectEnabled')
      } else {
        return i18next.t('octeract.data.octeractImprovedFree.effectDisabled')
      }
    },
    name: () => i18next.t('octeract.data.octeractImprovedFree.name'),
    description: () => {
      const power = 0.6
        + getOcteractUpgradeEffect('octeractImprovedFree2', 'freeLevelPowerIncrease')
        + getOcteractUpgradeEffect('octeractImprovedFree3', 'freeLevelPowerIncrease')
        + getOcteractUpgradeEffect('octeractImprovedFree4', 'freeLevelPowerIncrease')
      return i18next.t('octeract.data.octeractImprovedFree.description', {
        power: format(power, 2, true)
      })
    },
    qualityOfLife: false
  },
  octeractImprovedFree2: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctImprFree2Cost,
    maxLevel: 1,
    costPerLevel: 1e7,
    effect: logicOctImprFree2Effect,
    effectDescription: (n: number) =>
      i18next.t('octeract.data.octeractImprovedFree2.effect', { n: format(n / 20, 2, true) }),
    name: () => i18next.t('octeract.data.octeractImprovedFree2.name'),
    description: () => i18next.t('octeract.data.octeractImprovedFree2.description'),
    qualityOfLife: false
  },
  octeractImprovedFree3: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctImprFree3Cost,
    maxLevel: 1,
    costPerLevel: 1e17,
    effect: logicOctImprFree3Effect,
    effectDescription: (n: number) =>
      i18next.t('octeract.data.octeractImprovedFree3.effect', { n: format(n / 20, 2, true) }),
    name: () => i18next.t('octeract.data.octeractImprovedFree3.name'),
    description: () => i18next.t('octeract.data.octeractImprovedFree3.description'),
    qualityOfLife: false
  },
  octeractImprovedFree4: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctImprFree4Cost,
    maxLevel: 40,
    costPerLevel: 1e20,
    effect: logicOctImprFree4Effect,
    effectDescription: function(_n: number) {
      const freeLevelPowerIncrease = getOcteractUpgradeEffect('octeractImprovedFree4', 'freeLevelPowerIncrease')
      return i18next.t('octeract.data.octeractImprovedFree4.effect', { n: format(freeLevelPowerIncrease, 3, true) })
    },
    name: () => i18next.t('octeract.data.octeractImprovedFree4.name'),
    description: () => i18next.t('octeract.data.octeractImprovedFree4.description'),
    qualityOfLife: false
  },
  octeractSingUpgradeCap: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctSingUpgradeCapCost,
    maxLevel: 10,
    costPerLevel: 1e10,
    effect: logicOctSingUpgradeCapEffect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractSingUpgradeCap.effect', { n }),
    name: () => i18next.t('octeract.data.octeractSingUpgradeCap.name'),
    description: () => i18next.t('octeract.data.octeractSingUpgradeCap.description'),
    qualityOfLife: true
  },
  octeractOfferings1: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctOfferings1Cost,
    maxLevel: -1,
    costPerLevel: 1e-15,
    effect: logicOctOfferings1Effect,
    effectDescription: function(_n: number) {
      const offeringMult = getOcteractUpgradeEffect('octeractOfferings1', 'offeringMult')
      return i18next.t('octeract.data.octeractOfferings1.effect', { n: formatAsPercentIncrease(offeringMult, 2) })
    },
    name: () => i18next.t('octeract.data.octeractOfferings1.name'),
    description: () => i18next.t('octeract.data.octeractOfferings1.description'),
    qualityOfLife: false
  },
  octeractObtainium1: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctObtainium1Cost,
    maxLevel: -1,
    costPerLevel: 1e-15,
    effect: logicOctObtainium1Effect,
    effectDescription: function(_n: number) {
      const obtainiumMult = getOcteractUpgradeEffect('octeractObtainium1', 'obtainiumMult')
      return i18next.t('octeract.data.octeractObtainium1.effect', { n: formatAsPercentIncrease(obtainiumMult, 2) })
    },
    name: () => i18next.t('octeract.data.octeractObtainium1.name'),
    description: () => i18next.t('octeract.data.octeractObtainium1.description'),
    qualityOfLife: false
  },
  octeractAscensions: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAscensionsCost,
    maxLevel: 1000000,
    costPerLevel: 1,
    effect: logicOctAscensionsEffect,
    effectDescription: function(_n: number) {
      const ascensionCountMult = getOcteractUpgradeEffect('octeractAscensions', 'ascensionCountMult')
      return i18next.t('octeract.data.octeractAscensions.effect', {
        n: format((ascensionCountMult - 1) * 100, 1, true)
      })
    },
    name: () => i18next.t('octeract.data.octeractAscensions.name'),
    description: () => i18next.t('octeract.data.octeractAscensions.description'),
    qualityOfLife: false
  },
  octeractAscensions2: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAscensions2Cost,
    maxLevel: -1,
    costPerLevel: 1e12,
    effect: logicOctAscensions2Effect,
    effectDescription: function(_n: number) {
      const ascensionCountMult = getOcteractUpgradeEffect('octeractAscensions2', 'ascensionCountMult')
      return i18next.t('octeract.data.octeractAscensions2.effect', {
        n: format((ascensionCountMult - 1) * 100, 1, true)
      })
    },
    name: () => i18next.t('octeract.data.octeractAscensions2.name'),
    description: () => i18next.t('octeract.data.octeractAscensions2.description'),
    qualityOfLife: false
  },
  octeractAscensionsOcteractGain: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAscOctGainCost,
    maxLevel: -1,
    costPerLevel: 1000,
    effect: (n: number) => logicOctAscOctGainEffect(n, player.ascensionCount),
    effectDescription: (n: number) => {
      const octeractMult = getOcteractUpgradeEffect('octeractAscensionsOcteractGain', 'octeractMult')
      return i18next.t('octeract.data.octeractAscensionsOcteractGain.effect', {
        n: format(n, 1, true),
        mult: formatAsPercentIncrease(octeractMult, 1)
      })
    },
    name: () => i18next.t('octeract.data.octeractAscensionsOcteractGain.name'),
    description: () => i18next.t('octeract.data.octeractAscensionsOcteractGain.description'),
    qualityOfLife: false
  },
  octeractFastForward: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctFastForwardCost,
    maxLevel: 2,
    costPerLevel: 1e8,
    effect: logicOctFastForwardEffect,
    effectDescription: (n: number) =>
      i18next.t('octeract.data.octeractFastForward.effect', { n100: format(2.5 * n, 2, true), n }),
    name: () => i18next.t('octeract.data.octeractFastForward.name'),
    description: () => i18next.t('octeract.data.octeractFastForward.description'),
    qualityOfLife: false
  },
  octeractAutoPotionSpeed: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAutoPotionSpeedCost,
    maxLevel: -1,
    costPerLevel: 1e-10,
    effect: logicOctAutoPotionSpeedEffect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractAutoPotionSpeed.effect', { n: 4 * n }),
    name: () => i18next.t('octeract.data.octeractAutoPotionSpeed.name'),
    description: () => i18next.t('octeract.data.octeractAutoPotionSpeed.description'),
    qualityOfLife: false
  },
  octeractAutoPotionEfficiency: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAutoPotionEffCost,
    maxLevel: 100,
    costPerLevel: 1e-10 * Math.pow(10, 0.5),
    effect: logicOctAutoPotionEffEffect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractAutoPotionEfficiency.effect', { n: 2 * n }),
    name: () => i18next.t('octeract.data.octeractAutoPotionEfficiency.name'),
    description: () => i18next.t('octeract.data.octeractAutoPotionEfficiency.description'),
    qualityOfLife: false
  },
  octeractOneMindImprover: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctOneMindCost,
    maxLevel: 20,
    costPerLevel: 1e25,
    effect: logicOctOneMindEffect,
    effectDescription: function(_n: number) {
      const ascendSpeedExponent = getOcteractUpgradeEffect('octeractOneMindImprover', 'ascendSpeedExponent')
      return i18next.t('octeract.data.octeractOneMindImprover.effect', { n: format(ascendSpeedExponent, 3, true) })
    },
    name: () => i18next.t('octeract.data.octeractOneMindImprover.name'),
    description: () => i18next.t('octeract.data.octeractOneMindImprover.description'),
    qualityOfLife: true
  },
  octeractAmbrosiaLuck: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAmbLuckCost,
    maxLevel: -1,
    costPerLevel: 1e60 / 9,
    effect: logicOctAmbLuckEffect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractAmbrosiaLuck.effect', { n: format(4 * n) }),
    name: () => i18next.t('octeract.data.octeractAmbrosiaLuck.name'),
    description: () => i18next.t('octeract.data.octeractAmbrosiaLuck.description'),
    qualityOfLife: true
  },
  octeractAmbrosiaLuck2: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAmbLuck2Cost,
    maxLevel: 30,
    costPerLevel: 1,
    effect: logicOctAmbLuck2Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractAmbrosiaLuck2.effect', { n: format(2 * n) }),
    name: () => i18next.t('octeract.data.octeractAmbrosiaLuck2.name'),
    description: () => i18next.t('octeract.data.octeractAmbrosiaLuck2.description'),
    qualityOfLife: true
  },
  octeractAmbrosiaLuck3: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAmbLuck3Cost,
    maxLevel: 30,
    costPerLevel: 1e30,
    effect: logicOctAmbLuck3Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractAmbrosiaLuck3.effect', { n: format(3 * n) }),
    name: () => i18next.t('octeract.data.octeractAmbrosiaLuck3.name'),
    description: () => i18next.t('octeract.data.octeractAmbrosiaLuck3.description'),
    qualityOfLife: true
  },
  octeractAmbrosiaLuck4: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAmbLuck4Cost,
    maxLevel: 50,
    costPerLevel: 1e70 / 2,
    effect: logicOctAmbLuck4Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractAmbrosiaLuck4.effect', { n: format(5 * n) }),
    name: () => i18next.t('octeract.data.octeractAmbrosiaLuck4.name'),
    description: () => i18next.t('octeract.data.octeractAmbrosiaLuck4.description'),
    qualityOfLife: true
  },
  octeractAmbrosiaGeneration: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAmbGenCost,
    maxLevel: -1,
    costPerLevel: 1e60 / 9,
    effect: logicOctAmbGenEffect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractAmbrosiaGeneration.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractAmbrosiaGeneration.name'),
    description: () => i18next.t('octeract.data.octeractAmbrosiaGeneration.description'),
    qualityOfLife: true
  },
  octeractAmbrosiaGeneration2: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAmbGen2Cost,
    maxLevel: 20,
    costPerLevel: 1,
    effect: logicOctAmbGen2Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractAmbrosiaGeneration2.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractAmbrosiaGeneration2.name'),
    description: () => i18next.t('octeract.data.octeractAmbrosiaGeneration2.description'),
    qualityOfLife: true
  },
  octeractAmbrosiaGeneration3: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAmbGen3Cost,
    maxLevel: 35,
    costPerLevel: 1e30,
    effect: logicOctAmbGen3Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractAmbrosiaGeneration3.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractAmbrosiaGeneration3.name'),
    description: () => i18next.t('octeract.data.octeractAmbrosiaGeneration3.description'),
    qualityOfLife: true
  },
  octeractAmbrosiaGeneration4: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctAmbGen4Cost,
    maxLevel: 50,
    costPerLevel: 1e70 / 2,
    effect: logicOctAmbGen4Effect,
    effectDescription: (n: number) =>
      i18next.t('octeract.data.octeractAmbrosiaGeneration4.effect', { n: format(2 * n) }),
    name: () => i18next.t('octeract.data.octeractAmbrosiaGeneration4.name'),
    description: () => i18next.t('octeract.data.octeractAmbrosiaGeneration4.description'),
    qualityOfLife: true
  },
  octeractBonusTokens1: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctBonusTokens1Cost,
    maxLevel: 10,
    costPerLevel: 1e-5,
    effect: logicOctBonusTokens1Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractBonusTokens1.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractBonusTokens1.name'),
    description: () => i18next.t('octeract.data.octeractBonusTokens1.description'),
    qualityOfLife: false
  },
  octeractBonusTokens2: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctBonusTokens2Cost,
    maxLevel: 5,
    costPerLevel: 1e8,
    effect: logicOctBonusTokens2Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractBonusTokens2.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractBonusTokens2.name'),
    description: () => i18next.t('octeract.data.octeractBonusTokens2.description'),
    qualityOfLife: false
  },
  octeractBonusTokens3: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctBonusTokens3Cost,
    maxLevel: 5,
    costPerLevel: 1e40,
    effect: logicOctBonusTokens3Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractBonusTokens3.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractBonusTokens3.name'),
    description: () => i18next.t('octeract.data.octeractBonusTokens3.description'),
    qualityOfLife: false
  },
  octeractBonusTokens4: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    costFormula: logicOctBonusTokens4Cost,
    maxLevel: 50,
    costPerLevel: 1e75,
    effect: logicOctBonusTokens4Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractBonusTokens4.effect', { n: format(2 * n) }),
    name: () => i18next.t('octeract.data.octeractBonusTokens4.name'),
    description: () => i18next.t('octeract.data.octeractBonusTokens4.description'),
    qualityOfLife: false
  },
  octeractBlueberries: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    maxLevel: 6,
    costPerLevel: 1,
    costFormula: logicOctBlueberriesCost,
    effect: logicOctBlueberriesEffect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractBlueberries.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractBlueberries.name'),
    description: () => i18next.t('octeract.data.octeractBlueberries.description'),
    qualityOfLife: true
  },
  octeractInfiniteShopUpgrades: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    maxLevel: 80,
    costPerLevel: 1e30,
    costFormula: logicOctInfShopCost,
    effect: logicOctInfShopEffect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractInfiniteShopUpgrades.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractInfiniteShopUpgrades.name'),
    description: () => i18next.t('octeract.data.octeractInfiniteShopUpgrades.description'),
    qualityOfLife: false
  },
  octeractTalismanLevelCap1: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    maxLevel: 25,
    costPerLevel: 1e-5,
    costFormula: logicOctTalismanCap1Cost,
    effect: logicOctTalismanCap1Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractTalismanLevelCap1.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractTalismanLevelCap1.name'),
    description: () => i18next.t('octeract.data.octeractTalismanLevelCap1.description'),
    qualityOfLife: false
  },
  octeractTalismanLevelCap2: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    maxLevel: 35,
    costPerLevel: 1e10,
    costFormula: logicOctTalismanCap2Cost,
    effect: logicOctTalismanCap2Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractTalismanLevelCap2.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractTalismanLevelCap2.name'),
    description: () => i18next.t('octeract.data.octeractTalismanLevelCap2.description'),
    qualityOfLife: false
  },
  octeractTalismanLevelCap3: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    maxLevel: 40,
    costPerLevel: 1e20,
    costFormula: logicOctTalismanCap3Cost,
    effect: logicOctTalismanCap3Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractTalismanLevelCap3.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractTalismanLevelCap3.name'),
    description: () => i18next.t('octeract.data.octeractTalismanLevelCap3.description'),
    qualityOfLife: false
  },
  octeractTalismanLevelCap4: {
    level: 0,
    freeLevel: 0,
    octeractsInvested: 0,
    maxLevel: -1,
    costPerLevel: 1e40,
    costFormula: logicOctTalismanCap4Cost,
    effect: logicOctTalismanCap4Effect,
    effectDescription: (n: number) => i18next.t('octeract.data.octeractTalismanLevelCap4.effect', { n: format(n) }),
    name: () => i18next.t('octeract.data.octeractTalismanLevelCap4.name'),
    description: () => i18next.t('octeract.data.octeractTalismanLevelCap4.description'),
    qualityOfLife: false
  }
}

export const maxOcteractUpgradeAP = Object.values(octeractUpgrades).reduce((acc, upgrade) => {
  if (upgrade.maxLevel === -1) {
    return acc
  }
  return acc + 8
}, 0)

export const blankOcteractLevelObject: Record<
  OcteractUpgrades,
  { level: number; freeLevel: number; octeractsInvested: number }
> = Object
  .fromEntries(
    Object.keys(octeractUpgrades).map((key) => [
      key as OcteractUpgrades,
      {
        level: 0,
        freeLevel: 0,
        octeractsInvested: 0
      }
    ])
  ) as Record<OcteractUpgrades, { level: number; freeLevel: number; octeractsInvested: number }>

export const getOcteractUpgradeCostTNL = (upgradeKey: OcteractUpgrades): number => {
  const upgrade = octeractUpgrades[upgradeKey]

  if (upgrade.level === upgrade.maxLevel) {
    return 0
  }

  return upgrade.costFormula(upgrade.level, upgrade.costPerLevel)
}

// Thin shim — sources cubeUpgrades[78] for the multiplier.
const computeFreeLevelMultiplier = (): number => logicOcteractFreeLevelMultiplier(player.cubeUpgrades[78])

export const computeOcteractFreeLevelSoftcap = (upgradeKey: OcteractUpgrades): number =>
  logicOcteractFreeLevelSoftcap(octeractUpgrades[upgradeKey].freeLevel, computeFreeLevelMultiplier())

// Thin shim over @synergism/logic. Sources level / freeLevel / qualityOfLife
// off the upgrade snapshot and the noOcteracts / sadisticPrequel gate flags
// off the player.
export const actualOcteractUpgradeTotalLevels = (upgradeKey: OcteractUpgrades): number => {
  const upgrade = octeractUpgrades[upgradeKey]
  return logicActualOcteractUpgradeTotalLevels({
    level: upgrade.level,
    freeLevel: upgrade.freeLevel,
    qualityOfLife: upgrade.qualityOfLife,
    cubeUpgrade78: player.cubeUpgrades[78],
    inNoOcteracts: player.singularityChallenges.noOcteracts.enabled,
    inSadisticPrequel: player.singularityChallenges.sadisticPrequel.enabled
  })
}

export const upgradeOcteractToString = (upgradeKey: OcteractUpgrades): string => {
  const upgrade = octeractUpgrades[upgradeKey]
  const name = upgrade.name()
  const costNextLevel = getOcteractUpgradeCostTNL(upgradeKey)
  const freeLevelMult = computeFreeLevelMultiplier()
  const freeLevelsWithMult = upgrade.freeLevel * freeLevelMult
  const totalEffectiveLevels = actualOcteractUpgradeTotalLevels(upgradeKey)

  const maxLevel = upgrade.maxLevel === -1
    ? ''
    : `/${format(upgrade.maxLevel, 0, true)}`

  const isMaxLevel = upgrade.maxLevel === upgrade.level
  const color = isMaxLevel ? 'plum' : 'white'

  const nameHTML = `<span style="color: gold">${name}</span>`
  const descriptionHTML = `<span style="color: lightblue">${upgrade.description()}</span>`

  const freeLevelMultText = freeLevelMult > 1
    ? `<span style="color: crimson"> (x${format(freeLevelMult, 2, true)})</span>`
    : ''

  let freeLevelText = upgrade.freeLevel > 0
    ? `<span style="color: orange"> [+${format(upgrade.freeLevel, 1, true)}${freeLevelMultText}]</span>`
    : ''

  if (freeLevelsWithMult > upgrade.level) {
    freeLevelText = `${freeLevelText} <span style="color: var(--maroon-text-color)">${
      i18next.t('general.softCapped')
    }</span>`
  }

  const effectiveLevelText = totalEffectiveLevels !== upgrade.level + upgrade.freeLevel
    ? `<br><b><span style="color: white">${
      i18next.t('general.effectiveLevel', {
        level: format(totalEffectiveLevels, 2, true)
      })
    }</span></b>`
    : ''

  const levelHTML = `<span style="color: ${color}"> ${i18next.t('general.level')} ${
    format(upgrade.level, 0, true)
  }${maxLevel}${freeLevelText}</span>`

  const isAffordable = costNextLevel <= player.wowOcteracts
  let affordTime = ''
  if (!isMaxLevel && !isAffordable) {
    const octPerSecond = calculateOcteractMultiplier()
    affordTime = octPerSecond > 0
      ? formatTimeShort((costNextLevel - player.wowOcteracts) / octPerSecond)
      : i18next.t('general.infinity')
  }

  const affordableInfo = isMaxLevel
    ? `<span style="color: plum"> ${i18next.t('general.maxed')}</span>`
    : isAffordable
    ? `<span style="color: var(--green-text-color)"> ${i18next.t('general.affordable')}</span>`
    : `<span style="color: yellow"> ${i18next.t('octeract.toString.becomeAffordable', { n: affordTime })}</span>`

  const totalLevels = actualOcteractUpgradeTotalLevels(upgradeKey)
  const effectHTML = `<span style="color: gold">${upgrade.effectDescription(totalLevels)}</span>`

  const costHTML = (upgrade.level === upgrade.maxLevel && upgrade.maxLevel !== -1)
    ? ''
    : `${
      i18next.t('octeract.toString.costNextLevel', {
        amount: format(costNextLevel, 2, true, true, true)
      })
    } ${affordableInfo}`

  const investedOcteractsHTML = upgrade.octeractsInvested > 0
    ? `<br><span style="color: turquoise">${
      i18next.t('octeract.toString.spentOcteracts', {
        spent: format(upgrade.octeractsInvested, 2, true, true, true)
      })
    }</span>`
    : ''

  const qualityOfLifeText = upgrade.qualityOfLife
    ? `<br><span style="color: orchid">${i18next.t('general.alwaysEnabled')}</span>`
    : ''

  return `${nameHTML}<br>${levelHTML}${effectiveLevelText}<br>${descriptionHTML}<br>${effectHTML}<br>${costHTML}${investedOcteractsHTML}${qualityOfLifeText}`
}

export const updateMobileOcteractHTML = (upgradeKey: OcteractUpgrades): void => {
  const elm = DOMCacheGetOrSet('singularityOcteractsMultiline')
  elm.innerHTML = upgradeOcteractToString(upgradeKey)

  // MOBILE ONLY - Add a button for buying upgrades
  if (isMobile) {
    const buttonDiv = document.createElement('div')

    const buyOne = document.createElement('button')
    const buyMax = document.createElement('button')

    buyOne.classList.add('modalBtnBuy')
    buyOne.textContent = i18next.t('general.buyOne')
    buyOne.addEventListener('click', (event: MouseEvent) => {
      buyOcteractUpgradeLevel(upgradeKey, event, false)
      updateMobileOcteractHTML(upgradeKey)
    })

    buyMax.classList.add('modalBtnBuy')
    buyMax.textContent = i18next.t('general.buyMax')
    buyMax.addEventListener('click', (event: MouseEvent) => {
      buyOcteractUpgradeLevel(upgradeKey, event, true)
      updateMobileOcteractHTML(upgradeKey)
    })

    buttonDiv.appendChild(buyOne)
    buttonDiv.appendChild(buyMax)
    elm.appendChild(buttonDiv)
  }
}

export const buyOcteractUpgradeLevel = async (
  upgradeKey: OcteractUpgrades,
  event: MouseEvent,
  buyMax = false
): Promise<void> => {
  const upgrade = octeractUpgrades[upgradeKey]
  let purchased = 0
  let maxPurchasable = 1
  let OCTBudget = player.wowOcteracts

  if (event.shiftKey || buyMax) {
    maxPurchasable = 100000000
    const buy = Number(
      await Prompt(i18next.t('octeract.buyLevel.buyPrompt', { n: format(player.wowOcteracts, 0, true) }))
    )

    if (isNaN(buy) || !isFinite(buy) || !Number.isInteger(buy)) {
      return Alert(i18next.t('general.validation.finite'))
    }

    if (buy === -1) {
      OCTBudget = player.wowOcteracts
    } else if (buy <= 0) {
      return Alert(i18next.t('octeract.buyLevel.cancelPurchase'))
    } else {
      OCTBudget = buy
    }
    OCTBudget = Math.min(player.wowOcteracts, OCTBudget)
  }

  if (upgrade.maxLevel > 0) {
    maxPurchasable = Math.min(maxPurchasable, upgrade.maxLevel - upgrade.level)
  }

  if (maxPurchasable === 0) {
    return Alert(i18next.t('octeract.buyLevel.alreadyMax'))
  }

  while (maxPurchasable > 0) {
    const cost = upgrade.costFormula(upgrade.level, upgrade.costPerLevel)
    if (player.wowOcteracts < cost || OCTBudget < cost) {
      break
    } else {
      player.wowOcteracts -= cost
      upgrade.octeractsInvested += cost
      OCTBudget -= cost
      upgrade.level += 1
      purchased += 1
      maxPurchasable -= 1
    }
  }

  if (purchased === 0) {
    return Alert(i18next.t('octeract.buyLevel.cannotAfford'))
  }

  if (purchased > 1) {
    Alert(i18next.t('octeract.buyLevel.multiBuy', { n: format(purchased) }))
  }

  updateTokens()
  updateMaxTokens()
}

export const getOcteractUpgradeEffect = <
  T extends OcteractUpgrades,
  K extends keyof OcteractUpgradeRewards[T]
>(upgradeKey: T, key: K): OcteractUpgradeRewards[T][K] => {
  const upgrade = octeractUpgrades[upgradeKey]
  const totalLevels = actualOcteractUpgradeTotalLevels(upgradeKey)
  return upgrade.effect(totalLevels, key) as OcteractUpgradeRewards[T][K]
}
