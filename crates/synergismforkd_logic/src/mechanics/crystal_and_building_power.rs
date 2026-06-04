//! Crystal and building-power production formulas.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/crystalAndBuildingPower.ts`
//! (in turn lifted from the legacy `packages/web_ui/src/Synergism.ts`
//! lines 2661-2745).
//!
//! All eight functions are pure given pre-extracted player / ant-upgrade /
//! rune / challenge inputs. The two `*_multiplier` functions accept a
//! pre-computed base (`building_power` / `crystal_exponent` / `base`) so
//! callers can avoid double evaluation when they already have it.

use synergismforkd_bignum::Decimal;

// ─── Building power ───────────────────────────────────────────────────────

/// Inputs to [`calculate_building_power`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateBuildingPowerInput {
    /// `CalcECC('reincarnation', player.challengecompletions[8])` —
    /// `×0.25` contribution.
    pub c8_reincarnation_ecc: f64,
    /// `player.reincarnationShards` — additive atom-bonus contribution
    /// via `log10(shards + 1)`.
    pub reincarnation_shards: Decimal,
    /// `player.researches[36]` — `×1/20` contribution to the additive
    /// base.
    pub research_36: f64,
    /// `player.researches[37]` — `×1/40` contribution.
    pub research_37: f64,
    /// `player.researches[38]` — `×1/40` contribution.
    pub research_38: f64,
    /// `getAntUpgradeEffect(AntUpgrades.BuildingCostScale).buildingPowerMult`.
    pub building_cost_scale_ant_upgrade_building_power_mult: f64,
    /// `player.cubeUpgrades[12]` — exponent bump of `×0.09`.
    pub cube_upgrade_12: f64,
    /// `player.cubeUpgrades[36]` — exponent bump of `×0.05`.
    pub cube_upgrade_36: f64,
    /// Whether `player.currentChallenge.reincarnation === 7` — collapses
    /// the final value to `1 + 0.05 * power`.
    pub in_reincarnation_challenge_7: bool,
}

/// Per-building power scalar. Aggregates atom bonus (log of
/// `reincarnation_shards`), challenge-8 reincarnation completions, the
/// three research multipliers, the building-cost-scale ant-upgrade mult,
/// the two cube-upgrade exponent bumps, and the challenge-7 final-fold
/// case.
#[must_use]
pub fn calculate_building_power(input: &CalculateBuildingPowerInput) -> f64 {
    let challenge_8_bonus = 0.25 * input.c8_reincarnation_ecc;

    let mut power = 1.0_f64;
    // Atom bonus
    power += (1.0 - 2.0_f64.powf(-1.0 / 160.0))
        * (input.reincarnation_shards + Decimal::one())
            .log10()
            .to_number();
    // Challenge 8 reward
    power += challenge_8_bonus;

    // Researches
    power *= 1.0 + (1.0 / 20.0) * input.research_36;
    power *= 1.0 + (1.0 / 40.0) * input.research_37;
    power *= 1.0 + (1.0 / 40.0) * input.research_38;

    // Ant
    power *= input.building_cost_scale_ant_upgrade_building_power_mult;

    // Cube upgrades raise the base to a power
    power = power.powf(1.0 + input.cube_upgrade_12 * 0.09);
    power = power.powf(1.0 + input.cube_upgrade_36 * 0.05);

    // Challenge 7 — collapses to a much smaller power
    if input.in_reincarnation_challenge_7 {
        power = 1.0 + 0.05 * power;
    }

    power
}

/// Coin-side building-power multiplier: `building_power ^ total_owned_coin`.
/// Caller passes pre-computed `building_power` and `total_owned_coin`.
#[must_use]
pub fn calculate_building_power_coin_multiplier(
    building_power: f64,
    total_owned_coin: f64,
) -> Decimal {
    Decimal::from_finite(building_power).pow(Decimal::from_finite(total_owned_coin))
}

/// Above this total ascend-building count, `ascend_building_dr` switches
/// from the raw sum to a square-root diminishing-returns curve.
const ASCEND_BUILDING_DR_THRESHOLD: f64 = 100_000.0;

/// `ascendBuildingDR()` — diminishing-returns value over the summed
/// ascend-building (tesseract) owned count. Above the threshold it
/// becomes `sqrt(threshold) * sqrt(sum)`; below, it's the raw sum.
/// Feeds the `ascend_building_dr_value` slot of
/// [`compute_global_multipliers`].
#[must_use]
pub fn ascend_building_dr(total_ascend_buildings_owned: f64) -> f64 {
    if total_ascend_buildings_owned > ASCEND_BUILDING_DR_THRESHOLD {
        ASCEND_BUILDING_DR_THRESHOLD.sqrt() * total_ascend_buildings_owned.sqrt()
    } else {
        total_ascend_buildings_owned
    }
}

// ─── Crystal exponent ─────────────────────────────────────────────────────

/// Inputs to [`crystal_upgrade_4_max_exponent`].
#[derive(Debug, Clone, Copy)]
pub struct CrystalUpgrade4MaxExponentInput {
    /// `player.researches[129]` — `×0.05 × log_4(commonFragments + 1)`.
    pub research_129: f64,
    /// `player.commonFragments` — `log` base 4.
    pub common_fragments: Decimal,
    /// `getRuneSpiritEffect('prism').crystalCaps` — additive
    /// contribution.
    pub prism_spirit_crystal_caps: f64,
}

/// Cap on crystal-upgrade-3's exponent contribution.
#[must_use]
pub fn crystal_upgrade_4_max_exponent(input: &CrystalUpgrade4MaxExponentInput) -> f64 {
    let mut exponent = 10.0_f64;
    exponent += 0.05
        * input.research_129
        * (input.common_fragments + Decimal::one())
            .log(Decimal::from_finite(4.0))
            .to_number();
    exponent += input.prism_spirit_crystal_caps;
    exponent
}

/// Inputs to [`calculate_crystal_exponent`].
#[derive(Debug, Clone, Copy)]
pub struct CalculateCrystalExponentInput {
    /// Result of [`crystal_upgrade_4_max_exponent`] — the cap.
    pub crystal_upgrade_3_max_exponent: f64,
    /// `player.crystalUpgrades[3]` — drives the `×(1 - 0.995^N)` approach
    /// to the cap.
    pub crystal_upgrade_3: f64,
    /// `CalcECC('transcend', player.challengecompletions[3])` — `×0.04`
    /// contribution.
    pub c3_transcend_ecc: f64,
    /// `player.researches[28]` — Research 2x3, `×0.08` contribution.
    pub research_28: f64,
    /// `player.researches[29]` — Research 2x4, `×0.08` contribution.
    pub research_29: f64,
    /// `player.researches[30]` — Research 2x5, `×0.04` contribution.
    pub research_30: f64,
    /// `player.cubeUpgrades[17]` — Cube 2x7, `×8` contribution.
    pub cube_upgrade_17: f64,
}

/// Crystal exponent for the prestige-shards production formula. Base
/// `1/3` plus capped crystal-upgrade-3 contribution, challenge-3 ECC,
/// three research lines, and cube-upgrade-17 (`×8` contribution per
/// level).
#[must_use]
pub fn calculate_crystal_exponent(input: &CalculateCrystalExponentInput) -> f64 {
    let mut exponent = 1.0 / 3.0;
    exponent +=
        input.crystal_upgrade_3_max_exponent * (1.0 - 0.995_f64.powf(input.crystal_upgrade_3));
    exponent += 0.04 * input.c3_transcend_ecc;
    exponent += 0.08 * input.research_28;
    exponent += 0.08 * input.research_29;
    exponent += 0.04 * input.research_30;
    exponent += 8.0 * input.cube_upgrade_17;
    exponent
}

/// Coin-side crystal multiplier: `(prestige_shards + 1) ^ crystal_exponent`.
#[must_use]
pub fn calculate_crystal_coin_multiplier(
    prestige_shards: Decimal,
    crystal_exponent: f64,
) -> Decimal {
    (prestige_shards + Decimal::one()).pow(Decimal::from_finite(crystal_exponent))
}

// ─── Crystal upgrade 3 base ───────────────────────────────────────────────

/// Inputs to [`crystal_upgrade_3_max_base`].
#[derive(Debug, Clone, Copy)]
pub struct CrystalUpgrade3MaxBaseInput {
    /// `player.upgrades[122]` — `×1` contribution.
    pub upgrade_122: f64,
    /// `player.researches[129]` — `×0.001 × log_4(commonFragments + 1)`.
    pub research_129: f64,
    /// `player.commonFragments` — `log` base 4.
    pub common_fragments: Decimal,
}

/// Cap on crystal-upgrade-3's base.
#[must_use]
pub fn crystal_upgrade_3_max_base(input: &CrystalUpgrade3MaxBaseInput) -> f64 {
    let mut max_base = 2.0_f64;
    max_base += input.upgrade_122;
    max_base += 0.001
        * input.research_129
        * (input.common_fragments + Decimal::one())
            .log(Decimal::from_finite(4.0))
            .to_number();
    max_base
}

/// Inputs to [`crystal_upgrade_3_base`].
#[derive(Debug, Clone, Copy)]
pub struct CrystalUpgrade3BaseInput {
    /// Result of [`crystal_upgrade_3_max_base`].
    pub max_base: f64,
    /// `player.crystalUpgrades[2]` — drives the `×(1 - 0.999^N)` approach
    /// to `max_base`.
    pub crystal_upgrade_2: f64,
}

/// Effective base for crystal-upgrade-3's contribution to crystal
/// production.
#[must_use]
pub fn crystal_upgrade_3_base(input: &CrystalUpgrade3BaseInput) -> f64 {
    1.0 + (input.max_base - 1.0) * (1.0 - 0.999_f64.powf(input.crystal_upgrade_2))
}

/// Inputs to [`crystal_upgrade_3_crystal_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct CrystalUpgrade3CrystalMultiplierInput {
    /// Result of [`crystal_upgrade_3_base`].
    pub base: f64,
    /// Sum of `player.first/second/third/fourth/fifthOwnedDiamonds` —
    /// diamond producer count.
    pub crystal_producers_owned: f64,
}

/// Crystal-side multiplier: `base ^ crystal_producers_owned`.
#[must_use]
pub fn crystal_upgrade_3_crystal_multiplier(
    input: &CrystalUpgrade3CrystalMultiplierInput,
) -> Decimal {
    Decimal::from_finite(input.base).pow(Decimal::from_finite(input.crystal_producers_owned))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn building_power_baseline() -> CalculateBuildingPowerInput {
        CalculateBuildingPowerInput {
            c8_reincarnation_ecc: 0.0,
            reincarnation_shards: Decimal::zero(),
            research_36: 0.0,
            research_37: 0.0,
            research_38: 0.0,
            building_cost_scale_ant_upgrade_building_power_mult: 1.0,
            cube_upgrade_12: 0.0,
            cube_upgrade_36: 0.0,
            in_reincarnation_challenge_7: false,
        }
    }

    #[test]
    fn building_power_baseline_is_one() {
        // No shards, no challenges, unit multipliers → power = 1.
        let result = calculate_building_power(&building_power_baseline());
        assert!((result - 1.0).abs() < 1e-12);
    }

    #[test]
    fn building_power_atom_bonus_scales_with_shards() {
        // 1e10 shards → log10(1e10 + 1) ≈ 10 → bonus ≈ 10 * (1 - 2^(-1/160))
        let input = CalculateBuildingPowerInput {
            reincarnation_shards: Decimal::from_finite(1e10),
            ..building_power_baseline()
        };
        let result = calculate_building_power(&input);
        let expected_bonus = 10.0 * (1.0 - 2.0_f64.powf(-1.0 / 160.0));
        assert!((result - (1.0 + expected_bonus)).abs() < 1e-9);
    }

    #[test]
    fn building_power_challenge_8_adds_quarter_per_ecc() {
        let input = CalculateBuildingPowerInput {
            c8_reincarnation_ecc: 4.0,
            ..building_power_baseline()
        };
        // 1 + 0.25*4 = 2.0
        let result = calculate_building_power(&input);
        assert!((result - 2.0).abs() < 1e-12);
    }

    #[test]
    fn building_power_research_36_at_one_adds_5_percent() {
        let input = CalculateBuildingPowerInput {
            research_36: 1.0,
            ..building_power_baseline()
        };
        // 1 * 1.05 = 1.05
        assert!((calculate_building_power(&input) - 1.05).abs() < 1e-12);
    }

    #[test]
    fn building_power_challenge_7_collapses_to_one_plus_5pct() {
        let input = CalculateBuildingPowerInput {
            reincarnation_shards: Decimal::from_finite(1e10),
            in_reincarnation_challenge_7: true,
            ..building_power_baseline()
        };
        // power before C7 fold = 1 + atom_bonus; after fold = 1 + 0.05 * (1+atom_bonus)
        let result = calculate_building_power(&input);
        let pre_c7 = 1.0 + 10.0 * (1.0 - 2.0_f64.powf(-1.0 / 160.0));
        let expected = 1.0 + 0.05 * pre_c7;
        assert!((result - expected).abs() < 1e-9);
    }

    #[test]
    fn building_power_coin_multiplier_is_pow() {
        let result = calculate_building_power_coin_multiplier(2.0, 10.0);
        // 2^10 = 1024
        assert!((result.to_number() - 1024.0).abs() < 1e-9);
    }

    #[test]
    fn ascend_building_dr_is_raw_sum_below_threshold() {
        assert_eq!(ascend_building_dr(0.0), 0.0);
        assert_eq!(ascend_building_dr(50_000.0), 50_000.0);
        // Exactly at the threshold stays raw (strict `>`).
        assert_eq!(ascend_building_dr(100_000.0), 100_000.0);
    }

    #[test]
    fn ascend_building_dr_uses_sqrt_curve_above_threshold() {
        // sum = 400_000 → sqrt(1e5) * sqrt(4e5) = sqrt(4e10) = 2e5.
        let result = ascend_building_dr(400_000.0);
        assert!((result - 200_000.0).abs() < 1e-6);
        // Always below the raw sum once past the threshold.
        assert!(ascend_building_dr(1e6) < 1e6);
    }

    // ─── crystal exponent ──────────────────────────────────────────────────

    #[test]
    fn crystal_upgrade_4_max_exponent_baseline_is_10() {
        let input = CrystalUpgrade4MaxExponentInput {
            research_129: 0.0,
            common_fragments: Decimal::zero(),
            prism_spirit_crystal_caps: 0.0,
        };
        assert_eq!(crystal_upgrade_4_max_exponent(&input), 10.0);
    }

    #[test]
    fn crystal_upgrade_4_max_exponent_includes_spirit_caps() {
        let input = CrystalUpgrade4MaxExponentInput {
            research_129: 0.0,
            common_fragments: Decimal::zero(),
            prism_spirit_crystal_caps: 5.0,
        };
        assert_eq!(crystal_upgrade_4_max_exponent(&input), 15.0);
    }

    #[test]
    fn calculate_crystal_exponent_baseline_is_one_third() {
        let input = CalculateCrystalExponentInput {
            crystal_upgrade_3_max_exponent: 10.0,
            crystal_upgrade_3: 0.0,
            c3_transcend_ecc: 0.0,
            research_28: 0.0,
            research_29: 0.0,
            research_30: 0.0,
            cube_upgrade_17: 0.0,
        };
        // 1/3 + 10 * (1 - 0.995^0) = 1/3 + 0 = 1/3
        assert!((calculate_crystal_exponent(&input) - 1.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn calculate_crystal_exponent_research_lines_stack() {
        let input = CalculateCrystalExponentInput {
            crystal_upgrade_3_max_exponent: 10.0,
            crystal_upgrade_3: 0.0,
            c3_transcend_ecc: 0.0,
            research_28: 1.0,
            research_29: 1.0,
            research_30: 1.0,
            cube_upgrade_17: 0.0,
        };
        // 1/3 + 0.08 + 0.08 + 0.04 = 1/3 + 0.20
        let expected = 1.0 / 3.0 + 0.20;
        assert!((calculate_crystal_exponent(&input) - expected).abs() < 1e-12);
    }

    #[test]
    fn calculate_crystal_coin_multiplier_is_pow() {
        // (1e10 + 1)^(1/3) ≈ 2154.43
        let result = calculate_crystal_coin_multiplier(Decimal::from_finite(1e10), 1.0 / 3.0);
        assert!((result.to_number() - 2154.434690_f64).abs() / 2154.434690_f64 < 1e-6);
    }

    // ─── crystal upgrade 3 base ────────────────────────────────────────────

    #[test]
    fn crystal_upgrade_3_max_base_baseline_is_2() {
        let input = CrystalUpgrade3MaxBaseInput {
            upgrade_122: 0.0,
            research_129: 0.0,
            common_fragments: Decimal::zero(),
        };
        assert_eq!(crystal_upgrade_3_max_base(&input), 2.0);
    }

    #[test]
    fn crystal_upgrade_3_max_base_with_upgrade_122() {
        let input = CrystalUpgrade3MaxBaseInput {
            upgrade_122: 1.0,
            research_129: 0.0,
            common_fragments: Decimal::zero(),
        };
        assert_eq!(crystal_upgrade_3_max_base(&input), 3.0);
    }

    #[test]
    fn crystal_upgrade_3_base_approaches_max_base_at_zero_level() {
        // crystal_upgrade_2 = 0 → 0.999^0 = 1 → 1 - 1 = 0 → base = 1
        let input = CrystalUpgrade3BaseInput {
            max_base: 2.0,
            crystal_upgrade_2: 0.0,
        };
        assert_eq!(crystal_upgrade_3_base(&input), 1.0);
    }

    #[test]
    fn crystal_upgrade_3_base_approaches_max_base_with_many_levels() {
        // crystal_upgrade_2 large → 0.999^N → 0 → base → max_base
        let input = CrystalUpgrade3BaseInput {
            max_base: 2.0,
            crystal_upgrade_2: 100_000.0,
        };
        let result = crystal_upgrade_3_base(&input);
        assert!((result - 2.0).abs() < 1e-6);
    }

    #[test]
    fn crystal_upgrade_3_crystal_multiplier_is_pow() {
        let input = CrystalUpgrade3CrystalMultiplierInput {
            base: 2.0,
            crystal_producers_owned: 10.0,
        };
        let result = crystal_upgrade_3_crystal_multiplier(&input);
        assert!((result.to_number() - 1024.0).abs() < 1e-9);
    }
}
