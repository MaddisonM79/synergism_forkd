import type { Decimal } from '../math/bignum'

// Player-configurable per-click purchase cap. Mirrors the UI's "x1 / x10 /
// x100 / ..." selector — see packages/web_ui/src/types/Synergism.ts.
export type BuyAmount = 1 | 10 | 100 | 1000 | 10_000 | 100_000

// Slice of GameState read/written by the accelerator-purchase machinery.
//
// This intentionally starts as a per-mechanic slice rather than a single
// monolithic GameState — fields land here as their owning mechanics migrate,
// and the slices compose into the full state shape over time.
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
