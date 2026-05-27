//! Per-milestone level scaling formulas.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/levelMilestones.ts`
//! (lifted from the legacy `packages/web_ui/src/Levels.ts`). Almost
//! every milestone is either a pure `(level: f64) -> f64` or a
//! constant `1.0` unlock flag — they all dispatch via
//! [`get_level_milestone`].
//!
//! The one exception is `salvageChallengeBuff`, whose value depends
//! on which challenge the player is in. It lives as a standalone
//! [`salvage_challenge_buff_effect`] here; the UI side sources the
//! challenge state and calls it directly.

/// Key for [`get_level_milestone`]. Mirrors the legacy
/// `LevelMilestoneKey` string union.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelMilestoneKey {
    /// Offering-timer scaling unlock (level 5).
    OfferingTimerScaling,
    /// Auto-prestige unlock (level 7).
    AutoPrestige,
    /// Speed-rune milestone scaling (from level 20).
    SpeedRune,
    /// Duplication-rune milestone scaling (from level 40).
    DuplicationRune,
    /// Prism-rune milestone scaling (from level 60).
    PrismRune,
    /// Thrift-rune milestone scaling (from level 80).
    ThriftRune,
    /// SI-rune milestone scaling (from level 100).
    SiRune,
    /// Tier-1 crystal autobuy unlock (level 6).
    Tier1CrystalAutobuy,
    /// Tier-2 crystal autobuy unlock (level 9).
    Tier2CrystalAutobuy,
    /// Tier-3 crystal autobuy unlock (level 12).
    Tier3CrystalAutobuy,
    /// Tier-4 crystal autobuy unlock (level 15).
    Tier4CrystalAutobuy,
    /// Tier-5 crystal autobuy unlock (level 20).
    Tier5CrystalAutobuy,
    /// Achievement-talisman unlock (level 100).
    AchievementTalismanUnlock,
    /// Rune-autobuy interval improver (from level 130).
    RuneAutobuyImprover,
    /// Achievement-talisman enhancement (from level 160).
    AchievementTalismanEnhancement,
    /// Ant-speed-2 autobuyer unlock (level 65).
    AntSpeed2Autobuyer,
    /// Wow-cubes autobuyer unlock (level 80).
    WowCubesAutobuyer,
    /// Ascension-score autobuyer unlock (level 80).
    AscensionScoreAutobuyer,
    /// Mortuus-2 autobuyer unlock (level 225).
    Mortuus2Autobuyer,
}

// ─── Per-milestone effect formulas ─────────────────────────────────────────

fn rune_scaling_effect(level: f64, coeff: f64, threshold: f64) -> f64 {
    coeff * (level - threshold)
}

/// `0.5 * (level - 19)` from level 20.
#[must_use]
pub fn speed_rune_milestone_effect(level: f64) -> f64 {
    rune_scaling_effect(level, 0.5, 19.0)
}

/// `0.4 * (level - 39)` from level 40.
#[must_use]
pub fn duplication_rune_milestone_effect(level: f64) -> f64 {
    rune_scaling_effect(level, 0.4, 39.0)
}

/// `0.3 * (level - 59)` from level 60.
#[must_use]
pub fn prism_rune_milestone_effect(level: f64) -> f64 {
    rune_scaling_effect(level, 0.3, 59.0)
}

/// `0.2 * (level - 79)` from level 80.
#[must_use]
pub fn thrift_rune_milestone_effect(level: f64) -> f64 {
    rune_scaling_effect(level, 0.2, 79.0)
}

/// `0.1 * (level - 99)` from level 100.
#[must_use]
pub fn si_rune_milestone_effect(level: f64) -> f64 {
    rune_scaling_effect(level, 0.1, 99.0)
}

/// Rune autobuyer interval improver —
/// `1.1 + 0.01 * (level - 130)`. Returns `1.0` below level 130 via
/// [`get_level_milestone`]'s default-value gate.
#[must_use]
pub fn rune_autobuy_improver_effect(level: f64) -> f64 {
    1.1 + 0.01 * (level - 130.0)
}

/// Achievement-talisman enhancement — passes the level through.
#[must_use]
pub fn achievement_talisman_enhancement_effect(level: f64) -> f64 {
    level
}

// ─── salvageChallengeBuff (impure — reads challenge state) ─────────────────

/// Inputs to [`salvage_challenge_buff_effect`].
#[derive(Debug, Clone, Copy)]
pub struct SalvageChallengeBuffInput {
    /// `true` when ANY of
    /// `player.currentChallenge.{transcension, reincarnation, ascension}`
    /// is non-zero. Doubles the base buff.
    pub in_any_challenge: bool,
    /// `true` when
    /// `player.currentChallenge.ascension === 15`. Doubles again on
    /// top.
    pub in_ascension_15: bool,
    /// `player.insideSingularityChallenge`. Triples on top of any
    /// other multipliers.
    pub inside_singularity_challenge: bool,
}

/// Salvage buff inside challenges. Base `25`; `×2` inside any normal
/// challenge, `×2` again inside C15 specifically (so `×4` total
/// there), `×3` again inside any singularity challenge (cumulative
/// with the prior multipliers).
#[must_use]
pub fn salvage_challenge_buff_effect(input: &SalvageChallengeBuffInput) -> f64 {
    let mut base_val = 25.0_f64;
    if input.in_any_challenge {
        base_val *= 2.0;
    }
    if input.in_ascension_15 {
        base_val *= 2.0;
    }
    if input.inside_singularity_challenge {
        base_val *= 3.0;
    }
    base_val
}

// ─── Dispatcher ────────────────────────────────────────────────────────────

/// Returns the active milestone value for a given achievement level.
/// Below the milestone's `level_req`, returns the `default_value`;
/// otherwise invokes the milestone's effect. Does **not** cover
/// `SalvageChallengeBuff` — that one needs challenge state and is
/// exposed separately as [`salvage_challenge_buff_effect`].
#[must_use]
pub fn get_level_milestone(milestone: LevelMilestoneKey, level: f64) -> f64 {
    use LevelMilestoneKey as K;
    match milestone {
        // Unlock-flag milestones — return 1 once unlocked, 0 below.
        K::OfferingTimerScaling => {
            if level >= 5.0 {
                1.0
            } else {
                0.0
            }
        }
        K::AutoPrestige => {
            if level >= 7.0 {
                1.0
            } else {
                0.0
            }
        }
        K::Tier1CrystalAutobuy => {
            if level >= 6.0 {
                1.0
            } else {
                0.0
            }
        }
        K::Tier2CrystalAutobuy => {
            if level >= 9.0 {
                1.0
            } else {
                0.0
            }
        }
        K::Tier3CrystalAutobuy => {
            if level >= 12.0 {
                1.0
            } else {
                0.0
            }
        }
        K::Tier4CrystalAutobuy => {
            if level >= 15.0 {
                1.0
            } else {
                0.0
            }
        }
        K::Tier5CrystalAutobuy => {
            if level >= 20.0 {
                1.0
            } else {
                0.0
            }
        }
        K::AchievementTalismanUnlock => {
            if level >= 100.0 {
                1.0
            } else {
                0.0
            }
        }
        K::AntSpeed2Autobuyer => {
            if level >= 65.0 {
                1.0
            } else {
                0.0
            }
        }
        K::WowCubesAutobuyer => {
            if level >= 80.0 {
                1.0
            } else {
                0.0
            }
        }
        K::AscensionScoreAutobuyer => {
            if level >= 80.0 {
                1.0
            } else {
                0.0
            }
        }
        K::Mortuus2Autobuyer => {
            if level >= 225.0 {
                1.0
            } else {
                0.0
            }
        }

        // Rune-scaling milestones — default 0 below level_req.
        K::SpeedRune => {
            if level >= 20.0 {
                speed_rune_milestone_effect(level)
            } else {
                0.0
            }
        }
        K::DuplicationRune => {
            if level >= 40.0 {
                duplication_rune_milestone_effect(level)
            } else {
                0.0
            }
        }
        K::PrismRune => {
            if level >= 60.0 {
                prism_rune_milestone_effect(level)
            } else {
                0.0
            }
        }
        K::ThriftRune => {
            if level >= 80.0 {
                thrift_rune_milestone_effect(level)
            } else {
                0.0
            }
        }
        K::SiRune => {
            if level >= 100.0 {
                si_rune_milestone_effect(level)
            } else {
                0.0
            }
        }

        // Rune-autobuy improver — default 1 below level 130.
        K::RuneAutobuyImprover => {
            if level >= 130.0 {
                rune_autobuy_improver_effect(level)
            } else {
                1.0
            }
        }

        // Achievement-talisman enhancement — default 0 below level 160.
        K::AchievementTalismanEnhancement => {
            if level >= 160.0 {
                achievement_talisman_enhancement_effect(level)
            } else {
                0.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offering_timer_scaling_unlocks_at_5() {
        assert_eq!(
            get_level_milestone(LevelMilestoneKey::OfferingTimerScaling, 4.0),
            0.0
        );
        assert_eq!(
            get_level_milestone(LevelMilestoneKey::OfferingTimerScaling, 5.0),
            1.0
        );
    }

    #[test]
    fn speed_rune_scales_after_level_20() {
        assert_eq!(get_level_milestone(LevelMilestoneKey::SpeedRune, 19.0), 0.0);
        // 0.5 * (20 - 19) = 0.5
        assert_eq!(get_level_milestone(LevelMilestoneKey::SpeedRune, 20.0), 0.5);
    }

    #[test]
    fn rune_autobuy_improver_defaults_to_one_below_130() {
        assert_eq!(
            get_level_milestone(LevelMilestoneKey::RuneAutobuyImprover, 100.0),
            1.0
        );
        assert_eq!(
            get_level_milestone(LevelMilestoneKey::RuneAutobuyImprover, 130.0),
            1.1
        );
    }

    #[test]
    fn salvage_buff_base_is_25() {
        let input = SalvageChallengeBuffInput {
            in_any_challenge: false,
            in_ascension_15: false,
            inside_singularity_challenge: false,
        };
        assert_eq!(salvage_challenge_buff_effect(&input), 25.0);
    }

    #[test]
    fn salvage_buff_c15_quadruples() {
        let input = SalvageChallengeBuffInput {
            in_any_challenge: true,
            in_ascension_15: true,
            inside_singularity_challenge: false,
        };
        // 25 × 2 × 2 = 100
        assert_eq!(salvage_challenge_buff_effect(&input), 100.0);
    }

    #[test]
    fn salvage_buff_full_stack() {
        let input = SalvageChallengeBuffInput {
            in_any_challenge: true,
            in_ascension_15: true,
            inside_singularity_challenge: true,
        };
        // 25 × 2 × 2 × 3 = 300
        assert_eq!(salvage_challenge_buff_effect(&input), 300.0);
    }
}
