// Hepteract EFFECTS — the pure formulas that convert a hepteract balance
// (`hept`) into a multiplier or stat. There are 8 hepteract types; 7 of them
// produce their effect directly from `hept` alone. The 8th (`quark`) raises
// its base to an exponent that combines a fixed DR with a `DR_INCREASE` term
// that sums contributions from several external effect sources (singularity
// upgrades, octeract upgrades, shop upgrades). The shim recomputes that
// exponent on each call and passes it in.
//
// The rest of Hepteracts.ts — BAL state, crafting/expanding, descriptions,
// per-hepteract UNLOCKED checks — stays in web_ui per the Phase 2 plan.

export interface ChronosHepteractEffects {
  ascensionSpeed: number
}

export function chronosHepteractEffects(hept: number): ChronosHepteractEffects {
  return { ascensionSpeed: 1 + 6 * hept / 10000 }
}

export interface HyperrealismHepteractEffects {
  hypercubeMultiplier: number
}

export function hyperrealismHepteractEffects(hept: number): HyperrealismHepteractEffects {
  return { hypercubeMultiplier: 1 + 6 * hept / 10000 }
}

export interface QuarkHepteractEffects {
  quarkMultiplier: number
}

/**
 * Quark hepteract is the only one whose exponent isn't a constant — it sums a
 * fixed DR with contributions from singularity / octeract / shop upgrades.
 * Callers precompute that sum and pass it in via `drExponent` (= DR + DR_INCREASE()).
 */
export function quarkHepteractEffects(hept: number, drExponent: number): QuarkHepteractEffects {
  return { quarkMultiplier: Math.pow(1 + 0.2 * Math.log2(1 + hept / 500), drExponent) }
}

export interface ChallengeHepteractEffects {
  c15ScoreMultiplier: number
}

export function challengeHepteractEffects(hept: number): ChallengeHepteractEffects {
  return { c15ScoreMultiplier: 1 + 5 * hept / 10000 }
}

export interface AbyssHepteractEffects {
  salvage: number
}

export function abyssHepteractEffects(hept: number): AbyssHepteractEffects {
  // The Math.max(1, hept*2) guard avoids log2(0) producing -Infinity at hept=0.
  return { salvage: 0.1 * Math.floor(10 * Math.log2(Math.max(1, hept * 2))) }
}

export interface AcceleratorHepteractEffects {
  accelerators: number
  acceleratorMultiplier: number
}

export function acceleratorHepteractEffects(hept: number): AcceleratorHepteractEffects {
  return {
    accelerators: 2000 * hept,
    acceleratorMultiplier: 1 + 3 * hept / 10000
  }
}

export interface AcceleratorBoostHepteractEffects {
  acceleratorBoostMultiplier: number
}

export function acceleratorBoostHepteractEffects(hept: number): AcceleratorBoostHepteractEffects {
  return { acceleratorBoostMultiplier: 1 + hept / 1000 }
}

export interface MultiplierHepteractEffects {
  multiplier: number
  multiplierMultiplier: number
}

export function multiplierHepteractEffects(hept: number): MultiplierHepteractEffects {
  return {
    multiplier: 1000 * hept,
    multiplierMultiplier: 1 + 3 * hept / 10000
  }
}
