// Pure seeded RNG. Lifted from packages/web_ui/src/RNG.ts.
//
// Legacy web_ui exposes `seededRandom(index)` that reads + post-increments
// `player.seed[index]` and uses `MersenneTwister(seed).random()`. Logic
// can't touch player state, so the migrated function is pure: takes the
// current seed, returns both the random value AND the next seed value.
// The web_ui shim does the read/write against player.seed.
//
// The Seed enum is the index into player.seed[]. It lives here because
// the pure RNG functions consume it and because future Rust state code
// will declare the same indices — keeping it co-located with the RNG
// helpers makes the contract one-stop.

import { MersenneTwister } from 'fast-mersenne-twister'

export const Seed = {
  PromoCodes: 0,
  Ambrosia: 1,
  RedAmbrosia: 2
} as const

export type SeedValues = typeof Seed[keyof typeof Seed]

export interface SeededRandomResult {
  /** Random value in [0, 1) from `MersenneTwister(seed).random()`. */
  value: number
  /** `seed + 1`. Caller writes back to its mutable seed store. */
  newSeed: number
}

/**
 * Single Mersenne Twister roll. Constructs a fresh MT seeded with the
 * input and returns `.random()`. Each call advances the seed by 1.
 * Pure: caller manages the seed lifecycle.
 */
export const seededRandom = (seed: number): SeededRandomResult => ({
  value: MersenneTwister(seed).random(),
  newSeed: seed + 1
})

export interface SeededBetweenResult {
  /** Inclusive integer in [min, max]. */
  value: number
  newSeed: number
}

/**
 * Inclusive integer roll in [min, max]. Same seed-advance semantics
 * as `seededRandom`.
 */
export const seededBetween = (seed: number, min: number, max: number): SeededBetweenResult => {
  const { value, newSeed } = seededRandom(seed)
  return {
    value: Math.floor(value * (max - min + 1) + min),
    newSeed
  }
}
