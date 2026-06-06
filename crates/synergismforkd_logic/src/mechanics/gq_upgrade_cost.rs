//! GQ upgrade cost-to-next-level formula.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/gqUpgradeCost.ts`.
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

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::state::{GoldenQuarksState, StoredSpecialCostForm};

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

// ─── buy_gq_upgrade ─────────────────────────────────────────────────────────

impl From<StoredSpecialCostForm> for GQUpgradeSpecialCostForm {
    fn from(form: StoredSpecialCostForm) -> Self {
        match form {
            StoredSpecialCostForm::Exponential2 => Self::Exponential2,
            StoredSpecialCostForm::Cubic => Self::Cubic,
            StoredSpecialCostForm::Quadratic => Self::Quadratic,
            StoredSpecialCostForm::None => Self::None,
        }
    }
}

/// Inputs to [`buy_gq_upgrade`].
#[derive(Debug, Clone, Copy)]
pub struct BuyGQUpgradeInput {
    /// GQ-upgrade index (`0..80`, via the `GQ_*` constants). Out-of-range
    /// is a no-op.
    pub index: usize,
    /// `computeGQUpgradeMaxLevel(k)` — base cap + overclock perks + octeract
    /// cap bonus. Caller pre-evaluates (UI-tier); with no bonuses this equals
    /// the upgrade's `max_level`. Used for the maxed check and the cost
    /// overcap.
    pub computed_max_level: f64,
}

/// Buy one level of golden-quark upgrade `index` with golden quarks — the
/// per-level step of the legacy `singularity.ts` buy loop (the
/// `getGQUpgradeCostTNL` → spend → `level += 1` body). Every cost input is
/// read from the upgrade's own [`GoldenQuarkUpgrade`] state; only
/// `computed_max_level` is caller-provided. Emits
/// [`CoreEvent::GoldenQuarkUpgradePurchased`].
///
/// Faithful-at-current-state deferrals:
/// - **buy-max**: the legacy loops `maxPurchasable` levels (a buy-amount
///   toggle + budget cap); this buys a single level (the buy-amount-1 case);
/// - the `minimumSingularity` prerequisite is UI-tier (not in logic state),
///   so — like the other `buy_*` helpers — this is ungated on it;
/// - per-upgrade buy-time special effects (e.g. `oneMind` zeroing the
///   ascension counters) and `goldenQuarksInvested` respec tracking (no logic
///   field) are not applied — both unreachable at low singularity.
#[must_use]
pub fn buy_gq_upgrade(
    state: &mut GoldenQuarksState,
    input: BuyGQUpgradeInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events = SmallVec::new();
    if input.index >= state.upgrades.len() {
        return events;
    }
    let upgrade = state.upgrades[input.index];
    let cost = gq_upgrade_cost_tnl(&GQUpgradeCostTNLInput {
        level: upgrade.level,
        max_level: upgrade.max_level,
        computed_max_level: input.computed_max_level,
        cost_per_level: upgrade.cost_per_level,
        special_cost_form: upgrade.special_cost_form.into(),
    });

    // Legacy gate: not maxed AND affordable. The cap only applies to bounded
    // upgrades (`maxLevel > 0`); `maxLevel <= 0` is the unlimited sentinel.
    let not_maxed = upgrade.max_level <= 0.0 || upgrade.level < input.computed_max_level;
    if not_maxed && state.golden_quarks.to_number() >= cost {
        let before = upgrade.level;
        state.golden_quarks -= Decimal::from_finite(cost);
        state.upgrades[input.index].level += 1.0;
        events.push(CoreEvent::GoldenQuarkUpgradePurchased {
            index: input.index as u32,
            before,
            after: state.upgrades[input.index].level,
            spent: cost,
        });
    }
    events
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

    // ─── buy_gq_upgrade ──────────────────────────────────────────────────

    use crate::state::GoldenQuarkUpgrade;

    fn gq_state(golden_quarks: f64, cost_per_level: f64, max_level: f64) -> GoldenQuarksState {
        let mut state = GoldenQuarksState {
            golden_quarks: Decimal::from_finite(golden_quarks),
            ..GoldenQuarksState::default()
        };
        state.upgrades[0] = GoldenQuarkUpgrade {
            cost_per_level,
            max_level,
            ..GoldenQuarkUpgrade::default()
        };
        state
    }

    #[test]
    fn buy_gq_upgrade_levels_up_and_spends() {
        let mut state = gq_state(500.0, 100.0, 10.0);
        let events = buy_gq_upgrade(
            &mut state,
            BuyGQUpgradeInput {
                index: 0,
                computed_max_level: 10.0,
            },
        );
        // None-form at level 0: cost = ceil(100 * 1 * 1) = 100.
        assert_eq!(state.upgrades[0].level, 1.0);
        assert_eq!(state.golden_quarks.to_number(), 400.0);
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::GoldenQuarkUpgradePurchased {
                index,
                before,
                after,
                spent,
            } => {
                assert_eq!(*index, 0);
                assert_eq!(*before, 0.0);
                assert_eq!(*after, 1.0);
                assert_eq!(*spent, 100.0);
            }
            other => panic!("expected GoldenQuarkUpgradePurchased, got {other:?}"),
        }
    }

    #[test]
    fn buy_gq_upgrade_unaffordable_is_noop() {
        let mut state = gq_state(50.0, 100.0, 10.0); // 50 < cost 100
        let events = buy_gq_upgrade(
            &mut state,
            BuyGQUpgradeInput {
                index: 0,
                computed_max_level: 10.0,
            },
        );
        assert_eq!(state.upgrades[0].level, 0.0);
        assert_eq!(state.golden_quarks.to_number(), 50.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_gq_upgrade_maxed_is_noop() {
        let mut state = gq_state(1e9, 100.0, 10.0);
        state.upgrades[0].level = 10.0; // at the cap
        let events = buy_gq_upgrade(
            &mut state,
            BuyGQUpgradeInput {
                index: 0,
                computed_max_level: 10.0,
            },
        );
        assert_eq!(state.upgrades[0].level, 10.0);
        assert_eq!(state.golden_quarks.to_number(), 1e9);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_gq_upgrade_unlimited_ignores_cap() {
        // max_level == -1 (unlimited sentinel): the cap gate is skipped.
        let mut state = gq_state(1000.0, 1.0, -1.0);
        let events = buy_gq_upgrade(
            &mut state,
            BuyGQUpgradeInput {
                index: 0,
                computed_max_level: -1.0,
            },
        );
        assert_eq!(state.upgrades[0].level, 1.0);
        assert!(!events.is_empty());
    }

    #[test]
    fn buy_gq_upgrade_out_of_range_is_noop() {
        let mut state = gq_state(1e9, 100.0, 10.0);
        assert!(buy_gq_upgrade(
            &mut state,
            BuyGQUpgradeInput {
                index: 80,
                computed_max_level: 10.0,
            }
        )
        .is_empty());
    }
}
