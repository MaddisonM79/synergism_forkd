import type { HypercubeBlessings } from '../../state/schema'

// Hypercube blessings: 10 pure multiplier-yield functions. Eight of them
// follow the soft-cap + DR shape and additionally scale `effectPerBlessing`
// by an amplifier sourced from the platonic-blessings layer
// (`calculateHypercubeBlessingMultiplierPlatonicBlessing` in
// mechanics/cubes/platonicBlessings.ts). Callers precompute that amplifier
// and pass it as the second arg.
//
// The two outliers — Salvage and AntELO — are amplifier-free logarithms.

// Shared soft-cap+DR body used by 8 of the 10 functions. limit is fixed at
// 1000 across all of them; only the DR varies. Inlining this slightly
// reduces duplication while keeping each public function readable.
function softCapDR(count: number, DR: number, platonicAmplifier: number): number {
  const effectPerBlessing = platonicAmplifier / 1000
  const limit = 1000
  if (count < limit) return 1 + effectPerBlessing * count
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(count, DR)
}

export function calculateAcceleratorHypercubeBlessing(state: HypercubeBlessings, platonicAmplifier: number): number {
  return softCapDR(state.accelerator, 1 / 12, platonicAmplifier)
}

export function calculateMultiplierHypercubeBlessing(state: HypercubeBlessings, platonicAmplifier: number): number {
  return softCapDR(state.multiplier, 1 / 12, platonicAmplifier)
}

export function calculateOfferingHypercubeBlessing(state: HypercubeBlessings, platonicAmplifier: number): number {
  return softCapDR(state.offering, 1 / 6, platonicAmplifier)
}

export function calculateObtainiumHypercubeBlessing(state: HypercubeBlessings, platonicAmplifier: number): number {
  return softCapDR(state.obtainium, 1 / 6, platonicAmplifier)
}

export function calculateAntSpeedHypercubeBlessing(state: HypercubeBlessings, platonicAmplifier: number): number {
  return softCapDR(state.antSpeed, 1 / 2, platonicAmplifier)
}

export function calculateAntSacrificeHypercubeBlessing(state: HypercubeBlessings, platonicAmplifier: number): number {
  return softCapDR(state.antSacrifice, 1 / 12, platonicAmplifier)
}

export function calculateRuneEffectivenessHypercubeBlessing(
  state: HypercubeBlessings,
  platonicAmplifier: number
): number {
  return softCapDR(state.talismanBonus, 1 / 64, platonicAmplifier)
}

export function calculateGlobalSpeedHypercubeBlessing(state: HypercubeBlessings, platonicAmplifier: number): number {
  return softCapDR(state.globalSpeed, 1 / 64, platonicAmplifier)
}

// Salvage and AntELO don't take the platonic amplifier — they're amplifier-free
// log-scale curves.
export function calculateSalvageHypercubeBlessing(state: HypercubeBlessings): number {
  const factor = Math.pow(Math.log10(state.runeExp + 1), 1.25)
  const cap = 3 / 2
  return 1 + cap * factor / (40 + factor)
}

export function calculateAntELOHypercubeBlessing(state: HypercubeBlessings): number {
  return 1 + Math.log10(state.antELO + 1) / 25
}
