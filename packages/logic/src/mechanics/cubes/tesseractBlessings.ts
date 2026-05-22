import { Decimal } from '../../math/bignum'
import type { TesseractBlessings } from '../../state/schema'

// Tesseract blessings: 10 pure multiplier-yield functions. Most follow the
// soft-cap+DR shape (with the per-blessing-tier hypercube result playing the
// `effectPerBlessing` numerator's role). Three outliers — Salvage, AntSpeed,
// AntELO — diverge from the shape (log curve with hypercube cap; linear
// growth × hypercube; log × hypercube). Each takes the precomputed
// hypercube-blessing value of the *same name* as a second parameter; callers
// thread that through (the shim does the composition).

// Shared soft-cap+DR body used by 7 of the 10 functions. Limit fixed at 1000;
// only DR varies. hypercubeBlessing is divided by 1000 to form
// effectPerBlessing — matches the original `calculateXHypercubeBlessing()/1000`
// expression in each web_ui callee.
function softCapDR(count: number, DR: number, hypercubeBlessing: number): number {
  const effectPerBlessing = hypercubeBlessing / 1000
  const limit = 1000
  if (count < limit) return 1 + effectPerBlessing * count
  const limitMult = Math.pow(limit, 1 - DR)
  return 1 + effectPerBlessing * limitMult * Math.pow(count, DR)
}

export function calculateAcceleratorTesseractBlessing(state: TesseractBlessings, hypercubeBlessing: number): number {
  return softCapDR(state.accelerator, 1 / 6, hypercubeBlessing)
}

export function calculateMultiplierTesseractBlessing(state: TesseractBlessings, hypercubeBlessing: number): number {
  return softCapDR(state.multiplier, 1 / 6, hypercubeBlessing)
}

export function calculateOfferingTesseractBlessing(state: TesseractBlessings, hypercubeBlessing: number): number {
  return softCapDR(state.offering, 1 / 3, hypercubeBlessing)
}

export function calculateObtainiumTesseractBlessing(state: TesseractBlessings, hypercubeBlessing: number): number {
  return softCapDR(state.obtainium, 1 / 3, hypercubeBlessing)
}

export function calculateAntSacrificeTesseractBlessing(state: TesseractBlessings, hypercubeBlessing: number): number {
  return softCapDR(state.antSacrifice, 1 / 6, hypercubeBlessing)
}

export function calculateRuneEffectivenessTesseractBlessing(
  state: TesseractBlessings,
  hypercubeBlessing: number
): number {
  return softCapDR(state.talismanBonus, 1 / 32, hypercubeBlessing)
}

export function calculateGlobalSpeedTesseractBlessing(state: TesseractBlessings, hypercubeBlessing: number): number {
  return softCapDR(state.globalSpeed, 1 / 32, hypercubeBlessing)
}

// Outlier 1: log-based factor with hypercube as cap (no soft-cap branch).
export function calculateSalvageTesseractBlessing(
  state: TesseractBlessings,
  hypercubeSalvageBlessing: number
): number {
  const factor = Math.pow(Math.log10(state.runeExp + 1), 1.25)
  const cap = 1 / 2 * hypercubeSalvageBlessing
  return 1 + cap * factor / (20 + factor)
}

// Outlier 2: linear growth multiplied by the hypercube blessing. Returns
// Decimal (not number) because the result feeds into Decimal arithmetic
// downstream — matches the OLD signature.
export function calculateAntSpeedTesseractBlessing(
  state: TesseractBlessings,
  hypercubeAntSpeedBlessing: number
): Decimal {
  const effectPerBlessing = 1 / 1000
  return new Decimal(1 + effectPerBlessing * state.antSpeed).times(hypercubeAntSpeedBlessing)
}

// Outlier 3: log curve scaled by hypercube / 100.
export function calculateAntELOTesseractBlessing(
  state: TesseractBlessings,
  hypercubeAntELOBlessing: number
): number {
  return 1 + Math.log10(state.antELO + 1) * hypercubeAntELOBlessing / 100
}
