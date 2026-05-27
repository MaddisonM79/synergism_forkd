// Quark export accumulator math. The web_ui side wraps this and gathers
// the player/octeract inputs; the QuarkHandler class and the personal/global
// quark bonus state stay in web_ui (DOM + i18next + fetch).

export interface QuarkHandlerInput {
  /**
   * player.researches[195] — "Research 8x20". Each level adds 18000s
   * (5 hours) to the export cap. Zero means the base 90000s window.
   */
  research195: number
  /**
   * Sum of player.researches at the five quark-yielding slots:
   * researches[99] + [100] + [125] + [180] + [195]. Added to the base
   * 5 quarks/hour rate before the octeract multiplier.
   */
  researchesSum: number
  /**
   * getOcteractUpgradeEffect('octeractExportQuarks', 'exportQuarkMult') —
   * multiplicative boost on the per-hour rate. Defaults to 1 if the upgrade
   * isn't bought.
   */
  exportQuarkMult: number
  /**
   * player.quarkstimer — seconds of accumulated export time. The actual
   * quark gain is `floor(quarksTimer * perHour / 3600)`; capped externally
   * by web_ui when timer exceeds maxTime.
   */
  quarksTimer: number
  /**
   * calculateCubeQuarkMultiplier() — already migrated, passed through
   * unchanged. Kept here so callers get a single object to destructure.
   */
  cubeMult: number
}

export interface QuarkHandlerResult {
  /** Maximum accumulator window, in seconds. */
  maxTime: number
  /** Effective quarks/hour given all bonuses. */
  perHour: number
  /** floor(perHour * maxTime / 3600) — total quarks the cap can hold. */
  capacity: number
  /** floor(quarksTimer * perHour / 3600) — quarks gained right now. */
  gain: number
  /** Pass-through of input.cubeMult. */
  cubeMult: number
}

/**
 * Computes the export-time / per-hour / capacity / current-gain quartet
 * used by every quark-display surface. Verbatim from web_ui's quarkHandler
 * with the five player/G reads lifted into explicit inputs.
 */
export function quarkHandler(input: QuarkHandlerInput): QuarkHandlerResult {
  let maxTime = 90000
  if (input.research195 > 0) {
    maxTime += 18000 * input.research195
  }

  const perHour = (5 + input.researchesSum) * input.exportQuarkMult
  const capacity = Math.floor(perHour * maxTime / 3600)
  const gain = Math.floor(input.quarksTimer * perHour / 3600)

  return {
    maxTime,
    perHour,
    capacity,
    gain,
    cubeMult: input.cubeMult
  }
}
