//! Platonic-upgrade cost table + price-multiplier formula + per-resource
//! affordability check.
//!
//! Verbatim port of
//! `legacy_core_split/packages/logic/src/mechanics/platonicUpgradeCosts.ts`
//! (lifted from the legacy `packages/web_ui/src/Platonic.ts`, lines
//! 8-319). The display function (`createPlatonicDescription`), DOM
//! updates, and the player-mutating buy loop stay in the UI tier;
//! logic owns the static data and the math.
//!
//! Price multiplier shape:
//!
//! ```text
//! priceMultiplier =
//!   (priceMult is undefined ? 1 : priceMult ^ (currentLevel / (maxLevel-1))^1.25)
//!   × singularityDebuff
//! ```
//!
//! Affordability is per-resource:
//! `cost = floor(baseCost * priceMultiplier) ≤ playerHas`. The
//! auto-mode flag exempts obtainium / offerings from the check
//! (auto-buy doesn't actually consume those for platonic upgrades).

/// Per-platonic-upgrade base costs across all 7 resources, plus the
/// `max_level` cap and the optional `price_mult` for level-scaling
/// upgrades.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlatonicUpgradeBaseCost {
    /// Obtainium cost (single-buy upgrades use `1`).
    pub obtainium: f64,
    /// Offering cost.
    pub offerings: f64,
    /// Wow-cube cost.
    pub cubes: f64,
    /// Tesseract cost.
    pub tesseracts: f64,
    /// Hypercube cost.
    pub hypercubes: f64,
    /// Platonic-cube cost.
    pub platonics: f64,
    /// Abyssal (hepteract) cost. `0` for upgrades that don't consume
    /// abyssals.
    pub abyssals: f64,
    /// Maximum level for this upgrade.
    pub max_level: f64,
    /// `None` for upgrades that don't scale with level (single-buy
    /// upgrades like #5, #9-#11). Otherwise the base of the level-cost
    /// exponential.
    pub price_mult: Option<f64>,
}

/// Resource keys in fixed order, matching the legacy iteration. The
/// first six are checked against `current_resources`; `abyssals` is
/// checked against `abyssal_balance` because it lives on
/// `hepteracts.abyss.BAL`, not on the player object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatonicResourceKey {
    /// `currentResources.obtainium`.
    Obtainium,
    /// `currentResources.offerings`.
    Offerings,
    /// `currentResources.cubes`.
    Cubes,
    /// `currentResources.tesseracts`.
    Tesseracts,
    /// `currentResources.hypercubes`.
    Hypercubes,
    /// `currentResources.platonics`.
    Platonics,
    /// `hepteracts.abyss.BAL`.
    Abyssals,
}

/// 20-entry cost table, indexed `1..=20`. The lookup helper does the
/// `-1` to convert to 0-based array index.
const PLATONIC_UPGRADE_BASE_COSTS: [PlatonicUpgradeBaseCost; 20] = [
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e45,
        cubes: 1e13,
        tesseracts: 1e6,
        hypercubes: 1e5,
        platonics: 1e4,
        abyssals: 0.0,
        max_level: 300.0,
        price_mult: Some(2.0),
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 3e45,
        cubes: 1e11,
        tesseracts: 1e8,
        hypercubes: 1e5,
        platonics: 1e4,
        abyssals: 0.0,
        max_level: 300.0,
        price_mult: Some(2.0),
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e46,
        cubes: 1e11,
        tesseracts: 1e6,
        hypercubes: 1e7,
        platonics: 1e4,
        abyssals: 0.0,
        max_level: 300.0,
        price_mult: Some(2.0),
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 3e46,
        cubes: 1e12,
        tesseracts: 1e7,
        hypercubes: 1e6,
        platonics: 1e6,
        abyssals: 0.0,
        max_level: 300.0,
        price_mult: Some(2.0),
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e59,
        cubes: 1e14,
        tesseracts: 1e9,
        hypercubes: 1e8,
        platonics: 1e7,
        abyssals: 0.0,
        max_level: 1.0,
        price_mult: None,
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e61,
        cubes: 1e15,
        tesseracts: 1e9,
        hypercubes: 1e8,
        platonics: 1e7,
        abyssals: 0.0,
        max_level: 10.0,
        price_mult: None,
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 3e62,
        cubes: 2e15,
        tesseracts: 2e9,
        hypercubes: 2e8,
        platonics: 1.5e7,
        abyssals: 0.0,
        max_level: 15.0,
        price_mult: None,
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e64,
        cubes: 4e15,
        tesseracts: 4e9,
        hypercubes: 4e8,
        platonics: 3e7,
        abyssals: 0.0,
        max_level: 5.0,
        price_mult: None,
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e66,
        cubes: 1e16,
        tesseracts: 1e10,
        hypercubes: 1e9,
        platonics: 5e7,
        abyssals: 0.0,
        max_level: 1.0,
        price_mult: None,
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e68,
        cubes: 1e18,
        tesseracts: 1e12,
        hypercubes: 1e11,
        platonics: 1e9,
        abyssals: 0.0,
        max_level: 1.0,
        price_mult: None,
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e70,
        cubes: 2e17,
        tesseracts: 2e11,
        hypercubes: 2e10,
        platonics: 2e8,
        abyssals: 0.0,
        max_level: 1.0,
        price_mult: None,
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e72,
        cubes: 1e18,
        tesseracts: 1e12,
        hypercubes: 1e11,
        platonics: 1e9,
        abyssals: 0.0,
        max_level: 10.0,
        price_mult: None,
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e74,
        cubes: 2e19,
        tesseracts: 4e12,
        hypercubes: 4e11,
        platonics: 4e9,
        abyssals: 0.0,
        max_level: 1.0,
        price_mult: None,
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e77,
        cubes: 4e20,
        tesseracts: 1e13,
        hypercubes: 1e12,
        platonics: 1e10,
        abyssals: 0.0,
        max_level: 1.0,
        price_mult: None,
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e80,
        cubes: 1e23,
        tesseracts: 1e15,
        hypercubes: 1e14,
        platonics: 1e12,
        abyssals: 1.0,
        max_level: 1.0,
        price_mult: None,
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e110,
        cubes: 0.0,
        tesseracts: 0.0,
        hypercubes: 2.5e15,
        platonics: 0.0,
        abyssals: 0.0,
        max_level: 100.0,
        price_mult: Some(10.0),
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e113,
        cubes: 0.0,
        tesseracts: 0.0,
        hypercubes: 1e19,
        platonics: 0.0,
        abyssals: 2.0,
        max_level: 20.0,
        price_mult: Some(10.0),
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e116,
        cubes: 0.0,
        tesseracts: 0.0,
        hypercubes: 1e19,
        platonics: 0.0,
        abyssals: 4.0,
        max_level: 40.0,
        price_mult: Some(500.0),
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e121,
        cubes: 0.0,
        tesseracts: 0.0,
        hypercubes: 1e21,
        platonics: 0.0,
        abyssals: 64.0,
        max_level: 50.0,
        price_mult: Some(200.0),
    },
    PlatonicUpgradeBaseCost {
        obtainium: 1.0,
        offerings: 1e130,
        cubes: 1e45,
        tesseracts: 1e28,
        hypercubes: 1e25,
        platonics: 1e25,
        abyssals: 1_073_741_823.0,
        max_level: 1.0,
        price_mult: None,
    },
];

/// Look up the base cost row for platonic upgrade `index` (1..=20).
#[must_use]
pub fn platonic_upgrade_base_cost(index: u8) -> PlatonicUpgradeBaseCost {
    debug_assert!(
        matches!(index, 1..=20),
        "platonic upgrade index out of range: {index}"
    );
    PLATONIC_UPGRADE_BASE_COSTS[usize::from(index - 1)]
}

/// Inputs to [`platonic_upgrade_price_multiplier`].
#[derive(Debug, Clone, Copy)]
pub struct PlatonicUpgradePriceMultiplierInput {
    /// `base_cost.price_mult` — `None` for unscaled upgrades (no
    /// per-level cost growth).
    pub price_mult: Option<f64>,
    /// `player.platonicUpgrades[index]`.
    pub current_level: f64,
    /// `base_cost.max_level`.
    pub max_level: f64,
    /// `calculateSingularityDebuff('Platonic Costs')`.
    pub singularity_debuff: f64,
}

/// Cost-scaling multiplier applied to every resource cost for one
/// platonic upgrade. The level-scaling exponent
/// `(current_level / (max_level - 1)) ^ 1.25` produces accelerating
/// costs as the upgrade approaches its cap. Then multiplied by the
/// singularity debuff.
///
/// When `price_mult` is `None` (single-buy upgrades like #5,
/// #9-#11), the level-scaling factor is `1` and only the singularity
/// debuff applies.
#[must_use]
pub fn platonic_upgrade_price_multiplier(input: &PlatonicUpgradePriceMultiplierInput) -> f64 {
    let mut price_multiplier = 1.0;
    if let Some(price_mult) = input.price_mult {
        price_multiplier =
            price_mult.powf((input.current_level / (input.max_level - 1.0)).powf(1.25));
    }
    price_multiplier * input.singularity_debuff
}

/// Per-resource balances for the six non-abyssal platonic-upgrade
/// resources. Mirrors the legacy
/// `Record<Exclude<PlatonicResourceKey, 'abyssals'>, number>`.
#[derive(Debug, Clone, Copy)]
pub struct PlatonicResourceBalances {
    /// Current obtainium balance.
    pub obtainium: f64,
    /// Current offerings balance.
    pub offerings: f64,
    /// Current wow-cube balance.
    pub cubes: f64,
    /// Current tesseract balance.
    pub tesseracts: f64,
    /// Current hypercube balance.
    pub hypercubes: f64,
    /// Current platonic-cube balance.
    pub platonics: f64,
}

/// Inputs to [`check_platonic_upgrade_affordability`].
#[derive(Debug, Clone, Copy)]
pub struct CheckPlatonicUpgradeInput {
    /// Upgrade index (1..=20).
    pub index: u8,
    /// `player.platonicUpgrades[index]`.
    pub current_level: f64,
    /// Already-computed price multiplier — caller passes the result
    /// of [`platonic_upgrade_price_multiplier`].
    pub price_multiplier: f64,
    /// `auto = true` ⇒ skip obtainium and offerings checks (auto-buy
    /// doesn't actually consume them).
    pub auto_mode: bool,
    /// Per-resource current balances.
    pub current_resources: PlatonicResourceBalances,
    /// `hepteracts.abyss.BAL` — abyssals balance lives on the
    /// hepteract, not on player directly.
    pub abyssal_balance: f64,
}

/// Per-resource affordability flags. `can_buy` is `true` iff every
/// checked resource passed AND the upgrade isn't already maxed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlatonicUpgradeAffordability {
    /// `currentResources.obtainium ≥ cost` (or skipped under
    /// `auto_mode`).
    pub obtainium: bool,
    /// `currentResources.offerings ≥ cost` (or skipped under
    /// `auto_mode`).
    pub offerings: bool,
    /// `currentResources.cubes ≥ cost`.
    pub cubes: bool,
    /// `currentResources.tesseracts ≥ cost`.
    pub tesseracts: bool,
    /// `currentResources.hypercubes ≥ cost`.
    pub hypercubes: bool,
    /// `currentResources.platonics ≥ cost`.
    pub platonics: bool,
    /// `abyssal_balance ≥ cost` or the upgrade has no abyssal cost.
    pub abyssals: bool,
    /// `true` iff every resource passed AND `current_level <
    /// max_level`.
    pub can_buy: bool,
}

/// Affordability check across all 7 resources + max-level gate.
/// Mirrors the legacy iteration order and the auto-mode
/// obtainium/offerings exemption.
///
/// The abyssals branch has an extra "if base cost is 0, it's a free
/// check" shortcut so upgrades that don't cost abyssals always pass
/// that check regardless of the player's hepteract balance.
#[must_use]
pub fn check_platonic_upgrade_affordability(
    input: &CheckPlatonicUpgradeInput,
) -> PlatonicUpgradeAffordability {
    let base_cost = platonic_upgrade_base_cost(input.index);
    let mut checks = PlatonicUpgradeAffordability::default();
    let mut checksum = 0_u8;

    // Per-resource helper: cost = floor(base * price_mult); compares to
    // current balance. The auto-mode exemption only applies to
    // obtainium / offerings.
    let check =
        |base: f64, current: f64| -> bool { (base * input.price_multiplier).floor() <= current };

    if input.auto_mode || check(base_cost.obtainium, input.current_resources.obtainium) {
        checks.obtainium = true;
        checksum += 1;
    }
    if input.auto_mode || check(base_cost.offerings, input.current_resources.offerings) {
        checks.offerings = true;
        checksum += 1;
    }
    if check(base_cost.cubes, input.current_resources.cubes) {
        checks.cubes = true;
        checksum += 1;
    }
    if check(base_cost.tesseracts, input.current_resources.tesseracts) {
        checks.tesseracts = true;
        checksum += 1;
    }
    if check(base_cost.hypercubes, input.current_resources.hypercubes) {
        checks.hypercubes = true;
        checksum += 1;
    }
    if check(base_cost.platonics, input.current_resources.platonics) {
        checks.platonics = true;
        checksum += 1;
    }

    // Abyssals: either upgrade doesn't cost any, or hepteract balance
    // covers cost.
    if input.abyssal_balance >= (base_cost.abyssals * input.price_multiplier).floor()
        || base_cost.abyssals == 0.0
    {
        checks.abyssals = true;
        checksum += 1;
    }

    if checksum == 7 && input.current_level < base_cost.max_level {
        checks.can_buy = true;
    }
    checks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_cost_index_1_matches_table() {
        let cost = platonic_upgrade_base_cost(1);
        assert_eq!(cost.max_level, 300.0);
        assert_eq!(cost.price_mult, Some(2.0));
        assert_eq!(cost.offerings, 1e45);
    }

    #[test]
    fn base_cost_index_5_has_no_price_mult() {
        let cost = platonic_upgrade_base_cost(5);
        assert_eq!(cost.price_mult, None);
        assert_eq!(cost.max_level, 1.0);
    }

    #[test]
    fn base_cost_index_20_uses_max_u32() {
        let cost = platonic_upgrade_base_cost(20);
        assert_eq!(cost.abyssals, 1_073_741_823.0); // 2^30 - 1
        assert_eq!(cost.max_level, 1.0);
        assert_eq!(cost.price_mult, None);
    }

    #[test]
    fn price_multiplier_no_price_mult_is_singularity_only() {
        let mult = platonic_upgrade_price_multiplier(&PlatonicUpgradePriceMultiplierInput {
            price_mult: None,
            current_level: 50.0,
            max_level: 100.0,
            singularity_debuff: 3.0,
        });
        assert_eq!(mult, 3.0);
    }

    #[test]
    fn price_multiplier_at_zero_level_is_singularity_only() {
        // (0 / (100-1))^1.25 = 0; 2^0 = 1; * 1 = 1
        let mult = platonic_upgrade_price_multiplier(&PlatonicUpgradePriceMultiplierInput {
            price_mult: Some(2.0),
            current_level: 0.0,
            max_level: 100.0,
            singularity_debuff: 1.0,
        });
        assert_eq!(mult, 1.0);
    }

    #[test]
    fn price_multiplier_at_max_minus_one_uses_full_power() {
        // (99 / 99)^1.25 = 1; 2^1 = 2; * 1 = 2
        let mult = platonic_upgrade_price_multiplier(&PlatonicUpgradePriceMultiplierInput {
            price_mult: Some(2.0),
            current_level: 99.0,
            max_level: 100.0,
            singularity_debuff: 1.0,
        });
        assert!((mult - 2.0).abs() < 1e-9);
    }

    fn baseline_resources() -> PlatonicResourceBalances {
        PlatonicResourceBalances {
            obtainium: 1e100,
            offerings: 1e100,
            cubes: 1e100,
            tesseracts: 1e100,
            hypercubes: 1e100,
            platonics: 1e100,
        }
    }

    #[test]
    fn affordability_with_full_balances_can_buy() {
        let result = check_platonic_upgrade_affordability(&CheckPlatonicUpgradeInput {
            index: 1,
            current_level: 0.0,
            price_multiplier: 1.0,
            auto_mode: false,
            current_resources: baseline_resources(),
            abyssal_balance: 0.0,
        });
        assert!(result.can_buy);
        assert!(result.obtainium);
        assert!(result.offerings);
        assert!(result.cubes);
        assert!(result.tesseracts);
        assert!(result.hypercubes);
        assert!(result.platonics);
        assert!(result.abyssals); // upgrade 1 has 0 abyssal cost
    }

    #[test]
    fn affordability_blocked_by_one_resource() {
        let mut resources = baseline_resources();
        resources.platonics = 0.0; // can't afford platonics
        let result = check_platonic_upgrade_affordability(&CheckPlatonicUpgradeInput {
            index: 1,
            current_level: 0.0,
            price_multiplier: 1.0,
            auto_mode: false,
            current_resources: resources,
            abyssal_balance: 0.0,
        });
        assert!(!result.can_buy);
        assert!(!result.platonics);
        // Other resources still pass.
        assert!(result.cubes);
    }

    #[test]
    fn affordability_blocked_by_max_level() {
        let result = check_platonic_upgrade_affordability(&CheckPlatonicUpgradeInput {
            index: 5,
            current_level: 1.0, // == max_level for upgrade 5
            price_multiplier: 1.0,
            auto_mode: false,
            current_resources: baseline_resources(),
            abyssal_balance: 0.0,
        });
        // All resources affordable but cap reached → can_buy false.
        assert!(!result.can_buy);
        assert!(result.cubes); // resource checks still pass
    }

    #[test]
    fn affordability_auto_mode_exempts_obtainium_and_offerings() {
        let mut resources = baseline_resources();
        resources.obtainium = 0.0;
        resources.offerings = 0.0;
        let result = check_platonic_upgrade_affordability(&CheckPlatonicUpgradeInput {
            index: 1,
            current_level: 0.0,
            price_multiplier: 1.0,
            auto_mode: true,
            current_resources: resources,
            abyssal_balance: 0.0,
        });
        // Auto mode exempts obtainium/offerings → still can_buy.
        assert!(result.can_buy);
        assert!(result.obtainium);
        assert!(result.offerings);
    }

    #[test]
    fn affordability_abyssals_required_for_upgrade_15() {
        let result_no_abyssal = check_platonic_upgrade_affordability(&CheckPlatonicUpgradeInput {
            index: 15,
            current_level: 0.0,
            price_multiplier: 1.0,
            auto_mode: false,
            current_resources: baseline_resources(),
            abyssal_balance: 0.0,
        });
        let result_with_abyssal =
            check_platonic_upgrade_affordability(&CheckPlatonicUpgradeInput {
                index: 15,
                current_level: 0.0,
                price_multiplier: 1.0,
                auto_mode: false,
                current_resources: baseline_resources(),
                abyssal_balance: 1.0,
            });
        assert!(!result_no_abyssal.abyssals);
        assert!(!result_no_abyssal.can_buy);
        assert!(result_with_abyssal.abyssals);
        assert!(result_with_abyssal.can_buy);
    }
}
