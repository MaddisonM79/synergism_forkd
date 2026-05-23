// Overflux-derived multipliers lifted from packages/web_ui/src/Calculate.ts.
// Two pure-powder formulas plus the orbs-based cube-to-quark multiplier
// (a 12-term sigmoid stack gated by singularity-count unlocks).

import { calculateSigmoid } from '../math/sigmoid'

// ─── Powder → cube / quark mults ───────────────────────────────────────────

/**
 * Cube multiplier from overflux powder. Linear in `overfluxPowder/10000`
 * below the 10k threshold; switches to a log10² scaling above it.
 */
export function calculateCubeMultFromPowder(overfluxPowder: number): number {
  return overfluxPowder > 10000
    ? 1 + (1 / 16) * Math.pow(Math.log10(overfluxPowder), 2)
    : 1 + (1 / 10000) * overfluxPowder
}

/**
 * Quark multiplier from overflux powder. Same boundary as the cube version,
 * but linear log10 above the threshold.
 */
export function calculateQuarkMultFromPowder(overfluxPowder: number): number {
  return overfluxPowder > 10000
    ? 1 + (1 / 40) * Math.log10(overfluxPowder)
    : 1 + (1 / 100000) * overfluxPowder
}

// ─── Orbs → cube-quark multiplier ──────────────────────────────────────────

export interface CalculateCubeQuarkMultiplierInput {
  /** player.overfluxOrbs — the main scaling input for every sigmoid. */
  overfluxOrbs: number
  /**
   * player.highestSingularityCount — gates the last nine sigmoid contributors
   * at thresholds 1, 2, 5, 10, 15, 20, 25, 30, 35.
   */
  highestSingularityCount: number
  /**
   * getShopUpgradeEffects('cubeToQuarkAll', 'quarkMult') — final outer
   * multiplier.
   */
  cubeToQuarkAllMult: number
  /**
   * player.autoWarpCheck. When true, the result is further multiplied by
   * `1 + dailyPowderResetUses`.
   */
  autoWarpCheck: boolean
  /** player.dailyPowderResetUses. */
  dailyPowderResetUses: number
}

/**
 * Cube → quark multiplier from overflux orbs. Sums 12 sigmoid contributors
 * (with progressively higher constants and divisors), each of the last 9
 * gated by a singularity-count threshold. Subtracts 11 (the all-zero sum
 * baseline is 12 because each sigmoid returns 1 at factor=0), then
 * multiplies by the `cubeToQuarkAll` shop effect and an optional
 * `dailyPowderResetUses` bonus when auto-warp is on.
 */
export function calculateCubeQuarkMultiplier(input: CalculateCubeQuarkMultiplierInput): number {
  const orbs = input.overfluxOrbs
  const high = input.highestSingularityCount

  const sigmoids = calculateSigmoid(2, Math.pow(orbs, 0.5), 40)
    + calculateSigmoid(1.5, Math.pow(orbs, 0.5), 160)
    + calculateSigmoid(1.5, Math.pow(orbs, 0.5), 640)
    + calculateSigmoid(1.15, +(high >= 1) * Math.pow(orbs, 0.45), 2560)
    + calculateSigmoid(1.15, +(high >= 2) * Math.pow(orbs, 0.4), 10000)
    + calculateSigmoid(1.25, +(high >= 5) * Math.pow(orbs, 0.35), 40000)
    + calculateSigmoid(1.25, +(high >= 10) * Math.pow(orbs, 0.32), 160000)
    + calculateSigmoid(1.35, +(high >= 15) * Math.pow(orbs, 0.27), 640000)
    + calculateSigmoid(1.45, +(high >= 20) * Math.pow(orbs, 0.24), 2e6)
    + calculateSigmoid(1.55, +(high >= 25) * Math.pow(orbs, 0.21), 1e7)
    + calculateSigmoid(1.85, +(high >= 30) * Math.pow(orbs, 0.18), 4e7)
    + calculateSigmoid(3, +(high >= 35) * Math.pow(orbs, 0.15), 1e8)

  const warpBonus = input.autoWarpCheck ? 1 + input.dailyPowderResetUses : 1
  return (sigmoids - 11) * input.cubeToQuarkAllMult * warpBonus
}
