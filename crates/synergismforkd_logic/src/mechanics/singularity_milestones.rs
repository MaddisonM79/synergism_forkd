//! Singularity-milestone bonuses.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/singularityMilestones.ts`.
//! Every function in this module follows the same shape: read a
//! single (or two) counter from the player — current
//! `singularityCount` or all-time `highestSingularityCount` —
//! count how many entries of a hardcoded threshold array have been
//! crossed, and return a numeric bonus. The threshold arrays are
//! data (not state) and live here alongside the formulas.

const SING_QUARK_MILESTONE_THRESHOLDS: &[f64] = &[
    5.0, 7.0, 10.0, 20.0, 35.0, 50.0, 65.0, 80.0, 90.0, 100.0, 121.0, 144.0, 150.0, 160.0, 166.0,
    169.0, 170.0, 175.0, 180.0, 190.0, 196.0, 200.0, 201.0, 202.0, 203.0, 204.0, 205.0, 210.0,
    213.0, 216.0, 219.0, 225.0, 228.0, 231.0, 234.0, 237.0, 240.0, 244.0, 248.0, 252.0, 256.0,
    260.0, 264.0, 268.0, 272.0, 276.0, 280.0, 284.0, 288.0, 290.0,
];

const AMBROSIA_LUCK_SING_THRESHOLDS_1: &[f64] = &[35.0, 42.0, 49.0, 56.0, 63.0, 70.0, 77.0];
const AMBROSIA_LUCK_SING_THRESHOLDS_2: &[f64] = &[135.0, 142.0, 149.0, 156.0, 163.0, 170.0, 177.0];

const DERPSMITH_SING_COUNTS: &[f64] = &[
    18.0, 38.0, 58.0, 78.0, 88.0, 98.0, 118.0, 148.0, 178.0, 188.0, 198.0, 208.0, 218.0, 228.0,
    238.0, 248.0,
];

const IMMACULATE_ALCHEMY_THRESHOLDS: &[f64] = &[
    50.0, 90.0, 130.0, 170.0, 200.0, 217.0, 235.0, 253.0, 271.0, 289.0,
];

const INHERITANCE_LEVELS: &[f64] = &[
    2.0, 5.0, 10.0, 17.0, 26.0, 37.0, 50.0, 65.0, 82.0, 101.0, 220.0, 240.0, 260.0, 270.0, 277.0,
];
const INHERITANCE_TOKEN_VALUES: &[f64] = &[
    1.0, 10.0, 25.0, 40.0, 75.0, 100.0, 150.0, 200.0, 250.0, 300.0, 350.0, 400.0, 500.0, 600.0,
    750.0,
];

const BONUS_TOKEN_LEVELS: &[f64] = &[41.0, 58.0, 113.0, 163.0, 229.0];

const DILATED_FIVE_LEAF_SING_THRESHOLDS: &[f64] = &[
    100.0, 150.0, 200.0, 225.0, 250.0, 255.0, 260.0, 265.0, 269.0, 272.0,
];

// ─── Quark milestone multiplier ───────────────────────────────────────────

/// Compounds a `1.05×` multiplier for every entry of the quark
/// milestone threshold list crossed by the current
/// `singularity_count`. Resets with the singularity; uses the live
/// count, not the all-time max.
#[must_use]
pub fn calculate_singularity_quark_milestone_multiplier(singularity_count: f64) -> f64 {
    let mut multiplier = 1.0_f64;
    for &sing in SING_QUARK_MILESTONE_THRESHOLDS {
        if singularity_count >= sing {
            multiplier *= 1.05;
        }
    }
    multiplier
}

// ─── Base golden quarks earned at a singularity ───────────────────────────

/// Inputs to [`calculate_base_golden_quarks`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateBaseGoldenQuarksInput {
    /// The singularity count being entered — exponent base for the
    /// minimum value.
    pub singularity: f64,
    /// `player.quarksThisSingularity` — `/ 1e5` contribution.
    pub quarks_this_singularity: f64,
    /// `player.highestSingularityCount` — capped at 10 for the
    /// first-ten `+10` each bonus.
    pub highest_singularity_count: f64,
}

/// Base GQ award before any milestone / shop / GQ-upgrade
/// multipliers.
///
/// `floor(100 × 1.04^singularity + quarksThisSingularity/1e5 + 10
/// × min(highest, 10))`.
#[must_use]
pub fn calculate_base_golden_quarks(input: &CalculateBaseGoldenQuarksInput) -> f64 {
    let minimum_value = 100.0 * 1.04_f64.powf(input.singularity);
    let contribution_from_quarks = input.quarks_this_singularity / 1e5;
    let first_ten_bonus = 10.0 * input.highest_singularity_count.min(10.0);
    (minimum_value + contribution_from_quarks + first_ten_bonus).floor()
}

// ─── Ambrosia luck milestone bonus ────────────────────────────────────────

/// Additive ambrosia-luck bonus from two singularity-count
/// threshold tables: `+5` per entry of `thresholds_1` crossed, `+6`
/// per entry of `thresholds_2` crossed. Uses the all-time max count.
#[must_use]
pub fn calculate_singularity_ambrosia_luck_milestone_bonus(highest_singularity_count: f64) -> f64 {
    let mut bonus = 0.0_f64;
    for &sing in AMBROSIA_LUCK_SING_THRESHOLDS_1 {
        if highest_singularity_count >= sing {
            bonus += 5.0;
        }
    }
    for &sing in AMBROSIA_LUCK_SING_THRESHOLDS_2 {
        if highest_singularity_count >= sing {
            bonus += 6.0;
        }
    }
    bonus
}

// ─── Dilated Five Leaf bonus ──────────────────────────────────────────────

/// Returns the fraction (`0.00`–`0.10`) representing how many of
/// the dilated-five-leaf thresholds have been crossed by the
/// all-time max singularity count. The first un-crossed threshold's
/// index `/ 100` is returned; if all are crossed, returns
/// `thresholds.len() / 100`.
#[must_use]
pub fn calculate_dilated_five_leaf_bonus(highest_singularity_count: f64) -> f64 {
    for (i, &threshold) in DILATED_FIVE_LEAF_SING_THRESHOLDS.iter().enumerate() {
        if highest_singularity_count < threshold {
            return (i as f64) / 100.0;
        }
    }
    (DILATED_FIVE_LEAF_SING_THRESHOLDS.len() as f64) / 100.0
}

// ─── Derpsmith Cornucopia ─────────────────────────────────────────────────

/// `1 + (count_of_thresholds_crossed × highestSingularityCount) /
/// 100`. The count grows in coarse steps from the
/// derpsmith-sing-counts table; the per-count weight is the
/// all-time max singularity count itself.
#[must_use]
pub fn derpsmith_cornucopia_bonus(highest_singularity_count: f64) -> f64 {
    let mut counter = 0.0_f64;
    for &sing in DERPSMITH_SING_COUNTS {
        if highest_singularity_count >= sing {
            counter += 1.0;
        }
    }
    1.0 + (counter * highest_singularity_count) / 100.0
}

// ─── Immaculate Alchemy ───────────────────────────────────────────────────

/// `1 + 0.4` per immaculate-alchemy threshold crossed by the
/// current `singularity_count` (not the all-time max).
#[must_use]
pub fn calculate_immaculate_alchemy_bonus(singularity_count: f64) -> f64 {
    let mut bonus = 1.0_f64;
    for &threshold in IMMACULATE_ALCHEMY_THRESHOLDS {
        if singularity_count >= threshold {
            bonus += 0.4;
        }
    }
    bonus
}

// ─── Inheritance Tokens ───────────────────────────────────────────────────

/// Returns the inheritance-token value for the highest level the
/// player has crossed (`1 ≤ i ≤ 14`). Returns `0` if nothing
/// crossed. The legacy loop iterates `i = 15..=1` but
/// `inheritanceLevels[15]` is `undefined` (JS) and never matches —
/// the effective range is `1..=14`. Index `0` is unused.
#[must_use]
pub fn inheritance_tokens(highest_singularity_count: f64) -> f64 {
    for i in (1..INHERITANCE_LEVELS.len()).rev() {
        if highest_singularity_count >= INHERITANCE_LEVELS[i] {
            return INHERITANCE_TOKEN_VALUES[i];
        }
    }
    0.0
}

// ─── Sum of exalt completions ─────────────────────────────────────────────

/// Sum of `.completions` across every singularity challenge.
#[must_use]
pub fn sum_of_exalt_completions(completions_list: &[f64]) -> f64 {
    completions_list.iter().sum()
}

// ─── Singularity bonus token mult ─────────────────────────────────────────

/// Returns `1 + 0.02 × i`, where `i` is the highest index `1..5`
/// such that the player's all-time max singularity count is `>=
/// BONUS_TOKEN_LEVELS[i - 1]`.
#[must_use]
pub fn singularity_bonus_token_mult(highest_singularity_count: f64) -> f64 {
    for i in (1..=5_usize).rev() {
        if highest_singularity_count >= BONUS_TOKEN_LEVELS[i - 1] {
            return 1.0 + 0.02 * (i as f64);
        }
    }
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quark_milestone_multiplier_at_zero_is_one() {
        assert_eq!(calculate_singularity_quark_milestone_multiplier(0.0), 1.0);
    }

    #[test]
    fn quark_milestone_multiplier_compounds() {
        // sing=5 crosses 1 threshold → 1.05
        let result = calculate_singularity_quark_milestone_multiplier(5.0);
        assert!((result - 1.05).abs() < 1e-12);
    }

    #[test]
    fn quark_milestone_multiplier_at_huge_sing_compounds_all() {
        let result = calculate_singularity_quark_milestone_multiplier(1_000.0);
        let expected = 1.05_f64.powi(SING_QUARK_MILESTONE_THRESHOLDS.len() as i32);
        assert!((result - expected).abs() / expected < 1e-9);
    }

    #[test]
    fn base_golden_quarks_first_ten_bonus_caps_at_100() {
        let result = calculate_base_golden_quarks(&CalculateBaseGoldenQuarksInput {
            singularity: 0.0,
            quarks_this_singularity: 0.0,
            highest_singularity_count: 100.0,
        });
        // 100 * 1.04^0 + 0 + 10*10 = 100 + 100 = 200
        assert_eq!(result, 200.0);
    }

    #[test]
    fn ambrosia_luck_milestone_increments_at_thresholds() {
        // sing=35 → +5 from first table only → 5
        assert_eq!(
            calculate_singularity_ambrosia_luck_milestone_bonus(35.0),
            5.0
        );
        // sing=135 → all 7 of first (+35) + 1 of second (+6) = 41
        assert_eq!(
            calculate_singularity_ambrosia_luck_milestone_bonus(135.0),
            41.0
        );
    }

    #[test]
    fn dilated_five_leaf_below_first_is_zero() {
        assert_eq!(calculate_dilated_five_leaf_bonus(50.0), 0.0);
    }

    #[test]
    fn dilated_five_leaf_at_100_is_0p01() {
        assert!((calculate_dilated_five_leaf_bonus(100.0) - 0.01).abs() < 1e-12);
    }

    #[test]
    fn dilated_five_leaf_max_at_full_crossing() {
        assert!((calculate_dilated_five_leaf_bonus(1_000.0) - 0.10).abs() < 1e-12);
    }

    #[test]
    fn derpsmith_cornucopia_at_zero_is_one() {
        assert_eq!(derpsmith_cornucopia_bonus(0.0), 1.0);
    }

    #[test]
    fn derpsmith_cornucopia_at_18_increments_counter() {
        // count=1 * sing=18 / 100 = 0.18 → 1.18
        let result = derpsmith_cornucopia_bonus(18.0);
        assert!((result - 1.18).abs() < 1e-12);
    }

    #[test]
    fn immaculate_alchemy_at_zero_is_one() {
        assert_eq!(calculate_immaculate_alchemy_bonus(0.0), 1.0);
    }

    #[test]
    fn immaculate_alchemy_at_50_is_1p4() {
        assert!((calculate_immaculate_alchemy_bonus(50.0) - 1.4).abs() < 1e-12);
    }

    #[test]
    fn inheritance_tokens_returns_correct_tier() {
        // sing=4 → just past level[0]=2 → ?
        // legacy loop: for i=15..1, returns inheritanceTokenValues[i] when
        // crossed. At sing=4: only i=1 (level=5) is not crossed; i=0 (level=2)
        // IS crossed but the loop never checks i=0. So we need sing >= 5 to
        // return 1. sing=4 → 0.
        assert_eq!(inheritance_tokens(4.0), 0.0);
        assert_eq!(inheritance_tokens(5.0), 10.0); // index 1
    }

    #[test]
    fn inheritance_tokens_max_tier() {
        // sing=300 → crosses all up to index 14 (level=277), returns 750
        assert_eq!(inheritance_tokens(300.0), 750.0);
    }

    #[test]
    fn sum_of_exalt_completions_empty_is_zero() {
        assert_eq!(sum_of_exalt_completions(&[]), 0.0);
    }

    #[test]
    fn sum_of_exalt_completions_adds() {
        assert_eq!(sum_of_exalt_completions(&[1.0, 2.0, 3.0]), 6.0);
    }

    #[test]
    fn bonus_token_mult_below_first_is_one() {
        assert_eq!(singularity_bonus_token_mult(40.0), 1.0);
    }

    #[test]
    fn bonus_token_mult_at_first_is_1p02() {
        assert!((singularity_bonus_token_mult(41.0) - 1.02).abs() < 1e-12);
    }

    #[test]
    fn bonus_token_mult_caps_at_1p10_at_top_threshold() {
        assert!((singularity_bonus_token_mult(229.0) - 1.10).abs() < 1e-12);
    }
}
