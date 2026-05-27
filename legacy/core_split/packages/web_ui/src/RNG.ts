import {
  Seed,
  seededBetween as logicSeededBetween,
  seededRandom as logicSeededRandom,
  type SeedValues
} from '@synergism/logic'
import { player } from './Synergism'

export { Seed }

export const seededRandom = (index: SeedValues): number => {
  const { value, newSeed } = logicSeededRandom(player.seed[index])
  player.seed[index] = newSeed
  return value
}

/**
 * Generates a random number (inclusive) between {@param min} and {@param max}.
 */
export const seededBetween = (index: SeedValues, min: number, max: number): number => {
  const { value, newSeed } = logicSeededBetween(player.seed[index], min, max)
  player.seed[index] = newSeed
  return value
}
