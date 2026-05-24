// Per-spirit effect formulas. Pure 1-parameter functions extracted from the
// `runeSpirits.<key>.effects` fields in packages/web_ui/src/RuneSpirits.ts.
// The surrounding plumbing — EXP/level state, spirit-power multiplier,
// buySpiritLevels loop, UI strings — stays in web_ui.
//
// All five spirits (speed, duplication, prism, thrift, superiorIntellect)
// take a single `level` argument and return an object with one effect field.
// Four of them share the same `1 + level / 1e9` shape. Prism is the odd one
// out — `level / 1e9` with no `1 +` prefix, because crystalCaps is an
// additive cap bonus rather than a multiplier.

export interface SpeedRuneSpiritEffects {
  globalSpeed: number
}
export function speedRuneSpiritEffects (level: number): SpeedRuneSpiritEffects {
  return { globalSpeed: 1 + level / 1_000_000_000 }
}

export interface DuplicationRuneSpiritEffects {
  wowCubes: number
}
export function duplicationRuneSpiritEffects (level: number): DuplicationRuneSpiritEffects {
  return { wowCubes: 1 + level / 1_000_000_000 }
}

export interface PrismRuneSpiritEffects {
  crystalCaps: number
}
export function prismRuneSpiritEffects (level: number): PrismRuneSpiritEffects {
  // No `1 +` prefix: crystalCaps is an additive bonus, not a multiplier.
  return { crystalCaps: level / 1_000_000_000 }
}

export interface ThriftRuneSpiritEffects {
  offerings: number
}
export function thriftRuneSpiritEffects (level: number): ThriftRuneSpiritEffects {
  return { offerings: 1 + level / 1_000_000_000 }
}

export interface SuperiorIntellectRuneSpiritEffects {
  obtainium: number
}
export function superiorIntellectRuneSpiritEffects (level: number): SuperiorIntellectRuneSpiritEffects {
  return { obtainium: 1 + level / 1_000_000_000 }
}
