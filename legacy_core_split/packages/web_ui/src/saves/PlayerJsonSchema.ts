import { z } from 'zod'
import { CampaignManager } from '../Campaign'
import { type Corruptions, CorruptionLoadout, CorruptionSaves } from '../Corruptions'
import { WowCubes, WowHypercubes, WowPlatonicCubes, WowTesseracts } from '../CubeExperimental'
import { QuarkHandler } from '../Quark'
import { SingularityChallenge } from '../SingularityChallenges'
import type { Player } from '../types/Synergism'
import { playerSchema } from './PlayerSchema'

export const convertArrayToCorruption = (array: number[]): Corruptions => {
  return {
    viscosity: array[2],
    drought: array[8],
    deflation: array[6],
    extinction: array[7],
    illiteracy: array[5],
    recession: array[9],
    dilation: array[3],
    hyperchallenge: array[4]
  }
}

// Save-direction (serialize) schema: extends the base playerSchema and
// overrides the runtime-shaped fields with serializers that convert class
// instances back into JSON-safe primitives.
//
// Each override uses z.custom<X>(v => v instanceof X) — not the previous
// z.any() — so a runtime save of an unexpected shape (e.g. a partially-
// restored player after a botched migration) surfaces a structured schema
// error instead of silently coercing whatever was passed in. Closes #9 (S2).
//
// We use z.custom rather than z.instanceof because PlayerJsonSchema sits
// inside the existing circular import cluster (#76 / T3). z.instanceof(X)
// evaluates X eagerly as a function argument at module load and hits the
// TDZ for classes whose modules are still resolving. The z.custom form puts
// the class reference inside the validator closure, which is only invoked
// once safeParse runs and the cycle has settled.
const isCampaignManager = (v: unknown): v is CampaignManager => v instanceof CampaignManager
const isCorruptionLoadout = (v: unknown): v is CorruptionLoadout => v instanceof CorruptionLoadout
const isCorruptionSaves = (v: unknown): v is CorruptionSaves => v instanceof CorruptionSaves
const isQuarkHandler = (v: unknown): v is QuarkHandler => v instanceof QuarkHandler
const isSingularityChallenge = (v: unknown): v is SingularityChallenge => v instanceof SingularityChallenge
const isWowCubes = (v: unknown): v is WowCubes => v instanceof WowCubes
const isWowHypercubes = (v: unknown): v is WowHypercubes => v instanceof WowHypercubes
const isWowPlatonicCubes = (v: unknown): v is WowPlatonicCubes => v instanceof WowPlatonicCubes
const isWowTesseracts = (v: unknown): v is WowTesseracts => v instanceof WowTesseracts

export const playerJsonSchema = playerSchema.extend({
  codes: z.custom<Player['codes']>((v) => v instanceof Map).transform((codes) => Array.from(codes)),
  worlds: z.custom<QuarkHandler>(isQuarkHandler).transform((worlds) => Number(worlds)),
  wowCubes: z.custom<WowCubes>(isWowCubes).transform((cubes) => Number(cubes)),
  wowTesseracts: z.custom<WowTesseracts>(isWowTesseracts).transform((tesseracts) => Number(tesseracts)),
  wowHypercubes: z.custom<WowHypercubes>(isWowHypercubes).transform((hypercubes) => Number(hypercubes)),
  wowPlatonicCubes: z.custom<WowPlatonicCubes>(isWowPlatonicCubes).transform((cubes) => Number(cubes)),

  campaigns: z.custom<CampaignManager>(isCampaignManager).transform((campaigns) => campaigns.campaignManagerData),
  /*campaigns: playerCampaignSchema.transform((campaignManager: Player['campaigns']) => {
    return {
      currentCampaign: campaignManager.current,
      campaigns: campaignManager.allC10Completions,
    }
  }),*/

  // Platonic (or somebody I'm so tired): Figure out why the hell using `playerCorruptionsSchema` does not work for saves
  // But it does work for the other three fields.
  corruptions: z.object({
    used: z.custom<CorruptionLoadout>(isCorruptionLoadout),
    next: z.custom<CorruptionLoadout>(isCorruptionLoadout),
    saves: z.custom<CorruptionSaves>(isCorruptionSaves),
    showStats: z.boolean()
  }).transform((stuff) => ({
    used: stuff.used.loadout,
    next: stuff.next.loadout,
    saves: stuff.saves.corrSaveData,
    showStats: stuff.showStats
  })),

  singularityChallenges: z.record(
    z.string(),
    z.custom<SingularityChallenge>(isSingularityChallenge)
  ).transform((challenges) =>
    Object.fromEntries(
      Object.entries(challenges).map(([key, value]) => [
        key,
        {
          completions: value.completions,
          highestSingularityCompleted: value.highestSingularityCompleted,
          enabled: value.enabled
        }
      ])
    )
  ),

  dayCheck: z.custom<Date | null>((v) => v === null || v instanceof Date).transform((dayCheck) =>
    dayCheck?.toISOString() ?? null
  )
})
