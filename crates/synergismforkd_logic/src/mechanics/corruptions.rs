//! Corruption math.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/corruptions.ts`
//! (migrated from the legacy `packages/web_ui/src/Corruptions.ts`). The
//! `CorruptionLoadout` / `CorruptionSaves` classes and the UI loadout
//! table stay in the UI tier — this module owns the per-corruption
//! multipliers, the cap formula, and the score / difficulty
//! calculators.

// ─── Cap on per-corruption level ───────────────────────────────────────────

/// Inputs to [`max_corruption_level`].
#[derive(Debug, Clone, Copy)]
pub struct MaxCorruptionLevelInput {
    /// `player.challengecompletions[11]`. `+5` to cap when any
    /// completion exists.
    pub challenge_11_completions: f64,
    /// `player.challengecompletions[12]`. `+2` when any.
    pub challenge_12_completions: f64,
    /// `player.challengecompletions[13]`. `+2` when any.
    pub challenge_13_completions: f64,
    /// `player.challengecompletions[14]`. `+2` when any.
    pub challenge_14_completions: f64,
    /// `player.platonicUpgrades[5]`. `+1` when any.
    pub platonic_upgrade_5: f64,
    /// `player.platonicUpgrades[10]`. `+1` when any.
    pub platonic_upgrade_10: f64,
    /// `getGQUpgradeEffect('platonicTau', 'unlocked')`. Floors at 13 —
    /// applied **after** the challenge/platonic adds, **before**
    /// `corruption_fourteen`.
    pub platonic_tau_unlocked: bool,
    /// `getGQUpgradeEffect('corruptionFourteen', 'unlocked')`. `+1` to
    /// the final cap (after the platonic-tau floor).
    pub corruption_fourteen_unlocked: bool,
    /// `getOcteractUpgradeEffect('octeractCorruption', 'corruptionLevelCapIncrease')`.
    /// Added to the final cap.
    pub octeract_corruption_cap_increase: f64,
}

/// Maximum corruption level players can set on any single corruption.
/// Sum of challenge / platonic / GQ / octeract contributions, with a
/// `platonic_tau` floor of `13` if that upgrade is unlocked.
#[must_use]
pub fn max_corruption_level(input: &MaxCorruptionLevelInput) -> f64 {
    let mut max = 0.0_f64;
    if input.challenge_11_completions > 0.0 {
        max += 5.0;
    }
    if input.challenge_12_completions > 0.0 {
        max += 2.0;
    }
    if input.challenge_13_completions > 0.0 {
        max += 2.0;
    }
    if input.challenge_14_completions > 0.0 {
        max += 2.0;
    }
    if input.platonic_upgrade_5 > 0.0 {
        max += 1.0;
    }
    if input.platonic_upgrade_10 > 0.0 {
        max += 1.0;
    }

    if input.platonic_tau_unlocked {
        max = max.max(13.0);
    }

    if input.corruption_fourteen_unlocked {
        max += 1.0;
    }
    max += input.octeract_corruption_cap_increase;

    max
}

// ─── Per-corruption effect calculators ─────────────────────────────────────

/// `G.viscosityPower` lookup table — viscosity production exponent
/// indexed by `player.corruptions.used.viscosity` corruption level.
/// 17 entries (`0..=16`); levels past `16` collapse to `0.0`.
/// Verbatim port of the constant in
/// `legacy/original/src/Variables.ts:149`.
pub const VISCOSITY_POWER: [f64; 17] = [
    1.0, 0.87, 0.80, 0.75, 0.70, 0.6, 0.54, 0.45, 0.39, 0.33, 0.3, 0.2, 0.1, 0.05, 0.0, 0.0, 0.0,
];

/// `G.viscosityPower[level]` with a safe out-of-range fallback to
/// `0.0` (matches the legacy lookup behavior at the highest
/// corruption tier).
#[must_use]
pub fn viscosity_power_at_level(level: u32) -> f64 {
    VISCOSITY_POWER.get(level as usize).copied().unwrap_or(0.0)
}

/// `G.recessionPower` lookup table — recession production exponent
/// indexed by `player.corruptions.used.recession` corruption level.
/// 17 entries (`0..=16`); levels past `16` collapse to the last
/// value's tail. Verbatim port of the constant in
/// `legacy/original/src/Variables.ts`.
pub const RECESSION_POWER: [f64; 17] = [
    1.0, 0.9, 0.7, 0.6, 0.5, 0.37, 0.30, 0.23, 0.18, 0.15, 0.12, 0.09, 0.03, 0.01, 0.007, 0.0007,
    0.000_07,
];

/// `G.recessionPower[level]` with a safe out-of-range fallback to
/// `0.0`.
#[must_use]
pub fn recession_power_at_level(level: u32) -> f64 {
    RECESSION_POWER.get(level as usize).copied().unwrap_or(0.0)
}

/// `G.illiteracyPower` lookup table — illiteracy obtainium DR exponent
/// indexed by `player.corruptions.used.illiteracy` corruption level.
/// 17 entries (`0..=16`); levels past `16` collapse to `0.0`. Verbatim
/// port of the constant in
/// `legacy/core_split/packages/web_ui/src/Variables.ts:170`.
pub const ILLITERACY_POWER: [f64; 17] = [
    1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.45, 0.4, 0.35, 0.3, 0.25, 0.20, 0.15, 0.10, 0.08, 0.06, 0.04,
];

/// `G.illiteracyPower[level]` with a safe out-of-range fallback to
/// `0.0`.
#[must_use]
pub fn illiteracy_power_at_level(level: u32) -> f64 {
    ILLITERACY_POWER.get(level as usize).copied().unwrap_or(0.0)
}

/// `G.extinctionDivisor` lookup table — ant-upgrade true-level divisor
/// indexed by `player.corruptions.used.extinction` corruption level.
/// 17 entries (`0..=16`); levels past `16` fall back to `0.0`.
/// Verbatim port of the constant in
/// `legacy/original/src/Variables.ts:191`.
///
/// Used by [`calculate_true_ant_level`](crate::mechanics::ant_upgrade_levels::calculate_true_ant_level)
/// via the `corruption_extinction_divisor` input field.
pub const EXTINCTION_DIVISOR: [f64; 17] = [
    1.0, 1.25, 1.5, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
];

/// `G.extinctionDivisor[level]` with a safe out-of-range fallback to
/// `0.0`.
#[must_use]
pub fn extinction_divisor_at_level(level: u32) -> f64 {
    EXTINCTION_DIVISOR
        .get(level as usize)
        .copied()
        .unwrap_or(0.0)
}

/// `G.deflationMultiplier` lookup table — prestige-power scaler indexed by
/// `player.corruptions.used.deflation` corruption level. 17 entries
/// (`0..=16`); the tail is `0`. Verbatim port of the constant in
/// `legacy/core_split/packages/web_ui/src/Variables.ts`.
pub const DEFLATION_MULTIPLIER: [f64; 17] = [
    1.0,
    0.3,
    0.1,
    0.03,
    0.01,
    1.0 / 1e6,
    1.0 / 1e8,
    1.0 / 1e10,
    1.0 / 1e12,
    1.0 / 1e15,
    1.0 / 1e18,
    1.0 / 1e25,
    1.0 / 1e35,
    1.0 / 1e50,
    1.0 / 1e77,
    0.0,
    0.0,
];

/// `G.deflationMultiplier[level]` with a safe out-of-range fallback to
/// `0.0` (consistent with the table's zero tail).
#[must_use]
pub fn deflation_multiplier_at_level(level: u32) -> f64 {
    DEFLATION_MULTIPLIER
        .get(level as usize)
        .copied()
        .unwrap_or(0.0)
}

/// `G.dilationMultiplier` lookup table — global-speed scaler indexed by
/// `player.corruptions.used.dilation` corruption level. 17 entries
/// (`0..=16`). Verbatim port of the constant in
/// `legacy/core_split/packages/web_ui/src/Variables.ts`. The dilation
/// corruption feeds only the global-speed StatLine product (not
/// ascension speed).
pub const DILATION_MULTIPLIER: [f64; 17] = [
    1.0,
    1.0 / 3.0,
    1.0 / 10.0,
    1.0 / 40.0,
    1.0 / 200.0,
    1.0 / 3e4,
    1.0 / 3e6,
    1.0 / 3e9,
    1.0 / 3e12,
    1.0 / 1e15,
    1.0 / 1e19,
    1.0 / 1e24,
    1.0 / 1e34,
    1.0 / 1e48,
    1.0 / 1e65,
    1.0 / 1e80,
    1.0 / 1e100,
];

/// `G.dilationMultiplier[level]` with a safe out-of-range fallback to
/// `0.0` (corruption levels are capped within the table in practice).
#[must_use]
pub fn dilation_multiplier_at_level(level: u32) -> f64 {
    DILATION_MULTIPLIER
        .get(level as usize)
        .copied()
        .unwrap_or(0.0)
}

/// Inputs to [`viscosity_effect`].
#[derive(Debug, Clone, Copy)]
pub struct ViscosityEffectInput {
    /// `G.viscosityPower[level]` — the level-indexed base exponent.
    pub base_power: f64,
    /// `player.platonicUpgrades[6]`. Multiplies base by
    /// `(1 + n / 30)`.
    pub platonic_upgrade_6: f64,
}

/// Viscosity production exponent. Clamped to `≤ 1` — buffs can only
/// soften the corruption, never reverse it.
#[must_use]
pub fn viscosity_effect(input: &ViscosityEffectInput) -> f64 {
    (input.base_power * (1.0 + input.platonic_upgrade_6 / 30.0)).min(1.0)
}

/// Inputs to [`drought_effect`].
#[derive(Debug, Clone, Copy)]
pub struct DroughtEffectInput {
    /// `G.droughtSalvage[level]`.
    pub base_salvage: f64,
    /// `player.platonicUpgrades[13]`. When `> 0`, halves the salvage
    /// reduction.
    pub platonic_upgrade_13: f64,
}

/// Drought salvage reduction multiplier. Platonic 13 halves the
/// reduction.
#[must_use]
pub fn drought_effect(input: &DroughtEffectInput) -> f64 {
    if input.platonic_upgrade_13 > 0.0 {
        input.base_salvage * 0.5
    } else {
        input.base_salvage
    }
}

/// Inputs to [`illiteracy_effect`].
#[derive(Debug, Clone, Copy)]
pub struct IlliteracyEffectInput {
    /// `G.illiteracyPower[level]`.
    pub base_power: f64,
    /// `player.platonicUpgrades[9]`.
    pub platonic_upgrade_9: f64,
    /// `player.obtainium.gte(1)` AND `log10(player.obtainium)` — the
    /// obtainium-based boost only applies when `obtainium ≥ 1`. Pass:
    /// - `None` if `obtainium < 1` (boost path skipped)
    /// - the `log10` value otherwise (this function clamps to 100)
    ///
    /// Keeping `Option<f64>` here keeps the `Decimal` dependency on
    /// the wrapper side.
    pub obtainium_log10: Option<f64>,
}

/// Illiteracy production exponent. When `obtainium ≥ 1`, gets bumped
/// by `1 + (platonic_9 / 100) * min(100, log10(obtainium))`. Clamped
/// to `≤ 1`.
#[must_use]
pub fn illiteracy_effect(input: &IlliteracyEffectInput) -> f64 {
    let multiplier = match input.obtainium_log10 {
        None => 1.0,
        Some(log10) => 1.0 + (1.0 / 100.0) * input.platonic_upgrade_9 * 100.0_f64.min(log10),
    };
    (input.base_power * multiplier).min(1.0)
}

/// Inputs to [`hyperchallenge_effect`].
#[derive(Debug, Clone, Copy)]
pub struct HyperchallengeEffectInput {
    /// `G.hyperchallengeMultiplier[level]`.
    pub base_effect: f64,
    /// `player.platonicUpgrades[8]`. Divides base by
    /// `(1 + 2/5 * n)`.
    pub platonic_upgrade_8: f64,
}

/// Hyperchallenge requirement multiplier. Floored at `1` — platonic-8
/// can soften the corruption but never make challenges easier than
/// baseline.
#[must_use]
pub fn hyperchallenge_effect(input: &HyperchallengeEffectInput) -> f64 {
    let divisor = 1.0 + 2.0 / 5.0 * input.platonic_upgrade_8;
    (input.base_effect / divisor).max(1.0)
}

// ─── Per-corruption score multiplier ───────────────────────────────────────

/// Score-multiplier table indexed by total corruption level (level +
/// bonus levels). For total levels at or beyond the last index, the
/// formula extrapolates with a `1.2^x` geometric tail.
pub const CORRUPTION_SCORE_MULTS: [f64; 19] = [
    1.0, 3.0, 4.0, 5.0, 6.0, 7.0, 7.75, 8.5, 9.25, 10.0, 10.75, 11.5, 12.25, 13.0, 16.0, 20.0,
    25.0, 33.0, 35.0,
];

/// Inputs to [`calculate_corruption_raw_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct CorruptionRawMultiplierInput {
    /// Per-corruption level + bonus levels (cookieGrandma + GQ
    /// corruption15 + SC oneChallengeCap + finiteDescent rune).
    pub total_level: f64,
    /// Additive score increase applied inside the power base. Sum of:
    /// - `getGQUpgradeEffect('advancedPack', 'corruptionScoreIncrease')`
    /// - `getSingularityChallengeEffect('oneChallengeCap', 'corrScoreIncrease')`
    /// - `0.3 * player.cubeUpgrades[74]`
    pub bonus_val: f64,
    /// Exponent applied to the result. Equal to `1` in the common
    /// case. When `player.platonicUpgrades[17] > 0` AND the corruption
    /// is `viscosity` AND `levels.viscosity >= 10`, callers pass
    /// `3 + 0.04 * platonicUpgrades[17]` (the P4x2 "exponent" buff).
    pub viscosity_power: f64,
}

/// Per-corruption score multiplier. Interpolates the static
/// [`CORRUPTION_SCORE_MULTS`] table for total levels under the table
/// length, then extrapolates with a `1.2^x` geometric tail for higher
/// levels. Both branches raised to `viscosity_power`.
#[must_use]
pub fn calculate_corruption_raw_multiplier(input: &CorruptionRawMultiplierInput) -> f64 {
    let score_mult_length = CORRUPTION_SCORE_MULTS.len() as f64;
    let total_level = input.total_level;

    if total_level < score_mult_length - 1.0 {
        let portion_above_level = total_level.ceil() - total_level;
        let floor_idx = total_level.floor() as usize;
        let ceil_idx = total_level.ceil() as usize;
        (CORRUPTION_SCORE_MULTS[floor_idx]
            + input.bonus_val
            + portion_above_level
                * (CORRUPTION_SCORE_MULTS[ceil_idx] - CORRUPTION_SCORE_MULTS[floor_idx]))
            .powf(input.viscosity_power)
    } else {
        ((CORRUPTION_SCORE_MULTS[CORRUPTION_SCORE_MULTS.len() - 1] + input.bonus_val)
            * 1.2_f64.powf(total_level - score_mult_length + 1.0))
        .powf(input.viscosity_power)
    }
}

// ─── Total ascension-score multiplier ──────────────────────────────────────

/// Inputs to [`calculate_total_corruption_score_mult`].
#[derive(Debug, Clone, Copy)]
pub struct TotalCorruptionScoreMultInput<'a> {
    /// The active corruption levels (the first 8 entries are the real
    /// corruptions; viscosity is index 0).
    pub levels: &'a [u32],
    /// `bonusLevels` — added to every corruption's level before lookup
    /// (`corruptionFifteen` + `oneChallengeCap` + `cookieGrandma` +
    /// finiteDescent rune free levels).
    pub bonus_levels: f64,
    /// `bonusVal` — additive score increase inside the power base
    /// (`advancedPack` + `oneChallengeCap` + `0.3·cubeUpgrades[74]`).
    pub bonus_val: f64,
    /// `player.platonicUpgrades[17]` — the viscosity-only P4x2 exponent
    /// buff (applies when viscosity level `>= 10`).
    pub viscosity_platonic_17: f64,
}

/// `player.corruptions.used.totalCorruptionAscensionMultiplier`
/// (`Corruptions.ts` `#calcTotalScoreMult`) — the product of each
/// corruption's [`calculate_corruption_raw_multiplier`]. Viscosity
/// (index 0) gets the `3 + 0.04·platonicUpgrades[17]` exponent when its
/// level is `>= 10` and the platonic upgrade is owned; all other
/// corruptions use exponent `1`.
#[must_use]
pub fn calculate_total_corruption_score_mult(input: &TotalCorruptionScoreMultInput<'_>) -> f64 {
    let mut product = 1.0_f64;
    for (index, &level) in input.levels.iter().take(8).enumerate() {
        let viscosity_power = if index == crate::state::VISCOSITY_INDEX
            && input.viscosity_platonic_17 > 0.0
            && level >= 10
        {
            3.0 + 0.04 * input.viscosity_platonic_17
        } else {
            1.0
        };
        product *= calculate_corruption_raw_multiplier(&CorruptionRawMultiplierInput {
            total_level: f64::from(level) + input.bonus_levels,
            bonus_val: input.bonus_val,
            viscosity_power,
        });
    }
    product
}

// ─── Difficulty score ──────────────────────────────────────────────────────

/// Total corruption difficulty score. Starts at 400 and adds
/// `16 * (total_level)²` per corruption. Callers pass the
/// per-corruption total levels (level + bonus levels), in any order.
#[must_use]
pub fn calculate_corruption_difficulty_score(total_levels: &[f64]) -> f64 {
    let mut base_points = 400.0_f64;
    for &lvl in total_levels {
        base_points += 16.0 * lvl * lvl;
    }
    base_points
}

/// `CorruptionLoadout.totalCorruptionDifficultyScore` (Corruptions.ts:251-254)
/// for a stored loadout: the difficulty over the 8 real corruptions at
/// `level + bonusLevels` each (the bonus applies to every corruption,
/// including zero-level ones).
#[must_use]
pub fn corruption_loadout_difficulty_score(levels: &[u32], bonus_levels: f64) -> f64 {
    let mut totals = [0.0_f64; 8];
    for (total, &level) in totals.iter_mut().zip(levels.iter().take(8)) {
        *total = f64::from(level) + bonus_levels;
    }
    calculate_corruption_difficulty_score(&totals)
}

// ─── Level clipping / validation ───────────────────────────────────────────

/// Clip a single corruption level to a valid stored value. Returns
/// `0` if the input isn't a finite integer; otherwise clamps to
/// `[0, max_level]`.
#[must_use]
pub fn clip_corruption_level(level: f64, max_level: f64) -> f64 {
    if !level.is_finite() || level.is_nan() || level.fract() != 0.0 {
        return 0.0;
    }
    level.clamp(0.0, max_level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viscosity_power_table_matches_legacy() {
        // Legacy `G.viscosityPower` from `legacy/original/src/Variables.ts:149`.
        assert_eq!(viscosity_power_at_level(0), 1.0);
        assert_eq!(viscosity_power_at_level(1), 0.87);
        assert_eq!(viscosity_power_at_level(7), 0.45);
        assert_eq!(viscosity_power_at_level(14), 0.0);
        // Past the last entry — saturates to 0 (matches the legacy tail).
        assert_eq!(viscosity_power_at_level(100), 0.0);
    }

    #[test]
    fn recession_power_table_matches_legacy() {
        // Legacy `G.recessionPower` from `legacy/original/src/Variables.ts`.
        assert_eq!(recession_power_at_level(0), 1.0);
        assert_eq!(recession_power_at_level(1), 0.9);
        assert_eq!(recession_power_at_level(5), 0.37);
        assert!((recession_power_at_level(16) - 0.000_07).abs() < 1e-9);
        // Past the last entry — saturates to 0.
        assert_eq!(recession_power_at_level(100), 0.0);
    }

    #[test]
    fn illiteracy_power_table_matches_legacy() {
        // Legacy `G.illiteracyPower` from
        // `legacy/core_split/packages/web_ui/src/Variables.ts:170`.
        assert_eq!(illiteracy_power_at_level(0), 1.0);
        assert_eq!(illiteracy_power_at_level(1), 0.9);
        assert_eq!(illiteracy_power_at_level(6), 0.45);
        assert_eq!(illiteracy_power_at_level(16), 0.04);
        // Past the last entry — saturates to 0.
        assert_eq!(illiteracy_power_at_level(100), 0.0);
    }

    #[test]
    fn extinction_divisor_table_matches_legacy() {
        // Legacy `G.extinctionDivisor` from `Variables.ts:191`.
        assert_eq!(extinction_divisor_at_level(0), 1.0); // no corruption → no penalty
        assert_eq!(extinction_divisor_at_level(1), 1.25);
        assert_eq!(extinction_divisor_at_level(4), 3.0);
        assert_eq!(extinction_divisor_at_level(5), 4.0);
        assert_eq!(extinction_divisor_at_level(16), 15.0);
        // Past the last entry — falls back to 0.
        assert_eq!(extinction_divisor_at_level(100), 0.0);
    }

    #[test]
    fn deflation_multiplier_table_matches_legacy() {
        // Legacy `G.deflationMultiplier` from `Variables.ts`.
        assert_eq!(deflation_multiplier_at_level(0), 1.0);
        assert_eq!(deflation_multiplier_at_level(4), 0.01);
        assert!((deflation_multiplier_at_level(5) - 1.0 / 1e6).abs() < 1e-18);
        assert!((deflation_multiplier_at_level(14) - 1.0 / 1e77).abs() < 1e-90);
        assert_eq!(deflation_multiplier_at_level(15), 0.0);
        // Past the last entry — saturates to 0.
        assert_eq!(deflation_multiplier_at_level(100), 0.0);
    }

    #[test]
    fn dilation_multiplier_table_matches_legacy() {
        // Legacy `G.dilationMultiplier` from `Variables.ts` (17 entries).
        assert_eq!(dilation_multiplier_at_level(0), 1.0);
        assert!((dilation_multiplier_at_level(1) - 1.0 / 3.0).abs() < 1e-15);
        assert!((dilation_multiplier_at_level(2) - 0.1).abs() < 1e-15);
        assert!((dilation_multiplier_at_level(4) - 1.0 / 200.0).abs() < 1e-15);
        assert!((dilation_multiplier_at_level(5) - 1.0 / 3e4).abs() < 1e-12);
        assert!((dilation_multiplier_at_level(16) - 1.0 / 1e100).abs() < 1e-115);
        // Past the last entry — saturates to 0.
        assert_eq!(dilation_multiplier_at_level(17), 0.0);
        assert_eq!(dilation_multiplier_at_level(100), 0.0);
    }

    fn baseline_max_input() -> MaxCorruptionLevelInput {
        MaxCorruptionLevelInput {
            challenge_11_completions: 0.0,
            challenge_12_completions: 0.0,
            challenge_13_completions: 0.0,
            challenge_14_completions: 0.0,
            platonic_upgrade_5: 0.0,
            platonic_upgrade_10: 0.0,
            platonic_tau_unlocked: false,
            corruption_fourteen_unlocked: false,
            octeract_corruption_cap_increase: 0.0,
        }
    }

    #[test]
    fn max_corruption_level_baseline_is_zero() {
        assert_eq!(max_corruption_level(&baseline_max_input()), 0.0);
    }

    #[test]
    fn max_corruption_level_full_challenges_and_platonics() {
        let input = MaxCorruptionLevelInput {
            challenge_11_completions: 1.0,
            challenge_12_completions: 1.0,
            challenge_13_completions: 1.0,
            challenge_14_completions: 1.0,
            platonic_upgrade_5: 1.0,
            platonic_upgrade_10: 1.0,
            ..baseline_max_input()
        };
        // 5 + 2 + 2 + 2 + 1 + 1 = 13
        assert_eq!(max_corruption_level(&input), 13.0);
    }

    #[test]
    fn max_corruption_level_platonic_tau_floor_at_13() {
        let input = MaxCorruptionLevelInput {
            platonic_tau_unlocked: true,
            ..baseline_max_input()
        };
        assert_eq!(max_corruption_level(&input), 13.0);
    }

    #[test]
    fn max_corruption_level_corruption_fourteen_after_floor() {
        let input = MaxCorruptionLevelInput {
            platonic_tau_unlocked: true,
            corruption_fourteen_unlocked: true,
            ..baseline_max_input()
        };
        assert_eq!(max_corruption_level(&input), 14.0);
    }

    #[test]
    fn viscosity_clamped_at_one() {
        let result = viscosity_effect(&ViscosityEffectInput {
            base_power: 1.5,
            platonic_upgrade_6: 30.0, // multiplier of 2 → base 3
        });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn viscosity_below_one_unchanged() {
        let result = viscosity_effect(&ViscosityEffectInput {
            base_power: 0.5,
            platonic_upgrade_6: 0.0,
        });
        assert_eq!(result, 0.5);
    }

    #[test]
    fn drought_platonic_13_halves() {
        let base = 0.4;
        let plain = drought_effect(&DroughtEffectInput {
            base_salvage: base,
            platonic_upgrade_13: 0.0,
        });
        let halved = drought_effect(&DroughtEffectInput {
            base_salvage: base,
            platonic_upgrade_13: 1.0,
        });
        assert_eq!(plain, base);
        assert_eq!(halved, base * 0.5);
    }

    #[test]
    fn illiteracy_no_obtainium_unchanged() {
        let result = illiteracy_effect(&IlliteracyEffectInput {
            base_power: 0.5,
            platonic_upgrade_9: 10.0,
            obtainium_log10: None,
        });
        // multiplier = 1 → result = 0.5
        assert_eq!(result, 0.5);
    }

    #[test]
    fn illiteracy_with_obtainium_bumps_base() {
        // base = 0.5, platonic_9 = 100, log10 = 50
        // multiplier = 1 + (1/100) * 100 * 50 = 1 + 50 = 51
        // result = min(0.5 * 51, 1) = min(25.5, 1) = 1
        let result = illiteracy_effect(&IlliteracyEffectInput {
            base_power: 0.5,
            platonic_upgrade_9: 100.0,
            obtainium_log10: Some(50.0),
        });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn illiteracy_obtainium_log_capped_at_100() {
        // log10 = 1000 → clamped to 100
        let capped_at_100 = illiteracy_effect(&IlliteracyEffectInput {
            base_power: 0.0,
            platonic_upgrade_9: 1.0,
            obtainium_log10: Some(1_000.0),
        });
        let no_cap_needed = illiteracy_effect(&IlliteracyEffectInput {
            base_power: 0.0,
            platonic_upgrade_9: 1.0,
            obtainium_log10: Some(100.0),
        });
        // Both should give same result since 100 ≤ 100.
        assert_eq!(capped_at_100, no_cap_needed);
    }

    #[test]
    fn hyperchallenge_floored_at_one() {
        let result = hyperchallenge_effect(&HyperchallengeEffectInput {
            base_effect: 0.5,
            platonic_upgrade_8: 10.0,
        });
        assert_eq!(result, 1.0);
    }

    #[test]
    fn hyperchallenge_normal_divisor() {
        // base = 100, platonic_8 = 5 → divisor = 1 + 2/5 * 5 = 3
        let result = hyperchallenge_effect(&HyperchallengeEffectInput {
            base_effect: 100.0,
            platonic_upgrade_8: 5.0,
        });
        assert!((result - 100.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn corruption_raw_multiplier_at_zero_level() {
        let result = calculate_corruption_raw_multiplier(&CorruptionRawMultiplierInput {
            total_level: 0.0,
            bonus_val: 0.0,
            viscosity_power: 1.0,
        });
        // table[0] = 1, raised to 1 → 1
        assert_eq!(result, 1.0);
    }

    #[test]
    fn corruption_raw_multiplier_interpolates() {
        // total_level = 0.5: floor=0 (1.0), ceil=1 (3.0)
        // portion_above = ceil - level = 1 - 0.5 = 0.5
        // (1 + 0.5*(3-1)) ^ 1 = 1 + 1 = 2
        let result = calculate_corruption_raw_multiplier(&CorruptionRawMultiplierInput {
            total_level: 0.5,
            bonus_val: 0.0,
            viscosity_power: 1.0,
        });
        assert!((result - 2.0).abs() < 1e-9);
    }

    #[test]
    fn corruption_raw_multiplier_extrapolates_past_table() {
        // total_level = 19 (== len), table[len-1] = 35
        // formula = 35 * 1.2^(19 - 19 + 1) = 35 * 1.2 = 42
        let result = calculate_corruption_raw_multiplier(&CorruptionRawMultiplierInput {
            total_level: 19.0,
            bonus_val: 0.0,
            viscosity_power: 1.0,
        });
        assert!((result - 42.0).abs() < 1e-9);
    }

    #[test]
    fn total_corruption_score_mult_is_product_of_raw() {
        let single = calculate_corruption_raw_multiplier(&CorruptionRawMultiplierInput {
            total_level: 0.0,
            bonus_val: 0.0,
            viscosity_power: 1.0,
        });
        let total = calculate_total_corruption_score_mult(&TotalCorruptionScoreMultInput {
            levels: &[0u32; 8],
            bonus_levels: 0.0,
            bonus_val: 0.0,
            viscosity_platonic_17: 0.0,
        });
        // Product of the 8 zero-level corruptions.
        assert!((total - single.powi(8)).abs() < 1e-9);
        // Higher levels strictly increase the product.
        let leveled = calculate_total_corruption_score_mult(&TotalCorruptionScoreMultInput {
            levels: &[5u32; 8],
            bonus_levels: 0.0,
            bonus_val: 0.0,
            viscosity_platonic_17: 0.0,
        });
        assert!(leveled > total);
    }

    #[test]
    fn corruption_difficulty_score_baseline_400() {
        assert_eq!(calculate_corruption_difficulty_score(&[]), 400.0);
    }

    #[test]
    fn corruption_difficulty_score_sums_squares() {
        // 400 + 16*1 + 16*4 + 16*9 = 400 + 16 + 64 + 144 = 624
        let result = calculate_corruption_difficulty_score(&[1.0, 2.0, 3.0]);
        assert_eq!(result, 624.0);
    }

    #[test]
    fn clip_corruption_level_rejects_non_integers() {
        assert_eq!(clip_corruption_level(1.5, 10.0), 0.0);
        assert_eq!(clip_corruption_level(f64::NAN, 10.0), 0.0);
        assert_eq!(clip_corruption_level(f64::INFINITY, 10.0), 0.0);
    }

    #[test]
    fn clip_corruption_level_clamps_to_range() {
        assert_eq!(clip_corruption_level(-5.0, 10.0), 0.0);
        assert_eq!(clip_corruption_level(5.0, 10.0), 5.0);
        assert_eq!(clip_corruption_level(15.0, 10.0), 10.0);
    }
}
