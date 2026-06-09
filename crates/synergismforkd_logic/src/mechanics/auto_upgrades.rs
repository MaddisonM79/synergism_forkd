//! `autoUpgrades` support — the upgrade cost table, the generator /
//! autobuyer-unlock buy primitives, the `clickUpgrades` dispatcher, and the
//! diamond-upgrade achievement-reward readers.
//!
//! These back the upgrade-tab autobuyer (`autoUpgrades`,
//! `legacy/original/src/Automation.ts:50`). The classic per-tier upgrade
//! loops reuse the already-ported [`buy_upgrades`]; the three things that
//! file needs beyond it live here:
//! - [`buy_generator`] (`Automation.ts:8`) — generator upgrades 101..=120,
//! - [`buy_autobuyers`] (`Automation.ts:33`) — autobuyer-unlock upgrades
//!   81..=100,
//! - [`click_upgrades`] (`Upgrades.ts:458`) — the index dispatcher used by
//!   the singularity-25 branch.
//!
//! `G.upgradeCosts` is exposed as [`UPGRADE_COSTS`]. The legacy
//! `unlocks.generation` set and the `generationAch1..4` ungrouped-achievement
//! awards inside `buyGenerator` are **not** ported (the `generation` unlock is
//! UI-tier and inert; the generation achievements are not wired in the
//! achievement-awarding subsystem) — the buy itself (currency + bitmap) is
//! faithful.

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::{CoreEvent, UpgradeTier};
use crate::mechanics::upgrades::{buy_upgrades, BuyUpgradeInput};
use crate::state::UpgradesState;

/// `G.upgradeCosts` (`Variables.ts:25`) — `log10` of each upgrade's cost,
/// indexed `0..=125` (slot 0 unused). Actual cost is `10 ^ UPGRADE_COSTS[i]`.
pub const UPGRADE_COSTS: [f64; 126] = [
    // 0..=20 — coin upgrades (slot 0 unused)
    0.0, 6.0, 7.0, 8.0, 10.0, 12.0, 20.0, 35.0, 50.0, 75.0, 100.0, 55.0, 75.0, 125.0, 150.0, 200.0,
    250.0, 500.0, 750.0, 1000.0, 1500.0, // 21..=40 — prestige (diamond) upgrades
    5.0, 15.0, 25.0, 40.0, 60.0, 45.0, 75.0, 100.0, 125.0, 150.0, 150.0, 400.0, 800.0, 1600.0,
    3200.0, 10000.0, 20000.0, 50000.0, 100000.0,
    200000.0, // 41..=60 — transcend (mythos) upgrades
    1.0, 2.0, 3.0, 5.0, 6.0, 7.0, 42.0, 65.0, 87.0, 150.0, 300.0, 500.0, 1000.0, 1500.0, 2000.0,
    3000.0, 6000.0, 12000.0, 25000.0,
    75000.0, // 61..=80 — reincarnation (particle) upgrades
    0.0, 1.0, 2.0, 2.0, 3.0, 5.0, 6.0, 10.0, 15.0, 22.0, 30.0, 37.0, 45.0, 52.0, 60.0, 1900.0,
    2500.0, 3000.0, 10000.0, 21397.0, // 81..=100 — autobuyer-unlock upgrades
    3.0, 6.0, 9.0, 12.0, 15.0, 60.0, 90.0, 6.0, 8.0, 8.0, 10.0, 13.0, 60.0, 1.0, 2.0, 4.0, 8.0,
    16.0, 25.0, 40.0, // 101..=120 — generator upgrades
    12.0, 16.0, 20.0, 30.0, 50.0, 500.0, 1250.0, 5000.0, 25000.0, 125000.0, 1500.0, 7500.0,
    30000.0, 150000.0, 1000000.0, 250.0, 1000.0, 5000.0, 25000.0, 125000.0,
    // 121..=125 — extended coin upgrades
    1e3, 1e6, 1e9, 1e12, 1e15,
];

/// Achievement index whose ownership is `getAchievementReward('diamondUpgrade18')`
/// — challenge-7 first completion (`Achievements.ts`, 0-indexed).
pub const DIAMOND_UPGRADE_18_ACHIEVEMENT: usize = 120;
/// `getAchievementReward('diamondUpgrade19')` — challenge-8 first completion.
pub const DIAMOND_UPGRADE_19_ACHIEVEMENT: usize = 127;
/// `getAchievementReward('diamondUpgrade20')` — challenge-9 first completion.
pub const DIAMOND_UPGRADE_20_ACHIEVEMENT: usize = 134;

/// `getAchievementReward('diamondUpgrade18'..'20')` =
/// `Boolean(player.achievements[idx])` — whether the gating achievement is
/// owned. Gates auto-buying diamond (prestige) upgrades 38/39/40.
#[must_use]
pub fn diamond_upgrade_reward(achievements: &[u8], achievement_index: usize) -> bool {
    achievements
        .get(achievement_index)
        .is_some_and(|&owned| owned != 0)
}

/// `buyGenerator(i, true)` (`Automation.ts:8`) — buy generator upgrade
/// `q = 100 + i` (`i` in `1..=20`) if affordable. The paying currency is
/// chosen by index: coins for `106..=110`, prestige points for `101..=105`
/// and `111..=115`, transcend points for `116..=120`. The `unlocks.generation`
/// flag and `generationAch1..4` awards are not ported.
#[must_use]
pub fn buy_generator(upgrades: &mut UpgradesState, i: usize) -> SmallVec<[CoreEvent; 4]> {
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();
    let q = 100 + i;
    let cost = Decimal::from_finite(10.0).pow(Decimal::from_finite(UPGRADE_COSTS[q]));
    let (current, tier) = if (106..=110).contains(&q) {
        (upgrades.coins, UpgradeTier::Coin)
    } else if q <= 115 {
        (upgrades.prestige_points, UpgradeTier::Prestige)
    } else {
        (upgrades.transcend_points, UpgradeTier::Transcend)
    };

    if upgrades.upgrades[q] == 0 && current >= cost {
        deduct(upgrades, tier, cost);
        upgrades.upgrades[q] = 1;
        events.push(CoreEvent::UpgradePurchased {
            tier,
            pos: q as u32,
            spent: cost,
        });
    }
    events
}

/// `buyAutobuyers(i, true)` (`Automation.ts:33`) — buy autobuyer-unlock
/// upgrade `q = i + 80` (`i` in `1..=20`) if affordable. Paying currency by
/// index: prestige for `81..=87`, transcend for `88..=93`, reincarnation for
/// `94..=100`.
#[must_use]
pub fn buy_autobuyers(upgrades: &mut UpgradesState, i: usize) -> SmallVec<[CoreEvent; 4]> {
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();
    let q = i + 80;
    let cost = Decimal::from_finite(10.0).pow(Decimal::from_finite(UPGRADE_COSTS[q]));
    let (current, tier) = if q <= 87 {
        (upgrades.prestige_points, UpgradeTier::Prestige)
    } else if q <= 93 {
        (upgrades.transcend_points, UpgradeTier::Transcend)
    } else {
        (upgrades.reincarnation_points, UpgradeTier::Reincarnation)
    };

    if upgrades.upgrades[q] == 0 && current >= cost {
        deduct(upgrades, tier, cost);
        upgrades.upgrades[q] = 1;
        events.push(CoreEvent::UpgradePurchased {
            tier,
            pos: q as u32,
            spent: cost,
        });
    }
    events
}

/// Per-run unlock gates read by [`click_upgrades`] (`player.unlocks.*`).
#[derive(Debug, Clone, Copy, Default)]
pub struct ClickUpgradesUnlocks {
    /// `player.unlocks.prestige`.
    pub prestige: bool,
    /// `player.unlocks.transcend`.
    pub transcend: bool,
    /// `player.unlocks.reincarnate`.
    pub reincarnate: bool,
}

/// `clickUpgrades(i, true)` (`Upgrades.ts:458`) — the index dispatcher: buys
/// upgrade `i` via [`buy_upgrades`] (`1..=80`, `121..=125`),
/// [`buy_autobuyers`] (`81..=100`) or [`buy_generator`] (`101..=120`), gated
/// on the upgrade being unowned and the relevant tier unlock. The DOM
/// `display === 'none'` guard is UI-tier and dropped.
#[must_use]
pub fn click_upgrades(
    upgrades: &mut UpgradesState,
    unlocks: ClickUpgradesUnlocks,
    i: usize,
) -> SmallVec<[CoreEvent; 4]> {
    let locked = upgrades.upgrades[i] != 0
        || ((21..=40).contains(&i) && !unlocks.prestige)
        || ((41..=60).contains(&i) && !unlocks.transcend)
        || ((61..=80).contains(&i) && !unlocks.reincarnate)
        || ((81..=120).contains(&i) && !unlocks.prestige);
    if locked {
        return SmallVec::new();
    }

    let mk = |tier: UpgradeTier| BuyUpgradeInput {
        tier,
        pos: i as u32,
        cost_exponent: UPGRADE_COSTS[i],
        requirement_exists: true,
    };
    match i {
        1..=20 | 121..=125 => buy_upgrades(upgrades, mk(UpgradeTier::Coin)),
        21..=40 => buy_upgrades(upgrades, mk(UpgradeTier::Prestige)),
        41..=60 => buy_upgrades(upgrades, mk(UpgradeTier::Transcend)),
        61..=80 => buy_upgrades(upgrades, mk(UpgradeTier::Reincarnation)),
        81..=100 => buy_autobuyers(upgrades, i - 80),
        101..=120 => buy_generator(upgrades, i - 100),
        _ => SmallVec::new(),
    }
}

/// Deduct `cost` from the upgrade tier's currency.
fn deduct(upgrades: &mut UpgradesState, tier: UpgradeTier, cost: Decimal) {
    match tier {
        UpgradeTier::Coin => upgrades.coins -= cost,
        UpgradeTier::Prestige => upgrades.prestige_points -= cost,
        UpgradeTier::Transcend => upgrades.transcend_points -= cost,
        UpgradeTier::Reincarnation => upgrades.reincarnation_points -= cost,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_costs_table_shape() {
        assert_eq!(UPGRADE_COSTS.len(), 126);
        assert_eq!(UPGRADE_COSTS[1], 6.0); // first coin upgrade
        assert_eq!(UPGRADE_COSTS[81], 3.0); // first autobuyer-unlock upgrade
        assert_eq!(UPGRADE_COSTS[101], 12.0); // first generator upgrade
        assert_eq!(UPGRADE_COSTS[125], 1e15); // last extended coin upgrade
    }

    #[test]
    fn diamond_upgrade_reward_reads_bitmap() {
        let mut ach = [0u8; 509];
        assert!(!diamond_upgrade_reward(
            &ach,
            DIAMOND_UPGRADE_18_ACHIEVEMENT
        ));
        ach[DIAMOND_UPGRADE_18_ACHIEVEMENT] = 1;
        assert!(diamond_upgrade_reward(&ach, DIAMOND_UPGRADE_18_ACHIEVEMENT));
        assert!(!diamond_upgrade_reward(
            &ach,
            DIAMOND_UPGRADE_19_ACHIEVEMENT
        ));
    }

    #[test]
    fn buy_generator_tier1_costs_prestige() {
        // q=101 -> cost 10^12, prestige tier.
        let mut up = UpgradesState {
            prestige_points: Decimal::from_finite(1e12),
            ..Default::default()
        };
        let events = buy_generator(&mut up, 1);
        assert_eq!(up.upgrades[101], 1);
        assert_eq!(up.prestige_points.to_number(), 0.0);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_generator_blocked_when_unaffordable() {
        let mut up = UpgradesState {
            prestige_points: Decimal::from_finite(1e6), // < 10^12
            ..Default::default()
        };
        let events = buy_generator(&mut up, 1);
        assert_eq!(up.upgrades[101], 0);
        assert!(events.is_empty());
    }

    #[test]
    fn buy_autobuyers_tier1_costs_prestige() {
        // q=81 -> cost 10^3, prestige tier.
        let mut up = UpgradesState {
            prestige_points: Decimal::from_finite(1e3),
            ..Default::default()
        };
        let events = buy_autobuyers(&mut up, 1);
        assert_eq!(up.upgrades[81], 1);
        assert_eq!(up.prestige_points.to_number(), 0.0);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn click_upgrades_routes_autobuyer_when_prestige_unlocked() {
        let mut up = UpgradesState {
            prestige_points: Decimal::from_finite(1e3),
            ..Default::default()
        };
        // i=81 needs unlocks.prestige; without it the buy is skipped.
        let blocked = click_upgrades(&mut up, ClickUpgradesUnlocks::default(), 81);
        assert!(blocked.is_empty());
        assert_eq!(up.upgrades[81], 0);
        // With the unlock it routes to buy_autobuyers.
        let unlocks = ClickUpgradesUnlocks {
            prestige: true,
            ..Default::default()
        };
        let bought = click_upgrades(&mut up, unlocks, 81);
        assert_eq!(up.upgrades[81], 1);
        assert_eq!(bought.len(), 1);
    }

    #[test]
    fn click_upgrades_routes_generator() {
        let mut up = UpgradesState {
            prestige_points: Decimal::from_finite(1e12),
            ..Default::default()
        };
        // i=101 generator needs unlocks.prestige (the 81..=120 gate).
        let unlocks = ClickUpgradesUnlocks {
            prestige: true,
            ..Default::default()
        };
        let _ = click_upgrades(&mut up, unlocks, 101);
        assert_eq!(up.upgrades[101], 1);
    }
}
