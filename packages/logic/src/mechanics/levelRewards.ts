// Per-reward level scaling formulas, lifted from
// packages/web_ui/src/Levels.ts (the `synergismLevelRewards.<key>.effect`
// fields). Each reward is a pure `(level: number) => number` paired with the
// `minLevel` at which it activates and the `defaultValue` returned below
// that. The i18n strings, name colors, and DOM wiring stay in web_ui — this
// module is just the numbers.

export type LevelRewardKey =
  | 'salvage'
  | 'quarks'
  | 'offerings'
  | 'obtainium'
  | 'ants'
  | 'wowCubes'
  | 'wowTesseracts'
  | 'wowHyperCubes'
  | 'wowPlatonicCubes'
  | 'wowHepteractCubes'
  | 'wowOcteracts'
  | 'ambrosiaLuck'
  | 'redAmbrosiaLuck'

export interface LevelRewardData {
  /** Pure effect formula. Only called by `getLevelReward` once level ≥ minLevel. */
  effect: (level: number) => number
  /** Level at which the reward unlocks. Below this, callers get `defaultValue`. */
  minLevel: number
  /** Returned by `getLevelReward` when level < minLevel. */
  defaultValue: number
}

// ─── Per-reward effect formulas ────────────────────────────────────────────

/**
 * Salvage grows by 1/level for the first 100 levels, then 2/level for the
 * next 100, then 3/level for the next 100, etc. Computed as a stepwise loop
 * that subtracts a 100-level block per pass, bumping the per-level rate by 1.
 */
export function salvageEffect (level: number): number {
  let salvage = 0
  let salvagePerLevel = 1
  let remainingLevels = level
  while (remainingLevels >= 100) {
    salvage += salvagePerLevel * 100
    remainingLevels -= 100
    salvagePerLevel += 1
  }
  salvage += salvagePerLevel * remainingLevels
  return salvage
}

/** Quark multiplier — `1.01^floor(level/20)`. Steps up every 20 levels. */
export function quarksEffect (level: number): number {
  return Math.pow(1.01, Math.floor(level / 20))
}

/** Offering multiplier — `1.01^level * 1.02^max(0, level-100)`. */
export function offeringsEffect (level: number): number {
  return Math.pow(1.01, level) * Math.pow(1.02, Math.max(0, level - 100))
}

/** Obtainium multiplier — `1.01^(level-15) * 1.02^max(0, level-100)`. */
export function obtainiumEffect (level: number): number {
  return Math.pow(1.01, level - 15) * Math.pow(1.02, Math.max(0, level - 100))
}

/**
 * Ant ELO bonus. Three-band linear formula:
 *   - Levels 60–130: +25 per level (capped at 71 levels' worth)
 *   - Levels 100–200: +50 per level (capped at 100 levels' worth)
 *   - Levels 200+: +100 per level
 * The web_ui-visible behavior is to add these three contributions together,
 * which gives the documented 25/50/100 step at each band.
 */
export function antsEffect (level: number): number {
  const first100Levels = Math.min(71, level - 59) * 25
  const next100Levels = Math.max(0, Math.min(100, level - 100)) * 50
  const remainingLevels = Math.max(0, level - 200) * 100
  return first100Levels + next100Levels + remainingLevels
}

/**
 * Shape shared by the six wow-cube rewards:
 *   `(1 + (level - linearOffset) / 20) * 1.07^(floor(level/10) - tenthOffset)`
 *
 * Each cube tier has different offsets and minLevel — see the per-cube
 * exports below. Pulled out so the web_ui table doesn't have to repeat the
 * formula six times verbatim.
 */
function wowCubeShapedEffect (level: number, linearOffset: number, tenthOffset: number): number {
  return (1 + (level - linearOffset) / 20) * Math.pow(1.07, Math.floor(level / 10) - tenthOffset)
}

export function wowCubesEffect (level: number): number {
  return wowCubeShapedEffect(level, 60, 6)
}
export function wowTesseractsEffect (level: number): number {
  return wowCubeShapedEffect(level, 80, 8)
}
export function wowHyperCubesEffect (level: number): number {
  return wowCubeShapedEffect(level, 100, 10)
}
export function wowPlatonicCubesEffect (level: number): number {
  return wowCubeShapedEffect(level, 120, 12)
}
export function wowHepteractCubesEffect (level: number): number {
  return wowCubeShapedEffect(level, 150, 15)
}

/** Octeract multiplier — uses 1.02 base (not 1.07) and a per-level (not per-tenth) exponent. */
export function wowOcteractsEffect (level: number): number {
  return (1 + (level - 209) / 20) * Math.pow(1.02, level - 209)
}

/** Flat 4 ambrosia luck per level past 229. */
export function ambrosiaLuckEffect (level: number): number {
  return 4 * (level - 229)
}

/** Flat 1 red-ambrosia luck per level past 259. */
export function redAmbrosiaLuckEffect (level: number): number {
  return level - 259
}

// ─── Data table ────────────────────────────────────────────────────────────

export const levelRewards: Record<LevelRewardKey, LevelRewardData> = {
  salvage: { effect: salvageEffect, minLevel: 0, defaultValue: 0 },
  quarks: { effect: quarksEffect, minLevel: 20, defaultValue: 1 },
  offerings: { effect: offeringsEffect, minLevel: 0, defaultValue: 1 },
  obtainium: { effect: obtainiumEffect, minLevel: 15, defaultValue: 1 },
  ants: { effect: antsEffect, minLevel: 60, defaultValue: 1 },
  wowCubes: { effect: wowCubesEffect, minLevel: 70, defaultValue: 1 },
  wowTesseracts: { effect: wowTesseractsEffect, minLevel: 90, defaultValue: 1 },
  wowHyperCubes: { effect: wowHyperCubesEffect, minLevel: 110, defaultValue: 1 },
  wowPlatonicCubes: { effect: wowPlatonicCubesEffect, minLevel: 140, defaultValue: 1 },
  wowHepteractCubes: { effect: wowHepteractCubesEffect, minLevel: 170, defaultValue: 1 },
  wowOcteracts: { effect: wowOcteractsEffect, minLevel: 210, defaultValue: 1 },
  ambrosiaLuck: { effect: ambrosiaLuckEffect, minLevel: 230, defaultValue: 0 },
  redAmbrosiaLuck: { effect: redAmbrosiaLuckEffect, minLevel: 260, defaultValue: 0 }
}

/**
 * Returns the active reward value for a given achievement level. Below the
 * reward's `minLevel`, returns the `defaultValue`; otherwise invokes the
 * reward's `effect`.
 */
export function getLevelReward (reward: LevelRewardKey, level: number): number {
  const data = levelRewards[reward]
  if (level >= data.minLevel) {
    return data.effect(level)
  }
  return data.defaultValue
}
