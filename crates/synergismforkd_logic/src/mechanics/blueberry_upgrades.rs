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

use smallvec::SmallVec;

use crate::events::CoreEvent;
use crate::state::ambrosia::AMBROSIA_UPGRADES_LEN;
use crate::state::AmbrosiaState;

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

// ─── Cost dispatch + manual buy ───────────────────────────────────────────

/// Per-index `costFormula` dispatch table. Slot `i` is the cost formula for
/// ambrosia upgrade `i` — the array order IS the `AMBROSIA_*` index
/// convention (legacy `ambrosiaUpgrades` key order), so positions here must
/// stay in lockstep with `state::ambrosia`.
const AMBROSIA_COST_FORMULAS: [fn(u32, f64) -> f64; AMBROSIA_UPGRADES_LEN] = [
    ambrosia_tutorial_cost_formula,                  // 0  tutorial
    ambrosia_quarks_1_cost_formula,                  // 1  quarks1
    ambrosia_cubes_1_cost_formula,                   // 2  cubes1
    ambrosia_luck_1_cost_formula,                    // 3  luck1
    ambrosia_quark_cube_1_cost_formula,              // 4  quarkCube1
    ambrosia_luck_cube_1_cost_formula,               // 5  luckCube1
    ambrosia_cube_quark_1_cost_formula,              // 6  cubeQuark1
    ambrosia_luck_quark_1_cost_formula,              // 7  luckQuark1
    ambrosia_cube_luck_1_cost_formula,               // 8  cubeLuck1
    ambrosia_quark_luck_1_cost_formula,              // 9  quarkLuck1
    ambrosia_quarks_2_cost_formula,                  // 10 quarks2
    ambrosia_cubes_2_cost_formula,                   // 11 cubes2
    ambrosia_luck_2_cost_formula,                    // 12 luck2
    ambrosia_quarks_3_cost_formula,                  // 13 quarks3
    ambrosia_cubes_3_cost_formula,                   // 14 cubes3
    ambrosia_luck_3_cost_formula,                    // 15 luck3
    ambrosia_luck_4_cost_formula,                    // 16 luck4
    ambrosia_patreon_cost_formula,                   // 17 patreon
    ambrosia_obtainium_1_cost_formula,               // 18 obtainium1
    ambrosia_offering_1_cost_formula,                // 19 offering1
    ambrosia_hyperflux_cost_formula,                 // 20 hyperflux
    ambrosia_base_offering_1_cost_formula,           // 21 baseOffering1
    ambrosia_base_obtainium_1_cost_formula,          // 22 baseObtainium1
    ambrosia_base_offering_2_cost_formula,           // 23 baseOffering2
    ambrosia_base_obtainium_2_cost_formula,          // 24 baseObtainium2
    ambrosia_sing_reduction_1_cost_formula,          // 25 singReduction1
    ambrosia_infinite_shop_upgrades_1_cost_formula,  // 26 infiniteShopUpgrades1
    ambrosia_infinite_shop_upgrades_2_cost_formula,  // 27 infiniteShopUpgrades2
    ambrosia_sing_reduction_2_cost_formula,          // 28 singReduction2
    ambrosia_talisman_bonus_rune_level_cost_formula, // 29 talismanBonusRuneLevel
    ambrosia_rune_oom_bonus_cost_formula,            // 30 runeOOMBonus
    ambrosia_brick_of_lead_cost_formula,             // 31 brickOfLead
    ambrosia_free_luck_upgrades_cost_formula,        // 32 freeLuckUpgrades
    ambrosia_free_generation_upgrades_cost_formula,  // 33 freeGenerationUpgrades
    ambrosia_free_red_luck_upgrades_cost_formula,    // 34 freeRedLuckUpgrades
    ambrosia_free_quark_upgrades_cost_formula,       // 35 freeQuarkUpgrades
];

/// Cost of the next level of ambrosia upgrade `index`, via the upgrade's own
/// `costFormula(level, costPerLevel)`. The UI tier owns the `costPerLevel`
/// data table; the formula and the index→formula binding live here. `index`
/// out of range returns `0.0`.
#[must_use]
pub fn ambrosia_upgrade_cost(index: usize, level: u32, cost_per_level: f64) -> f64 {
    AMBROSIA_COST_FORMULAS
        .get(index)
        .map_or(0.0, |formula| formula(level, cost_per_level))
}

/// Inputs to [`buy_ambrosia_upgrade`].
#[derive(Debug, Clone, Copy)]
pub struct BuyAmbrosiaUpgradeInput {
    /// Ambrosia-upgrade index (`0..36`, via the `AMBROSIA_*` constants).
    /// Out-of-range is a no-op.
    pub index: usize,
    /// `ambrosiaUpgrades[key].costPerLevel` — per-upgrade base cost (UI-tier
    /// data-table value). Fed to the index-dispatched cost formula.
    pub cost_per_level: f64,
    /// `ambrosiaUpgrades[key].maxLevel` — the purchase cap (UI-tier). The
    /// `maxLevel <= 0` sentinel means unlimited; otherwise the buy stops once
    /// `level == max_level`.
    pub max_level: f64,
    /// `ambrosiaUpgrades[key].blueberryCost` — blueberry slots the upgrade
    /// occupies (UI-tier). Paid once, when the upgrade leaves level 0.
    pub blueberry_cost: f64,
    /// `calculateBlueberryInventory()` — total blueberries available (the
    /// reducer is UI-tier / unported, like `ambrosia_luck_3_effect`'s
    /// `blueberry_inventory`). Gates the first-level slot payment together
    /// with the already-spent count in state.
    pub blueberry_inventory: f64,
}

/// Buy one level of ambrosia upgrade `index` with ambrosia — the single-level
/// step of the legacy `buyAmbrosiaUpgradeLevel` loop. The cost is computed
/// logic-side via [`ambrosia_upgrade_cost`]; the caller supplies the static
/// `cost_per_level` / `max_level` / `blueberry_cost` data-table values and the
/// `blueberry_inventory` total (UI-tier). Spends `ambrosia.ambrosia`; the
/// first level out of level 0 also debits `blueberry_cost` to
/// `ambrosia.spent_blueberries`. Emits [`CoreEvent::AmbrosiaUpgradePurchased`].
///
/// Faithful-at-current-state deferrals:
/// - **prerequisites**: `checkAmbrosiaUpgradePrerequisites` walks the UI data
///   table, so — like the octeract/GQ unlock gates — the caller checks them
///   before dispatching; this buy is ungated on prerequisites;
/// - **buy-max**: the legacy loops `maxPurchasable` levels (a shift-click /
///   buy-amount prompt); this buys a single level (the buy-amount-1 case);
/// - `ambrosiaInvested` / `blueberriesInvested` respec tracking have no
///   logic-state fields (like the GQ buy's `goldenQuarksInvested`), so they
///   are not accumulated.
#[must_use]
pub fn buy_ambrosia_upgrade(
    ambrosia: &mut AmbrosiaState,
    input: BuyAmbrosiaUpgradeInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events = SmallVec::new();
    if input.index >= ambrosia.upgrades.len() {
        return events;
    }
    let before = ambrosia.upgrades[input.index].level;

    // Legacy gate: bounded upgrades stop at `maxLevel`; `maxLevel <= 0` is the
    // unlimited sentinel.
    let not_maxed = input.max_level <= 0.0 || before < input.max_level;
    if !not_maxed {
        return events;
    }

    let cost = ambrosia_upgrade_cost(input.index, before as u32, input.cost_per_level);
    if ambrosia.ambrosia < cost {
        return events;
    }

    // The blueberry-slot cost is paid once, when the upgrade leaves level 0.
    // Gated by the blueberries still free (inventory minus already-spent).
    if before == 0.0 {
        let available = input.blueberry_inventory - ambrosia.spent_blueberries;
        if available < input.blueberry_cost {
            return events;
        }
        ambrosia.spent_blueberries += input.blueberry_cost;
    }

    ambrosia.ambrosia -= cost;
    ambrosia.upgrades[input.index].level += 1.0;
    events.push(CoreEvent::AmbrosiaUpgradePurchased {
        index: input.index as u32,
        before,
        after: ambrosia.upgrades[input.index].level,
        spent: cost,
    });
    events
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

    // ─── Cost dispatch + buy ──────────────────────────────────────────────

    #[test]
    fn ambrosia_upgrade_cost_dispatches_by_index() {
        // Slot 0 = tutorial (quadratic diff), base 1: level 0 → 1, level 1 → 3.
        assert_eq!(ambrosia_upgrade_cost(0, 0, 1.0), 1.0);
        assert_eq!(ambrosia_upgrade_cost(0, 1, 1.0), 3.0);
        // Slot 1 = quarks1 (cubic diff), base 1: level 1 → (2^3 - 1^3) = 7.
        assert_eq!(ambrosia_upgrade_cost(1, 1, 1.0), 7.0);
        // Out of range → 0.
        assert_eq!(ambrosia_upgrade_cost(AMBROSIA_UPGRADES_LEN, 0, 1.0), 0.0);
    }

    fn amb_state(level: f64, ambrosia: f64) -> AmbrosiaState {
        let mut s = AmbrosiaState {
            ambrosia,
            ..AmbrosiaState::default()
        };
        s.upgrades[0].level = level;
        s
    }

    #[test]
    fn buy_ambrosia_upgrade_levels_up_and_spends() {
        // tutorial at level 0, costPerLevel 1 → cost 1; blueberryCost 0.
        let mut ambrosia = amb_state(0.0, 10.0);
        let events = buy_ambrosia_upgrade(
            &mut ambrosia,
            BuyAmbrosiaUpgradeInput {
                index: 0,
                cost_per_level: 1.0,
                max_level: 10.0,
                blueberry_cost: 0.0,
                blueberry_inventory: 0.0,
            },
        );
        assert_eq!(ambrosia.upgrades[0].level, 1.0);
        assert!((ambrosia.ambrosia - 9.0).abs() < 1e-9);
        assert!(matches!(
            events.as_slice(),
            [CoreEvent::AmbrosiaUpgradePurchased { index: 0, .. }]
        ));
    }

    #[test]
    fn buy_ambrosia_upgrade_unaffordable_is_noop() {
        let mut ambrosia = amb_state(0.0, 0.5);
        let events = buy_ambrosia_upgrade(
            &mut ambrosia,
            BuyAmbrosiaUpgradeInput {
                index: 0,
                cost_per_level: 1.0,
                max_level: 10.0,
                blueberry_cost: 0.0,
                blueberry_inventory: 0.0,
            },
        );
        assert_eq!(ambrosia.upgrades[0].level, 0.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_ambrosia_upgrade_maxed_is_noop() {
        let mut ambrosia = amb_state(10.0, 1e9);
        let events = buy_ambrosia_upgrade(
            &mut ambrosia,
            BuyAmbrosiaUpgradeInput {
                index: 0,
                cost_per_level: 1.0,
                max_level: 10.0,
                blueberry_cost: 0.0,
                blueberry_inventory: 0.0,
            },
        );
        assert_eq!(ambrosia.upgrades[0].level, 10.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_ambrosia_upgrade_first_level_pays_blueberry_slot() {
        // Leaving level 0 with blueberryCost 2 and 3 free blueberries: pays 2.
        let mut ambrosia = amb_state(0.0, 10.0);
        let events = buy_ambrosia_upgrade(
            &mut ambrosia,
            BuyAmbrosiaUpgradeInput {
                index: 0,
                cost_per_level: 1.0,
                max_level: 10.0,
                blueberry_cost: 2.0,
                blueberry_inventory: 3.0,
            },
        );
        assert_eq!(ambrosia.upgrades[0].level, 1.0);
        assert!((ambrosia.spent_blueberries - 2.0).abs() < 1e-9);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_ambrosia_upgrade_blocked_when_no_free_blueberries() {
        // blueberryCost 2 but only 1 free blueberry: no purchase, no spend.
        let mut ambrosia = amb_state(0.0, 10.0);
        let events = buy_ambrosia_upgrade(
            &mut ambrosia,
            BuyAmbrosiaUpgradeInput {
                index: 0,
                cost_per_level: 1.0,
                max_level: 10.0,
                blueberry_cost: 2.0,
                blueberry_inventory: 1.0,
            },
        );
        assert_eq!(ambrosia.upgrades[0].level, 0.0);
        assert_eq!(ambrosia.spent_blueberries, 0.0);
        assert!((ambrosia.ambrosia - 10.0).abs() < 1e-9);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_ambrosia_upgrade_above_level_0_skips_blueberry_slot() {
        // Already past level 0: the slot cost is not re-paid even with no free
        // blueberries. cost = quadratic_difference(5, 1) = 36 - 25 = 11.
        let mut ambrosia = amb_state(5.0, 100.0);
        let events = buy_ambrosia_upgrade(
            &mut ambrosia,
            BuyAmbrosiaUpgradeInput {
                index: 0,
                cost_per_level: 1.0,
                max_level: 10.0,
                blueberry_cost: 99.0,
                blueberry_inventory: 0.0,
            },
        );
        assert_eq!(ambrosia.upgrades[0].level, 6.0);
        assert_eq!(ambrosia.spent_blueberries, 0.0);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_ambrosia_upgrade_out_of_range_is_noop() {
        let mut ambrosia = amb_state(0.0, 1e9);
        let events = buy_ambrosia_upgrade(
            &mut ambrosia,
            BuyAmbrosiaUpgradeInput {
                index: AMBROSIA_UPGRADES_LEN,
                cost_per_level: 1.0,
                max_level: 10.0,
                blueberry_cost: 0.0,
                blueberry_inventory: 0.0,
            },
        );
        assert!(events.is_empty());
    }
}
