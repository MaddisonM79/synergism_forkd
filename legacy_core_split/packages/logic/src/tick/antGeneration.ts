// Per-tick ant production. Lifted from
// packages/web_ui/src/Features/Ants/AntProducers/lib/generate-ant-producers.ts
// (the `generateAntsAndCrumbs` function, producer loop + crumb math).
//
// Web_ui's pre-migration shape:
//   - Loop from HolySpirit (8) down to Breeders (1): each tier reads its
//     own producers state + writes to (tier-1).generated, using
//     calculateBaseAntsToBeGenerated × dt.
//   - After the loop, Workers (0) computes crumbsToGenerate from its own
//     (now-updated) generated count and adds to the three crumb fields.
//   - Finally calls activateELO(dt) — stays in web_ui (depends on
//     Date.now, getLotusTimeExpiresAt, updateAntLeaderboards DOM,
//     player.worlds.add quark crediting).
//
// The producer loop has a real iteration dependency: iter for antType=N
// writes to (N-1).generated, and iter for antType=N-1 (next iteration,
// since the loop decrements) reads (N-1).generated for its own base
// computation. We preserve this by mutating a local `updatedGenerated`
// array in place across iterations.

import { Decimal } from '../math/bignum'
import { antMasteryData, calculateSelfSpeedFromMastery } from '../mechanics/antMasteries'
import { antProducerData, calculateBaseAntsToBeGenerated } from '../mechanics/antProducers'

/** AntProducers.HolySpirit (top of producer chain). Web_ui's enum mirrors
 * this — Workers=0, Breeders=1, ..., HolySpirit=8. We use bare numbers
 * here to avoid coupling logic to the web_ui enum. */
const LAST_ANT_PRODUCER = 8

export interface GenerateAntsAndCrumbsProducerInput {
  /** player.ants.producers[i].generated — current generated count (Decimal). */
  generated: Decimal
  /** player.ants.producers[i].purchased — manually purchased count (number). */
  purchased: number
  /** player.ants.masteries[i].mastery — current mastery level (0..12). */
  masteryLevel: number
}

export interface GenerateAntsAndCrumbsInput {
  /** Tick delta in seconds. */
  dt: number
  /** Pre-evaluated `calculateActualAntSpeedMult()` — outer speed multiplier
   * applied to every producer's base generation. */
  antSpeedMult: Decimal
  /** 9-entry array indexed 0..8 (Workers..HolySpirit). */
  producers: readonly GenerateAntsAndCrumbsProducerInput[]
  /** player.ants.crumbs — running total for sacrifice consumption. */
  crumbs: Decimal
  /** player.ants.crumbsThisSacrifice — resets on ant sacrifice. */
  crumbsThisSacrifice: Decimal
  /** player.ants.crumbsEverMade — lifetime accumulator (never resets). */
  crumbsEverMade: Decimal
}

export interface GenerateAntsAndCrumbsResult {
  /** Updated `generated` values for each tier, indexed 0..8. */
  producersGenerated: readonly Decimal[]
  crumbs: Decimal
  crumbsThisSacrifice: Decimal
  crumbsEverMade: Decimal
}

/**
 * Per-tick ant production. The producer loop iterates from
 * LAST_ANT_PRODUCER (HolySpirit) down to 1 (Breeders), each iteration
 * crediting the tier below via `baseGeneration × dt`. After the loop,
 * Workers (tier 0) produces crumbs that fan out into three accumulators.
 *
 * Loop iteration N reads `updatedGenerated[N]` (which may have been
 * updated by a higher-tier iteration earlier) and writes
 * `updatedGenerated[antProducerData[N].produces]`.
 */
export function generateAntsAndCrumbs (input: GenerateAntsAndCrumbsInput): GenerateAntsAndCrumbsResult {
  // Local mutable copy of the generated values — the loop writes here.
  const updatedGenerated: Decimal[] = input.producers.map((p) => p.generated)

  // Producer loop: each higher-tier produces the tier directly below.
  for (let antType = LAST_ANT_PRODUCER; antType > 0; antType--) {
    const selfSpeedMult = calculateSelfSpeedFromMastery({
      antData: antMasteryData[antType],
      masteryLevel: input.producers[antType].masteryLevel,
      purchased: input.producers[antType].purchased
    })
    const baseGeneration = calculateBaseAntsToBeGenerated({
      generated: updatedGenerated[antType],
      purchased: input.producers[antType].purchased,
      baseProduction: antProducerData[antType].baseProduction,
      selfSpeedMult,
      antSpeedMult: input.antSpeedMult
    })
    const producedAnt = antProducerData[antType].produces
    if (producedAnt === undefined) {
      // Shouldn't happen for antType > 0, but guard for type safety.
      continue
    }
    updatedGenerated[producedAnt] = updatedGenerated[producedAnt].add(
      baseGeneration.times(input.dt)
    )
  }

  // Workers (tier 0) produces crumbs separately. Uses the
  // already-updated Workers.generated from the loop above.
  const workersSelfSpeed = calculateSelfSpeedFromMastery({
    antData: antMasteryData[0],
    masteryLevel: input.producers[0].masteryLevel,
    purchased: input.producers[0].purchased
  })
  const crumbsToGenerate = calculateBaseAntsToBeGenerated({
    generated: updatedGenerated[0],
    purchased: input.producers[0].purchased,
    baseProduction: antProducerData[0].baseProduction,
    selfSpeedMult: workersSelfSpeed,
    antSpeedMult: input.antSpeedMult
  }).times(input.dt)

  return {
    producersGenerated: updatedGenerated,
    crumbs: Decimal.add(input.crumbs, crumbsToGenerate),
    crumbsThisSacrifice: Decimal.add(input.crumbsThisSacrifice, crumbsToGenerate),
    crumbsEverMade: Decimal.add(input.crumbsEverMade, crumbsToGenerate)
  }
}
