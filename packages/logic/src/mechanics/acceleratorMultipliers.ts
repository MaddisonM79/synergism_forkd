// Total free-accelerator-boost and accelerator-multiplier formulas lifted
// from packages/web_ui/src/Calculate.ts. Both functions read a lot of
// player / G state; the web_ui shim collects every input field and passes
// them in scalar form.
//
// G side-effect note: the old `calculateTotalAcceleratorBoost` and
// `calculateAcceleratorMultiplier` wrote to G.freeAcceleratorBoost /
// G.totalAcceleratorBoost / G.acceleratorMultiplier directly. Logic stays
// pure — it returns the computed values; the web_ui shim does the G writes.

import { CalcECC } from './challenges'

// ─── Total accelerator boost (free + total) ────────────────────────────────

export interface CalculateTotalAcceleratorBoostInput {
  /** player.upgrades[26] — +1 free boost when > 0.5. */
  upgrade26: number
  /** player.upgrades[31] — adds totalCoinOwned/2000 (floored × 100 / 100) when > 0.5. */
  upgrade31: number
  /** Precomputed by `calculateTotalCoinOwned` in web_ui. */
  totalCoinOwned: number
  /**
   * +getAchievementReward('accelBoosts') — the achievement reward already
   * comes back as a number (the unary + is preserved by callers if needed).
   */
  achievementAccelBoosts: number
  /** player.researches[93] — multiplies floor(sumOfRuneLevels/20). */
  research93: number
  /** sumOfRuneLevels() in web_ui. */
  sumOfRuneLevels: number
  /** player.researches[3]. */
  research3: number
  /** player.challengecompletions[14] — feeds CalcECC('ascension', cc14). */
  challengeCompletions14: number
  /** player.researches[16], 17. */
  research16: number
  research17: number
  /** player.researches[88]. */
  research88: number
  /** getAntUpgradeEffect(AntUpgrades.AcceleratorBoosts).acceleratorBoostMult. */
  antBuildingAcceleratorBoostMult: number
  /** player.researches[127], 142, 157, 172, 187, 200. */
  research127: number
  research142: number
  research157: number
  research172: number
  research187: number
  research200: number
  /** player.cubeUpgrades[50]. */
  cubeUpgrade50: number
  /** hepteractEffective('acceleratorBoost') in web_ui. */
  hepteractEffectiveAcceleratorBoost: number
  /** player.upgrades[73] — doubles boost when also in a reincarnation challenge. */
  upgrade73: number
  /** True when player.currentChallenge.reincarnation !== 0. */
  inReincarnationChallenge: boolean
  /** player.acceleratorBoostBought — added to free boost for the total. */
  acceleratorBoostBought: number
}

export interface CalculateTotalAcceleratorBoostResult {
  /** What web_ui assigns to G.freeAcceleratorBoost. */
  freeAcceleratorBoost: number
  /** What web_ui assigns to G.totalAcceleratorBoost. */
  totalAcceleratorBoost: number
}

/**
 * Computes the "free" accelerator-boost amount from upgrades, achievements,
 * researches, rune levels, ant building effects, hepteract effectiveness,
 * and various cube upgrades. The reincarnation-challenge × upgrade[73] gate
 * doubles the result. Floored, then capped at 1e100.
 *
 * `totalAcceleratorBoost` = floor(bought + free × 100) / 100 — the original
 * floor-to-1-decimal that web_ui assigned to G.totalAcceleratorBoost.
 */
export function calculateTotalAcceleratorBoost(
  input: CalculateTotalAcceleratorBoostInput
): CalculateTotalAcceleratorBoostResult {
  let b = 0
  if (input.upgrade26 > 0.5) {
    b += 1
  }
  if (input.upgrade31 > 0.5) {
    b += (Math.floor(input.totalCoinOwned / 2000) * 100) / 100
  }
  b += input.achievementAccelBoosts

  b += input.research93 * Math.floor((1 / 20) * input.sumOfRuneLevels)
  b *= 1
    + (1 / 5)
      * input.research3
      * (1 + (1 / 2) * CalcECC('ascension', input.challengeCompletions14))
  b *= 1 + (1 / 20) * input.research16 + (1 / 20) * input.research17
  b *= 1 + (1 / 20) * input.research88
  b *= input.antBuildingAcceleratorBoostMult
  b *= 1 + (1 / 100) * input.research127
  b *= 1 + (0.8 / 100) * input.research142
  b *= 1 + (0.6 / 100) * input.research157
  b *= 1 + (0.4 / 100) * input.research172
  b *= 1 + (0.2 / 100) * input.research187
  b *= 1 + (0.01 / 100) * input.research200
  b *= 1 + (0.01 / 100) * input.cubeUpgrade50
  b *= 1 + (1 / 1000) * input.hepteractEffectiveAcceleratorBoost
  if (input.upgrade73 > 0.5 && input.inReincarnationChallenge) {
    b *= 2
  }
  b = Math.min(1e100, Math.floor(b))

  const freeAcceleratorBoost = b
  const totalAcceleratorBoost = (Math.floor(input.acceleratorBoostBought + freeAcceleratorBoost) * 100) / 100
  return { freeAcceleratorBoost, totalAcceleratorBoost }
}

// ─── Accelerator multiplier ────────────────────────────────────────────────

export interface CalculateAcceleratorMultiplierInput {
  /** player.researches[1]. */
  research1: number
  /** player.challengecompletions[14] — feeds CalcECC('ascension', cc14). */
  challengeCompletions14: number
  /** player.researches[6..10]. */
  research6: number
  research7: number
  research8: number
  research9: number
  research10: number
  /** player.researches[86], 126, 141, 156, 171, 186, 200. */
  research86: number
  research126: number
  research141: number
  research156: number
  research171: number
  research186: number
  research200: number
  /** player.cubeUpgrades[50]. */
  cubeUpgrade50: number
  /** player.upgrades[21..25] — sum is the exponent on 1.01. */
  upgrade21: number
  upgrade22: number
  upgrade23: number
  upgrade24: number
  upgrade25: number
  /** player.upgrades[50] — combined with the in-challenge gate, multiplies by 1.25. */
  upgrade50: number
  /**
   * True when player.currentChallenge.transcension !== 0 OR
   * player.currentChallenge.reincarnation !== 0.
   */
  inTranscensionOrReincarnationChallenge: boolean
}

/**
 * Compounding multiplier built from research levels, cube upgrades, the
 * 21..25 upgrade pentad (1.01^sum), and an optional 1.25× from
 * upgrade[50] while in a transcension or reincarnation challenge.
 */
export function calculateAcceleratorMultiplier(input: CalculateAcceleratorMultiplierInput): number {
  let multiplier = 1
  multiplier *= 1
    + (1 / 5)
      * input.research1
      * (1 + (1 / 2) * CalcECC('ascension', input.challengeCompletions14))
  multiplier *= 1
    + (1 / 20) * input.research6
    + (1 / 25) * input.research7
    + (1 / 40) * input.research8
    + (3 / 200) * input.research9
    + (1 / 200) * input.research10
  multiplier *= 1 + (1 / 20) * input.research86
  multiplier *= 1 + (1 / 100) * input.research126
  multiplier *= 1 + (0.8 / 100) * input.research141
  multiplier *= 1 + (0.6 / 100) * input.research156
  multiplier *= 1 + (0.4 / 100) * input.research171
  multiplier *= 1 + (0.2 / 100) * input.research186
  multiplier *= 1 + (0.01 / 100) * input.research200
  multiplier *= 1 + (0.01 / 100) * input.cubeUpgrade50
  multiplier *= Math.pow(
    1.01,
    input.upgrade21 + input.upgrade22 + input.upgrade23 + input.upgrade24 + input.upgrade25
  )
  if (input.inTranscensionOrReincarnationChallenge && input.upgrade50 > 0.5) {
    multiplier *= 1.25
  }
  return multiplier
}
