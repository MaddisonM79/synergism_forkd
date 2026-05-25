import { antMasteryData as logicAntMasteryData } from '@synergism/logic'
import { AntProducers } from '../../structs/structs'
import type { AntMasteryData } from '../structs/structs'

// Re-exported from @synergism/logic as a Record<AntProducers, AntMasteryData>
// so existing call sites that index by `AntProducers.X` keep compiling
// unchanged. Logic stores the table as a 9-tuple indexed 0..8, which lines
// up with the AntProducers enum values exactly.
export const antMasteryData: Record<AntProducers, AntMasteryData> = {
  [AntProducers.Workers]: logicAntMasteryData[0],
  [AntProducers.Breeders]: logicAntMasteryData[1],
  [AntProducers.MetaBreeders]: logicAntMasteryData[2],
  [AntProducers.MegaBreeders]: logicAntMasteryData[3],
  [AntProducers.Queens]: logicAntMasteryData[4],
  [AntProducers.LordRoyals]: logicAntMasteryData[5],
  [AntProducers.Almighties]: logicAntMasteryData[6],
  [AntProducers.Disciples]: logicAntMasteryData[7],
  [AntProducers.HolySpirit]: logicAntMasteryData[8]
}
