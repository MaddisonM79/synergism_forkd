//! Level / XP state slice.
//!
//! Mirrors `player.level`, `player.levelTier`, and `player.levelXP`.
//! Backs [`crate::mechanics::level_milestones`] and
//! [`crate::mechanics::level_rewards`].

use serde::{Deserialize, Serialize};

use synergismforkd_bignum::Decimal;

/// Slice of `GameState` for the level system.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LevelState {
    /// `player.level` — current level within the active tier.
    pub level: f64,
    /// `player.levelTier` — current tier (`0` = base).
    pub level_tier: f64,
    /// `player.levelXP` — XP accumulator toward next level.
    pub level_xp: Decimal,
}

impl Default for LevelState {
    fn default() -> Self {
        Self {
            level: 0.0,
            level_tier: 0.0,
            level_xp: Decimal::zero(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_zero() {
        let s = LevelState::default();
        assert_eq!(s.level, 0.0);
        assert_eq!(s.level_tier, 0.0);
        assert_eq!(s.level_xp.to_number(), 0.0);
    }
}
