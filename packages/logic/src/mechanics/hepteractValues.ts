// Hepteract effective-value and cap helpers, lifted from
// packages/web_ui/src/Hepteracts.ts. Pure math given the per-hept snapshot:
//
//   - hepteractEffective: applies the diminishing-returns formula past
//     LIMIT. Special-cased for `quark` (which uses a custom nonpolynomial
//     formula web_ui handles separately — here we just pass BAL through
//     when the caller flags it as the quark hept).
//   - hepteractCap: BASE_CAP * 2^TIMES_CAP_EXTENDED — the player's
//     expanded cap before any Exalt 3 doubling.
//   - hepteractFinalCap: hepteractCap * (limitedAscensions Exalt 3 reward
//     active ? 2 : 1) — the post-Exalt cap actually used by the UI.

export interface HepteractEffectiveInput {
  /** hepteracts[k].BAL — raw accumulated value. */
  rawAmount: number
  /** hepteracts[k].LIMIT — threshold past which DR applies. */
  limit: number
  /**
   * hepteracts[k].DR + hepteracts[k].DR_INCREASE() — combined diminishing-
   * returns exponent. Web_ui evaluates DR_INCREASE since it can depend on
   * upgrade state; this module gets the resolved scalar.
   */
  drExponent: number
  /**
   * True when this is the `quark` hept. Quark hept has a custom non-
   * polynomial formula that web_ui owns; logic just passes BAL through.
   */
  isQuark: boolean
}

/**
 * Effective hepteract value with diminishing returns past LIMIT.
 *
 * - quark: just returns `rawAmount` (web_ui's custom formula owns this).
 * - rawAmount ≤ LIMIT: linear, returns `rawAmount`.
 * - rawAmount > LIMIT: `LIMIT * (rawAmount/LIMIT)^drExponent` — the
 *   value past LIMIT is softened by the DR exponent (drExponent < 1
 *   for most hepts, so growth past LIMIT is sub-linear).
 */
export function hepteractEffective (input: HepteractEffectiveInput): number {
  if (input.isQuark) {
    return input.rawAmount
  }

  let effectiveValue = Math.min(input.rawAmount, input.limit)
  if (input.rawAmount > input.limit) {
    effectiveValue *= Math.pow(input.rawAmount / input.limit, input.drExponent)
  }
  return effectiveValue
}

/**
 * Player's expanded hepteract cap before the Exalt 3 doubling.
 * `BASE_CAP * 2^TIMES_CAP_EXTENDED` — each expansion doubles the cap.
 */
export function hepteractCap (baseCap: number, timesCapExtended: number): number {
  return Math.pow(2, timesCapExtended) * baseCap
}

/**
 * The cap actually used by the UI — multiplies hepteractCap by 2 if the
 * limitedAscensions (Exalt 3) `hepteractCap` reward is active, else 1.
 */
export function hepteractFinalCap (baseCap: number, timesCapExtended: number, exalt3HepteractCap: boolean): number {
  const specialMultiplier = exalt3HepteractCap ? 2 : 1
  return hepteractCap(baseCap, timesCapExtended) * specialMultiplier
}
