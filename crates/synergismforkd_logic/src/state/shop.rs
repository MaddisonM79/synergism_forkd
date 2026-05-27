//! Shop state slice — the ~83 named shop upgrades plus toggles.
//!
//! Mirrors `player.shopUpgrades`, `player.shopPotionsConsumed`, and
//! `player.shopBuyMaxToggle`. Backs [`crate::mechanics::shop_costs`]
//! and [`crate::mechanics::shop_upgrades`].

/// Shop buy-max toggle. Mirrors `player.shopBuyMaxToggle` in the
/// legacy schema.
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShopBuyMaxMode {
    /// Buy exactly one per click.
    #[default]
    One,
    /// Buy the max affordable up to the upgrade cap.
    Max,
}

/// Fixed cardinality of the shop-upgrade array. Tier B item 12 /
/// Anvil F4.
pub const SHOP_UPGRADES_LEN: usize = 83;

/// Slice of `GameState` for the shop feature.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ShopState {
    /// Per-upgrade purchased level. UI maintains the name ↔ index
    /// mapping.
    #[serde(with = "BigArray")]
    pub upgrades: [f64; SHOP_UPGRADES_LEN],
    /// `player.shopPotionsConsumed` — lifetime potion-use count.
    pub shop_potions_consumed: f64,
    /// `player.shopBuyMaxToggle`.
    pub shop_buy_max_toggle: ShopBuyMaxMode,
}

impl Default for ShopState {
    fn default() -> Self {
        Self {
            upgrades: [0.0; SHOP_UPGRADES_LEN],
            shop_potions_consumed: 0.0,
            shop_buy_max_toggle: ShopBuyMaxMode::One,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_83_upgrade_slots() {
        let s = ShopState::default();
        assert_eq!(s.upgrades.len(), SHOP_UPGRADES_LEN);
        assert!(matches!(s.shop_buy_max_toggle, ShopBuyMaxMode::One));
    }
}
