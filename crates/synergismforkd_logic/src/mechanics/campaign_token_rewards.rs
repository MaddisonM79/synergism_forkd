//! Per-reward campaign-token bonus formulas.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/campaignTokenRewards.ts`
//! (lifted from the legacy `packages/web_ui/src/Campaign.ts`). The UI
//! still owns the `CampaignManager` class wrapper and the
//! per-campaign data table; this module owns the 14 pure bonus
//! formulas that each take the current `campaign_tokens` count and
//! return a scalar (or, for `tutorial_bonus`, a 3-field struct).
//! All formulas are pure functions of `campaign_tokens` with no
//! player or game-state reads.

/// Result of [`tutorial_bonus`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CampaignTutorialBonus {
    /// Cube bonus multiplier.
    pub cube_bonus: f64,
    /// Obtainium bonus multiplier.
    pub obtainium_bonus: f64,
    /// Offering bonus multiplier.
    pub offering_bonus: f64,
}

// ─── Threshold constants ──────────────────────────────────────────────────

/// Verbatim from legacy. Used by
/// [`campaign_time_threshold_reduction`] — returns the index into
/// this array (clamped by the array length, max 2).
const TIME_THRESHOLD_REQS: [f64; 8] = [
    20.0, 100.0, 250.0, 500.0, 1_000.0, 2_000.0, 3_500.0, 5_000.0,
];

/// Verbatim from legacy. Used by [`campaign_bonus_rune_6`].
const BONUS_RUNE_6_THRESHOLD_REQS: [f64; 12] = [
    500.0, 750.0, 1_000.0, 1_250.0, 1_500.0, 1_750.0, 2_000.0, 3_000.0, 4_000.0, 6_000.0, 8_000.0,
    10_000.0,
];

// ─── Bonus formulas ───────────────────────────────────────────────────────

/// `tutorial_bonus`: three independent boolean-shaped bonuses, all
/// active iff any campaign token has been earned.
#[must_use]
pub fn tutorial_bonus(campaign_tokens: f64) -> CampaignTutorialBonus {
    let active = if campaign_tokens > 0.0 { 1.0 } else { 0.0 };
    CampaignTutorialBonus {
        cube_bonus: 1.0 + 0.25 * active,
        obtainium_bonus: 1.0 + 0.2 * active,
        offering_bonus: 1.0 + 0.2 * active,
    }
}

/// Cube bonus — three-band piecewise: linear-ramp `0..25`, then
/// saturating exp toward bigger caps.
#[must_use]
pub fn campaign_cube_bonus(campaign_tokens: f64) -> f64 {
    1.0 + 0.4 * (1.0 / 25.0) * campaign_tokens.min(25.0)
        + 0.6 * (1.0 - (-(campaign_tokens - 25.0).max(0.0) / 500.0).exp())
        + 1.0 * (1.0 - (-(campaign_tokens - 2_500.0).max(0.0) / 5_000.0).exp())
}

/// Obtainium bonus — same shape as [`campaign_cube_bonus`] with
/// different cap weights.
#[must_use]
pub fn campaign_obtainium_bonus(campaign_tokens: f64) -> f64 {
    1.0 + 0.1 * (1.0 / 25.0) * campaign_tokens.min(25.0)
        + 0.4 * (1.0 - (-(campaign_tokens - 25.0).max(0.0) / 500.0).exp())
        + 0.5 * (1.0 - (-(campaign_tokens - 2_500.0).max(0.0) / 5_000.0).exp())
}

/// Offering bonus — same shape as [`campaign_obtainium_bonus`].
#[must_use]
pub fn campaign_offering_bonus(campaign_tokens: f64) -> f64 {
    1.0 + 0.1 * (1.0 / 25.0) * campaign_tokens.min(25.0)
        + 0.4 * (1.0 - (-(campaign_tokens - 25.0).max(0.0) / 500.0).exp())
        + 0.5 * (1.0 - (-(campaign_tokens - 2_500.0).max(0.0) / 5_000.0).exp())
}

/// Ascension-score multiplier — wider linear band (`0..100`) before
/// the exp terms kick in.
#[must_use]
pub fn campaign_ascension_score_multiplier(campaign_tokens: f64) -> f64 {
    1.0 + 0.2 * (1.0 / 100.0) * campaign_tokens.min(100.0)
        + 0.3 * (1.0 - (-(campaign_tokens - 100.0).max(0.0) / 1_000.0).exp())
        + 0.5 * (1.0 - (-(campaign_tokens - 2_500.0).max(0.0) / 5_000.0).exp())
}

/// Time-threshold reduction — staircase that returns `i/4` for the
/// first index past which `campaign_tokens` falls below, capped at
/// `2` (after 8 thresholds). Each step adds `0.25`, total range
/// `0..=2`.
#[must_use]
pub fn campaign_time_threshold_reduction(campaign_tokens: f64) -> f64 {
    for (i, req) in TIME_THRESHOLD_REQS.iter().enumerate() {
        if campaign_tokens < *req {
            return i as f64 / 4.0;
        }
    }
    2.0
}

/// Quark bonus — gated until `campaign_tokens >= 100`, then a
/// three-band piecewise. Below the gate returns `1`.
#[must_use]
pub fn campaign_quark_bonus(campaign_tokens: f64) -> f64 {
    if campaign_tokens < 100.0 {
        return 1.0;
    }
    1.0 + 0.05 * (campaign_tokens - 100.0).min(100.0) / 100.0
        + 0.05 * (1.0 - (-(campaign_tokens - 200.0).max(0.0) / 3_000.0).exp())
        + 0.1 * (1.0 - (-(campaign_tokens - 2_500.0).max(0.0) / 10_000.0).exp())
}

/// Tax multiplier — gated at `>= 250`, then a **negative**
/// three-band piecewise. Output decreases below `1` (tax-reducing).
#[must_use]
pub fn campaign_tax_multiplier(campaign_tokens: f64) -> f64 {
    if campaign_tokens < 250.0 {
        return 1.0;
    }
    1.0 - 0.05 * (1.0 / 250.0) * (campaign_tokens - 250.0).min(250.0)
        - 0.15 * (1.0 - (-(campaign_tokens - 500.0).max(0.0) / 1_250.0).exp())
        - 0.05 * (1.0 - (-(campaign_tokens - 4_000.0).max(0.0) / 5_000.0).exp())
}

/// C15 bonus — gated at `>= 250`, two-band positive piecewise.
#[must_use]
pub fn campaign_c15_bonus(campaign_tokens: f64) -> f64 {
    if campaign_tokens < 250.0 {
        return 1.0;
    }
    1.0 + 0.05 * (1.0 / 250.0) * (campaign_tokens - 250.0).min(250.0)
        + 0.95 * (1.0 - (-(campaign_tokens - 500.0).max(0.0) / 1_250.0).exp())
}

/// Bonus-rune-6 staircase — returns the index of the first
/// threshold the player hasn't passed. Returns `0..=12` (capped at
/// `12` after all 12 thresholds).
#[must_use]
pub fn campaign_bonus_rune_6(campaign_tokens: f64) -> f64 {
    for (i, req) in BONUS_RUNE_6_THRESHOLD_REQS.iter().enumerate() {
        if campaign_tokens < *req {
            return i as f64;
        }
    }
    12.0
}

/// Golden-quark bonus — gated at `>= 500`, two-band piecewise.
#[must_use]
pub fn campaign_golden_quark_bonus(campaign_tokens: f64) -> f64 {
    if campaign_tokens < 500.0 {
        return 1.0;
    }
    1.0 + 0.05 * (1.0 / 500.0) * (campaign_tokens - 500.0).min(500.0)
        + 0.05 * (1.0 - (-(campaign_tokens - 1_000.0).max(0.0) / 2_500.0).exp())
}

/// Octeract bonus — gated at `>= 1000`, two-band piecewise.
#[must_use]
pub fn campaign_octeract_bonus(campaign_tokens: f64) -> f64 {
    if campaign_tokens < 1_000.0 {
        return 1.0;
    }
    1.0 + 0.1 * (1.0 / 1_000.0) * (campaign_tokens - 1_000.0).min(1_000.0)
        + 0.15 * (1.0 - (-(campaign_tokens - 2_000.0).max(0.0) / 4_000.0).exp())
}

/// Ambrosia-luck bonus — gated at `>= 2000`, **additive** style.
/// Base value is `10` (not `1`) past the gate. Returns `0` below.
#[must_use]
pub fn campaign_ambrosia_luck_bonus(campaign_tokens: f64) -> f64 {
    if campaign_tokens < 2_000.0 {
        return 0.0;
    }
    10.0 + 40.0 * (1.0 / 2_000.0) * (campaign_tokens - 2_000.0).min(2_000.0)
        + 50.0 * (1.0 - (-(campaign_tokens - 4_000.0).max(0.0) / 2_500.0).exp())
}

/// Blueberry-speed bonus — gated at `>= 2000`, two-band piecewise.
#[must_use]
pub fn campaign_blueberry_speed_bonus(campaign_tokens: f64) -> f64 {
    if campaign_tokens < 2_000.0 {
        return 1.0;
    }
    1.0 + 0.02 * (1.0 / 2_000.0) * (campaign_tokens - 2_000.0).min(2_000.0)
        + 0.03 * (1.0 - (-(campaign_tokens - 4_000.0).max(0.0) / 2_000.0).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tutorial_zero_tokens_returns_unit_bonuses() {
        let bonus = tutorial_bonus(0.0);
        assert_eq!(bonus.cube_bonus, 1.0);
        assert_eq!(bonus.obtainium_bonus, 1.0);
        assert_eq!(bonus.offering_bonus, 1.0);
    }

    #[test]
    fn tutorial_one_token_activates_bonuses() {
        let bonus = tutorial_bonus(1.0);
        assert_eq!(bonus.cube_bonus, 1.25);
        assert_eq!(bonus.obtainium_bonus, 1.2);
        assert_eq!(bonus.offering_bonus, 1.2);
    }

    #[test]
    fn cube_bonus_below_25_is_linear() {
        // tokens = 12.5 → 1 + 0.4 * 12.5/25 = 1.2
        let result = campaign_cube_bonus(12.5);
        assert!((result - 1.2).abs() < 1e-12);
    }

    #[test]
    fn cube_bonus_grows_with_tokens() {
        let small = campaign_cube_bonus(100.0);
        let big = campaign_cube_bonus(10_000.0);
        assert!(big > small);
    }

    #[test]
    fn time_threshold_reduction_staircase() {
        // 20 → still index 0 (campaign_tokens >= 20 → skip first threshold)
        // 19 → index 0
        assert_eq!(campaign_time_threshold_reduction(19.0), 0.0);
        // 50 → index 1 → 0.25
        assert_eq!(campaign_time_threshold_reduction(50.0), 0.25);
        // 10000 → capped at 2
        assert_eq!(campaign_time_threshold_reduction(10_000.0), 2.0);
    }

    #[test]
    fn quark_bonus_gated_at_100() {
        assert_eq!(campaign_quark_bonus(99.0), 1.0);
        // At exactly 100, the linear band is at 0 progress and the exp
        // terms are zero — gate just opens, value still 1.0. Above the
        // gate, the linear band ramps up.
        assert_eq!(campaign_quark_bonus(100.0), 1.0);
        assert!(campaign_quark_bonus(150.0) > 1.0);
    }

    #[test]
    fn tax_multiplier_decreases_below_one() {
        let result = campaign_tax_multiplier(1_000.0);
        assert!(result < 1.0);
        assert!(result > 0.0);
    }

    #[test]
    fn bonus_rune_6_staircase() {
        assert_eq!(campaign_bonus_rune_6(0.0), 0.0);
        assert_eq!(campaign_bonus_rune_6(500.0), 1.0);
        assert_eq!(campaign_bonus_rune_6(20_000.0), 12.0);
    }

    #[test]
    fn ambrosia_luck_bonus_uses_additive_10_base() {
        assert_eq!(campaign_ambrosia_luck_bonus(1_999.0), 0.0);
        // At 2000: 10 + 0 + 0 = 10
        assert_eq!(campaign_ambrosia_luck_bonus(2_000.0), 10.0);
    }
}
