//! Reborn-ELO stages + per-stage modifiers + total-production math
//! + the two ant-related singularity perks (ELO bonus mult &
//!   invigorated-spirits ELO gift).
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/antRebornELO.ts`.
//! The reborn-speed-mult-stats reducer and the leaderboard/quark
//! math that depends on player.worlds.applyBonus() stay in the UI
//! tier because they couple to stats arrays / external bonuses.

// ─── Tranches & constants ─────────────────────────────────────────────────

/// One row of the reborn-ELO tranche table. Each tranche covers
/// `stages` levels; reaching a stage inside the tranche costs
/// `per_stage` ELO and awards `quark_per_stage` lifetime quarks
/// (used by the daily-quark reward).
#[derive(Debug, Clone, Copy)]
pub struct RebornELOTranche {
    /// How many stages this tranche covers. `f64::INFINITY` for the
    /// terminal open-ended tranche.
    pub stages: f64,
    /// ELO required to advance one stage inside this tranche.
    pub per_stage: f64,
    /// Per-stage quark reward inside this tranche.
    pub quark_per_stage: f64,
}

/// Reborn-ELO tranches. Each tranche covers `stages` levels; each
/// level inside the tranche costs `per_stage` ELO; reaching it
/// grants `quark_per_stage` lifetime quarks.
pub const REBORN_ELO_THRESHOLD_TRANCHES: &[RebornELOTranche] = &[
    RebornELOTranche {
        stages: 100.0,
        per_stage: 100.0,
        quark_per_stage: 1.0,
    },
    RebornELOTranche {
        stages: 100.0,
        per_stage: 1_000.0,
        quark_per_stage: 2.0,
    },
    RebornELOTranche {
        stages: 100.0,
        per_stage: 3_000.0,
        quark_per_stage: 3.0,
    },
    RebornELOTranche {
        stages: 700.0,
        per_stage: 20_000.0,
        quark_per_stage: 4.0,
    },
    RebornELOTranche {
        stages: f64::INFINITY,
        per_stage: 100_000.0,
        quark_per_stage: 7.0,
    },
];

// Compile-time invariant: the terminal tranche covers all remaining ELO
// via `stages = INFINITY`. Every loop over this table relies on this to
// guarantee the search terminates; if the terminator is ever removed,
// the `unreachable!()` in `calculate_to_next_reborn_elo_threshold`
// becomes reachable and this assertion fires at build time instead.
const _: () = {
    let last_idx = REBORN_ELO_THRESHOLD_TRANCHES.len() - 1;
    let last_stages = REBORN_ELO_THRESHOLD_TRANCHES[last_idx].stages;
    assert!(last_stages.to_bits() == f64::INFINITY.to_bits());
};

/// Multiplier applied per stage to the daily-quark reward.
pub const QUARK_MULTIPLIER_PER_REBORN_ELO_THRESHOLD: f64 = 1.002;

/// Base per-stage modifier values (before exponentiation by stage
/// count).
pub mod per_reborn_elo_stage_modifiers {
    /// Reborn-speed multiplier base; combines with singularity perk.
    pub const REBORN_SPEED_MULT: f64 = 0.98;
    /// Obtainium per-stage base.
    pub const ANT_SACRIFICE_OBTAINIUM_MULT: f64 = 1.05;
    /// Offering per-stage base.
    pub const ANT_SACRIFICE_OFFERING_MULT: f64 = 1.05;
    /// Talisman-fragment per-stage base.
    pub const ANT_SACRIFICE_TALISMAN_FRAGMENT_MULT: f64 = 1.2;
}

/// Singularity counts at which the reborn-speed-perk tier advances.
/// Index `i` means: at `sing_count >= levels[i]`, tier `i` is
/// active.
pub const REBORN_SPEED_PERK_LEVELS: &[f64] = &[
    1.0, 9.0, 25.0, 49.0, 81.0, 121.0, 169.0, 196.0, 225.0, 256.0, 289.0,
];

/// Singularity counts at which the ELO-bonus-mult perk tier
/// advances.
pub const SINGULARITY_ELO_BONUS_MULT_LEVELS: &[f64] = &[
    3.0, 11.0, 27.0, 51.0, 83.0, 123.0, 171.0, 198.0, 227.0, 258.0, 291.0,
];

/// Singularity counts at which the invigorated-spirits ELO perk
/// tier advances.
pub const SINGULARITY_PERK_ELO_LEVELS: &[f64] = &[
    2.0, 10.0, 26.0, 50.0, 82.0, 122.0, 170.0, 197.0, 226.0, 257.0, 290.0,
];

// ─── Stage / speed math ───────────────────────────────────────────────────

/// Reborn-speed perk modifier — staircase lookup over the perk
/// levels. Each tier above the base adds `0.00009`; the base tier
/// starts at `0.0001`. Returns `0` for `sing_count` below the first
/// level.
#[must_use]
pub fn singularity_reborn_speed_mult_modifier(sing_count: f64) -> f64 {
    for (i, &level) in REBORN_SPEED_PERK_LEVELS.iter().enumerate().rev() {
        if sing_count >= level {
            return 0.0001 + 0.00009 * (i as f64);
        }
    }
    0.0
}

/// Per-stage reborn-speed multiplier. Floor of `1` to avoid the
/// perk making stages free. Base `0.98 + singularity-perk increase`.
#[must_use]
pub fn calculate_stage_reborn_speed_mult(sing_count: f64) -> f64 {
    let base = per_reborn_elo_stage_modifiers::REBORN_SPEED_MULT;
    let increase = singularity_reborn_speed_mult_modifier(sing_count);
    1.0_f64.min(base + increase)
}

// ─── Threshold (stage-count) computation ──────────────────────────────────

/// How many reborn-ELO stages have been reached for a given ELO
/// total. Walks the tranche list, consuming ELO at each tranche's
/// per-stage cost until either ELO runs out or the tranche is
/// exhausted.
#[must_use]
pub fn calculate_reborn_elo_thresholds(reborn_elo: f64) -> f64 {
    let mut budget = reborn_elo;
    let mut thresholds = 0.0_f64;
    for tranche in REBORN_ELO_THRESHOLD_TRANCHES {
        let stages_added = tranche.stages.min((budget / tranche.per_stage).floor());
        thresholds += stages_added;
        budget -= stages_added * tranche.per_stage;
        if stages_added < tranche.stages {
            break;
        }
    }
    thresholds
}

/// ELO required to reach the *next* stage boundary, given current
/// ELO. `stage` short-circuits the inner re-calculation if the
/// caller already has it; pass `None` to compute it inline.
///
/// # Panics
/// Panics if the tranche list is exhausted before finding the next
/// boundary, which is unreachable because the terminal tranche has
/// `stages = INFINITY`.
#[must_use]
pub fn calculate_to_next_reborn_elo_threshold(reborn_elo: f64, stage: Option<f64>) -> f64 {
    let thresholds = stage.unwrap_or_else(|| calculate_reborn_elo_thresholds(reborn_elo));
    let mut stages_checked = 0.0_f64;
    let mut temp_elo = reborn_elo;
    for tranche in REBORN_ELO_THRESHOLD_TRANCHES {
        if thresholds < stages_checked + tranche.stages {
            let req_elo_this_threshold = tranche.per_stage;
            return (1.0 + (temp_elo / req_elo_this_threshold).floor()) * req_elo_this_threshold
                - temp_elo;
        }
        stages_checked += tranche.stages;
        temp_elo -= tranche.stages * tranche.per_stage;
    }
    unreachable!("calculate_to_next_reborn_elo_threshold ran off the tranche list");
}

/// ELO that's been accumulated but doesn't yet contribute to a
/// completed stage (progress within the current stage boundary).
#[must_use]
pub fn calculate_leftover_reborn_elo(reborn_elo: f64, stage: Option<f64>) -> f64 {
    let thresholds = stage.unwrap_or_else(|| calculate_reborn_elo_thresholds(reborn_elo));
    let mut used_elo = 0.0_f64;
    let mut stages_checked = 0.0_f64;
    for tranche in REBORN_ELO_THRESHOLD_TRANCHES {
        let stages_in_tranche = tranche.stages.min(thresholds - stages_checked);
        used_elo += stages_in_tranche * tranche.per_stage;
        stages_checked += stages_in_tranche;
        if stages_checked >= thresholds {
            break;
        }
    }
    reborn_elo - used_elo
}

// ─── Stage-modifier aggregator ────────────────────────────────────────────

/// Per-stage multipliers raised to the current stage count.
#[derive(Debug, Clone, Copy)]
pub struct RebornELOStageModifiers {
    /// Reborn-speed mult (compounds at the stage rate, which itself
    /// depends on the singularity perk).
    pub reborn_speed_mult: f64,
    /// Obtainium per-stage cumulative mult.
    pub ant_sacrifice_obtainium_mult: f64,
    /// Offering per-stage cumulative mult.
    pub ant_sacrifice_offering_mult: f64,
    /// Talisman-fragment per-stage cumulative mult.
    pub ant_sacrifice_talisman_fragment_mult: f64,
}

/// Inputs to [`reborn_elo_stage_modifiers`].
#[derive(Debug, Clone, Copy)]
pub struct RebornELOStageModifiersInput {
    /// `player.ants.rebornELO`.
    pub reborn_elo: f64,
    /// `player.singularityCount`.
    pub sing_count: f64,
}

/// Per-stage multipliers raised to the current stage count — the
/// cumulative effect of all reached thresholds.
#[must_use]
pub fn reborn_elo_stage_modifiers(input: &RebornELOStageModifiersInput) -> RebornELOStageModifiers {
    let thresholds = calculate_reborn_elo_thresholds(input.reborn_elo);
    RebornELOStageModifiers {
        reborn_speed_mult: calculate_stage_reborn_speed_mult(input.sing_count).powf(thresholds),
        ant_sacrifice_obtainium_mult: per_reborn_elo_stage_modifiers::ANT_SACRIFICE_OBTAINIUM_MULT
            .powf(thresholds),
        ant_sacrifice_offering_mult: per_reborn_elo_stage_modifiers::ANT_SACRIFICE_OFFERING_MULT
            .powf(thresholds),
        ant_sacrifice_talisman_fragment_mult:
            per_reborn_elo_stage_modifiers::ANT_SACRIFICE_TALISMAN_FRAGMENT_MULT.powf(thresholds),
    }
}

// ─── Available / total-production helpers ─────────────────────────────────

/// Inputs to [`calculate_available_reborn_elo`].
#[derive(Debug, Clone, Copy)]
pub struct AvailableRebornELOInput {
    /// `player.ants.immortalELO`.
    pub immortal_elo: f64,
    /// `player.ants.rebornELO`.
    pub reborn_elo: f64,
}

/// Immortal ELO that has not yet been activated as Reborn ELO.
/// Floor at 0 (rebornELO can technically exceed immortalELO
/// mid-frame).
#[must_use]
pub fn calculate_available_reborn_elo(input: &AvailableRebornELOInput) -> f64 {
    (input.immortal_elo - input.reborn_elo).max(0.0)
}

/// Closed-form sum `r^startIndex + r^(startIndex+1) + … + r^endIndex`.
/// When `r == 1`, returns the trivial linear sum. Returns `0` when
/// `endIndex < startIndex`.
fn geometric_series(start_index: f64, end_index: f64, ratio: f64) -> f64 {
    if end_index < start_index {
        return 0.0;
    }
    if (ratio - 1.0).abs() < f64::EPSILON {
        return end_index - start_index + 1.0;
    }
    (ratio.powf(end_index + 1.0) - ratio.powf(start_index)) / (ratio - 1.0)
}

/// Inputs to [`calculate_total_production_for_reborn_elo`].
#[derive(Debug, Clone, Copy)]
pub struct TotalProductionForRebornELOInput {
    /// `player.ants.rebornELO`.
    pub reborn_elo: f64,
    /// `calculate_stage_reborn_speed_mult(sing_count)` — the
    /// per-stage speed mult.
    pub stage_reborn_speed_mult: f64,
}

/// Total ELO-equivalent production required to reach `reborn_elo`.
/// Each tranche contributes a geometric-series sum: each stage
/// costs `per_stage` production weighted by
/// `(1 / stage_reborn_speed_mult)^stage_index`.
#[must_use]
pub fn calculate_total_production_for_reborn_elo(input: &TotalProductionForRebornELOInput) -> f64 {
    let stage = calculate_reborn_elo_thresholds(input.reborn_elo);
    let leftover = calculate_leftover_reborn_elo(input.reborn_elo, Some(stage));

    // Reciprocal: you need 1/modifier times as much production to
    // get the same ELO/sec.
    let per_stage_mult = 1.0 / input.stage_reborn_speed_mult;

    let mut production = 0.0_f64;
    let mut stages_spent = 0.0_f64;
    for tranche in REBORN_ELO_THRESHOLD_TRANCHES {
        let start_index = stages_spent;
        let stages_in_tranche = tranche.stages.min(stage - stages_spent);
        let end_index = stages_spent + stages_in_tranche - 1.0;
        let production_this_tranche =
            geometric_series(start_index, end_index, per_stage_mult) * tranche.per_stage;
        production += production_this_tranche;
        stages_spent += stages_in_tranche;
        if stages_spent >= stage {
            production += leftover * per_stage_mult.powf(stage);
            break;
        }
    }
    production
}

// ─── Leaderboard + daily-quark math ───────────────────────────────────────

/// Per-rank multipliers applied to the top-N daily/all-time
/// reborn-ELO leaderboard entries.
pub const LEADERBOARD_WEIGHTS: &[f64] = &[1.0, 0.8, 0.6, 0.4, 0.2];

/// Weighted-sum of a leaderboard ELO slice, floor'd to an integer.
/// Walks at most `min(leaderboard.len(), LEADERBOARD_WEIGHTS.len())`
/// entries.
#[must_use]
pub fn calculate_leaderboard_value(leaderboard_elos: &[f64]) -> f64 {
    let n = leaderboard_elos.len().min(LEADERBOARD_WEIGHTS.len());
    let mut total = 0.0_f64;
    for i in 0..n {
        total += leaderboard_elos[i] * LEADERBOARD_WEIGHTS[i];
    }
    total.floor()
}

/// Lifetime-ELO quark multiplier — sigmoid-ish ramp
/// `2 − 0.8^(stages/100)`, asymptoting at 2× as the player
/// accumulates reborn stages.
#[must_use]
pub fn quarks_from_elo_mult(lifetime_leaderboard_elo: f64) -> f64 {
    let num_stages = calculate_reborn_elo_thresholds(lifetime_leaderboard_elo);
    2.0 - 0.8_f64.powf(num_stages / 100.0)
}

/// Result of [`base_quarks_from_reborn_elo_stages`].
#[derive(Debug, Clone, Copy)]
pub struct BaseQuarksFromRebornELOStagesResult {
    /// Sum of per-stage quark rewards across tranches.
    pub base_quarks: f64,
    /// Per-stage quark multiplier (capped at 1000 stages contributing).
    pub stage_mult: f64,
}

/// Per-tranche base-quark loop. Walks each tranche, accumulating
/// `min(tranche.stages, remaining) × quark_per_stage` until stages
/// are exhausted. Separately computes the stage multiplier as
/// `quark_multiplier_per_reborn_elo_threshold ^ min(num_stages, 1000)`.
#[must_use]
pub fn base_quarks_from_reborn_elo_stages(num_stages: f64) -> BaseQuarksFromRebornELOStagesResult {
    let mut base_quarks = 0.0_f64;
    let mut remaining = num_stages;
    for tranche in REBORN_ELO_THRESHOLD_TRANCHES {
        let stages_in_tranche = tranche.stages.min(remaining);
        base_quarks += stages_in_tranche * tranche.quark_per_stage;
        remaining -= stages_in_tranche;
        if remaining <= 0.0 {
            break;
        }
    }
    let used = num_stages.min(1_000.0);
    let stage_mult = QUARK_MULTIPLIER_PER_REBORN_ELO_THRESHOLD.powf(used);
    BaseQuarksFromRebornELOStagesResult {
        base_quarks,
        stage_mult,
    }
}

// ─── Ant-related singularity perks ────────────────────────────────────────

/// "Advanced... Cheating Tactics?" perk — additive ELO multiplier
/// from singularity. Staircase: `0.001` base, `+0.0009` per tier.
#[must_use]
pub fn singularity_elo_bonus_mult(sing_count: f64) -> f64 {
    for (i, &level) in SINGULARITY_ELO_BONUS_MULT_LEVELS.iter().enumerate().rev() {
        if sing_count >= level {
            return 0.001 + 0.0009 * (i as f64);
        }
    }
    0.0
}

/// Inputs to [`calculate_singularity_perk_elo`].
#[derive(Debug, Clone, Copy)]
pub struct SingularityPerkELOInput {
    /// `player.singularityCount`.
    pub sing_count: f64,
    /// `player.ants.immortalELO`.
    pub immortal_elo: f64,
}

/// "Invigorated Spirits!" perk — flat ELO gift scaled by
/// immortal-ELO bands. First 200,000 immortal ELO contributes at
/// the higher per-unit rate; the next 1,800,000 at the lower rate;
/// anything above 2,000,000 is ignored.
#[must_use]
pub fn calculate_singularity_perk_elo(input: &SingularityPerkELOInput) -> f64 {
    for (i, &level) in SINGULARITY_PERK_ELO_LEVELS.iter().enumerate().rev() {
        if input.sing_count >= level {
            let first_tranch_mult = 0.02 + 0.018 * (i as f64);
            let second_tranch_mult = 0.001 + 0.0009 * (i as f64);
            let immortal = input.immortal_elo;
            return 200_000.0_f64.min(immortal) * first_tranch_mult
                + 0.0_f64.max(1_800_000.0_f64.min(immortal - 200_000.0)) * second_tranch_mult;
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singularity_reborn_speed_mult_below_first_level_is_zero() {
        assert_eq!(singularity_reborn_speed_mult_modifier(0.0), 0.0);
    }

    #[test]
    fn singularity_reborn_speed_mult_at_first_level_is_0p0001() {
        assert!((singularity_reborn_speed_mult_modifier(1.0) - 0.0001).abs() < 1e-12);
    }

    #[test]
    fn singularity_reborn_speed_mult_staircases() {
        // At level index 5 (sing 121): 0.0001 + 0.00009*5 = 0.00055
        assert!((singularity_reborn_speed_mult_modifier(121.0) - 0.000_55).abs() < 1e-12);
    }

    #[test]
    fn stage_reborn_speed_mult_floors_at_one() {
        // Even with very high singCount, base 0.98 + perk increase
        // never exceeds 1; min clamps it anyway.
        let result = calculate_stage_reborn_speed_mult(1e6);
        assert!(result <= 1.0);
    }

    #[test]
    fn reborn_elo_thresholds_zero_for_zero_elo() {
        assert_eq!(calculate_reborn_elo_thresholds(0.0), 0.0);
    }

    #[test]
    fn reborn_elo_thresholds_first_tranche() {
        // 100 elo → 1 stage; 9999 elo → 99 stages
        assert_eq!(calculate_reborn_elo_thresholds(100.0), 1.0);
        assert_eq!(calculate_reborn_elo_thresholds(9_999.0), 99.0);
    }

    #[test]
    fn reborn_elo_thresholds_into_second_tranche() {
        // First tranche fully exhausted at 100*100 = 10,000 → 100 stages
        // Then second tranche: 1000 elo per stage. 12,500 leftover → 12 more
        // Total = 112
        let result = calculate_reborn_elo_thresholds(10_000.0 + 12_500.0);
        assert_eq!(result, 112.0);
    }

    #[test]
    fn to_next_threshold_at_zero_is_first_tranche_cost() {
        let result = calculate_to_next_reborn_elo_threshold(0.0, None);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn to_next_threshold_mid_stage_is_remainder() {
        // 50 elo → next stage at 100 → need 50 more
        let result = calculate_to_next_reborn_elo_threshold(50.0, None);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn leftover_reborn_elo_within_stage() {
        // 150 elo → 1 stage (cost 100) → leftover 50
        let result = calculate_leftover_reborn_elo(150.0, None);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn available_reborn_elo_clamps_at_zero() {
        let result = calculate_available_reborn_elo(&AvailableRebornELOInput {
            immortal_elo: 100.0,
            reborn_elo: 200.0,
        });
        assert_eq!(result, 0.0);
    }

    #[test]
    fn available_reborn_elo_returns_delta() {
        let result = calculate_available_reborn_elo(&AvailableRebornELOInput {
            immortal_elo: 500.0,
            reborn_elo: 200.0,
        });
        assert_eq!(result, 300.0);
    }

    #[test]
    fn stage_modifiers_at_zero_elo_are_ones() {
        let result = reborn_elo_stage_modifiers(&RebornELOStageModifiersInput {
            reborn_elo: 0.0,
            sing_count: 0.0,
        });
        // x^0 = 1
        assert_eq!(result.reborn_speed_mult, 1.0);
        assert_eq!(result.ant_sacrifice_obtainium_mult, 1.0);
        assert_eq!(result.ant_sacrifice_offering_mult, 1.0);
        assert_eq!(result.ant_sacrifice_talisman_fragment_mult, 1.0);
    }

    #[test]
    fn leaderboard_value_weighted_sum_floored() {
        let elos = [1_000.0, 500.0, 200.0];
        // 1000*1 + 500*0.8 + 200*0.6 = 1520
        assert_eq!(calculate_leaderboard_value(&elos), 1_520.0);
    }

    #[test]
    fn quarks_from_elo_mult_asymptotes_at_2() {
        let huge = quarks_from_elo_mult(1e9);
        assert!(huge > 1.99 && huge < 2.0);
    }

    #[test]
    fn base_quarks_first_tranche_only() {
        // 50 stages, first tranche = 1 quark/stage → 50 quarks
        let result = base_quarks_from_reborn_elo_stages(50.0);
        assert_eq!(result.base_quarks, 50.0);
    }

    #[test]
    fn base_quarks_into_second_tranche() {
        // 150 stages: first 100 × 1 + next 50 × 2 = 200
        let result = base_quarks_from_reborn_elo_stages(150.0);
        assert_eq!(result.base_quarks, 200.0);
    }

    #[test]
    fn base_quarks_stage_mult_caps_at_1000() {
        let at_1000 = base_quarks_from_reborn_elo_stages(1_000.0).stage_mult;
        let at_5000 = base_quarks_from_reborn_elo_stages(5_000.0).stage_mult;
        assert!((at_1000 - at_5000).abs() < 1e-9);
    }

    #[test]
    fn singularity_elo_bonus_mult_below_first_is_zero() {
        assert_eq!(singularity_elo_bonus_mult(0.0), 0.0);
    }

    #[test]
    fn singularity_elo_bonus_mult_at_first_level_is_0p001() {
        assert!((singularity_elo_bonus_mult(3.0) - 0.001).abs() < 1e-12);
    }

    #[test]
    fn singularity_perk_elo_below_first_level_is_zero() {
        let result = calculate_singularity_perk_elo(&SingularityPerkELOInput {
            sing_count: 0.0,
            immortal_elo: 1_000_000.0,
        });
        assert_eq!(result, 0.0);
    }

    #[test]
    fn singularity_perk_elo_first_tranch() {
        // sing=2 → tier 0 → firstTranch=0.02, secondTranch=0.001
        // immortal=100,000 (all in first band) → 100,000 * 0.02 = 2,000
        let result = calculate_singularity_perk_elo(&SingularityPerkELOInput {
            sing_count: 2.0,
            immortal_elo: 100_000.0,
        });
        assert!((result - 2_000.0).abs() < 1e-9);
    }

    #[test]
    fn singularity_perk_elo_above_2m_caps() {
        // sing=2, immortal=3M → 200k*0.02 + 1.8M*0.001 = 4000 + 1800 = 5800
        let result = calculate_singularity_perk_elo(&SingularityPerkELOInput {
            sing_count: 2.0,
            immortal_elo: 3_000_000.0,
        });
        assert!((result - 5_800.0).abs() < 1e-9);
    }
}
