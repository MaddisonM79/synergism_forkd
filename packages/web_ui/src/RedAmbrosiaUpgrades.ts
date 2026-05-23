import {
  blueberriesCostFormula as logicBlueberriesCostFormula,
  blueberriesEffect as logicBlueberriesEffect,
  blueberryGenerationSpeed2CostFormula as logicBlueberryGenerationSpeed2CostFormula,
  blueberryGenerationSpeed2Effect as logicBlueberryGenerationSpeed2Effect,
  blueberryGenerationSpeedCostFormula as logicBlueberryGenerationSpeedCostFormula,
  blueberryGenerationSpeedEffect as logicBlueberryGenerationSpeedEffect,
  conversionImprovement1CostFormula as logicConversionImprovement1CostFormula,
  conversionImprovement1Effect as logicConversionImprovement1Effect,
  conversionImprovement2CostFormula as logicConversionImprovement2CostFormula,
  conversionImprovement2Effect as logicConversionImprovement2Effect,
  conversionImprovement3CostFormula as logicConversionImprovement3CostFormula,
  conversionImprovement3Effect as logicConversionImprovement3Effect,
  freeCubeUpgradesCostFormula as logicFreeCubeUpgradesCostFormula,
  freeCubeUpgradesEffect as logicFreeCubeUpgradesEffect,
  freeLevelsRow2CostFormula as logicFreeLevelsRow2CostFormula,
  freeLevelsRow2Effect as logicFreeLevelsRow2Effect,
  freeLevelsRow3CostFormula as logicFreeLevelsRow3CostFormula,
  freeLevelsRow3Effect as logicFreeLevelsRow3Effect,
  freeLevelsRow4CostFormula as logicFreeLevelsRow4CostFormula,
  freeLevelsRow4Effect as logicFreeLevelsRow4Effect,
  freeLevelsRow5CostFormula as logicFreeLevelsRow5CostFormula,
  freeLevelsRow5Effect as logicFreeLevelsRow5Effect,
  freeObtainiumUpgradesCostFormula as logicFreeObtainiumUpgradesCostFormula,
  freeObtainiumUpgradesEffect as logicFreeObtainiumUpgradesEffect,
  freeOfferingUpgradesCostFormula as logicFreeOfferingUpgradesCostFormula,
  freeOfferingUpgradesEffect as logicFreeOfferingUpgradesEffect,
  freeSpeedUpgradesCostFormula as logicFreeSpeedUpgradesCostFormula,
  freeSpeedUpgradesEffect as logicFreeSpeedUpgradesEffect,
  freeTutorialLevelsCostFormula as logicFreeTutorialLevelsCostFormula,
  freeTutorialLevelsEffect as logicFreeTutorialLevelsEffect,
  infiniteShopUpgradesCostFormula as logicInfiniteShopUpgradesCostFormula,
  infiniteShopUpgradesEffect as logicInfiniteShopUpgradesEffect,
  redAmbrosiaAcceleratorCostFormula as logicRedAmbrosiaAcceleratorCostFormula,
  redAmbrosiaAcceleratorEffect as logicRedAmbrosiaAcceleratorEffect,
  redAmbrosiaCubeCostFormula as logicRedAmbrosiaCubeCostFormula,
  redAmbrosiaCubeEffect as logicRedAmbrosiaCubeEffect,
  redAmbrosiaCubeImproverCostFormula as logicRedAmbrosiaCubeImproverCostFormula,
  redAmbrosiaCubeImproverEffect as logicRedAmbrosiaCubeImproverEffect,
  redAmbrosiaFreeAccumulatorCostFormula as logicRedAmbrosiaFreeAccumulatorCostFormula,
  redAmbrosiaFreeAccumulatorEffect as logicRedAmbrosiaFreeAccumulatorEffect,
  type RedAmbrosiaNames as LogicRedAmbrosiaNames,
  redAmbrosiaObtainiumCostFormula as logicRedAmbrosiaObtainiumCostFormula,
  redAmbrosiaObtainiumEffect as logicRedAmbrosiaObtainiumEffect,
  redAmbrosiaOfferingCostFormula as logicRedAmbrosiaOfferingCostFormula,
  redAmbrosiaOfferingEffect as logicRedAmbrosiaOfferingEffect,
  type RedAmbrosiaUpgradeRewards as LogicRedAmbrosiaUpgradeRewards,
  redGenerationSpeedCostFormula as logicRedGenerationSpeedCostFormula,
  redGenerationSpeedEffect as logicRedGenerationSpeedEffect,
  redLuckCostFormula as logicRedLuckCostFormula,
  redLuckEffect as logicRedLuckEffect,
  regularLuck2CostFormula as logicRegularLuck2CostFormula,
  regularLuck2Effect as logicRegularLuck2Effect,
  regularLuckCostFormula as logicRegularLuckCostFormula,
  regularLuckEffect as logicRegularLuckEffect,
  salvageYinYangCostFormula as logicSalvageYinYangCostFormula,
  salvageYinYangEffect as logicSalvageYinYangEffect,
  tutorialCostFormula as logicTutorialCostFormula,
  tutorialEffect as logicTutorialEffect,
  viscountCostFormula as logicViscountCostFormula,
  viscountEffect as logicViscountEffect
} from '@synergism/logic'
import i18next from 'i18next'
import { DOMCacheGetOrSet } from './Cache/DOM'
import { calculateRedAmbrosiaCubes, calculateRedAmbrosiaObtainium, calculateRedAmbrosiaOffering } from './Calculate'
import { format, formatAsPercentIncrease, player } from './Synergism'
import { Alert, Prompt } from './UpdateHTML'
import { isMobile } from './Utility'

// Re-exported from @synergism/logic so existing call sites that import these
// types from this module keep compiling unchanged.
type RedAmbrosiaUpgradeRewards = LogicRedAmbrosiaUpgradeRewards
export type RedAmbrosiaNames = LogicRedAmbrosiaNames

interface RedAmbrosiaUpgrade<T extends RedAmbrosiaNames, K extends keyof RedAmbrosiaUpgradeRewards[T]> {
  name: () => string
  description: () => string
  level: number
  maxLevel: number
  costPerLevel: number
  redAmbrosiaInvested: number
  costFormula: (level: number, baseCost: number) => number
  effects: (n: number, key: K) => RedAmbrosiaUpgradeRewards[T][K]
  effectsDescription: (n: number) => string
}

export const redAmbrosiaUpgrades: {
  [K in RedAmbrosiaNames]: RedAmbrosiaUpgrade<K, keyof RedAmbrosiaUpgradeRewards[K]>
} = {
  tutorial: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicTutorialCostFormula,
    effects: logicTutorialEffect,
    effectsDescription: (_n: number) => {
      const val = getRedAmbrosiaUpgradeEffects('tutorial', 'cubeMult')
      return i18next.t('redAmbrosia.data.tutorial.effect', {
        amount: formatAsPercentIncrease(val)
      })
    },
    maxLevel: 100,
    costPerLevel: 1,
    name: () => i18next.t('redAmbrosia.data.tutorial.name'),
    description: () => i18next.t('redAmbrosia.data.tutorial.description')
  },
  conversionImprovement1: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicConversionImprovement1CostFormula,
    effects: logicConversionImprovement1Effect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.conversionImprovement1.effect', { amount: n })
    },
    maxLevel: 5,
    costPerLevel: 5,
    name: () => i18next.t('redAmbrosia.data.conversionImprovement1.name'),
    description: () => i18next.t('redAmbrosia.data.conversionImprovement1.description')
  },
  conversionImprovement2: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicConversionImprovement2CostFormula,
    effects: logicConversionImprovement2Effect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.conversionImprovement2.effect', { amount: n })
    },
    maxLevel: 3,
    costPerLevel: 200,
    name: () => i18next.t('redAmbrosia.data.conversionImprovement2.name'),
    description: () => i18next.t('redAmbrosia.data.conversionImprovement2.description')
  },
  conversionImprovement3: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicConversionImprovement3CostFormula,
    effects: logicConversionImprovement3Effect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.conversionImprovement3.effect', { amount: n })
    },
    maxLevel: 2,
    costPerLevel: 10000,
    name: () => i18next.t('redAmbrosia.data.conversionImprovement3.name'),
    description: () => i18next.t('redAmbrosia.data.conversionImprovement3.description')
  },
  freeTutorialLevels: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicFreeTutorialLevelsCostFormula,
    effects: logicFreeTutorialLevelsEffect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.freeTutorialLevels.effect', { amount: n })
    },
    maxLevel: 5,
    costPerLevel: 1,
    name: () => i18next.t('redAmbrosia.data.freeTutorialLevels.name'),
    description: () => i18next.t('redAmbrosia.data.freeTutorialLevels.description')
  },
  freeLevelsRow2: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicFreeLevelsRow2CostFormula,
    effects: logicFreeLevelsRow2Effect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.freeLevelsRow2.effect', { amount: n })
    },
    maxLevel: 5,
    costPerLevel: 10,
    name: () => i18next.t('redAmbrosia.data.freeLevelsRow2.name'),
    description: () => i18next.t('redAmbrosia.data.freeLevelsRow2.description')
  },
  freeLevelsRow3: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicFreeLevelsRow3CostFormula,
    effects: logicFreeLevelsRow3Effect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.freeLevelsRow3.effect', { amount: n })
    },
    maxLevel: 5,
    costPerLevel: 250,
    name: () => i18next.t('redAmbrosia.data.freeLevelsRow3.name'),
    description: () => i18next.t('redAmbrosia.data.freeLevelsRow3.description')
  },
  freeLevelsRow4: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicFreeLevelsRow4CostFormula,
    effects: logicFreeLevelsRow4Effect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.freeLevelsRow4.effect', { amount: n })
    },
    maxLevel: 5,
    costPerLevel: 5000,
    name: () => i18next.t('redAmbrosia.data.freeLevelsRow4.name'),
    description: () => i18next.t('redAmbrosia.data.freeLevelsRow4.description')
  },
  freeLevelsRow5: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicFreeLevelsRow5CostFormula,
    effects: logicFreeLevelsRow5Effect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.freeLevelsRow5.effect', { amount: n })
    },
    maxLevel: 5,
    costPerLevel: 50000,
    name: () => i18next.t('redAmbrosia.data.freeLevelsRow5.name'),
    description: () => i18next.t('redAmbrosia.data.freeLevelsRow5.description')
  },
  blueberryGenerationSpeed: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicBlueberryGenerationSpeedCostFormula,
    effects: logicBlueberryGenerationSpeedEffect,
    effectsDescription: (_n: number) => {
      const val = getRedAmbrosiaUpgradeEffects('blueberryGenerationSpeed', 'blueberryGenerationSpeed')
      return i18next.t('redAmbrosia.data.blueberryGenerationSpeed.effect', { amount: formatAsPercentIncrease(val) })
    },
    maxLevel: 100,
    costPerLevel: 1,
    name: () => i18next.t('redAmbrosia.data.blueberryGenerationSpeed.name'),
    description: () => i18next.t('redAmbrosia.data.blueberryGenerationSpeed.description')
  },
  regularLuck: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicRegularLuckCostFormula,
    effects: logicRegularLuckEffect,
    effectsDescription: (_n: number) => {
      const val = getRedAmbrosiaUpgradeEffects('regularLuck', 'ambrosiaLuck')
      return i18next.t('redAmbrosia.data.regularLuck.effect', { amount: val })
    },
    maxLevel: 100,
    costPerLevel: 1,
    name: () => i18next.t('redAmbrosia.data.regularLuck.name'),
    description: () => i18next.t('redAmbrosia.data.regularLuck.description')
  },
  redGenerationSpeed: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicRedGenerationSpeedCostFormula,
    effects: logicRedGenerationSpeedEffect,
    effectsDescription: (_n: number) => {
      const val = getRedAmbrosiaUpgradeEffects('redGenerationSpeed', 'redAmbrosiaGenerationSpeed')
      return i18next.t('redAmbrosia.data.redGenerationSpeed.effect', { amount: formatAsPercentIncrease(val) })
    },
    maxLevel: 100,
    costPerLevel: 12,
    name: () => i18next.t('redAmbrosia.data.redGenerationSpeed.name'),
    description: () => i18next.t('redAmbrosia.data.redGenerationSpeed.description')
  },
  redLuck: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicRedLuckCostFormula,
    effects: logicRedLuckEffect,
    effectsDescription: (_n: number) => {
      const val = getRedAmbrosiaUpgradeEffects('redLuck', 'redAmbrosiaLuck')
      return i18next.t('redAmbrosia.data.redLuck.effect', { amount: val })
    },
    maxLevel: 100,
    costPerLevel: 4,
    name: () => i18next.t('redAmbrosia.data.redLuck.name'),
    description: () => i18next.t('redAmbrosia.data.redLuck.description')
  },
  redAmbrosiaCube: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicRedAmbrosiaCubeCostFormula,
    effects: logicRedAmbrosiaCubeEffect,
    effectsDescription: (n: number) => {
      const exponent = 0.4 + getRedAmbrosiaUpgradeEffects('redAmbrosiaCubeImprover', 'extraExponent')
      if (n > 0) {
        const cubeMult = calculateRedAmbrosiaCubes()
        return i18next.t('redAmbrosia.data.redAmbrosiaCube.effectEnabled', {
          exponent: format(exponent, 2, true),
          amount: formatAsPercentIncrease(cubeMult, 2)
        })
      } else {
        return i18next.t('redAmbrosia.data.redAmbrosiaCube.effect', {
          exponent: format(exponent, 2, true)
        })
      }
    },
    maxLevel: 1,
    costPerLevel: 500,
    name: () => i18next.t('redAmbrosia.data.redAmbrosiaCube.name'),
    description: () => i18next.t('redAmbrosia.data.redAmbrosiaCube.description')
  },
  redAmbrosiaObtainium: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicRedAmbrosiaObtainiumCostFormula,
    effects: logicRedAmbrosiaObtainiumEffect,
    effectsDescription: (n: number) => {
      if (n > 0) {
        const obtainiumMult = calculateRedAmbrosiaObtainium()
        return i18next.t('redAmbrosia.data.redAmbrosiaObtainium.effectEnabled', {
          amount: formatAsPercentIncrease(obtainiumMult, 2)
        })
      } else {
        return i18next.t('redAmbrosia.data.redAmbrosiaObtainium.effect')
      }
    },
    maxLevel: 1,
    costPerLevel: 1250,
    name: () => i18next.t('redAmbrosia.data.redAmbrosiaObtainium.name'),
    description: () => i18next.t('redAmbrosia.data.redAmbrosiaObtainium.description')
  },
  redAmbrosiaOffering: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicRedAmbrosiaOfferingCostFormula,
    effects: logicRedAmbrosiaOfferingEffect,
    effectsDescription: (n: number) => {
      if (n > 0) {
        const offeringMult = calculateRedAmbrosiaOffering()
        return i18next.t('redAmbrosia.data.redAmbrosiaOffering.effectEnabled', {
          amount: formatAsPercentIncrease(offeringMult, 2)
        })
      }
      return i18next.t('redAmbrosia.data.redAmbrosiaOffering.effect')
    },
    maxLevel: 1,
    costPerLevel: 4000,
    name: () => i18next.t('redAmbrosia.data.redAmbrosiaOffering.name'),
    description: () => i18next.t('redAmbrosia.data.redAmbrosiaOffering.description')
  },
  redAmbrosiaCubeImprover: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicRedAmbrosiaCubeImproverCostFormula,
    effects: logicRedAmbrosiaCubeImproverEffect,
    effectsDescription: (_n: number) => {
      const extraExponent = getRedAmbrosiaUpgradeEffects('redAmbrosiaCubeImprover', 'extraExponent')
      return i18next.t('redAmbrosia.data.redAmbrosiaCubeImprover.effect', {
        newExponent: format(0.4 + extraExponent, 2, true)
      })
    },
    maxLevel: 20,
    costPerLevel: 100,
    name: () => i18next.t('redAmbrosia.data.redAmbrosiaCubeImprover.name'),
    description: () => i18next.t('redAmbrosia.data.redAmbrosiaCubeImprover.description')
  },
  viscount: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicViscountCostFormula,
    effects: logicViscountEffect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.viscount.effect', { mark: n > 0 ? '✔' : '❌' })
    },
    maxLevel: 1,
    costPerLevel: 99999,
    name: () => i18next.t('redAmbrosia.data.viscount.name'),
    description: () => i18next.t('redAmbrosia.data.viscount.description')
  },
  infiniteShopUpgrades: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicInfiniteShopUpgradesCostFormula,
    effects: logicInfiniteShopUpgradesEffect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.infiniteShopUpgrades.effect', { amount: n })
    },
    maxLevel: 40,
    costPerLevel: 200,
    name: () => i18next.t('redAmbrosia.data.infiniteShopUpgrades.name'),
    description: () => i18next.t('redAmbrosia.data.infiniteShopUpgrades.description')
  },
  redAmbrosiaAccelerator: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicRedAmbrosiaAcceleratorCostFormula,
    effects: logicRedAmbrosiaAcceleratorEffect,
    effectsDescription: (_n: number) => {
      const ambrosiaTimePerRedAmbrosia = getRedAmbrosiaUpgradeEffects(
        'redAmbrosiaAccelerator',
        'ambrosiaTimePerRedAmbrosia'
      )
      return i18next.t('redAmbrosia.data.redAmbrosiaAccelerator.effect', {
        amount: format(ambrosiaTimePerRedAmbrosia, 2, true)
      })
    },
    maxLevel: 100,
    costPerLevel: 1000,
    name: () => i18next.t('redAmbrosia.data.redAmbrosiaAccelerator.name'),
    description: () => i18next.t('redAmbrosia.data.redAmbrosiaAccelerator.description')
  },
  regularLuck2: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicRegularLuck2CostFormula,
    effects: logicRegularLuck2Effect,
    effectsDescription: (_n: number) => {
      const ambrosiaLuck = getRedAmbrosiaUpgradeEffects('regularLuck2', 'ambrosiaLuck')
      return i18next.t('redAmbrosia.data.regularLuck2.effect', { amount: ambrosiaLuck })
    },
    maxLevel: 250,
    costPerLevel: 8000,
    name: () => i18next.t('redAmbrosia.data.regularLuck2.name'),
    description: () => i18next.t('redAmbrosia.data.regularLuck2.description')
  },
  blueberryGenerationSpeed2: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicBlueberryGenerationSpeed2CostFormula,
    effects: logicBlueberryGenerationSpeed2Effect,
    effectsDescription: (_n: number) => {
      const val = getRedAmbrosiaUpgradeEffects('blueberryGenerationSpeed2', 'blueberryGenerationSpeed')
      return i18next.t('redAmbrosia.data.blueberryGenerationSpeed2.effect', { amount: formatAsPercentIncrease(val) })
    },
    maxLevel: 250,
    costPerLevel: 8000,
    name: () => i18next.t('redAmbrosia.data.blueberryGenerationSpeed2.name'),
    description: () => i18next.t('redAmbrosia.data.blueberryGenerationSpeed2.description')
  },
  salvageYinYang: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicSalvageYinYangCostFormula,
    effects: (n, key) => logicSalvageYinYangEffect(n, key, player.singularityChallenges.taxmanLastStand.enabled),
    effectsDescription: (n: number) => {
      const bonus = player.singularityChallenges.taxmanLastStand.enabled ? 0 : 10 * n
      return i18next.t('redAmbrosia.data.salvageYinYang.effect', { amount: bonus })
    },
    maxLevel: 100,
    costPerLevel: 200,
    name: () => i18next.t('redAmbrosia.data.salvageYinYang.name'),
    description: () => i18next.t('redAmbrosia.data.salvageYinYang.description')
  },
  blueberries: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicBlueberriesCostFormula,
    effects: logicBlueberriesEffect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.blueberries.effect', { amount: n })
    },
    maxLevel: 5,
    costPerLevel: 1e5,
    name: () => i18next.t('redAmbrosia.data.blueberries.name'),
    description: () => i18next.t('redAmbrosia.data.blueberries.description')
  },
  redAmbrosiaFreeAccumulator: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicRedAmbrosiaFreeAccumulatorCostFormula,
    effects: logicRedAmbrosiaFreeAccumulatorEffect,
    effectsDescription: (_n: number) => {
      const freeLevels = getRedAmbrosiaUpgradeEffects('redAmbrosiaFreeAccumulator', 'freeAccumulatorLevels')
      const capIncrease = getRedAmbrosiaUpgradeEffects('redAmbrosiaFreeAccumulator', 'freeAccumulatorLevelCapIncrease')
      return i18next.t('redAmbrosia.data.redAmbrosiaFreeAccumulator.effect', {
        levels: format(freeLevels, 3, true),
        cap: format(capIncrease + 1, 1, true)
      })
    },
    maxLevel: 10,
    costPerLevel: 1,
    name: () => i18next.t('redAmbrosia.data.redAmbrosiaFreeAccumulator.name'),
    description: () => i18next.t('redAmbrosia.data.redAmbrosiaFreeAccumulator.description')
  },
  freeOfferingUpgrades: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicFreeOfferingUpgradesCostFormula,
    effects: logicFreeOfferingUpgradesEffect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.freeOfferingUpgrades.effect', { amount: n })
    },
    maxLevel: 5,
    costPerLevel: 1,
    name: () => i18next.t('redAmbrosia.data.freeOfferingUpgrades.name'),
    description: () => i18next.t('redAmbrosia.data.freeOfferingUpgrades.description')
  },
  freeObtainiumUpgrades: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicFreeObtainiumUpgradesCostFormula,
    effects: logicFreeObtainiumUpgradesEffect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.freeObtainiumUpgrades.effect', { amount: n })
    },
    maxLevel: 5,
    costPerLevel: 1,
    name: () => i18next.t('redAmbrosia.data.freeObtainiumUpgrades.name'),
    description: () => i18next.t('redAmbrosia.data.freeObtainiumUpgrades.description')
  },
  freeCubeUpgrades: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicFreeCubeUpgradesCostFormula,
    effects: logicFreeCubeUpgradesEffect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.freeCubeUpgrades.effect', { amount: n })
    },
    maxLevel: 5,
    costPerLevel: 1,
    name: () => i18next.t('redAmbrosia.data.freeCubeUpgrades.name'),
    description: () => i18next.t('redAmbrosia.data.freeCubeUpgrades.description')
  },
  freeSpeedUpgrades: {
    level: 0,
    redAmbrosiaInvested: 0,
    costFormula: logicFreeSpeedUpgradesCostFormula,
    effects: logicFreeSpeedUpgradesEffect,
    effectsDescription: (n: number) => {
      return i18next.t('redAmbrosia.data.freeSpeedUpgrades.effect', { amount: n })
    },
    maxLevel: 5,
    costPerLevel: 1,
    name: () => i18next.t('redAmbrosia.data.freeSpeedUpgrades.name'),
    description: () => i18next.t('redAmbrosia.data.freeSpeedUpgrades.description')
  }
}

export const maxRedAmbrosiaUpgradeAP = Object.values(redAmbrosiaUpgrades).reduce((acc, upgrade) => {
  if (upgrade.maxLevel === -1) {
    return acc
  }
  return acc + 10
}, 0)

export const setRedAmbrosiaUpgradeLevels = (): void => {
  for (const upgradeKey of Object.keys(redAmbrosiaUpgrades) as RedAmbrosiaNames[]) {
    const upgrade = redAmbrosiaUpgrades[upgradeKey]
    const invested = player.redAmbrosiaUpgrades[upgradeKey] || 0

    let level = 0
    let budget = invested

    let nextCost = upgrade.costFormula(level, upgrade.costPerLevel)

    while (budget >= nextCost) {
      budget -= nextCost
      level += 1
      nextCost = upgrade.costFormula(level, upgrade.costPerLevel)

      if (level >= upgrade.maxLevel) {
        break
      }
    }

    // If there is leftover budget, then the formulae has probably changed, or above max.
    // We refund the remaining budget.
    if (budget > 0) {
      player.redAmbrosiaUpgrades[upgradeKey] -= budget
      player.redAmbrosia += budget
    }

    upgrade.level = level
    upgrade.redAmbrosiaInvested = invested - budget
  }
}

export const blankRedAmbrosiaUpgradeObject: Record<RedAmbrosiaNames, number> = Object.fromEntries(
  Object.keys(redAmbrosiaUpgrades).map((key) => [
    key as RedAmbrosiaNames,
    0
  ])
) as Record<RedAmbrosiaNames, number>

export const getRedAmbrosiaUpgradeEffects = <T extends RedAmbrosiaNames, K extends keyof RedAmbrosiaUpgradeRewards[T]>(
  upgradeKey: T,
  key: K
): RedAmbrosiaUpgradeRewards[T][K] => {
  const currentLevel = redAmbrosiaUpgrades[upgradeKey].level
  return redAmbrosiaUpgrades[upgradeKey].effects(currentLevel, key) as RedAmbrosiaUpgradeRewards[T][K]
}

const getRedAmbrosiaUpgradeEffectsDescription = (upgradeKey: RedAmbrosiaNames): string => {
  const currentLevel = redAmbrosiaUpgrades[upgradeKey].level
  return redAmbrosiaUpgrades[upgradeKey].effectsDescription(currentLevel)
}

const getRedAmbrosiaUpgradeCostTNL = (upgradeKey: RedAmbrosiaNames): number => {
  const upgrade = redAmbrosiaUpgrades[upgradeKey]
  if (upgrade.level === upgrade.maxLevel) {
    return 0
  }
  return upgrade.costFormula(upgrade.level, upgrade.costPerLevel)
}

export const redAmbrosiaUpgradeToString = (upgradeKey: RedAmbrosiaNames): string => {
  const upgrade = redAmbrosiaUpgrades[upgradeKey]
  const costNextLevel = getRedAmbrosiaUpgradeCostTNL(upgradeKey)
  const maxLevel = upgrade.maxLevel === -1 ? '' : `/${format(upgrade.maxLevel, 0, true)}`
  const isMaxLevel = upgrade.maxLevel === upgrade.level
  const color = isMaxLevel ? 'plum' : 'white'

  const nameSpan = `<span style="color: gold">${upgrade.name()}</span>`
  const levelSpan = `<span style="color: ${color}"> ${i18next.t('general.level')} ${
    format(upgrade.level, 0, true)
  }${maxLevel}</span>`
  const descriptionSpan = `<span style="color: lightblue">${upgrade.description()}</span>`
  const rewardDescSpan = `<span style="color: gold">${getRedAmbrosiaUpgradeEffectsDescription(upgradeKey)}</span>`

  const costNextLevelSpan = i18next.t('redAmbrosia.redAmbrosiaCost', {
    amount: format(costNextLevel, 0, true)
  })

  const spentSpan = i18next.t('redAmbrosia.redAmbrosiaSpent', {
    amount: format(upgrade.redAmbrosiaInvested, 0, true)
  })

  const purchaseWarningSpan = `<span>${i18next.t('redAmbrosia.purchaseWarning')}</span>`

  return `${nameSpan} <br> ${levelSpan} <br> ${descriptionSpan} <br> ${rewardDescSpan} <br> ${
    (!isMaxLevel) ? `${costNextLevelSpan} <br>` : ''
  } ${spentSpan} <br> ${purchaseWarningSpan}`
}

export const updateMobileRedAmbrosiaHTML = (k: RedAmbrosiaNames) => {
  const elm = DOMCacheGetOrSet('singularityAmbrosiaMultiline')
  elm.innerHTML = redAmbrosiaUpgradeToString(k)
  // MOBILE ONLY - Add a button for buying upgrades
  if (isMobile) {
    const buttonDiv = document.createElement('div')

    const buyOne = document.createElement('button')
    const buyMax = document.createElement('button')

    buyOne.classList.add('modalBtnBuy')
    buyOne.textContent = i18next.t('general.buyOne')
    buyOne.addEventListener('click', (event: MouseEvent) => {
      buyRedAmbrosiaUpgradeLevel(k, event, false)
      updateMobileRedAmbrosiaHTML(k)
    })

    buyMax.classList.add('modalBtnBuy')
    buyMax.textContent = i18next.t('general.buyMax')
    buyMax.addEventListener('click', (event: MouseEvent) => {
      buyRedAmbrosiaUpgradeLevel(k, event, true)
      updateMobileRedAmbrosiaHTML(k)
    })

    buttonDiv.appendChild(buyOne)
    buttonDiv.appendChild(buyMax)
    elm.appendChild(buttonDiv)
  }
}

export const buyRedAmbrosiaUpgradeLevel = async (
  upgradeKey: RedAmbrosiaNames,
  event: MouseEvent,
  buyMax = false
): Promise<void> => {
  const upgrade = redAmbrosiaUpgrades[upgradeKey]
  let purchased = 0
  let maxPurchasable = 1
  let redAmbrosiaBudget = player.redAmbrosia

  if (event.shiftKey || buyMax) {
    maxPurchasable = 100000000
    const buy = Number(
      await Prompt(
        i18next.t('redAmbrosia.redAmbrosiaBuyPrompt', {
          amount: format(player.redAmbrosia, 0, true)
        })
      )
    )

    if (isNaN(buy) || !isFinite(buy) || !Number.isInteger(buy)) {
      // nan + Infinity checks
      return Alert(i18next.t('general.validation.finite'))
    }

    if (buy === -1) {
      redAmbrosiaBudget = player.redAmbrosia
    } else if (buy <= 0) {
      return Alert(i18next.t('octeract.buyLevel.cancelPurchase'))
    } else {
      redAmbrosiaBudget = buy
    }
    redAmbrosiaBudget = Math.min(player.redAmbrosia, redAmbrosiaBudget)
  }

  if (upgrade.maxLevel > 0) {
    maxPurchasable = Math.min(maxPurchasable, upgrade.maxLevel - upgrade.level)
  }

  if (maxPurchasable === 0) {
    return Alert(i18next.t('octeract.buyLevel.alreadyMax'))
  }

  while (maxPurchasable > 0) {
    const cost = getRedAmbrosiaUpgradeCostTNL(upgradeKey)
    if (player.redAmbrosia < cost || redAmbrosiaBudget < cost) {
      break
    } else {
      player.redAmbrosia -= cost
      redAmbrosiaBudget -= cost
      upgrade.redAmbrosiaInvested += cost
      upgrade.level += 1
      purchased += 1
      maxPurchasable -= 1

      // Update the player storage
      player.redAmbrosiaUpgrades[upgradeKey] += cost
    }
  }

  if (purchased === 0) {
    return Alert(i18next.t('octeract.buyLevel.cannotAfford'))
  }
  if (purchased > 1) {
    return Alert(i18next.t('octeract.buyLevel.multiBuy', { n: format(purchased) }))
  }
}

export const displayRedAmbrosiaLevels = () => {
  for (const key of Object.keys(redAmbrosiaUpgrades)) {
    const k = key as RedAmbrosiaNames

    const capKey = key.charAt(0).toUpperCase() + key.slice(1)
    const name = `redAmbrosia${capKey}`
    const elm = DOMCacheGetOrSet(name)
    // There is an image in the elm. find it.
    const img = elm.querySelector('img') as HTMLImageElement
    const level = redAmbrosiaUpgrades[k].level || 0

    img.classList.add('dimmed')
    let levelOverlay = elm.querySelector('.level-overlay') as HTMLDivElement
    if (!levelOverlay) {
      levelOverlay = document.createElement('div')
      levelOverlay.classList.add('level-overlay')

      if (level === redAmbrosiaUpgrades[k].maxLevel) {
        levelOverlay.classList.add('maxRedAmbrosiaLevel')
      } else {
        levelOverlay.classList.add('notMaxRedAmbrosiaLevel')
      }

      elm.classList.add('relative-container') // Apply relative container to the element
      elm.appendChild(levelOverlay) // Append to the element

      levelOverlay.textContent = String(level) // Set the level text
    }
  }
}

export const resetRedAmbrosiaDisplay = () => {
  for (const key of Object.keys(redAmbrosiaUpgrades)) {
    const capKey = key.charAt(0).toUpperCase() + key.slice(1)
    const name = `redAmbrosia${capKey}`
    const elm = DOMCacheGetOrSet(name)
    const img = elm.querySelector('img') as HTMLImageElement
    img.classList.remove('dimmed') // Remove the dimmed class

    // Remove the level overlay if it exists
    const levelOverlay = elm.querySelector('.level-overlay')
    if (levelOverlay) {
      levelOverlay.remove()
      elm.classList.remove('relative-container') // Remove relative container
    }
  }
}
