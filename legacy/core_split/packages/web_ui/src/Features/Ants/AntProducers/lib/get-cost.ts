import type Decimal from 'break_infinity.js'
import {
  getCostMaxAntProducers as logicGetCostMaxAntProducers,
  getCostNextAntProducer as logicGetCostNextAntProducer,
  getMaxPurchasableAntProducers as logicGetMaxPurchasableAntProducers
} from '@synergism/logic'
import { player } from '../../../../Synergism'
import type { AntProducers } from '../../structs/structs'
import { antProducerData } from '../data/data'

export const getCostNextAnt = (ant: AntProducers): Decimal =>
  logicGetCostNextAntProducer({
    baseCost: antProducerData[ant].baseCost,
    costIncrease: antProducerData[ant].costIncrease,
    purchased: player.ants.producers[ant].purchased
  })

export const getCostMaxAnts = (ant: AntProducers): Decimal => {
  const data = antProducerData[ant]
  const maxBuyable = logicGetMaxPurchasableAntProducers({
    baseCost: data.baseCost,
    costIncrease: data.costIncrease,
    purchased: player.ants.producers[ant].purchased,
    budget: player.ants.crumbs
  })
  return logicGetCostMaxAntProducers({
    baseCost: data.baseCost,
    costIncrease: data.costIncrease,
    purchased: player.ants.producers[ant].purchased,
    maxBuyable
  })
}

export const getMaxPurchasableAnts = (ant: AntProducers, budget: Decimal): number =>
  logicGetMaxPurchasableAntProducers({
    baseCost: antProducerData[ant].baseCost,
    costIncrease: antProducerData[ant].costIncrease,
    purchased: player.ants.producers[ant].purchased,
    budget
  })
