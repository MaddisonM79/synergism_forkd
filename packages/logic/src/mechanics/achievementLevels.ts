// Achievement-level math, lifted from packages/web_ui/src/Achievements.ts.
// The level-from-points and exp-to-next-level formulas are pure functions of
// the achievement points total. Both share a 2500-point regime switch — below
// that, levels are 50 points apart; above, 100 points apart.

const REGIME_SWITCH_POINTS = 2500
const REGIME_SWITCH_LEVEL = 50
const LOW_REGIME_POINTS_PER_LEVEL = 50
const HIGH_REGIME_POINTS_PER_LEVEL = 100

/**
 * Achievement level for a given points total. Below 2500 points the level
 * advances every 50 points; above 2500 it advances every 100 points (with
 * level 50 reached at exactly 2500). Uses `Math.floor` so partial progress
 * doesn't count.
 */
export function achievementLevelFromPoints (points: number): number {
  if (points < REGIME_SWITCH_POINTS) {
    return Math.floor(points / LOW_REGIME_POINTS_PER_LEVEL)
  }
  return REGIME_SWITCH_LEVEL + Math.floor((points - REGIME_SWITCH_POINTS) / HIGH_REGIME_POINTS_PER_LEVEL)
}

/**
 * Points remaining until the next achievement level. Uses the same 2500-point
 * regime switch: 50 - (points % 50) below, 100 - (points % 100) above. Note
 * that the value is the *gap* to the next level, so the caller always sees a
 * positive number — at the exact threshold, returns the full level cost
 * (50 below 2500, 100 above).
 */
export function toNextAchievementLevelEXP (points: number): number {
  if (points < REGIME_SWITCH_POINTS) {
    return LOW_REGIME_POINTS_PER_LEVEL - (points % LOW_REGIME_POINTS_PER_LEVEL)
  }
  return HIGH_REGIME_POINTS_PER_LEVEL - (points % HIGH_REGIME_POINTS_PER_LEVEL)
}
