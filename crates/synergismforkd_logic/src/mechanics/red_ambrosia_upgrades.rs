//! Per-upgrade cost-formula and effect formulas for red-ambrosia
//! upgrades.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/redAmbrosiaUpgrades.ts`.
//! The UI tier still owns the `redAmbrosiaUpgrades` data table (it
//! has UI fields the logic tier can't see — i18n-bound
//! name/description closures, DOM-driven renderers, the buy flow).
//! This module owns the two pure-formula fields each upgrade has:
//! `cost_formula(level, base_cost) -> f64` and
//! `effect(n, [key]) -> reward field`.
//!
//! One impure entry: salvage-yin-yang's effect reads
//! `player.singularityChallenges.taxmanLastStand.enabled`. The logic
//! version takes the gate as an extra parameter.

// ─── Constant cost-table arrays ───────────────────────────────────────────
// Five upgrades use level-indexed lookup tables instead of formulas.
// Hoisted to module scope to match the legacy file.

const BLUEBERRY_COST_VALUES: &[f64] = &[
    100_000.0,
    1_400_000.0,
    3_000_000.0,
    3_250_000.0,
    3_500_000.0,
];

const RED_AMBROSIA_FREE_ACCUMULATOR_VALUES: &[f64] = &[
    100.0,
    400.0,
    1_000.0,
    3_000.0,
    10_000.0,
    25_000.0,
    75_000.0,
    150_000.0,
    400_000.0,
    1_000_000.0,
];

const FREE_OFFERING_UPGRADES_VALUES: &[f64] = &[1_000.0, 3_000.0, 9_000.0, 27_000.0, 81_000.0];

const FREE_OBTAINIUM_UPGRADES_VALUES: &[f64] = &[1_500.0, 4_500.0, 13_500.0, 40_500.0, 121_500.0];

const FREE_CUBE_UPGRADES_VALUES: &[f64] = &[10_000.0, 30_000.0, 90_000.0, 270_000.0, 810_000.0];

const FREE_SPEED_UPGRADES_VALUES: &[f64] = &[15_000.0, 45_000.0, 135_000.0, 405_000.0, 1_215_000.0];

fn lookup_cost(table: &[f64], level: u32) -> f64 {
    table.get(level as usize).copied().unwrap_or(0.0)
}

// ─── Per-upgrade costFormula functions ────────────────────────────────────

/// Tutorial cost: level has no effect.
#[must_use]
pub fn tutorial_cost_formula(_level: u32, base_cost: f64) -> f64 {
    base_cost
}

/// Conversion improvement 1: `base_cost × 2^level`.
#[must_use]
pub fn conversion_improvement_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * 2.0_f64.powi(level as i32)
}

/// Conversion improvement 2: `base_cost × 4^level`.
#[must_use]
pub fn conversion_improvement_2_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * 4.0_f64.powi(level as i32)
}

/// Conversion improvement 3: `base_cost × 10^level`.
#[must_use]
pub fn conversion_improvement_3_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * 10.0_f64.powi(level as i32)
}

/// Free tutorial levels: `base_cost + level`.
#[must_use]
pub fn free_tutorial_levels_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost + f64::from(level)
}

/// Free levels rows 2-5: `base_cost × 2^level`.
#[must_use]
pub fn free_levels_row_2_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * 2.0_f64.powi(level as i32)
}

/// Free levels row 3: same shape as row 2.
#[must_use]
pub fn free_levels_row_3_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * 2.0_f64.powi(level as i32)
}

/// Free levels row 4: same shape as row 2.
#[must_use]
pub fn free_levels_row_4_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * 2.0_f64.powi(level as i32)
}

/// Free levels row 5: same shape as row 2.
#[must_use]
pub fn free_levels_row_5_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * 2.0_f64.powi(level as i32)
}

/// Linear cost: `base_cost × (level + 1)`.
#[must_use]
pub fn blueberry_generation_speed_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * f64::from(level + 1)
}

/// Linear cost: `base_cost × (level + 1)`.
#[must_use]
pub fn regular_luck_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * f64::from(level + 1)
}

/// Linear cost: `base_cost × (level + 1)`.
#[must_use]
pub fn red_generation_speed_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * f64::from(level + 1)
}

/// Linear cost: `base_cost × (level + 1)`.
#[must_use]
pub fn red_luck_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * f64::from(level + 1)
}

/// Linear cost: `base_cost × (level + 1)`.
#[must_use]
pub fn red_ambrosia_cube_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * f64::from(level + 1)
}

/// Linear cost: `base_cost × (level + 1)`.
#[must_use]
pub fn red_ambrosia_obtainium_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * f64::from(level + 1)
}

/// Linear cost: `base_cost × (level + 1)`.
#[must_use]
pub fn red_ambrosia_offering_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * f64::from(level + 1)
}

/// Linear cost: `base_cost × (level + 1)`.
#[must_use]
pub fn red_ambrosia_cube_improver_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * f64::from(level + 1)
}

/// Linear cost: `base_cost × (level + 1)`.
#[must_use]
pub fn viscount_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * f64::from(level + 1)
}

/// Additive `100`-per-level cost: `base_cost + 100 × level`.
#[must_use]
pub fn infinite_shop_upgrades_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost + 100.0 * f64::from(level)
}

/// No-op cost: `base_cost` (level ignored).
#[must_use]
pub fn red_ambrosia_accelerator_cost_formula(_level: u32, base_cost: f64) -> f64 {
    base_cost
}

/// No-op cost.
#[must_use]
pub fn regular_luck_2_cost_formula(_level: u32, base_cost: f64) -> f64 {
    base_cost
}

/// No-op cost.
#[must_use]
pub fn blueberry_generation_speed_2_cost_formula(_level: u32, base_cost: f64) -> f64 {
    base_cost
}

/// Linear cost: `base_cost × (level + 1)`.
#[must_use]
pub fn salvage_yin_yang_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * f64::from(level + 1)
}

/// Table lookup; out-of-range returns `0`.
#[must_use]
pub fn blueberries_cost_formula(level: u32, _base_cost: f64) -> f64 {
    lookup_cost(BLUEBERRY_COST_VALUES, level)
}

/// Table lookup; out-of-range returns `0`.
#[must_use]
pub fn red_ambrosia_free_accumulator_cost_formula(level: u32, _base_cost: f64) -> f64 {
    lookup_cost(RED_AMBROSIA_FREE_ACCUMULATOR_VALUES, level)
}

/// Table lookup; out-of-range returns `0`.
#[must_use]
pub fn free_offering_upgrades_cost_formula(level: u32, _base_cost: f64) -> f64 {
    lookup_cost(FREE_OFFERING_UPGRADES_VALUES, level)
}

/// Table lookup; out-of-range returns `0`.
#[must_use]
pub fn free_obtainium_upgrades_cost_formula(level: u32, _base_cost: f64) -> f64 {
    lookup_cost(FREE_OBTAINIUM_UPGRADES_VALUES, level)
}

/// Table lookup; out-of-range returns `0`.
#[must_use]
pub fn free_cube_upgrades_cost_formula(level: u32, _base_cost: f64) -> f64 {
    lookup_cost(FREE_CUBE_UPGRADES_VALUES, level)
}

/// Table lookup; out-of-range returns `0`.
#[must_use]
pub fn free_speed_upgrades_cost_formula(level: u32, _base_cost: f64) -> f64 {
    lookup_cost(FREE_SPEED_UPGRADES_VALUES, level)
}

// ─── Per-upgrade effect functions ─────────────────────────────────────────

/// Tutorial effect — all three reward keys
/// (cubeMult / obtainiumMult / offeringMult) share `1.01^n`. The
/// legacy code ignores the key argument entirely.
#[must_use]
pub fn tutorial_effect(n: f64) -> f64 {
    1.01_f64.powf(n)
}

/// Conversion improvement 1 effect: `-n`.
#[must_use]
pub fn conversion_improvement_1_effect(n: f64) -> f64 {
    -n
}

/// Conversion improvement 2 effect: `-n`.
#[must_use]
pub fn conversion_improvement_2_effect(n: f64) -> f64 {
    -n
}

/// Conversion improvement 3 effect: `-n`.
#[must_use]
pub fn conversion_improvement_3_effect(n: f64) -> f64 {
    -n
}

/// Free tutorial levels effect: identity (`n`).
#[must_use]
pub fn free_tutorial_levels_effect(n: f64) -> f64 {
    n
}

/// Free levels row 2 effect: identity.
#[must_use]
pub fn free_levels_row_2_effect(n: f64) -> f64 {
    n
}

/// Free levels row 3 effect: identity.
#[must_use]
pub fn free_levels_row_3_effect(n: f64) -> f64 {
    n
}

/// Free levels row 4 effect: identity.
#[must_use]
pub fn free_levels_row_4_effect(n: f64) -> f64 {
    n
}

/// Free levels row 5 effect: identity.
#[must_use]
pub fn free_levels_row_5_effect(n: f64) -> f64 {
    n
}

/// Blueberry generation speed effect: `1 + n/500`.
#[must_use]
pub fn blueberry_generation_speed_effect(n: f64) -> f64 {
    1.0 + n / 500.0
}

/// Regular luck effect: `2n`.
#[must_use]
pub fn regular_luck_effect(n: f64) -> f64 {
    2.0 * n
}

/// Red generation speed effect: `1 + 3n / 1000`.
#[must_use]
pub fn red_generation_speed_effect(n: f64) -> f64 {
    1.0 + 3.0 * n / 1_000.0
}

/// Red luck effect: identity.
#[must_use]
pub fn red_luck_effect(n: f64) -> f64 {
    n
}

/// Red ambrosia cube unlock: `n > 0`.
#[must_use]
pub fn red_ambrosia_cube_effect(n: f64) -> bool {
    n > 0.0
}

/// Red ambrosia obtainium unlock: `n > 0`.
#[must_use]
pub fn red_ambrosia_obtainium_effect(n: f64) -> bool {
    n > 0.0
}

/// Red ambrosia offering unlock: `n > 0`.
#[must_use]
pub fn red_ambrosia_offering_effect(n: f64) -> bool {
    n > 0.0
}

/// Red ambrosia cube improver effect: `0.01 × n`.
#[must_use]
pub fn red_ambrosia_cube_improver_effect(n: f64) -> f64 {
    0.01 * n
}

/// Viscount upgrade — multi-key dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViscountEffectKey {
    /// Role-unlock flag (`n > 0`).
    RoleUnlock,
    /// Quark bonus (`1 + 0.1n`).
    QuarkBonus,
    /// Luck bonus (`125n`).
    LuckBonus,
    /// Red-luck bonus (`25n`).
    RedLuckBonus,
}

/// Tagged result for [`viscount_effect`].
#[derive(Debug, Clone, Copy)]
pub enum ViscountEffectValue {
    /// Role unlock bool.
    RoleUnlock(bool),
    /// Scalar value (other three keys).
    Scalar(f64),
}

/// Viscount effect dispatcher.
#[must_use]
pub fn viscount_effect(n: f64, key: ViscountEffectKey) -> ViscountEffectValue {
    match key {
        ViscountEffectKey::RoleUnlock => ViscountEffectValue::RoleUnlock(n > 0.0),
        ViscountEffectKey::QuarkBonus => ViscountEffectValue::Scalar(1.0 + 0.1 * n),
        ViscountEffectKey::LuckBonus => ViscountEffectValue::Scalar(125.0 * n),
        ViscountEffectKey::RedLuckBonus => ViscountEffectValue::Scalar(25.0 * n),
    }
}

/// Infinite shop upgrades effect: identity.
#[must_use]
pub fn infinite_shop_upgrades_effect(n: f64) -> f64 {
    n
}

/// Red ambrosia accelerator effect: `0.02n + 1` when `n > 0`, else
/// `0` (the `0.02n` term is zero at `n = 0`).
#[must_use]
pub fn red_ambrosia_accelerator_effect(n: f64) -> f64 {
    0.02 * n + if n > 0.0 { 1.0 } else { 0.0 }
}

/// Regular luck 2 effect: `2n`.
#[must_use]
pub fn regular_luck_2_effect(n: f64) -> f64 {
    2.0 * n
}

/// Blueberry generation speed 2 effect: `1 + n/1000`.
#[must_use]
pub fn blueberry_generation_speed_2_effect(n: f64) -> f64 {
    1.0 + n / 1_000.0
}

/// Salvage-yin-yang key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SalvageYinYangEffectKey {
    /// Positive salvage (`10n` unless taxmanLastStand is on).
    PositiveSalvage,
    /// Negative salvage (`-10n` unless taxmanLastStand is on).
    NegativeSalvage,
}

/// Salvage-yin-yang effect. Gated by `taxmanLastStand` — when the
/// challenge is enabled, both keys return `0`.
#[must_use]
pub fn salvage_yin_yang_effect(
    n: f64,
    key: SalvageYinYangEffectKey,
    taxman_last_stand_enabled: bool,
) -> f64 {
    if taxman_last_stand_enabled {
        return 0.0;
    }
    match key {
        SalvageYinYangEffectKey::PositiveSalvage => 10.0 * n,
        SalvageYinYangEffectKey::NegativeSalvage => -10.0 * n,
    }
}

/// Blueberries effect: identity (count of blueberries unlocked).
#[must_use]
pub fn blueberries_effect(n: f64) -> f64 {
    n
}

/// Free-accumulator key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedAmbrosiaFreeAccumulatorEffectKey {
    /// Free accumulator levels (`n/1000 + 0.01` if `n > 0`, else `0`).
    FreeAccumulatorLevels,
    /// Free accumulator level cap increase (`0.1n`).
    FreeAccumulatorLevelCapIncrease,
}

/// Red-ambrosia free-accumulator effect dispatcher.
#[must_use]
pub fn red_ambrosia_free_accumulator_effect(
    n: f64,
    key: RedAmbrosiaFreeAccumulatorEffectKey,
) -> f64 {
    match key {
        RedAmbrosiaFreeAccumulatorEffectKey::FreeAccumulatorLevels => {
            n / 1_000.0 + if n > 0.0 { 0.01 } else { 0.0 }
        }
        RedAmbrosiaFreeAccumulatorEffectKey::FreeAccumulatorLevelCapIncrease => 0.1 * n,
    }
}

/// Free offering upgrades effect: identity.
#[must_use]
pub fn free_offering_upgrades_effect(n: f64) -> f64 {
    n
}

/// Free obtainium upgrades effect: identity.
#[must_use]
pub fn free_obtainium_upgrades_effect(n: f64) -> f64 {
    n
}

/// Free cube upgrades effect: identity.
#[must_use]
pub fn free_cube_upgrades_effect(n: f64) -> f64 {
    n
}

/// Free speed upgrades effect: identity.
#[must_use]
pub fn free_speed_upgrades_effect(n: f64) -> f64 {
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tutorial_cost_is_noop() {
        assert_eq!(tutorial_cost_formula(100, 50.0), 50.0);
    }

    #[test]
    fn conversion_improvement_1_doubles_per_level() {
        assert_eq!(conversion_improvement_1_cost_formula(0, 10.0), 10.0);
        assert_eq!(conversion_improvement_1_cost_formula(3, 10.0), 80.0);
    }

    #[test]
    fn conversion_improvement_3_decuples_per_level() {
        assert_eq!(conversion_improvement_3_cost_formula(2, 1.0), 100.0);
    }

    #[test]
    fn linear_cost_formulas() {
        assert_eq!(regular_luck_cost_formula(0, 100.0), 100.0);
        assert_eq!(regular_luck_cost_formula(4, 100.0), 500.0);
    }

    #[test]
    fn blueberries_table_lookup() {
        assert_eq!(blueberries_cost_formula(0, 0.0), 100_000.0);
        assert_eq!(blueberries_cost_formula(4, 0.0), 3_500_000.0);
        assert_eq!(blueberries_cost_formula(5, 0.0), 0.0); // out of range
    }

    #[test]
    fn red_ambrosia_free_accumulator_table_lookup() {
        assert_eq!(red_ambrosia_free_accumulator_cost_formula(0, 0.0), 100.0);
        assert_eq!(
            red_ambrosia_free_accumulator_cost_formula(9, 0.0),
            1_000_000.0
        );
        assert_eq!(red_ambrosia_free_accumulator_cost_formula(99, 0.0), 0.0);
    }

    #[test]
    fn tutorial_effect_compounds_at_1p01() {
        assert_eq!(tutorial_effect(0.0), 1.0);
        assert!((tutorial_effect(100.0) - 1.01_f64.powi(100)).abs() < 1e-9);
    }

    #[test]
    fn conversion_improvement_effect_negates() {
        assert_eq!(conversion_improvement_1_effect(5.0), -5.0);
        assert_eq!(conversion_improvement_2_effect(10.0), -10.0);
        assert_eq!(conversion_improvement_3_effect(15.0), -15.0);
    }

    #[test]
    fn blueberry_generation_speed_effect_starts_at_1() {
        assert_eq!(blueberry_generation_speed_effect(0.0), 1.0);
        assert!((blueberry_generation_speed_effect(500.0) - 2.0).abs() < 1e-12);
    }

    #[test]
    fn red_ambrosia_cube_unlock_at_one() {
        assert!(!red_ambrosia_cube_effect(0.0));
        assert!(red_ambrosia_cube_effect(1.0));
    }

    #[test]
    fn viscount_role_unlock_at_one() {
        let result = viscount_effect(0.0, ViscountEffectKey::RoleUnlock);
        assert!(matches!(result, ViscountEffectValue::RoleUnlock(false)));
        let result = viscount_effect(1.0, ViscountEffectKey::RoleUnlock);
        assert!(matches!(result, ViscountEffectValue::RoleUnlock(true)));
    }

    #[test]
    fn viscount_quark_bonus() {
        let result = viscount_effect(5.0, ViscountEffectKey::QuarkBonus);
        match result {
            ViscountEffectValue::Scalar(v) => assert!((v - 1.5).abs() < 1e-12),
            ViscountEffectValue::RoleUnlock(_) => panic!("wrong variant"),
        }
    }

    #[test]
    fn red_ambrosia_accelerator_zero_at_zero() {
        assert_eq!(red_ambrosia_accelerator_effect(0.0), 0.0);
    }

    #[test]
    fn red_ambrosia_accelerator_adds_1_when_positive() {
        // 0.02*1 + 1 = 1.02
        assert!((red_ambrosia_accelerator_effect(1.0) - 1.02).abs() < 1e-12);
    }

    #[test]
    fn salvage_yin_yang_zeroed_by_taxman() {
        assert_eq!(
            salvage_yin_yang_effect(10.0, SalvageYinYangEffectKey::PositiveSalvage, true),
            0.0
        );
        assert_eq!(
            salvage_yin_yang_effect(10.0, SalvageYinYangEffectKey::NegativeSalvage, true),
            0.0
        );
    }

    #[test]
    fn salvage_yin_yang_positive_and_negative() {
        assert_eq!(
            salvage_yin_yang_effect(5.0, SalvageYinYangEffectKey::PositiveSalvage, false),
            50.0
        );
        assert_eq!(
            salvage_yin_yang_effect(5.0, SalvageYinYangEffectKey::NegativeSalvage, false),
            -50.0
        );
    }

    #[test]
    fn red_ambrosia_free_accumulator_levels_at_zero() {
        let v = red_ambrosia_free_accumulator_effect(
            0.0,
            RedAmbrosiaFreeAccumulatorEffectKey::FreeAccumulatorLevels,
        );
        assert_eq!(v, 0.0);
    }

    #[test]
    fn red_ambrosia_free_accumulator_levels_at_1000() {
        // 1000/1000 + 0.01 = 1.01
        let v = red_ambrosia_free_accumulator_effect(
            1_000.0,
            RedAmbrosiaFreeAccumulatorEffectKey::FreeAccumulatorLevels,
        );
        assert!((v - 1.01).abs() < 1e-12);
    }
}
