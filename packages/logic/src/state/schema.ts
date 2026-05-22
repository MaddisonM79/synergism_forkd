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

// Player's accumulated hypercube blessings. Inputs to the 10 calculate*
// HypercubeBlessing functions. Mirrors player.hypercubeBlessings; each
// function reads exactly one field.
export interface HypercubeBlessings {
  accelerator: number
  multiplier: number
  offering: number
  runeExp: number
  obtainium: number
  antSpeed: number
  antSacrifice: number
  antELO: number
  talismanBonus: number
  globalSpeed: number
}

// Player's accumulated platonic-cube blessings. Inputs to the 8 calculate*
// PlatonicBlessing functions in mechanics/cubes/platonicBlessings.ts. Mirrors
// player.platonicBlessings; each function reads exactly one field.
export interface PlatonicBlessings {
  cubes: number
  tesseracts: number
  hypercubes: number
  platonics: number
  hypercubeBonus: number
  taxes: number
  globalSpeed: number
}

// Slice of GameState read/written by buyCrystalUpgrades. prestigeShards is
// the spend resource; crystalUpgrades[u] holds the current level for each
// crystal upgrade index (0-based). Caller passes 1-based `i` as input — the
// function does the -1 internally.
export interface CrystalUpgradesState {
  prestigeShards: Decimal
  crystalUpgrades: number[]
}

// Slice of GameState read/written by buyUpgrades. All four reset-tier resources
// live here so the function can dispatch on the upgrade tier without taking
// four overloads. The seven `*no*upgrades` flags are achievement gates that
// flip false depending on the tier purchased — see mechanics/upgrades.ts for
// the per-tier flip matrix.
export interface UpgradesState {
  coins: Decimal
  prestigePoints: Decimal
  transcendPoints: Decimal
  reincarnationPoints: Decimal
  /** Bitmap of owned upgrades; 0 = unowned, 1 = owned. Indexed by `pos`. */
  upgrades: number[]
  prestigenocoinupgrades: boolean
  transcendnocoinupgrades: boolean
  transcendnocoinorprestigeupgrades: boolean
  reincarnatenocoinupgrades: boolean
  reincarnatenocoinorprestigeupgrades: boolean
  reincarnatenocoinprestigeortranscendupgrades: boolean
  reincarnatenocoinprestigetranscendorgeneratorupgrades: boolean
}

// Generic slice for one producer family (Coin / Diamonds / Mythos / Particles).
// Field names are family-agnostic — the shim translates between the typed
// player fields (firstOwnedCoin, firstCostDiamonds, etc.) and this shape.
// Used by buyMax in mechanics/producers.ts.
export interface ProducerFamilyState {
  /** Resource the family buys with (coins / prestigePoints / transcendPoints / reincarnationPoints). */
  resource: Decimal
  firstOwned: number
  firstCost: Decimal
  secondOwned: number
  secondCost: Decimal
  thirdOwned: number
  thirdCost: Decimal
  fourthOwned: number
  fourthCost: Decimal
  fifthOwned: number
  fifthCost: Decimal
}

// One position of the ascension-tier building family (tesseract buildings).
// Subset of the player's ascendBuilding{N} shape — only the fields the buy
// machinery touches; generated/multiplier stay in web_ui until those mechanics
// migrate.
export interface AscendBuildingState {
  owned: number
  cost: number
}

// Slice of GameState read/written by the tesseract-building-purchase
// machinery. wowTesseracts is the spend resource (mirrored as a number via
// Number(player.wowTesseracts) at the boundary — the WowTesseracts wrapper
// class stays in web_ui).
export interface TesseractBuildingsState {
  wowTesseracts: number
  ascendBuilding1: AscendBuildingState
  ascendBuilding2: AscendBuildingState
  ascendBuilding3: AscendBuildingState
  ascendBuilding4: AscendBuildingState
  ascendBuilding5: AscendBuildingState
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
