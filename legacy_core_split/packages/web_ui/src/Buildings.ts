// Precomputed key tables for the 5-building/4-resource layout.
// Hoisted so the per-tick recompute in tack() (≈200 Hz) does not
// allocate template-literal strings, and so the OneToFive / ZeroToFour
// casts that previously littered the hot loops can be replaced with
// typed indexing.

import type { OneToFive, ZeroToFour } from './types/Synergism'

export type BuildingResource = 'Coin' | 'Diamonds' | 'Mythos'

export const buildingResources = ['Coin', 'Diamonds', 'Mythos'] as const satisfies readonly BuildingResource[]

export const particleOriginalCosts = [1, 1e2, 1e4, 1e8, 1e16] as const

export const ordinalsZeroToFour = [0, 1, 2, 3, 4] as const satisfies readonly ZeroToFour[]
export const ordinalsOneToFive = [1, 2, 3, 4, 5] as const satisfies readonly OneToFive[]

// First-to-fifth ordinal names as a typed tuple. GlobalVariables['ordinals']
// is `readonly ['first', ..., 'eighth', ...string[]]`, so indexing it by a
// `number` widens to `string` and loses the literal type. This 5-name tuple
// keeps the narrow union for the building/particle hot paths.
export const buildingOrdinalNames = [
  'first',
  'second',
  'third',
  'fourth',
  'fifth'
] as const

export const ascendBuildingKeys = [
  'ascendBuilding1',
  'ascendBuilding2',
  'ascendBuilding3',
  'ascendBuilding4',
  'ascendBuilding5'
] as const satisfies readonly `ascendBuilding${OneToFive}`[]

export const buildingCostKeys = {
  Coin: ['firstCostCoin', 'secondCostCoin', 'thirdCostCoin', 'fourthCostCoin', 'fifthCostCoin'],
  Diamonds: ['firstCostDiamonds', 'secondCostDiamonds', 'thirdCostDiamonds', 'fourthCostDiamonds', 'fifthCostDiamonds'],
  Mythos: ['firstCostMythos', 'secondCostMythos', 'thirdCostMythos', 'fourthCostMythos', 'fifthCostMythos']
} as const

export const buildingOwnedKeys = {
  Coin: ['firstOwnedCoin', 'secondOwnedCoin', 'thirdOwnedCoin', 'fourthOwnedCoin', 'fifthOwnedCoin'],
  Diamonds: ['firstOwnedDiamonds', 'secondOwnedDiamonds', 'thirdOwnedDiamonds', 'fourthOwnedDiamonds', 'fifthOwnedDiamonds'],
  Mythos: ['firstOwnedMythos', 'secondOwnedMythos', 'thirdOwnedMythos', 'fourthOwnedMythos', 'fifthOwnedMythos']
} as const

export const particleCostKeys = [
  'firstCostParticles',
  'secondCostParticles',
  'thirdCostParticles',
  'fourthCostParticles',
  'fifthCostParticles'
] as const

export const particleOwnedKeys = [
  'firstOwnedParticles',
  'secondOwnedParticles',
  'thirdOwnedParticles',
  'fourthOwnedParticles',
  'fifthOwnedParticles'
] as const

// Used by the buy-hotkey handler to map a narrowed digit case to OneToFive
// without an unchecked cast.
export const digitToOneToFive = {
  '1': 1,
  '2': 2,
  '3': 3,
  '4': 4,
  '5': 5
} as const satisfies Record<'1' | '2' | '3' | '4' | '5', OneToFive>
