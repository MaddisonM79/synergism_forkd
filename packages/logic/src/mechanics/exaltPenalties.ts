// Singularity-challenge penalty math for Exalt 3 (limitedAscensions),
// Exalt 4 (noOcteracts), and Exalt 6 (limitedTime). Lifted verbatim from
// packages/web_ui/src/Calculate.ts. Most of these were already parameterized
// by `comps` or `(comps, time)` and read no player state — the lifts are
// near-identity. The two that read player state (Exalt 3 penalty, Exalt 4
// multiplier) take their state as an input object.

// ─── Exalt 3 (limitedAscensions) ───────────────────────────────────────────

/**
 * Max ascensions allowed in Exalt 3 before the doubling penalty kicks in.
 * Drops by 2 per challenge completion, floored at 0.
 */
export function calculateExalt3AscensionLimit(comps: number): number {
  return Math.max(15 - comps * 2, 0)
}

export interface CalculateExalt3PenaltyInput {
  /** player.singularityChallenges.limitedAscensions.enabled — gates the penalty. */
  limitedAscensionsEnabled: boolean
  /** player.singularityChallenges.limitedAscensions.completions — feeds the ascension limit. */
  limitedAscensionsCompletions: number
  /** player.ascensionCount — the current run's ascension count. */
  ascensionCount: number
}

/**
 * Returns `2 ^ (ascensions - limit)` once the player crosses the
 * limit, or 1 otherwise. Outside Exalt 3 the penalty is always 1.
 */
export function calculateExalt3Penalty(input: CalculateExalt3PenaltyInput): number {
  if (!input.limitedAscensionsEnabled) {
    return 1
  }
  const ascensionLimit = calculateExalt3AscensionLimit(input.limitedAscensionsCompletions)
  return Math.pow(2, Math.max(input.ascensionCount - ascensionLimit, 0))
}

// ─── Exalt 4 (noOcteracts) ─────────────────────────────────────────────────

export interface CalculateExalt4EffectiveSingularityMultiplierInput {
  /** noOcteracts challenge completion count being evaluated. */
  comps: number
  /**
   * Force the bonus on even outside the challenge — used by previewers /
   * Statistics displays that show what the bonus *would* be.
   */
  force: boolean
  /** player.singularityChallenges.noOcteracts.enabled — gates the bonus. */
  inExalt4: boolean
}

/**
 * `(comps + 1)^3` if the player is currently in Exalt 4 OR `force` is true,
 * else 1. Used as a singularity-count multiplier when computing rewards under
 * the no-octeracts challenge.
 */
export function calculateExalt4EffectiveSingularityMultiplier(
  input: CalculateExalt4EffectiveSingularityMultiplierInput
): number {
  return input.inExalt4 || input.force ? Math.pow(input.comps + 1, 3) : 1
}

// ─── Exalt 6 (limitedTime) ─────────────────────────────────────────────────

/**
 * Soft time-cap (in seconds) for an Exalt 6 attempt. Goes from 600s at 0
 * comps, drops by 60s/comp until 10 comps (at 60s base, but the formula
 * switches), then 115s minus 5s/comp beyond 10.
 */
export function calculateExalt6TimeLimit(comps: number): number {
  if (comps >= 10) {
    return 115 - 5 * (comps - 10)
  }
  return 600 - 60 * comps
}

/**
 * Per-minute penalty rate scaling with comp count. Switches to a faster
 * scaling at 10+ comps. Internal — only consumed by
 * `calculateExalt6PenaltyPerSecond`.
 */
function calculateExalt6PenaltyPerMinute(comps: number): number {
  if (comps >= 10) {
    return 60 + 10 * (comps - 10)
  }
  return 10 + 3 * comps
}

/**
 * 60-th root of the per-minute penalty rate. Compounding base for the final
 * `^(-displacedTime)` Exalt 6 penalty.
 */
export function calculateExalt6PenaltyPerSecond(comps: number): number {
  return Math.pow(calculateExalt6PenaltyPerMinute(comps), 1 / 60)
}

/**
 * Final Exalt 6 penalty multiplier: 1 if the player finishes within the
 * `calculateExalt6TimeLimit`, otherwise `penaltyPerSecond ^ -displacedTime`
 * where `displacedTime = time - timeLimit`.
 */
export function calculateExalt6Penalty(comps: number, time: number): number {
  const timeLimit = calculateExalt6TimeLimit(comps)
  const displacedTime = Math.max(0, time - timeLimit)
  if (displacedTime === 0) {
    return 1
  }
  const penaltyPerSecond = calculateExalt6PenaltyPerSecond(comps)
  return Math.pow(penaltyPerSecond, -displacedTime)
}
