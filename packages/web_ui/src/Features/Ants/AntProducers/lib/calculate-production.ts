import Decimal from 'break_infinity.js'
import { calculateBaseAntsToBeGenerated as logicCalculateBaseAntsToBeGenerated } from '@synergism/logic'
import { player } from '../../../../Synergism'
import { calculateSelfSpeedFromMastery } from '../../AntMasteries/lib/ant-speed'
import type { AntProducers } from '../../structs/structs'
import { antProducerData } from '../data/data'

export const calculateBaseAntsToBeGenerated = (ant: AntProducers, antSpeedMult = Decimal.fromString('1')): Decimal =>
  logicCalculateBaseAntsToBeGenerated({
    generated: player.ants.producers[ant].generated,
    purchased: player.ants.producers[ant].purchased,
    baseProduction: antProducerData[ant].baseProduction,
    selfSpeedMult: calculateSelfSpeedFromMastery(ant),
    antSpeedMult
  })
