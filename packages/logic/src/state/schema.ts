import type { Decimal } from '../math/bignum'

// Player-configurable per-click purchase cap. Mirrors the UI's "x1 / x10 /
// x100 / ..." selector — see packages/web_ui/src/types/Synergism.ts.
export type BuyAmount = 1 | 10 | 100 | 1000 | 10_000 | 100_000

// Slice of GameState read/written by the accelerator-purchase machinery.
//
// State is sliced per-mechanic rather than as a single monolithic GameState —
// fields land here as their owning mechanics migrate, and the slices compose
// into the full state shape over time.
export interface AcceleratorState {
  acceleratorBought: number
  acceleratorCost: Decimal
  coins: Decimal
  /** Set false once any accelerator is owned; gates a no-accelerator-prestige achievement. */
  prestigenoaccelerator: boolean
  /** Same flag, transcension lineage. */
  transcendnoaccelerator: boolean
  /** Same flag, reincarnation lineage. */
  reincarnatenoaccelerator: boolean
}

// Slice of GameState read/written by the multiplier-purchase machinery.
// Mirror of AcceleratorState — same flag pattern, different field names.
export interface MultiplierState {
  multiplierBought: number
  multiplierCost: Decimal
  coins: Decimal
  prestigenomultiplier: boolean
  transcendnomultiplier: boolean
  reincarnatenomultiplier: boolean
}

// Slice of GameState read/written by the particle-building-purchase machinery.
// Five positions (first..fifth) each have an owned count + a current cost; the
// shared resource is reincarnationPoints. No no-purchase-flag bookkeeping —
// particle buildings don't gate any "didn't buy" achievements like the
// accelerator/multiplier mechanics do.
export interface ParticleBuildingsState {
  reincarnationPoints: Decimal
  firstOwnedParticles: number
  firstCostParticles: Decimal
  secondOwnedParticles: number
  secondCostParticles: Decimal
  thirdOwnedParticles: number
  thirdCostParticles: Decimal
  fourthOwnedParticles: number
  fourthCostParticles: Decimal
  fifthOwnedParticles: number
  fifthCostParticles: Decimal
}
