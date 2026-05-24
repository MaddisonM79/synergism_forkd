// Reborn-ELO stages + per-stage modifiers + total-production math + the two
// ant-related singularity perks (ELO bonus mult & invigorated-spirits ELO
// gift). Lifted from:
//   packages/web_ui/src/Features/Ants/AntSacrifice/Rewards/ELO/RebornELO/
//     Stages/lib/threshold.ts
//   packages/web_ui/src/Features/Ants/AntSacrifice/Rewards/ELO/RebornELO/
//     lib/calculate.ts (pure portions only)
//   packages/web_ui/src/Features/Ants/AntSacrifice/Rewards/ELO/AntELO/
//     lib/singularity-perk.ts
//
// The reborn-speed-mult-stats reducer and the leaderboard/quark math stay in
// web_ui because they depend on stats arrays / player.worlds.applyBonus(),
// which are themselves UI-coupled.

// ─── Tranches & constants ─────────────────────────────────────────────────

/**
 * Reborn-ELO tranches. Each tranche covers `stages` levels; each level inside
 * the tranche costs `perStage` ELO; reaching that level grants `quarkPerStage`
 * lifetime quarks (used by the daily-quark reward).
 */
export interface RebornELOTranche {
  stages: number
  perStage: number
  quarkPerStage: number
}

export const rebornELOThresholdTranches: readonly RebornELOTranche[] = [
  { stages: 100, perStage: 100, quarkPerStage: 1 },
  { stages: 100, perStage: 1000, quarkPerStage: 2 },
  { stages: 100, perStage: 3000, quarkPerStage: 3 },
  { stages: 700, perStage: 20000, quarkPerStage: 4 },
  { stages: Number.POSITIVE_INFINITY, perStage: 100000, quarkPerStage: 7 }
]

/** Multiplier applied per stage to the daily-quark reward. */
export const quarkMultiplierPerRebornELOThreshold = 1.002

/** Base per-stage modifier values (before exponentiation by stage count). */
export const perRebornELOStageModifiers = {
  rebornSpeedMult: 0.98,
  antSacrificeObtainiumMult: 1.05,
  antSacrificeOfferingMult: 1.05,
  antSacrificeTalismanFragmentMult: 1.2
}

/** Singularity counts at which the reborn-speed-perk tier advances. Index `i`
 * means: at singCount ≥ this, you've earned tier `i`. */
export const rebornSpeedPerkLevels = [1, 9, 25, 49, 81, 121, 169, 196, 225, 256, 289]

/** Singularity counts at which the ELO-bonus-mult perk tier advances. */
export const singularityELOBonusMultLevels = [3, 11, 27, 51, 83, 123, 171, 198, 227, 258, 291]

/** Singularity counts at which the invigorated-spirits ELO perk tier advances. */
export const singularityPerkELOLevels = [2, 10, 26, 50, 82, 122, 170, 197, 226, 257, 290]

// ─── Stage / speed math ───────────────────────────────────────────────────

/**
 * Reborn-speed perk modifier — staircase lookup over the perk levels.
 * Each tier above the base adds 0.00009; the base tier (i=0) starts at
 * 0.0001. Returns 0 for singCount below the first level.
 */
export function singularityRebornSpeedMultModifier (singCount: number): number {
  for (let i = rebornSpeedPerkLevels.length - 1; i >= 0; i--) {
    if (singCount >= rebornSpeedPerkLevels[i]) {
      return 0.0001 + 0.00009 * i
    }
  }
  return 0
}

/**
 * Per-stage reborn-speed multiplier. Floor of `1` to avoid the perk making
 * stages free. Base 0.98 ÷ singularity-perk increase.
 */
export function calculateStageRebornSpeedMult (singCount: number): number {
  const base = perRebornELOStageModifiers.rebornSpeedMult
  const increase = singularityRebornSpeedMultModifier(singCount)
  return Math.min(1, base + increase)
}

// ─── Threshold (stage-count) computation ──────────────────────────────────

/**
 * How many reborn-ELO stages have been reached for a given ELO total. Walks
 * the tranche list, consuming ELO at each tranche's per-stage cost until
 * either ELO runs out or the tranche is exhausted.
 */
export function calculateRebornELOThresholds (rebornELO: number): number {
  let rebornELOBudget = rebornELO
  let thresholds = 0
  for (const tranche of rebornELOThresholdTranches) {
    const stagesAdded = Math.min(tranche.stages, Math.floor(rebornELOBudget / tranche.perStage))
    thresholds += stagesAdded
    rebornELOBudget -= stagesAdded * tranche.perStage
    if (stagesAdded < tranche.stages) {
      break
    }
  }
  return thresholds
}

/**
 * ELO required to reach the *next* stage boundary, given current ELO. The
 * optional `stage` short-circuits the inner re-calculation if the caller
 * already has it.
 */
export function calculateToNextRebornELOThreshold (rebornELO: number, stage?: number): number {
  const thresholds = stage ?? calculateRebornELOThresholds(rebornELO)
  let stagesChecked = 0
  let tempELO = rebornELO
  for (const tranche of rebornELOThresholdTranches) {
    if (thresholds < stagesChecked + tranche.stages) {
      const reqELOThisThreshold = tranche.perStage
      return (1 + Math.floor(tempELO / reqELOThisThreshold)) * reqELOThisThreshold - tempELO
    }
    stagesChecked += tranche.stages
    tempELO -= tranche.stages * tranche.perStage
  }
  throw new Error('Unreachable code in calculateToNextRebornELOThreshold')
}

/**
 * ELO that's been accumulated but doesn't yet contribute to a completed
 * stage (progress within the current stage boundary).
 */
export function calculateLeftoverRebornELO (rebornELO: number, stage?: number): number {
  const thresholds = stage ?? calculateRebornELOThresholds(rebornELO)
  let usedELO = 0
  let stagesChecked = 0
  for (const tranche of rebornELOThresholdTranches) {
    const stagesInThisTranche = Math.min(tranche.stages, thresholds - stagesChecked)
    usedELO += stagesInThisTranche * tranche.perStage
    stagesChecked += stagesInThisTranche
    if (stagesChecked >= thresholds) {
      break
    }
  }
  return rebornELO - usedELO
}

// ─── Stage-modifier aggregator ────────────────────────────────────────────

export interface RebornELOStageModifiers {
  rebornSpeedMult: number
  antSacrificeObtainiumMult: number
  antSacrificeOfferingMult: number
  antSacrificeTalismanFragmentMult: number
}

export interface RebornELOStageModifiersInput {
  rebornELO: number
  singCount: number
}

/**
 * Per-stage multipliers raised to the current stage count — the cumulative
 * effect of all reached thresholds. Reborn-speed compounds at the
 * per-stage rate (which itself depends on the singularity perk).
 */
export function rebornELOStageModifiers (input: RebornELOStageModifiersInput): RebornELOStageModifiers {
  const thresholds = calculateRebornELOThresholds(input.rebornELO)
  return {
    rebornSpeedMult: Math.pow(calculateStageRebornSpeedMult(input.singCount), thresholds),
    antSacrificeObtainiumMult: Math.pow(perRebornELOStageModifiers.antSacrificeObtainiumMult, thresholds),
    antSacrificeOfferingMult: Math.pow(perRebornELOStageModifiers.antSacrificeOfferingMult, thresholds),
    antSacrificeTalismanFragmentMult: Math.pow(
      perRebornELOStageModifiers.antSacrificeTalismanFragmentMult,
      thresholds
    )
  }
}

// ─── Available / total-production helpers ─────────────────────────────────

export interface AvailableRebornELOInput {
  immortalELO: number
  rebornELO: number
}

/**
 * Immortal ELO that has not yet been activated as Reborn ELO. Floor at 0
 * (rebornELO can technically exceed immortalELO mid-frame).
 */
export function calculateAvailableRebornELO (input: AvailableRebornELOInput): number {
  return Math.max(0, input.immortalELO - input.rebornELO)
}

/**
 * Closed-form geometric-series sum of `r^startIndex + r^(startIndex+1) +
 * ... + r^endIndex`. When `r === 1`, returns the trivial linear sum. Lifted
 * from web_ui's Utility.geometricSeries so this module stays self-contained
 * (the formula is small; duplicating it here keeps the boundary clean).
 */
function geometricSeries (startIndex: number, endIndex: number, ratio: number): number {
  if (endIndex < startIndex) return 0
  if (ratio === 1) return endIndex - startIndex + 1
  return (Math.pow(ratio, endIndex + 1) - Math.pow(ratio, startIndex)) / (ratio - 1)
}

export interface TotalProductionForRebornELOInput {
  rebornELO: number
  /** calculateStageRebornSpeedMult(singCount) — the per-stage speed mult. */
  stageRebornSpeedMult: number
}

/**
 * Total ELO-equivalent production required to reach `rebornELO`. Each
 * tranche contributes a geometric-series sum: each stage costs `perStage`
 * production weighted by (1/stageRebornSpeedMult)^stageIndex.
 */
export function calculateTotalProductionForRebornELO (input: TotalProductionForRebornELOInput): number {
  const stage = calculateRebornELOThresholds(input.rebornELO)
  const leftover = calculateLeftoverRebornELO(input.rebornELO, stage)

  // Reciprocal: you need 1/modifier times as much production to get the same ELO/sec.
  const perStageMult = 1 / input.stageRebornSpeedMult

  let production = 0
  let stagesSpent = 0
  for (const tranch of rebornELOThresholdTranches) {
    const startIndex = stagesSpent
    const stagesInThisTranche = Math.min(tranch.stages, stage - stagesSpent)
    const endIndex = stagesSpent + stagesInThisTranche - 1
    const productionThisTranche = geometricSeries(startIndex, endIndex, perStageMult) * tranch.perStage
    production += productionThisTranche
    stagesSpent += stagesInThisTranche
    if (stagesSpent >= stage) {
      production += leftover * perStageMult ** stage
      break
    }
  }

  return production
}

// ─── Leaderboard + daily-quark math ───────────────────────────────────────

/**
 * Per-rank multipliers applied to the top-N daily/all-time reborn-ELO
 * leaderboard entries. Lifted from
 *   packages/web_ui/src/Features/Ants/AntSacrifice/Rewards/ELO/RebornELO/
 *     QuarkCorner/lib/leaderboard-update.ts (LEADERBOARD_WEIGHTS).
 */
export const LEADERBOARD_WEIGHTS: readonly number[] = [1, 0.8, 0.6, 0.4, 0.2]

/** Weighted-sum of a leaderboard array, floor'd to an integer. Walks at
 * most `min(leaderboard.length, LEADERBOARD_WEIGHTS.length)` entries. */
export function calculateLeaderboardValue (
  leaderboard: ReadonlyArray<{ elo: number; sacrificeId: number }>
): number {
  let total = 0
  const n = Math.min(leaderboard.length, LEADERBOARD_WEIGHTS.length)
  for (let i = 0; i < n; i++) {
    total += leaderboard[i].elo * LEADERBOARD_WEIGHTS[i]
  }
  return Math.floor(total)
}

/**
 * Lifetime-ELO quark multiplier — sigmoid-ish ramp `2 − 0.8^(stages/100)`,
 * asymptoting at 2× as the player accumulates reborn stages.
 *
 * Caller passes the result of calculateLeaderboardValue on the all-time
 * leaderboard (player.ants.highestRebornELOEver in legacy).
 */
export function quarksFromELOMult (lifetimeLeaderboardELO: number): number {
  const numStages = calculateRebornELOThresholds(lifetimeLeaderboardELO)
  return 2 - Math.pow(0.8, numStages / 100)
}

export interface BaseQuarksFromRebornELOStagesResult {
  /** Sum of per-stage quark rewards across tranches. */
  baseQuarks: number
  /** Per-stage quark multiplier (capped at 1000 stages contributing). */
  stageMult: number
}

/**
 * Per-tranche base-quark loop from availableQuarksFromELO. Walks each
 * tranche, accumulating `min(tranche.stages, remainingStages) × quarkPerStage`
 * until stages are exhausted. Separately computes the stage multiplier
 * as `quarkMultiplierPerRebornELOThreshold ^ min(numStages, 1000)`.
 *
 * Callers (web_ui's availableQuarksFromELO) combine `baseQuarks` with
 * `player.worlds.applyBonus()` and `stageMult` × additional ant-quark
 * multipliers.
 */
export function baseQuarksFromRebornELOStages (
  numStages: number
): BaseQuarksFromRebornELOStagesResult {
  let baseQuarks = 0
  let remaining = numStages
  for (const tranch of rebornELOThresholdTranches) {
    const stagesInThisTranche = Math.min(tranch.stages, remaining)
    baseQuarks += stagesInThisTranche * tranch.quarkPerStage
    remaining -= stagesInThisTranche
    if (remaining <= 0) {
      break
    }
  }
  const usedNumberStagesForMult = Math.min(numStages, 1000)
  const stageMult = Math.pow(quarkMultiplierPerRebornELOThreshold, usedNumberStagesForMult)
  return { baseQuarks, stageMult }
}

// ─── Ant-related singularity perks ────────────────────────────────────────

/**
 * "Advanced... Cheating Tactics?" perk — additive ELO multiplier from
 * singularity. Staircase: 0.001 base, +0.0009 per tier.
 */
export function singularityELOBonusMult (singCount: number): number {
  for (let i = singularityELOBonusMultLevels.length - 1; i >= 0; i--) {
    if (singCount >= singularityELOBonusMultLevels[i]) {
      return 0.001 + 0.0009 * i
    }
  }
  return 0
}

export interface SingularityPerkELOInput {
  singCount: number
  immortalELO: number
}

/**
 * "Invigorated Spirits!" perk — flat ELO gift scaled by immortal-ELO bands.
 * First 200,000 immortal ELO contributes at the higher per-unit rate; the
 * next 1.8M at the lower rate; anything above 2M is ignored.
 */
export function calculateSingularityPerkELO (input: SingularityPerkELOInput): number {
  for (let i = singularityPerkELOLevels.length - 1; i >= 0; i--) {
    if (input.singCount >= singularityPerkELOLevels[i]) {
      const firstTranchMult = 0.02 + 0.018 * i
      const secondTranchMult = 0.001 + 0.0009 * i
      const immortalELO = input.immortalELO
      return Math.min(200_000, immortalELO) * firstTranchMult
        + Math.max(0, Math.min(1_800_000, immortalELO - 200_000)) * secondTranchMult
    }
  }
  return 0
}
