import { calculateImmortalELOGain as logicCalculateImmortalELOGain } from '@synergism/logic'
import { player } from '../../../../../../../Synergism'
import { calculateEffectiveAntELO } from '../../AntELO/lib/calculate'

export const calculateImmortalELOGain = (): number =>
  logicCalculateImmortalELOGain({
    effectiveELO: calculateEffectiveAntELO(),
    immortalELO: player.ants.immortalELO
  })
