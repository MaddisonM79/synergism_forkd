// Per-blessing effect formulas. Pure 1-parameter functions extracted from
// the `runeBlessings.<key>.effects` fields in
// packages/web_ui/src/RuneBlessings.ts. The surrounding plumbing — EXP/level
// state, blessing-power multiplier, buyBlessingLevels loop, UI strings —
// stays in web_ui.
//
// All five blessings (speed, duplication, prism, thrift, superiorIntellect)
// take a single `level` argument and return an object with one effect field.
// Four of them share the same `1 + level / 1e6` shape; the fifth (SI) uses
// a logarithm.

export interface SpeedRuneBlessingEffects {
  globalSpeed: number
}
export function speedRuneBlessingEffects(level: number): SpeedRuneBlessingEffects {
  return { globalSpeed: 1 + level / 1_000_000 }
}

export interface DuplicationRuneBlessingEffects {
  multiplierBoosts: number
}
export function duplicationRuneBlessingEffects(level: number): DuplicationRuneBlessingEffects {
  return { multiplierBoosts: 1 + level / 1_000_000 }
}

export interface PrismRuneBlessingEffects {
  antSacrificeMult: number
}
export function prismRuneBlessingEffects(level: number): PrismRuneBlessingEffects {
  return { antSacrificeMult: 1 + level / 1_000_000 }
}

export interface ThriftRuneBlessingEffects {
  accelBoostCostDelay: number
}
export function thriftRuneBlessingEffects(level: number): ThriftRuneBlessingEffects {
  return { accelBoostCostDelay: 1 + level / 1_000_000 }
}

export interface SuperiorIntellectRuneBlessingEffects {
  obtToAntExponent: number
}
export function superiorIntellectRuneBlessingEffects(level: number): SuperiorIntellectRuneBlessingEffects {
  return { obtToAntExponent: Math.log(1 + level / 1_000_000) }
}
