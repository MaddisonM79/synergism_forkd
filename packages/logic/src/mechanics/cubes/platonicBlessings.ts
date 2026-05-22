import type { PlatonicBlessings } from '../../state/schema'

// Platonic-cube blessings: 8 pure multiplier-yield functions, one per
// `platonicBlessings.*` field. Each follows the same shape — a soft cap
// (`limit`) above which the field's effect transitions to a power-law with
// diminishing-returns exponent `DR`. The constants vary per function and are
// preserved verbatim from packages/web_ui/src/PlatonicCubes.ts.

export function calculateCubeMultiplierPlatonicBlessing(state: PlatonicBlessings): number {
  const DR = 1 / 5
  const effectPerBlessing = 2 / 4e6
  const limit = 4e6
  if (state.cubes < limit) return 1 + effectPerBlessing * state.cubes
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(state.cubes, DR)
}

export function calculateTesseractMultiplierPlatonicBlessing(state: PlatonicBlessings): number {
  const DR = 1 / 5
  const effectPerBlessing = 1.5 / 4e6
  const limit = 4e6
  if (state.tesseracts < limit) return 1 + effectPerBlessing * state.tesseracts
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(state.tesseracts, DR)
}

export function calculateHypercubeMultiplierPlatonicBlessing(state: PlatonicBlessings): number {
  const DR = 1 / 5
  const effectPerBlessing = 1 / 4e6
  const limit = 4e6
  if (state.hypercubes < limit) return 1 + effectPerBlessing * state.hypercubes
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(state.hypercubes, DR)
}

export function calculatePlatonicMultiplierPlatonicBlessing(state: PlatonicBlessings): number {
  const DR = 1 / 5
  const effectPerBlessing = 1 / 8e4
  const limit = 8e4
  if (state.platonics < limit) return 1 + effectPerBlessing * state.platonics
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(state.platonics, DR)
}

export function calculateHypercubeBlessingMultiplierPlatonicBlessing(state: PlatonicBlessings): number {
  const DR = 1 / 16
  const effectPerBlessing = 1 / 1e4
  const limit = 1e4
  if (state.hypercubeBonus < limit) return 1 + effectPerBlessing * state.hypercubeBonus
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(state.hypercubeBonus, DR)
}

// Tax effect — bounded between 0 and 1, no hard limit branch.
export function calculateTaxPlatonicBlessing(state: PlatonicBlessings): number {
  const factor = Math.pow(Math.log10(1 + state.taxes), 1.5)
  return factor / (125 + factor)
}

// Two-stage diminishing returns: a tighter DR1 between limit1 and limit2, then
// an even tighter DR2 above limit2. The limitMult products preserve continuity
// at the boundaries.
export function calculateAscensionScorePlatonicBlessing(state: PlatonicBlessings): number {
  const DR1 = 1 / 4
  const DR2 = 1 / 8
  const limit1 = 1e4
  const limit2 = 1e20
  const effectPerBlessing = 1 / 1e4
  if (state.globalSpeed < limit1) {
    return 1 + effectPerBlessing * state.globalSpeed
  } else if (limit1 <= state.globalSpeed && state.globalSpeed < limit2) {
    const limitMult = Math.pow(limit1, 1 - DR1)
    return 1 + effectPerBlessing * limitMult * Math.pow(state.globalSpeed, DR1)
  } else {
    const limitMult1 = Math.pow(limit1, 1 - DR1)
    const limitMult2 = Math.pow(limit2, DR1 - DR2)
    return 1 + effectPerBlessing * limitMult1 * limitMult2 * Math.pow(state.globalSpeed, DR2)
  }
}

export function calculateGlobalSpeedPlatonicBlessing(state: PlatonicBlessings): number {
  const DR = 1 / 8
  const limit = 1e4
  const effectPerBlessing = 1 / 1e4
  if (state.globalSpeed < limit) return 1 + effectPerBlessing * state.globalSpeed
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(state.globalSpeed, DR)
}
