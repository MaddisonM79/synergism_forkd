//! Achievement points math.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/achievementPoints.ts`
//! (lifted from the legacy `packages/web_ui/src/Achievements.ts`). The
//! UI side keeps the achievements data table (i18n descriptions,
//! unlock predicates, group classifications) and the
//! `progressiveAchievements` dispatch map; this module owns the pure
//! formulas that convert per-progressive cached values + the
//! aggregated points sum.

// ─── Progressive achievement: rune levels ──────────────────────────────────

/// Points from the cumulative rune-level progressive achievement.
/// Three-knee staircase: `1pt per 1000` (cap 200), `1pt per 2500`
/// (cap 400), `1pt per 12500` (cap 400). Theoretical max: 1000.
#[must_use]
pub fn rune_level_points(sum_of_rune_levels: f64) -> f64 {
    200.0_f64.min((sum_of_rune_levels / 1_000.0).floor())
        + 400.0_f64.min((sum_of_rune_levels / 2_500.0).floor())
        + 400.0_f64.min((sum_of_rune_levels / 12_500.0).floor())
}

/// Points from the cumulative free-rune-level progressive achievement.
/// Three-knee staircase: `1pt per 250` (cap 100), `1pt per 750` (cap
/// 200), `1pt per 2500` (cap 200). Theoretical max: 500.
#[must_use]
pub fn free_rune_level_points(sum_of_free_rune_levels: f64) -> f64 {
    100.0_f64.min((sum_of_free_rune_levels / 250.0).floor())
        + 200.0_f64.min((sum_of_free_rune_levels / 750.0).floor())
        + 200.0_f64.min((sum_of_free_rune_levels / 2_500.0).floor())
}

// ─── Progressive achievement: ant masteries ────────────────────────────────

/// Points from the ant-masteries progressive achievement. For each
/// ant producer's `highest_mastery`, awards `3 * mastery` plus an
/// extra `+4` when mastery reaches 12.
#[must_use]
pub fn ant_mastery_points(highest_masteries: &[f64]) -> f64 {
    let mut point_value = 0.0_f64;
    for &mastery in highest_masteries {
        point_value += 3.0 * mastery;
        if mastery >= 12.0 {
            point_value += 4.0;
        }
    }
    point_value
}

// ─── Progressive achievement: reborn ELO ───────────────────────────────────

/// Points from the reborn-ELO progressive achievement. Five-knee
/// staircase over the leaderboard value: `1pt per 100` (cap 100),
/// `1pt per 1000` (cap 150), `1pt per 9000` (cap 150), `1pt per 75000`
/// (cap 200), `1pt per 150000` (cap 400). Theoretical max: 1000.
#[must_use]
pub fn reborn_elo_points(leaderboard_elo: f64) -> f64 {
    100.0_f64.min((leaderboard_elo / 100.0).floor())
        + 150.0_f64.min((leaderboard_elo / 1_000.0).floor())
        + 150.0_f64.min((leaderboard_elo / 9_000.0).floor())
        + 200.0_f64.min((leaderboard_elo / 75_000.0).floor())
        + 400.0_f64.min((leaderboard_elo / 150_000.0).floor())
}

// ─── Progressive achievement: singularity count ────────────────────────────

/// Points from the singularity-count progressive achievement.
/// Three-knee accumulator: `9` per singularity, `+3` per singularity
/// above 100, `+3` per singularity above 200.
#[must_use]
pub fn singularity_count_points(highest_singularity_count: f64) -> f64 {
    9.0 * highest_singularity_count
        + 3.0 * 0.0_f64.max(highest_singularity_count - 100.0)
        + 3.0 * 0.0_f64.max(highest_singularity_count - 200.0)
}

// ─── Progressive achievement: ambrosia counts ──────────────────────────────

/// Points from the lifetime-ambrosia progressive achievement.
/// Three-knee staircase: `1pt per 100` (cap 200), `1pt per 10000`
/// (cap 200), sqrt-tail `floor(400 * sqrt(cached / 1e8))` (cap 400).
/// Theoretical max: 800.
#[must_use]
pub fn ambrosia_count_points(lifetime_ambrosia: f64) -> f64 {
    200.0_f64.min((lifetime_ambrosia / 100.0).floor())
        + 200.0_f64.min((lifetime_ambrosia / 10_000.0).floor())
        + 400.0_f64.min((400.0 * (lifetime_ambrosia / 1e8).sqrt()).floor())
}

/// Points from the lifetime-red-ambrosia progressive achievement.
/// Four-knee staircase: `1pt per 25` (cap 200), `1pt per 2500` (cap
/// 200), `400 * cached / 5e6` floored (cap 400), `200 * cached /
/// 1.25e7` floored (cap 200). Theoretical max: 1000.
#[must_use]
pub fn red_ambrosia_count_points(lifetime_red_ambrosia: f64) -> f64 {
    200.0_f64.min((lifetime_red_ambrosia / 25.0).floor())
        + 200.0_f64.min((lifetime_red_ambrosia / 2_500.0).floor())
        + 400.0_f64.min((400.0 * lifetime_red_ambrosia / 5e6).floor())
        + 200.0_f64.min((200.0 * lifetime_red_ambrosia / 1.25e7).floor())
}

// ─── Progressive achievement: talisman rarities ────────────────────────────

/// Points from the talisman-rarities progressive achievement. Trivial
/// `5×` multiplier over the cached sum-of-rarities.
#[must_use]
pub fn talisman_rarity_points(sum_of_rarities: f64) -> f64 {
    5.0 * sum_of_rarities
}

// ─── Progressive achievement: exalts ───────────────────────────────────────

/// Points from the exalt-achievement progressive entry. Sum of
/// `reward_ap` across all singularity challenges.
#[must_use]
pub fn exalt_points(reward_aps: &[f64]) -> f64 {
    reward_aps.iter().sum()
}

// ─── Progressive achievement: fully-maxed upgrade families ─────────────────

/// Generic "count of maxed upgrades × point multiplier" formula. Used
/// by the three upgrade-family progressives:
/// - singularity upgrades: `points_per_maxed = 5`
/// - octeract upgrades: `points_per_maxed = 8`
/// - red-ambrosia upgrades: `points_per_maxed = 10`
#[must_use]
pub fn maxed_upgrade_family_points(maxed_count: f64, points_per_maxed: f64) -> f64 {
    maxed_count * points_per_maxed
}

// ─── Achievement-completion quark reward ───────────────────────────────────

/// Quarks awarded for completing a brand-new achievement. Starts at
/// `5 * global_quark_multiplier`; above a 100× multiplier, applies a
/// softcap of `100^0.6 * mult^0.4` so the bonus stays meaningful at
/// high multipliers without blowing up. Final result is floored.
#[must_use]
pub fn get_achievement_quarks(global_quark_multiplier: f64) -> f64 {
    let mut actual_multiplier = global_quark_multiplier;
    if actual_multiplier > 100.0 {
        actual_multiplier = 100.0_f64.powf(0.6) * actual_multiplier.powf(0.4);
    }
    (5.0 * actual_multiplier).floor()
}

// ─── Total achievement points ──────────────────────────────────────────────

/// Inputs to [`compute_achievement_points`].
#[derive(Debug, Clone, Copy)]
pub struct ComputeAchievementPointsInput<'a> {
    /// Per-achievement `point_value` from the achievements data
    /// table.
    pub point_values: &'a [f64],
    /// Per-achievement unlocked flag. Truthy (typically 1) when
    /// awarded. Indices align with `point_values`. Any non-zero
    /// value counts as unlocked.
    pub saved_achievements: &'a [u8],
    /// Per-progressive-achievement awarded points. Caller assembles
    /// this by calling each progressive's points-awarded formula and
    /// dropping the result into the slice in any order.
    pub progressive_points_awarded: &'a [f64],
}

/// Total achievement points: sum of per-achievement point values for
/// unlocked achievements, plus the sum of per-progressive awarded
/// points.
#[must_use]
pub fn compute_achievement_points(input: &ComputeAchievementPointsInput<'_>) -> f64 {
    let mut points = 0.0_f64;
    for (i, value) in input.point_values.iter().enumerate() {
        if input.saved_achievements.get(i).copied().unwrap_or(0) != 0 {
            points += value;
        }
    }
    for awarded in input.progressive_points_awarded {
        points += awarded;
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rune_level_points_below_first_knee() {
        assert_eq!(rune_level_points(0.0), 0.0);
        // 999 → floor(999/1000) = 0; floor(999/2500) = 0; floor(999/12500) = 0
        assert_eq!(rune_level_points(999.0), 0.0);
    }

    #[test]
    fn rune_level_points_at_first_knee() {
        // 1000 → 1 + 0 + 0 = 1
        assert_eq!(rune_level_points(1_000.0), 1.0);
    }

    #[test]
    fn rune_level_points_caps_at_max() {
        // Very high values cap at 200 + 400 + 400 = 1000
        let result = rune_level_points(1e9);
        assert_eq!(result, 1_000.0);
    }

    #[test]
    fn ant_mastery_points_per_ant() {
        // 1 ant at mastery 5 → 15; mastery 12 → 3*12 + 4 = 40
        let result = ant_mastery_points(&[5.0, 12.0]);
        assert_eq!(result, 15.0 + 40.0);
    }

    #[test]
    fn singularity_count_points_three_knee() {
        // 250 sings → 9*250 + 3*150 + 3*50 = 2250 + 450 + 150 = 2850
        assert_eq!(singularity_count_points(250.0), 2_850.0);
    }

    #[test]
    fn singularity_count_points_below_first_knee() {
        // 50 sings → 9*50 + 0 + 0 = 450
        assert_eq!(singularity_count_points(50.0), 450.0);
    }

    #[test]
    fn talisman_rarity_points_is_5x() {
        assert_eq!(talisman_rarity_points(20.0), 100.0);
    }

    #[test]
    fn exalt_points_sums_slice() {
        let aps = [10.0, 20.0, 30.0, 5.0];
        assert_eq!(exalt_points(&aps), 65.0);
    }

    #[test]
    fn maxed_upgrade_family_points_is_count_times_multiplier() {
        assert_eq!(maxed_upgrade_family_points(10.0, 5.0), 50.0);
        assert_eq!(maxed_upgrade_family_points(20.0, 8.0), 160.0);
    }

    #[test]
    fn achievement_quarks_below_softcap_is_5x() {
        assert_eq!(get_achievement_quarks(10.0), 50.0);
    }

    #[test]
    fn achievement_quarks_above_softcap_uses_pow_blend() {
        // mult = 1000 → 100^0.6 * 1000^0.4
        // ≈ 15.849 * 15.849 ≈ 251.19; *5 = 1255.9; floor = 1255
        let result = get_achievement_quarks(1_000.0);
        let expected = (5.0 * 100.0_f64.powf(0.6) * 1_000.0_f64.powf(0.4)).floor();
        assert_eq!(result, expected);
    }

    #[test]
    fn compute_achievement_points_sums_unlocked_and_progressive() {
        let input = ComputeAchievementPointsInput {
            point_values: &[10.0, 20.0, 30.0, 40.0],
            saved_achievements: &[1, 0, 1, 1],
            progressive_points_awarded: &[100.0, 200.0],
        };
        // unlocked: 10 + 30 + 40 = 80; progressive: 300; total = 380
        assert_eq!(compute_achievement_points(&input), 380.0);
    }

    #[test]
    fn compute_achievement_points_treats_any_truthy_as_unlocked() {
        let input = ComputeAchievementPointsInput {
            point_values: &[10.0, 20.0],
            saved_achievements: &[5, 0], // 5 is truthy
            progressive_points_awarded: &[],
        };
        assert_eq!(compute_achievement_points(&input), 10.0);
    }
}
