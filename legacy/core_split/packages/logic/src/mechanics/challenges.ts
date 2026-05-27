// Challenge math. The full Challenges.ts in web_ui is mostly UI + automation
// state — but a handful of helpers are pure number-in-number-out functions
// that get called from many other modules (Buy.ts, Runes.ts, Statistics, ...).
// Migrate those one at a time; the UI + sweep state machine stays in web_ui.

import { Decimal } from '../math/bignum'

export type ChallengeType = 'transcend' | 'reincarnation' | 'ascension'

/**
 * Effective Challenge Completions. Three piecewise linear curves keyed by
 * challenge tier, each diminishing returns past the first knee:
 *
 *   transcend:      [0..100] 1×, [100..1000] 0.05×, past 1000 0.01×
 *   reincarnation:  [0..25]  1×, [25..75]   0.5×,   past 75   0.1×
 *   ascension:      [0..10]  1×, past 10   0.5×
 *
 * Pure: depends only on its two arguments. Used everywhere — the cost
 * formulas in producers/accelerators/multipliers all read transcendECC for
 * their challenge-4 amplifier, and the rune EXP curve uses it too.
 */
export function CalcECC(
  type: ChallengeType,
  completions: number
): number {
  let effective = 0
  switch (type) {
    case 'transcend':
      effective += Math.min(100, completions)
      effective += 1 / 20 * (Math.min(1000, Math.max(100, completions)) - 100)
      effective += 1 / 100 * (Math.max(1000, completions) - 1000)
      return effective
    case 'reincarnation':
      effective += Math.min(25, completions)
      effective += 1 / 2 * (Math.min(75, Math.max(25, completions)) - 25)
      effective += 1 / 10 * (Math.max(75, completions) - 75)
      return effective
    case 'ascension':
      effective += Math.min(10, completions)
      effective += 1 / 2 * (Math.max(10, completions) - 10)
      return effective
    default: {
      throw new Error(`Unhandled challenge type: ${type satisfies never}`)
    }
  }
}

// ─── Challenge-15 score-tier arrays ────────────────────────────────────────
// Per-completion ascension-score weights, keyed by `[challengeIndex]` (1-10).
// Each row is the rate inside one completion band; the bands are:
//   transcend:     1..74, 75..749, 750..8999, 9000+
//   reincarnation: 1..24, 25..59,  60+
// (Banding lives in challengeScoreDisplay / calculateAscensionScore below.)
const challengeScoreArray1 = [0, 8, 10, 12, 15, 20, 60, 80, 120, 180, 300]
const challengeScoreArray2 = [0, 10, 12, 15, 20, 30, 80, 120, 180, 300, 450]
const challengeScoreArray3 = [0, 20, 30, 50, 100, 200, 250, 300, 400, 500, 750]
const challengeScoreArray4 = [0, 10000, 10000, 10000, 10000, 10000, 2000, 3000, 4000, 5000, 7500]

/**
 * Per-completion score weight shown in the challenge UI ("each future
 * completion is worth X score"). Matches the banding used inside
 * calculateAscensionScore. Returns 0 for challenges outside 1..10.
 */
export function challengeScoreDisplay(
  challenge: number,
  highestCompletions: number
): number {
  if (challenge >= 1 && challenge <= 5) {
    if (highestCompletions >= 9000) return challengeScoreArray4[challenge]
    if (highestCompletions >= 750) return challengeScoreArray3[challenge]
    if (highestCompletions >= 75) return challengeScoreArray2[challenge]
    return challengeScoreArray1[challenge]
  }
  if (challenge >= 6 && challenge <= 10) {
    if (highestCompletions >= 60) return challengeScoreArray3[challenge]
    if (highestCompletions >= 25) return challengeScoreArray2[challenge]
    return challengeScoreArray1[challenge]
  }
  return 0
}

// ─── getMaxChallenges ──────────────────────────────────────────────────────

export interface GetMaxChallengesInput {
  /** 1..15. Out-of-range and 15 return 0. */
  challenge: number
  /** player.singularityChallenges.oneChallengeCap.enabled — caps every tier to 1. */
  oneChallengeCapEnabled: boolean

  // ── Transcension tier (1..5)
  /** player.researches[105] — "Infinite T. Challenges" research; returns 9001 if > 0. */
  infiniteTranscendResearch: number
  /** player.researches[65 + challenge] for the matching T. challenge slot (3x16..3x20). */
  transcendResearchForChallenge: number

  // ── Reincarnation tier (6..10)
  /** player.cubeUpgrades[29] — +4/level. */
  cubeUpgrade29: number
  /** getShopUpgradeEffects('challengeExtension', 'reincarnationChallengeCap'). */
  challengeExtensionCap: number
  /** Sum of GQ singChallengeExtension/2/3 'reincarnationCapIncrease'. */
  gqReincarnationCapIncrease: number
  /** Sum of singularity-challenge oneChallengeCap capIncrease + reinCapIncrease2. */
  singReincarnationCapIncrease: number

  // ── Ascension tier (11..14; 15 has no completions)
  /** Sum of GQ singChallengeExtension/2/3 'ascensionCapIncrease'. */
  gqAscensionCapIncrease: number
  /** singularity-challenge oneChallengeCap 'ascCapIncrease2'. */
  singAscensionCapIncrease: number

  // ── Shared platonic flags (apply to both reinc and ascension tiers)
  /** player.platonicUpgrades[5] > 0 — ALPHA. Reinc: +10, Asc: +5. */
  platonicUpgrade5: number
  /** player.platonicUpgrades[10] > 0 — BETA. Reinc: +10, Asc: +5. */
  platonicUpgrade10: number
  /** player.platonicUpgrades[15] > 0 — OMEGA. Reinc: +30, Asc: +20. */
  platonicUpgrade15: number
}

/**
 * Max completions for a given challenge, given the constellation of unlocks
 * that can extend the cap. Mirrors the web_ui body verbatim — every branch is
 * either pure arithmetic on these inputs or one of the early-return sentinels
 * (`oneChallengeCap → 1`, `research105 > 0 → 9001`).
 *
 * Challenge 15 has no completions — returns 0 even if `oneChallengeCap` is set
 * (the original short-circuits the same way: the `i === 15` branch comes
 * before the cap check inside the asc tier).
 */
export function getMaxChallenges(input: GetMaxChallengesInput): number {
  const i = input.challenge
  let maxChallenge = 0

  if (i >= 1 && i <= 5) {
    if (input.oneChallengeCapEnabled) return 1
    maxChallenge = 25
    if (input.infiniteTranscendResearch > 0) return 9001
    maxChallenge += 5 * input.transcendResearchForChallenge
    return maxChallenge
  }

  if (i >= 6 && i <= 10) {
    if (input.oneChallengeCapEnabled) return 1
    maxChallenge = 40
    maxChallenge += 4 * input.cubeUpgrade29
    maxChallenge += input.challengeExtensionCap
    if (input.platonicUpgrade5 > 0) maxChallenge += 10
    if (input.platonicUpgrade10 > 0) maxChallenge += 10
    if (input.platonicUpgrade15 > 0) maxChallenge += 30
    maxChallenge += input.gqReincarnationCapIncrease
    maxChallenge += input.singReincarnationCapIncrease
    return maxChallenge
  }

  if (i >= 11 && i <= 15) {
    if (i === 15) return 0
    if (input.oneChallengeCapEnabled) return 1
    maxChallenge = 30
    if (input.platonicUpgrade5 > 0) maxChallenge += 5
    if (input.platonicUpgrade10 > 0) maxChallenge += 5
    if (input.platonicUpgrade15 > 0) maxChallenge += 20
    maxChallenge += input.gqAscensionCapIncrease
    maxChallenge += input.singAscensionCapIncrease
    return maxChallenge
  }

  return maxChallenge
}

// ─── Challenge requirement (target value to beat the challenge) ────────────

export interface ChallengeRequirementMultiplierInput {
  type: ChallengeType
  completions: number
  /**
   * For reincarnation challenges: which challenge "special" multipliers apply
   * (6/7/8/9/10 each scale differently past the 60/70/80/90 thresholds).
   * For ascension: 15 selects the Decimal.pow(1000, completions) branch.
   * Transcend ignores it. 0 means "no special".
   */
  special: number
  /**
   * G.hyperchallengeMultiplier[player.corruptions.used.hyperchallenge] —
   * baseline corruption-driven scaling. Transcend/reincarnation only;
   * ascension forces this to 1 internally.
   */
  hyperchallengeMultiplier: number
  /** player.platonicUpgrades[8] — divides the corruption baseline by 1 + n/2.5. */
  platonicUpgrade8: number
  /** G.challenge15Rewards.transcendChallengeReduction.value (defaults 1). */
  challenge15TranscendReduction: number
  /** G.challenge15Rewards.reincarnationChallengeReduction.value (defaults 1). */
  challenge15ReincarnationReduction: number
  /** getShopUpgradeEffects('challengeTome', 'c9c10ScalingReduction'). */
  challengeTomeC9C10ScalingReduction: number
  /** getShopUpgradeEffects('challengeTome2', 'c9c10ScalingReduction'). */
  challengeTome2C9C10ScalingReduction: number
}

/**
 * Multiplier on the base challenge requirement. The transcend and
 * reincarnation branches are piles of `if completions >= K` softcap stages;
 * ascension just scales linearly past 10 completions (or geometrically for
 * c15). Identical to web_ui's calculateChallengeRequirementMultiplier.
 */
export function calculateChallengeRequirementMultiplier(
  input: ChallengeRequirementMultiplierInput
): number {
  const { type, completions, special } = input

  let requirementMultiplier = Math.max(
    1,
    input.hyperchallengeMultiplier / (1 + input.platonicUpgrade8 / 2.5)
  )
  if (type === 'ascension') {
    // Normalize back to 1 for ascension; the corruption baseline only applies
    // to T/R tiers.
    requirementMultiplier = 1
  }

  switch (type) {
    case 'transcend':
      requirementMultiplier *= input.challenge15TranscendReduction
      if (completions >= 75) {
        requirementMultiplier *= Math.pow(1 + completions, 12) / Math.pow(75, 8)
      } else {
        requirementMultiplier *= Math.pow(1 + completions, 2)
      }
      if (completions >= 1000) {
        requirementMultiplier *= 10 * Math.pow(completions / 1000, 3)
      }
      if (completions >= 9000) {
        requirementMultiplier *= 1337
      }
      if (completions >= 9001) {
        requirementMultiplier *= completions - 8999
      }
      return requirementMultiplier

    case 'reincarnation':
      if (completions >= 100 && (special === 9 || special === 10)) {
        requirementMultiplier *= Math.pow(1.05, (completions - 100) * (1 + (completions - 100) / 20))
      }
      if (completions >= 90) {
        if (special === 6) requirementMultiplier *= 100
        else if (special === 7) requirementMultiplier *= 50
        else if (special === 8) requirementMultiplier *= 10
        else requirementMultiplier *= 4
      }
      if (completions >= 80) {
        if (special === 6) requirementMultiplier *= 50
        else if (special === 7) requirementMultiplier *= 20
        else if (special === 8) requirementMultiplier *= 4
        else requirementMultiplier *= 2
      }
      if (completions >= 70) {
        if (special === 6) requirementMultiplier *= 20
        else if (special === 7) requirementMultiplier *= 10
        else if (special === 8) requirementMultiplier *= 2
        else requirementMultiplier *= 1
      }
      if (completions >= 60 && (special === 9 || special === 10)) {
        requirementMultiplier *= Math.pow(
          1000,
          (completions - 60)
            * (1 + input.challengeTomeC9C10ScalingReduction + input.challengeTome2C9C10ScalingReduction)
            / 10
        )
      }
      if (completions >= 25) {
        requirementMultiplier *= Math.pow(1 + completions, 5) / 625
      }
      if (completions < 25) {
        requirementMultiplier *= Math.min(Math.pow(1 + completions, 2), Math.pow(1.3797, completions))
      }
      requirementMultiplier *= input.challenge15ReincarnationReduction
      return requirementMultiplier

    case 'ascension':
      if (special !== 15) {
        if (completions >= 10) {
          requirementMultiplier *= 2 * (1 + completions) - 10
        } else {
          requirementMultiplier *= 1 + completions
        }
      } else {
        requirementMultiplier *= Math.pow(1000, completions)
      }
      return requirementMultiplier

    default: {
      throw new Error(`Unhandled challenge type: ${type satisfies never}`)
    }
  }
}

export interface ChallengeRequirementInput {
  challenge: number
  completion: number
  /** See ChallengeRequirementMultiplierInput.special. */
  special: number
  /** G.challengeBaseRequirements[challenge - 1]. */
  challengeBaseRequirement: number
  /**
   * Subtracted from the base for challenge 10 only:
   *   1e8 * (researches[140] + 155 + 170 + 185)
   *   + challengeTome 'c10RequirementReduction'
   *   + challengeTome2 'c10RequirementReduction'
   * Pass 0 for any other challenge.
   */
  c10RequirementReduction: number
  hyperchallengeMultiplier: number
  platonicUpgrade8: number
  challenge15TranscendReduction: number
  challenge15ReincarnationReduction: number
  challengeTomeC9C10ScalingReduction: number
  challengeTome2C9C10ScalingReduction: number
}

/**
 * Target value to beat the challenge. T/R: 10^(base * multiplier). Ascension
 * 11..14: just the multiplier. Ascension 15: 10^(1e30 * multiplier). Challenges
 * outside 1..15 return 0.
 *
 * Returns Decimal for T/R/15 (can exceed JS number range) and number for
 * 11..14.
 */
export function challengeRequirement(input: ChallengeRequirementInput): Decimal | number {
  const { challenge, completion, special } = input
  const multInput: ChallengeRequirementMultiplierInput = {
    type: 'transcend',
    completions: completion,
    special,
    hyperchallengeMultiplier: input.hyperchallengeMultiplier,
    platonicUpgrade8: input.platonicUpgrade8,
    challenge15TranscendReduction: input.challenge15TranscendReduction,
    challenge15ReincarnationReduction: input.challenge15ReincarnationReduction,
    challengeTomeC9C10ScalingReduction: input.challengeTomeC9C10ScalingReduction,
    challengeTome2C9C10ScalingReduction: input.challengeTome2C9C10ScalingReduction
  }

  if (challenge >= 1 && challenge <= 5) {
    multInput.type = 'transcend'
    return Decimal.pow(10, input.challengeBaseRequirement * calculateChallengeRequirementMultiplier(multInput))
  }
  if (challenge >= 6 && challenge <= 10) {
    multInput.type = 'reincarnation'
    return Decimal.pow(
      10,
      (input.challengeBaseRequirement - input.c10RequirementReduction)
        * calculateChallengeRequirementMultiplier(multInput)
    )
  }
  if (challenge >= 11 && challenge <= 14) {
    multInput.type = 'ascension'
    return calculateChallengeRequirementMultiplier(multInput)
  }
  if (challenge === 15) {
    multInput.type = 'ascension'
    return Decimal.pow(
      10,
      1 * Math.pow(10, 30) * calculateChallengeRequirementMultiplier(multInput)
    )
  }
  return 0
}

// ─── Challenge 15 score multiplier ─────────────────────────────────────────

export interface Challenge15ScoreMultiplierInput {
  /** player.campaigns.c15Bonus. */
  c15Bonus: number
  /** hepteractEffective('challenge') — the challenge-hepteract effective count. */
  challengeHepteractEffective: number
  /** player.platonicUpgrades[15]. */
  platonicUpgrade15: number
}

/**
 * Score multiplier for the C15 ascension challenge. Three independent legs
 * multiplied together: campaign bonus, challenge-hepteract scaling, and
 * platonic OMEGA bonus.
 */
export function challenge15ScoreMultiplier(input: Challenge15ScoreMultiplierInput): number {
  return input.c15Bonus
    * (1 + 5 / 10000 * input.challengeHepteractEffective)
    * (1 + 0.25 * input.platonicUpgrade15)
}

// ─── Auto-challenge sweep traversal helpers ────────────────────────────────

const NUM_ELIGIBLE_CHALLENGES = 10

export interface GetNextRegularChallengeInput {
  /** Where to start the scan; 1..10. */
  startIndex: number
  /** Already-attempted challenges this sweep round. */
  explored: ReadonlySet<number>
  /**
   * Indexed by challenge number 1..10. The caller precomputes one slot per
   * challenge — the same getMaxChallenges shape, evaluated for each tier.
   */
  maxChallenges: readonly number[]
  /** player.highestchallengecompletions, indexed by challenge number. */
  highestCompletions: readonly number[]
  /** player.autoChallengeToggles, indexed by challenge number. */
  autoChallengeToggles: readonly boolean[]
}

/**
 * "Next eligible normal (non-asc) challenge" — wraps around 10 → 1.
 * Returns -1 if no challenge in 1..10 is eligible (toggled on AND under cap
 * AND not already explored this round).
 */
export function getNextRegularChallenge(input: GetNextRegularChallengeInput): number {
  let challenge = input.startIndex
  for (let i = 0; i < NUM_ELIGIBLE_CHALLENGES; i++) {
    if (
      !input.explored.has(challenge)
      && input.highestCompletions[challenge] < input.maxChallenges[challenge]
      && input.autoChallengeToggles[challenge]
    ) {
      return challenge
    }
    challenge++
    if (challenge > NUM_ELIGIBLE_CHALLENGES) {
      challenge = 1
    }
  }
  return -1
}

export interface GetNextAscensionChallengeInput {
  /** Where to start the scan; 11..15. */
  startIndex: number
  /** Indexed by challenge number 11..15; maxChallenges[15] is ignored. */
  maxChallenges: readonly number[]
  /** player.highestchallengecompletions. */
  highestCompletions: readonly number[]
  /** player.autoChallengeToggles. */
  autoChallengeToggles: readonly boolean[]
}

/**
 * "Next eligible ascension challenge" — wraps 15 → 11. Returns the same value
 * as startIndex if nothing else is eligible (mirrors the web_ui contract —
 * no `explored` set is threaded through, only one wrap is attempted).
 *
 * c15 is treated as always-eligible since it has no completions cap.
 */
export function getNextAscensionChallenge(input: GetNextAscensionChallengeInput): number {
  let nextChallenge = input.startIndex
  for (let i = 0; i < 5; i++) {
    nextChallenge++
    if (nextChallenge > 15) {
      nextChallenge = 11
    }
    if (
      input.autoChallengeToggles[nextChallenge]
      && (input.highestCompletions[nextChallenge] < input.maxChallenges[nextChallenge] || nextChallenge === 15)
    ) {
      return nextChallenge
    }
  }
  return nextChallenge
}

// ─── Misc unlock checks ────────────────────────────────────────────────────

/**
 * Ascension-challenge auto-sweep is gated behind singularity 101 + the
 * instantChallenge2 shop upgrade. The web_ui caller passes both bits in.
 */
export function autoAscensionChallengeSweepUnlock(
  highestSingularityCount: number,
  instantChallenge2Unlocked: boolean
): boolean {
  return highestSingularityCount >= 101 && instantChallenge2Unlocked
}
