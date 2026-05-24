import {
  calculateSingularityPerkELO as logicCalculateSingularityPerkELO,
  singularityELOBonusMult as logicSingularityELOBonusMult
} from '@synergism/logic'
import { player } from '../../../../../../../Synergism'

/**
 * @returns Value of perk "Advanced... Cheating Tactics?"
 */
export const singularityELOBonusMult = () =>
  logicSingularityELOBonusMult(player.singularityCount)

/**
 * @returns Value of perk "Invigorated Spirits!"
 */
export const calculateSingularityPerkELO = () =>
  logicCalculateSingularityPerkELO({
    singCount: player.singularityCount,
    immortalELO: player.ants.immortalELO
  })
