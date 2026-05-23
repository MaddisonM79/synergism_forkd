import {
  canBuyAntMastery as logicCanBuyAntMastery,
  getBuyableAntMasteryLevels as logicGetBuyableAntMasteryLevels
} from '@synergism/logic'
import { player } from '../../../../Synergism'
import type { AntProducers } from '../../structs/structs'
import { antMasteryData } from '../data/data'
import { getMaxAntMasteryLevel } from './max-level'

export const canBuyAntMastery = (ant: AntProducers): boolean =>
  logicCanBuyAntMastery({
    antData: antMasteryData[ant],
    masteryLevel: player.ants.masteries[ant].mastery,
    maxLevel: getMaxAntMasteryLevel(),
    currentELO: player.ants.rebornELO,
    currentParticles: player.reincarnationPoints
  })

export const getBuyableMasteryLevels = (ant: AntProducers): number =>
  logicGetBuyableAntMasteryLevels({
    antData: antMasteryData[ant],
    masteryLevel: player.ants.masteries[ant].mastery,
    maxLevel: getMaxAntMasteryLevel(),
    currentELO: player.ants.rebornELO,
    currentParticles: player.reincarnationPoints
  })
