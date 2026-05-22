// Public API surface for @synergism/logic.
//
// Re-export the pieces that the UI tier is allowed to consume here. Anything
// not exported from this file should be considered internal to the package.

export type { CoreEvent } from './events/types'
export type { AcceleratorState, BuyAmount } from './state/schema'
export type {
  BuyAcceleratorInput,
  GetCostAcceleratorInput
} from './mechanics/accelerators'
export { buyAccelerator, getCostAccelerator } from './mechanics/accelerators'
