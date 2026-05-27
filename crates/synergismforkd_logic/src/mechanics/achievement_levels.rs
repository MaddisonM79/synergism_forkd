//! Achievement-level math.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/achievementLevels.ts`
//! (lifted from the legacy `packages/web_ui/src/Achievements.ts`).
//! The level-from-points and exp-to-next-level formulas are pure
//! functions of the achievement-points total. Both share a 2500-point
//! regime switch — below that, levels are 50 points apart; above,
//! 100 points apart.

const REGIME_SWITCH_POINTS: f64 = 2_500.0;
const REGIME_SWITCH_LEVEL: f64 = 50.0;
const LOW_REGIME_POINTS_PER_LEVEL: f64 = 50.0;
const HIGH_REGIME_POINTS_PER_LEVEL: f64 = 100.0;

/// Achievement level for a given points total. Below 2500 points the
/// level advances every 50 points; above 2500 it advances every 100
/// points (with level 50 reached at exactly 2500). Uses `floor` so
/// partial progress doesn't count.
#[must_use]
pub fn achievement_level_from_points(points: f64) -> f64 {
    if points < REGIME_SWITCH_POINTS {
        return (points / LOW_REGIME_POINTS_PER_LEVEL).floor();
    }
    REGIME_SWITCH_LEVEL + ((points - REGIME_SWITCH_POINTS) / HIGH_REGIME_POINTS_PER_LEVEL).floor()
}

/// Points remaining until the next achievement level. Uses the same
/// 2500-point regime switch: `50 - (points % 50)` below,
/// `100 - (points % 100)` above. Note that the value is the *gap* to
/// the next level — at the exact threshold, returns the full level
/// cost (50 below 2500, 100 above).
#[must_use]
pub fn to_next_achievement_level_exp(points: f64) -> f64 {
    if points < REGIME_SWITCH_POINTS {
        return LOW_REGIME_POINTS_PER_LEVEL - (points % LOW_REGIME_POINTS_PER_LEVEL);
    }
    HIGH_REGIME_POINTS_PER_LEVEL - (points % HIGH_REGIME_POINTS_PER_LEVEL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_zero_points_is_zero() {
        assert_eq!(achievement_level_from_points(0.0), 0.0);
    }

    #[test]
    fn level_below_regime_switch_uses_50_per_level() {
        assert_eq!(achievement_level_from_points(49.0), 0.0);
        assert_eq!(achievement_level_from_points(50.0), 1.0);
        assert_eq!(achievement_level_from_points(2_499.0), 49.0);
    }

    #[test]
    fn level_at_regime_switch_is_50() {
        assert_eq!(achievement_level_from_points(2_500.0), 50.0);
    }

    #[test]
    fn level_above_regime_switch_uses_100_per_level() {
        assert_eq!(achievement_level_from_points(2_600.0), 51.0);
        assert_eq!(achievement_level_from_points(3_500.0), 60.0);
    }

    #[test]
    fn to_next_at_zero_returns_low_regime_cost() {
        assert_eq!(to_next_achievement_level_exp(0.0), 50.0);
    }

    #[test]
    fn to_next_partial_progress_below_regime_switch() {
        assert_eq!(to_next_achievement_level_exp(40.0), 10.0);
    }

    #[test]
    fn to_next_at_regime_switch_returns_high_regime_cost() {
        assert_eq!(to_next_achievement_level_exp(2_500.0), 100.0);
    }

    #[test]
    fn to_next_partial_progress_above_regime_switch() {
        // 2600 % 100 = 0 → 100 - 0 = 100 (full level cost ahead)
        assert_eq!(to_next_achievement_level_exp(2_600.0), 100.0);
        // 2650 % 100 = 50 → 100 - 50 = 50
        assert_eq!(to_next_achievement_level_exp(2_650.0), 50.0);
    }
}
