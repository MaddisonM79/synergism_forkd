//! Cube-upgrade cost + max-level math.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/cubeUpgrades.ts`
//! (migrated from the legacy `packages/web_ui/src/Cubes.ts`). The
//! UI-side getters (`cubeUpgradeDesc`, `updateCubeUpgradeBG`,
//! `buyCubeUpgrades`) stay in the UI — logic owns just the pure
//! cost-curve and cap math.

use crate::math::summations::{
    calculate_cubic_sum_data, calculate_summation_non_linear, SummationError,
};

// ─── Truth tables ──────────────────────────────────────────────────────────

/// Per-upgrade base cost (index `0` ignored — cube upgrades are
/// 1-indexed).
const CUBE_BASE_COSTS: [f64; 80] = [
    200.0,
    200.0,
    200.0,
    500.0,
    500.0,
    500.0,
    500.0,
    500.0,
    2_000.0,
    40_000.0,
    5_000.0,
    1_000.0,
    10_000.0,
    20_000.0,
    40_000.0,
    10_000.0,
    4_000.0,
    1e4,
    50_000.0,
    12_500.0,
    5e4,
    3e4,
    3e4,
    4e4,
    2e5,
    4e5,
    1e5,
    177_777.0,
    1e5,
    1e6,
    5e5,
    3e5,
    2e6,
    4e7,
    4e7,
    1e8,
    1e8,
    1e9,
    2e9,
    2e8,
    2e8,
    5e8,
    1e9,
    2e9,
    2e9,
    5e8,
    9_876_543_210.0,
    1e10,
    42_934_819_467.0,
    1e8,
    1.0,
    1e4,
    1e8,
    1e12,
    1e16,
    10.0,
    1e5,
    1e9,
    1e13,
    1e17,
    1e2,
    1e6,
    1e10,
    1e14,
    1e18,
    1e20,
    1e30,
    1e40,
    1e50,
    1e60,
    1.0,
    1.0,
    1e8,
    1e16,
    1e30,
    1e100,
    1e100,
    1e200,
    1e250,
    1e300,
];

/// Per-upgrade maximum level. Cube upgrade 57 (cookie row-leader
/// bonus) bumps indices `1 / 11 / 21 / 31 / 41` by `+1` — see
/// [`get_cube_max`].
const CUBE_MAX_LEVELS: [f64; 80] = [
    3.0, 10.0, 5.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 3.0, 10.0, 1.0, 10.0, 10.0, 10.0, 5.0, 1.0,
    1.0, 1.0, 5.0, 10.0, 1.0, 10.0, 10.0, 10.0, 1.0, 1.0, 5.0, 1.0, 5.0, 1.0, 1.0, 10.0, 10.0,
    10.0, 10.0, 1.0, 1.0, 10.0, 5.0, 10.0, 10.0, 10.0, 10.0, 20.0, 1.0, 1.0, 1.0, 100_000.0, 1.0,
    900.0, 100.0, 900.0, 900.0, 20.0, 1.0, 1.0, 400.0, 10_000.0, 100.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    1.0, 1_000.0, 1.0, 100_000.0, 1.0, 1.0, 5.0, 1.0, 30.0, 2.0, 25.0, 30.0, 1.0, 1.0,
];

/// Base cost (in wow cubes) for a single level of cube upgrade
/// `index` (1-indexed). The growth curve on top of this base differs
/// by tier — see [`get_cube_cost`].
#[must_use]
pub fn get_cube_upgrade_base_cost(index: u8) -> f64 {
    debug_assert!(
        matches!(index, 1..=80),
        "cube upgrade index out of range: {index}"
    );
    CUBE_BASE_COSTS[usize::from(index - 1)]
}

// ─── get_cube_max ──────────────────────────────────────────────────────────

/// Inputs to [`get_cube_max`].
#[derive(Debug, Clone, Copy)]
pub struct GetCubeMaxInput {
    /// `1..=80` (the cube upgrade index).
    pub cube_upgrade_index: u8,
    /// `player.cubeUpgrades[57]`. Once bought, the "row leader"
    /// upgrades (indices `1, 11, 21, 31, 41` — i.e.
    /// `i % 10 == 1` and `i < 50`) get `+1` max level.
    pub cube_upgrade_57: f64,
}

/// Per-upgrade maximum purchasable level.
#[must_use]
pub fn get_cube_max(input: &GetCubeMaxInput) -> f64 {
    let mut base_value = CUBE_MAX_LEVELS[usize::from(input.cube_upgrade_index - 1)];
    if input.cube_upgrade_57 > 0.0
        && input.cube_upgrade_index < 50
        && input.cube_upgrade_index % 10 == 1
    {
        base_value += 1.0;
    }
    base_value
}

// ─── get_cube_cost ─────────────────────────────────────────────────────────

/// Inputs to [`get_cube_cost`].
#[derive(Debug, Clone, Copy)]
pub struct GetCubeCostInput {
    /// `1..=80`.
    pub cube_upgrade_index: u8,
    /// `false` = buy 1 level; `true` = buy up to `1e5` (non-cubic) or
    /// max (cubic).
    pub buy_max: bool,
    /// `player.cubeUpgrades[i]`.
    pub current_level: f64,
    /// Precomputed via [`get_cube_max`]; avoids the wrapper
    /// double-computing it.
    pub max_level: f64,
    /// `Number(player.wowCubes)`.
    pub wow_cubes: f64,
    /// For `i <= 50`: `calculateSingularityDebuff('Cube Upgrades')`.
    /// For `i > 50`: pass `1` (the original code never applied the
    /// debuff above 50).
    pub singularity_debuff: f64,
}

/// Result of [`get_cube_cost`] — unified `{ level_can_buy, cost }`
/// shape that both summation primitives produce.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GetCubeCostResult {
    /// Highest level the player can reach.
    pub level_can_buy: f64,
    /// Cost in wow cubes to reach `level_can_buy` from the current
    /// level.
    pub cost: f64,
}

/// Cube cost curve. Three regimes:
///
/// - `i == 50` — linear growth `0.01` + singularity-debuffed base.
/// - `i < 50` (≠ 50) — flat per-level + singularity-debuffed base.
/// - `i > 50` — cubic sum, no singularity debuff.
///
/// Returns the unified `{ level_can_buy, cost }` shape. Callers feed
/// `cost` back into `player.wowCubes.sub()`.
///
/// Can return [`SummationError`] from the cubic branch — the legacy
/// code never checks for this, but propagating maintains the
/// `clippy::unwrap_used = "deny"` policy in `logic`.
pub fn get_cube_cost(input: &GetCubeCostInput) -> Result<GetCubeCostResult, SummationError> {
    let i = input.cube_upgrade_index;
    let lin_growth = if i == 50 { 0.01 } else { 0.0 };
    let cubic = i > 50;
    let cube_upgrade = input.current_level;
    let base_cost = CUBE_BASE_COSTS[usize::from(i - 1)];

    if cubic {
        // Cubic regime uses a different buy-amount rule: buy_max goes
        // up to max_level; single-purchase goes one above current.
        let amount_to_buy = if input.buy_max {
            input.max_level
        } else {
            input.max_level.min(cube_upgrade + 1.0)
        };
        let result =
            calculate_cubic_sum_data(cube_upgrade, base_cost, input.wow_cubes, amount_to_buy)?;
        return Ok(GetCubeCostResult {
            level_can_buy: result.level_can_buy,
            cost: result.cost,
        });
    }

    let mut amount_to_buy = if input.buy_max { 1e5 } else { 1.0 };
    amount_to_buy = (input.max_level - cube_upgrade).min(amount_to_buy);
    let result = calculate_summation_non_linear(
        cube_upgrade,
        base_cost * input.singularity_debuff,
        input.wow_cubes,
        lin_growth,
        amount_to_buy,
    );
    Ok(GetCubeCostResult {
        level_can_buy: result.level_can_buy,
        cost: result.cost,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_cost_lookup() {
        assert_eq!(get_cube_upgrade_base_cost(1), 200.0);
        assert_eq!(get_cube_upgrade_base_cost(10), 40_000.0);
        assert_eq!(get_cube_upgrade_base_cost(80), 1e300);
    }

    #[test]
    fn cube_max_bumps_row_leaders_with_upgrade_57() {
        let baseline = GetCubeMaxInput {
            cube_upgrade_index: 1,
            cube_upgrade_57: 0.0,
        };
        let with_57 = GetCubeMaxInput {
            cube_upgrade_index: 1,
            cube_upgrade_57: 1.0,
        };
        assert_eq!(get_cube_max(&baseline), 3.0);
        assert_eq!(get_cube_max(&with_57), 4.0);
    }

    #[test]
    fn cube_max_no_bump_for_non_row_leader() {
        let input = GetCubeMaxInput {
            cube_upgrade_index: 2,
            cube_upgrade_57: 1.0,
        };
        assert_eq!(get_cube_max(&input), 10.0);
    }

    #[test]
    fn cube_max_no_bump_above_50() {
        // Index 51 is in the cubic regime; upgrade 57 doesn't apply.
        let input = GetCubeMaxInput {
            cube_upgrade_index: 51,
            cube_upgrade_57: 1.0,
        };
        assert_eq!(get_cube_max(&input), 1.0);
    }

    #[test]
    fn cube_cost_below_50_uses_linear_summation() {
        // Index 1: base = 200, lin_growth = 0, sing_debuff = 1, max = 3.
        // Player has 0 owned, 1000 wow cubes, buy_max = false → buy 1.
        // calculate_summation_non_linear with diff_per_level = 0 →
        // simple linear: 1000 / 200 = 5 levels affordable, capped at 0+1=1.
        let result = get_cube_cost(&GetCubeCostInput {
            cube_upgrade_index: 1,
            buy_max: false,
            current_level: 0.0,
            max_level: 3.0,
            wow_cubes: 1_000.0,
            singularity_debuff: 1.0,
        })
        .unwrap();
        assert_eq!(result.level_can_buy, 1.0);
    }

    #[test]
    fn cube_cost_above_50_uses_cubic_summation() {
        // Index 51: cubic regime. base = 1, max = 1, current = 0.
        // amount_to_buy (non-buy-max) = min(1, 0+1) = 1.
        // total_to_spend = 0 + wow_cubes, solve for max level.
        let result = get_cube_cost(&GetCubeCostInput {
            cube_upgrade_index: 51,
            buy_max: false,
            current_level: 0.0,
            max_level: 1.0,
            wow_cubes: 100.0,
            singularity_debuff: 1.0, // ignored in cubic branch
        })
        .unwrap();
        // Cap at max=1 since single-purchase mode.
        assert!(result.level_can_buy <= 1.0);
    }

    #[test]
    fn cube_cost_50_uses_linear_growth_in_summation() {
        // Index 50 has lin_growth = 0.01 — should still produce a
        // sensible result.
        let result = get_cube_cost(&GetCubeCostInput {
            cube_upgrade_index: 50,
            buy_max: false,
            current_level: 0.0,
            max_level: 100_000.0,
            wow_cubes: 1e9,
            singularity_debuff: 1.0,
        })
        .unwrap();
        assert!(result.level_can_buy > 0.0);
    }
}
