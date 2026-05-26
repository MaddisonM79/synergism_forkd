//! Tier-keyed upgrade purchases.
//!
//! Verbatim port of `buyUpgrades` from
//! `legacy_core_split/packages/logic/src/mechanics/upgrades.ts`. Each
//! resource tier (coin / prestige / transcend / reincarnation) dispatches
//! to its own currency and, on a successful buy, flips a different set
//! of `*_no_*_upgrades` achievement-gate flags. The flag-flip happens
//! unconditionally per call (not gated on affordability) — matches the
//! original behavior.

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
pub fn buy_upgrades(
    state: &UpgradesState,
    input: BuyUpgradeInput,
) -> (UpgradesState, Vec<CoreEvent>) {
    let mut events: Vec<CoreEvent> = Vec::new();
    let mut next = state.clone();

    // Out-of-bounds guard from the original buy_upgrades. Returns the
    // cloned state untouched (no flag flips, no purchase) when the
    // requirement entry doesn't exist.
    if !input.requirement_exists {
        return (next, events);
    }

    let cost = Decimal::from_finite(10.0).pow(Decimal::from_finite(input.cost_exponent));

    // Helper: read the currency for the requested tier.
    let current = match input.tier {
        UpgradeTier::Coin => next.coins,
        UpgradeTier::Prestige => next.prestige_points,
        UpgradeTier::Transcend => next.transcend_points,
        UpgradeTier::Reincarnation => next.reincarnation_points,
    };

    let pos_index = input.pos as usize;
    let already_owned = next.upgrades.get(pos_index).is_none_or(|&owned| owned != 0);

    // Purchase attempt. Mirrors the legacy guard exactly: affordable AND
    // not already owned. On success, deduct cost, set bitmap entry, emit
    // event.
    if current >= cost && !already_owned {
        match input.tier {
            UpgradeTier::Coin => next.coins -= cost,
            UpgradeTier::Prestige => next.prestige_points -= cost,
            UpgradeTier::Transcend => next.transcend_points -= cost,
            UpgradeTier::Reincarnation => next.reincarnation_points -= cost,
        }
        next.upgrades[pos_index] = 1;
        events.push(CoreEvent::UpgradePurchased {
            tier: input.tier,
            pos: input.pos,
            spent: cost,
        });
    }

    // Flag-flip matrix — independent of buy success.
    match input.tier {
        UpgradeTier::Transcend => {
            next.reincarnate_no_coin_prestige_or_transcend_upgrades = false;
            next.reincarnate_no_coin_prestige_transcend_or_generator_upgrades = false;
        }
        UpgradeTier::Prestige => {
            next.transcend_no_coin_or_prestige_upgrades = false;
            next.reincarnate_no_coin_or_prestige_upgrades = false;
            next.reincarnate_no_coin_prestige_or_transcend_upgrades = false;
            next.reincarnate_no_coin_prestige_transcend_or_generator_upgrades = false;
        }
        UpgradeTier::Coin => {
            next.prestige_no_coin_upgrades = false;
            next.transcend_no_coin_upgrades = false;
            next.transcend_no_coin_or_prestige_upgrades = false;
            next.reincarnate_no_coin_upgrades = false;
            next.reincarnate_no_coin_or_prestige_upgrades = false;
            next.reincarnate_no_coin_prestige_or_transcend_upgrades = false;
            next.reincarnate_no_coin_prestige_transcend_or_generator_upgrades = false;
        }
        UpgradeTier::Reincarnation => {}
    }

    (next, events)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_state() -> UpgradesState {
        UpgradesState {
            coins: Decimal::from_finite(1e10),
            prestige_points: Decimal::from_finite(1e10),
            transcend_points: Decimal::from_finite(1e10),
            reincarnation_points: Decimal::from_finite(1e10),
            upgrades: vec![0; 50],
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
        let state = baseline_state();
        let mut inp = input(UpgradeTier::Coin, 1, 2.0);
        inp.requirement_exists = false;
        let (next, events) = buy_upgrades(&state, inp);
        assert_eq!(next.upgrades, state.upgrades);
        assert_eq!(next.coins, state.coins);
        assert!(events.is_empty());
        // Flag flips skipped too.
        assert!(next.prestige_no_coin_upgrades);
    }

    #[test]
    fn buy_coin_upgrade_deducts_coins_and_sets_bitmap() {
        let state = baseline_state();
        // Cost = 10^2 = 100.
        let (next, events) = buy_upgrades(&state, input(UpgradeTier::Coin, 5, 2.0));
        assert_eq!(next.upgrades[5], 1);
        assert!((next.coins.to_number() - (1e10 - 100.0)).abs() < 1.0);
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
        let state = UpgradesState {
            coins: Decimal::from_finite(50.0),
            ..baseline_state()
        };
        // Cost 100, only 50 coins.
        let (next, events) = buy_upgrades(&state, input(UpgradeTier::Coin, 5, 2.0));
        assert_eq!(next.upgrades[5], 0);
        assert_eq!(next.coins, state.coins);
        assert!(events.is_empty());
        // Flag flips still fire — coin-tier touch.
        assert!(!next.prestige_no_coin_upgrades);
        assert!(!next.transcend_no_coin_upgrades);
        assert!(!next.reincarnate_no_coin_upgrades);
    }

    #[test]
    fn already_owned_is_skipped_but_flips_flags() {
        let mut state = baseline_state();
        state.upgrades[5] = 1;
        let (next, events) = buy_upgrades(&state, input(UpgradeTier::Coin, 5, 2.0));
        assert_eq!(next.coins, state.coins);
        assert!(events.is_empty());
        assert!(!next.prestige_no_coin_upgrades);
    }

    #[test]
    fn coin_tier_flips_all_seven_flags() {
        let state = baseline_state();
        let (next, _) = buy_upgrades(&state, input(UpgradeTier::Coin, 5, 2.0));
        assert!(!next.prestige_no_coin_upgrades);
        assert!(!next.transcend_no_coin_upgrades);
        assert!(!next.transcend_no_coin_or_prestige_upgrades);
        assert!(!next.reincarnate_no_coin_upgrades);
        assert!(!next.reincarnate_no_coin_or_prestige_upgrades);
        assert!(!next.reincarnate_no_coin_prestige_or_transcend_upgrades);
        assert!(!next.reincarnate_no_coin_prestige_transcend_or_generator_upgrades);
    }

    #[test]
    fn prestige_tier_flips_only_prestige_aware_flags() {
        let state = baseline_state();
        let (next, _) = buy_upgrades(&state, input(UpgradeTier::Prestige, 5, 2.0));
        // Coin-aware flags should still be true.
        assert!(next.prestige_no_coin_upgrades);
        assert!(next.transcend_no_coin_upgrades);
        assert!(next.reincarnate_no_coin_upgrades);
        // Prestige-aware flags should flip.
        assert!(!next.transcend_no_coin_or_prestige_upgrades);
        assert!(!next.reincarnate_no_coin_or_prestige_upgrades);
        assert!(!next.reincarnate_no_coin_prestige_or_transcend_upgrades);
        assert!(!next.reincarnate_no_coin_prestige_transcend_or_generator_upgrades);
    }

    #[test]
    fn transcend_tier_flips_two_reincarnate_flags() {
        let state = baseline_state();
        let (next, _) = buy_upgrades(&state, input(UpgradeTier::Transcend, 5, 2.0));
        // Only the two transcend-aware reincarnate flags flip.
        assert!(next.prestige_no_coin_upgrades);
        assert!(next.transcend_no_coin_upgrades);
        assert!(next.transcend_no_coin_or_prestige_upgrades);
        assert!(next.reincarnate_no_coin_upgrades);
        assert!(next.reincarnate_no_coin_or_prestige_upgrades);
        assert!(!next.reincarnate_no_coin_prestige_or_transcend_upgrades);
        assert!(!next.reincarnate_no_coin_prestige_transcend_or_generator_upgrades);
    }

    #[test]
    fn reincarnation_tier_flips_no_flags() {
        let state = baseline_state();
        let (next, _) = buy_upgrades(&state, input(UpgradeTier::Reincarnation, 5, 2.0));
        // Every flag still true.
        assert!(next.prestige_no_coin_upgrades);
        assert!(next.transcend_no_coin_upgrades);
        assert!(next.transcend_no_coin_or_prestige_upgrades);
        assert!(next.reincarnate_no_coin_upgrades);
        assert!(next.reincarnate_no_coin_or_prestige_upgrades);
        assert!(next.reincarnate_no_coin_prestige_or_transcend_upgrades);
        assert!(next.reincarnate_no_coin_prestige_transcend_or_generator_upgrades);
    }

    #[test]
    fn each_tier_pays_from_correct_currency() {
        let cost_exp = 2.0; // 100 in each currency

        // Coin
        let state = baseline_state();
        let (next, _) = buy_upgrades(&state, input(UpgradeTier::Coin, 5, cost_exp));
        assert!(next.coins < state.coins);
        assert_eq!(next.prestige_points, state.prestige_points);

        // Prestige
        let (next, _) = buy_upgrades(&state, input(UpgradeTier::Prestige, 6, cost_exp));
        assert_eq!(next.coins, state.coins);
        assert!(next.prestige_points < state.prestige_points);
        assert_eq!(next.transcend_points, state.transcend_points);

        // Transcend
        let (next, _) = buy_upgrades(&state, input(UpgradeTier::Transcend, 7, cost_exp));
        assert_eq!(next.prestige_points, state.prestige_points);
        assert!(next.transcend_points < state.transcend_points);
        assert_eq!(next.reincarnation_points, state.reincarnation_points);

        // Reincarnation
        let (next, _) = buy_upgrades(&state, input(UpgradeTier::Reincarnation, 8, cost_exp));
        assert_eq!(next.transcend_points, state.transcend_points);
        assert!(next.reincarnation_points < state.reincarnation_points);
    }
}
