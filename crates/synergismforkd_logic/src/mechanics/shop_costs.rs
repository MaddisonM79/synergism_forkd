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

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::state::ShopState;

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

// ─── buy_shop ────────────────────────────────────────────────────────────────

/// Inputs to [`buy_shop`].
#[derive(Debug, Clone, Copy)]
pub struct BuyShopInput {
    /// Shop-upgrade index (`0..83`, via the `SHOP_*` constants). Out-of-range
    /// is a no-op.
    pub index: usize,
    /// `shopUpgrades[k].type === CONSUMABLE` — consumables are flat-priced.
    pub is_consumable: bool,
    /// `shopUpgrades[k].maxLevel` (or stock capacity for consumables).
    pub max_level: f64,
    /// `shopUpgrades[k].price`.
    pub price: f64,
    /// `shopUpgrades[k].priceIncrease`.
    pub price_increase: f64,
}

/// Buy one level of shop upgrade `index` with quarks (`worlds`) — the
/// `shopBuyMaxToggle === false` case of `buyShopUpgrades` (`Shop.ts:2131`).
/// The buy is uniform across leveled upgrades and consumables: both increment
/// `shop.upgrades[index]` (a level or a unit of stock) and spend the
/// [`shop_cost`]. The per-upgrade data table (price / priceIncrease / maxLevel
/// / type) is UI-tier, so the caller passes the snapshot; the cost itself is
/// computed here. Emits [`CoreEvent::ShopUpgradePurchased`].
///
/// Faithful-at-current-state deferrals: the `shopBuyMaxToggle` "buy 10 / buy
/// max" summation, and the UI-tier `isUnlocked` gate (the caller dispatches
/// only unlocked upgrades, as with the other `buy_*` helpers).
#[must_use]
pub fn buy_shop(
    shop: &mut ShopState,
    worlds: &mut Decimal,
    input: BuyShopInput,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events = SmallVec::new();
    if input.index >= shop.upgrades.len() {
        return events;
    }
    let before = shop.upgrades[input.index];
    if before >= input.max_level {
        return events;
    }
    let cost = shop_cost(&ShopCostInput {
        is_consumable: input.is_consumable,
        max_level: input.max_level,
        price: input.price,
        price_increase: input.price_increase,
        current_level: before,
    });
    if worlds.to_number() >= cost {
        *worlds -= Decimal::from_finite(cost);
        let after = before + 1.0;
        shop.upgrades[input.index] = after;
        events.push(CoreEvent::ShopUpgradePurchased {
            index: input.index as u32,
            before,
            after,
            spent: cost,
        });
    }
    events
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

    // ─── buy_shop ────────────────────────────────────────────────────────

    fn stacked_input(index: usize) -> BuyShopInput {
        BuyShopInput {
            index,
            is_consumable: false,
            max_level: 10.0,
            price: 100.0,
            price_increase: 25.0,
        }
    }

    #[test]
    fn buy_shop_levels_up_and_spends() {
        let mut shop = ShopState::default();
        let mut worlds = Decimal::from_finite(500.0);
        let events = buy_shop(&mut shop, &mut worlds, stacked_input(8));
        // Level 0: cost = 100 + 25*0 = 100.
        assert_eq!(shop.upgrades[8], 1.0);
        assert_eq!(worlds.to_number(), 400.0);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CoreEvent::ShopUpgradePurchased { index: 8, before, after, spent }
                if before == 0.0 && after == 1.0 && spent == 100.0
        ));
    }

    #[test]
    fn buy_shop_consumable_increments_stock_at_flat_price() {
        let mut shop = ShopState::default();
        let mut worlds = Decimal::from_finite(200.0);
        let events = buy_shop(
            &mut shop,
            &mut worlds,
            BuyShopInput {
                index: 0,
                is_consumable: true,
                max_level: 100.0,
                price: 50.0,
                price_increase: 0.0,
            },
        );
        assert_eq!(shop.upgrades[0], 1.0); // stock += 1
        assert_eq!(worlds.to_number(), 150.0); // flat 50 spent
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_shop_unaffordable_is_noop() {
        let mut shop = ShopState::default();
        let mut worlds = Decimal::from_finite(50.0); // 50 < cost 100
        let events = buy_shop(&mut shop, &mut worlds, stacked_input(8));
        assert_eq!(shop.upgrades[8], 0.0);
        assert_eq!(worlds.to_number(), 50.0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_shop_maxed_is_noop() {
        let mut shop = ShopState::default();
        shop.upgrades[8] = 10.0; // == max_level
        let mut worlds = Decimal::from_finite(1e9);
        let events = buy_shop(&mut shop, &mut worlds, stacked_input(8));
        assert_eq!(shop.upgrades[8], 10.0);
        assert_eq!(worlds.to_number(), 1e9);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_shop_out_of_range_is_noop() {
        let mut shop = ShopState::default();
        let mut worlds = Decimal::from_finite(1e9);
        assert!(buy_shop(&mut shop, &mut worlds, stacked_input(83)).is_empty());
    }
}
