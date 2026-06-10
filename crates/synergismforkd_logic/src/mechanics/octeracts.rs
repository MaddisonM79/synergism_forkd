//! Per-upgrade cost-formula + effect formulas for octeract upgrades.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/octeracts.ts`. The
//! UI tier owns the data table; this module owns the two
//! pure-formula fields each upgrade has: `cost_formula(level,
//! base_cost) -> f64` and `effect(n, [key], ...) -> reward field`.
//!
//! Five effects read mutable state outside the logic tier — those
//! take the extra value as a parameter.
//!
//! Beyond the formulas, this module owns the per-index cost dispatch
//! table ([`octeract_upgrade_cost`]) and the manual buy
//! ([`buy_octeract_upgrade`]) — the UI tier supplies only the static
//! `costPerLevel` / `maxLevel` data-table values.

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::state::octeract_upgrades::OCTERACT_UPGRADES_LEN;
use crate::state::OcteractUpgradesState;

// Cost-lookup table for octeractBlueberries — fixed sequence of
// costs per level. Mirrors the legacy `octeractBlueberryCostArr`.
const OCTERACT_BLUEBERRY_COST_ARR: &[f64] = &[1.0, 1e3, 1e9, 1e27, 1e81, 1e111];

// ─── Shape helpers ────────────────────────────────────────────────────────

fn ten_power_diff(level: f64, base_cost: f64) -> f64 {
    let use_level = level + 1.0;
    base_cost * (10.0_f64.powf(use_level) - 10.0_f64.powf(use_level - 1.0))
}

fn sixth_power_diff(level: f64, base_cost: f64) -> f64 {
    base_cost * ((level + 1.0).powi(6) - level.powi(6))
}

fn eighth_power_diff(level: f64, base_cost: f64) -> f64 {
    base_cost * ((level + 1.0).powi(8) - level.powi(8))
}

fn three_power_diff(level: f64, base_cost: f64) -> f64 {
    let use_level = level + 1.0;
    base_cost * (3.0_f64.powf(use_level) - 3.0_f64.powf(use_level - 1.0))
}

// ─── Per-upgrade costFormula functions ────────────────────────────────────

/// `octeractStarter` cost.
#[must_use]
pub fn octeract_starter_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * (level + 1.0)
}

/// `octeractGain` cost — 6th-power difference.
#[must_use]
pub fn octeract_gain_cost_formula(level: f64, base_cost: f64) -> f64 {
    sixth_power_diff(level, base_cost)
}

/// `octeractGain2` cost — `base × 10^(sqrt(level) / 3)`.
#[must_use]
pub fn octeract_gain_2_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 10.0_f64.powf(level.sqrt() / 3.0)
}

/// `octeractQuarkGain` cost — `(level+1)^7 - level^7` below 1000,
/// log blow-up past with two extra fastness multipliers at 10k and
/// 15k.
#[must_use]
pub fn octeract_quark_gain_cost_formula(level: f64, base_cost: f64) -> f64 {
    if level < 1_000.0 {
        return base_cost * ((level + 1.0).powi(7) - level.powi(7));
    }
    let faster_mult = if level >= 10_000.0 {
        10.0_f64.powf((level - 10_000.0) / 250.0)
    } else {
        1.0
    };
    let faster_mult_2 = if level >= 15_000.0 {
        10.0_f64.powf((level - 15_000.0) / 250.0)
    } else {
        1.0
    };
    base_cost
        * (1_001.0_f64.powi(7) - 1_000.0_f64.powi(7))
        * 10.0_f64.powf(level / 1_000.0)
        * faster_mult
        * faster_mult_2
}

/// `octeractQuarkGain2` cost — `base × 1e20^level`.
#[must_use]
pub fn octeract_quark_gain_2_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 1e20_f64.powf(level)
}

/// `octeractCorruption` cost — `base × 10^(level × 10)`.
#[must_use]
pub fn octeract_corruption_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 10.0_f64.powf(level * 10.0)
}

/// `octeractGQCostReduce` cost — `base × 2^level`.
#[must_use]
pub fn octeract_gq_cost_reduce_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 2.0_f64.powf(level)
}

/// `octeractExportQuarks` cost — `base × (level + 1)^3`.
#[must_use]
pub fn octeract_export_quarks_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * (level + 1.0).powi(3)
}

/// `octeractImprovedDaily` cost — `base × 1.6^level`.
#[must_use]
pub fn octeract_improved_daily_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 1.6_f64.powf(level)
}

/// `octeractImprovedDaily2` cost — `base × 2^level`.
#[must_use]
pub fn octeract_improved_daily_2_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 2.0_f64.powf(level)
}

/// `octeractImprovedDaily3` cost — `base × 20^level`.
#[must_use]
pub fn octeract_improved_daily_3_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 20.0_f64.powf(level)
}

/// `octeractImprovedQuarkHept` cost — `base × 1e3^level`.
#[must_use]
pub fn octeract_improved_quark_hept_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 1e3_f64.powf(level)
}

/// `octeractImprovedGlobalSpeed` cost — `(level + 1)^3`.
#[must_use]
pub fn octeract_improved_global_speed_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * (level + 1.0).powi(3)
}

/// `octeractImprovedAscensionSpeed` cost — `base × 1e9^(level/100)`.
#[must_use]
pub fn octeract_improved_ascension_speed_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 1e9_f64.powf(level / 100.0)
}

/// `octeractImprovedAscensionSpeed2` cost — `base × 1e12^(level/250)`.
#[must_use]
pub fn octeract_improved_ascension_speed_2_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 1e12_f64.powf(level / 250.0)
}

/// `octeractImprovedFree` cost — `(level + 1)^3`.
#[must_use]
pub fn octeract_improved_free_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * (level + 1.0).powi(3)
}

/// `octeractImprovedFree2` cost — `(level + 1)^3`.
#[must_use]
pub fn octeract_improved_free_2_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * (level + 1.0).powi(3)
}

/// `octeractImprovedFree3` cost — `(level + 1)^3`.
#[must_use]
pub fn octeract_improved_free_3_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * (level + 1.0).powi(3)
}

/// `octeractImprovedFree4` cost — `base × 1e20^(level/40)`.
#[must_use]
pub fn octeract_improved_free_4_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 1e20_f64.powf(level / 40.0)
}

/// `octeractSingUpgradeCap` cost — `base × 1e3^level`.
#[must_use]
pub fn octeract_sing_upgrade_cap_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 1e3_f64.powf(level)
}

fn octeract_offerings_obtainium_cost(level: f64, base_cost: f64) -> f64 {
    if level < 25.0 {
        return base_cost * (level + 1.0).powi(5);
    }
    base_cost * 1e15 * 10.0_f64.powf(level / 25.0 - 1.0)
}

/// `octeractOfferings1` cost — quintic below 25, log past.
#[must_use]
pub fn octeract_offerings_1_cost_formula(level: f64, base_cost: f64) -> f64 {
    octeract_offerings_obtainium_cost(level, base_cost)
}

/// `octeractObtainium1` cost — same shape as offerings.
#[must_use]
pub fn octeract_obtainium_1_cost_formula(level: f64, base_cost: f64) -> f64 {
    octeract_offerings_obtainium_cost(level, base_cost)
}

/// `octeractAscensions` cost — `(level + 1)^3`.
#[must_use]
pub fn octeract_ascensions_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * (level + 1.0).powi(3)
}

/// `octeractAscensions2` cost — `base × 10^(sqrt(level)/3)`.
#[must_use]
pub fn octeract_ascensions_2_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 10.0_f64.powf(level.sqrt() / 3.0)
}

/// `octeractAscensionsOcteractGain` cost — `base × 40^level`.
#[must_use]
pub fn octeract_ascensions_octeract_gain_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 40.0_f64.powf(level)
}

/// `octeractFastForward` cost — `base × 1e8^level`.
#[must_use]
pub fn octeract_fast_forward_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 1e8_f64.powf(level)
}

/// `octeractAutoPotionSpeed` cost — `base × 10^level`.
#[must_use]
pub fn octeract_auto_potion_speed_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 10.0_f64.powf(level)
}

/// `octeractAutoPotionEfficiency` cost — `base × 10^level`.
#[must_use]
pub fn octeract_auto_potion_efficiency_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 10.0_f64.powf(level)
}

/// `octeractOneMindImprover` cost — `1e5^level` then extra `1e3^(level - 10)`
/// past 10.
#[must_use]
pub fn octeract_one_mind_improver_cost_formula(level: f64, base_cost: f64) -> f64 {
    let faster_mult = if level >= 10.0 {
        1e3_f64.powf(level - 10.0)
    } else {
        1.0
    };
    base_cost * 1e5_f64.powf(level) * faster_mult
}

/// `octeractAmbrosiaLuck` cost — 10-power diff.
#[must_use]
pub fn octeract_ambrosia_luck_cost_formula(level: f64, base_cost: f64) -> f64 {
    ten_power_diff(level, base_cost)
}

/// `octeractAmbrosiaLuck2` cost — 6th-power diff.
#[must_use]
pub fn octeract_ambrosia_luck_2_cost_formula(level: f64, base_cost: f64) -> f64 {
    sixth_power_diff(level, base_cost)
}

/// `octeractAmbrosiaLuck3` cost — 8th-power diff.
#[must_use]
pub fn octeract_ambrosia_luck_3_cost_formula(level: f64, base_cost: f64) -> f64 {
    eighth_power_diff(level, base_cost)
}

/// `octeractAmbrosiaLuck4` cost — 3-power diff.
#[must_use]
pub fn octeract_ambrosia_luck_4_cost_formula(level: f64, base_cost: f64) -> f64 {
    three_power_diff(level, base_cost)
}

/// `octeractAmbrosiaGeneration` cost — 10-power diff.
#[must_use]
pub fn octeract_ambrosia_generation_cost_formula(level: f64, base_cost: f64) -> f64 {
    ten_power_diff(level, base_cost)
}

/// `octeractAmbrosiaGeneration2` cost — 6th-power diff.
#[must_use]
pub fn octeract_ambrosia_generation_2_cost_formula(level: f64, base_cost: f64) -> f64 {
    sixth_power_diff(level, base_cost)
}

/// `octeractAmbrosiaGeneration3` cost — 8th-power diff.
#[must_use]
pub fn octeract_ambrosia_generation_3_cost_formula(level: f64, base_cost: f64) -> f64 {
    eighth_power_diff(level, base_cost)
}

/// `octeractAmbrosiaGeneration4` cost — 3-power diff.
#[must_use]
pub fn octeract_ambrosia_generation_4_cost_formula(level: f64, base_cost: f64) -> f64 {
    three_power_diff(level, base_cost)
}

/// `octeractBonusTokens1` cost — `base × 100^level`.
#[must_use]
pub fn octeract_bonus_tokens_1_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 1e2_f64.powf(level)
}

/// `octeractBonusTokens2` cost — `base × 1e8^level`.
#[must_use]
pub fn octeract_bonus_tokens_2_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 1e8_f64.powf(level)
}

/// `octeractBonusTokens3` cost — `base × 1e10^level`.
#[must_use]
pub fn octeract_bonus_tokens_3_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 1e10_f64.powf(level)
}

/// `octeractBonusTokens4` cost — `base × 4^level`.
#[must_use]
pub fn octeract_bonus_tokens_4_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 4.0_f64.powf(level)
}

/// `octeractBlueberries` cost — fixed table lookup. Returns `0` once
/// the cap of 6 is reached.
#[must_use]
pub fn octeract_blueberries_cost_formula(level: f64, _base_cost: f64) -> f64 {
    if level == 6.0 {
        return 0.0;
    }
    let idx = level as usize;
    OCTERACT_BLUEBERRY_COST_ARR.get(idx).copied().unwrap_or(0.0)
}

/// `octeractInfiniteShopUpgrades` cost — `base × 16^level`.
#[must_use]
pub fn octeract_infinite_shop_upgrades_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 16.0_f64.powf(level)
}

/// `octeractTalismanLevelCap1` cost — `base × (level + 1)^5`.
#[must_use]
pub fn octeract_talisman_level_cap_1_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * (level + 1.0).powi(5)
}

/// `octeractTalismanLevelCap2` cost — `base × (level + 1)^10`.
#[must_use]
pub fn octeract_talisman_level_cap_2_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * (level + 1.0).powi(10)
}

/// `octeractTalismanLevelCap3` cost — `base × (level + 1)^20`.
#[must_use]
pub fn octeract_talisman_level_cap_3_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * (level + 1.0).powi(20)
}

/// `octeractTalismanLevelCap4` cost — `base × 10^level`.
#[must_use]
pub fn octeract_talisman_level_cap_4_cost_formula(level: f64, base_cost: f64) -> f64 {
    base_cost * 10.0_f64.powf(level)
}

// ─── Cost dispatch + manual buy ───────────────────────────────────────────

/// Per-index `costFormula` dispatch table. Slot `i` is the cost formula for
/// octeract upgrade `i` — the array order IS the `OCTERACT_*` index
/// convention (legacy `octeractUpgrades` key order), so the positions here
/// must stay in lockstep with `state::octeract_upgrades`.
const OCTERACT_COST_FORMULAS: [fn(f64, f64) -> f64; OCTERACT_UPGRADES_LEN] = [
    octeract_starter_cost_formula,                    // 0  octeractStarter
    octeract_gain_cost_formula,                       // 1  octeractGain
    octeract_gain_2_cost_formula,                     // 2  octeractGain2
    octeract_quark_gain_cost_formula,                 // 3  octeractQuarkGain
    octeract_quark_gain_2_cost_formula,               // 4  octeractQuarkGain2
    octeract_corruption_cost_formula,                 // 5  octeractCorruption
    octeract_gq_cost_reduce_cost_formula,             // 6  octeractGQCostReduce
    octeract_export_quarks_cost_formula,              // 7  octeractExportQuarks
    octeract_improved_daily_cost_formula,             // 8  octeractImprovedDaily
    octeract_improved_daily_2_cost_formula,           // 9  octeractImprovedDaily2
    octeract_improved_daily_3_cost_formula,           // 10 octeractImprovedDaily3
    octeract_improved_quark_hept_cost_formula,        // 11 octeractImprovedQuarkHept
    octeract_improved_global_speed_cost_formula,      // 12 octeractImprovedGlobalSpeed
    octeract_improved_ascension_speed_cost_formula,   // 13 octeractImprovedAscensionSpeed
    octeract_improved_ascension_speed_2_cost_formula, // 14 octeractImprovedAscensionSpeed2
    octeract_improved_free_cost_formula,              // 15 octeractImprovedFree
    octeract_improved_free_2_cost_formula,            // 16 octeractImprovedFree2
    octeract_improved_free_3_cost_formula,            // 17 octeractImprovedFree3
    octeract_improved_free_4_cost_formula,            // 18 octeractImprovedFree4
    octeract_sing_upgrade_cap_cost_formula,           // 19 octeractSingUpgradeCap
    octeract_offerings_1_cost_formula,                // 20 octeractOfferings1
    octeract_obtainium_1_cost_formula,                // 21 octeractObtainium1
    octeract_ascensions_cost_formula,                 // 22 octeractAscensions
    octeract_ascensions_2_cost_formula,               // 23 octeractAscensions2
    octeract_ascensions_octeract_gain_cost_formula,   // 24 octeractAscensionsOcteractGain
    octeract_fast_forward_cost_formula,               // 25 octeractFastForward
    octeract_auto_potion_speed_cost_formula,          // 26 octeractAutoPotionSpeed
    octeract_auto_potion_efficiency_cost_formula,     // 27 octeractAutoPotionEfficiency
    octeract_one_mind_improver_cost_formula,          // 28 octeractOneMindImprover
    octeract_ambrosia_luck_cost_formula,              // 29 octeractAmbrosiaLuck
    octeract_ambrosia_luck_2_cost_formula,            // 30 octeractAmbrosiaLuck2
    octeract_ambrosia_luck_3_cost_formula,            // 31 octeractAmbrosiaLuck3
    octeract_ambrosia_luck_4_cost_formula,            // 32 octeractAmbrosiaLuck4
    octeract_ambrosia_generation_cost_formula,        // 33 octeractAmbrosiaGeneration
    octeract_ambrosia_generation_2_cost_formula,      // 34 octeractAmbrosiaGeneration2
    octeract_ambrosia_generation_3_cost_formula,      // 35 octeractAmbrosiaGeneration3
    octeract_ambrosia_generation_4_cost_formula,      // 36 octeractAmbrosiaGeneration4
    octeract_bonus_tokens_1_cost_formula,             // 37 octeractBonusTokens1
    octeract_bonus_tokens_2_cost_formula,             // 38 octeractBonusTokens2
    octeract_bonus_tokens_3_cost_formula,             // 39 octeractBonusTokens3
    octeract_bonus_tokens_4_cost_formula,             // 40 octeractBonusTokens4
    octeract_blueberries_cost_formula,                // 41 octeractBlueberries
    octeract_infinite_shop_upgrades_cost_formula,     // 42 octeractInfiniteShopUpgrades
    octeract_talisman_level_cap_1_cost_formula,       // 43 octeractTalismanLevelCap1
    octeract_talisman_level_cap_2_cost_formula,       // 44 octeractTalismanLevelCap2
    octeract_talisman_level_cap_3_cost_formula,       // 45 octeractTalismanLevelCap3
    octeract_talisman_level_cap_4_cost_formula,       // 46 octeractTalismanLevelCap4
];

/// Cost of the next level of octeract upgrade `index`, via the upgrade's own
/// `costFormula(level, costPerLevel)`. The UI tier owns the `costPerLevel`
/// data table; the formula and the index→formula binding live here. `index`
/// out of range returns `0.0`.
#[must_use]
pub fn octeract_upgrade_cost(index: usize, level: f64, cost_per_level: f64) -> f64 {
    OCTERACT_COST_FORMULAS
        .get(index)
        .map_or(0.0, |formula| formula(level, cost_per_level))
}

/// Inputs to [`buy_octeract_upgrade`].
#[derive(Debug, Clone, Copy)]
pub struct BuyOcteractUpgradeInput {
    /// Octeract-upgrade index (`0..47`, via the `OCTERACT_*` constants).
    /// Out-of-range is a no-op.
    pub index: usize,
    /// `octeractUpgrades[key].costPerLevel` — the per-upgrade base cost
    /// (UI-tier data-table value). Fed to the index-dispatched cost formula.
    pub cost_per_level: f64,
    /// `octeractUpgrades[key].maxLevel` — the purchase cap (UI-tier). The
    /// `maxLevel <= 0` sentinel means unlimited; otherwise the buy stops once
    /// `level == max_level`.
    pub max_level: f64,
}

/// Buy one level of octeract upgrade `index` with octeracts — the
/// single-level step of the legacy `buyOcteractUpgradeLevel` loop (the
/// `costFormula` → spend → `level += 1` body, buy-amount 1). The cost is
/// computed logic-side via [`octeract_upgrade_cost`]; only the static
/// `cost_per_level` / `max_level` data-table values are caller-provided
/// (UI-tier). Emits [`CoreEvent::OcteractUpgradePurchased`].
///
/// Faithful-at-current-state deferrals:
/// - **buy-max**: the legacy loops `maxPurchasable` levels (a shift-click /
///   buy-amount prompt); this buys a single level (the buy-amount-1 case);
/// - `octeractsInvested` respec tracking has no logic-state field (like the
///   GQ buy's `goldenQuarksInvested`), so it is not accumulated.
#[must_use]
pub fn buy_octeract_upgrade(
    upgrades: &mut OcteractUpgradesState,
    wow_octeracts: &mut Decimal,
    input: BuyOcteractUpgradeInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events = SmallVec::new();
    if input.index >= upgrades.upgrades.len() {
        return events;
    }
    let before = upgrades.upgrades[input.index].level;

    // Legacy gate: bounded upgrades stop at `maxLevel`; `maxLevel <= 0` is the
    // unlimited sentinel.
    let not_maxed = input.max_level <= 0.0 || before < input.max_level;
    if !not_maxed {
        return events;
    }

    let cost = octeract_upgrade_cost(input.index, before, input.cost_per_level);
    if wow_octeracts.to_number() >= cost {
        *wow_octeracts -= Decimal::from_finite(cost);
        upgrades.upgrades[input.index].level += 1.0;
        events.push(CoreEvent::OcteractUpgradePurchased {
            index: input.index as u32,
            before,
            after: upgrades.upgrades[input.index].level,
            spent: cost,
        });
    }
    events
}

// ─── Per-upgrade effect functions ─────────────────────────────────────────

/// `octeractStarter` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcteractStarterKey {
    /// `quarkMult` — `1 + 0.25n`.
    QuarkMult,
    /// `antSpeedMult` — `1 + 99999n`.
    AntSpeedMult,
    /// `octeractMult` — `1 + 0.4n`.
    OcteractMult,
}

/// `octeractStarter` effect.
#[must_use]
pub fn octeract_starter_effect(n: f64, key: OcteractStarterKey) -> f64 {
    match key {
        OcteractStarterKey::QuarkMult => 1.0 + 0.25 * n,
        OcteractStarterKey::AntSpeedMult => 1.0 + 99_999.0 * n,
        OcteractStarterKey::OcteractMult => 1.0 + 0.4 * n,
    }
}

/// `octeractGain` effect — `1 + 0.01n`.
#[must_use]
pub fn octeract_gain_effect(n: f64) -> f64 {
    1.0 + 0.01 * n
}

/// `octeractGain2` effect — `1 + 0.01n`.
#[must_use]
pub fn octeract_gain_2_effect(n: f64) -> f64 {
    1.0 + 0.01 * n
}

/// `octeractQuarkGain` effect — `1 + 0.011n`.
#[must_use]
pub fn octeract_quark_gain_effect(n: f64) -> f64 {
    1.0 + 0.011 * n
}

/// `octeractQuarkGain2` effect — quark bonus scales with sibling
/// `quarkGainLevel` and `log10(max(1, hepteractQuarkBAL))`.
#[must_use]
pub fn octeract_quark_gain_2_effect(
    n: f64,
    quark_gain_level: f64,
    hepteract_quark_bal: f64,
) -> f64 {
    1.0 + (1.0 / 10_000.0)
        * (quark_gain_level / 111.0).floor()
        * n
        * (1.0 + 1.0_f64.max(hepteract_quark_bal).log10()).floor()
}

/// `octeractCorruption` effect — identity (corruption level cap).
#[must_use]
pub fn octeract_corruption_effect(n: f64) -> f64 {
    n
}

/// `octeractGQCostReduce` effect — `1 - n / 100`.
#[must_use]
pub fn octeract_gq_cost_reduce_effect(n: f64) -> f64 {
    1.0 - n / 100.0
}

/// `octeractExportQuarks` effect — `4n / 10 + 1`.
#[must_use]
pub fn octeract_export_quarks_effect(n: f64) -> f64 {
    4.0 * n / 10.0 + 1.0
}

/// `octeractImprovedDaily` effect — identity.
#[must_use]
pub fn octeract_improved_daily_effect(n: f64) -> f64 {
    n
}

/// `octeractImprovedDaily2` effect — `1 + 0.01n`.
#[must_use]
pub fn octeract_improved_daily_2_effect(n: f64) -> f64 {
    1.0 + 0.01 * n
}

/// `octeractImprovedDaily3` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcteractImprovedDaily3Key {
    /// `goldenQuarkMult` — `1 + 0.005n`.
    GoldenQuarkMult,
    /// `extraGoldenQuarks` — `n`.
    ExtraGoldenQuarks,
}

/// `octeractImprovedDaily3` effect.
#[must_use]
pub fn octeract_improved_daily_3_effect(n: f64, key: OcteractImprovedDaily3Key) -> f64 {
    match key {
        OcteractImprovedDaily3Key::GoldenQuarkMult => 1.0 + 0.005 * n,
        OcteractImprovedDaily3Key::ExtraGoldenQuarks => n,
    }
}

/// `octeractImprovedQuarkHept` effect — `n / 100`.
#[must_use]
pub fn octeract_improved_quark_hept_effect(n: f64) -> f64 {
    n / 100.0
}

/// `octeractImprovedGlobalSpeed` effect — scales with singularity.
#[must_use]
pub fn octeract_improved_global_speed_effect(n: f64, singularity_count: f64) -> f64 {
    1.0 + n * singularity_count / 100.0
}

/// `octeractImprovedAscensionSpeed` effect — scales with singularity.
#[must_use]
pub fn octeract_improved_ascension_speed_effect(n: f64, singularity_count: f64) -> f64 {
    1.0 + n * singularity_count / 2_000.0
}

/// `octeractImprovedAscensionSpeed2` effect — scales with singularity.
#[must_use]
pub fn octeract_improved_ascension_speed_2_effect(n: f64, singularity_count: f64) -> f64 {
    1.0 + n * singularity_count / 2_000.0
}

/// `octeractImprovedFree` key selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcteractImprovedFreeKey {
    /// `unlocked` — `n > 0`.
    Unlocked,
    /// `freeLevelPower` — `0.6n`.
    FreeLevelPower,
}

/// `octeractImprovedFree` tagged result.
#[derive(Debug, Clone, Copy)]
pub enum OcteractImprovedFreeValue {
    /// Unlock flag.
    Unlock(bool),
    /// Scalar value.
    Scalar(f64),
}

/// `octeractImprovedFree` effect.
#[must_use]
pub fn octeract_improved_free_effect(
    n: f64,
    key: OcteractImprovedFreeKey,
) -> OcteractImprovedFreeValue {
    match key {
        OcteractImprovedFreeKey::Unlocked => OcteractImprovedFreeValue::Unlock(n > 0.0),
        OcteractImprovedFreeKey::FreeLevelPower => OcteractImprovedFreeValue::Scalar(0.6 * n),
    }
}

/// `octeractImprovedFree2` effect — `0.05n`.
#[must_use]
pub fn octeract_improved_free_2_effect(n: f64) -> f64 {
    0.05 * n
}

/// `octeractImprovedFree3` effect — `0.05n`.
#[must_use]
pub fn octeract_improved_free_3_effect(n: f64) -> f64 {
    0.05 * n
}

/// `octeractImprovedFree4` effect — `0.001n + 0.01` if `n > 0`,
/// else `0`.
#[must_use]
pub fn octeract_improved_free_4_effect(n: f64) -> f64 {
    0.001 * n + if n > 0.0 { 0.01 } else { 0.0 }
}

/// `octeractSingUpgradeCap` effect — identity.
#[must_use]
pub fn octeract_sing_upgrade_cap_effect(n: f64) -> f64 {
    n
}

/// `octeractOfferings1` effect — `1 + 0.01n`.
#[must_use]
pub fn octeract_offerings_1_effect(n: f64) -> f64 {
    1.0 + 0.01 * n
}

/// `octeractObtainium1` effect — `1 + 0.01n`.
#[must_use]
pub fn octeract_obtainium_1_effect(n: f64) -> f64 {
    1.0 + 0.01 * n
}

#[inline]
fn ascension_count_mult_effect(n: f64) -> f64 {
    (1.0 + n / 100.0) * (1.0 + 2.0 * (n / 10.0).floor() / 100.0)
}

/// `octeractAscensions` effect.
#[must_use]
pub fn octeract_ascensions_effect(n: f64) -> f64 {
    ascension_count_mult_effect(n)
}

/// `octeractAscensions2` effect — same as ascensions.
#[must_use]
pub fn octeract_ascensions_2_effect(n: f64) -> f64 {
    ascension_count_mult_effect(n)
}

/// `octeractAscensionsOcteractGain` effect —
/// `(1 + n/100)^(1 + floor(log10(1 + ascensionCount)))`.
#[must_use]
pub fn octeract_ascensions_octeract_gain_effect(n: f64, ascension_count: f64) -> f64 {
    (1.0 + n / 100.0).powf(1.0 + (1.0 + ascension_count).log10().floor())
}

/// `octeractFastForward` effect — identity (lookahead).
#[must_use]
pub fn octeract_fast_forward_effect(n: f64) -> f64 {
    n
}

/// `octeractAutoPotionSpeed` effect — `1 + 4n/100`.
#[must_use]
pub fn octeract_auto_potion_speed_effect(n: f64) -> f64 {
    1.0 + 4.0 * n / 100.0
}

/// `octeractAutoPotionEfficiency` effect — `1 + 2n/100`.
#[must_use]
pub fn octeract_auto_potion_efficiency_effect(n: f64) -> f64 {
    1.0 + 2.0 * n / 100.0
}

/// `octeractOneMindImprover` effect — `0.55 + n/150`.
#[must_use]
pub fn octeract_one_mind_improver_effect(n: f64) -> f64 {
    0.55 + n / 150.0
}

/// `octeractAmbrosiaLuck` effect — `4n`.
#[must_use]
pub fn octeract_ambrosia_luck_effect(n: f64) -> f64 {
    4.0 * n
}

/// `octeractAmbrosiaLuck2` effect — `2n`.
#[must_use]
pub fn octeract_ambrosia_luck_2_effect(n: f64) -> f64 {
    2.0 * n
}

/// `octeractAmbrosiaLuck3` effect — `3n`.
#[must_use]
pub fn octeract_ambrosia_luck_3_effect(n: f64) -> f64 {
    3.0 * n
}

/// `octeractAmbrosiaLuck4` effect — `5n`.
#[must_use]
pub fn octeract_ambrosia_luck_4_effect(n: f64) -> f64 {
    5.0 * n
}

/// `octeractAmbrosiaGeneration` effect — `1 + n/100`.
#[must_use]
pub fn octeract_ambrosia_generation_effect(n: f64) -> f64 {
    1.0 + n / 100.0
}

/// `octeractAmbrosiaGeneration2` effect — `1 + n/100`.
#[must_use]
pub fn octeract_ambrosia_generation_2_effect(n: f64) -> f64 {
    1.0 + n / 100.0
}

/// `octeractAmbrosiaGeneration3` effect — `1 + n/100`.
#[must_use]
pub fn octeract_ambrosia_generation_3_effect(n: f64) -> f64 {
    1.0 + n / 100.0
}

/// `octeractAmbrosiaGeneration4` effect — `1 + 2n/100`.
#[must_use]
pub fn octeract_ambrosia_generation_4_effect(n: f64) -> f64 {
    1.0 + 2.0 * n / 100.0
}

/// `octeractBonusTokens1` effect — identity.
#[must_use]
pub fn octeract_bonus_tokens_1_effect(n: f64) -> f64 {
    n
}

/// `octeractBonusTokens2` effect — `1 + n/100`.
#[must_use]
pub fn octeract_bonus_tokens_2_effect(n: f64) -> f64 {
    1.0 + n / 100.0
}

/// `octeractBonusTokens3` effect — identity.
#[must_use]
pub fn octeract_bonus_tokens_3_effect(n: f64) -> f64 {
    n
}

/// `octeractBonusTokens4` effect — `2n`.
#[must_use]
pub fn octeract_bonus_tokens_4_effect(n: f64) -> f64 {
    2.0 * n
}

/// `octeractBlueberries` effect — identity (blueberry count).
#[must_use]
pub fn octeract_blueberries_effect(n: f64) -> f64 {
    n
}

/// `octeractInfiniteShopUpgrades` effect — identity.
#[must_use]
pub fn octeract_infinite_shop_upgrades_effect(n: f64) -> f64 {
    n
}

/// `octeractTalismanLevelCap1` effect — identity.
#[must_use]
pub fn octeract_talisman_level_cap_1_effect(n: f64) -> f64 {
    n
}

/// `octeractTalismanLevelCap2` effect — identity.
#[must_use]
pub fn octeract_talisman_level_cap_2_effect(n: f64) -> f64 {
    n
}

/// `octeractTalismanLevelCap3` effect — identity.
#[must_use]
pub fn octeract_talisman_level_cap_3_effect(n: f64) -> f64 {
    n
}

/// `octeractTalismanLevelCap4` effect — identity.
#[must_use]
pub fn octeract_talisman_level_cap_4_effect(n: f64) -> f64 {
    n
}

// ─── Max levels ────────────────────────────────────────────────────────────

/// Per-upgrade `maxLevel`, in `octeractUpgrades` key order (`-1` =
/// unlimited). Identical in both legacy snapshots. Static data the UI tier
/// also owns — kept here (not in the state struct) since it never varies
/// per player.
pub const OCTERACT_MAX_LEVELS: [f64; OCTERACT_UPGRADES_LEN] = [
    1.0,      // 0 octeractStarter
    1e8,      // 1 octeractGain
    -1.0,     // 2 octeractGain2
    20_000.0, // 3 octeractQuarkGain
    5.0,      // 4 octeractQuarkGain2
    2.0,      // 5 octeractCorruption
    50.0,     // 6 octeractGQCostReduce
    100.0,    // 7 octeractExportQuarks
    50.0,     // 8 octeractImprovedDaily
    50.0,     // 9 octeractImprovedDaily2
    -1.0,     // 10 octeractImprovedDaily3
    25.0,     // 11 octeractImprovedQuarkHept
    1_000.0,  // 12 octeractImprovedGlobalSpeed
    100.0,    // 13 octeractImprovedAscensionSpeed
    250.0,    // 14 octeractImprovedAscensionSpeed2
    1.0,      // 15 octeractImprovedFree
    1.0,      // 16 octeractImprovedFree2
    1.0,      // 17 octeractImprovedFree3
    40.0,     // 18 octeractImprovedFree4
    10.0,     // 19 octeractSingUpgradeCap
    -1.0,     // 20 octeractOfferings1
    -1.0,     // 21 octeractObtainium1
    1e6,      // 22 octeractAscensions
    -1.0,     // 23 octeractAscensions2
    -1.0,     // 24 octeractAscensionsOcteractGain
    2.0,      // 25 octeractFastForward
    -1.0,     // 26 octeractAutoPotionSpeed
    100.0,    // 27 octeractAutoPotionEfficiency
    20.0,     // 28 octeractOneMindImprover
    -1.0,     // 29 octeractAmbrosiaLuck
    30.0,     // 30 octeractAmbrosiaLuck2
    30.0,     // 31 octeractAmbrosiaLuck3
    50.0,     // 32 octeractAmbrosiaLuck4
    -1.0,     // 33 octeractAmbrosiaGeneration
    20.0,     // 34 octeractAmbrosiaGeneration2
    35.0,     // 35 octeractAmbrosiaGeneration3
    50.0,     // 36 octeractAmbrosiaGeneration4
    10.0,     // 37 octeractBonusTokens1
    5.0,      // 38 octeractBonusTokens2
    5.0,      // 39 octeractBonusTokens3
    50.0,     // 40 octeractBonusTokens4
    6.0,      // 41 octeractBlueberries
    80.0,     // 42 octeractInfiniteShopUpgrades
    25.0,     // 43 octeractTalismanLevelCap1
    35.0,     // 44 octeractTalismanLevelCap2
    40.0,     // 45 octeractTalismanLevelCap3
    -1.0,     // 46 octeractTalismanLevelCap4
];

/// Count of octeract upgrades whose **purchased** level has reached their
/// `maxLevel` (the `octeractUpgrades` progressive-achievement closure:
/// `maxLevel !== -1 && level >= maxLevel`; free levels don't count).
#[must_use]
pub fn count_maxed_octeract_upgrades(state: &OcteractUpgradesState) -> f64 {
    let mut count = 0.0;
    for (upgrade, max) in state.upgrades.iter().zip(OCTERACT_MAX_LEVELS) {
        if max != -1.0 && upgrade.level >= max {
            count += 1.0;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quark_gain_cost_two_band_continuous() {
        // Just below 1000: (1000^7 - 999^7) ≈ 7e18
        // At 1000: (1001^7 - 1000^7) * 10^1 (=10) ≈ same magnitude
        let just_below = octeract_quark_gain_cost_formula(999.0, 1.0);
        let at_knee = octeract_quark_gain_cost_formula(1_000.0, 1.0);
        // Both should be on the order of 1e19; verify they're within 2 OOMs
        assert!(at_knee > just_below);
        assert!(at_knee.log10() - just_below.log10() < 2.0);
    }

    #[test]
    fn quark_gain_cost_with_10k_kicker() {
        // Past 10k, faster_mult kicks in (1 → 10 at +250 levels)
        let at_10k = octeract_quark_gain_cost_formula(10_000.0, 1.0);
        let at_10250 = octeract_quark_gain_cost_formula(10_250.0, 1.0);
        // 250-level gap → fasterMult goes from 1 to 10, plus 10^(250/1000) = ~1.78
        // Combined ratio ≈ 10 * 1.78 = 17.8
        let ratio = at_10250 / at_10k;
        assert!(ratio > 15.0 && ratio < 22.0);
    }

    #[test]
    fn offerings_cost_quintic_below_25() {
        // level=10, base=1 → (11)^5 = 161051
        assert_eq!(octeract_offerings_1_cost_formula(10.0, 1.0), 161_051.0);
    }

    #[test]
    fn offerings_cost_log_branch_at_25() {
        // level=25, base=1 → 1e15 * 10^(1 - 1) = 1e15
        assert_eq!(octeract_offerings_1_cost_formula(25.0, 1.0), 1e15);
    }

    #[test]
    fn blueberries_cost_table_lookup() {
        assert_eq!(octeract_blueberries_cost_formula(0.0, 0.0), 1.0);
        assert_eq!(octeract_blueberries_cost_formula(5.0, 0.0), 1e111);
        assert_eq!(octeract_blueberries_cost_formula(6.0, 0.0), 0.0);
    }

    #[test]
    fn one_mind_improver_cost_below_10() {
        // level=5 → 1e5^5 * 1 = 1e25
        assert_eq!(octeract_one_mind_improver_cost_formula(5.0, 1.0), 1e25);
    }

    #[test]
    fn one_mind_improver_cost_above_10() {
        // level=11 → 1e5^11 * 1e3^1 = 1e55 * 1e3 = 1e58
        let result = octeract_one_mind_improver_cost_formula(11.0, 1.0);
        assert!((result.log10() - 58.0).abs() < 1e-9);
    }

    #[test]
    fn starter_ant_speed_mult() {
        // n=1 → 1 + 99999 = 100000
        assert_eq!(
            octeract_starter_effect(1.0, OcteractStarterKey::AntSpeedMult),
            100_000.0
        );
    }

    #[test]
    fn ascensions_octeract_gain_uses_log_floor() {
        // n=100, ascensionCount=99 → log10(100) = 2 → floor = 2 → exp = 3
        // (1 + 1)^3 = 8
        let result = octeract_ascensions_octeract_gain_effect(100.0, 99.0);
        assert!((result - 8.0).abs() < 1e-9);
    }

    #[test]
    fn one_mind_improver_at_zero() {
        // 0.55 + 0 = 0.55
        assert_eq!(octeract_one_mind_improver_effect(0.0), 0.55);
    }

    #[test]
    fn improved_free_4_floor_kick_at_first_level() {
        // n=0 → 0; n=1 → 0.001 + 0.01 = 0.011
        assert_eq!(octeract_improved_free_4_effect(0.0), 0.0);
        assert!((octeract_improved_free_4_effect(1.0) - 0.011).abs() < 1e-12);
    }

    #[test]
    fn improved_free_unlocked_above_zero() {
        let result = octeract_improved_free_effect(1.0, OcteractImprovedFreeKey::Unlocked);
        assert!(matches!(result, OcteractImprovedFreeValue::Unlock(true)));
        let result = octeract_improved_free_effect(0.0, OcteractImprovedFreeKey::Unlocked);
        assert!(matches!(result, OcteractImprovedFreeValue::Unlock(false)));
    }

    #[test]
    fn quark_gain_2_effect_uses_floor_dividends() {
        // n=111, gain=222, hep=10 → floor(222/111)=2, floor(1+log10(10))=2
        // → 1 + (1/10000) * 2 * 111 * 2 = 1 + 0.0444 = 1.0444
        let result = octeract_quark_gain_2_effect(111.0, 222.0, 10.0);
        assert!((result - 1.044_4).abs() < 1e-9);
    }

    // ─── Cost dispatch + buy ──────────────────────────────────────────────

    #[test]
    fn octeract_upgrade_cost_dispatches_by_index() {
        // Slot 0 = octeractStarter: base × (level + 1) = 10 × 4 = 40.
        assert_eq!(octeract_upgrade_cost(0, 3.0, 10.0), 40.0);
        // Last slot = octeractTalismanLevelCap4: base × 10^level = 5 × 100 = 500.
        assert_eq!(
            octeract_upgrade_cost(OCTERACT_UPGRADES_LEN - 1, 2.0, 5.0),
            500.0
        );
        // Out of range → 0.
        assert_eq!(octeract_upgrade_cost(OCTERACT_UPGRADES_LEN, 1.0, 1.0), 0.0);
    }

    fn oct_state(starter_level: f64) -> OcteractUpgradesState {
        let mut s = OcteractUpgradesState::default();
        s.upgrades[0].level = starter_level;
        s
    }

    #[test]
    fn buy_octeract_upgrade_levels_up_and_spends() {
        // octeractStarter at level 0, costPerLevel 100 → cost 100 × (0+1) = 100.
        let mut upgrades = oct_state(0.0);
        let mut oct = Decimal::from_finite(250.0);
        let events = buy_octeract_upgrade(
            &mut upgrades,
            &mut oct,
            BuyOcteractUpgradeInput {
                index: 0,
                cost_per_level: 100.0,
                max_level: 10.0,
            },
        );
        assert_eq!(upgrades.upgrades[0].level, 1.0);
        assert!((oct.to_number() - 150.0).abs() < 1e-9);
        assert!(matches!(
            events.as_slice(),
            [CoreEvent::OcteractUpgradePurchased { index: 0, .. }]
        ));
    }

    #[test]
    fn buy_octeract_upgrade_unaffordable_is_noop() {
        let mut upgrades = oct_state(0.0);
        let mut oct = Decimal::from_finite(50.0);
        let events = buy_octeract_upgrade(
            &mut upgrades,
            &mut oct,
            BuyOcteractUpgradeInput {
                index: 0,
                cost_per_level: 100.0,
                max_level: 10.0,
            },
        );
        assert_eq!(upgrades.upgrades[0].level, 0.0);
        assert!((oct.to_number() - 50.0).abs() < 1e-9);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_octeract_upgrade_maxed_is_noop() {
        let mut upgrades = oct_state(10.0);
        let mut oct = Decimal::from_finite(1e30);
        let events = buy_octeract_upgrade(
            &mut upgrades,
            &mut oct,
            BuyOcteractUpgradeInput {
                index: 0,
                cost_per_level: 100.0,
                max_level: 10.0,
            },
        );
        assert_eq!(upgrades.upgrades[0].level, 10.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_octeract_upgrade_unlimited_ignores_cap() {
        // maxLevel <= 0 is the unlimited sentinel: buys past any level.
        // octeractStarter at level 999, costPerLevel 1 → cost 1 × 1000 = 1000.
        let mut upgrades = oct_state(999.0);
        let mut oct = Decimal::from_finite(1e30);
        let events = buy_octeract_upgrade(
            &mut upgrades,
            &mut oct,
            BuyOcteractUpgradeInput {
                index: 0,
                cost_per_level: 1.0,
                max_level: 0.0,
            },
        );
        assert_eq!(upgrades.upgrades[0].level, 1000.0);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_octeract_upgrade_out_of_range_is_noop() {
        let mut upgrades = oct_state(0.0);
        let mut oct = Decimal::from_finite(1e30);
        let events = buy_octeract_upgrade(
            &mut upgrades,
            &mut oct,
            BuyOcteractUpgradeInput {
                index: OCTERACT_UPGRADES_LEN,
                cost_per_level: 100.0,
                max_level: 10.0,
            },
        );
        assert!(events.is_empty());
    }
}
