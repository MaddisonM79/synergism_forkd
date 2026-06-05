//! Reset *execution*. Ports the cascading legacy `reset()`
//! (`legacy/core_split/packages/web_ui/src/Reset.ts:412`).
//!
//! Where [`auto_reset`](super::auto_reset) only *decides* whether a reset
//! should fire (emitting a [`CoreEvent::AutoResetTriggered`] intent), this
//! module actually *performs* the reset against `&mut GameState`.
//!
//! The legacy `reset()` is one function whose blocks cascade: a
//! transcension also runs the prestige base, a reincarnation also runs the
//! transcension block, etc. We mirror that with layered helpers
//! ([`apply_base_reset`] → [`apply_transcension_layer`] →
//! [`apply_reincarnation_layer`]), each composed by a public
//! `perform_*_reset`. Tiers wired today: **prestige, transcension,
//! reincarnation**. Ascension / singularity (cubes, hepteracts,
//! corruptions, the paused singularity layer) are not ported yet.

use smallvec::{smallvec, SmallVec};

use synergismforkd_bignum::Decimal;

use crate::events::{AutoResetTier, CoreEvent};
use crate::mechanics::reset_currency::ResetCurrencyResult;
use crate::state::{GameState, DEFLATION_INDEX};
use crate::tick::ResetRequest;

/// Per-tier coin-producer base cost the prestige reset restores. Mirrors
/// `player.{first..fifth}CostCoin` (Reset.ts:428-440).
const COIN_BASE_COSTS: [f64; 5] = [100.0, 1_000.0, 20_000.0, 400_000.0, 8_000_000.0];
/// Diamond-producer base costs restored by the transcension layer
/// (`player.{first..fifth}CostDiamonds`, Reset.ts:492-500).
const DIAMOND_BASE_COSTS: [f64; 5] = [100.0, 1e5, 1e15, 1e40, 1e100];
/// Mythos-producer base costs restored by the reincarnation layer
/// (`player.{first..fifth}CostMythos`, Reset.ts:577-585).
const MYTHOS_BASE_COSTS: [f64; 5] = [1.0, 1e2, 1e4, 1e8, 1e16];

/// `coinsThisTranscension` floor for a transcension to credit a count
/// (`transcensionCheck`, Reset.ts:416).
const TRANSCEND_COUNT_THRESHOLD: f64 = 1e100;
/// `transcendShards` floor for a reincarnation to credit obtainium + a
/// count (`reincarnationCheck`, Reset.ts:417).
const REINCARNATION_COUNT_THRESHOLD: f64 = 1e300;

/// Execute a manual reset. The dispatch mirrors
/// [`dispatch_buy`](super::dispatch_buy).
pub(crate) fn perform_reset(
    state: &mut GameState,
    request: ResetRequest,
    gains: &ResetCurrencyResult,
) -> SmallVec<[CoreEvent; 2]> {
    match request {
        ResetRequest::Prestige => perform_prestige_reset(state, gains.prestige_point_gain),
        ResetRequest::Transcension => perform_transcension_reset(state, gains),
        ResetRequest::Reincarnation => perform_reincarnation_reset(state, gains),
    }
}

// ─── Prestige ────────────────────────────────────────────────────────────

/// `reset('prestige')` — the always-runs base reset, emitting a single
/// [`CoreEvent::ResetPerformed`]. The award value is the tick's
/// `G.prestigePointGain` (computed by `compute_reset_currency_gains` at the
/// top of [`tack`](super::tack), before this phase runs).
///
/// Invoked by both the manual dispatch ([`perform_reset`]) and the
/// auto-reset tail in [`phase_automation`](super::phase_automation).
pub(crate) fn perform_prestige_reset(
    state: &mut GameState,
    prestige_point_gain: Decimal,
) -> SmallVec<[CoreEvent; 2]> {
    apply_base_reset(state, prestige_point_gain);
    smallvec![CoreEvent::ResetPerformed {
        tier: AutoResetTier::Prestige,
        points_gained: prestige_point_gain,
    }]
}

/// The always-runs base reset (Reset.ts:422-486) — no event. Zeroes the
/// coin economy, restores the coin producers, awards `prestige_point_gain`,
/// bumps the prestige count, and clears the prestige timers + gates.
///
/// Faithful to `reset()`, this is **ungated** — too-few coins simply award
/// `0`; any can-I-reset guard is a UI-tier concern.
///
/// Deferred from the legacy (each faithful at current state):
/// - `resetOfferings()` (Runes.ts:934) — pulls in `calculateOfferings()`.
/// - the `updatePrestigeCount` multiplier — `1` at default, so the count
///   rises by exactly `1`; the `transcendToPrestige` chain is `false`.
/// - `awardAchievementGroup` — achievement *awarding* is a separate
///   subsystem.
/// - `G.generatorPower = 1` (recomputed per tick) and `fastestprestige`
///   (a record-keeping stat with no state field).
fn apply_base_reset(state: &mut GameState, prestige_point_gain: Decimal) {
    state.upgrades.coins = Decimal::from_finite(102.0);
    state.coin_counters.coins_this_prestige = Decimal::from_finite(100.0);
    for (tier, cost) in state.coin_producers.tiers.iter_mut().zip(COIN_BASE_COSTS) {
        tier.owned = 0.0;
        tier.generated = Decimal::zero();
        tier.cost = Decimal::from_finite(cost);
    }
    // Prestige-tier (diamond) producers persist through a prestige; only
    // their auto-generated cascade count resets here (Reset.ts:441-445).
    for tier in &mut state.diamond_producers.tiers {
        tier.generated = Decimal::zero();
    }
    state.multiplier.multiplier_cost = Decimal::from_finite(10_000.0);
    state.multiplier.multiplier_bought = 0.0;
    state.accelerator.accelerator_cost = Decimal::from_finite(500.0);
    state.accelerator.accelerator_bought = 0.0;

    // resetUpgrades(1) (Reset.ts:1369-1377) — the always-runs slots.
    reset_upgrade_slots(state, 1..=20);
    reset_upgrade_slots(state, 106..=110);
    reset_upgrade_slots(state, 121..=125);

    state.reset_counters.prestige_count += 1.0;
    state.upgrades.prestige_points += prestige_point_gain;
    // `player.prestigeShards = 0`. The current-shards value lives in
    // `crystal_upgrades`; `reset_counters.prestige_shards` is a (currently
    // always-zero) read-base in `resource_gain` — zero both so the reset
    // holds regardless of which the next tick reads.
    state.crystal_upgrades.prestige_shards = Decimal::zero();
    state.reset_counters.prestige_shards = Decimal::zero();
    state.accelerator.prestige_no_accelerator = true;
    state.multiplier.prestige_no_multiplier = true;
    state.upgrades.prestige_no_coin_upgrades = true;
    state.reset_counters.prestige_unlocked = true;
    state.reset_counters.prestige_counter = 0.0;
    state.automation.auto_reset_timer_prestige = 0.0;
}

// ─── Transcension ──────────────────────────────────────────────────────────

/// `reset('transcension')` — base reset + transcension layer, emitting one
/// [`CoreEvent::ResetPerformed`].
pub(crate) fn perform_transcension_reset(
    state: &mut GameState,
    gains: &ResetCurrencyResult,
) -> SmallVec<[CoreEvent; 2]> {
    apply_base_reset(state, gains.prestige_point_gain);
    apply_transcension_layer(state, gains.transcend_point_gain);
    smallvec![CoreEvent::ResetPerformed {
        tier: AutoResetTier::Transcension,
        points_gained: gains.transcend_point_gain,
    }]
}

/// The transcension block (Reset.ts:488-547) — no event. Assumes the base
/// reset already ran. The base awards prestige points; this layer then
/// zeroes them and converts to the transcend layer (faithful to the
/// legacy award-then-zero ordering at lines 453 / 515).
///
/// Deferred (faithful at current state): the `resetUpgrades(2)`
/// `crystalUpgradesCost` reset (costs aren't modeled), `acceleratorBoostCost`
/// (no state field — the buy-boost action is unported), the `tierNCrystalAutobuy`
/// level-milestone diamond grants (`0` at default), `awardAchievementGroup`,
/// and `fastesttranscend`.
fn apply_transcension_layer(state: &mut GameState, transcend_point_gain: Decimal) {
    // `transcensionCheck` is the *pre-reset* coin total — capture it before
    // this layer zeroes `coins_this_transcension`.
    let transcension_check = state.coin_counters.coins_this_transcension
        >= Decimal::from_finite(TRANSCEND_COUNT_THRESHOLD);

    // resetUpgrades(2) — the `i > 1.5` additions on top of the base's
    // always-runs slots (Reset.ts:1379-1408).
    reset_upgrade_slots(state, 21..=40);
    reset_upgrade_slots(state, 101..=105);
    reset_upgrade_slots(state, 111..=115);
    // Crystal-upgrade levels reset, with the legacy `+10` bonus when
    // upgrade-73 is owned inside a reincarnation challenge (Reset.ts:1400-1407).
    let crystal_level = if state.upgrades.upgrades[73] > 0
        && state.challenges.current_reincarnation_challenge != 0
    {
        10.0
    } else {
        0.0
    };
    state.crystal_upgrades.crystal_upgrades = [crystal_level; 8];

    state.coin_counters.coins_this_transcension = Decimal::from_finite(100.0);
    for (tier, cost) in state
        .diamond_producers
        .tiers
        .iter_mut()
        .zip(DIAMOND_BASE_COSTS)
    {
        tier.owned = 0.0;
        tier.cost = Decimal::from_finite(cost);
    }
    // Transcend-tier (mythos) producers persist; only their cascade resets.
    for tier in &mut state.mythos_producers.tiers {
        tier.generated = Decimal::zero();
    }
    state.accelerator.accelerator_boost_bought = 0.0;

    // updateTranscensionCount(1) — multiplier is `1` at default, so +1.
    if transcension_check {
        state.reset_counters.transcend_count += 1.0;
    }
    state.upgrades.prestige_points = Decimal::zero();
    state.upgrades.transcend_points += transcend_point_gain;
    state.reset_counters.transcend_shards = Decimal::zero();
    state.upgrades.transcend_no_coin_upgrades = true;
    state.upgrades.transcend_no_coin_or_prestige_upgrades = true;
    state.accelerator.transcend_no_accelerator = true;
    state.multiplier.transcend_no_multiplier = true;
    state.reset_counters.transcend_counter = 0.0;
    state.automation.auto_reset_timer_transcension = 0.0;
}

// ─── Reincarnation ─────────────────────────────────────────────────────────

/// `reset('reincarnation')` — base + transcension + reincarnation layers,
/// emitting one [`CoreEvent::ResetPerformed`].
pub(crate) fn perform_reincarnation_reset(
    state: &mut GameState,
    gains: &ResetCurrencyResult,
) -> SmallVec<[CoreEvent; 2]> {
    // `reincarnationCheck` is the *pre-reset* shard total — capture it
    // before the transcension layer zeroes `transcend_shards`.
    let reincarnation_check = state.reset_counters.transcend_shards
        >= Decimal::from_finite(REINCARNATION_COUNT_THRESHOLD);
    apply_base_reset(state, gains.prestige_point_gain);
    apply_transcension_layer(state, gains.transcend_point_gain);
    apply_reincarnation_layer(state, gains.reincarnation_point_gain, reincarnation_check);
    smallvec![CoreEvent::ResetPerformed {
        tier: AutoResetTier::Reincarnation,
        points_gained: gains.reincarnation_point_gain,
    }]
}

/// The reincarnation block (Reset.ts:549-626) — no event. Assumes the base
/// + transcension layers already ran.
///
/// Deferred (faithful at current state): the **obtainium award**
/// (`obtainium += calculateObtainium()`) — `calculateObtainium` needs the
/// unported `calculateBaseObtainium`, and is `0` at default; the
/// `instantChallenge` shop completion-restore (shop level `0`);
/// `awardAchievementGroup` / `awardUngroupedAchievement`; and
/// `fastestreincarnate`.
fn apply_reincarnation_layer(
    state: &mut GameState,
    reincarnation_point_gain: Decimal,
    reincarnation_check: bool,
) {
    // Deflation-corruption bonus (Reset.ts:550-552) — false at default
    // (deflation level `0`, platonic upgrade `0`).
    if state.corruptions.used.levels[DEFLATION_INDEX] > 10
        && state.cube_upgrade_levels.platonic_upgrades[11] > 0.0
    {
        state.upgrades.prestige_points += reincarnation_point_gain;
    }

    if reincarnation_check {
        // Obtainium award deferred (see fn doc): calculateObtainium unported.
        // updateReincarnationCount(1) — multiplier is `1` at default, so +1.
        state.reset_counters.reincarnation_count += 1.0;
    }

    state.challenges.current_transcension_challenge = 0;

    // resetUpgrades(3) — the `i > 2.5` additions (Reset.ts:1336-1366).
    for slot in 41..=60 {
        if slot != 46 {
            state.upgrades.upgrades[slot] = 0;
        }
    }
    if state.researches.researches[41] == 0.0 {
        state.upgrades.upgrades[46] = 0;
        state.upgrades.upgrades[88] = 0;
    }
    if state.researches.researches[42] == 0.0 {
        state.upgrades.upgrades[90] = 0;
    }
    if state.researches.researches[43] == 0.0 {
        state.upgrades.upgrades[91] = 0;
    }
    if state.researches.researches[44] == 0.0 {
        state.upgrades.upgrades[92] = 0;
    }
    if state.researches.researches[45] == 0.0 {
        state.upgrades.upgrades[93] = 0;
    }
    reset_upgrade_slots(state, 116..=120);

    state.coin_counters.coins_this_reincarnation = Decimal::from_finite(100.0);
    for (tier, cost) in state
        .mythos_producers
        .tiers
        .iter_mut()
        .zip(MYTHOS_BASE_COSTS)
    {
        tier.owned = 0.0;
        tier.cost = Decimal::from_finite(cost);
    }
    // Reincarnation-tier (particle) producers persist; only their cascade
    // count resets.
    for tier in &mut state.particle_producers.tiers {
        tier.generated = Decimal::zero();
    }

    state.upgrades.transcend_points = Decimal::zero();
    state.upgrades.reincarnation_points += reincarnation_point_gain;
    state.reset_counters.reincarnation_shards = Decimal::zero();
    for completion in &mut state.challenges.challenge_completions[1..=5] {
        *completion = 0.0;
    }
    state.upgrades.reincarnate_no_coin_upgrades = true;
    state.upgrades.reincarnate_no_coin_or_prestige_upgrades = true;
    state
        .upgrades
        .reincarnate_no_coin_prestige_or_transcend_upgrades = true;
    state
        .upgrades
        .reincarnate_no_coin_prestige_transcend_or_generator_upgrades = true;
    state.accelerator.reincarnate_no_accelerator = true;
    state.multiplier.reincarnate_no_multiplier = true;
    state.reset_counters.reincarnation_counter = 0.0;
    state.automation.auto_reset_timer_reincarnation = 0.0;
}

/// Zero a contiguous run of `player.upgrades` slots (the `resetUpgrades`
/// loops). Indices are the legacy 1-based positions, in range for the
/// `[u8; UPGRADES_DEFAULT_LEN]` bitmap.
fn reset_upgrade_slots(state: &mut GameState, slots: std::ops::RangeInclusive<usize>) {
    for slot in slots {
        state.upgrades.upgrades[slot] = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::reset_currency::ResetCurrencyResult;

    fn gains(prestige: f64, transcend: f64, reincarnation: f64) -> ResetCurrencyResult {
        ResetCurrencyResult {
            prestige_point_gain: Decimal::from_finite(prestige),
            transcend_point_gain: Decimal::from_finite(transcend),
            reincarnation_point_gain: Decimal::from_finite(reincarnation),
        }
    }

    #[test]
    fn prestige_reset_zeroes_coin_economy_and_awards_points() {
        let mut state = GameState::default();
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

    #[test]
    fn transcension_reset_resets_diamond_layer_and_converts_points() {
        let mut state = GameState::default();
        // Above the transcension count threshold (1e100).
        state.coin_counters.coins_this_transcension = Decimal::from_finite(1e120);
        state.diamond_producers.tiers[0].owned = 8.0;
        state.diamond_producers.tiers[0].cost = Decimal::from_finite(7.0);
        state.mythos_producers.tiers[1].generated = Decimal::from_finite(4.0);
        state.mythos_producers.tiers[1].owned = 6.0; // transcend producers SURVIVE
        state.crystal_upgrades.crystal_upgrades = [3.0; 8];
        state.upgrades.upgrades[30] = 1; // resetUpgrades(2) slot — must reset
        state.accelerator.accelerator_boost_bought = 4.0;

        let events = perform_transcension_reset(&mut state, &gains(50.0, 25.0, 0.0));

        // Base ran (prestige economy reset) then transcension layered on.
        assert_eq!(state.coin_counters.coins_this_prestige.to_number(), 100.0);
        assert_eq!(
            state.coin_counters.coins_this_transcension.to_number(),
            100.0
        );
        assert_eq!(state.diamond_producers.tiers[0].owned, 0.0);
        assert_eq!(state.diamond_producers.tiers[0].cost.to_number(), 100.0);
        assert_eq!(state.mythos_producers.tiers[1].generated.to_number(), 0.0);
        assert_eq!(state.mythos_producers.tiers[1].owned, 6.0); // survived
        assert_eq!(state.crystal_upgrades.crystal_upgrades, [0.0; 8]);
        assert_eq!(state.upgrades.upgrades[30], 0);
        assert_eq!(state.accelerator.accelerator_boost_bought, 0.0);
        // prestige points awarded by the base are zeroed by transcension;
        // transcend points credited; transcend count bumped.
        assert_eq!(state.upgrades.prestige_points.to_number(), 0.0);
        assert_eq!(state.upgrades.transcend_points.to_number(), 25.0);
        assert_eq!(state.reset_counters.transcend_count, 1.0);
        assert_eq!(state.reset_counters.prestige_count, 1.0); // base still counted
        assert_eq!(state.reset_counters.transcend_counter, 0.0);

        assert!(matches!(
            events[0],
            CoreEvent::ResetPerformed {
                tier: AutoResetTier::Transcension,
                points_gained,
            } if points_gained.to_number() == 25.0
        ));
    }

    #[test]
    fn reincarnation_reset_resets_mythos_layer_and_clears_challenges() {
        let mut state = GameState::default();
        // Above the reincarnation count threshold (1e300).
        state.reset_counters.transcend_shards = Decimal::from_finite(1e305);
        state.mythos_producers.tiers[0].owned = 9.0;
        state.mythos_producers.tiers[0].cost = Decimal::from_finite(2.0);
        state.particle_producers.tiers[2].generated = Decimal::from_finite(5.0);
        state.particle_producers.tiers[2].owned = 7.0; // particle producers SURVIVE
        state.challenges.challenge_completions[3] = 12.0; // c1-5 cleared
        state.challenges.challenge_completions[7] = 4.0; // c7 survives
        state.challenges.current_transcension_challenge = 2;
        state.upgrades.upgrades[55] = 1; // resetUpgrades(3) slot — must reset

        let events = perform_reincarnation_reset(&mut state, &gains(50.0, 25.0, 9.0));

        // All three layers ran.
        assert_eq!(
            state.coin_counters.coins_this_reincarnation.to_number(),
            100.0
        );
        assert_eq!(state.mythos_producers.tiers[0].owned, 0.0);
        assert_eq!(state.mythos_producers.tiers[0].cost.to_number(), 1.0);
        assert_eq!(state.particle_producers.tiers[2].generated.to_number(), 0.0);
        assert_eq!(state.particle_producers.tiers[2].owned, 7.0); // survived
        assert_eq!(state.challenges.challenge_completions[3], 0.0);
        assert_eq!(state.challenges.challenge_completions[7], 4.0); // untouched
        assert_eq!(state.challenges.current_transcension_challenge, 0);
        assert_eq!(state.upgrades.upgrades[55], 0);
        // transcend points (from the transcension layer) zeroed; reincarnation
        // points credited; reincarnation count bumped.
        assert_eq!(state.upgrades.transcend_points.to_number(), 0.0);
        assert_eq!(state.upgrades.reincarnation_points.to_number(), 9.0);
        assert_eq!(state.reset_counters.reincarnation_count, 1.0);
        assert_eq!(state.reset_counters.reincarnation_counter, 0.0);

        assert!(matches!(
            events[0],
            CoreEvent::ResetPerformed {
                tier: AutoResetTier::Reincarnation,
                points_gained,
            } if points_gained.to_number() == 9.0
        ));
    }
}
