// Platonic-upgrade cost table + price-multiplier formula + affordability
// check. Lifted from packages/web_ui/src/Platonic.ts (lines 8-319). The
// display function (createPlatonicDescription), DOM updates, and the
// player-mutating buy loop stay in web_ui. Logic owns the static data and
// the math.
//
// Price multiplier shape:
//   priceMultiplier =
//     (priceMult is undefined ? 1 : priceMult ^ (currentLevel/(maxLevel-1))^1.25)
//     × singularityDebuff
// Affordability is per-resource: cost = floor(baseCost * priceMultiplier) ≤
// playerHas. The auto-mode flag exempts obtainium/offerings from the check
// (auto-buy doesn't actually consume those for platonic upgrades).

export interface PlatonicUpgradeBaseCost {
  obtainium: number
  offerings: number
  cubes: number
  tesseracts: number
  hypercubes: number
  platonics: number
  abyssals: number
  maxLevel: number
  /** Undefined for upgrades that don't scale with level. */
  priceMult?: number
}

/** Resource keys in fixed order, matching the legacy iteration. The first
 * six are checked against `currentResources`; `abyssals` is checked against
 * `abyssalBalance` because it lives on hepteracts.abyss.BAL, not player. */
export const platonicResources = [
  'obtainium',
  'offerings',
  'cubes',
  'tesseracts',
  'hypercubes',
  'platonics',
  'abyssals'
] as const

export type PlatonicResourceKey = typeof platonicResources[number]

/** 20-entry cost table, indices 1..20. Note: index 0 is intentionally
 * unused — platonic upgrade IDs are 1-based. */
export const platonicUpgradeBaseCosts: Record<number, PlatonicUpgradeBaseCost> = {
  1: {
    obtainium: 1,
    offerings: 1e45,
    cubes: 1e13,
    tesseracts: 1e6,
    hypercubes: 1e5,
    platonics: 1e4,
    abyssals: 0,
    maxLevel: 300,
    priceMult: 2
  },
  2: {
    obtainium: 1,
    offerings: 3e45,
    cubes: 1e11,
    tesseracts: 1e8,
    hypercubes: 1e5,
    platonics: 1e4,
    abyssals: 0,
    maxLevel: 300,
    priceMult: 2
  },
  3: {
    obtainium: 1,
    offerings: 1e46,
    cubes: 1e11,
    tesseracts: 1e6,
    hypercubes: 1e7,
    platonics: 1e4,
    abyssals: 0,
    maxLevel: 300,
    priceMult: 2
  },
  4: {
    obtainium: 1,
    offerings: 3e46,
    cubes: 1e12,
    tesseracts: 1e7,
    hypercubes: 1e6,
    platonics: 1e6,
    abyssals: 0,
    maxLevel: 300,
    priceMult: 2
  },
  5: {
    obtainium: 1,
    offerings: 1e59,
    cubes: 1e14,
    tesseracts: 1e9,
    hypercubes: 1e8,
    platonics: 1e7,
    abyssals: 0,
    maxLevel: 1
  },
  6: {
    obtainium: 1,
    offerings: 1e61,
    cubes: 1e15,
    tesseracts: 1e9,
    hypercubes: 1e8,
    platonics: 1e7,
    abyssals: 0,
    maxLevel: 10
  },
  7: {
    obtainium: 1,
    offerings: 3e62,
    cubes: 2e15,
    tesseracts: 2e9,
    hypercubes: 2e8,
    platonics: 1.5e7,
    abyssals: 0,
    maxLevel: 15
  },
  8: {
    obtainium: 1,
    offerings: 1e64,
    cubes: 4e15,
    tesseracts: 4e9,
    hypercubes: 4e8,
    platonics: 3e7,
    abyssals: 0,
    maxLevel: 5
  },
  9: {
    obtainium: 1,
    offerings: 1e66,
    cubes: 1e16,
    tesseracts: 1e10,
    hypercubes: 1e9,
    platonics: 5e7,
    abyssals: 0,
    maxLevel: 1
  },
  10: {
    obtainium: 1,
    offerings: 1e68,
    cubes: 1e18,
    tesseracts: 1e12,
    hypercubes: 1e11,
    platonics: 1e9,
    abyssals: 0,
    maxLevel: 1
  },
  11: {
    obtainium: 1,
    offerings: 1e70,
    cubes: 2e17,
    tesseracts: 2e11,
    hypercubes: 2e10,
    platonics: 2e8,
    abyssals: 0,
    maxLevel: 1
  },
  12: {
    obtainium: 1,
    offerings: 1e72,
    cubes: 1e18,
    tesseracts: 1e12,
    hypercubes: 1e11,
    platonics: 1e9,
    abyssals: 0,
    maxLevel: 10
  },
  13: {
    obtainium: 1,
    offerings: 1e74,
    cubes: 2e19,
    tesseracts: 4e12,
    hypercubes: 4e11,
    platonics: 4e9,
    abyssals: 0,
    maxLevel: 1
  },
  14: {
    obtainium: 1,
    offerings: 1e77,
    cubes: 4e20,
    tesseracts: 1e13,
    hypercubes: 1e12,
    platonics: 1e10,
    abyssals: 0,
    maxLevel: 1
  },
  15: {
    obtainium: 1,
    offerings: 1e80,
    cubes: 1e23,
    tesseracts: 1e15,
    hypercubes: 1e14,
    platonics: 1e12,
    abyssals: 1,
    maxLevel: 1
  },
  16: {
    obtainium: 1,
    offerings: 1e110,
    cubes: 0,
    tesseracts: 0,
    hypercubes: 2.5e15,
    platonics: 0,
    abyssals: 0,
    maxLevel: 100,
    priceMult: 10
  },
  17: {
    obtainium: 1,
    offerings: 1e113,
    cubes: 0,
    tesseracts: 0,
    hypercubes: 1e19,
    platonics: 0,
    abyssals: 2,
    maxLevel: 20,
    priceMult: 10
  },
  18: {
    obtainium: 1,
    offerings: 1e116,
    cubes: 0,
    tesseracts: 0,
    hypercubes: 1e19,
    platonics: 0,
    abyssals: 4,
    maxLevel: 40,
    priceMult: 500
  },
  19: {
    obtainium: 1,
    offerings: 1e121,
    cubes: 0,
    tesseracts: 0,
    hypercubes: 1e21,
    platonics: 0,
    abyssals: 64,
    maxLevel: 50,
    priceMult: 200
  },
  20: {
    obtainium: 1,
    offerings: 1e130,
    cubes: 1e45,
    tesseracts: 1e28,
    hypercubes: 1e25,
    platonics: 1e25,
    abyssals: Math.pow(2, 30) - 1,
    maxLevel: 1
  }
}

export interface PlatonicUpgradePriceMultiplierInput {
  /** baseCost.priceMult — undefined for unscaled upgrades (no per-level cost growth). */
  priceMult: number | undefined
  /** player.platonicUpgrades[index]. */
  currentLevel: number
  /** baseCost.maxLevel. */
  maxLevel: number
  /** calculateSingularityDebuff('Platonic Costs'). */
  singularityDebuff: number
}

/**
 * Cost-scaling multiplier applied to every resource cost for one platonic
 * upgrade. The level-scaling exponent `(currentLevel / (maxLevel-1)) ^ 1.25`
 * produces accelerating costs as the upgrade approaches its cap. Then
 * multiplied by the singularity debuff.
 *
 * When `priceMult` is undefined (single-buy upgrades like #5, #9-#11), the
 * level-scaling factor is 1 and only the singularity debuff applies.
 */
export function platonicUpgradePriceMultiplier (input: PlatonicUpgradePriceMultiplierInput): number {
  let priceMultiplier = 1
  if (input.priceMult !== undefined) {
    priceMultiplier = Math.pow(
      input.priceMult,
      Math.pow(input.currentLevel / (input.maxLevel - 1), 1.25)
    )
  }
  return priceMultiplier * input.singularityDebuff
}

export interface CheckPlatonicUpgradeInput {
  /** Upgrade index (1..20). */
  index: number
  /** player.platonicUpgrades[index]. */
  currentLevel: number
  /** Already-computed price multiplier — caller passes the result of
   * `platonicUpgradePriceMultiplier()`. */
  priceMultiplier: number
  /** auto = true ⇒ skip obtainium and offerings checks (auto-buy doesn't
   * actually consume them). */
  autoMode: boolean
  /** Per-resource current balances. Indexed by the same keys as
   * PlatonicUpgradeBaseCost (excluding abyssals + maxLevel + priceMult). */
  currentResources: Record<Exclude<PlatonicResourceKey, 'abyssals'>, number>
  /** hepteracts.abyss.BAL — abyssals balance lives on the hepteract, not
   * on player directly. */
  abyssalBalance: number
}

/** Per-resource affordability flags. `canBuy` is `true` iff every checked
 * resource passed AND the upgrade isn't already maxed. */
export type PlatonicUpgradeAffordability = Record<PlatonicResourceKey | 'canBuy', boolean>

/**
 * Affordability check across all 7 resources + max-level gate. Mirrors the
 * legacy iteration order and the auto-mode obtainium/offerings exemption.
 *
 * The abyssals branch has an extra "if baseCost is 0, it's a free check"
 * shortcut so upgrades that don't cost abyssals always pass that check
 * regardless of the player's hepteract balance.
 */
export function checkPlatonicUpgradeAffordability (
  input: CheckPlatonicUpgradeInput
): PlatonicUpgradeAffordability {
  const baseCost = platonicUpgradeBaseCosts[input.index]
  const checks: PlatonicUpgradeAffordability = {
    obtainium: false,
    offerings: false,
    cubes: false,
    tesseracts: false,
    hypercubes: false,
    platonics: false,
    abyssals: false,
    canBuy: false
  }

  let checksum = 0
  // Loop the first six resources (everything except abyssals).
  for (let i = 0; i < platonicResources.length - 1; i++) {
    const key = platonicResources[i] as Exclude<PlatonicResourceKey, 'abyssals'>
    if (input.autoMode && (key === 'obtainium' || key === 'offerings')) {
      checksum++
      checks[key] = true
    } else if (
      Math.floor(baseCost[key] * input.priceMultiplier) <= input.currentResources[key]
    ) {
      checksum++
      checks[key] = true
    }
  }

  // Abyssals: either upgrade doesn't cost any, or hepteract balance covers cost.
  if (
    input.abyssalBalance >= Math.floor(baseCost.abyssals * input.priceMultiplier)
    || baseCost.abyssals === 0
  ) {
    checksum++
    checks.abyssals = true
  }

  if (checksum === platonicResources.length && input.currentLevel < baseCost.maxLevel) {
    checks.canBuy = true
  }
  return checks
}
