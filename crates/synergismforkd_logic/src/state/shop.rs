//! Shop state slice — the ~83 named shop upgrades plus toggles.
//!
//! Mirrors `player.shopUpgrades`, `player.shopPotionsConsumed`, and
//! `player.shopBuyMaxToggle`. Backs [`crate::mechanics::shop_costs`]
//! and [`crate::mechanics::shop_upgrades`].

/// Shop buy-max toggle. Mirrors `player.shopBuyMaxToggle` in the
/// legacy schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShopBuyMaxMode {
    /// Buy exactly one per click.
    #[default]
    One,
    /// Buy the max affordable up to the upgrade cap.
    Max,
}

/// Slice of `GameState` for the shop feature.
#[derive(Debug, Clone, PartialEq)]
pub struct ShopState {
    /// Per-upgrade purchased level. UI maintains the name ↔ index
    /// mapping. Legacy has 83 named upgrades.
    pub upgrades: Vec<f64>,
    /// `player.shopPotionsConsumed` — lifetime potion-use count.
    pub shop_potions_consumed: f64,
    /// `player.shopBuyMaxToggle`.
    pub shop_buy_max_toggle: ShopBuyMaxMode,
}

impl ShopState {
    /// Build with `n_upgrades` slots.
    #[must_use]
    pub fn new(n_upgrades: usize) -> Self {
        Self {
            upgrades: vec![0.0; n_upgrades],
            shop_potions_consumed: 0.0,
            shop_buy_max_toggle: ShopBuyMaxMode::One,
        }
    }
}

impl Default for ShopState {
    fn default() -> Self {
        Self::new(83)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_83_upgrade_slots() {
        let s = ShopState::default();
        assert_eq!(s.upgrades.len(), 83);
        assert!(matches!(s.shop_buy_max_toggle, ShopBuyMaxMode::One));
    }
}
