//! Challenges state slice.
//!
//! Mirrors `player.challengecompletions`,
//! `player.highestchallengecompletions`, and the three
//! `player.currentChallenge.*` fields. Backs
//! [`crate::mechanics::challenges`] and is read by most
//! per-tick aggregators.

/// Slice of `GameState` for challenge completion + current-challenge
/// tracking. Indices `1..=15` are used; index `0` is unused and held
/// at `0` to match the legacy shape.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChallengesState {
    /// Current completion count per challenge (resets with the
    /// relevant prestige tier).
    pub challenge_completions: [f64; 16],
    /// All-time highest completion count per challenge.
    pub highest_challenge_completions: [f64; 16],
    /// Active transcension challenge ID (`0` = none). Range `1..=5`.
    pub current_transcension_challenge: u32,
    /// Active reincarnation challenge ID (`0` = none). Range `6..=10`.
    pub current_reincarnation_challenge: u32,
    /// Active ascension challenge ID (`0` = none). Range `11..=15`.
    pub current_ascension_challenge: u32,
    /// `player.challenge15Exponent` — cumulative Challenge-15 exponent;
    /// gates the c15 reward tiers (`mechanics::challenge_15_rewards`).
    pub challenge15_exponent: f64,
}

impl Default for ChallengesState {
    fn default() -> Self {
        Self {
            challenge_completions: [0.0; 16],
            highest_challenge_completions: [0.0; 16],
            current_transcension_challenge: 0,
            current_reincarnation_challenge: 0,
            current_ascension_challenge: 0,
            challenge15_exponent: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_16_slots() {
        let s = ChallengesState::default();
        assert_eq!(s.challenge_completions.len(), 16);
        assert_eq!(s.highest_challenge_completions.len(), 16);
    }

    #[test]
    fn default_no_active_challenge() {
        let s = ChallengesState::default();
        assert_eq!(s.current_transcension_challenge, 0);
        assert_eq!(s.current_reincarnation_challenge, 0);
        assert_eq!(s.current_ascension_challenge, 0);
    }
}
