// Cube-upgrade cost + max-level math, migrated from packages/web_ui/src/Cubes.ts.
// The UI-side getters (cubeUpgradeDesc, updateCubeUpgradeBG, buyCubeUpgrades)
// stay in web_ui — they touch DOM, i18next, and call upgradeupdate. Logic
// owns just the pure cost-curve and cap math.

import {
  calculateCubicSumData,
  type CalculateCubicSumDataResult,
  calculateSummationNonLinear,
  type CalculateSummationNonLinearResult
} from '../math/summations'

// ─── Truth tables ──────────────────────────────────────────────────────────
//
// Per-upgrade base cost (index 0 ignored — cube upgrades are 1-indexed).
// dprint-ignore
const CUBE_BASE_COSTS: readonly number[] = [
  200, 200, 200, 500, 500, 500, 500, 500, 2000, 40000,
  5000, 1000, 10000, 20000, 40000, 10000, 4000, 1e4, 50000, 12500,
  5e4, 3e4, 3e4, 4e4, 2e5, 4e5, 1e5, 177777, 1e5, 1e6,
  5e5, 3e5, 2e6, 4e7, 4e7, 1e8, 1e8, 1e9, 2e9, 2e8,
  2e8, 5e8, 1e9, 2e9, 2e9, 5e8, 9876543210, 1e10, 42934819467, 1e8,
  1, 1e4, 1e8, 1e12, 1e16, 10, 1e5, 1e9, 1e13, 1e17,
  1e2, 1e6, 1e10, 1e14, 1e18, 1e20, 1e30, 1e40, 1e50, 1e60,
  1, 1, 1e8, 1e16, 1e30, 1e100, 1e100, 1e200, 1e250, 1e300
]

// Per-upgrade maximum level. Cube upgrade 57 (cookie row-leader bonus) bumps
// indices 1/11/21/31/41 by +1 — see getCubeMax.
// dprint-ignore
const CUBE_MAX_LEVELS: readonly number[] = [
  3, 10, 5, 1, 1, 1, 1, 1, 1, 1,
  3, 10, 1, 10, 10, 10, 5, 1, 1, 1,
  5, 10, 1, 10, 10, 10, 1, 1, 5, 1,
  5, 1, 1, 10, 10, 10, 10, 1, 1, 10,
  5, 10, 10, 10, 10, 20, 1, 1, 1, 100000,
  1, 900, 100, 900, 900, 20, 1, 1, 400, 10000,
  100, 1, 1, 1, 1, 1, 1, 1000, 1, 100000,
  1, 1, 5, 1, 30, 2, 25, 30, 1, 1
]

/**
 * Base cost (in wow cubes) for a single level of cube upgrade `index` (1-indexed).
 * The growth curve on top of this base differs by tier — see getCubeCost.
 * Web_ui's updateCubeUpgradeBG calls this when refunding over-leveled
 * upgrades after a max-level drop.
 */
export function getCubeUpgradeBaseCost(index: number): number {
  return CUBE_BASE_COSTS[index - 1]
}

// ─── getCubeMax ────────────────────────────────────────────────────────────

export interface GetCubeMaxInput {
  /** 1..80 (the cube upgrade index). */
  cubeUpgradeIndex: number
  /**
   * player.cubeUpgrades[57]. Once bought, the "row leader" upgrades (indices
   * 1, 11, 21, 31, 41 — i.e. `i % 10 === 1` and `i < 50`) get +1 max level.
   */
  cubeUpgrade57: number
}

export function getCubeMax(input: GetCubeMaxInput): number {
  let baseValue = CUBE_MAX_LEVELS[input.cubeUpgradeIndex - 1]
  if (
    input.cubeUpgrade57 > 0
    && input.cubeUpgradeIndex < 50
    && input.cubeUpgradeIndex % 10 === 1
  ) {
    baseValue += 1
  }
  return baseValue
}

// ─── getCubeCost ───────────────────────────────────────────────────────────

export interface GetCubeCostInput {
  /** 1..80. */
  cubeUpgradeIndex: number
  /** False = buy 1 level; true = buy up to 1e5 (non-cubic) or max (cubic). */
  buyMax: boolean
  /** player.cubeUpgrades[i]. */
  currentLevel: number
  /** Precomputed via getCubeMax(); avoids the wrapper double-computing it. */
  maxLevel: number
  /** Number(player.wowCubes). */
  wowCubes: number
  /**
   * For i ≤ 50: calculateSingularityDebuff('Cube Upgrades'). For i > 50:
   * pass 1 (the original web_ui code never applied the debuff above 50).
   */
  singularityDebuff: number
}

export type GetCubeCostResult = CalculateSummationNonLinearResult | CalculateCubicSumDataResult

/**
 * Cube cost curve. Three regimes:
 *
 *   i === 50           linear growth 0.01 + singularity-debuffed base
 *   i  <  50 (≠ 50)    flat per-level + singularity-debuffed base
 *   i  >  50           cubic sum, no singularity debuff
 *
 * Returns the standard `{ levelCanBuy, cost }` shape from the summation
 * primitives. Callers feed `cost` back into player.wowCubes.sub().
 */
export function getCubeCost(input: GetCubeCostInput): GetCubeCostResult {
  const i = input.cubeUpgradeIndex
  const linGrowth = i === 50 ? 0.01 : 0
  const cubic = i > 50
  const cubeUpgrade = input.currentLevel
  const baseCost = CUBE_BASE_COSTS[i - 1]

  if (cubic) {
    // Cubic regime uses a different buy-amount rule: buyMax goes up to
    // maxLevel; single-purchase goes one above current.
    const amountToBuy = input.buyMax ? input.maxLevel : Math.min(input.maxLevel, cubeUpgrade + 1)
    return calculateCubicSumData(cubeUpgrade, baseCost, input.wowCubes, amountToBuy)
  }

  let amountToBuy = input.buyMax ? 1e5 : 1
  amountToBuy = Math.min(input.maxLevel - cubeUpgrade, amountToBuy)
  return calculateSummationNonLinear(
    cubeUpgrade,
    baseCost * input.singularityDebuff,
    input.wowCubes,
    linGrowth,
    amountToBuy
  )
}
