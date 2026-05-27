//! Shop upgrade cost formula.
//!
//! Verbatim port of
//! `legacy/core_split/packages/logic/src/mechanics/shopCosts.ts`
//! (lifted from the legacy `packages/web_ui/src/Shop.ts`). Pure math
//! given the per-upgrade snapshot — the UI side owns the shop data
//! table and feeds the relevant fields in.
//!
//! Two branches:
//! - Consumables and one-shot upgrades (`max_level == 1`): cost is
//!   just `price`. They have no level-based scaling.
//! - Stacked upgrades: cost is
//!   `price + price_increase * current_level` — linear in the
//!   current level.

/// Inputs to [`shop_cost`].
#[derive(Debug, Clone, Copy)]
pub struct ShopCostInput {
    /// `shopUpgrades[k].type === shopUpgradeTypes.CONSUMABLE` —
    /// consumables are flat-priced.
    pub is_consumable: bool,
    /// `shopUpgrades[k].maxLevel`. Single-purchase upgrades
    /// (`max_level == 1`) are flat-priced.
    pub max_level: f64,
    /// `shopUpgrades[k].price` — base cost.
    pub price: f64,
    /// `shopUpgrades[k].priceIncrease` — per-level cost increment.
    pub price_increase: f64,
    /// `player.shopUpgrades[k]` — currently owned level.
    pub current_level: f64,
}

/// Cost to purchase the next level of a shop upgrade. Flat `price`
/// for consumables and one-shot upgrades; linear scaling for stacked
/// upgrades.
#[must_use]
pub fn shop_cost(input: &ShopCostInput) -> f64 {
    if input.is_consumable || input.max_level == 1.0 {
        return input.price;
    }
    input.price + input.price_increase * input.current_level
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumable_is_flat_priced() {
        let input = ShopCostInput {
            is_consumable: true,
            max_level: 100.0,
            price: 50.0,
            price_increase: 100.0,
            current_level: 5.0,
        };
        // Price increase ignored.
        assert_eq!(shop_cost(&input), 50.0);
    }

    #[test]
    fn one_shot_is_flat_priced() {
        let input = ShopCostInput {
            is_consumable: false,
            max_level: 1.0,
            price: 50.0,
            price_increase: 100.0,
            current_level: 0.0,
        };
        assert_eq!(shop_cost(&input), 50.0);
    }

    #[test]
    fn stacked_uses_linear_scaling() {
        let input = ShopCostInput {
            is_consumable: false,
            max_level: 10.0,
            price: 100.0,
            price_increase: 25.0,
            current_level: 4.0,
        };
        // 100 + 25*4 = 200
        assert_eq!(shop_cost(&input), 200.0);
    }
}
