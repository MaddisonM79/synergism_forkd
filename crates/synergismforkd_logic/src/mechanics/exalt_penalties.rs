//! Singularity-challenge penalty math for Exalt 3 (limitedAscensions),
//! Exalt 4 (noOcteracts), and Exalt 6 (limitedTime).
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/exaltPenalties.ts`.
//! Most of these were already parameterized by `comps` or
//! `(comps, time)` and read no player state — the lifts are
//! near-identity. The two that read player state (Exalt 3 penalty,
//! Exalt 4 multiplier) take their state as an input object.

// ─── Exalt 3 (limitedAscensions) ───────────────────────────────────────────

/// Max ascensions allowed in Exalt 3 before the doubling penalty
/// kicks in. Drops by `2` per challenge completion, floored at `0`.
#[must_use]
pub fn calculate_exalt_3_ascension_limit(comps: f64) -> f64 {
    (15.0 - comps * 2.0).max(0.0)
}

/// Inputs to [`calculate_exalt_3_penalty`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateExalt3PenaltyInput {
    /// `player.singularityChallenges.limitedAscensions.enabled` —
    /// gates the penalty.
    pub limited_ascensions_enabled: bool,
    /// `player.singularityChallenges.limitedAscensions.completions` —
    /// feeds the ascension limit.
    pub limited_ascensions_completions: f64,
    /// `player.ascensionCount` — the current run's ascension count.
    pub ascension_count: f64,
}

/// Returns `2 ^ (ascensions - limit)` once the player crosses the
/// limit, or `1` otherwise. Outside Exalt 3 the penalty is always
/// `1`.
#[must_use]
pub fn calculate_exalt_3_penalty(input: &CalculateExalt3PenaltyInput) -> f64 {
    if !input.limited_ascensions_enabled {
        return 1.0;
    }
    let ascension_limit = calculate_exalt_3_ascension_limit(input.limited_ascensions_completions);
    2.0_f64.powf((input.ascension_count - ascension_limit).max(0.0))
}

// ─── Exalt 4 (noOcteracts) ─────────────────────────────────────────────────

/// Inputs to [`calculate_exalt_4_effective_singularity_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateExalt4EffectiveSingularityMultiplierInput {
    /// `noOcteracts` challenge completion count being evaluated.
    pub comps: f64,
    /// Force the bonus on even outside the challenge — used by
    /// previewers / Statistics displays that show what the bonus
    /// *would* be.
    pub force: bool,
    /// `player.singularityChallenges.noOcteracts.enabled` — gates
    /// the bonus.
    pub in_exalt_4: bool,
}

/// `(comps + 1)^3` if the player is currently in Exalt 4 OR `force`
/// is `true`, else `1`. Used as a singularity-count multiplier when
/// computing rewards under the no-octeracts challenge.
#[must_use]
pub fn calculate_exalt_4_effective_singularity_multiplier(
    input: &CalculateExalt4EffectiveSingularityMultiplierInput,
) -> f64 {
    if input.in_exalt_4 || input.force {
        (input.comps + 1.0).powi(3)
    } else {
        1.0
    }
}

// ─── Exalt 6 (limitedTime) ─────────────────────────────────────────────────

/// Soft time-cap (in seconds) for an Exalt 6 attempt. Goes from
/// `600s` at `0` comps, drops by `60s/comp` until `10` comps (at
/// `60s` base, but the formula switches), then `115s` minus
/// `5s/comp` beyond `10`.
#[must_use]
pub fn calculate_exalt_6_time_limit(comps: f64) -> f64 {
    if comps >= 10.0 {
        115.0 - 5.0 * (comps - 10.0)
    } else {
        600.0 - 60.0 * comps
    }
}

/// Per-minute penalty rate scaling with comp count. Switches to a
/// faster scaling at `10+` comps. Internal — only consumed by
/// [`calculate_exalt_6_penalty_per_second`].
fn calculate_exalt_6_penalty_per_minute(comps: f64) -> f64 {
    if comps >= 10.0 {
        60.0 + 10.0 * (comps - 10.0)
    } else {
        10.0 + 3.0 * comps
    }
}

/// 60-th root of the per-minute penalty rate. Compounding base for
/// the final `^(-displaced_time)` Exalt 6 penalty.
#[must_use]
pub fn calculate_exalt_6_penalty_per_second(comps: f64) -> f64 {
    calculate_exalt_6_penalty_per_minute(comps).powf(1.0 / 60.0)
}

/// Final Exalt 6 penalty multiplier: `1` if the player finishes
/// within [`calculate_exalt_6_time_limit`], otherwise
/// `penalty_per_second ^ -displaced_time` where
/// `displaced_time = time - time_limit`.
#[must_use]
pub fn calculate_exalt_6_penalty(comps: f64, time: f64) -> f64 {
    let time_limit = calculate_exalt_6_time_limit(comps);
    let displaced_time = (time - time_limit).max(0.0);
    if displaced_time == 0.0 {
        return 1.0;
    }
    let penalty_per_second = calculate_exalt_6_penalty_per_second(comps);
    penalty_per_second.powf(-displaced_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Exalt 3 ───────────────────────────────────────────────────────────

    #[test]
    fn exalt_3_limit_starts_at_15() {
        assert_eq!(calculate_exalt_3_ascension_limit(0.0), 15.0);
    }

    #[test]
    fn exalt_3_limit_drops_2_per_comp() {
        assert_eq!(calculate_exalt_3_ascension_limit(3.0), 9.0);
    }

    #[test]
    fn exalt_3_limit_floors_at_zero() {
        assert_eq!(calculate_exalt_3_ascension_limit(20.0), 0.0);
    }

    #[test]
    fn exalt_3_penalty_disabled_returns_one() {
        let result = calculate_exalt_3_penalty(&CalculateExalt3PenaltyInput {
            limited_ascensions_enabled: false,
            limited_ascensions_completions: 0.0,
            ascension_count: 100.0,
        });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn exalt_3_penalty_within_limit_is_one() {
        // limit = 15, ascension_count = 10 → penalty 2^0 = 1
        let result = calculate_exalt_3_penalty(&CalculateExalt3PenaltyInput {
            limited_ascensions_enabled: true,
            limited_ascensions_completions: 0.0,
            ascension_count: 10.0,
        });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn exalt_3_penalty_doubles_per_ascension_past_limit() {
        // limit = 15, ascension_count = 18 → 2^3 = 8
        let result = calculate_exalt_3_penalty(&CalculateExalt3PenaltyInput {
            limited_ascensions_enabled: true,
            limited_ascensions_completions: 0.0,
            ascension_count: 18.0,
        });
        assert_eq!(result, 8.0);
    }

    // ─── Exalt 4 ───────────────────────────────────────────────────────────

    #[test]
    fn exalt_4_outside_challenge_is_one() {
        let result = calculate_exalt_4_effective_singularity_multiplier(
            &CalculateExalt4EffectiveSingularityMultiplierInput {
                comps: 10.0,
                force: false,
                in_exalt_4: false,
            },
        );
        assert_eq!(result, 1.0);
    }

    #[test]
    fn exalt_4_in_challenge_uses_comps_cubed() {
        // (3+1)^3 = 64
        let result = calculate_exalt_4_effective_singularity_multiplier(
            &CalculateExalt4EffectiveSingularityMultiplierInput {
                comps: 3.0,
                force: false,
                in_exalt_4: true,
            },
        );
        assert_eq!(result, 64.0);
    }

    #[test]
    fn exalt_4_force_overrides_in_challenge_check() {
        let result = calculate_exalt_4_effective_singularity_multiplier(
            &CalculateExalt4EffectiveSingularityMultiplierInput {
                comps: 3.0,
                force: true,
                in_exalt_4: false,
            },
        );
        assert_eq!(result, 64.0);
    }

    // ─── Exalt 6 ───────────────────────────────────────────────────────────

    #[test]
    fn exalt_6_time_limit_at_zero_comps_is_600() {
        assert_eq!(calculate_exalt_6_time_limit(0.0), 600.0);
    }

    #[test]
    fn exalt_6_time_limit_drops_60s_per_comp() {
        // 5 comps → 600 - 300 = 300
        assert_eq!(calculate_exalt_6_time_limit(5.0), 300.0);
    }

    #[test]
    fn exalt_6_time_limit_switches_at_10_comps() {
        // At 10 comps, second branch: 115 - 0 = 115
        assert_eq!(calculate_exalt_6_time_limit(10.0), 115.0);
    }

    #[test]
    fn exalt_6_time_limit_drops_5s_per_comp_past_10() {
        // 15 comps → 115 - 25 = 90
        assert_eq!(calculate_exalt_6_time_limit(15.0), 90.0);
    }

    #[test]
    fn exalt_6_penalty_within_time_is_one() {
        // time = 100, limit = 600 → no displaced time → penalty 1
        let result = calculate_exalt_6_penalty(0.0, 100.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn exalt_6_penalty_past_time_is_lt_one() {
        // 0 comps, time = 1000, limit = 600 → displaced = 400
        // penalty_per_minute = 10; per_second = 10^(1/60) ≈ 1.0398
        // penalty = 1.0398^-400 ≈ very small
        let result = calculate_exalt_6_penalty(0.0, 1_000.0);
        assert!(result < 1.0);
        assert!(result > 0.0);
    }
}
