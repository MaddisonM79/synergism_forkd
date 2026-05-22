import type { DecimalSource } from '../../math/bignum'
import { Decimal } from '../../math/bignum'
import type { CubeBlessings } from '../../state/schema'

// Cube (wow-cube) blessings: 10 pure multiplier-yield functions, one per
// CubeBlessings field. Each composes the corresponding tesseract-blessing
// value (as effectPerBlessing's numerator or a direct multiplier) with a
// per-function `cubeUpgrade` level that contributes a DR-increase term.
//
// Compared to the platonic / hypercube / tesseract layers, cube blessings
// have:
//   - A per-function `DRIncrease = cubeUpgrade[N] / K` term that adds to BOTH
//     the limitMult exponent and the count exponent.
//   - Several Decimal returns (AntSpeed, AntSacrifice) where downstream code
//     needs to multiply big numbers without overflowing.
//   - Offering/Obtainium use Decimal arithmetic with .toNumber() at the end
//     because the multiplication can briefly exceed 1e308.

export function calculateAcceleratorCubeBlessing(
  state: CubeBlessings,
  tesseractBlessing: number,
  cubeUpgradeLevel: number
): number {
  const DR = 1 / 3
  const effectPerBlessing = tesseractBlessing / 500
  const limit = 1000
  const DRIncrease = cubeUpgradeLevel / 300
  if (state.accelerator < limit) {
    return Math.pow(effectPerBlessing * state.accelerator, 1 + DRIncrease)
  }
  const limitMult = Math.pow(limit, 1 - DR + DRIncrease)
  return effectPerBlessing * limitMult * Math.pow(state.accelerator, DR + DRIncrease)
}

export function calculateMultiplierCubeBlessing(
  state: CubeBlessings,
  tesseractBlessing: number,
  cubeUpgradeLevel: number
): number {
  const DR = 1 / 3
  const effectPerBlessing = tesseractBlessing / 5000
  const limit = 1000
  const DRIncrease = cubeUpgradeLevel / 300
  if (state.multiplier < limit) {
    return Math.pow(1 + effectPerBlessing * state.multiplier, 1 + DRIncrease)
  }
  const limitMult = Math.pow(limit, 1 - DR + DRIncrease)
  return 1 + effectPerBlessing * limitMult * Math.pow(state.multiplier, DR + DRIncrease)
}

export function calculateOfferingCubeBlessing(
  state: CubeBlessings,
  tesseractBlessing: number,
  cubeUpgradeLevel: number
): number {
  const DR = 2 / 3
  // Decimal arithmetic to avoid intermediate overflow past 1e308 — the
  // final .toNumber() clamps via Decimal.min(1e300, …).
  const effectPerBlessing = new Decimal(tesseractBlessing).div(2000)
  const limit = 1000
  const DRIncrease = cubeUpgradeLevel * 2 / 300
  if (state.offering < limit) {
    return Decimal.min(
      1e300,
      Decimal.pow(effectPerBlessing.times(state.offering).plus(1), 1 + DRIncrease)
    ).toNumber()
  }
  const limitMult = Decimal.pow(limit, 1 - DR + DRIncrease)
  return Decimal.min(
    1e300,
    limitMult.times(effectPerBlessing).times(Math.pow(state.offering, DR + DRIncrease)).plus(1)
  ).toNumber()
}

export function calculateSalvageCubeBlessing(
  state: CubeBlessings,
  tesseractBlessing: number,
  cubeUpgradeLevel: number
): number {
  const limit = 1000
  const effectMultiplier = (1 + cubeUpgradeLevel / 100) * tesseractBlessing
  if (state.runeExp < limit) {
    return effectMultiplier * (state.runeExp * 10 / limit)
  }
  const limitBonus = 10
  return effectMultiplier * (limitBonus + 10 * Math.log10(state.runeExp / limit))
}

export function calculateObtainiumCubeBlessing(
  state: CubeBlessings,
  tesseractBlessing: number,
  cubeUpgradeLevel: number
): number {
  const DR = 2 / 3
  const effectPerBlessing = new Decimal(tesseractBlessing).div(2000)
  const limit = 1000
  const DRIncrease = cubeUpgradeLevel * 2 / 300
  if (state.obtainium < limit) {
    return Decimal.min(
      1e300,
      Decimal.pow(effectPerBlessing.times(state.obtainium).plus(1), 1 + DRIncrease)
    ).toNumber()
  }
  const limitMult = Decimal.pow(limit, 1 - DR + DRIncrease)
  return Decimal.min(
    1e300,
    limitMult.times(effectPerBlessing).times(Math.pow(state.obtainium, DR + DRIncrease)).plus(1)
  ).toNumber()
}

// firstBonus = 0.1 when antSpeed >= 1, else 0.1*antSpeed (linear ramp from 0
// to 0.1 across the first blessing). The tesseractBlessing parameter accepts
// a DecimalSource because the upstream calculateAntSpeedTesseractBlessing
// returns a Decimal that can exceed Number precision at late game.
export function calculateAntSpeedCubeBlessing(
  state: CubeBlessings,
  tesseractBlessing: DecimalSource,
  cubeUpgradeLevel: number
): Decimal {
  const effectPerBlessing = 1 / 1000
  const exponentIncrease = cubeUpgradeLevel / 40
  const firstBonus = 0.1 * Math.min(state.antSpeed, 1)
  return Decimal.pow(1 + effectPerBlessing * state.antSpeed + firstBonus, 2 + exponentIncrease)
    .times(tesseractBlessing)
}

export function calculateAntSacrificeCubeBlessing(
  state: CubeBlessings,
  tesseractBlessing: number,
  cubeUpgradeLevel: number
): Decimal {
  const DR = 2 / 3
  const effectPerBlessing = tesseractBlessing / 5000
  const limit = 1000
  const DRIncrease = cubeUpgradeLevel / 50
  if (state.antSacrifice < limit) {
    return Decimal.pow(1 + effectPerBlessing * state.antSacrifice, 1 + DRIncrease)
  }
  const limitMult = Math.pow(limit, 1 - DR + DRIncrease)
  return Decimal.pow(state.antSacrifice, DR + DRIncrease).times(effectPerBlessing).times(limitMult).add(1)
}

export function calculateAntELOCubeBlessing(
  state: CubeBlessings,
  tesseractBlessing: number,
  cubeUpgradeLevel: number
): number {
  const effectExponent = 1 + cubeUpgradeLevel / 100
  return Math.pow(1 + 0.1 * Math.log10(1 + state.antELO) * tesseractBlessing, effectExponent)
}

export function calculateRuneEffectivenessCubeBlessing(
  state: CubeBlessings,
  tesseractBlessing: number,
  cubeUpgradeLevel: number
): number {
  const DR = 1 / 16
  const effectPerBlessing = tesseractBlessing / 10000
  const limit = 1000
  const DRIncrease = cubeUpgradeLevel / 1600
  if (state.talismanBonus < limit) {
    return Math.pow(1 + effectPerBlessing * state.talismanBonus, 1 + DRIncrease)
  }
  const limitMult = Math.pow(limit, 1 - DR + DRIncrease)
  return Math.min(
    1e300,
    1 + limitMult * effectPerBlessing * Math.pow(state.talismanBonus, DR + DRIncrease)
  )
}

export function calculateGlobalSpeedCubeBlessing(
  state: CubeBlessings,
  tesseractBlessing: number,
  cubeUpgradeLevel: number
): number {
  const DR = 1 / 16
  const effectPerBlessing = tesseractBlessing / 1000
  const limit = 1000
  const DRIncrease = cubeUpgradeLevel / 1600
  if (state.globalSpeed < limit) {
    return Math.pow(1 + effectPerBlessing * state.globalSpeed, 1 + DRIncrease)
  }
  const limitMult = Math.pow(limit, 1 - DR + DRIncrease)
  return Math.min(
    1e300,
    1 + limitMult * effectPerBlessing * Math.pow(state.globalSpeed, DR + DRIncrease)
  )
}
