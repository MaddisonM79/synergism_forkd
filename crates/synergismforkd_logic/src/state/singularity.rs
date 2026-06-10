//! Singularity state slice — counts + the 9-challenge tracker.
//!
//! Mirrors `player.singularityCount`, `player.highestSingularityCount`,
//! `player.singularityCounter`, and `player.singularityChallenges.X`
//! for each of the 9 named challenges. Backs
//! [`crate::mechanics::singularity_helpers`],
//! [`crate::mechanics::singularity_milestones`],
//! [`crate::mechanics::singularity_penalties`], and
//! [`crate::mechanics::singularity_challenges`].

/// Per-challenge tracker for one singularity (Exalt) challenge.
/// Mirrors `player.singularityChallenges.X` shape.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub struct SingularityChallengeState {
    /// Whether the player is currently inside this challenge.
    pub enabled: bool,
    /// Total times this challenge has been completed.
    pub completions: f64,
    /// Highest singularity count at which this challenge was
    /// completed.
    pub highest_singularity_completed: f64,
}

/// Slice of `GameState` for the singularity feature.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct SingularityState {
    /// Current singularity count (resets each ascension-reset).
    pub singularity_count: f64,
    /// All-time highest singularity count reached.
    pub highest_singularity_count: f64,
    /// In-singularity timer (seconds since this singularity began).
    pub singularity_counter: f64,
    /// `player.singChallengeTimer` — time inside the current singularity
    /// challenge (accumulates while `enabled`, resets to 0 otherwise).
    pub sing_challenge_timer: f64,
    /// `player.singularityElevatorTarget` — the elevator's chosen floor.
    /// The configure action clamps it to
    /// `[1, max(1, highest, count + lookahead if antiquities)]`.
    pub elevator_target: f64,
    /// `player.singularityElevatorSlowClimb` — when set (the blank-save
    /// default), a normal singularity advances the count by exactly one
    /// instead of jumping by the fast-forward lookahead.
    pub elevator_slow_climb: bool,
    /// `player.singularityElevatorLocked` — when set, a normal singularity
    /// goes to [`Self::elevator_target`] instead of advancing.
    pub elevator_locked: bool,
    /// `noSingularityUpgrades` Exalt 1.
    pub no_singularity_upgrades: SingularityChallengeState,
    /// `oneChallengeCap` Exalt 2.
    pub one_challenge_cap: SingularityChallengeState,
    /// `noOcteracts` Exalt 4.
    pub no_octeracts: SingularityChallengeState,
    /// `limitedAscensions` Exalt 3.
    pub limited_ascensions: SingularityChallengeState,
    /// `noAmbrosiaUpgrades` Exalt 5.
    pub no_ambrosia_upgrades: SingularityChallengeState,
    /// `noQuarkUpgrades`.
    pub no_quark_upgrades: SingularityChallengeState,
    /// `limitedTime` Exalt 6.
    pub limited_time: SingularityChallengeState,
    /// `sadisticPrequel` Exalt 7.
    pub sadistic_prequel: SingularityChallengeState,
    /// `taxmanLastStand` Exalt 8.
    pub taxman_last_stand: SingularityChallengeState,
}

impl Default for SingularityState {
    fn default() -> Self {
        Self {
            singularity_count: 0.0,
            highest_singularity_count: 0.0,
            singularity_counter: 0.0,
            sing_challenge_timer: 0.0,
            // blankSave: target 1, slowClimb TRUE, locked false
            // (Synergism.ts:1076-1078).
            elevator_target: 1.0,
            elevator_slow_climb: true,
            elevator_locked: false,
            no_singularity_upgrades: SingularityChallengeState::default(),
            one_challenge_cap: SingularityChallengeState::default(),
            no_octeracts: SingularityChallengeState::default(),
            limited_ascensions: SingularityChallengeState::default(),
            no_ambrosia_upgrades: SingularityChallengeState::default(),
            no_quark_upgrades: SingularityChallengeState::default(),
            limited_time: SingularityChallengeState::default(),
            sadistic_prequel: SingularityChallengeState::default(),
            taxman_last_stand: SingularityChallengeState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_zero() {
        let s = SingularityState::default();
        assert_eq!(s.singularity_count, 0.0);
        assert!(!s.no_octeracts.enabled);
        assert_eq!(s.taxman_last_stand.completions, 0.0);
    }
}
