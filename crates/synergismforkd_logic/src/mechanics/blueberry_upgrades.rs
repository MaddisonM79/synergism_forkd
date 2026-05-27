//! Per-upgrade cost-formula and effect formulas for blueberry
//! (ambrosia) upgrades.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/blueberryUpgrades.ts`.
//! The UI tier still owns the `ambrosiaUpgrades` data table; this
//! module owns the two pure-formula fields each upgrade has:
//! `cost_formula(level, base_cost) -> f64` and
//! `effect(n, [key], ...) -> reward field`.
//!
//! Several effects read player state — those functions take the
//! player-derived value as an extra parameter; the UI data-table
//! closure forwards.

// ─── Shape helpers ────────────────────────────────────────────────────────

#[inline]
fn cubic_difference(level: u32, base_cost: f64) -> f64 {
    let l = f64::from(level);
    base_cost * ((l + 1.0).powi(3) - l.powi(3))
}

#[inline]
fn quadratic_difference(level: u32, base_cost: f64) -> f64 {
    let l = f64::from(level);
    base_cost * ((l + 1.0).powi(2) - l.powi(2))
}

// ─── Per-upgrade costFormula functions ────────────────────────────────────

/// Tutorial cost — quadratic difference shape.
#[must_use]
pub fn ambrosia_tutorial_cost_formula(level: u32, base_cost: f64) -> f64 {
    quadratic_difference(level, base_cost)
}

/// Quarks 1 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_quarks_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Cubes 1 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_cubes_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Luck 1 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_luck_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Quark-cube 1 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_quark_cube_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Luck-cube 1 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_luck_cube_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Cube-quark 1 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_cube_quark_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Luck-quark 1 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_luck_quark_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Cube-luck 1 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_cube_luck_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Quark-luck 1 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_quark_luck_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Quarks 2 cost — quadratic-difference shape.
#[must_use]
pub fn ambrosia_quarks_2_cost_formula(level: u32, base_cost: f64) -> f64 {
    quadratic_difference(level, base_cost)
}

/// Cubes 2 cost — quadratic-difference shape.
#[must_use]
pub fn ambrosia_cubes_2_cost_formula(level: u32, base_cost: f64) -> f64 {
    quadratic_difference(level, base_cost)
}

/// Luck 2 cost — quadratic-difference shape.
#[must_use]
pub fn ambrosia_luck_2_cost_formula(level: u32, base_cost: f64) -> f64 {
    quadratic_difference(level, base_cost)
}

/// Quarks 3 cost: `base_cost + 50000 × level`.
#[must_use]
pub fn ambrosia_quarks_3_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost + 50_000.0 * f64::from(level)
}

/// Cubes 3 cost: `base_cost + 5000 × level`.
#[must_use]
pub fn ambrosia_cubes_3_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost + 5_000.0 * f64::from(level)
}

/// Luck 3 cost: level has no effect.
#[must_use]
pub fn ambrosia_luck_3_cost_formula(_level: u32, base_cost: f64) -> f64 {
    base_cost
}

/// Luck 4 cost: `base_cost + 20000 × level`.
#[must_use]
pub fn ambrosia_luck_4_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost + 20_000.0 * f64::from(level)
}

/// Patreon cost — quadratic-difference shape.
#[must_use]
pub fn ambrosia_patreon_cost_formula(level: u32, base_cost: f64) -> f64 {
    quadratic_difference(level, base_cost)
}

/// Obtainium 1 cost: `base_cost × 25^level`.
#[must_use]
pub fn ambrosia_obtainium_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * 25.0_f64.powi(level as i32)
}

/// Offering 1 cost: `base_cost × 25^level`.
#[must_use]
pub fn ambrosia_offering_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * 25.0_f64.powi(level as i32)
}

/// Hyperflux cost: linear within first 4 levels, then exponential
/// via `max(1, 3^(level - 4))`.
#[must_use]
pub fn ambrosia_hyperflux_cost_formula(level: u32, base_cost: f64) -> f64 {
    let l = f64::from(level);
    let linear = base_cost + 33_333.0 * 4.0_f64.min(l);
    let exponential = 1.0_f64.max(3.0_f64.powf(l - 4.0));
    linear * exponential
}

/// Base offering 1 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_base_offering_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Base obtainium 1 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_base_obtainium_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Base offering 2 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_base_offering_2_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Base obtainium 2 cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_base_obtainium_2_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Singularity reduction 1 cost: `base_cost × 99^level`.
#[must_use]
pub fn ambrosia_sing_reduction_1_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * 99.0_f64.powi(level as i32)
}

/// Infinite shop upgrades 1 cost: no-op.
#[must_use]
pub fn ambrosia_infinite_shop_upgrades_1_cost_formula(_level: u32, base_cost: f64) -> f64 {
    base_cost
}

/// Infinite shop upgrades 2 cost: no-op.
#[must_use]
pub fn ambrosia_infinite_shop_upgrades_2_cost_formula(_level: u32, base_cost: f64) -> f64 {
    base_cost
}

/// Singularity reduction 2 cost: `base_cost × 3^level`.
#[must_use]
pub fn ambrosia_sing_reduction_2_cost_formula(level: u32, base_cost: f64) -> f64 {
    base_cost * 3.0_f64.powi(level as i32)
}

/// Talisman bonus rune level cost — quadratic-difference shape.
#[must_use]
pub fn ambrosia_talisman_bonus_rune_level_cost_formula(level: u32, base_cost: f64) -> f64 {
    quadratic_difference(level, base_cost)
}

/// Rune OOM bonus cost: `ceil(base_cost × ((level+1)^1.5 - level^1.5))`.
#[must_use]
pub fn ambrosia_rune_oom_bonus_cost_formula(level: u32, base_cost: f64) -> f64 {
    let l = f64::from(level);
    (base_cost * ((l + 1.0).powf(1.5) - l.powf(1.5))).ceil()
}

/// Brick of lead cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_brick_of_lead_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

/// Free luck upgrades cost — quadratic-difference shape.
#[must_use]
pub fn ambrosia_free_luck_upgrades_cost_formula(level: u32, base_cost: f64) -> f64 {
    quadratic_difference(level, base_cost)
}

/// Free generation upgrades cost: `base_cost × (10^(level+1) - 10^level)`.
#[must_use]
pub fn ambrosia_free_generation_upgrades_cost_formula(level: u32, base_cost: f64) -> f64 {
    let l = level as i32;
    base_cost * (10.0_f64.powi(l + 1) - 10.0_f64.powi(l))
}

/// Free red-luck upgrades cost — quadratic-difference shape.
#[must_use]
pub fn ambrosia_free_red_luck_upgrades_cost_formula(level: u32, base_cost: f64) -> f64 {
    quadratic_difference(level, base_cost)
}

/// Free quark upgrades cost — cubic-difference shape.
#[must_use]
pub fn ambrosia_free_quark_upgrades_cost_formula(level: u32, base_cost: f64) -> f64 {
    cubic_difference(level, base_cost)
}

// ─── Multi-key effect dispatchers ─────────────────────────────────────────

/// Tutorial effect key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbrosiaTutorialEffectKey {
    /// `cubes` reward (`1 + 0.05n`).
    Cubes,
    /// `quarks` reward (`1 + 0.01n`).
    Quarks,
}

/// Tutorial effect dispatcher.
#[must_use]
pub fn ambrosia_tutorial_effect(n: f64, key: AmbrosiaTutorialEffectKey) -> f64 {
    match key {
        AmbrosiaTutorialEffectKey::Cubes => 1.0 + 0.05 * n,
        AmbrosiaTutorialEffectKey::Quarks => 1.0 + 0.01 * n,
    }
}

/// Rune-OOM-bonus effect key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbrosiaRuneOOMBonusEffectKey {
    /// `runeOOMBonus` (identity, `n`).
    RuneOOMBonus,
    /// `infiniteAscentOOMBonus` (`n / 1000`).
    InfiniteAscentOOMBonus,
}

/// Rune-OOM-bonus effect dispatcher.
#[must_use]
pub fn ambrosia_rune_oom_bonus_effect(n: f64, key: AmbrosiaRuneOOMBonusEffectKey) -> f64 {
    match key {
        AmbrosiaRuneOOMBonusEffectKey::RuneOOMBonus => n,
        AmbrosiaRuneOOMBonusEffectKey::InfiniteAscentOOMBonus => n / 1_000.0,
    }
}

/// Brick-of-lead effect key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbrosiaBrickOfLeadEffectKey {
    /// `barRequirementMult` (`1 / (1 - n/50)`).
    BarRequirementMult,
    /// `additiveLuckMult` (`n / 50`).
    AdditiveLuckMult,
    /// `singularitySpeedMult` (`1 - n/100`).
    SingularitySpeedMult,
}

/// Brick-of-lead effect dispatcher.
#[must_use]
pub fn ambrosia_brick_of_lead_effect(n: f64, key: AmbrosiaBrickOfLeadEffectKey) -> f64 {
    match key {
        AmbrosiaBrickOfLeadEffectKey::BarRequirementMult => 1.0 / (1.0 - n / 50.0),
        AmbrosiaBrickOfLeadEffectKey::AdditiveLuckMult => n / 50.0,
        AmbrosiaBrickOfLeadEffectKey::SingularitySpeedMult => 1.0 - n / 100.0,
    }
}

// ─── Single-key effect functions ──────────────────────────────────────────

/// Quarks 1 effect: `1 + 0.01n`.
#[must_use]
pub fn ambrosia_quarks_1_effect(n: f64) -> f64 {
    1.0 + 0.01 * n
}

/// Cubes 1 effect: `+5%` per level plus `+10%` mult every 5 levels.
#[must_use]
pub fn ambrosia_cubes_1_effect(n: f64) -> f64 {
    (1.0 + 0.05 * n) * 1.1_f64.powf((n / 5.0).floor())
}

/// Luck 1 effect: `2n + 12 × floor(n / 10)`.
#[must_use]
pub fn ambrosia_luck_1_effect(n: f64) -> f64 {
    2.0 * n + 12.0 * (n / 10.0).floor()
}

/// Quark-cube 1 effect: cube bonus scales with
/// `floor((log10(worlds + 1) + 1)^2)`.
#[must_use]
pub fn ambrosia_quark_cube_1_effect(n: f64, worlds: f64) -> f64 {
    let base_val = 0.001 * n;
    1.0 + base_val * ((worlds + 1.0).log10() + 1.0).powi(2).floor()
}

/// Luck-cube 1 effect: cube bonus scales with `ambrosia_luck`.
#[must_use]
pub fn ambrosia_luck_cube_1_effect(n: f64, ambrosia_luck: f64) -> f64 {
    let base_val = 0.0005 * n;
    1.0 + base_val * ambrosia_luck
}

/// Cube-quark 1 effect: quark bonus scales with the precomputed
/// `wow_cube_log_sum + 6`.
#[must_use]
pub fn ambrosia_cube_quark_1_effect(n: f64, wow_cube_log_sum: f64) -> f64 {
    let base_val = 0.0001 * n;
    1.0 + base_val * (wow_cube_log_sum + 6.0)
}

/// Luck-quark 1 effect: quark bonus scales with
/// `min(luck, sqrt(1000 × luck))`.
#[must_use]
pub fn ambrosia_luck_quark_1_effect(n: f64, ambrosia_luck: f64) -> f64 {
    let base_val = 0.0001 * n;
    let effective_luck = ambrosia_luck.min(1_000.0_f64.powf(0.5) * ambrosia_luck.powf(0.5));
    1.0 + base_val * effective_luck
}

/// Cube-luck 1 effect: luck bonus scales with `wow_cube_log_sum + 6`.
#[must_use]
pub fn ambrosia_cube_luck_1_effect(n: f64, wow_cube_log_sum: f64) -> f64 {
    let base_val = 0.02 * n;
    base_val * (wow_cube_log_sum + 6.0)
}

/// Quark-luck 1 effect: luck bonus scales with
/// `floor((log10(worlds+1) + 1)^2)`.
#[must_use]
pub fn ambrosia_quark_luck_1_effect(n: f64, worlds: f64) -> f64 {
    let base_val = 0.02 * n;
    base_val * ((worlds + 1.0).log10() + 1.0).powi(2).floor()
}

/// Quarks 2 effect: scaled by `floor(quarks1 / 10) / 1000`.
#[must_use]
pub fn ambrosia_quarks_2_effect(n: f64, quarks_1_effective_levels: f64) -> f64 {
    1.0 + (0.01 + (quarks_1_effective_levels / 10.0).floor() / 1_000.0) * n
}

/// Cubes 2 effect: cubes-1 milestone scaling + ×1.15 every 5 levels.
#[must_use]
pub fn ambrosia_cubes_2_effect(n: f64, cubes_1_effective_levels: f64) -> f64 {
    (1.0 + (0.1 + 10.0 * (cubes_1_effective_levels / 10.0).floor() / 1_000.0) * n)
        * 1.15_f64.powf((n / 5.0).floor())
}

/// Luck 2 effect: scales with luck-1 milestones, +40 per 10 of `n`.
#[must_use]
pub fn ambrosia_luck_2_effect(n: f64, luck_1_effective_levels: f64) -> f64 {
    (3.0 + 0.3 * (luck_1_effective_levels / 10.0).floor()) * n + 40.0 * (n / 10.0).floor()
}

/// Quarks 3 effect: 5% per level multiplied by quarks-2 milestone.
#[must_use]
pub fn ambrosia_quarks_3_effect(n: f64, quarks_2_effective_levels: f64) -> f64 {
    let quark_2_mult = 1.0 + quarks_2_effective_levels / 100.0;
    let quark_3_base = 0.05 * n;
    1.0 + quark_3_base * quark_2_mult
}

/// Cubes 3 effect: 20% scaled by cubes-2 milestone + ×1.2 every 5 levels.
#[must_use]
pub fn ambrosia_cubes_3_effect(n: f64, cubes_2_effective_levels: f64) -> f64 {
    let cube_2_multi = 1.0 + 3.0 * cubes_2_effective_levels / 100.0;
    let cube_3_base = 0.2 * n;
    let cube_3_exponential = 1.2_f64.powf((n / 5.0).floor());
    (1.0 + cube_3_base * cube_2_multi) * cube_3_exponential
}

/// Luck 3 effect: linear, scaled by blueberry inventory.
#[must_use]
pub fn ambrosia_luck_3_effect(n: f64, blueberry_inventory: f64) -> f64 {
    blueberry_inventory * n
}

/// Luck 4 effect: luck percentage from the OOM digits of two
/// lifetime totals.
#[must_use]
pub fn ambrosia_luck_4_effect(n: f64, lifetime_red_ambrosia: f64, lifetime_ambrosia: f64) -> f64 {
    let digits =
        (lifetime_red_ambrosia + 1.0).log10().ceil() + (lifetime_ambrosia + 1.0).log10().ceil();
    digits * n / 10_000.0
}

/// Patreon effect: linear, scaled by current quark bonus.
#[must_use]
pub fn ambrosia_patreon_effect(n: f64, quark_bonus: f64) -> f64 {
    1.0 + (n * quark_bonus) / 100.0
}

/// Obtainium 1 effect: scales with ambrosia luck.
#[must_use]
pub fn ambrosia_obtainium_1_effect(n: f64, ambrosia_luck: f64) -> f64 {
    1.0 + n * ambrosia_luck / 1_000.0
}

/// Offering 1 effect: same formula as obtainium 1.
#[must_use]
pub fn ambrosia_offering_1_effect(n: f64, ambrosia_luck: f64) -> f64 {
    1.0 + n * ambrosia_luck / 1_000.0
}

/// Hyperflux effect: `(1 + n/100)^platonicUpgrade19`.
#[must_use]
pub fn ambrosia_hyperflux_effect(n: f64, platonic_upgrade_19: f64) -> f64 {
    (1.0 + n / 100.0).powf(platonic_upgrade_19)
}

/// Base offering 1 effect: identity.
#[must_use]
pub fn ambrosia_base_offering_1_effect(n: f64) -> f64 {
    n
}

/// Base obtainium 1 effect: identity.
#[must_use]
pub fn ambrosia_base_obtainium_1_effect(n: f64) -> f64 {
    n
}

/// Base offering 2 effect: identity.
#[must_use]
pub fn ambrosia_base_offering_2_effect(n: f64) -> f64 {
    n
}

/// Base obtainium 2 effect: identity.
#[must_use]
pub fn ambrosia_base_obtainium_2_effect(n: f64) -> f64 {
    n
}

/// Singularity reduction 1 effect: gated OFF while inside a
/// singularity challenge.
#[must_use]
pub fn ambrosia_sing_reduction_1_effect(n: f64, inside_singularity_challenge: bool) -> f64 {
    if inside_singularity_challenge {
        0.0
    } else {
        n
    }
}

/// Infinite shop upgrades 1 effect: identity.
#[must_use]
pub fn ambrosia_infinite_shop_upgrades_1_effect(n: f64) -> f64 {
    n
}

/// Infinite shop upgrades 2 effect: identity.
#[must_use]
pub fn ambrosia_infinite_shop_upgrades_2_effect(n: f64) -> f64 {
    n
}

/// Singularity reduction 2 effect: gated ON only while inside a
/// singularity challenge.
#[must_use]
pub fn ambrosia_sing_reduction_2_effect(n: f64, inside_singularity_challenge: bool) -> f64 {
    if inside_singularity_challenge {
        n
    } else {
        0.0
    }
}

/// Talisman bonus rune level effect: `n / 200`.
#[must_use]
pub fn ambrosia_talisman_bonus_rune_level_effect(n: f64) -> f64 {
    n / 200.0
}

/// Free luck upgrades effect: identity.
#[must_use]
pub fn ambrosia_free_luck_upgrades_effect(n: f64) -> f64 {
    n
}

/// Free generation upgrades effect: identity.
#[must_use]
pub fn ambrosia_free_generation_upgrades_effect(n: f64) -> f64 {
    n
}

/// Free red-luck upgrades effect: identity.
#[must_use]
pub fn ambrosia_free_red_luck_upgrades_effect(n: f64) -> f64 {
    n
}

/// Free quark upgrades effect: `n / 10`.
#[must_use]
pub fn ambrosia_free_quark_upgrades_effect(n: f64) -> f64 {
    n / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubic_difference_at_level_0_is_base_cost() {
        // base * (1^3 - 0^3) = base
        assert_eq!(cubic_difference(0, 10.0), 10.0);
    }

    #[test]
    fn cubic_difference_at_level_3() {
        // base * (4^3 - 3^3) = base * (64 - 27) = base * 37
        assert_eq!(cubic_difference(3, 1.0), 37.0);
    }

    #[test]
    fn quadratic_difference_at_level_2() {
        // base * (3^2 - 2^2) = base * 5
        assert_eq!(quadratic_difference(2, 1.0), 5.0);
    }

    #[test]
    fn hyperflux_cost_linear_below_level_4() {
        // level=0: (base + 0) * max(1, 3^-4) = base * 1 = base
        assert_eq!(ambrosia_hyperflux_cost_formula(0, 100.0), 100.0);
    }

    #[test]
    fn hyperflux_cost_exponential_above_level_4() {
        // level=5: (base + 33333*4) * 3 = (100 + 133332) * 3 = 400_296
        let result = ambrosia_hyperflux_cost_formula(5, 100.0);
        let expected = (100.0 + 33_333.0 * 4.0) * 3.0;
        assert_eq!(result, expected);
    }

    #[test]
    fn sing_reduction_1_cost_uses_99_pow_level() {
        assert_eq!(ambrosia_sing_reduction_1_cost_formula(0, 1.0), 1.0);
        assert_eq!(ambrosia_sing_reduction_1_cost_formula(2, 1.0), 99.0 * 99.0);
    }

    #[test]
    fn rune_oom_bonus_cost_ceils_floor() {
        // base=1.0, level=1: (2^1.5 - 1^1.5) = 1.828... → ceil = 2
        assert_eq!(ambrosia_rune_oom_bonus_cost_formula(1, 1.0), 2.0);
    }

    #[test]
    fn tutorial_effect_dispatch() {
        assert_eq!(
            ambrosia_tutorial_effect(10.0, AmbrosiaTutorialEffectKey::Cubes),
            1.5
        );
        assert_eq!(
            ambrosia_tutorial_effect(10.0, AmbrosiaTutorialEffectKey::Quarks),
            1.1
        );
    }

    #[test]
    fn brick_of_lead_bar_req_mult_inverts() {
        // n=25 → 1 / (1 - 25/50) = 1 / 0.5 = 2
        let result =
            ambrosia_brick_of_lead_effect(25.0, AmbrosiaBrickOfLeadEffectKey::BarRequirementMult);
        assert!((result - 2.0).abs() < 1e-12);
    }

    #[test]
    fn brick_of_lead_singularity_speed_mult() {
        let result =
            ambrosia_brick_of_lead_effect(50.0, AmbrosiaBrickOfLeadEffectKey::SingularitySpeedMult);
        assert!((result - 0.5).abs() < 1e-12);
    }

    #[test]
    fn cubes_1_effect_stair_steps_every_5() {
        // n=0 → 1 * 1 = 1
        // n=5 → 1.25 * 1.1 = 1.375
        let at_0 = ambrosia_cubes_1_effect(0.0);
        let at_5 = ambrosia_cubes_1_effect(5.0);
        assert_eq!(at_0, 1.0);
        assert!((at_5 - 1.375).abs() < 1e-12);
    }

    #[test]
    fn luck_1_effect_milestone_kick() {
        // n=10 → 20 + 12*1 = 32
        let result = ambrosia_luck_1_effect(10.0);
        assert_eq!(result, 32.0);
    }

    #[test]
    fn sing_reduction_1_gated_off_in_challenge() {
        assert_eq!(ambrosia_sing_reduction_1_effect(10.0, true), 0.0);
        assert_eq!(ambrosia_sing_reduction_1_effect(10.0, false), 10.0);
    }

    #[test]
    fn sing_reduction_2_gated_on_in_challenge() {
        assert_eq!(ambrosia_sing_reduction_2_effect(10.0, true), 10.0);
        assert_eq!(ambrosia_sing_reduction_2_effect(10.0, false), 0.0);
    }

    #[test]
    fn rune_oom_bonus_effect_dispatch() {
        assert_eq!(
            ambrosia_rune_oom_bonus_effect(50.0, AmbrosiaRuneOOMBonusEffectKey::RuneOOMBonus),
            50.0
        );
        assert!(
            (ambrosia_rune_oom_bonus_effect(
                500.0,
                AmbrosiaRuneOOMBonusEffectKey::InfiniteAscentOOMBonus
            ) - 0.5)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn luck_quark_1_effective_luck_caps_at_sqrt_1000_luck() {
        // For ambrosia_luck=10 → min(10, sqrt(1000*10)) = min(10, 100) = 10
        let result = ambrosia_luck_quark_1_effect(100.0, 10.0);
        // 1 + 0.01 * 10 = 1.1
        assert!((result - 1.1).abs() < 1e-12);
    }

    #[test]
    fn luck_quark_1_caps_with_sqrt_at_high_luck() {
        // For ambrosia_luck=100000 → sqrt(1e8) = 10000 < 100000 → use 10000
        let result = ambrosia_luck_quark_1_effect(1.0, 100_000.0);
        // 1 + 0.0001 * 10000 = 2
        assert!((result - 2.0).abs() < 1e-12);
    }
}
