//! Ant-upgrade base costs + pure effect formulas + cost solvers.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/antUpgrades.ts`.
//! Data table is indexed `0..=15` to match the legacy `AntUpgrades`
//! enum:
//!
//! `AntSpeed=0, Coins=1, Taxes=2, AcceleratorBoosts=3, Multipliers=4,
//! Offerings=5, BuildingCostScale=6, Salvage=7, FreeRunes=8,
//! Obtainium=9, AntSacrifice=10, Mortuus=11, AntELO=12, WowCubes=13,
//! AscensionScore=14, Mortuus2=15`.
//!
//! Cost shape: `cost-to-reach-level-N = base_cost × 10^((N-1) × exp)`;
//! cost-of-just-level-N is the delta from level-(N-1) to level-N. The
//! data table uses [`Decimal::from_mantissa_exponent`] because the
//! Mortuus2 base (1e37777) far exceeds `f64::MAX`.
//!
//! Effect functions: most are pure 1-arg formulas. AntSpeed / Coins /
//! AntELO read additional player state; those are parameterized
//! through input objects.

use crate::math::calculate_sigmoid_exponential;
use synergismforkd_bignum::Decimal;

// ─── Cost data (function-keyed because 1e37777 > f64::MAX) ────────────────

/// Base cost (level-0 → level-1) for upgrade `index`.
#[must_use]
pub fn ant_upgrade_base_cost(index: u8) -> Decimal {
    debug_assert!(
        matches!(index, 0..=15),
        "ant upgrade index out of range: {index}"
    );
    match index {
        0 => Decimal::from_finite(100.0),                    // AntSpeed
        1 => Decimal::from_finite(100.0),                    // Coins
        2 => Decimal::from_finite(1_000.0),                  // Taxes
        3 => Decimal::from_finite(1_000.0),                  // AcceleratorBoosts
        4 => Decimal::from_finite(1e5),                      // Multipliers
        5 => Decimal::from_finite(1e6),                      // Offerings
        6 => Decimal::from_finite(1e11),                     // BuildingCostScale
        7 => Decimal::from_finite(1e15),                     // Salvage
        8 => Decimal::from_finite(1e20),                     // FreeRunes
        9 => Decimal::from_finite(1e6),                      // Obtainium
        10 => Decimal::from_mantissa_exponent(1.0, 120.0),   // AntSacrifice
        11 => Decimal::from_mantissa_exponent(1.0, 300.0),   // Mortuus
        12 => Decimal::from_mantissa_exponent(1.0, 70.0),    // AntELO
        13 => Decimal::from_mantissa_exponent(1.0, 400.0),   // WowCubes
        14 => Decimal::from_mantissa_exponent(1.0, 300.0),   // AscensionScore
        _ => Decimal::from_mantissa_exponent(1.0, 37_777.0), // Mortuus2
    }
}

/// Per-level log-10 cost increase exponent for upgrade `index`.
#[must_use]
pub const fn ant_upgrade_cost_increase_exponent(index: u8) -> f64 {
    debug_assert!(matches!(index, 0..=15), "ant upgrade index out of range");
    match index {
        0..=3 => 1.0,
        4 | 5 | 6 | 9 | 14 => 2.0,
        7 | 8 => 3.0,
        12 => 4.0,
        13 => 10.0,
        10 => 20.0,
        11 => 100.0,
        _ => 2_000.0, // Mortuus2
    }
}

// ─── Cost solvers ─────────────────────────────────────────────────────────

/// Inputs to [`get_cost_next_ant_upgrade`].
#[derive(Debug, Clone, Copy)]
pub struct AntUpgradeCostInput {
    /// `ant_upgrade_base_cost(index)`.
    pub base_cost: Decimal,
    /// `ant_upgrade_cost_increase_exponent(index)`.
    pub cost_increase_exponent: f64,
    /// `player.ants.upgrades[index]` — current owned level.
    pub current_level: f64,
}

/// Cost of buying the next level. `cost-to-reach-N = base_cost ×
/// 10^((N-1) × exp)`; the delta from current to next is `next_cost -
/// last_cost` (with `last_cost = 0` when `current_level == 0`).
#[must_use]
pub fn get_cost_next_ant_upgrade(input: &AntUpgradeCostInput) -> Decimal {
    let ten = Decimal::from_finite(10.0);
    let next_cost = input.base_cost
        * ten.pow(Decimal::from_finite(
            input.current_level * input.cost_increase_exponent,
        ));
    let last_cost = if input.current_level > 0.0 {
        input.base_cost
            * ten.pow(Decimal::from_finite(
                (input.current_level - 1.0) * input.cost_increase_exponent,
            ))
    } else {
        Decimal::zero()
    };
    next_cost - last_cost
}

/// Inputs to [`get_max_purchasable_ant_upgrades`].
#[derive(Debug, Clone, Copy)]
pub struct AntUpgradeMaxPurchasableInput {
    /// `ant_upgrade_base_cost(index)`.
    pub base_cost: Decimal,
    /// `ant_upgrade_cost_increase_exponent(index)`.
    pub cost_increase_exponent: f64,
    /// `player.ants.upgrades[index]`.
    pub current_level: f64,
    /// Budget to spend (`player.ants.crumbs` in legacy).
    pub budget: Decimal,
}

/// Max level reachable with `budget`. Re-adds the sunk cost (cost
/// paid for the current level) then solves the inverse:
///
/// ```text
/// level = 1 + floor(log10(real_budget / base_cost) / exp)
/// ```
///
/// Floored at 0.
#[must_use]
pub fn get_max_purchasable_ant_upgrades(input: &AntUpgradeMaxPurchasableInput) -> f64 {
    let ten = Decimal::from_finite(10.0);
    let sunk_cost = if input.current_level > 0.0 {
        input.base_cost
            * ten.pow(Decimal::from_finite(
                input.cost_increase_exponent * (input.current_level - 1.0),
            ))
    } else {
        Decimal::zero()
    };
    let real_budget = input.budget + sunk_cost;
    0.0_f64.max(
        1.0 + ((real_budget / input.base_cost).log10().to_number() / input.cost_increase_exponent)
            .floor(),
    )
}

/// Inputs to [`get_cost_max_ant_upgrades`].
#[derive(Debug, Clone, Copy)]
pub struct AntUpgradeMaxCostInput {
    /// `ant_upgrade_base_cost(index)`.
    pub base_cost: Decimal,
    /// `ant_upgrade_cost_increase_exponent(index)`.
    pub cost_increase_exponent: f64,
    /// `player.ants.upgrades[index]`.
    pub current_level: f64,
    /// Result of [`get_max_purchasable_ant_upgrades`] for the same
    /// `base_cost / exp / current_level / budget`.
    pub max_buyable: f64,
}

/// Total cost to buy from `current_level` up to `max_buyable`. The
/// sunk cost (cost-of-current-level) is subtracted from
/// `cost-to-reach-max_buyable`.
#[must_use]
pub fn get_cost_max_ant_upgrades(input: &AntUpgradeMaxCostInput) -> Decimal {
    let ten = Decimal::from_finite(10.0);
    let spent = if input.current_level > 0.0 {
        ten.pow(Decimal::from_finite(
            input.cost_increase_exponent * (input.current_level - 1.0),
        )) * input.base_cost
    } else {
        Decimal::zero()
    };
    let max_cost = ten.pow(Decimal::from_finite(
        input.cost_increase_exponent * (input.max_buyable - 1.0),
    )) * input.base_cost;
    max_cost - spent
}

// ─── Pure effect functions (per upgrade) ──────────────────────────────────

/// Inputs to [`ant_speed_ant_upgrade_effect`].
#[derive(Debug, Clone, Copy)]
pub struct AntSpeedAntUpgradeInput {
    /// Current effective ant-upgrade level.
    pub level: f64,
    /// `player.researches[101]` — Research 5×1.
    pub research_101: f64,
    /// `player.researches[162]` — Research 7×12.
    pub research_162: f64,
}

/// AntSpeed (index 0): `1.1 + r101/1000 + r162/1000` raised to
/// `level`.
#[must_use]
pub fn ant_speed_ant_upgrade_effect(input: &AntSpeedAntUpgradeInput) -> Decimal {
    let base_mul = 1.1 + input.research_101 / 1_000.0 + input.research_162 / 1_000.0;
    Decimal::from_finite(base_mul).pow(Decimal::from_finite(input.level))
}

/// Inputs to [`coins_ant_upgrade_effect`].
#[derive(Debug, Clone, Copy)]
pub struct CoinsAntUpgradeInput {
    /// Current effective ant-upgrade level.
    pub level: f64,
    /// `player.currentChallenge.ascension` — modifies the divisor when
    /// `== 15`.
    pub ascension_challenge: u32,
    /// `player.ants.crumbs` — coin mult is `max(1, crumbs^exponent)`.
    pub crumbs: Decimal,
}

/// Result of [`coins_ant_upgrade_effect`].
#[derive(Debug, Clone, Copy)]
pub struct CoinsAntUpgradeEffect {
    /// Exponent applied to `crumbs` to produce the coin mult.
    pub crumb_to_coin_exp: f64,
    /// `max(1, crumbs^exponent)`.
    pub coin_multiplier: Decimal,
}

/// Coins (index 1): sigmoid-staircased exponent applied to crumbs,
/// then floor-clamped at 1.
#[must_use]
pub fn coins_ant_upgrade_effect(input: &CoinsAntUpgradeInput) -> CoinsAntUpgradeEffect {
    let n = input.level;
    let divisor = if input.ascension_challenge == 15 {
        100.0 + 9_900.0 * (1_000.0 + n) / (1_000.0 + n.powi(2))
    } else {
        1.0
    };
    let base_exponent = 999_999.0 + calculate_sigmoid_exponential(49_000_001.0, n / 3_000.0);
    let bonus_exponent = 250.0 * n;
    let exponent = (base_exponent + bonus_exponent) / divisor;
    let coin_mult = Decimal::one().max(input.crumbs.pow(Decimal::from_finite(exponent)));
    CoinsAntUpgradeEffect {
        crumb_to_coin_exp: exponent,
        coin_multiplier: coin_mult,
    }
}

/// Taxes (index 2): `0.005 + 0.995 * 0.99^level`.
#[must_use]
pub fn taxes_ant_upgrade_effect(level: f64) -> f64 {
    0.005 + 0.995 * 0.99_f64.powf(level)
}

/// AcceleratorBoosts (index 3): sigmoid-exp at coefficient `level/1000`.
#[must_use]
pub fn accelerator_boosts_ant_upgrade_effect(level: f64) -> f64 {
    calculate_sigmoid_exponential(20.0, level / 1_000.0)
}

/// Multipliers (index 4): sigmoid-exp at coefficient `level/1000`.
#[must_use]
pub fn multipliers_ant_upgrade_effect(level: f64) -> f64 {
    calculate_sigmoid_exponential(40.0, level / 1_000.0)
}

/// Offerings (index 5): `sqrt(1 + level/10)`.
#[must_use]
pub fn offerings_ant_upgrade_effect(level: f64) -> f64 {
    (1.0 + level / 10.0).powf(0.5)
}

/// Result of [`building_cost_scale_ant_upgrade_effect`].
#[derive(Debug, Clone, Copy)]
pub struct BuildingCostScaleAntUpgradeEffect {
    /// Cost-divisor scale applied to building costs.
    pub building_cost_scale: f64,
    /// Building-power multiplier (additive `1 + level/100`).
    pub building_power_mult: f64,
}

/// BuildingCostScale (index 6): `3*level/100` cost scale, `1 +
/// level/100` building-power mult.
#[must_use]
pub fn building_cost_scale_ant_upgrade_effect(level: f64) -> BuildingCostScaleAntUpgradeEffect {
    BuildingCostScaleAntUpgradeEffect {
        building_cost_scale: (3.0 * level) / 100.0,
        building_power_mult: 1.0 + level / 100.0,
    }
}

/// Salvage (index 7): `120 * (1 - 0.995^level)`.
#[must_use]
pub fn salvage_ant_upgrade_effect(level: f64) -> f64 {
    120.0 * (1.0 - 0.995_f64.powf(level))
}

/// FreeRunes (index 8): `3000 * (1 - (1 - 1/3000)^level)`.
#[must_use]
pub fn free_runes_ant_upgrade_effect(level: f64) -> f64 {
    3_000.0 * (1.0 - (1.0_f64 - 1.0 / 3_000.0).powf(level))
}

/// Obtainium (index 9): `sqrt(1 + level/10)`.
#[must_use]
pub fn obtainium_ant_upgrade_effect(level: f64) -> f64 {
    (1.0 + level / 10.0).powf(0.5)
}

/// Result of [`ant_sacrifice_ant_upgrade_effect`].
#[derive(Debug, Clone, Copy)]
pub struct AntSacrificeAntUpgradeEffect {
    /// `sqrt(1 + level/10)`.
    pub ant_sacrifice_multiplier: f64,
    /// `round(5 * min(200, level))`.
    pub elo: f64,
}

/// AntSacrifice (index 10): yields a multiplier and an ELO bonus.
#[must_use]
pub fn ant_sacrifice_ant_upgrade_effect(level: f64) -> AntSacrificeAntUpgradeEffect {
    AntSacrificeAntUpgradeEffect {
        ant_sacrifice_multiplier: (1.0 + level / 10.0).powf(0.5),
        elo: (5.0 * level.min(200.0)).round(),
    }
}

/// Result of [`mortuus_ant_upgrade_effect`].
#[derive(Debug, Clone, Copy)]
pub struct MortuusAntUpgradeEffect {
    /// Whether the talisman feature is unlocked (`level > 0`).
    pub talisman_unlock: bool,
    /// `2 - 0.99^level` — applied to global speed.
    pub global_speed: f64,
}

/// Mortuus (index 11): unlocks talismans, scales global speed.
#[must_use]
pub fn mortuus_ant_upgrade_effect(level: f64) -> MortuusAntUpgradeEffect {
    MortuusAntUpgradeEffect {
        talisman_unlock: level > 0.0,
        global_speed: 2.0 - 0.99_f64.powf(level),
    }
}

/// Inputs to [`ant_elo_ant_upgrade_effect`].
#[derive(Debug, Clone, Copy)]
pub struct AntELOAntUpgradeInput {
    /// Current effective ant-upgrade level.
    pub level: f64,
    /// `player.ants.antSacrificeCount`.
    pub ant_sacrifice_count: f64,
    /// `+getAchievementReward('antSpeed2UpgradeImprover')`.
    pub ant_speed_2_upgrade_improver: f64,
}

/// Result of [`ant_elo_ant_upgrade_effect`].
#[derive(Debug, Clone, Copy)]
pub struct AntELOAntUpgradeEffect {
    /// Effective sacrifice-derived ELO.
    pub ant_elo: f64,
    /// Hard limit on sac-count contribution (`n + 200·min(1, n)`).
    pub ant_sacrifice_limit_count: f64,
}

/// AntELO (index 12): caps the sac-count contribution against the
/// per-level ceiling and the achievement improver.
#[must_use]
pub fn ant_elo_ant_upgrade_effect(input: &AntELOAntUpgradeInput) -> AntELOAntUpgradeEffect {
    let n = input.level;
    let ant_sacrifice_limit_count = n + 200.0 * n.min(1.0);
    let upgrade_improver = n.min(input.ant_speed_2_upgrade_improver);
    let effective_sacs = (ant_sacrifice_limit_count + upgrade_improver)
        .min(input.ant_sacrifice_count + upgrade_improver);
    AntELOAntUpgradeEffect {
        ant_elo: effective_sacs,
        ant_sacrifice_limit_count,
    }
}

/// Result of [`mortuus_2_ant_upgrade_effect`].
#[derive(Debug, Clone, Copy)]
pub struct Mortuus2AntUpgradeEffect {
    /// `min(1200, floor(level/2))`.
    pub talisman_level_increaser: f64,
    /// `1 + 0.65*(1 - 0.999^level) + 0.005*min(20, level)`.
    pub talisman_effect_buff: f64,
    /// `1 + 0.5*(1 - 0.996^level)`.
    pub ascension_speed: f64,
}

/// Mortuus2 (index 15): three independent buffs to talismans /
/// ascension speed.
#[must_use]
pub fn mortuus_2_ant_upgrade_effect(level: f64) -> Mortuus2AntUpgradeEffect {
    Mortuus2AntUpgradeEffect {
        talisman_level_increaser: 1_200.0_f64.min((level / 2.0).floor()),
        talisman_effect_buff: 1.0 + 0.65 * (1.0 - 0.999_f64.powf(level)) + 0.005 * level.min(20.0),
        ascension_speed: 1.0 + 0.5 * (1.0 - 0.996_f64.powf(level)),
    }
}

/// Result of [`ascension_score_ant_upgrade_effect`].
#[derive(Debug, Clone, Copy)]
pub struct AscensionScoreAntUpgradeEffect {
    /// `100000 * (1 - 0.999^level)` — added to ascension-score base.
    pub ascension_score_base: f64,
    /// `3·min(200, level) + 2500·(1 - (1 - 1/2750)^level)
    ///   + 96900·(1 - (1 - 1/969000)^level)`.
    pub cubes_banked: f64,
}

/// AscensionScore (index 14).
#[must_use]
pub fn ascension_score_ant_upgrade_effect(level: f64) -> AscensionScoreAntUpgradeEffect {
    AscensionScoreAntUpgradeEffect {
        ascension_score_base: 100_000.0 * (1.0 - 0.999_f64.powf(level)),
        cubes_banked: 3.0 * level.min(200.0)
            + 2_500.0 * (1.0 - (1.0_f64 - 1.0 / 2_750.0).powf(level))
            + 96_900.0 * (1.0 - (1.0_f64 - 1.0 / 969_000.0).powf(level)),
    }
}

/// WowCubes (index 13): `2 - 0.999^level`.
#[must_use]
pub fn wow_cubes_ant_upgrade_effect(level: f64) -> f64 {
    2.0 - 0.999_f64.powf(level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_cost_index_0_is_100() {
        assert_eq!(ant_upgrade_base_cost(0).to_number(), 100.0);
    }

    #[test]
    fn base_cost_mortuus2_exceeds_f64() {
        let cost = ant_upgrade_base_cost(15);
        // 1e37777 → log10 ≈ 37777
        assert!((cost.log10().to_number() - 37_777.0).abs() < 1e-9);
    }

    #[test]
    fn cost_increase_exponents_match_table() {
        assert_eq!(ant_upgrade_cost_increase_exponent(0), 1.0);
        assert_eq!(ant_upgrade_cost_increase_exponent(4), 2.0);
        assert_eq!(ant_upgrade_cost_increase_exponent(7), 3.0);
        assert_eq!(ant_upgrade_cost_increase_exponent(10), 20.0);
        assert_eq!(ant_upgrade_cost_increase_exponent(11), 100.0);
        assert_eq!(ant_upgrade_cost_increase_exponent(12), 4.0);
        assert_eq!(ant_upgrade_cost_increase_exponent(13), 10.0);
        assert_eq!(ant_upgrade_cost_increase_exponent(15), 2_000.0);
    }

    #[test]
    fn next_cost_at_level_0_is_base_cost() {
        let result = get_cost_next_ant_upgrade(&AntUpgradeCostInput {
            base_cost: Decimal::from_finite(100.0),
            cost_increase_exponent: 1.0,
            current_level: 0.0,
        });
        assert_eq!(result.to_number(), 100.0);
    }

    #[test]
    fn next_cost_at_level_2_uses_delta() {
        // base=100, exp=1, level=2 → next = 100*10^2 = 10000; last = 100*10 = 1000
        // delta = 9000
        let result = get_cost_next_ant_upgrade(&AntUpgradeCostInput {
            base_cost: Decimal::from_finite(100.0),
            cost_increase_exponent: 1.0,
            current_level: 2.0,
        });
        assert_eq!(result.to_number(), 9_000.0);
    }

    #[test]
    fn max_purchasable_with_zero_budget_is_zero() {
        let result = get_max_purchasable_ant_upgrades(&AntUpgradeMaxPurchasableInput {
            base_cost: Decimal::from_finite(100.0),
            cost_increase_exponent: 1.0,
            current_level: 0.0,
            budget: Decimal::zero(),
        });
        assert!(result <= 0.0);
    }

    #[test]
    fn ant_speed_effect_at_level_0_is_one() {
        let result = ant_speed_ant_upgrade_effect(&AntSpeedAntUpgradeInput {
            level: 0.0,
            research_101: 0.0,
            research_162: 0.0,
        });
        // 1.1^0 = 1
        assert!((result.to_number() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn ant_speed_effect_at_level_10_compounds() {
        let result = ant_speed_ant_upgrade_effect(&AntSpeedAntUpgradeInput {
            level: 10.0,
            research_101: 0.0,
            research_162: 0.0,
        });
        // 1.1^10 ≈ 2.59374
        assert!((result.to_number() - 1.1_f64.powi(10)).abs() < 1e-9);
    }

    #[test]
    fn coins_effect_floor_clamps_at_1() {
        // Tiny crumbs → coin mult floor-clamped at 1
        let result = coins_ant_upgrade_effect(&CoinsAntUpgradeInput {
            level: 1.0,
            ascension_challenge: 0,
            crumbs: Decimal::from_finite(0.5),
        });
        assert!(result.coin_multiplier.to_number() >= 1.0);
    }

    #[test]
    fn coins_effect_uses_ascension_15_divisor() {
        let plain = coins_ant_upgrade_effect(&CoinsAntUpgradeInput {
            level: 100.0,
            ascension_challenge: 0,
            crumbs: Decimal::from_finite(1e3),
        });
        let in_c15 = coins_ant_upgrade_effect(&CoinsAntUpgradeInput {
            level: 100.0,
            ascension_challenge: 15,
            crumbs: Decimal::from_finite(1e3),
        });
        // Same crumbs but higher divisor → smaller exponent → smaller mult.
        assert!(in_c15.crumb_to_coin_exp < plain.crumb_to_coin_exp);
    }

    #[test]
    fn taxes_effect_at_zero_is_one() {
        assert!((taxes_ant_upgrade_effect(0.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn taxes_effect_floors_at_0p005() {
        // Very high level → 0.99^level → 0
        let result = taxes_ant_upgrade_effect(1e6);
        assert!((result - 0.005).abs() < 1e-9);
    }

    #[test]
    fn building_cost_scale_effect_splits_cleanly() {
        let result = building_cost_scale_ant_upgrade_effect(100.0);
        assert!((result.building_cost_scale - 3.0).abs() < 1e-12);
        assert!((result.building_power_mult - 2.0).abs() < 1e-12);
    }

    #[test]
    fn mortuus_effect_unlocks_at_level_1() {
        let zero = mortuus_ant_upgrade_effect(0.0);
        let one = mortuus_ant_upgrade_effect(1.0);
        assert!(!zero.talisman_unlock);
        assert!(one.talisman_unlock);
    }

    #[test]
    fn ant_sacrifice_effect_elo_caps_at_1000() {
        // 5*min(200, level) → max 1000 at level >=200
        let at_200 = ant_sacrifice_ant_upgrade_effect(200.0).elo;
        let at_500 = ant_sacrifice_ant_upgrade_effect(500.0).elo;
        assert_eq!(at_200, 1_000.0);
        assert_eq!(at_500, 1_000.0);
    }

    #[test]
    fn ant_elo_effect_takes_min_of_caps() {
        // n=50, sac_count=10, improver=0:
        // limit = 50 + 200*1 = 250; improver_eff = min(50, 0) = 0;
        // effective = min(250+0, 10+0) = 10
        let result = ant_elo_ant_upgrade_effect(&AntELOAntUpgradeInput {
            level: 50.0,
            ant_sacrifice_count: 10.0,
            ant_speed_2_upgrade_improver: 0.0,
        });
        assert_eq!(result.ant_elo, 10.0);
        assert_eq!(result.ant_sacrifice_limit_count, 250.0);
    }

    #[test]
    fn mortuus_2_effect_caps_talisman_level_increaser() {
        // floor(3000/2) = 1500 → capped at 1200
        let result = mortuus_2_ant_upgrade_effect(3_000.0);
        assert_eq!(result.talisman_level_increaser, 1_200.0);
    }

    #[test]
    fn wow_cubes_effect_caps_near_2() {
        let result = wow_cubes_ant_upgrade_effect(1e6);
        // 2 - 0.999^1e6 ≈ 2 - 0 = 2 (within f64 tolerance)
        assert!((result - 2.0).abs() < 1e-9);
    }
}
