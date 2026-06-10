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
//!
//! The token *total* (`updateTokens`) lives here too — the static
//! per-campaign `limit`/`isMeta` table, [`campaign_token_value`], and the
//! inheritance / singularity-multiplier grants. The `GameState` assembler
//! (`compute_campaign_tokens`) composes them in the tick.

use crate::state::campaigns::CAMPAIGNS_LEN;

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

// ─── Token total (`updateTokens`) ──────────────────────────────────────────
//
// `campaignTokens` is a derived total in the legacy code (a module global
// recomputed by `updateTokens()`): the sum of every campaign's
// `computeTokenValue()` plus `inheritanceTokens()` plus the GQ
// `singBonusTokens4` / octeract `octeractBonusTokens4` initial-token
// bonuses. Logic derives it live from `campaign_completions` — no cached
// state field.

/// Per-campaign completion `limit`, in `campaignDatas` key order
/// (`Campaign.ts`, `first` = 0 … `fiftieth` = 49; identical in both legacy
/// snapshots).
pub const CAMPAIGN_TOKEN_LIMITS: [f64; CAMPAIGNS_LEN] = [
    10.0, 10.0, 10.0, 10.0, 10.0, // 0-4
    15.0, 15.0, 15.0, 15.0, 15.0, // 5-9
    20.0, 20.0, 20.0, 20.0, 20.0, // 10-14
    25.0, 25.0, 25.0, 25.0, 25.0, // 15-19
    30.0, 30.0, 30.0, 30.0, 30.0, // 20-24
    35.0, 35.0, 35.0, 35.0, 35.0, // 25-29
    40.0, 40.0, // 30-31
    45.0, 45.0, // 32-33
    50.0, 50.0, // 34-35
    55.0, 55.0, // 36-37
    60.0, 60.0, // 38-39
    65.0, 70.0, 75.0, 80.0, 85.0, // 40-44
    95.0, 105.0, 115.0, 125.0, 140.0, // 45-49
];

/// Per-campaign `isMeta` flag (meta campaigns earn doubled tokens), same
/// key order as [`CAMPAIGN_TOKEN_LIMITS`].
pub const CAMPAIGN_IS_META: [bool; CAMPAIGNS_LEN] = [
    false, true, false, false, false, // 0-4
    false, true, false, false, false, // 5-9
    false, true, false, false, true, // 10-14
    false, false, true, false, false, // 15-19
    true, false, false, true, false, // 20-24
    false, false, true, false, true, // 25-29
    true, false, // 30-31
    false, true, // 32-33
    true, false, // 34-35
    false, true, // 36-37
    false, true, // 38-39
    true, true, true, true, true, // 40-44
    true, true, true, false, true, // 45-49
];

/// The cross-campaign bonus terms of `computeTokenValue` — identical for
/// every campaign, so the caller assembles them once. Each folds the
/// matching highest-singularity milestone with the GQ + octeract
/// bonus-token upgrade effects.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CampaignTokenBonuses {
    /// Added once `completed >= 1`: `(highestSing >= 16 ? 5 : 0) +
    /// singBonusTokens1 + octeractBonusTokens3`.
    pub first_completion_bonus: f64,
    /// Added once `completed == limit`: `(highestSing >= 69 ? 10 : 0) +
    /// singBonusTokens3 + octeractBonusTokens1`.
    pub last_completion_bonus: f64,
    /// Shared multiplier: `singularityBonusTokenMult() × singBonusTokens2
    /// × octeractBonusTokens2` (the per-campaign `isMeta` ×2 is applied
    /// inside [`campaign_token_value`]).
    pub token_multiplier: f64,
}

/// One campaign's token value — `Campaign.computeTokenValue()`
/// (`Campaign.ts:538`). `completed = min(c10Completions, limit)` is the
/// additive base; the first/last-completion bonuses join it, and the
/// product of the shared multiplier with the meta ×2 is floored.
#[must_use]
pub fn campaign_token_value(
    c10_completions: f64,
    limit: f64,
    is_meta: bool,
    bonuses: &CampaignTokenBonuses,
) -> f64 {
    let completed = c10_completions.min(limit);
    let mut additive = completed;
    if completed >= 1.0 {
        additive += bonuses.first_completion_bonus;
    }
    if completed == limit {
        additive += bonuses.last_completion_bonus;
    }
    let multiplier = if is_meta { 2.0 } else { 1.0 } * bonuses.token_multiplier;
    (additive * multiplier).floor()
}

/// `inheritanceLevels` — highest-singularity milestones for the
/// inheritance token grant (`Calculate.ts:135`).
const INHERITANCE_LEVELS: [f64; 15] = [
    2.0, 5.0, 10.0, 17.0, 26.0, 37.0, 50.0, 65.0, 82.0, 101.0, 220.0, 240.0, 260.0, 270.0, 277.0,
];

/// `inheritanceTokenValues` — token grant per inheritance tier.
const INHERITANCE_TOKEN_VALUES: [f64; 15] = [
    1.0, 10.0, 25.0, 40.0, 75.0, 100.0, 150.0, 200.0, 250.0, 300.0, 350.0, 400.0, 500.0, 600.0,
    750.0,
];

/// `inheritanceTokens()` (`Calculate.ts:1691`) — the flat token grant from
/// the highest singularity reached.
///
/// Faithful quirk: the legacy loop runs `i = 15` down to `i = 1` over the
/// 15-entry arrays — `i = 15` reads out of bounds (`>= undefined` is
/// false, a no-op) and `i = 0` is never tested, so the `inheritanceLevels[0]
/// = 2 → 1 token` tier is unreachable. The effective floor is singularity
/// 5 → 10 tokens.
#[must_use]
pub fn inheritance_tokens(highest_singularity_count: f64) -> f64 {
    for i in (1..=14).rev() {
        if highest_singularity_count >= INHERITANCE_LEVELS[i] {
            return INHERITANCE_TOKEN_VALUES[i];
        }
    }
    0.0
}

/// `bonusTokenLevels` — highest-singularity milestones for the global
/// token multiplier (`Calculate.ts:138`).
const BONUS_TOKEN_LEVELS: [f64; 5] = [41.0, 58.0, 113.0, 163.0, 229.0];

/// `singularityBonusTokenMult()` (`Calculate.ts:1701`) — `1 + 0.02·i` for
/// the highest tier `i` (1-based) whose milestone the player has reached;
/// `1` below singularity 41.
#[must_use]
pub fn singularity_bonus_token_mult(highest_singularity_count: f64) -> f64 {
    for i in (1..=5).rev() {
        if highest_singularity_count >= BONUS_TOKEN_LEVELS[i - 1] {
            return 1.0 + 0.02 * i as f64;
        }
    }
    1.0
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
    fn token_value_base_is_clamped_completions() {
        let none = CampaignTokenBonuses {
            token_multiplier: 1.0,
            ..Default::default()
        };
        assert_eq!(campaign_token_value(0.0, 10.0, false, &none), 0.0);
        assert_eq!(campaign_token_value(5.0, 10.0, false, &none), 5.0);
        // Clamped to the limit.
        assert_eq!(campaign_token_value(99.0, 10.0, false, &none), 10.0);
    }

    #[test]
    fn token_value_applies_first_last_meta_and_floor() {
        let bonuses = CampaignTokenBonuses {
            first_completion_bonus: 5.0,
            last_completion_bonus: 10.0,
            token_multiplier: 1.02,
        };
        // 1 completion: (1 + 5)·1.02 = 6.12 → floor 6.
        assert_eq!(campaign_token_value(1.0, 10.0, false, &bonuses), 6.0);
        // Full completion: (10 + 5 + 10)·1.02 = 25.5 → floor 25.
        assert_eq!(campaign_token_value(10.0, 10.0, false, &bonuses), 25.0);
        // Meta doubles the multiplier: 25·2·1.02 = 51.
        assert_eq!(campaign_token_value(10.0, 10.0, true, &bonuses), 51.0);
    }

    #[test]
    fn inheritance_tokens_floor_is_singularity_5() {
        // The legacy loop never tests index 0, so the level-2 → 1-token
        // tier is unreachable — faithful quirk.
        assert_eq!(inheritance_tokens(0.0), 0.0);
        assert_eq!(inheritance_tokens(2.0), 0.0);
        assert_eq!(inheritance_tokens(4.0), 0.0);
        assert_eq!(inheritance_tokens(5.0), 10.0);
        assert_eq!(inheritance_tokens(16.0), 25.0);
        assert_eq!(inheritance_tokens(277.0), 750.0);
    }

    #[test]
    fn singularity_bonus_token_mult_tiers() {
        assert_eq!(singularity_bonus_token_mult(0.0), 1.0);
        assert_eq!(singularity_bonus_token_mult(40.0), 1.0);
        assert!((singularity_bonus_token_mult(41.0) - 1.02).abs() < 1e-12);
        assert!((singularity_bonus_token_mult(229.0) - 1.1).abs() < 1e-12);
    }

    #[test]
    fn campaign_tables_cover_all_fifty_campaigns() {
        assert_eq!(CAMPAIGN_TOKEN_LIMITS.len(), CAMPAIGNS_LEN);
        assert_eq!(CAMPAIGN_IS_META.len(), CAMPAIGNS_LEN);
        // Spot anchors: first (10, not meta), second (10, meta),
        // forty-ninth (125, not meta), fiftieth (140, meta).
        assert_eq!(CAMPAIGN_TOKEN_LIMITS[0], 10.0);
        assert!(!CAMPAIGN_IS_META[0]);
        assert!(CAMPAIGN_IS_META[1]);
        assert_eq!(CAMPAIGN_TOKEN_LIMITS[48], 125.0);
        assert!(!CAMPAIGN_IS_META[48]);
        assert_eq!(CAMPAIGN_TOKEN_LIMITS[49], 140.0);
        assert!(CAMPAIGN_IS_META[49]);
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
