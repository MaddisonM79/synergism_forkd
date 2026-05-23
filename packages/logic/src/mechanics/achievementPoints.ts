// Achievement points math, lifted from packages/web_ui/src/Achievements.ts.
// The web_ui side keeps the achievements data table (i18n descriptions,
// unlock predicates, group classifications) and the progressiveAchievements
// dispatch map; this module owns the pure formulas that convert per-progressive
// cached values + the aggregated points sum.
//
// Each `<x>Points` function corresponds to one progressive achievement's
// `pointsAwarded` body. Inputs are pre-extracted from player state by the
// caller — antMasteries gets the per-ant highestMastery array, exalts gets
// the rewardAP[] from singularityChallenges, etc.

// ─── Progressive achievement: rune levels ──────────────────────────────────

/**
 * Points from the cumulative rune-level progressive achievement. Three-knee
 * staircase: 1pt per 1000 (cap 200) + 1pt per 2500 (cap 400) + 1pt per 12500
 * (cap 400). Theoretical max: 1000.
 */
export function runeLevelPoints (sumOfRuneLevels: number): number {
  return Math.min(200, Math.floor(sumOfRuneLevels / 1000))
    + Math.min(400, Math.floor(sumOfRuneLevels / 2500))
    + Math.min(400, Math.floor(sumOfRuneLevels / 12500))
}

/**
 * Points from the cumulative free-rune-level progressive achievement.
 * Three-knee staircase: 1pt per 250 (cap 100) + 1pt per 750 (cap 200) +
 * 1pt per 2500 (cap 200). Theoretical max: 500.
 */
export function freeRuneLevelPoints (sumOfFreeRuneLevels: number): number {
  return Math.min(100, Math.floor(sumOfFreeRuneLevels / 250))
    + Math.min(200, Math.floor(sumOfFreeRuneLevels / 750))
    + Math.min(200, Math.floor(sumOfFreeRuneLevels / 2500))
}

// ─── Progressive achievement: ant masteries ────────────────────────────────

/**
 * Points from the ant-masteries progressive achievement. For each ant
 * producer's `highestMastery`, awards `3 * mastery` + an extra +4 when mastery
 * reaches 12. Theoretical max: 360 (12 ants × 30 + 12 × 4 = 360 + 48 = 408,
 * but maxPointValue is set to 360 so the displayed cap doesn't match the
 * formula's actual peak).
 */
export function antMasteryPoints (highestMasteries: readonly number[]): number {
  let pointValue = 0
  for (const mastery of highestMasteries) {
    pointValue += 3 * mastery
    if (mastery >= 12) {
      pointValue += 4
    }
  }
  return pointValue
}

// ─── Progressive achievement: reborn ELO ───────────────────────────────────

/**
 * Points from the reborn-ELO progressive achievement. Five-knee staircase
 * over the leaderboard value: 1pt per 100 (cap 100) + 1pt per 1000 (cap 150)
 * + 1pt per 9000 (cap 150) + 1pt per 75000 (cap 200) + 1pt per 150000 (cap
 * 400). Theoretical max: 1000.
 *
 * The caller resolves `leaderboardELO` via `calculateLeaderboardValue(player.
 * ants.highestRebornELOEver)` since that helper depends on the
 * antSacrifice-domain leaderboard structure.
 */
export function rebornELOPoints (leaderboardELO: number): number {
  return Math.min(100, Math.floor(leaderboardELO / 100))
    + Math.min(150, Math.floor(leaderboardELO / 1000))
    + Math.min(150, Math.floor(leaderboardELO / 9000))
    + Math.min(200, Math.floor(leaderboardELO / 75000))
    + Math.min(400, Math.floor(leaderboardELO / 150000))
}

// ─── Progressive achievement: singularity count ────────────────────────────

/**
 * Points from the singularity-count progressive achievement. Three-knee
 * accumulator: 9 per singularity, +3 per singularity above 100, +3 per
 * singularity above 200. Theoretical max stated as 3600 in the data table.
 */
export function singularityCountPoints (highestSingularityCount: number): number {
  return 9 * highestSingularityCount
    + 3 * Math.max(0, highestSingularityCount - 100)
    + 3 * Math.max(0, highestSingularityCount - 200)
}

// ─── Progressive achievement: ambrosia counts ──────────────────────────────

/**
 * Points from the lifetime-ambrosia progressive achievement. Three-knee
 * staircase: 1pt per 100 (cap 200) + 1pt per 10000 (cap 200) + sqrt-tail
 * `floor(400 * sqrt(cached / 1e8))` (cap 400). Theoretical max: 800.
 */
export function ambrosiaCountPoints (lifetimeAmbrosia: number): number {
  return Math.min(200, Math.floor(lifetimeAmbrosia / 100))
    + Math.min(200, Math.floor(lifetimeAmbrosia / 10000))
    + Math.min(400, Math.floor(400 * Math.sqrt(lifetimeAmbrosia / 1e8)))
}

/**
 * Points from the lifetime-red-ambrosia progressive achievement. Four-knee
 * staircase: 1pt per 25 (cap 200) + 1pt per 2500 (cap 200) + `400 * cached
 * / 5e6` floored (cap 400) + `200 * cached / 1.25e7` floored (cap 200).
 * Theoretical max: 1000.
 */
export function redAmbrosiaCountPoints (lifetimeRedAmbrosia: number): number {
  return Math.min(200, Math.floor(lifetimeRedAmbrosia / 25))
    + Math.min(200, Math.floor(lifetimeRedAmbrosia / 2500))
    + Math.min(400, Math.floor(400 * lifetimeRedAmbrosia / 5e6))
    + Math.min(200, Math.floor(200 * lifetimeRedAmbrosia / 1.25e7))
}

// ─── Progressive achievement: talisman rarities ────────────────────────────

/**
 * Points from the talisman-rarities progressive achievement. Trivial 5×
 * multiplier over the cached sum-of-rarities. Theoretical max: 50 × number
 * of talismans.
 */
export function talismanRarityPoints (sumOfRarities: number): number {
  return 5 * sumOfRarities
}

// ─── Progressive achievement: exalts ───────────────────────────────────────

/**
 * Points from the exalt-achievement progressive entry. Just the sum of
 * `rewardAP` across all singularity challenges — callers extract the values
 * from `player.singularityChallenges[chal].rewardAP`.
 */
export function exaltPoints (rewardAPs: readonly number[]): number {
  let pointValue = 0
  for (const ap of rewardAPs) {
    pointValue += ap
  }
  return pointValue
}

// ─── Progressive achievement: fully-maxed upgrade families ─────────────────

/**
 * Generic "count of maxed upgrades × point multiplier" formula. Used by the
 * three upgrade-family progressives:
 *   - singularityUpgrades: pointsPerMaxed = 5
 *   - octeractUpgrades: pointsPerMaxed = 8
 *   - redAmbrosiaUpgrades: pointsPerMaxed = 10
 *
 * The caller computes `maxedCount` by walking the upgrade table — for GQ /
 * octeract upgrades that means `upgrade.maxLevel !== -1 && upgrade.level >=
 * upgrade.maxLevel`; for red-ambrosia upgrades the `-1` sentinel doesn't
 * exist so the inequality alone suffices.
 */
export function maxedUpgradeFamilyPoints (maxedCount: number, pointsPerMaxed: number): number {
  return maxedCount * pointsPerMaxed
}

// ─── Achievement-completion quark reward ───────────────────────────────────

/**
 * Quarks awarded for completing a brand-new achievement. Starts at `5 *
 * globalQuarkMultiplier`; above a 100× multiplier, applies a softcap of
 * `100^0.6 * mult^0.4` so the bonus stays meaningful at high multipliers
 * without blowing up. Final result is floored.
 *
 * `globalQuarkMultiplier` is `player.worlds.applyBonus(1)` in web_ui — the
 * full multiplicative quark bonus including patreon / shop / etc.
 */
export function getAchievementQuarks (globalQuarkMultiplier: number): number {
  let actualMultiplier = globalQuarkMultiplier
  if (actualMultiplier > 100) {
    actualMultiplier = Math.pow(100, 0.6) * Math.pow(actualMultiplier, 0.4)
  }
  return Math.floor(5 * actualMultiplier)
}

// ─── Total achievement points ──────────────────────────────────────────────

export interface ComputeAchievementPointsInput {
  /** Per-achievement `pointValue` from the achievements data table. */
  pointValues: readonly number[]
  /**
   * Per-achievement unlocked flag. Truthy (typically 1) when awarded.
   * Indices align with `pointValues`. The reduce treats any truthy value as
   * unlocked to match the legacy `savedAchievements[index] ? ach.pointValue
   * : 0` shape.
   */
  savedAchievements: readonly number[]
  /**
   * Per-progressive-achievement awarded points. Caller assembles this by
   * calling each progressive's `pointsAwarded` (the per-progressive formulas
   * above) and dropping the result into the array in any order.
   */
  progressivePointsAwarded: readonly number[]
}

/**
 * Total achievement points: sum of per-achievement point values for unlocked
 * achievements, plus the sum of per-progressive awarded points.
 */
export function computeAchievementPoints (input: ComputeAchievementPointsInput): number {
  let points = 0
  for (let i = 0; i < input.pointValues.length; i++) {
    if (input.savedAchievements[i]) {
      points += input.pointValues[i]
    }
  }
  for (const awarded of input.progressivePointsAwarded) {
    points += awarded
  }
  return points
}
