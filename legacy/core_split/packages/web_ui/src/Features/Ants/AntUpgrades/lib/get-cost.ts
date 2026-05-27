import {
  antUpgradeBaseCosts,
  antUpgradeCostIncreaseExponents,
  getCostMaxAntUpgrades as logicGetCostMaxAntUpgrades,
  getCostNextAntUpgrade as logicGetCostNextAntUpgrade,
  getMaxPurchasableAntUpgrades as logicGetMaxPurchasableAntUpgrades
} from '@synergism/logic'
import type Decimal from 'break_infinity.js'
import { player } from '../../../../Synergism'
import type { AntUpgrades } from '../structs/structs'

export const getCostNextAntUpgrade = (antUpgrade: AntUpgrades) =>
  logicGetCostNextAntUpgrade({
    baseCost: antUpgradeBaseCosts[antUpgrade],
    costIncreaseExponent: antUpgradeCostIncreaseExponents[antUpgrade],
    currentLevel: player.ants.upgrades[antUpgrade]
  })

export const getCostMaxAntUpgrades = (antUpgrade: AntUpgrades) => {
  const maxBuyable = logicGetMaxPurchasableAntUpgrades({
    baseCost: antUpgradeBaseCosts[antUpgrade],
    costIncreaseExponent: antUpgradeCostIncreaseExponents[antUpgrade],
    currentLevel: player.ants.upgrades[antUpgrade],
    budget: player.ants.crumbs
  })
  return logicGetCostMaxAntUpgrades({
    baseCost: antUpgradeBaseCosts[antUpgrade],
    costIncreaseExponent: antUpgradeCostIncreaseExponents[antUpgrade],
    currentLevel: player.ants.upgrades[antUpgrade],
    maxBuyable
  })
}

export const getMaxPurchasableAntUpgrades = (antUpgrade: AntUpgrades, budget: Decimal): number =>
  logicGetMaxPurchasableAntUpgrades({
    baseCost: antUpgradeBaseCosts[antUpgrade],
    costIncreaseExponent: antUpgradeCostIncreaseExponents[antUpgrade],
    currentLevel: player.ants.upgrades[antUpgrade],
    budget
  })
