//! Challenge math helpers.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/challenges.ts`.
//! Covers effective-completion math, per-tier max-challenge caps, the
//! challenge requirement / target-value formulas, the C15 score
//! multiplier, and the auto-sweep traversal helpers.

use synergismforkd_bignum::Decimal;

/// `G.challengeBaseRequirements` (`Variables.ts:121`) — per-challenge base
/// log10-coin requirement exponent, indexed `challenge - 1` (c1-10). Feeds
/// [`challenge_requirement`] (the completion goal) and the in-tick
/// auto-recompleter thresholds.
pub const CHALLENGE_BASE_REQUIREMENTS: [f64; 10] = [
    10.0, 20.0, 60.0, 100.0, 200.0, 125.0, 500.0, 7_500.0, 2.0e8, 2.5e9,
];

/// Which challenge tier the effective-completion curve applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChallengeType {
    /// Transcension challenges (1..5).
    Transcend,
    /// Reincarnation challenges (6..10).
    Reincarnation,
    /// Ascension challenges (11..15).
    Ascension,
}

/// Effective Challenge Completions. Three piecewise linear curves keyed by
/// challenge tier, each with diminishing returns past the first knee:
///
/// - **transcend**: `[0..100]` 1×, `[100..1000]` 0.05×, past 1000 0.01×
/// - **reincarnation**: `[0..25]` 1×, `[25..75]` 0.5×, past 75 0.1×
/// - **ascension**: `[0..10]` 1×, past 10 0.5×
///
/// Verbatim port of `CalcECC` from `mechanics/challenges.ts`. Pure: depends
/// only on its two arguments.
#[must_use]
pub fn calc_ecc(challenge_type: ChallengeType, completions: f64) -> f64 {
    let mut effective = 0.0;
    match challenge_type {
        ChallengeType::Transcend => {
            effective += completions.min(100.0);
            effective += 1.0 / 20.0 * (completions.clamp(100.0, 1000.0) - 100.0);
            effective += 1.0 / 100.0 * (completions.max(1000.0) - 1000.0);
        }
        ChallengeType::Reincarnation => {
            effective += completions.min(25.0);
            effective += 1.0 / 2.0 * (completions.clamp(25.0, 75.0) - 25.0);
            effective += 1.0 / 10.0 * (completions.max(75.0) - 75.0);
        }
        ChallengeType::Ascension => {
            effective += completions.min(10.0);
            effective += 1.0 / 2.0 * (completions.max(10.0) - 10.0);
        }
    }
    effective
}

// ─── Challenge-15 score-tier arrays ────────────────────────────────────────

/// Per-completion ascension-score weights, keyed by `[challenge_index]`
/// (1..=10). Each row is the rate inside one completion band; the bands
/// are:
/// - **transcend**: 1..74, 75..749, 750..8999, 9000+
/// - **reincarnation**: 1..24, 25..59, 60+
///
/// (Banding lives in [`challenge_score_display`] / the future
/// `calculateAscensionScore` port.)
const CHALLENGE_SCORE_ARRAY_1: [u32; 11] = [0, 8, 10, 12, 15, 20, 60, 80, 120, 180, 300];
const CHALLENGE_SCORE_ARRAY_2: [u32; 11] = [0, 10, 12, 15, 20, 30, 80, 120, 180, 300, 450];
const CHALLENGE_SCORE_ARRAY_3: [u32; 11] = [0, 20, 30, 50, 100, 200, 250, 300, 400, 500, 750];
const CHALLENGE_SCORE_ARRAY_4: [u32; 11] = [
    0, 10_000, 10_000, 10_000, 10_000, 10_000, 2_000, 3_000, 4_000, 5_000, 7_500,
];

/// Per-completion score weight shown in the challenge UI ("each future
/// completion is worth `X` score"). Matches the banding used inside the
/// `calculateAscensionScore` formula. Returns `0` for challenges outside
/// `1..=10`.
#[must_use]
pub fn challenge_score_display(challenge: u8, highest_completions: f64) -> f64 {
    let i = usize::from(challenge);
    if (1..=5).contains(&challenge) {
        if highest_completions >= 9_000.0 {
            return f64::from(CHALLENGE_SCORE_ARRAY_4[i]);
        }
        if highest_completions >= 750.0 {
            return f64::from(CHALLENGE_SCORE_ARRAY_3[i]);
        }
        if highest_completions >= 75.0 {
            return f64::from(CHALLENGE_SCORE_ARRAY_2[i]);
        }
        return f64::from(CHALLENGE_SCORE_ARRAY_1[i]);
    }
    if (6..=10).contains(&challenge) {
        if highest_completions >= 60.0 {
            return f64::from(CHALLENGE_SCORE_ARRAY_3[i]);
        }
        if highest_completions >= 25.0 {
            return f64::from(CHALLENGE_SCORE_ARRAY_2[i]);
        }
        return f64::from(CHALLENGE_SCORE_ARRAY_1[i]);
    }
    0.0
}

// ─── getMaxChallenges ──────────────────────────────────────────────────────

/// Inputs to [`get_max_challenges`].
#[derive(Debug, Clone, Copy)]
pub struct GetMaxChallengesInput {
    /// `1..=15`. Out-of-range and `15` return `0`.
    pub challenge: u8,
    /// `player.singularityChallenges.oneChallengeCap.enabled` — caps
    /// every tier to `1`.
    pub one_challenge_cap_enabled: bool,

    // ── Transcension tier (1..=5)
    /// `player.researches[105]` — "Infinite T. Challenges" research;
    /// returns `9001` when `> 0`.
    pub infinite_transcend_research: f64,
    /// `player.researches[65 + challenge]` for the matching transcension
    /// challenge slot (3x16..3x20).
    pub transcend_research_for_challenge: f64,

    // ── Reincarnation tier (6..=10)
    /// `player.cubeUpgrades[29]` — `+4/level`.
    pub cube_upgrade_29: f64,
    /// `getShopUpgradeEffects('challengeExtension', 'reincarnationChallengeCap')`.
    pub challenge_extension_cap: f64,
    /// Sum of GQ `singChallengeExtension`/2/3 `reincarnationCapIncrease`.
    pub gq_reincarnation_cap_increase: f64,
    /// Sum of singularity-challenge `oneChallengeCap` `capIncrease` +
    /// `reinCapIncrease2`.
    pub sing_reincarnation_cap_increase: f64,

    // ── Ascension tier (11..=14; 15 has no completions)
    /// Sum of GQ `singChallengeExtension`/2/3 `ascensionCapIncrease`.
    pub gq_ascension_cap_increase: f64,
    /// Singularity-challenge `oneChallengeCap` `ascCapIncrease2`.
    pub sing_ascension_cap_increase: f64,

    // ── Shared platonic flags (apply to both reinc and ascension tiers)
    /// `player.platonicUpgrades[5]` > 0 — ALPHA. Reinc: +10, Asc: +5.
    pub platonic_upgrade_5: f64,
    /// `player.platonicUpgrades[10]` > 0 — BETA. Reinc: +10, Asc: +5.
    pub platonic_upgrade_10: f64,
    /// `player.platonicUpgrades[15]` > 0 — OMEGA. Reinc: +30, Asc: +20.
    pub platonic_upgrade_15: f64,
}

/// Max completions for a given challenge, given the constellation of
/// unlocks that can extend the cap. Mirrors the legacy body verbatim —
/// every branch is either pure arithmetic on these inputs or one of the
/// early-return sentinels (`one_challenge_cap_enabled → 1`,
/// `infinite_transcend_research > 0 → 9001`).
///
/// Challenge 15 has no completions — returns `0` even if
/// `one_challenge_cap_enabled` is set (the original short-circuits the
/// same way: the `i === 15` branch comes before the cap check inside
/// the asc tier).
#[must_use]
pub fn get_max_challenges(input: &GetMaxChallengesInput) -> f64 {
    let i = input.challenge;
    let mut max_challenge = 0.0_f64;

    if (1..=5).contains(&i) {
        if input.one_challenge_cap_enabled {
            return 1.0;
        }
        max_challenge = 25.0;
        if input.infinite_transcend_research > 0.0 {
            return 9_001.0;
        }
        max_challenge += 5.0 * input.transcend_research_for_challenge;
        return max_challenge;
    }

    if (6..=10).contains(&i) {
        if input.one_challenge_cap_enabled {
            return 1.0;
        }
        max_challenge = 40.0;
        max_challenge += 4.0 * input.cube_upgrade_29;
        max_challenge += input.challenge_extension_cap;
        if input.platonic_upgrade_5 > 0.0 {
            max_challenge += 10.0;
        }
        if input.platonic_upgrade_10 > 0.0 {
            max_challenge += 10.0;
        }
        if input.platonic_upgrade_15 > 0.0 {
            max_challenge += 30.0;
        }
        max_challenge += input.gq_reincarnation_cap_increase;
        max_challenge += input.sing_reincarnation_cap_increase;
        return max_challenge;
    }

    if (11..=15).contains(&i) {
        if i == 15 {
            return 0.0;
        }
        if input.one_challenge_cap_enabled {
            return 1.0;
        }
        max_challenge = 30.0;
        if input.platonic_upgrade_5 > 0.0 {
            max_challenge += 5.0;
        }
        if input.platonic_upgrade_10 > 0.0 {
            max_challenge += 5.0;
        }
        if input.platonic_upgrade_15 > 0.0 {
            max_challenge += 20.0;
        }
        max_challenge += input.gq_ascension_cap_increase;
        max_challenge += input.sing_ascension_cap_increase;
        return max_challenge;
    }

    max_challenge
}

// ─── Challenge requirement multiplier ──────────────────────────────────────

/// Inputs to [`calculate_challenge_requirement_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct ChallengeRequirementMultiplierInput {
    /// Tier of the challenge being evaluated.
    pub challenge_type: ChallengeType,
    /// Completions of the challenge.
    pub completions: f64,
    /// For reincarnation challenges: which "special" multiplier set
    /// applies (`6/7/8/9/10` each scale differently past the
    /// `60/70/80/90` thresholds). For ascension: `15` selects the
    /// `10^(1000^completions)` branch. Transcend ignores it. `0` means
    /// "no special".
    pub special: u8,
    /// `G.hyperchallengeMultiplier[player.corruptions.used.hyperchallenge]`
    /// — baseline corruption-driven scaling. Transcend/reincarnation
    /// only; ascension forces this to `1` internally.
    pub hyperchallenge_multiplier: f64,
    /// `player.platonicUpgrades[8]` — divides the corruption baseline
    /// by `1 + n / 2.5`.
    pub platonic_upgrade_8: f64,
    /// `G.challenge15Rewards.transcendChallengeReduction.value`
    /// (defaults to `1`).
    pub challenge_15_transcend_reduction: f64,
    /// `G.challenge15Rewards.reincarnationChallengeReduction.value`
    /// (defaults to `1`).
    pub challenge_15_reincarnation_reduction: f64,
    /// `getShopUpgradeEffects('challengeTome', 'c9c10ScalingReduction')`.
    pub challenge_tome_c9c10_scaling_reduction: f64,
    /// `getShopUpgradeEffects('challengeTome2', 'c9c10ScalingReduction')`.
    pub challenge_tome_2_c9c10_scaling_reduction: f64,
}

/// Multiplier on the base challenge requirement. The transcend and
/// reincarnation branches are piles of `if completions >= K` softcap
/// stages; ascension just scales linearly past 10 completions (or
/// geometrically for `c15`). Identical to the legacy
/// `calculateChallengeRequirementMultiplier`.
#[must_use]
pub fn calculate_challenge_requirement_multiplier(
    input: &ChallengeRequirementMultiplierInput,
) -> f64 {
    let completions = input.completions;
    let special = input.special;

    let mut requirement_multiplier =
        1.0_f64.max(input.hyperchallenge_multiplier / (1.0 + input.platonic_upgrade_8 / 2.5));
    if input.challenge_type == ChallengeType::Ascension {
        // Normalize back to 1 for ascension; the corruption baseline
        // only applies to T/R tiers.
        requirement_multiplier = 1.0;
    }

    match input.challenge_type {
        ChallengeType::Transcend => {
            requirement_multiplier *= input.challenge_15_transcend_reduction;
            if completions >= 75.0 {
                requirement_multiplier *= (1.0 + completions).powi(12) / 75.0_f64.powi(8);
            } else {
                requirement_multiplier *= (1.0 + completions).powi(2);
            }
            if completions >= 1_000.0 {
                requirement_multiplier *= 10.0 * (completions / 1_000.0).powi(3);
            }
            if completions >= 9_000.0 {
                requirement_multiplier *= 1_337.0;
            }
            if completions >= 9_001.0 {
                requirement_multiplier *= completions - 8_999.0;
            }
            requirement_multiplier
        }

        ChallengeType::Reincarnation => {
            if completions >= 100.0 && (special == 9 || special == 10) {
                requirement_multiplier *=
                    1.05_f64.powf((completions - 100.0) * (1.0 + (completions - 100.0) / 20.0));
            }
            if completions >= 90.0 {
                requirement_multiplier *= match special {
                    6 => 100.0,
                    7 => 50.0,
                    8 => 10.0,
                    _ => 4.0,
                };
            }
            if completions >= 80.0 {
                requirement_multiplier *= match special {
                    6 => 50.0,
                    7 => 20.0,
                    8 => 4.0,
                    _ => 2.0,
                };
            }
            if completions >= 70.0 {
                requirement_multiplier *= match special {
                    6 => 20.0,
                    7 => 10.0,
                    8 => 2.0,
                    _ => 1.0,
                };
            }
            if completions >= 60.0 && (special == 9 || special == 10) {
                requirement_multiplier *= 1_000.0_f64.powf(
                    (completions - 60.0)
                        * (1.0
                            + input.challenge_tome_c9c10_scaling_reduction
                            + input.challenge_tome_2_c9c10_scaling_reduction)
                        / 10.0,
                );
            }
            if completions >= 25.0 {
                requirement_multiplier *= (1.0 + completions).powi(5) / 625.0;
            }
            if completions < 25.0 {
                requirement_multiplier *= (1.0 + completions)
                    .powi(2)
                    .min(1.3797_f64.powf(completions));
            }
            requirement_multiplier *= input.challenge_15_reincarnation_reduction;
            requirement_multiplier
        }

        ChallengeType::Ascension => {
            if special != 15 {
                if completions >= 10.0 {
                    requirement_multiplier *= 2.0 * (1.0 + completions) - 10.0;
                } else {
                    requirement_multiplier *= 1.0 + completions;
                }
            } else {
                requirement_multiplier *= 1_000.0_f64.powf(completions);
            }
            requirement_multiplier
        }
    }
}

// ─── Challenge requirement (target value to beat the challenge) ────────────

/// Inputs to [`challenge_requirement`].
#[derive(Debug, Clone, Copy)]
pub struct ChallengeRequirementInput {
    /// `1..=15`.
    pub challenge: u8,
    /// Completion count of this challenge.
    pub completion: f64,
    /// See [`ChallengeRequirementMultiplierInput::special`].
    pub special: u8,
    /// `G.challengeBaseRequirements[challenge - 1]`.
    pub challenge_base_requirement: f64,
    /// Subtracted from the base for challenge 10 only:
    ///
    /// ```text
    /// 1e8 * (researches[140] + 155 + 170 + 185)
    ///   + challengeTome  'c10RequirementReduction'
    ///   + challengeTome2 'c10RequirementReduction'
    /// ```
    ///
    /// Pass `0` for any other challenge.
    pub c10_requirement_reduction: f64,
    /// See [`ChallengeRequirementMultiplierInput`].
    pub hyperchallenge_multiplier: f64,
    /// See [`ChallengeRequirementMultiplierInput`].
    pub platonic_upgrade_8: f64,
    /// See [`ChallengeRequirementMultiplierInput`].
    pub challenge_15_transcend_reduction: f64,
    /// See [`ChallengeRequirementMultiplierInput`].
    pub challenge_15_reincarnation_reduction: f64,
    /// See [`ChallengeRequirementMultiplierInput`].
    pub challenge_tome_c9c10_scaling_reduction: f64,
    /// See [`ChallengeRequirementMultiplierInput`].
    pub challenge_tome_2_c9c10_scaling_reduction: f64,
}

/// Target value to beat the challenge.
/// - **T/R (1..=10)**: `10^(base * multiplier)`.
/// - **Ascension 11..=14**: just the multiplier (wrapped in `Decimal`).
/// - **Ascension 15**: `10^(1e30 * multiplier)`.
/// - **Out-of-range**: `0`.
///
/// The TS source returned `Decimal | number`; the Rust port collapses
/// to `Decimal` (the 11..=14 multiplier comfortably fits, and a single
/// return type simplifies callers).
#[must_use]
pub fn challenge_requirement(input: &ChallengeRequirementInput) -> Decimal {
    let challenge = input.challenge;
    let mut mult_input = ChallengeRequirementMultiplierInput {
        challenge_type: ChallengeType::Transcend,
        completions: input.completion,
        special: input.special,
        hyperchallenge_multiplier: input.hyperchallenge_multiplier,
        platonic_upgrade_8: input.platonic_upgrade_8,
        challenge_15_transcend_reduction: input.challenge_15_transcend_reduction,
        challenge_15_reincarnation_reduction: input.challenge_15_reincarnation_reduction,
        challenge_tome_c9c10_scaling_reduction: input.challenge_tome_c9c10_scaling_reduction,
        challenge_tome_2_c9c10_scaling_reduction: input.challenge_tome_2_c9c10_scaling_reduction,
    };

    if (1..=5).contains(&challenge) {
        mult_input.challenge_type = ChallengeType::Transcend;
        return Decimal::from_finite(10.0).pow(Decimal::from_finite(
            input.challenge_base_requirement
                * calculate_challenge_requirement_multiplier(&mult_input),
        ));
    }
    if (6..=10).contains(&challenge) {
        mult_input.challenge_type = ChallengeType::Reincarnation;
        return Decimal::from_finite(10.0).pow(Decimal::from_finite(
            (input.challenge_base_requirement - input.c10_requirement_reduction)
                * calculate_challenge_requirement_multiplier(&mult_input),
        ));
    }
    if (11..=14).contains(&challenge) {
        mult_input.challenge_type = ChallengeType::Ascension;
        return Decimal::from_finite(calculate_challenge_requirement_multiplier(&mult_input));
    }
    if challenge == 15 {
        mult_input.challenge_type = ChallengeType::Ascension;
        return Decimal::from_finite(10.0).pow(Decimal::from_finite(
            10.0_f64.powi(30) * calculate_challenge_requirement_multiplier(&mult_input),
        ));
    }
    Decimal::zero()
}

// ─── Challenge 15 score multiplier ─────────────────────────────────────────

/// Inputs to [`challenge_15_score_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct Challenge15ScoreMultiplierInput {
    /// `player.campaigns.c15Bonus`.
    pub c15_bonus: f64,
    /// `hepteractEffective('challenge')` — the challenge-hepteract
    /// effective count.
    pub challenge_hepteract_effective: f64,
    /// `player.platonicUpgrades[15]`.
    pub platonic_upgrade_15: f64,
}

/// Score multiplier for the C15 ascension challenge. Three independent
/// legs multiplied together: campaign bonus, challenge-hepteract
/// scaling, and platonic OMEGA bonus.
#[must_use]
pub fn challenge_15_score_multiplier(input: &Challenge15ScoreMultiplierInput) -> f64 {
    input.c15_bonus
        * (1.0 + 5.0 / 10_000.0 * input.challenge_hepteract_effective)
        * (1.0 + 0.25 * input.platonic_upgrade_15)
}

// ─── Auto-challenge sweep traversal helpers ────────────────────────────────

const NUM_ELIGIBLE_CHALLENGES: u8 = 10;

/// Inputs to [`get_next_regular_challenge`].
#[derive(Debug, Clone, Copy)]
pub struct GetNextRegularChallengeInput<'a> {
    /// Where to start the scan; `1..=10`.
    pub start_index: u8,
    /// Already-attempted challenges this sweep round.
    pub explored: &'a [u8],
    /// Indexed by challenge number `1..=10`. The caller precomputes one
    /// slot per challenge — the same [`get_max_challenges`] shape,
    /// evaluated for each tier.
    pub max_challenges: &'a [f64],
    /// `player.highestchallengecompletions`, indexed by challenge
    /// number.
    pub highest_completions: &'a [f64],
    /// `player.autoChallengeToggles`, indexed by challenge number.
    pub auto_challenge_toggles: &'a [bool],
}

/// "Next eligible normal (non-asc) challenge" — wraps around `10 → 1`.
/// Returns `-1` if no challenge in `1..=10` is eligible (toggled on,
/// under cap, and not already explored this round).
#[must_use]
pub fn get_next_regular_challenge(input: &GetNextRegularChallengeInput<'_>) -> i32 {
    let mut challenge = input.start_index;
    for _ in 0..NUM_ELIGIBLE_CHALLENGES {
        let idx = usize::from(challenge);
        let already_explored = input.explored.contains(&challenge);
        if !already_explored
            && input.highest_completions[idx] < input.max_challenges[idx]
            && input.auto_challenge_toggles[idx]
        {
            return i32::from(challenge);
        }
        challenge += 1;
        if challenge > NUM_ELIGIBLE_CHALLENGES {
            challenge = 1;
        }
    }
    -1
}

/// Inputs to [`get_next_ascension_challenge`].
#[derive(Debug, Clone, Copy)]
pub struct GetNextAscensionChallengeInput<'a> {
    /// Where to start the scan; `11..=15`.
    pub start_index: u8,
    /// Indexed by challenge number `11..=15`; `max_challenges[15]` is
    /// ignored.
    pub max_challenges: &'a [f64],
    /// `player.highestchallengecompletions`.
    pub highest_completions: &'a [f64],
    /// `player.autoChallengeToggles`.
    pub auto_challenge_toggles: &'a [bool],
}

/// "Next eligible ascension challenge" — wraps `15 → 11`. Returns the
/// same value as `start_index` if nothing else is eligible (mirrors the
/// legacy contract — no `explored` set is threaded through, only one
/// wrap is attempted).
///
/// C15 is treated as always-eligible since it has no completions cap.
#[must_use]
pub fn get_next_ascension_challenge(input: &GetNextAscensionChallengeInput<'_>) -> u8 {
    let mut next_challenge = input.start_index;
    for _ in 0..5 {
        next_challenge += 1;
        if next_challenge > 15 {
            next_challenge = 11;
        }
        let idx = usize::from(next_challenge);
        if input.auto_challenge_toggles[idx]
            && (input.highest_completions[idx] < input.max_challenges[idx] || next_challenge == 15)
        {
            return next_challenge;
        }
    }
    next_challenge
}

// ─── Misc unlock checks ────────────────────────────────────────────────────

/// Ascension-challenge auto-sweep is gated behind singularity 101 + the
/// `instantChallenge2` shop upgrade. The legacy caller passes both bits
/// in.
#[must_use]
pub fn auto_ascension_challenge_sweep_unlock(
    highest_singularity_count: f64,
    instant_challenge_2_unlocked: bool,
) -> bool {
    highest_singularity_count >= 101.0 && instant_challenge_2_unlocked
}

/// Pre-evaluated inputs to [`challenge_15_auto_exponent_check`].
#[derive(Debug, Clone, Copy)]
pub struct Challenge15AutoExponentCheckInput {
    /// [`auto_ascension_challenge_sweep_unlock`] result.
    pub sweep_unlocked: bool,
    /// `player.currentChallenge.ascension` — must be `15`.
    pub current_ascension_challenge: u32,
    /// `getShopUpgradeEffects('challenge15Auto', 'unlocked')` — when the
    /// shop auto already drives c15, the sweep does NOT take over.
    pub challenge_15_auto_shop_unlocked: bool,
    /// `player.autoAscend`.
    pub auto_ascend: bool,
    /// `player.cubeUpgrades[10]` — must be `> 0`.
    pub cube_upgrade_10: f64,
    /// `player.autoAscendMode === realAscensionTime`.
    pub auto_ascend_mode_is_real_time: bool,
    /// `player.ascensionCounterRealReal`.
    pub ascension_counter_real_real: f64,
    /// `player.autoAscendThreshold`.
    pub auto_ascend_threshold: f64,
}

/// The legacy `challenge15AutoExponentCheck()` guard (web_ui/Challenges.ts):
/// whether the challenge-sweep machine should pause in `c15_wait` to let the
/// real-time auto-ascend drive challenge 15, rather than restarting the sweep.
/// All seven conjuncts must hold.
#[must_use]
pub fn challenge_15_auto_exponent_check(input: &Challenge15AutoExponentCheckInput) -> bool {
    input.sweep_unlocked
        && input.current_ascension_challenge == 15
        && !input.challenge_15_auto_shop_unlocked
        && input.auto_ascend
        && input.cube_upgrade_10 > 0.0
        && input.auto_ascend_mode_is_real_time
        && input.ascension_counter_real_real >= 0.1_f64.max(input.auto_ascend_threshold - 5.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── transcend ─────────────────────────────────────────────────────────

    #[test]
    fn transcend_below_first_knee_is_linear() {
        assert_eq!(calc_ecc(ChallengeType::Transcend, 0.0), 0.0);
        assert_eq!(calc_ecc(ChallengeType::Transcend, 50.0), 50.0);
        assert_eq!(calc_ecc(ChallengeType::Transcend, 100.0), 100.0);
    }

    #[test]
    fn transcend_second_band_scales_at_one_twentieth() {
        // 100 → 100, +1/20 for each beyond.
        // At 200: 100 + (200-100)/20 = 105.
        assert_eq!(calc_ecc(ChallengeType::Transcend, 200.0), 105.0);
        // At 1000: 100 + 900/20 = 145.
        assert_eq!(calc_ecc(ChallengeType::Transcend, 1000.0), 145.0);
    }

    #[test]
    fn transcend_third_band_scales_at_one_hundredth() {
        // At 2000: 145 + (2000-1000)/100 = 145 + 10 = 155.
        assert_eq!(calc_ecc(ChallengeType::Transcend, 2000.0), 155.0);
    }

    // ─── reincarnation ─────────────────────────────────────────────────────

    #[test]
    fn reincarnation_band_scaling() {
        assert_eq!(calc_ecc(ChallengeType::Reincarnation, 0.0), 0.0);
        assert_eq!(calc_ecc(ChallengeType::Reincarnation, 25.0), 25.0);
        // 25 + (50-25)/2 = 25 + 12.5 = 37.5
        assert_eq!(calc_ecc(ChallengeType::Reincarnation, 50.0), 37.5);
        // 25 + (75-25)/2 = 25 + 25 = 50
        assert_eq!(calc_ecc(ChallengeType::Reincarnation, 75.0), 50.0);
        // 50 + (175-75)/10 = 50 + 10 = 60
        assert_eq!(calc_ecc(ChallengeType::Reincarnation, 175.0), 60.0);
    }

    // ─── ascension ─────────────────────────────────────────────────────────

    #[test]
    fn ascension_band_scaling() {
        assert_eq!(calc_ecc(ChallengeType::Ascension, 0.0), 0.0);
        assert_eq!(calc_ecc(ChallengeType::Ascension, 10.0), 10.0);
        // 10 + (20-10)/2 = 15
        assert_eq!(calc_ecc(ChallengeType::Ascension, 20.0), 15.0);
        // 10 + (110-10)/2 = 60
        assert_eq!(calc_ecc(ChallengeType::Ascension, 110.0), 60.0);
    }

    // ─── monotonicity ──────────────────────────────────────────────────────

    #[test]
    fn calc_ecc_is_monotonically_non_decreasing() {
        for ty in [
            ChallengeType::Transcend,
            ChallengeType::Reincarnation,
            ChallengeType::Ascension,
        ] {
            let mut prev = f64::NEG_INFINITY;
            for c in (0..200).map(|i| f64::from(i) * 10.0) {
                let v = calc_ecc(ty, c);
                assert!(v >= prev, "non-monotonic at {ty:?}, completions {c}");
                prev = v;
            }
        }
    }

    // ─── challenge_score_display ───────────────────────────────────────────

    #[test]
    fn challenge_score_display_out_of_range_returns_zero() {
        assert_eq!(challenge_score_display(0, 9_000.0), 0.0);
        assert_eq!(challenge_score_display(11, 9_000.0), 0.0);
        assert_eq!(challenge_score_display(20, 0.0), 0.0);
    }

    #[test]
    fn challenge_score_display_picks_correct_band() {
        // Challenge 1, completions = 0 → band 1 → 8
        assert_eq!(challenge_score_display(1, 0.0), 8.0);
        // Challenge 1, completions = 75 → band 2 → 10
        assert_eq!(challenge_score_display(1, 75.0), 10.0);
        // Challenge 1, completions = 750 → band 3 → 20
        assert_eq!(challenge_score_display(1, 750.0), 20.0);
        // Challenge 1, completions = 9000 → band 4 → 10_000
        assert_eq!(challenge_score_display(1, 9_000.0), 10_000.0);
    }

    #[test]
    fn challenge_score_display_reincarnation_band_thresholds() {
        // Challenge 6 (reinc tier), completions = 0 → band 1 → 60
        assert_eq!(challenge_score_display(6, 0.0), 60.0);
        // Challenge 6, completions = 25 → band 2 → 80
        assert_eq!(challenge_score_display(6, 25.0), 80.0);
        // Challenge 6, completions = 60 → band 3 → 250
        assert_eq!(challenge_score_display(6, 60.0), 250.0);
    }

    // ─── get_max_challenges ────────────────────────────────────────────────

    fn baseline_max_input() -> GetMaxChallengesInput {
        GetMaxChallengesInput {
            challenge: 1,
            one_challenge_cap_enabled: false,
            infinite_transcend_research: 0.0,
            transcend_research_for_challenge: 0.0,
            cube_upgrade_29: 0.0,
            challenge_extension_cap: 0.0,
            gq_reincarnation_cap_increase: 0.0,
            sing_reincarnation_cap_increase: 0.0,
            gq_ascension_cap_increase: 0.0,
            sing_ascension_cap_increase: 0.0,
            platonic_upgrade_5: 0.0,
            platonic_upgrade_10: 0.0,
            platonic_upgrade_15: 0.0,
        }
    }

    #[test]
    fn max_challenges_transcend_baseline_is_25() {
        let input = baseline_max_input();
        assert_eq!(get_max_challenges(&input), 25.0);
    }

    #[test]
    fn max_challenges_infinite_transcend_returns_9001() {
        let input = GetMaxChallengesInput {
            infinite_transcend_research: 1.0,
            ..baseline_max_input()
        };
        assert_eq!(get_max_challenges(&input), 9_001.0);
    }

    #[test]
    fn max_challenges_one_challenge_cap_returns_1() {
        for c in [1_u8, 5, 6, 10, 11, 14] {
            let input = GetMaxChallengesInput {
                challenge: c,
                one_challenge_cap_enabled: true,
                ..baseline_max_input()
            };
            assert_eq!(get_max_challenges(&input), 1.0, "challenge {c}");
        }
    }

    #[test]
    fn max_challenges_c15_always_zero() {
        let input = GetMaxChallengesInput {
            challenge: 15,
            one_challenge_cap_enabled: true,
            ..baseline_max_input()
        };
        // C15 has no completions — short-circuits before the cap check.
        assert_eq!(get_max_challenges(&input), 0.0);
    }

    #[test]
    fn max_challenges_reincarnation_includes_extension_cap() {
        let input = GetMaxChallengesInput {
            challenge: 6,
            challenge_extension_cap: 20.0,
            ..baseline_max_input()
        };
        // 40 + 20 = 60
        assert_eq!(get_max_challenges(&input), 60.0);
    }

    #[test]
    fn max_challenges_platonic_omega_adds_30_to_reinc() {
        let input = GetMaxChallengesInput {
            challenge: 6,
            platonic_upgrade_15: 1.0,
            ..baseline_max_input()
        };
        // 40 + 30 = 70
        assert_eq!(get_max_challenges(&input), 70.0);
    }

    #[test]
    fn max_challenges_ascension_baseline_is_30() {
        let input = GetMaxChallengesInput {
            challenge: 11,
            ..baseline_max_input()
        };
        assert_eq!(get_max_challenges(&input), 30.0);
    }

    // ─── calculate_challenge_requirement_multiplier ───────────────────────

    fn baseline_req_mult() -> ChallengeRequirementMultiplierInput {
        ChallengeRequirementMultiplierInput {
            challenge_type: ChallengeType::Transcend,
            completions: 0.0,
            special: 0,
            hyperchallenge_multiplier: 1.0,
            platonic_upgrade_8: 0.0,
            challenge_15_transcend_reduction: 1.0,
            challenge_15_reincarnation_reduction: 1.0,
            challenge_tome_c9c10_scaling_reduction: 0.0,
            challenge_tome_2_c9c10_scaling_reduction: 0.0,
        }
    }

    #[test]
    fn req_mult_transcend_zero_completions_is_one() {
        // 1 * 1 * (1+0)^2 = 1
        let result = calculate_challenge_requirement_multiplier(&baseline_req_mult());
        assert_eq!(result, 1.0);
    }

    #[test]
    fn req_mult_transcend_pre_softcap() {
        // completions = 5 → (1+5)^2 = 36
        let input = ChallengeRequirementMultiplierInput {
            completions: 5.0,
            ..baseline_req_mult()
        };
        assert_eq!(calculate_challenge_requirement_multiplier(&input), 36.0);
    }

    #[test]
    fn req_mult_transcend_softcap_at_75() {
        // completions = 75 → (1+75)^12 / 75^8
        let input = ChallengeRequirementMultiplierInput {
            completions: 75.0,
            ..baseline_req_mult()
        };
        let expected = 76.0_f64.powi(12) / 75.0_f64.powi(8);
        assert!(
            (calculate_challenge_requirement_multiplier(&input) - expected).abs() / expected < 1e-9
        );
    }

    #[test]
    fn req_mult_reincarnation_special_9_past_60() {
        // completions = 70, special = 9 → factor 1000^((70-60)*(1+0+0)/10) =
        // 1000^1 = 1000, plus the >= 25 softcap (1+70)^5 / 625, plus the
        // >= 70 case (special 9 → ×1).
        let input = ChallengeRequirementMultiplierInput {
            challenge_type: ChallengeType::Reincarnation,
            completions: 70.0,
            special: 9,
            ..baseline_req_mult()
        };
        let result = calculate_challenge_requirement_multiplier(&input);
        assert!(result > 0.0);
        assert!(result.is_finite());
    }

    #[test]
    fn req_mult_ascension_below_10_linear() {
        let input = ChallengeRequirementMultiplierInput {
            challenge_type: ChallengeType::Ascension,
            completions: 5.0,
            special: 11, // any non-15 value
            ..baseline_req_mult()
        };
        // hyperchallenge resets to 1 for ascension; then * (1+5) = 6
        assert_eq!(calculate_challenge_requirement_multiplier(&input), 6.0);
    }

    #[test]
    fn req_mult_ascension_above_10_linear_grows_2x() {
        let input = ChallengeRequirementMultiplierInput {
            challenge_type: ChallengeType::Ascension,
            completions: 20.0,
            special: 11,
            ..baseline_req_mult()
        };
        // 2*(1+20) - 10 = 32
        assert_eq!(calculate_challenge_requirement_multiplier(&input), 32.0);
    }

    #[test]
    fn req_mult_ascension_15_uses_geometric() {
        let input = ChallengeRequirementMultiplierInput {
            challenge_type: ChallengeType::Ascension,
            completions: 2.0,
            special: 15,
            ..baseline_req_mult()
        };
        // 1 * 1000^2 = 1_000_000
        assert!((calculate_challenge_requirement_multiplier(&input) - 1_000_000.0).abs() < 1e-6);
    }

    // ─── challenge_requirement ─────────────────────────────────────────────

    fn baseline_req() -> ChallengeRequirementInput {
        ChallengeRequirementInput {
            challenge: 1,
            completion: 0.0,
            special: 0,
            challenge_base_requirement: 5.0,
            c10_requirement_reduction: 0.0,
            hyperchallenge_multiplier: 1.0,
            platonic_upgrade_8: 0.0,
            challenge_15_transcend_reduction: 1.0,
            challenge_15_reincarnation_reduction: 1.0,
            challenge_tome_c9c10_scaling_reduction: 0.0,
            challenge_tome_2_c9c10_scaling_reduction: 0.0,
        }
    }

    #[test]
    fn requirement_transcend_returns_10_pow_base() {
        // multiplier = 1 (zero completions), base = 5 → 10^5
        let result = challenge_requirement(&baseline_req());
        assert!((result.to_number() - 100_000.0).abs() < 1e-3);
    }

    #[test]
    fn requirement_reincarnation_subtracts_c10_reduction() {
        let plain = ChallengeRequirementInput {
            challenge: 10,
            challenge_base_requirement: 10.0,
            ..baseline_req()
        };
        let reduced = ChallengeRequirementInput {
            c10_requirement_reduction: 5.0,
            ..plain
        };
        let plain_result = challenge_requirement(&plain);
        let reduced_result = challenge_requirement(&reduced);
        assert!(reduced_result < plain_result);
    }

    #[test]
    fn requirement_ascension_11_returns_multiplier_only() {
        // ascension 11-14: just the multiplier (1*(1+completions))
        let input = ChallengeRequirementInput {
            challenge: 11,
            completion: 5.0,
            ..baseline_req()
        };
        let result = challenge_requirement(&input);
        // multiplier at completions=5 → 1*(1+5) = 6
        assert_eq!(result.to_number(), 6.0);
    }

    #[test]
    fn requirement_out_of_range_returns_zero() {
        let input = ChallengeRequirementInput {
            challenge: 16,
            ..baseline_req()
        };
        assert_eq!(challenge_requirement(&input), Decimal::zero());
    }

    // ─── challenge_15_score_multiplier ─────────────────────────────────────

    #[test]
    fn challenge_15_score_multiplier_baseline() {
        let input = Challenge15ScoreMultiplierInput {
            c15_bonus: 1.0,
            challenge_hepteract_effective: 0.0,
            platonic_upgrade_15: 0.0,
        };
        assert_eq!(challenge_15_score_multiplier(&input), 1.0);
    }

    #[test]
    fn challenge_15_score_multiplier_legs_combine() {
        // 2 * (1 + 5/10000 * 1000) * (1 + 0.25*4) = 2 * 1.5 * 2 = 6
        let input = Challenge15ScoreMultiplierInput {
            c15_bonus: 2.0,
            challenge_hepteract_effective: 1_000.0,
            platonic_upgrade_15: 4.0,
        };
        assert!((challenge_15_score_multiplier(&input) - 6.0).abs() < 1e-12);
    }

    // ─── get_next_regular_challenge ────────────────────────────────────────

    #[test]
    fn next_regular_returns_start_when_eligible() {
        let max = [0.0_f64; 11];
        let mut max_filled = max;
        for v in max_filled.iter_mut().skip(1) {
            *v = 25.0;
        }
        let completions = [0.0_f64; 11];
        let toggles = [true; 11];
        let input = GetNextRegularChallengeInput {
            start_index: 3,
            explored: &[],
            max_challenges: &max_filled,
            highest_completions: &completions,
            auto_challenge_toggles: &toggles,
        };
        assert_eq!(get_next_regular_challenge(&input), 3);
    }

    #[test]
    fn next_regular_skips_explored_and_capped_and_disabled() {
        let max_challenges = [
            0.0_f64, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0,
        ];
        let mut completions = [0.0_f64; 11];
        completions[3] = 25.0; // 3 is capped
        let mut toggles = [true; 11];
        toggles[4] = false; // 4 is disabled
        let input = GetNextRegularChallengeInput {
            start_index: 2,
            explored: &[2],
            max_challenges: &max_challenges,
            highest_completions: &completions,
            auto_challenge_toggles: &toggles,
        };
        // 2 explored, 3 capped, 4 disabled → next is 5
        assert_eq!(get_next_regular_challenge(&input), 5);
    }

    #[test]
    fn next_regular_returns_neg_1_when_nothing_eligible() {
        let max_challenges = [0.0_f64; 11];
        let completions = [0.0_f64; 11];
        let toggles = [false; 11];
        let input = GetNextRegularChallengeInput {
            start_index: 1,
            explored: &[],
            max_challenges: &max_challenges,
            highest_completions: &completions,
            auto_challenge_toggles: &toggles,
        };
        assert_eq!(get_next_regular_challenge(&input), -1);
    }

    #[test]
    fn next_regular_wraps_10_to_1() {
        let max_challenges = [
            0.0_f64, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0,
        ];
        let completions = [0.0_f64; 11];
        let mut toggles = [false; 11];
        toggles[1] = true;
        let input = GetNextRegularChallengeInput {
            start_index: 9,
            explored: &[],
            max_challenges: &max_challenges,
            highest_completions: &completions,
            auto_challenge_toggles: &toggles,
        };
        // Only challenge 1 is enabled — should wrap from 9 → 10 → 1.
        assert_eq!(get_next_regular_challenge(&input), 1);
    }

    // ─── get_next_ascension_challenge ──────────────────────────────────────

    #[test]
    fn next_ascension_finds_eligible_after_start() {
        let max_challenges = vec![0.0_f64; 16];
        let mut max_filled = max_challenges;
        for v in max_filled.iter_mut().skip(11).take(4) {
            *v = 30.0;
        }
        let completions = vec![0.0_f64; 16];
        let toggles = vec![true; 16];
        let input = GetNextAscensionChallengeInput {
            start_index: 11,
            max_challenges: &max_filled,
            highest_completions: &completions,
            auto_challenge_toggles: &toggles,
        };
        // start = 11 → next checked = 12 → eligible.
        assert_eq!(get_next_ascension_challenge(&input), 12);
    }

    #[test]
    fn next_ascension_wraps_15_to_11() {
        let mut max_challenges = vec![0.0_f64; 16];
        max_challenges[11] = 30.0;
        let completions = vec![0.0_f64; 16];
        let mut toggles = vec![false; 16];
        toggles[11] = true;
        let input = GetNextAscensionChallengeInput {
            start_index: 15,
            max_challenges: &max_challenges,
            highest_completions: &completions,
            auto_challenge_toggles: &toggles,
        };
        // start = 15 → wrap → 11 → eligible.
        assert_eq!(get_next_ascension_challenge(&input), 11);
    }

    #[test]
    fn next_ascension_c15_always_eligible() {
        let max_challenges = vec![0.0_f64; 16]; // even max[15] = 0
        let completions = vec![0.0_f64; 16];
        let mut toggles = vec![false; 16];
        toggles[15] = true;
        let input = GetNextAscensionChallengeInput {
            start_index: 14,
            max_challenges: &max_challenges,
            highest_completions: &completions,
            auto_challenge_toggles: &toggles,
        };
        // start = 14 → next = 15 → toggle on → eligible despite max=0.
        assert_eq!(get_next_ascension_challenge(&input), 15);
    }

    // ─── auto_ascension_challenge_sweep_unlock ─────────────────────────────

    #[test]
    fn asc_sweep_unlock_requires_both_conditions() {
        assert!(!auto_ascension_challenge_sweep_unlock(100.0, true));
        assert!(!auto_ascension_challenge_sweep_unlock(101.0, false));
        assert!(auto_ascension_challenge_sweep_unlock(101.0, true));
        assert!(auto_ascension_challenge_sweep_unlock(500.0, true));
    }

    // ─── challenge_15_auto_exponent_check ──────────────────────────────────

    fn c15_check_all_true() -> Challenge15AutoExponentCheckInput {
        Challenge15AutoExponentCheckInput {
            sweep_unlocked: true,
            current_ascension_challenge: 15,
            challenge_15_auto_shop_unlocked: false,
            auto_ascend: true,
            cube_upgrade_10: 1.0,
            auto_ascend_mode_is_real_time: true,
            ascension_counter_real_real: 100.0,
            auto_ascend_threshold: 10.0, // max(0.1, 10−5) = 5 ≤ 100
        }
    }

    #[test]
    fn c15_auto_exponent_check_all_conjuncts_required() {
        assert!(challenge_15_auto_exponent_check(&c15_check_all_true()));
        // Each falsifier flips the result.
        let mut not_c15 = c15_check_all_true();
        not_c15.current_ascension_challenge = 14;
        assert!(!challenge_15_auto_exponent_check(&not_c15));
        let mut shop_drives = c15_check_all_true();
        shop_drives.challenge_15_auto_shop_unlocked = true;
        assert!(!challenge_15_auto_exponent_check(&shop_drives));
        let mut not_real_time = c15_check_all_true();
        not_real_time.auto_ascend_mode_is_real_time = false;
        assert!(!challenge_15_auto_exponent_check(&not_real_time));
        // Threshold floors at 0.1: counter just below max(0.1, thr−5) fails.
        let mut below = c15_check_all_true();
        below.auto_ascend_threshold = 5.0; // max(0.1, 0) = 0.1
        below.ascension_counter_real_real = 0.05;
        assert!(!challenge_15_auto_exponent_check(&below));
    }
}
