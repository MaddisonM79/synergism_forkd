// Golden-quark upgrade effective-level math, lifted from
// packages/web_ui/src/singularity.ts. The web_ui side keeps the GQ upgrade
// data table and the buy/UI flow; this module owns the pure formulas that
// convert per-upgrade snapshot + player-state inputs into the effective
// level used by effect lookups, the level cap, and the polynomial bonus
// path unlocked by `octeractImprovedFree`.
//
// NOTE: octeract upgrades have a parallel set of helpers in
// `octeractUpgradeLevels.ts`. The two `computeFreeLevelMultiplier`
// implementations differ (GQ side reads shop + cube[75]; Octeract side
// reads cube[78]), so the names are kept distinct (`gqFreeLevelMultiplier`
// vs `octeractFreeLevelMultiplier`).

// Singularity counts that unlock an extra +1 to a GQ upgrade's level cap
// when the upgrade is flagged `canExceedCap`. Sorted ascending; we walk the
// array and stop at the first unmet threshold.
const overclockPerks = [50, 60, 75, 100, 125, 150, 175, 200, 225, 250]

/**
 * GQ free-level multiplier. Sums the shop `freeUpgradeMult` contribution
 * with `0.3% * cubeUpgrades[75]`. Used by both the free-level softcap and
 * the polynomial-path bonus.
 */
export function gqFreeLevelMultiplier (shopFreeUpgradeMult: number, cubeUpgrade75: number): number {
  return shopFreeUpgradeMult + 0.3 / 100 * cubeUpgrade75
}

/**
 * Effective free levels for one GQ upgrade. Above the player's purchased
 * level, free levels accumulate at a square-root rate — so the formula is
 * `min(level, baseRealFreeLevels) + sqrt(max(0, baseRealFreeLevels - level))`.
 * Both pieces matter: the `min` caps the linear contribution at the player's
 * actual purchases, the `sqrt` adds a softened bonus for unowned free levels.
 */
export function gqUpgradeFreeLevelSoftcap (freeLevel: number, level: number, freeLevelMult: number): number {
  const baseRealFreeLevels = freeLevelMult * freeLevel
  return Math.min(level, baseRealFreeLevels) + Math.sqrt(Math.max(0, baseRealFreeLevels - level))
}

export interface GqUpgradeMaxLevelInput {
  /** `goldenQuarkUpgrades[k].canExceedCap`. When false, returns maxLevel as-is. */
  canExceedCap: boolean
  /** The upgrade's base cap — `goldenQuarkUpgrades[k].maxLevel`. */
  maxLevel: number
  /** `player.highestSingularityCount` — feeds the overclock-perks loop. */
  highestSingularityCount: number
  /**
   * `getOcteractUpgradeEffect('octeractSingUpgradeCap', 'goldenQuarkUpgradeCapIncrease')`.
   * Added to the final cap for canExceedCap upgrades.
   */
  octeractSingUpgradeCapIncrease: number
}

/**
 * Maximum level for a GQ upgrade. For `canExceedCap` upgrades, walks the
 * overclock-perks array adding +1 per crossed threshold (stops at the first
 * unmet one), then adds the octeract-cap bonus. For non-`canExceedCap`
 * upgrades, returns `maxLevel` unchanged.
 */
export function computeGQUpgradeMaxLevel (input: GqUpgradeMaxLevelInput): number {
  if (!input.canExceedCap) {
    return input.maxLevel
  }
  let cap = input.maxLevel
  for (const perk of overclockPerks) {
    if (input.highestSingularityCount >= perk) {
      cap += 1
    } else {
      break
    }
  }
  cap += input.octeractSingUpgradeCapIncrease
  return cap
}

export interface ActualGQUpgradeTotalLevelsInput {
  /** The upgrade's purchased level — `goldenQuarkUpgrades[k].level`. */
  level: number
  /** The upgrade's accumulated free levels — `goldenQuarkUpgrades[k].freeLevel`. */
  freeLevel: number
  /** `goldenQuarkUpgrades[k].qualityOfLife`. QoL upgrades stay active inside the gating challenges. */
  qualityOfLife: boolean
  /**
   * True when `upgradeKey === 'platonicDelta'`. That upgrade gets a hard 0
   * inside the three Exalt challenges that gate it. The check is
   * upgrade-key-specific in web_ui; we accept it as a boolean here so logic
   * doesn't have to know the GQ upgrade key set.
   */
  isPlatonicDelta: boolean
  /** `player.singularityChallenges.noSingularityUpgrades.enabled`. */
  inNoSingularityUpgrades: boolean
  /** `player.singularityChallenges.sadisticPrequel.enabled`. */
  inSadisticPrequel: boolean
  /** `player.singularityChallenges.limitedAscensions.enabled`. */
  inLimitedAscensions: boolean
  /** `player.singularityChallenges.limitedTime.enabled`. */
  inLimitedTime: boolean
  /** GQ free-level multiplier — `gqFreeLevelMultiplier(shopMult, cubeUpgrades[75])`. */
  freeLevelMult: number
  /**
   * `getOcteractUpgradeEffect('octeractImprovedFree', 'unlocked')`. Gates
   * the polynomial-bonus path entirely — when false, the polynomial term
   * is skipped and only the linear sum contributes.
   */
  improvedFreeUnlocked: boolean
  /**
   * Sum of the four improved-free `freeLevelPower` / `freeLevelPowerIncrease`
   * octeract-upgrade effects. The polynomial term is
   * `(level * actualFreeLevels) ^ exponent`.
   */
  improvedFreeExponent: number
}

/**
 * Effective total level for one GQ upgrade.
 *
 * Three gating layers, in order:
 *
 * 1. If inside `noSingularityUpgrades` or `sadisticPrequel` AND the upgrade
 *    isn't `qualityOfLife`, returns 0.
 * 2. If the upgrade is platonicDelta AND inside `limitedAscensions`,
 *    `limitedTime`, or `sadisticPrequel`, returns 0.
 * 3. Otherwise: `linearLevels = level + actualFreeLevels`. If
 *    `octeractImprovedFree` is unlocked, also computes
 *    `polynomialLevels = (level * actualFreeLevels) ^ exponent` and returns
 *    the max of the two. If not unlocked, just returns linearLevels.
 */
export function actualGQUpgradeTotalLevels (input: ActualGQUpgradeTotalLevelsInput): number {
  if ((input.inNoSingularityUpgrades || input.inSadisticPrequel) && !input.qualityOfLife) {
    return 0
  }
  if (
    (input.inLimitedAscensions || input.inLimitedTime || input.inSadisticPrequel)
    && input.isPlatonicDelta
  ) {
    return 0
  }

  const actualFreeLevels = gqUpgradeFreeLevelSoftcap(input.freeLevel, input.level, input.freeLevelMult)
  const linearLevels = input.level + actualFreeLevels
  let polynomialLevels = 0

  if (input.improvedFreeUnlocked) {
    polynomialLevels = Math.pow(input.level * actualFreeLevels, input.improvedFreeExponent)
  }

  return Math.max(linearLevels, polynomialLevels)
}
