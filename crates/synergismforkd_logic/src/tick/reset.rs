//! Manual reset *execution*. Ports the always-runs base of the legacy
//! `reset()` (`legacy/core_split/packages/web_ui/src/Reset.ts:412`).
//!
//! Where [`auto_reset`](super::auto_reset) only *decides* whether a reset
//! should fire (emitting a [`CoreEvent::AutoResetTriggered`] intent), this
//! module actually *performs* the reset against `&mut GameState`. Today
//! only the prestige tier — the always-runs base reset (Reset.ts:422-486)
//! — is wired; the higher tiers (transcension / reincarnation / ascension
//! / singularity) cascade on top of it and port behind the regroup.

use smallvec::{smallvec, SmallVec};

use synergismforkd_bignum::Decimal;

use crate::events::{AutoResetTier, CoreEvent};
use crate::mechanics::reset_currency::ResetCurrencyResult;
use crate::state::GameState;
use crate::tick::ResetRequest;

/// Per-tier coin-producer base cost the prestige reset restores. Mirrors
/// `player.{first..fifth}CostCoin` (Reset.ts:428-440).
const COIN_BASE_COSTS: [f64; 5] = [100.0, 1_000.0, 20_000.0, 400_000.0, 8_000_000.0];

/// Execute a manual reset. The dispatch mirrors
/// [`dispatch_buy`](super::dispatch_buy); only [`ResetRequest::Prestige`]
/// is wired today.
pub(crate) fn perform_reset(
    state: &mut GameState,
    request: ResetRequest,
    gains: &ResetCurrencyResult,
) -> SmallVec<[CoreEvent; 2]> {
    match request {
        ResetRequest::Prestige => perform_prestige_reset(state, gains.prestige_point_gain),
    }
}

/// Port of the always-runs base reset (`reset('prestige')`,
/// Reset.ts:422-486): zero the coin economy, restore the prestige-tier
/// producers, award `prestige_point_gain`, bump the prestige count, and
/// clear the prestige timers + achievement gates. The award value is the
/// tick's `G.prestigePointGain` (computed by `compute_reset_currency_gains`
/// at the top of [`tack`](super::tack), before this phase runs).
///
/// Faithful to `reset()`, this is **ungated** — prestiging with too few
/// coins simply awards `0`; any can-I-prestige guard is a UI-tier concern.
///
/// Deferred from the legacy (each faithful at current state, documented):
/// - `resetOfferings()` (Runes.ts:934) — the per-reset offering award
///   pulls in `calculateOfferings()`; offerings are left untouched.
/// - the `updatePrestigeCount` multiplier
///   (`getAchievementReward('prestigeCountMultiplier')` ×
///   `1 + 0.05·CalcECC('transcend', completions[5])`) — both factors are
///   `1` at default, so the count rises by exactly `1`.
/// - `awardAchievementGroup('prestigeCount')` — achievement *awarding*
///   (flag-setting) is a separate subsystem.
/// - `G.generatorPower = 1` (recomputed per tick, not stored) and
///   `player.fastestprestige` (a record-keeping stat with no state field).
fn perform_prestige_reset(
    state: &mut GameState,
    prestige_point_gain: Decimal,
) -> SmallVec<[CoreEvent; 2]> {
    // ── Coin economy ────────────────────────────────────────────────
    state.upgrades.coins = Decimal::from_finite(102.0);
    state.coin_counters.coins_this_prestige = Decimal::from_finite(100.0);
    for (tier, cost) in state.coin_producers.tiers.iter_mut().zip(COIN_BASE_COSTS) {
        tier.owned = 0.0;
        tier.generated = Decimal::zero();
        tier.cost = Decimal::from_finite(cost);
    }
    // Prestige-tier (diamond) producers persist through a prestige; only
    // their auto-generated cascade count resets (Reset.ts:441-445).
    for tier in &mut state.diamond_producers.tiers {
        tier.generated = Decimal::zero();
    }
    state.multiplier.multiplier_cost = Decimal::from_finite(10_000.0);
    state.multiplier.multiplier_bought = 0.0;
    state.accelerator.accelerator_cost = Decimal::from_finite(500.0);
    state.accelerator.accelerator_bought = 0.0;

    // ── resetUpgrades(1) (Reset.ts:1369-1377) — the always-runs slots ──
    // Coin-tier upgrades 1..=20 plus the paired 106..=110 / 121..=125
    // blocks. The `i > 1.5` / `i > 2.5` branches belong to higher tiers.
    for slot in 1..=20 {
        state.upgrades.upgrades[slot] = 0;
    }
    for slot in 106..=110 {
        state.upgrades.upgrades[slot] = 0;
    }
    for slot in 121..=125 {
        state.upgrades.upgrades[slot] = 0;
    }

    // ── Counters, currency award, achievement gates ─────────────────
    state.reset_counters.prestige_count += 1.0;
    state.upgrades.prestige_points += prestige_point_gain;
    // `player.prestigeShards = 0`. The current-shards value lives in
    // `crystal_upgrades`; `reset_counters.prestige_shards` is a
    // (currently always-zero) read-base in `resource_gain` — zero both so
    // the reset holds regardless of which the next tick reads.
    state.crystal_upgrades.prestige_shards = Decimal::zero();
    state.reset_counters.prestige_shards = Decimal::zero();
    state.accelerator.prestige_no_accelerator = true;
    state.multiplier.prestige_no_multiplier = true;
    state.upgrades.prestige_no_coin_upgrades = true;
    state.reset_counters.prestige_unlocked = true;
    state.reset_counters.prestige_counter = 0.0;
    state.automation.auto_reset_timer_prestige = 0.0;

    smallvec![CoreEvent::ResetPerformed {
        tier: AutoResetTier::Prestige,
        points_gained: prestige_point_gain,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fully self-contained prestige reset: dirty the coin economy +
    /// some coin upgrades, then assert every base-reset field lands on its
    /// post-reset value and the award is credited.
    #[test]
    fn prestige_reset_zeroes_coin_economy_and_awards_points() {
        let mut state = GameState::default();
        // Dirty everything the base reset is supposed to clear.
        state.upgrades.coins = Decimal::from_finite(1e20);
        state.coin_counters.coins_this_prestige = Decimal::from_finite(1e18);
        state.coin_producers.tiers[0].owned = 42.0;
        state.coin_producers.tiers[0].cost = Decimal::from_finite(999.0);
        state.coin_producers.tiers[4].generated = Decimal::from_finite(7.0);
        state.diamond_producers.tiers[2].generated = Decimal::from_finite(3.0);
        state.diamond_producers.tiers[2].owned = 5.0; // must SURVIVE prestige
        state.multiplier.multiplier_bought = 9.0;
        state.accelerator.accelerator_bought = 9.0;
        state.upgrades.upgrades[5] = 1; // coin upgrade — must reset
        state.upgrades.upgrades[124] = 1; // paired block — must reset
        state.upgrades.upgrades[50] = 1; // outside the prestige slots — survives
        state.upgrades.prestige_no_coin_upgrades = false;
        state.reset_counters.prestige_counter = 123.0;
        state.automation.auto_reset_timer_prestige = 4.5;

        let events = perform_prestige_reset(&mut state, Decimal::from_finite(1000.0));

        assert_eq!(state.upgrades.coins.to_number(), 102.0);
        assert_eq!(state.coin_counters.coins_this_prestige.to_number(), 100.0);
        assert_eq!(state.coin_producers.tiers[0].owned, 0.0);
        assert_eq!(state.coin_producers.tiers[0].cost.to_number(), 100.0);
        assert_eq!(state.coin_producers.tiers[4].generated.to_number(), 0.0);
        assert_eq!(state.diamond_producers.tiers[2].generated.to_number(), 0.0);
        assert_eq!(state.diamond_producers.tiers[2].owned, 5.0); // survived
        assert_eq!(state.multiplier.multiplier_bought, 0.0);
        assert_eq!(state.multiplier.multiplier_cost.to_number(), 10_000.0);
        assert_eq!(state.accelerator.accelerator_bought, 0.0);
        assert_eq!(state.accelerator.accelerator_cost.to_number(), 500.0);
        assert_eq!(state.upgrades.upgrades[5], 0);
        assert_eq!(state.upgrades.upgrades[124], 0);
        assert_eq!(state.upgrades.upgrades[50], 1); // untouched
        assert_eq!(state.upgrades.prestige_points.to_number(), 1000.0);
        assert!(state.upgrades.prestige_no_coin_upgrades);
        assert_eq!(state.reset_counters.prestige_count, 1.0);
        assert!(state.reset_counters.prestige_unlocked);
        assert_eq!(state.reset_counters.prestige_counter, 0.0);
        assert_eq!(state.automation.auto_reset_timer_prestige, 0.0);

        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CoreEvent::ResetPerformed {
                tier: AutoResetTier::Prestige,
                points_gained,
            } if points_gained.to_number() == 1000.0
        ));
    }

    /// Faithful to `reset()`: prestiging with no accumulated coins awards
    /// `0` points but still performs the reset (the count still rises).
    #[test]
    fn prestige_reset_is_ungated_zero_gain_still_resets() {
        let mut state = GameState::default();
        state.coin_producers.tiers[0].owned = 10.0;

        let events = perform_prestige_reset(&mut state, Decimal::zero());

        assert_eq!(state.upgrades.prestige_points.to_number(), 0.0);
        assert_eq!(state.coin_producers.tiers[0].owned, 0.0);
        assert_eq!(state.reset_counters.prestige_count, 1.0);
        assert_eq!(events.len(), 1);
    }
}
