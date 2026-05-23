import type Decimal from 'break_infinity.js'
import { calculateSelfSpeedFromMastery as logicCalculateSelfSpeedFromMastery } from '@synergism/logic'
import { player } from '../../../../Synergism'
import type { AntProducers } from '../../structs/structs'
import { antMasteryData } from '../data/data'

export const calculateSelfSpeedFromMastery = (ant: AntProducers): Decimal =>
  logicCalculateSelfSpeedFromMastery({
    antData: antMasteryData[ant],
    masteryLevel: player.ants.masteries[ant].mastery,
    purchased: player.ants.producers[ant].purchased
  })
