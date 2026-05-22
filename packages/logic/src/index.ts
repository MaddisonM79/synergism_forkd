// Public API surface for @synergism/logic.
//
// Re-export the pieces that the UI tier is allowed to consume here. Anything
// not exported from this file should be considered internal to the package.

export type { CoreEvent } from './events/types'
export type {
  AcceleratorState,
  AscendBuildingState,
  BuyAmount,
  MultiplierState,
  ParticleBuildingsState,
  TesseractBuildingsState
} from './state/schema'
export type {
  BuyAcceleratorInput,
  GetCostAcceleratorInput
} from './mechanics/accelerators'
export type {
  BuyMultiplierInput,
  GetCostMultiplierInput
} from './mechanics/multipliers'
export { buyAccelerator, getCostAccelerator } from './mechanics/accelerators'
export type { GetAcceleratorBoostCostInput } from './mechanics/acceleratorBoosts'
export { getAcceleratorBoostCost } from './mechanics/acceleratorBoosts'
export { buyMultiplier, getCostMultiplier } from './mechanics/multipliers'
export type {
  GetProducerCostInput,
  ProducerIndex,
  ProducerType
} from './mechanics/producers'
export { getProducerCost } from './mechanics/producers'
export type {
  BuyParticleBuildingInput,
  GetParticleCostInput,
  ParticleBuildingIndex
} from './mechanics/particleBuildings'
export { buyParticleBuilding, getParticleCost } from './mechanics/particleBuildings'
export type {
  BuyTesseractBuildingInput,
  GetTesseractCostInput,
  TesseractBuildingIndex,
  TesseractBuildings
} from './mechanics/tesseractBuildings'
export {
  buyTesseractBuilding,
  calculateTessBuildingsInBudget,
  getTesseractCost
} from './mechanics/tesseractBuildings'
