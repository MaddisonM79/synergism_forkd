// Red-ambrosia-derived bonuses lifted from packages/web_ui/src/Calculate.ts.
// Four functions, each one read `player.lifetimeRedAmbrosia` and combine it
// with a red-ambrosia-upgrade gate / exponent. The cookie-29 luck function
// also reads `player.cubeUpgrades[79]` as its second gate. The
// `getRedAmbrosiaUpgradeEffects(...)` lookups stay in web_ui — callers pass
// scalar inputs in.

// ─── Cookie upgrade 29 luck ────────────────────────────────────────────────

export interface CalculateCookieUpgrade29LuckInput {
  /** player.cubeUpgrades[79] — gates the bonus. 0 → no bonus. */
  cubeUpgrade79: number
  /** player.lifetimeRedAmbrosia — feeds the log10 in the formula. */
  lifetimeRedAmbrosia: number
}

/**
 * 10 × log10(lifetimeRedAmbrosia)^2 once both gates are non-zero, else 0.
 */
export function calculateCookieUpgrade29Luck(input: CalculateCookieUpgrade29LuckInput): number {
  if (input.cubeUpgrade79 === 0 || input.lifetimeRedAmbrosia === 0) {
    return 0
  }
  return 10 * Math.pow(Math.log10(input.lifetimeRedAmbrosia), 2)
}

// ─── Red ambrosia cube bonus ───────────────────────────────────────────────

export interface CalculateRedAmbrosiaCubesInput {
  /**
   * Truthy when `getRedAmbrosiaUpgradeEffects('redAmbrosiaCube',
   * 'unlockedRedAmbrosiaCube')` is set. Falsy → returns 1.
   */
  unlocked: boolean
  /** player.lifetimeRedAmbrosia. */
  lifetimeRedAmbrosia: number
  /**
   * `getRedAmbrosiaUpgradeEffects('redAmbrosiaCubeImprover',
   * 'extraExponent')` — added to the base 0.4 exponent.
   */
  extraExponent: number
}

/**
 * `1 + lifetimeRedAmbrosia ^ (0.4 + extraExponent) / 100` once the unlock is
 * set; otherwise 1.
 */
export function calculateRedAmbrosiaCubes(input: CalculateRedAmbrosiaCubesInput): number {
  if (!input.unlocked) {
    return 1
  }
  const exponent = 0.4 + input.extraExponent
  return 1 + Math.pow(input.lifetimeRedAmbrosia, exponent) / 100
}

// ─── Red ambrosia obtainium bonus ──────────────────────────────────────────

export interface CalculateRedAmbrosiaResourceInput {
  /**
   * Truthy when the corresponding red-ambrosia upgrade is unlocked. Falsy →
   * returns 1.
   */
  unlocked: boolean
  /** player.lifetimeRedAmbrosia. */
  lifetimeRedAmbrosia: number
}

/**
 * `1 + lifetimeRedAmbrosia^0.6 / 100` when unlocked; else 1.
 */
export function calculateRedAmbrosiaObtainium(input: CalculateRedAmbrosiaResourceInput): number {
  if (!input.unlocked) {
    return 1
  }
  return 1 + Math.pow(input.lifetimeRedAmbrosia, 0.6) / 100
}

/**
 * Same formula as the obtainium bonus, gated on the offering unlock instead.
 */
export function calculateRedAmbrosiaOffering(input: CalculateRedAmbrosiaResourceInput): number {
  if (!input.unlocked) {
    return 1
  }
  return 1 + Math.pow(input.lifetimeRedAmbrosia, 0.6) / 100
}
