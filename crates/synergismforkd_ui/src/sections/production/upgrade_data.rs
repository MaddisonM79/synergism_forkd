//! Static metadata for the 125 shop upgrades (the `state.upgrades.upgrades`
//! bitmap, indices `1..=125`): which shop each renders in, the currency it
//! costs (icon + affordability), and its per-row reveal gate.
//!
//! Ported from the legacy `upgradeRequirements` (`Upgrades.ts:250`, the reveal
//! gates) and the `upgradedescriptions` currency map (`Upgrades.ts:420`). The
//! currency split is verified against the logic tier's `buy_generator` /
//! `buy_autobuyers` so the affordability check spends what the buy actually
//! spends. The *cost* is read from `UPGRADE_COSTS` — one source of truth shared
//! with the autobuyers; never duplicated here.

use synergismforkd_bignum::Decimal;
use synergismforkd_logic::mechanics::auto_upgrades::{
    diamond_upgrade_reward, DIAMOND_UPGRADE_18_ACHIEVEMENT, DIAMOND_UPGRADE_19_ACHIEVEMENT,
    DIAMOND_UPGRADE_20_ACHIEVEMENT, UPGRADE_COSTS,
};
use synergismforkd_logic::{GameState, ShopAutobuyKind};

use crate::components::Resource;

/// Highest player-facing shop-upgrade index. The bitmap has 141 slots, but only
/// `1..=125` are buyable shop upgrades (slot 0 is unused).
pub const MAX_UPGRADE: usize = 125;

/// The six on-screen upgrade shops (legacy `upgrades.shopTitles`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shop {
    Coin,
    Diamond,
    Mythos,
    Particle,
    Automation,
    Generator,
}

impl Shop {
    /// Render order, matching the legacy tab layout.
    pub const ALL: [Shop; 6] = [
        Shop::Coin,
        Shop::Diamond,
        Shop::Mythos,
        Shop::Particle,
        Shop::Automation,
        Shop::Generator,
    ];

    /// i18n key for the section header.
    #[must_use]
    pub fn title_key(self) -> &'static str {
        match self {
            Shop::Coin => "upgrades.shopTitles.coin",
            Shop::Diamond => "upgrades.shopTitles.diamond",
            Shop::Mythos => "upgrades.shopTitles.mythos",
            Shop::Particle => "upgrades.shopTitles.particle",
            Shop::Automation => "upgrades.shopTitles.automation",
            Shop::Generator => "upgrades.shopTitles.generator",
        }
    }

    /// The upgrade-tab autobuy toggle this shop drives, if its family is
    /// auto-buyable. The `Automation` upgrades (81-100) only auto-buy at
    /// singularity 25 (no per-family `shoptoggle`), so they have no toggle.
    #[must_use]
    pub fn autobuy_kind(self) -> Option<ShopAutobuyKind> {
        match self {
            Shop::Coin => Some(ShopAutobuyKind::Coin),
            Shop::Diamond => Some(ShopAutobuyKind::Diamond),
            Shop::Mythos => Some(ShopAutobuyKind::Mythos),
            Shop::Particle => Some(ShopAutobuyKind::Reincarnation),
            Shop::Generator => Some(ShopAutobuyKind::Generators),
            Shop::Automation => None,
        }
    }

    /// Whether this shop's autobuyer is unlocked — its unlock upgrade is owned,
    /// matching the gates in `tick::auto_buy::auto_upgrades` (coin 91, diamond
    /// 92, mythos 99, generators 90, reincarnation cube-upgrade 8). Until then
    /// the toggle would have no effect, so it's hidden.
    #[must_use]
    pub fn autobuy_unlocked(self, s: &GameState) -> bool {
        match self {
            Shop::Coin => s.upgrades.upgrades[91] > 0,
            Shop::Diamond => s.upgrades.upgrades[92] > 0,
            Shop::Mythos => s.upgrades.upgrades[99] > 0,
            Shop::Generator => s.upgrades.upgrades[90] > 0,
            Shop::Particle => s.cube_upgrade_levels.cube_upgrades[8] > 0.0,
            Shop::Automation => false,
        }
    }
}

/// Per-upgrade reveal gate (legacy `upgradeRequirements[i]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reveal {
    /// Always visible.
    Always,
    /// `player.unlocks.prestige`.
    Prestige,
    /// `player.unlocks.generation`.
    Generation,
    /// `player.unlocks.transcend`.
    Transcend,
    /// `player.unlocks.reincarnate`.
    Reincarnate,
    /// `player.researches[n] > 0`.
    Research(usize),
    /// `getAchievementReward('diamondUpgrade{18,19,20}')`.
    DiamondUpgradeAch(u8),
    /// `player.cubeUpgrades[19] > 0`.
    CubeUpgrade19,
}

impl Reveal {
    fn satisfied(self, s: &GameState) -> bool {
        let rc = &s.reset_counters;
        match self {
            Reveal::Always => true,
            Reveal::Prestige => rc.prestige_unlocked,
            Reveal::Generation => rc.generation_unlocked,
            Reveal::Transcend => rc.transcend_unlocked,
            Reveal::Reincarnate => rc.reincarnate_unlocked,
            Reveal::Research(n) => s.researches.researches.get(n).is_some_and(|&v| v > 0.0),
            Reveal::DiamondUpgradeAch(which) => {
                let idx = match which {
                    18 => DIAMOND_UPGRADE_18_ACHIEVEMENT,
                    19 => DIAMOND_UPGRADE_19_ACHIEVEMENT,
                    _ => DIAMOND_UPGRADE_20_ACHIEVEMENT,
                };
                diamond_upgrade_reward(&s.achievements.achievements, idx)
            }
            Reveal::CubeUpgrade19 => s
                .cube_upgrade_levels
                .cube_upgrades
                .get(19)
                .is_some_and(|&v| v > 0.0),
        }
    }
}

/// Static display metadata for one shop upgrade.
#[derive(Debug, Clone, Copy)]
pub struct UpgradeMeta {
    /// Bitmap index, `1..=125`.
    pub idx: usize,
    /// Which shop section it renders in.
    pub shop: Shop,
    /// Cost currency (icon + affordability balance).
    pub resource: Resource,
    /// Per-row reveal gate.
    pub reveal: Reveal,
}

impl UpgradeMeta {
    /// Cost of this upgrade = `10 ^ UPGRADE_COSTS[idx]`.
    #[must_use]
    pub fn cost(self) -> Decimal {
        Decimal::from_finite(10.0).pow(Decimal::from_finite(UPGRADE_COSTS[self.idx]))
    }

    /// Whether the card should be shown (its reveal gate is met).
    #[must_use]
    pub fn revealed(self, s: &GameState) -> bool {
        self.reveal.satisfied(s)
    }

    /// Whether the upgrade is already owned.
    #[must_use]
    pub fn owned(self, s: &GameState) -> bool {
        s.upgrades.upgrades[self.idx] != 0
    }

    /// The player's current balance of this upgrade's cost currency.
    #[must_use]
    pub fn balance(self, s: &GameState) -> Decimal {
        match self.resource {
            Resource::Coins => s.upgrades.coins,
            Resource::Diamonds => s.upgrades.prestige_points,
            Resource::Mythos => s.upgrades.transcend_points,
            // Particles — the only remaining cost currency in this table.
            _ => s.upgrades.reincarnation_points,
        }
    }

    /// Buyable now: unowned and affordable (mirrors the mechanic's guard).
    #[must_use]
    pub fn affordable(self, s: &GameState) -> bool {
        !self.owned(s) && self.balance(s) >= self.cost()
    }
}

/// The cost currency for upgrade `idx` (legacy `upgradedescriptions`, verified
/// against `buy_generator` / `buy_autobuyers`).
const fn resource_of(idx: usize) -> Resource {
    match idx {
        1..=20 | 106..=110 | 121..=125 => Resource::Coins,
        21..=40 | 81..=87 | 101..=105 | 111..=115 => Resource::Diamonds,
        41..=60 | 88..=93 | 116..=120 => Resource::Mythos,
        // 61..=80 | 94..=100
        _ => Resource::Particles,
    }
}

/// The shop section for upgrade `idx`.
const fn shop_of(idx: usize) -> Shop {
    match idx {
        1..=20 | 121..=125 => Shop::Coin,
        21..=40 => Shop::Diamond,
        41..=60 => Shop::Mythos,
        61..=80 => Shop::Particle,
        81..=100 => Shop::Automation,
        // 101..=120
        _ => Shop::Generator,
    }
}

/// The reveal gate for upgrade `idx` (legacy `upgradeRequirements`).
const fn reveal_of(idx: usize) -> Reveal {
    match idx {
        1..=5 => Reveal::Always,
        6..=10 => Reveal::Prestige,
        11..=15 => Reveal::Generation,
        16..=20 => Reveal::Transcend,
        21..=30 => Reveal::Prestige,
        31..=35 => Reveal::Transcend,
        36..=37 => Reveal::Reincarnate,
        38 => Reveal::DiamondUpgradeAch(18),
        39 => Reveal::DiamondUpgradeAch(19),
        40 => Reveal::DiamondUpgradeAch(20),
        41..=50 => Reveal::Transcend,
        51..=60 => Reveal::Reincarnate,
        61..=65 => Reveal::Research(47),
        66..=70 => Reveal::Research(48),
        71..=75 => Reveal::Research(49),
        76..=80 => Reveal::Research(50),
        81..=87 => Reveal::Prestige,
        88..=93 => Reveal::Transcend,
        94..=100 => Reveal::Reincarnate,
        101 => Reveal::Prestige,
        102..=105 => Reveal::Generation,
        106..=120 => Reveal::Transcend,
        // 121..=125
        _ => Reveal::CubeUpgrade19,
    }
}

/// Metadata for upgrade `idx` (`1..=125`).
#[must_use]
pub fn meta(idx: usize) -> UpgradeMeta {
    debug_assert!(
        (1..=MAX_UPGRADE).contains(&idx),
        "upgrade index out of range"
    );
    UpgradeMeta {
        idx,
        shop: shop_of(idx),
        resource: resource_of(idx),
        reveal: reveal_of(idx),
    }
}

/// Every upgrade in `shop`, in ascending index order (the legacy row order).
pub fn shop_upgrades(shop: Shop) -> impl Iterator<Item = UpgradeMeta> {
    (1..=MAX_UPGRADE).map(meta).filter(move |m| m.shop == shop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_index_has_a_shop_and_currency() {
        // 125 upgrades, each landing in exactly one shop.
        let total: usize = Shop::ALL.iter().map(|&s| shop_upgrades(s).count()).sum();
        assert_eq!(total, MAX_UPGRADE);
    }

    #[test]
    fn shop_sizes_match_legacy_layout() {
        assert_eq!(shop_upgrades(Shop::Coin).count(), 25); // 1..=20 + 121..=125
        assert_eq!(shop_upgrades(Shop::Diamond).count(), 20);
        assert_eq!(shop_upgrades(Shop::Mythos).count(), 20);
        assert_eq!(shop_upgrades(Shop::Particle).count(), 20);
        assert_eq!(shop_upgrades(Shop::Automation).count(), 20);
        assert_eq!(shop_upgrades(Shop::Generator).count(), 20);
    }

    #[test]
    fn currency_map_spot_checks() {
        assert_eq!(meta(1).resource, Resource::Coins);
        assert_eq!(meta(21).resource, Resource::Diamonds);
        assert_eq!(meta(41).resource, Resource::Mythos);
        assert_eq!(meta(61).resource, Resource::Particles);
        assert_eq!(meta(81).resource, Resource::Diamonds); // automation 1
        assert_eq!(meta(90).resource, Resource::Mythos); // automation 10
        assert_eq!(meta(95).resource, Resource::Particles); // automation 15
        assert_eq!(meta(106).resource, Resource::Coins); // generator 6
        assert_eq!(meta(116).resource, Resource::Mythos); // generator 16
        assert_eq!(meta(121).resource, Resource::Coins); // extended coin
    }

    #[test]
    fn reveal_map_spot_checks() {
        assert_eq!(meta(1).reveal, Reveal::Always);
        assert_eq!(meta(6).reveal, Reveal::Prestige);
        assert_eq!(meta(11).reveal, Reveal::Generation);
        assert_eq!(meta(16).reveal, Reveal::Transcend);
        assert_eq!(meta(38).reveal, Reveal::DiamondUpgradeAch(18));
        assert_eq!(meta(61).reveal, Reveal::Research(47));
        assert_eq!(meta(76).reveal, Reveal::Research(50));
        assert_eq!(meta(101).reveal, Reveal::Prestige);
        assert_eq!(meta(102).reveal, Reveal::Generation);
        assert_eq!(meta(116).reveal, Reveal::Transcend);
        assert_eq!(meta(121).reveal, Reveal::CubeUpgrade19);
    }

    #[test]
    fn fresh_state_reveals_only_base_coin_row() {
        let s = GameState::default();
        let revealed: Vec<usize> = (1..=MAX_UPGRADE)
            .map(meta)
            .filter(|m| m.revealed(&s))
            .map(|m| m.idx)
            .collect();
        assert_eq!(revealed, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn cost_reads_from_logic_table() {
        // 10 ^ UPGRADE_COSTS[1] = 10^6.
        assert_eq!(meta(1).cost(), Decimal::from_finite(1e6));
    }
}
