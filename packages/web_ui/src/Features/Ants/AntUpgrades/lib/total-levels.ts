import { calculateTrueAntLevel as logicCalculateTrueAntLevel } from '@synergism/logic'
import { player } from '../../../../Synergism'
import { antUpgradeData } from '../data/data'
import type { AntUpgrades } from '../structs/structs'
import { computeFreeAntUpgradeLevels } from './free-levels'

export const calculateTrueAntLevel = (antUpgrade: AntUpgrades): number =>
  logicCalculateTrueAntLevel({
    currentLevel: player.ants.upgrades[antUpgrade],
    freeLevels: computeFreeAntUpgradeLevels(),
    exemptFromCorruption: antUpgradeData[antUpgrade].exemptFromCorruption,
    corruptionExtinctionDivisor: player.corruptions.used.corruptionEffects('extinction'),
    c11Active: player.currentChallenge.ascension === 11
  })
