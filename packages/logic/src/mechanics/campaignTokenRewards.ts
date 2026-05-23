// Per-reward campaign-token bonus formulas, lifted from
// packages/web_ui/src/Campaign.ts.
//
// Web_ui still owns the CampaignManager class wrapper (it bundles these
// formulas onto getter properties for ergonomic consumption) and the
// per-campaign data table. This module owns the 14 pure bonus formulas
// that each take the current campaignTokens count and return a scalar
// (or, for tutorialBonus, a 3-field object). All formulas are pure
// functions of `campaignTokens` with no player or game-state reads.
//
// The two threshold constant arrays (timeThreshold + bonusRune6) move
// here too — they're pure data driving the staircase functions.

export interface CampaignTutorialBonus {
  cubeBonus: number
  obtainiumBonus: number
  offeringBonus: number
}

// ─── Threshold constants ──────────────────────────────────────────────────

// Verbatim from legacy. The staircase functions return their index into
// these arrays (0..N) clamped by the array length. Pre-300-singularity
// max returns are baked in: timeThreshold tops out at 2, bonusRune6 at 12.
const timeThresholdReqs = [20, 100, 250, 500, 1000, 2000, 3500, 5000]
const bonusRune6ThresholdReqs = [500, 750, 1000, 1250, 1500, 1750, 2000, 3000, 4000, 6000, 8000, 10000]

// ─── Bonus formulas ───────────────────────────────────────────────────────

// tutorialBonus: three independent boolean-shaped bonuses, all active iff
// any campaign token has been earned. Verbatim from legacy.
export function tutorialBonus (campaignTokens: number): CampaignTutorialBonus {
  return {
    cubeBonus: 1 + 0.25 * +(campaignTokens > 0),
    obtainiumBonus: 1 + 0.2 * +(campaignTokens > 0),
    offeringBonus: 1 + 0.2 * +(campaignTokens > 0)
  }
}

// cubeBonus / obtainiumBonus / offeringBonus all share the same three-band
// piecewise: linear-ramp 0..25, then saturating exp toward bigger caps.
// Different scaling constants per bonus.

export function campaignCubeBonus (campaignTokens: number): number {
  return 1
    + 0.4 * 1 / 25 * Math.min(campaignTokens, 25)
    + 0.6 * (1 - Math.exp(-Math.max(campaignTokens - 25, 0) / 500))
    + 1 * (1 - Math.exp(-Math.max(campaignTokens - 2500, 0) / 5000))
}

export function campaignObtainiumBonus (campaignTokens: number): number {
  return 1
    + 0.1 * 1 / 25 * Math.min(campaignTokens, 25)
    + 0.4 * (1 - Math.exp(-Math.max(campaignTokens - 25, 0) / 500))
    + 0.5 * (1 - Math.exp(-Math.max(campaignTokens - 2500, 0) / 5000))
}

export function campaignOfferingBonus (campaignTokens: number): number {
  return 1
    + 0.1 * 1 / 25 * Math.min(campaignTokens, 25)
    + 0.4 * (1 - Math.exp(-Math.max(campaignTokens - 25, 0) / 500))
    + 0.5 * (1 - Math.exp(-Math.max(campaignTokens - 2500, 0) / 5000))
}

// ascensionScoreMultiplier uses a wider linear band (0..100) before the
// exp terms kick in.
export function campaignAscensionScoreMultiplier (campaignTokens: number): number {
  return 1
    + 0.2 * 1 / 100 * Math.min(campaignTokens, 100)
    + 0.3 * (1 - Math.exp(-Math.max(campaignTokens - 100, 0) / 1000))
    + 0.5 * (1 - Math.exp(-Math.max(campaignTokens - 2500, 0) / 5000))
}

// timeThresholdReduction: staircase that returns i/4 for the first index
// past which campaignTokens falls below, capped at 2 (after 8 thresholds).
// Each step adds 0.25, total range 0..2.
export function campaignTimeThresholdReduction (campaignTokens: number): number {
  for (let i = 0; i < timeThresholdReqs.length; i++) {
    if (campaignTokens < timeThresholdReqs[i]) {
      return i / 4
    }
  }
  return 2
}

// quarkBonus: gated until campaignTokens >= 100, then a three-band
// piecewise. Below the gate returns 1.
export function campaignQuarkBonus (campaignTokens: number): number {
  if (campaignTokens < 100) {
    return 1
  }
  return 1
    + 0.05 * Math.min(campaignTokens - 100, 100) / 100
    + 0.05 * (1 - Math.exp(-Math.max(campaignTokens - 200, 0) / 3000))
    + 0.1 * (1 - Math.exp(-Math.max(campaignTokens - 2500, 0) / 10000))
}

// taxMultiplier: gated at >=250, then a *negative* three-band piecewise.
// Output decreases below 1 (tax-reducing).
export function campaignTaxMultiplier (campaignTokens: number): number {
  if (campaignTokens < 250) {
    return 1
  }
  return 1
    - 0.05 * 1 / 250 * Math.min(campaignTokens - 250, 250)
    - 0.15 * (1 - Math.exp(-Math.max(campaignTokens - 500, 0) / 1250))
    - 0.05 * (1 - Math.exp(-Math.max(campaignTokens - 4000, 0) / 5000))
}

// c15Bonus: gated at >=250, two-band positive piecewise.
export function campaignC15Bonus (campaignTokens: number): number {
  if (campaignTokens < 250) {
    return 1
  }
  return 1
    + 0.05 * 1 / 250 * Math.min(campaignTokens - 250, 250)
    + 0.95 * (1 - Math.exp(-Math.max(campaignTokens - 500, 0) / 1250))
}

// bonusRune6: staircase that returns the index of the first threshold the
// player hasn't passed. Returns 0..12 (capped at 12 after all 12 thresholds).
export function campaignBonusRune6 (campaignTokens: number): number {
  for (let i = 0; i < bonusRune6ThresholdReqs.length; i++) {
    if (campaignTokens < bonusRune6ThresholdReqs[i]) {
      return i
    }
  }
  return 12
}

// goldenQuarkBonus: gated at >=500, two-band piecewise.
export function campaignGoldenQuarkBonus (campaignTokens: number): number {
  if (campaignTokens < 500) {
    return 1
  }
  return 1
    + 0.05 * 1 / 500 * Math.min(campaignTokens - 500, 500)
    + 0.05 * (1 - Math.exp(-Math.max(campaignTokens - 1000, 0) / 2500))
}

// octeractBonus: gated at >=1000, two-band piecewise.
export function campaignOcteractBonus (campaignTokens: number): number {
  if (campaignTokens < 1000) {
    return 1
  }
  return 1
    + 0.1 * 1 / 1000 * Math.min(campaignTokens - 1000, 1000)
    + 0.15 * (1 - Math.exp(-Math.max(campaignTokens - 2000, 0) / 4000))
}

// ambrosiaLuckBonus: gated at >=2000, *additive* style — base value 10
// (not 1) past the gate. Returns 0 below.
export function campaignAmbrosiaLuckBonus (campaignTokens: number): number {
  if (campaignTokens < 2000) {
    return 0
  }
  return 10
    + 40 * 1 / 2000 * Math.min(campaignTokens - 2000, 2000)
    + 50 * (1 - Math.exp(-Math.max(campaignTokens - 4000, 0) / 2500))
}

// blueberrySpeedBonus: gated at >=2000, two-band piecewise.
export function campaignBlueberrySpeedBonus (campaignTokens: number): number {
  if (campaignTokens < 2000) {
    return 1
  }
  return 1
    + 0.02 * 1 / 2000 * Math.min(campaignTokens - 2000, 2000)
    + 0.03 * (1 - Math.exp(-Math.max(campaignTokens - 4000, 0) / 2000))
}
