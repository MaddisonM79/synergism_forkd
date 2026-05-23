// Summation / cost helpers lifted from packages/web_ui/src/Calculate.ts.
// Generic algebraic primitives used by the upgrade-cost code in web_ui.
//
// Error path: the web_ui versions threw Error(i18next.t('calculate.*Error')).
// Logic isn't allowed to call i18next, so it throws plain Errors with
// machine-readable codes (`SUMMATIONS_QUADRATIC_IMPROPER` etc.). Callers
// that want to surface a translated message can catch and re-throw — but
// these guards fire only on invalid inputs (programmer error), so in
// practice they're dev-facing.

// ─── Linear non-linear-cost summation (cost = base * (1 + level * d)) ──────

export interface CalculateSummationNonLinearResult {
  levelCanBuy: number
  cost: number
}

/**
 * If costs grow as `base * (1 + level * diffPerLevel)` from `baseLevel`,
 * compute how many levels can be bought with `resourceAvailable` (capped at
 * `baseLevel + buyAmount`) and the total cost spent.
 */
export function calculateSummationNonLinear(
  baseLevel: number,
  baseCost: number,
  resourceAvailable: number,
  diffPerLevel: number,
  buyAmount: number
): CalculateSummationNonLinearResult {
  const c = diffPerLevel / 2
  resourceAvailable = resourceAvailable || 0
  const alreadySpent = baseCost * (c * Math.pow(baseLevel, 2) + baseLevel * (1 - c))
  resourceAvailable += alreadySpent
  const v = resourceAvailable / baseCost
  let buyToLevel = c > 0
    ? Math.max(
      0,
      Math.floor(
        (c - 1) / (2 * c)
          + Math.pow(Math.pow(1 - c, 2) + 4 * c * v, 1 / 2) / (2 * c)
      )
    )
    : Math.floor(v)

  buyToLevel = Math.min(buyToLevel, buyAmount + baseLevel)
  buyToLevel = Math.max(buyToLevel, baseLevel)
  let totalCost = baseCost * (c * Math.pow(buyToLevel, 2) + buyToLevel * (1 - c))
    - alreadySpent
  if (buyToLevel === baseLevel) {
    totalCost = baseCost * (1 + 2 * c * baseLevel)
  }
  return {
    levelCanBuy: buyToLevel,
    cost: totalCost
  }
}

// ─── Cubic summation + quadratic solver ────────────────────────────────────

/**
 * Sum of the first `n` positive cubes — closed form `(n(n+1)/2)^2`.
 * Returns 0 if `n === 0`, or -1 if `n` is negative / non-integer (matches
 * the validation behavior of the original web_ui helper).
 */
export function calculateSummationCubic(n: number): number {
  if (n < 0) {
    return -1
  }
  if (!Number.isInteger(n)) {
    return -1
  }
  return Math.pow((n * (n + 1)) / 2, 2)
}

/**
 * Real-root solver for `a * n^2 + b * n + c = 0`. `a` must be nonneg and
 * the discriminant must be non-negative; otherwise throws.
 *
 * `positive` selects which root to return: true → `(-b + sqrt(disc)) / (2a)`,
 * false → `(-b - sqrt(disc)) / (2a)`. When the discriminant is 0 both forms
 * collapse to `-b / (2a)`.
 */
export function solveQuadratic(a: number, b: number, c: number, positive: boolean): number {
  if (a < 0) {
    throw new Error('SUMMATIONS_QUADRATIC_IMPROPER')
  }
  const determinant = Math.pow(b, 2) - 4 * a * c
  if (determinant < 0) {
    throw new Error('SUMMATIONS_QUADRATIC_DETERMINANT')
  }

  if (determinant === 0) {
    return -b / (2 * a)
  }
  const root = Math.sqrt(determinant)
  return positive ? (-b + root) / (2 * a) : (-b - root) / (2 * a)
}

export interface CalculateCubicSumDataResult {
  levelCanBuy: number
  cost: number
}

/**
 * Cubic-cost upgrade-batch solver: if level i costs `baseCost * (i+1)^3`,
 * compute how many levels can be bought given `initialLevel` already owned
 * and `amountToSpend` more available, capped at `maxLevel`. Returns the
 * `{levelCanBuy, cost}` pair where `cost` is the actual amount that would
 * be spent.
 *
 * Throws on negative `totalToSpend` (programmer-error guard).
 */
export function calculateCubicSumData(
  initialLevel: number,
  baseCost: number,
  amountToSpend: number,
  maxLevel: number
): CalculateCubicSumDataResult {
  if (initialLevel >= maxLevel) {
    return {
      levelCanBuy: maxLevel,
      cost: 0
    }
  }
  const alreadySpent = baseCost * calculateSummationCubic(initialLevel)
  const totalToSpend = alreadySpent + amountToSpend

  if (totalToSpend < 0) {
    throw new Error('SUMMATIONS_CUBIC_SUM_NEGATIVE')
  }

  const determinantRoot = Math.pow(totalToSpend / baseCost, 0.5)
  const solution = solveQuadratic(1, 1, -2 * determinantRoot, true)

  const levelToBuy = Math.max(
    Math.min(maxLevel, Math.floor(solution)),
    initialLevel
  )
  const realCost = levelToBuy === initialLevel
    ? baseCost * Math.pow(initialLevel + 1, 3)
    : baseCost * calculateSummationCubic(levelToBuy) - alreadySpent

  return {
    levelCanBuy: levelToBuy,
    cost: realCost
  }
}
