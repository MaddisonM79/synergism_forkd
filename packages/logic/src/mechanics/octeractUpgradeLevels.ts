// Octeract upgrade effective-level math, lifted from
// packages/web_ui/src/Octeracts.ts. The web_ui side owns the OcteractUpgrades
// data tables and the buy/UI flow; this module owns the pure formulas that
// take a per-upgrade `level` / `freeLevel` / `qualityOfLife` snapshot plus
// the player-state inputs (cubeUpgrades[78] for the multiplier, the two
// no-octeracts-style challenge gates) and return the effective level used by
// effect lookups and cost-to-next-level calculations.
//
// NOTE: the same name `computeFreeLevelMultiplier` exists in
// packages/web_ui/src/singularity.ts for golden-quark upgrades, but that's a
// *different* formula (shop + cube[75], not just cube[78]). The GQ-side math
// has its own logic module (`gqUpgradeLevels.ts`).

/**
 * Octeract free-level multiplier. Just `1 + 0.3% * cubeUpgrades[78]`.
 * cubeUpgrades[78] is the only input the Octeract side reads.
 */
export function octeractFreeLevelMultiplier (cubeUpgrade78: number): number {
  return 1 + 0.3 / 100 * cubeUpgrade78
}

/**
 * Softcap on the effective free levels for one octeract upgrade. Just the
 * upgrade's `freeLevel` scaled by `freeLevelMult` — kept as a named export
 * because the web_ui display code calls it directly.
 */
export function octeractFreeLevelSoftcap (freeLevel: number, freeLevelMult: number): number {
  return freeLevel * freeLevelMult
}

export interface ActualOcteractUpgradeTotalLevelsInput {
  /** The upgrade's purchased level — `octeractUpgrades[k].level`. */
  level: number
  /** The upgrade's accumulated free levels — `octeractUpgrades[k].freeLevel`. */
  freeLevel: number
  /**
   * `octeractUpgrades[k].qualityOfLife`. When false, the upgrade is gated
   * off inside noOcteracts / sadisticPrequel (returns 0).
   */
  qualityOfLife: boolean
  /** `player.cubeUpgrades[78]` — feeds the multiplier. */
  cubeUpgrade78: number
  /** `player.singularityChallenges.noOcteracts.enabled`. */
  inNoOcteracts: boolean
  /** `player.singularityChallenges.sadisticPrequel.enabled`. */
  inSadisticPrequel: boolean
}

/**
 * Effective total level for one octeract upgrade.
 *
 * - Returns 0 if the player is in `noOcteracts` or `sadisticPrequel` AND the
 *   upgrade isn't flagged `qualityOfLife` (quality-of-life upgrades stay
 *   active inside those challenges).
 * - Otherwise: when `level ≥ actualFreeLevels`, returns
 *   `actualFreeLevels + level` (linear sum). When `level < actualFreeLevels`,
 *   returns `2 * sqrt(actualFreeLevels * level)` — a smoother softcap that
 *   matches the linear formula at `level == actualFreeLevels`.
 */
export function actualOcteractUpgradeTotalLevels (input: ActualOcteractUpgradeTotalLevelsInput): number {
  if ((input.inNoOcteracts || input.inSadisticPrequel) && !input.qualityOfLife) {
    return 0
  }

  const freeLevelMult = octeractFreeLevelMultiplier(input.cubeUpgrade78)
  const actualFreeLevels = octeractFreeLevelSoftcap(input.freeLevel, freeLevelMult)

  if (input.level >= actualFreeLevels) {
    return actualFreeLevels + input.level
  }
  return 2 * Math.sqrt(actualFreeLevels * input.level)
}
