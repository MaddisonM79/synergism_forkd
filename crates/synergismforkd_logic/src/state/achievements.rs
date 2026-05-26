//! Achievements state slice.
//!
//! Mirrors `player.achievements`, `player.progressiveAchievements`,
//! and `player.achievementPoints`. Backs the
//! [`crate::mechanics::achievement_levels`] and
//! [`crate::mechanics::achievement_points`] mechanics.

/// Saved cache for one progressive-achievement entry. The legacy
/// shape stores a single cached value used to detect updates each
/// tick; all 8 progressive achievements share this shape.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct ProgressiveAchievementCache {
    /// Cached input value last evaluated. Drives the
    /// recompute-on-change detection.
    pub cached_value: f64,
    /// Cached points awarded for this progressive entry.
    pub cached_points: f64,
}

/// Slice of `GameState` read/written by achievement mechanics.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AchievementsState {
    /// `player.achievements[i]` — 0 = unowned, non-zero = unlocked.
    /// 1-indexed (index 0 unused) to match legacy.
    pub achievements: Vec<u8>,
    /// Total achievement points earned.
    pub achievement_points: f64,
    /// Progressive achievement caches:
    /// 0=runeLevels, 1=freeRuneLevels, 2=antMasteries,
    /// 3=rebornELO, 4=singularityCount, 5=ambrosiaCounts,
    /// 6=redAmbrosiaCounts, 7=talismanRarities,
    /// 8=exaltPoints, 9=singularityUpgradesMaxed,
    /// 10=octeractUpgradesMaxed, 11=redAmbrosiaUpgradesMaxed.
    pub progressive: [ProgressiveAchievementCache; 12],
}

impl AchievementsState {
    /// Build with `n_achievements + 1` slots.
    #[must_use]
    pub fn new(n_achievements: usize) -> Self {
        Self {
            achievements: vec![0; n_achievements + 1],
            achievement_points: 0.0,
            progressive: [ProgressiveAchievementCache::default(); 12],
        }
    }
}

impl Default for AchievementsState {
    fn default() -> Self {
        // Legacy synergism has ~280 achievements.
        Self::new(280)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_281_slots() {
        let s = AchievementsState::default();
        assert_eq!(s.achievements.len(), 281);
        assert_eq!(s.achievement_points, 0.0);
    }

    #[test]
    fn progressive_caches_default_to_zero() {
        let s = AchievementsState::default();
        assert_eq!(s.progressive[0].cached_value, 0.0);
        assert_eq!(s.progressive[11].cached_points, 0.0);
    }
}
