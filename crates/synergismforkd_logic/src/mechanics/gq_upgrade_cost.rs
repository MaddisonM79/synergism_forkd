//! GQ upgrade cost-to-next-level formula.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/gqUpgradeCost.ts`.
//! Four cost-form branches:
//!
//! - [`GQUpgradeSpecialCostForm::Exponential2`]:
//!   `cost_per_level × sqrt(overcap) × 2^level`
//! - [`GQUpgradeSpecialCostForm::Cubic`]:
//!   `cost_per_level × overcap × ((level+1)^3 - level^3)`
//! - [`GQUpgradeSpecialCostForm::Quadratic`]:
//!   `cost_per_level × overcap × ((level+1)^2 - level^2)`
//! - [`GQUpgradeSpecialCostForm::None`] (default linear):
//!   `ceil(cost_per_level × (level+1) × overcap × no_max_level_mult)`
//!
//! The overcap multiplier (`4^(level - max_level + 1)`) applies
//! whenever `computed_max_level` exceeds `max_level` (via
//! overclock-perks / octeract cap bonuses) AND the player is past
//! the base `max_level`. The default branch also has a no-max-level
//! progression: `max_level == -1` upgrades get multiplied by
//! `level/50` past level 100 and `level/100` past level 400.
//!
//! Returns `0` when `level == computed_max_level` (fully maxed).

/// Cost-form selector for a GQ upgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GQUpgradeSpecialCostForm {
    /// `Exponential2` shape — soft sqrt(overcap) scaling × `2^level`.
    Exponential2,
    /// `Cubic` shape — overcap × `((level+1)^3 - level^3)` delta.
    Cubic,
    /// `Quadratic` shape — overcap × `((level+1)^2 - level^2)` delta.
    Quadratic,
    /// Default linear branch with no-max-level progression.
    None,
}

/// Inputs to [`gq_upgrade_cost_tnl`].
#[derive(Debug, Clone, Copy)]
pub struct GQUpgradeCostTNLInput {
    /// `goldenQuarkUpgrades[k].level` — current purchased level.
    pub level: f64,
    /// `goldenQuarkUpgrades[k].maxLevel` — base cap (`-1` sentinel
    /// for unlimited).
    pub max_level: f64,
    /// `compute_gq_upgrade_max_level(k)` — base cap plus
    /// overclock-perks plus octeract cap bonus.
    pub computed_max_level: f64,
    /// `goldenQuarkUpgrades[k].costPerLevel` — base cost coefficient.
    pub cost_per_level: f64,
    /// `goldenQuarkUpgrades[k].specialCostForm`.
    pub special_cost_form: GQUpgradeSpecialCostForm,
}

/// Cost to buy the next level of a GQ upgrade.
///
/// Returns `0` if already maxed (`level == computed_max_level`).
#[must_use]
pub fn gq_upgrade_cost_tnl(input: &GQUpgradeCostTNLInput) -> f64 {
    if input.computed_max_level == input.level {
        return 0.0;
    }

    let mut cost_multiplier = 1.0_f64;

    if input.computed_max_level > input.max_level && input.level >= input.max_level {
        cost_multiplier *= 4.0_f64.powf(input.level - input.max_level + 1.0);
    }

    match input.special_cost_form {
        GQUpgradeSpecialCostForm::Exponential2 => {
            input.cost_per_level * cost_multiplier.sqrt() * 2.0_f64.powf(input.level)
        }
        GQUpgradeSpecialCostForm::Cubic => {
            input.cost_per_level
                * cost_multiplier
                * ((input.level + 1.0).powi(3) - input.level.powi(3))
        }
        GQUpgradeSpecialCostForm::Quadratic => {
            input.cost_per_level
                * cost_multiplier
                * ((input.level + 1.0).powi(2) - input.level.powi(2))
        }
        GQUpgradeSpecialCostForm::None => {
            // No-max-level progression
            if input.max_level == -1.0 && input.level >= 100.0 {
                cost_multiplier *= input.level / 50.0;
            }
            if input.max_level == -1.0 && input.level >= 400.0 {
                cost_multiplier *= input.level / 100.0;
            }
            (input.cost_per_level * (1.0 + input.level) * cost_multiplier).ceil()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_zero_when_fully_maxed() {
        let result = gq_upgrade_cost_tnl(&GQUpgradeCostTNLInput {
            level: 10.0,
            max_level: 10.0,
            computed_max_level: 10.0,
            cost_per_level: 100.0,
            special_cost_form: GQUpgradeSpecialCostForm::None,
        });
        assert_eq!(result, 0.0);
    }

    #[test]
    fn default_linear_no_overcap() {
        // level=0, cost=100, default → ceil(100 * 1 * 1) = 100
        let result = gq_upgrade_cost_tnl(&GQUpgradeCostTNLInput {
            level: 0.0,
            max_level: 10.0,
            computed_max_level: 10.0,
            cost_per_level: 100.0,
            special_cost_form: GQUpgradeSpecialCostForm::None,
        });
        assert_eq!(result, 100.0);
    }

    #[test]
    fn default_linear_with_overcap() {
        // level=10, max_level=10, computed=15 (past max), cost=100
        // overcap = 4^(10-10+1) = 4 → ceil(100 * 11 * 4) = 4400
        let result = gq_upgrade_cost_tnl(&GQUpgradeCostTNLInput {
            level: 10.0,
            max_level: 10.0,
            computed_max_level: 15.0,
            cost_per_level: 100.0,
            special_cost_form: GQUpgradeSpecialCostForm::None,
        });
        assert_eq!(result, 4_400.0);
    }

    #[test]
    fn exponential2_at_level_0_uses_cost_per_level() {
        let result = gq_upgrade_cost_tnl(&GQUpgradeCostTNLInput {
            level: 0.0,
            max_level: 10.0,
            computed_max_level: 10.0,
            cost_per_level: 100.0,
            special_cost_form: GQUpgradeSpecialCostForm::Exponential2,
        });
        // 100 * sqrt(1) * 2^0 = 100
        assert_eq!(result, 100.0);
    }

    #[test]
    fn cubic_uses_cube_delta() {
        // level=2, cost=10, no overcap → 10 * 1 * (27 - 8) = 190
        let result = gq_upgrade_cost_tnl(&GQUpgradeCostTNLInput {
            level: 2.0,
            max_level: 10.0,
            computed_max_level: 10.0,
            cost_per_level: 10.0,
            special_cost_form: GQUpgradeSpecialCostForm::Cubic,
        });
        assert_eq!(result, 190.0);
    }

    #[test]
    fn default_no_max_level_progression_at_100() {
        // max_level=-1, level=100, cost=1, computed=-1 → no overcap fires
        // costMult = 1 * (100/50) = 2 → ceil(1 * 101 * 2) = 202
        let result = gq_upgrade_cost_tnl(&GQUpgradeCostTNLInput {
            level: 100.0,
            max_level: -1.0,
            computed_max_level: -1.0,
            cost_per_level: 1.0,
            special_cost_form: GQUpgradeSpecialCostForm::None,
        });
        assert_eq!(result, 202.0);
    }

    #[test]
    fn default_no_max_level_progression_at_400() {
        // max_level=-1, level=400, computed=-1 → costMult = (400/50)*(400/100) = 32
        // ceil(1 * 401 * 32) = 12832
        let result = gq_upgrade_cost_tnl(&GQUpgradeCostTNLInput {
            level: 400.0,
            max_level: -1.0,
            computed_max_level: -1.0,
            cost_per_level: 1.0,
            special_cost_form: GQUpgradeSpecialCostForm::None,
        });
        assert_eq!(result, 12_832.0);
    }
}
