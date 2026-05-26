//! Tier-keyed upgrade purchases.
//!
//! Verbatim port of `buyUpgrades` from
//! `legacy_core_split/packages/logic/src/mechanics/upgrades.ts`. Each
//! resource tier (coin / prestige / transcend / reincarnation) dispatches
//! to its own currency and, on a successful buy, flips a different set
//! of `*_no_*_upgrades` achievement-gate flags. The flag-flip happens
//! unconditionally per call (not gated on affordability) — matches the
//! original behavior.

use smallvec::SmallVec;
use synergismforkd_bignum::Decimal;

use crate::events::{CoreEvent, UpgradeTier};
use crate::state::UpgradesState;

/// Inputs to [`buy_upgrades`]. Mirrors `BuyUpgradeInput`.
#[derive(Debug, Clone, Copy)]
pub struct BuyUpgradeInput {
    /// Which resource tier pays for the upgrade.
    pub tier: UpgradeTier,
    /// Upgrade index in the bitmap (0..N). The legacy convention is
    /// 1-based but the bitmap is direct-indexed, so callers should
    /// position-align with whatever sizing they pre-allocated.
    pub pos: u32,
    /// `log10` of the cost in the tier's currency — actual cost is
    /// `10 ^ cost_exponent`.
    pub cost_exponent: f64,
    /// Mirror of the original `!upgradeRequirements[pos]` guard. The TS
    /// code checks whether the requirement *function* exists (an
    /// out-of-bounds guard) rather than calling it. All current entries
    /// are `() => true`, so this is effectively a bounds check; callers
    /// should pass `upgrade_requirements[pos] is not undefined`.
    pub requirement_exists: bool,
}

/// Buy a single upgrade if affordable + not already owned. Flag-flip
/// matrix runs unconditionally on every call (matches the legacy
/// behavior).
///
/// Flag-flip matrix (mirrors the TS, post-port):
/// - **coin** → flips 7 flags (1 prestige + 2 transcend + 4 reincarnate)
/// - **prestige** → flips 4 flags (2 transcend + 2 reincarnate)
/// - **transcend** → flips 2 flags (2 reincarnate)
/// - **reincarnation** → no flips
#[must_use]
pub fn buy_upgrades(state: &mut UpgradesState, input: BuyUpgradeInput) -> SmallVec<[CoreEvent; 4]> {
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();

    // Out-of-bounds guard from the original buy_upgrades. Returns the
    // state untouched (no flag flips, no purchase) when the requirement
    // entry doesn't exist.
    if !input.requirement_exists {
        return events;
    }

    let cost = Decimal::from_finite(10.0).pow(Decimal::from_finite(input.cost_exponent));

    // Helper: read the currency for the requested tier.
    let current = match input.tier {
        UpgradeTier::Coin => state.coins,
        UpgradeTier::Prestige => state.prestige_points,
        UpgradeTier::Transcend => state.transcend_points,
        UpgradeTier::Reincarnation => state.reincarnation_points,
    };

    let pos_index = input.pos as usize;
    let already_owned = state
        .upgrades
        .get(pos_index)
        .is_none_or(|&owned| owned != 0);

    // Purchase attempt. Mirrors the legacy guard exactly: affordable AND
    // not already owned. On success, deduct cost, set bitmap entry, emit
    // event.
    if current >= cost && !already_owned {
        match input.tier {
            UpgradeTier::Coin => state.coins -= cost,
            UpgradeTier::Prestige => state.prestige_points -= cost,
            UpgradeTier::Transcend => state.transcend_points -= cost,
            UpgradeTier::Reincarnation => state.reincarnation_points -= cost,
        }
        state.upgrades[pos_index] = 1;
        events.push(CoreEvent::UpgradePurchased {
            tier: input.tier,
            pos: input.pos,
            spent: cost,
        });
    }

    // Flag-flip matrix — independent of buy success.
    match input.tier {
        UpgradeTier::Transcend => {
            state.reincarnate_no_coin_prestige_or_transcend_upgrades = false;
            state.reincarnate_no_coin_prestige_transcend_or_generator_upgrades = false;
        }
        UpgradeTier::Prestige => {
            state.transcend_no_coin_or_prestige_upgrades = false;
            state.reincarnate_no_coin_or_prestige_upgrades = false;
            state.reincarnate_no_coin_prestige_or_transcend_upgrades = false;
            state.reincarnate_no_coin_prestige_transcend_or_generator_upgrades = false;
        }
        UpgradeTier::Coin => {
            state.prestige_no_coin_upgrades = false;
            state.transcend_no_coin_upgrades = false;
            state.transcend_no_coin_or_prestige_upgrades = false;
            state.reincarnate_no_coin_upgrades = false;
            state.reincarnate_no_coin_or_prestige_upgrades = false;
            state.reincarnate_no_coin_prestige_or_transcend_upgrades = false;
            state.reincarnate_no_coin_prestige_transcend_or_generator_upgrades = false;
        }
        UpgradeTier::Reincarnation => {}
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::UPGRADES_DEFAULT_LEN;

    fn baseline_state() -> UpgradesState {
        UpgradesState {
            coins: Decimal::from_finite(1e10),
            prestige_points: Decimal::from_finite(1e10),
            transcend_points: Decimal::from_finite(1e10),
            reincarnation_points: Decimal::from_finite(1e10),
            upgrades: [0; UPGRADES_DEFAULT_LEN],
            prestige_no_coin_upgrades: true,
            transcend_no_coin_upgrades: true,
            transcend_no_coin_or_prestige_upgrades: true,
            reincarnate_no_coin_upgrades: true,
            reincarnate_no_coin_or_prestige_upgrades: true,
            reincarnate_no_coin_prestige_or_transcend_upgrades: true,
            reincarnate_no_coin_prestige_transcend_or_generator_upgrades: true,
        }
    }

    fn input(tier: UpgradeTier, pos: u32, cost_exp: f64) -> BuyUpgradeInput {
        BuyUpgradeInput {
            tier,
            pos,
            cost_exponent: cost_exp,
            requirement_exists: true,
        }
    }

    #[test]
    fn missing_requirement_is_noop() {
        let mut state = baseline_state();
        let baseline_upgrades = state.upgrades;
        let baseline_coins = state.coins;
        let mut inp = input(UpgradeTier::Coin, 1, 2.0);
        inp.requirement_exists = false;
        let events = buy_upgrades(&mut state, inp);
        assert_eq!(state.upgrades, baseline_upgrades);
        assert_eq!(state.coins, baseline_coins);
        assert!(events.is_empty());
        // Flag flips skipped too.
        assert!(state.prestige_no_coin_upgrades);
    }

    #[test]
    fn buy_coin_upgrade_deducts_coins_and_sets_bitmap() {
        let mut state = baseline_state();
        // Cost = 10^2 = 100.
        let events = buy_upgrades(&mut state, input(UpgradeTier::Coin, 5, 2.0));
        assert_eq!(state.upgrades[5], 1);
        assert!((state.coins.to_number() - (1e10 - 100.0)).abs() < 1.0);
        assert_eq!(events.len(), 1);
        match &events[0] {
            CoreEvent::UpgradePurchased { tier, pos, spent } => {
                assert_eq!(*tier, UpgradeTier::Coin);
                assert_eq!(*pos, 5);
                assert!((spent.to_number() - 100.0).abs() < 1e-9);
            }
            other => panic!("expected UpgradePurchased, got {other:?}"),
        }
    }

    #[test]
    fn unaffordable_buy_is_skipped_but_flips_flags() {
        let mut state = UpgradesState {
            coins: Decimal::from_finite(50.0),
            ..baseline_state()
        };
        let baseline_coins = state.coins;
        // Cost 100, only 50 coins.
        let events = buy_upgrades(&mut state, input(UpgradeTier::Coin, 5, 2.0));
        assert_eq!(state.upgrades[5], 0);
        assert_eq!(state.coins, baseline_coins);
        assert!(events.is_empty());
        // Flag flips still fire — coin-tier touch.
        assert!(!state.prestige_no_coin_upgrades);
        assert!(!state.transcend_no_coin_upgrades);
        assert!(!state.reincarnate_no_coin_upgrades);
    }

    #[test]
    fn already_owned_is_skipped_but_flips_flags() {
        let mut state = baseline_state();
        state.upgrades[5] = 1;
        let baseline_coins = state.coins;
        let events = buy_upgrades(&mut state, input(UpgradeTier::Coin, 5, 2.0));
        assert_eq!(state.coins, baseline_coins);
        assert!(events.is_empty());
        assert!(!state.prestige_no_coin_upgrades);
    }

    #[test]
    fn coin_tier_flips_all_seven_flags() {
        let mut state = baseline_state();
        let _ = buy_upgrades(&mut state, input(UpgradeTier::Coin, 5, 2.0));
        assert!(!state.prestige_no_coin_upgrades);
        assert!(!state.transcend_no_coin_upgrades);
        assert!(!state.transcend_no_coin_or_prestige_upgrades);
        assert!(!state.reincarnate_no_coin_upgrades);
        assert!(!state.reincarnate_no_coin_or_prestige_upgrades);
        assert!(!state.reincarnate_no_coin_prestige_or_transcend_upgrades);
        assert!(!state.reincarnate_no_coin_prestige_transcend_or_generator_upgrades);
    }

    #[test]
    fn prestige_tier_flips_only_prestige_aware_flags() {
        let mut state = baseline_state();
        let _ = buy_upgrades(&mut state, input(UpgradeTier::Prestige, 5, 2.0));
        // Coin-aware flags should still be true.
        assert!(state.prestige_no_coin_upgrades);
        assert!(state.transcend_no_coin_upgrades);
        assert!(state.reincarnate_no_coin_upgrades);
        // Prestige-aware flags should flip.
        assert!(!state.transcend_no_coin_or_prestige_upgrades);
        assert!(!state.reincarnate_no_coin_or_prestige_upgrades);
        assert!(!state.reincarnate_no_coin_prestige_or_transcend_upgrades);
        assert!(!state.reincarnate_no_coin_prestige_transcend_or_generator_upgrades);
    }

    #[test]
    fn transcend_tier_flips_two_reincarnate_flags() {
        let mut state = baseline_state();
        let _ = buy_upgrades(&mut state, input(UpgradeTier::Transcend, 5, 2.0));
        // Only the two transcend-aware reincarnate flags flip.
        assert!(state.prestige_no_coin_upgrades);
        assert!(state.transcend_no_coin_upgrades);
        assert!(state.transcend_no_coin_or_prestige_upgrades);
        assert!(state.reincarnate_no_coin_upgrades);
        assert!(state.reincarnate_no_coin_or_prestige_upgrades);
        assert!(!state.reincarnate_no_coin_prestige_or_transcend_upgrades);
        assert!(!state.reincarnate_no_coin_prestige_transcend_or_generator_upgrades);
    }

    #[test]
    fn reincarnation_tier_flips_no_flags() {
        let mut state = baseline_state();
        let _ = buy_upgrades(&mut state, input(UpgradeTier::Reincarnation, 5, 2.0));
        // Every flag still true.
        assert!(state.prestige_no_coin_upgrades);
        assert!(state.transcend_no_coin_upgrades);
        assert!(state.transcend_no_coin_or_prestige_upgrades);
        assert!(state.reincarnate_no_coin_upgrades);
        assert!(state.reincarnate_no_coin_or_prestige_upgrades);
        assert!(state.reincarnate_no_coin_prestige_or_transcend_upgrades);
        assert!(state.reincarnate_no_coin_prestige_transcend_or_generator_upgrades);
    }

    #[test]
    fn each_tier_pays_from_correct_currency() {
        let cost_exp = 2.0; // 100 in each currency
        let baseline = baseline_state();

        // Coin
        let mut state = baseline.clone();
        let _ = buy_upgrades(&mut state, input(UpgradeTier::Coin, 5, cost_exp));
        assert!(state.coins < baseline.coins);
        assert_eq!(state.prestige_points, baseline.prestige_points);

        // Prestige
        let mut state = baseline.clone();
        let _ = buy_upgrades(&mut state, input(UpgradeTier::Prestige, 6, cost_exp));
        assert_eq!(state.coins, baseline.coins);
        assert!(state.prestige_points < baseline.prestige_points);
        assert_eq!(state.transcend_points, baseline.transcend_points);

        // Transcend
        let mut state = baseline.clone();
        let _ = buy_upgrades(&mut state, input(UpgradeTier::Transcend, 7, cost_exp));
        assert_eq!(state.prestige_points, baseline.prestige_points);
        assert!(state.transcend_points < baseline.transcend_points);
        assert_eq!(state.reincarnation_points, baseline.reincarnation_points);

        // Reincarnation
        let mut state = baseline.clone();
        let _ = buy_upgrades(&mut state, input(UpgradeTier::Reincarnation, 8, cost_exp));
        assert_eq!(state.transcend_points, baseline.transcend_points);
        assert!(state.reincarnation_points < baseline.reincarnation_points);
    }
}
