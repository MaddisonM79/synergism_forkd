//! Ascension constant upgrades — cost table + purchase.
//!
//! Verbatim port of `getConstUpgradeMetadata` / `buyConstantUpgrades`
//! (`legacy/original/src/Upgrades.ts:712` / `:761`). The ten constant
//! upgrades (`player.constantUpgrades[1..=10]`) are bought with
//! `ascendShards`; upgrades 9 and 10 are one-time (capped at level 1).
//!
//! When `researches[175] > 0` the buy is **free** — the level is added but
//! no shards are deducted. That is exactly the autobuyer path
//! (`updateAll` only calls `buyConstantUpgrades(i, true)` under
//! `researches[175] > 0`), but the free behavior is a property of the buy
//! itself, not the caller.

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::CoreEvent;
use crate::state::campaigns::CONSTANT_UPGRADES_LEN;

/// `G.constUpgradeCosts` (`Variables.ts:264`) — base cost per constant
/// upgrade, indexed `1..=10`. Slot 0 is unused (the legacy `null`).
pub const CONST_UPGRADE_COSTS: [f64; CONSTANT_UPGRADES_LEN] = [
    0.0, 1.0, 13.0, 17.0, 237.0, 316.0, 4216.0, 5623.0, 74989.0, 1e10, 1e24,
];

/// Input to [`buy_constant_upgrade`] carried by the buy dispatcher.
#[derive(Debug, Clone, Copy)]
pub struct BuyConstantUpgradeInput {
    /// Constant-upgrade index, `1..=10`.
    pub index: usize,
}

/// `getConstUpgradeMetadata(i)` (`Upgrades.ts:712`) — returns
/// `(levels_to_buy, cost)` for constant upgrade `i` given its current owned
/// level and the `ascendShards` balance. Upgrades `9` and `10` are one-time
/// (`toBuy` clamps to `1` and to `0` once owned).
///
/// `i` must be `1..=10`; slot 0's base cost is `0`, which the caller never
/// reaches (mirrors the legacy `null`).
#[must_use]
pub fn get_const_upgrade_metadata(
    i: usize,
    constant_upgrades_i: f64,
    ascend_shards: Decimal,
) -> (f64, Decimal) {
    let base = CONST_UPGRADE_COSTS[i];
    let log_shards = ascend_shards
        .max(Decimal::from_finite(0.01))
        .log10()
        .to_number();
    let raw = (1.0 + log_shards - base.log10()).floor().max(0.0);

    let to_buy = if i >= 9 {
        if constant_upgrades_i >= 1.0 {
            0.0
        } else {
            raw.min(1.0)
        }
    } else {
        raw
    };

    let base_dec = Decimal::from_finite(base);
    let cost = if to_buy > constant_upgrades_i {
        Decimal::from_finite(10.0).pow(Decimal::from_finite(to_buy - 1.0)) * base_dec
    } else if i >= 9 && constant_upgrades_i >= 1.0 {
        Decimal::zero()
    } else {
        Decimal::from_finite(10.0).pow(Decimal::from_finite(constant_upgrades_i)) * base_dec
    };

    ((to_buy - constant_upgrades_i).max(1.0), cost)
}

/// `buyConstantUpgrades(i, fast = true)` (`Upgrades.ts:761`) — buy constant
/// upgrade `i` if affordable. Upgrades `9` and `10` are skipped once owned.
/// Deducts `ascend_shards` **only when `researches_175 == 0`**; the autobuyer
/// path (`researches[175] > 0`) adds the level for free. Emits
/// [`CoreEvent::ConstantUpgradePurchased`] on a successful buy.
#[must_use]
pub fn buy_constant_upgrade(
    constant_upgrades: &mut [f64],
    ascend_shards: &mut Decimal,
    i: usize,
    researches_175: f64,
) -> SmallVec<[CoreEvent; 4]> {
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();
    let before = constant_upgrades[i];
    let (level, cost) = get_const_upgrade_metadata(i, before, *ascend_shards);

    let buyable = i <= 8 || (i >= 9 && before < 1.0);
    if buyable && *ascend_shards >= cost {
        constant_upgrades[i] += level;
        let spent = if researches_175 == 0.0 {
            *ascend_shards -= cost;
            cost
        } else {
            Decimal::zero()
        };
        events.push(CoreEvent::ConstantUpgradePurchased {
            index: i as u8,
            before,
            after: constant_upgrades[i],
            spent,
        });
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_table_matches_legacy() {
        // G.constUpgradeCosts = [null, 1, 13, 17, 237, 316, 4216, 5623, 74989, 1e10, 1e24]
        assert_eq!(CONST_UPGRADE_COSTS.len(), CONSTANT_UPGRADES_LEN);
        assert_eq!(CONST_UPGRADE_COSTS[1], 1.0);
        assert_eq!(CONST_UPGRADE_COSTS[8], 74989.0);
        assert_eq!(CONST_UPGRADE_COSTS[9], 1e10);
        assert_eq!(CONST_UPGRADE_COSTS[10], 1e24);
    }

    #[test]
    fn metadata_repeatable_upgrade() {
        // i=1 (base 1), 100 shards: raw = floor(1 + log10(100) - log10(1)) = 3.
        let (level, cost) = get_const_upgrade_metadata(1, 0.0, Decimal::from_finite(100.0));
        assert_eq!(level, 3.0);
        assert_eq!(cost.to_number(), 100.0); // 10^(3-1) * 1
    }

    #[test]
    fn metadata_one_time_upgrade_clamps_to_one() {
        // i=9 (base 1e10), 1e11 shards: raw = 2 but i>=9 clamps toBuy to 1.
        let (level, cost) = get_const_upgrade_metadata(9, 0.0, Decimal::from_finite(1e11));
        assert_eq!(level, 1.0);
        assert_eq!(cost.to_number(), 1e10); // 10^0 * 1e10
    }

    #[test]
    fn buy_deducts_when_not_free() {
        let mut levels = [0.0; CONSTANT_UPGRADES_LEN];
        let mut shards = Decimal::from_finite(100.0);
        let events = buy_constant_upgrade(&mut levels, &mut shards, 1, 0.0);
        assert_eq!(levels[1], 3.0);
        assert_eq!(shards.to_number(), 0.0); // 100 - 100
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn buy_is_free_with_research_175() {
        let mut levels = [0.0; CONSTANT_UPGRADES_LEN];
        let mut shards = Decimal::from_finite(100.0);
        let events = buy_constant_upgrade(&mut levels, &mut shards, 1, 1.0);
        assert_eq!(levels[1], 3.0); // level still added
        assert_eq!(shards.to_number(), 100.0); // but no deduction
        assert_eq!(events.len(), 1);
        match events[0] {
            CoreEvent::ConstantUpgradePurchased { spent, .. } => assert_eq!(spent.to_number(), 0.0),
            ref other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn one_time_upgrade_not_rebought() {
        let mut levels = [0.0; CONSTANT_UPGRADES_LEN];
        let mut shards = Decimal::from_finite(1e11);
        let first = buy_constant_upgrade(&mut levels, &mut shards, 9, 0.0);
        assert_eq!(levels[9], 1.0);
        assert_eq!(first.len(), 1);
        // Already owned -> the i>=9 guard skips, no further purchase.
        let second = buy_constant_upgrade(&mut levels, &mut shards, 9, 0.0);
        assert_eq!(levels[9], 1.0);
        assert!(second.is_empty());
    }
}
