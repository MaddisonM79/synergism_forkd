//! Challenge math helpers.
//!
//! Subset of `legacy_core_split/packages/logic/src/mechanics/challenges.ts`
//! — only the items needed downstream are ported. The full
//! `challenges.ts` covers `challengeScoreDisplay`, `getMaxChallenges`, and
//! more; those follow when their callers migrate. For now only
//! [`calc_ecc`] is needed by [`crate::mechanics::tax`] (and eventually by
//! the accelerator / multiplier / producer cost amplifiers, which currently
//! take `transcend_ecc` as a pre-computed number).

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
}
