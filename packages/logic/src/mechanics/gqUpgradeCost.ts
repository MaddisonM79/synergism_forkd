// GQ upgrade cost-to-next-level formula, lifted from
// packages/web_ui/src/singularity.ts. Four cost-form branches:
//
//   - Exponential2: costPerLevel * sqrt(overcapMult) * 2^level
//   - Cubic:        costPerLevel * overcapMult * ((level+1)^3 - level^3)
//   - Quadratic:    costPerLevel * overcapMult * ((level+1)^2 - level^2)
//   - default:      ceil(costPerLevel * (level+1) * overcapMult * noMaxLevelMult)
//
// The overcap multiplier (`4^(level - maxLevel + 1)`) applies whenever
// computedMaxLevel exceeds maxLevel (via overclock-perks / octeract cap
// bonuses) AND the player is past the base maxLevel. The default branch
// also has a no-max-level progression: x= maxLevel === -1 upgrades get
// multiplied by `level/50` past level 100 and `level/100` past level 400.
// Returns 0 when level === computedMaxLevel (fully maxed).

export type GQUpgradeSpecialCostForm = 'Exponential2' | 'Cubic' | 'Quadratic' | null

export interface GQUpgradeCostTNLInput {
  /** goldenQuarkUpgrades[k].level — current purchased level. */
  level: number
  /** goldenQuarkUpgrades[k].maxLevel — base cap (−1 sentinel for unlimited). */
  maxLevel: number
  /** computeGQUpgradeMaxLevel(k) — base cap plus overclock-perks plus octeract cap bonus. */
  computedMaxLevel: number
  /** goldenQuarkUpgrades[k].costPerLevel — base cost coefficient. */
  costPerLevel: number
  /**
   * goldenQuarkUpgrades[k].specialCostForm — one of three closed-form
   * shapes, or null for the default linear-with-overcap branch.
   */
  specialCostForm: GQUpgradeSpecialCostForm
}

/**
 * Cost to buy the next level of a GQ upgrade.
 *
 * Returns 0 if already maxed (level === computedMaxLevel).
 *
 * The overcap multiplier kicks in only for upgrades where overclock-perks /
 * octeract cap bonuses pushed `computedMaxLevel` past the base `maxLevel`,
 * AND only once the player is past the base `maxLevel`. The three
 * `specialCostForm` branches each use it differently:
 *   - Exponential2 takes sqrt(overcap) (softer scaling)
 *   - Cubic & Quadratic apply overcap directly to the level-delta cost
 *   - default applies overcap × the no-max-level progression
 *
 * The no-max-level progression (level/50 past 100, level/100 past 400) only
 * applies to default-form upgrades with maxLevel === -1.
 */
export function gqUpgradeCostTNL (input: GQUpgradeCostTNLInput): number {
  if (input.computedMaxLevel === input.level) {
    return 0
  }

  let costMultiplier = 1

  // Overcap multiplier — fires only when computedMaxLevel > base maxLevel
  // AND player is past the base maxLevel.
  if (input.computedMaxLevel > input.maxLevel && input.level >= input.maxLevel) {
    costMultiplier *= Math.pow(4, input.level - input.maxLevel + 1)
  }

  if (input.specialCostForm === 'Exponential2') {
    return input.costPerLevel * Math.sqrt(costMultiplier) * Math.pow(2, input.level)
  }

  if (input.specialCostForm === 'Cubic') {
    return input.costPerLevel * costMultiplier * (Math.pow(input.level + 1, 3) - Math.pow(input.level, 3))
  }

  if (input.specialCostForm === 'Quadratic') {
    return input.costPerLevel * costMultiplier * (Math.pow(input.level + 1, 2) - Math.pow(input.level, 2))
  }

  // Default linear branch with no-max-level progression.
  costMultiplier *= input.maxLevel === -1 && input.level >= 100 ? input.level / 50 : 1
  costMultiplier *= input.maxLevel === -1 && input.level >= 400 ? input.level / 100 : 1

  // Web_ui's original code re-checked maxed-out here; we've already returned
  // early at the top, so just compute the cost.
  return Math.ceil(input.costPerLevel * (1 + input.level) * costMultiplier)
}
