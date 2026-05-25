// Per-milestone level scaling formulas, lifted from
// packages/web_ui/src/Levels.ts (the `synergismLevelMilestones.<key>.effect`
// fields). Almost every milestone is either a pure `(level: number) => number`
// or a constant `() => 1` unlock flag — those go in the `levelMilestones`
// table, and `getLevelMilestone(key, level)` returns the effect or the
// default depending on the level gate.
//
// The one exception is `salvageChallengeBuff`, whose value depends on which
// challenge the player is in. It lives as a standalone
// `salvageChallengeBuffEffect(input)` here; web_ui sources the challenge
// state and calls it directly.

export type LevelMilestoneKey =
  | 'offeringTimerScaling'
  | 'autoPrestige'
  | 'speedRune'
  | 'duplicationRune'
  | 'prismRune'
  | 'thriftRune'
  | 'SIRune'
  | 'tier1CrystalAutobuy'
  | 'tier2CrystalAutobuy'
  | 'tier3CrystalAutobuy'
  | 'tier4CrystalAutobuy'
  | 'tier5CrystalAutobuy'
  | 'achievementTalismanUnlock'
  | 'runeAutobuyImprover'
  | 'achievementTalismanEnhancement'
  | 'antSpeed2Autobuyer'
  | 'wowCubesAutobuyer'
  | 'ascensionScoreAutobuyer'
  | 'mortuus2Autobuyer'

export interface LevelMilestoneData {
  /** Pure effect formula. Only called by `getLevelMilestone` once level ≥ levelReq. */
  effect: (level: number) => number
  /** Level at which the milestone unlocks. Below this, callers get `defaultValue`. */
  levelReq: number
  /** Returned by `getLevelMilestone` when level < levelReq. */
  defaultValue: number
}

// ─── Per-milestone effect formulas ─────────────────────────────────────────

/** Constant 1 — pure flag, the milestone exists only to unlock UI behavior. */
const unlockFlagEffect = () => 1

// The five "rune scaling" milestones share the shape `coeff * (level - threshold)`
// where (coeff, threshold) is fixed per rune. Pulled into a small helper so the
// per-rune exports don't repeat the formula five times.
function runeScalingEffect (level: number, coeff: number, threshold: number): number {
  return coeff * (level - threshold)
}

export function speedRuneMilestoneEffect (level: number): number {
  return runeScalingEffect(level, 0.5, 19)
}
export function duplicationRuneMilestoneEffect (level: number): number {
  return runeScalingEffect(level, 0.4, 39)
}
export function prismRuneMilestoneEffect (level: number): number {
  return runeScalingEffect(level, 0.3, 59)
}
export function thriftRuneMilestoneEffect (level: number): number {
  return runeScalingEffect(level, 0.2, 79)
}
export function siRuneMilestoneEffect (level: number): number {
  return runeScalingEffect(level, 0.1, 99)
}

/**
 * Rune autobuyer interval improver — `1.1 + 0.01 * (level - 130)`. Returns 1
 * below level 130 via `getLevelMilestone`'s default-value gate.
 */
export function runeAutobuyImproverEffect (level: number): number {
  return 1.1 + 0.01 * (level - 130)
}

/** Achievement Talisman Enhancement just passes the level through. */
export function achievementTalismanEnhancementEffect (level: number): number {
  return level
}

// ─── salvageChallengeBuff (impure — reads challenge state) ─────────────────

export interface SalvageChallengeBuffInput {
  /**
   * True when ANY of player.currentChallenge.{transcension,reincarnation,
   * ascension} is non-zero. Doubles the base buff.
   */
  inAnyChallenge: boolean
  /** True when player.currentChallenge.ascension === 15. Doubles again on top. */
  inAscension15: boolean
  /** player.insideSingularityChallenge. Triples on top of any other multipliers. */
  insideSingularityChallenge: boolean
}

/**
 * Salvage buff inside challenges. Base 25; ×2 inside any normal challenge,
 * ×2 again inside C15 specifically (so ×4 total there), ×3 again inside any
 * singularity challenge (cumulative with the prior multipliers).
 */
export function salvageChallengeBuffEffect (input: SalvageChallengeBuffInput): number {
  let baseVal = 25
  if (input.inAnyChallenge) {
    baseVal *= 2
  }
  if (input.inAscension15) {
    baseVal *= 2
  }
  if (input.insideSingularityChallenge) {
    baseVal *= 3
  }
  return baseVal
}

// ─── Data table (pure milestones only) ─────────────────────────────────────

export const levelMilestones: Record<LevelMilestoneKey, LevelMilestoneData> = {
  offeringTimerScaling: { effect: unlockFlagEffect, levelReq: 5, defaultValue: 0 },
  autoPrestige: { effect: unlockFlagEffect, levelReq: 7, defaultValue: 0 },
  speedRune: { effect: speedRuneMilestoneEffect, levelReq: 20, defaultValue: 0 },
  duplicationRune: { effect: duplicationRuneMilestoneEffect, levelReq: 40, defaultValue: 0 },
  prismRune: { effect: prismRuneMilestoneEffect, levelReq: 60, defaultValue: 0 },
  thriftRune: { effect: thriftRuneMilestoneEffect, levelReq: 80, defaultValue: 0 },
  SIRune: { effect: siRuneMilestoneEffect, levelReq: 100, defaultValue: 0 },
  tier1CrystalAutobuy: { effect: unlockFlagEffect, levelReq: 6, defaultValue: 0 },
  tier2CrystalAutobuy: { effect: unlockFlagEffect, levelReq: 9, defaultValue: 0 },
  tier3CrystalAutobuy: { effect: unlockFlagEffect, levelReq: 12, defaultValue: 0 },
  tier4CrystalAutobuy: { effect: unlockFlagEffect, levelReq: 15, defaultValue: 0 },
  tier5CrystalAutobuy: { effect: unlockFlagEffect, levelReq: 20, defaultValue: 0 },
  achievementTalismanUnlock: { effect: unlockFlagEffect, levelReq: 100, defaultValue: 0 },
  runeAutobuyImprover: { effect: runeAutobuyImproverEffect, levelReq: 130, defaultValue: 1 },
  achievementTalismanEnhancement: {
    effect: achievementTalismanEnhancementEffect,
    levelReq: 160,
    defaultValue: 0
  },
  antSpeed2Autobuyer: { effect: unlockFlagEffect, levelReq: 65, defaultValue: 0 },
  wowCubesAutobuyer: { effect: unlockFlagEffect, levelReq: 80, defaultValue: 0 },
  ascensionScoreAutobuyer: { effect: unlockFlagEffect, levelReq: 80, defaultValue: 0 },
  mortuus2Autobuyer: { effect: unlockFlagEffect, levelReq: 225, defaultValue: 0 }
}

/**
 * Returns the active milestone value for a given achievement level. Below the
 * milestone's `levelReq`, returns the `defaultValue`; otherwise invokes the
 * milestone's `effect`. Does NOT cover `salvageChallengeBuff` — that one
 * needs challenge state and is exposed separately as
 * `salvageChallengeBuffEffect`.
 */
export function getLevelMilestone (milestone: LevelMilestoneKey, level: number): number {
  const data = levelMilestones[milestone]
  if (level >= data.levelReq) {
    return data.effect(level)
  }
  return data.defaultValue
}
