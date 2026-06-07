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
//! [`apply_reincarnation_layer`] → [`apply_ascension_layer`]), each
//! composed by a public `perform_*_reset`. Tiers wired today: **prestige,
//! transcension, reincarnation, ascension** (the ascension tier ports the
//! structural reset + every per-feature sub-reset — `resetResearches` /
//! `resetChallengeSweep` / `resetRunes` / `resetAnts` / `resetTalismanData`;
//! its c10-gated cube/hepteract awards are implemented but inert at default;
//! see [`apply_ascension_layer`]). The singularity layer is paused.

use smallvec::{smallvec, SmallVec};

use synergismforkd_bignum::Decimal;

use crate::events::{AutoResetTier, CoreEvent, SweepState};
use crate::mechanics::reset_currency::ResetCurrencyResult;
use crate::state::{
    AntsState, GameState, TalismansState, DEFLATION_INDEX, RUNE_ANTIQUITIES, RUNE_COUNT,
    TALISMAN_COUNT,
};
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
/// Particle-producer base costs restored by the ascension layer
/// (`player.{first..fifth}CostParticles`, Reset.ts:666-670).
const PARTICLE_BASE_COSTS: [f64; 5] = [1.0, 100.0, 1e4, 1e8, 1e16];

/// `coinsThisTranscension` floor for a transcension to credit a count
/// (`transcensionCheck`, Reset.ts:416).
const TRANSCEND_COUNT_THRESHOLD: f64 = 1e100;
/// `transcendShards` floor for a reincarnation to credit obtainium + a
/// count (`reincarnationCheck`, Reset.ts:417).
const REINCARNATION_COUNT_THRESHOLD: f64 = 1e300;

/// `getResetResearches()` destroy-list (Reset.ts:1417-1431) — the research
/// slots an ascension wipes. 1-based legacy indices, in range for the
/// `[f64; RESEARCHES_LEN]` array.
const RESET_RESEARCHES: &[usize] = &[
    6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33,
    34, 35, 36, 37, 38, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 62, 63, 64, 65, 76, 81, 85, 86, 87,
    88, 89, 90, 91, 92, 93, 94, 96, 97, 98, 101, 102, 103, 104, 106, 107, 108, 109, 110, 116, 117,
    118, 121, 122, 123, 126, 127, 128, 129, 131, 132, 133, 134, 136, 137, 139, 141, 142, 143, 144,
    146, 147, 148, 149, 151, 152, 154, 156, 157, 158, 159, 161, 162, 163, 164, 166, 167, 169, 171,
    172, 173, 174, 176, 177, 178, 179, 181, 182, 184, 186, 187, 188, 189, 191, 192, 193, 194, 196,
    197, 199,
];
/// Extra researches the destroy-list adds while `highestSingularityCount`
/// is below 25 (Reset.ts:1433-1435).
const RESET_RESEARCHES_PRE_SING25: &[usize] = &[138, 153, 168, 183, 198];
/// `highestSingularityCount` threshold below which the pre-sing-25 extras
/// are wiped too.
const RESET_RESEARCHES_SING_THRESHOLD: f64 = 25.0;

/// Reset-count increment (legacy `updatePrestigeCount` / `updateTranscensionCount`
/// / `updateReincarnationCount`): `floor(count * multiplier)` with `count = 1` per
/// reset and `multiplier = achievementCountMultiplier * (1 + coeff * CalcECC(tier,
/// completions))`. The achievement count-multiplier reward is unported → neutral
/// `1.0`; the `CalcECC` term raises the increment once the gating challenge has
/// completions (transcend c5 → prestige, reincarnation c7 → transcension,
/// ascension c12 → reincarnation). Identity at the default state. (audit P1.6)
fn reset_count_increment(
    ecc_type: crate::mechanics::challenges::ChallengeType,
    completions: f64,
    ecc_coeff: f64,
) -> f64 {
    use crate::mechanics::challenges::calc_ecc;
    let achievement_count_multiplier = 1.0; // getAchievementReward(*CountMultiplier) unported
    (achievement_count_multiplier * (1.0 + ecc_coeff * calc_ecc(ecc_type, completions))).floor()
}
/// The Mortuus2 ant upgrade (index 15) — the one ant upgrade an ascension's
/// `resetAnts` leaves untouched, since it is singularity-tier while every
/// lower index is sacrifice- or ascension-tier (`AntUpgrades` data table).
const ANT_UPGRADE_MORTUUS2: usize = 15;

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
        ResetRequest::Ascension => perform_ascension_reset(state, gains),
        ResetRequest::AscensionChallenge => perform_ascension_challenge_reset(state, gains),
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

    state.reset_counters.prestige_count += reset_count_increment(
        crate::mechanics::challenges::ChallengeType::Transcend,
        state.challenges.challenge_completions[5],
        0.05,
    );
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

    // updateTranscensionCount(1) — floor(multiplier); multiplier = 1 at default.
    if transcension_check {
        state.reset_counters.transcend_count += reset_count_increment(
            crate::mechanics::challenges::ChallengeType::Reincarnation,
            state.challenges.challenge_completions[7],
            0.15,
        );
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
    // Obtainium award (Reset.ts:418, 568-569): `calculateObtainium()` is
    // captured *before* the reset mutates challenge completions / counters
    // (read via `calc_ecc`) and zeroes `reincarnation_counter` (the
    // reset-time multiplier reads it). `timeMultUsed = true` here — the
    // `offeringObtainiumTimeModifiers` product scales the award by reset
    // length.
    let obtainium_to_gain = if reincarnation_check {
        let base = super::compute_base_obtainium(state);
        let time_mult = super::compute_obtainium_time_multiplier(state);
        super::compute_obtainium(state, base, gains.reincarnation_point_gain, time_mult)
    } else {
        Decimal::zero()
    };
    apply_base_reset(state, gains.prestige_point_gain);
    apply_transcension_layer(state, gains.transcend_point_gain);
    apply_reincarnation_layer(
        state,
        gains.reincarnation_point_gain,
        reincarnation_check,
        obtainium_to_gain,
        // The deflation→prestige-points quirk (Reset.ts:549-552) is guarded
        // `input === 'reincarnation' | 'reincarnationChallenge'`, so it runs
        // for a manual reincarnation but NOT when an ascension cascades
        // through this layer.
        true,
    );
    smallvec![CoreEvent::ResetPerformed {
        tier: AutoResetTier::Reincarnation,
        points_gained: gains.reincarnation_point_gain,
    }]
}

/// The reincarnation block (Reset.ts:549-626) — no event. Assumes the base
/// and transcension layers already ran. `obtainium_to_gain` is the
/// pre-reset `calculateObtainium()` value computed by the caller.
///
/// `apply_deflation_prestige_quirk` gates the reincarnation-only
/// deflation→prestige-points bonus (Reset.ts:549-552): `true` for a manual
/// reincarnation, `false` when an ascension cascades through this layer
/// (the legacy block guards that bonus to reincarnation inputs).
///
/// Deferred (faithful at current state): the `instantChallenge` shop
/// completion-restore (shop level `0`), `awardAchievementGroup` /
/// `awardUngroupedAchievement`, and `fastestreincarnate`.
fn apply_reincarnation_layer(
    state: &mut GameState,
    reincarnation_point_gain: Decimal,
    reincarnation_check: bool,
    obtainium_to_gain: Decimal,
    apply_deflation_prestige_quirk: bool,
) {
    // Deflation-corruption bonus (Reset.ts:550-552) — false at default
    // (deflation level `0`, platonic upgrade `0`). Reincarnation-only: the
    // legacy guard excludes ascension.
    if apply_deflation_prestige_quirk
        && state.corruptions.used.levels[DEFLATION_INDEX] > 10
        && state.cube_upgrade_levels.platonic_upgrades[11] > 0.0
    {
        state.upgrades.prestige_points += reincarnation_point_gain;
    }

    if reincarnation_check {
        // Obtainium award (Reset.ts:568-569) — `obtainium_to_gain` was
        // computed pre-reset by `perform_reincarnation_reset`.
        state.researches.obtainium += obtainium_to_gain;
        // updateReincarnationCount(1) — floor(multiplier); multiplier = 1 at default.
        state.reset_counters.reincarnation_count += reset_count_increment(
            crate::mechanics::challenges::ChallengeType::Ascension,
            state.challenges.challenge_completions[12],
            0.2,
        );
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

// ─── Ascension ─────────────────────────────────────────────────────────────

/// `reset('ascension')` — base + transcension + reincarnation + ascension
/// layers, emitting one [`CoreEvent::ResetPerformed`].
///
/// The reincarnation layer runs with the deflation quirk **disabled** (the
/// legacy bonus is guarded to reincarnation inputs); its obtainium award — if
/// `reincarnationCheck` passes — is then wiped by [`apply_ascension_layer`]'s
/// `resetResearches`, faithful to the legacy award-then-`obtainium = 0`
/// ordering (Reset.ts:568 / 649). The pre-reset `reincarnationCheck` /
/// obtainium are captured here exactly as [`perform_reincarnation_reset`] does.
///
/// `points_gained` is `0` at current state: the legacy `ascensionCount`
/// award sits inside the `challengecompletions[10] > 0` gate alongside the
/// cube awards (Reset.ts:687-694), which this slice neutral-defaults. When
/// that block ports, the gained count populates the event.
pub(crate) fn perform_ascension_reset(
    state: &mut GameState,
    gains: &ResetCurrencyResult,
) -> SmallVec<[CoreEvent; 2]> {
    let reincarnation_check = state.reset_counters.transcend_shards
        >= Decimal::from_finite(REINCARNATION_COUNT_THRESHOLD);
    let obtainium_to_gain = if reincarnation_check {
        let base = super::compute_base_obtainium(state);
        let time_mult = super::compute_obtainium_time_multiplier(state);
        super::compute_obtainium(state, base, gains.reincarnation_point_gain, time_mult)
    } else {
        Decimal::zero()
    };
    apply_base_reset(state, gains.prestige_point_gain);
    apply_transcension_layer(state, gains.transcend_point_gain);
    apply_reincarnation_layer(
        state,
        gains.reincarnation_point_gain,
        reincarnation_check,
        obtainium_to_gain,
        // Deflation quirk excluded for ascension inputs (Reset.ts:549 guard).
        false,
    );
    let wow_cubes_gained = apply_ascension_layer(state);
    smallvec![CoreEvent::ResetPerformed {
        tier: AutoResetTier::Ascension,
        points_gained: wow_cubes_gained,
    }]
}

/// `reset('ascensionChallenge')` — identical ascension sub-resets to
/// [`perform_ascension_reset`] but fired on entering or leaving an ascension
/// challenge. The TS code runs the exact same cascade (Reset.ts:628 block
/// applies for both `'ascension'` and `'ascensionChallenge'`); the only
/// difference is that c10-gated cube/count awards are neutral-defaulted in
/// both paths at current state.
///
/// Emits [`CoreEvent::ResetPerformed`] with `tier = Ascension` (same reset
/// tier; the challenge context is already captured in
/// `current_ascension_challenge` before this runs).
pub(crate) fn perform_ascension_challenge_reset(
    state: &mut GameState,
    gains: &ResetCurrencyResult,
) -> SmallVec<[CoreEvent; 2]> {
    let reincarnation_check = state.reset_counters.transcend_shards
        >= Decimal::from_finite(REINCARNATION_COUNT_THRESHOLD);
    let obtainium_to_gain = if reincarnation_check {
        let base = super::compute_base_obtainium(state);
        let time_mult = super::compute_obtainium_time_multiplier(state);
        super::compute_obtainium(state, base, gains.reincarnation_point_gain, time_mult)
    } else {
        Decimal::zero()
    };
    apply_base_reset(state, gains.prestige_point_gain);
    apply_transcension_layer(state, gains.transcend_point_gain);
    apply_reincarnation_layer(
        state,
        gains.reincarnation_point_gain,
        reincarnation_check,
        obtainium_to_gain,
        false,
    );
    let wow_cubes_gained = apply_ascension_layer(state);
    smallvec![CoreEvent::ResetPerformed {
        tier: AutoResetTier::Ascension,
        points_gained: wow_cubes_gained,
    }]
}

/// The ascension block (Reset.ts:628-794) — no event. Assumes the base,
/// transcension, and reincarnation layers already ran. Ports the
/// **structural** reset: challenge state, the particle-producer economy, the
/// reincarnation-tier currencies, ascension counters, and the corruption
/// loadout swap. Faithful at current state, where every deferred input is
/// `0` / `false`.
///
/// Deferred (each documented inline at its site, neutral-default = no-op at
/// default):
/// - the `challengecompletions[10] > 0` reward block (`ascensionCount` +=
///   `calculateAscensionCount`, the `wow*` cube awards via
///   `CalcCorruptionStuff`) — gated off at default;
/// - the `autoChallengeIndex` / `roombaResearchIndex` / `autoResearch` UI
///   cursors (no logic field / UI-tier);
/// - the C15 corruption override (needs the `c15Corruptions` constant) and
///   every `highestSingularityCount`-gated convenience (campaigns, hepteract
///   auto-craft, tesseract auto-buyer, auto-open) — inert at default.
fn apply_ascension_layer(state: &mut GameState) -> Decimal {
    use crate::mechanics::calculate::{calc_corruption_stuff, CalcCorruptionStuffInput};
    use crate::mechanics::challenge_15_rewards;

    // c10 cube reward (Reset.ts:419-420): CalcCorruptionStuff + ascensionCount
    // must read PRE-reset state — the cube/tesseract/hyper/platonic/hepteract
    // multipliers depend on researches (incl. [192] Mortuus), ant upgrades, and
    // the finiteDescent rune, all of which the destroy-lists below zero. Capture
    // the award here and apply it after the resets. Inert at
    // challengecompletions[10] == 0 (default).
    let ascension_award = if state.challenges.challenge_completions[10] > 0.0 {
        let score = super::compute_ascension_score_result(state);
        let effective_score = score.effective_score;
        let all_cube = super::compute_all_cube_multiplier(state);
        let count_gain = super::compute_ascension_count(state, effective_score);
        let rewards = calc_corruption_stuff(&CalcCorruptionStuffInput {
            scores: score,
            cube_multiplier: super::compute_cube_multiplier(state, effective_score, all_cube),
            tesseract_multiplier: super::compute_tesseract_multiplier(
                state,
                effective_score,
                all_cube,
            ),
            hypercube_multiplier: super::compute_hypercube_multiplier(
                state,
                effective_score,
                all_cube,
            ),
            platonic_multiplier: super::compute_platonic_multiplier(
                state,
                effective_score,
                all_cube,
            ),
            hepteract_multiplier: super::compute_hepteract_multiplier(
                state,
                effective_score,
                all_cube,
            ),
            hepteracts_unlocked: challenge_15_rewards::hepteracts_unlocked(
                state.challenges.challenge15_exponent,
            ),
            singularity_count: state.singularity.singularity_count,
        });
        Some((count_gain, rewards))
    } else {
        None
    };

    // Clear the lower auto-challenge gates (Reset.ts:633-634). The ascension
    // challenge gate itself is intentionally left untouched.
    state.challenges.current_transcension_challenge = 0;
    state.challenges.current_reincarnation_challenge = 0;

    // resetChallengeSweep() (Reset.ts:636) — return the sweep machine to
    // Idle. The legacy `OFF` text toggle is UI-tier.
    state.automation.sweep_state = SweepState::Idle;
    state.automation.sweep_time_since_last_change = 0.0;

    // resetResearches() (Reset.ts:649) — zero obtainium (this also wipes the
    // award the reincarnation layer just made) and the destroy-list research
    // slots. The pre-sing-25 extras are included while
    // `highestSingularityCount < 25` (true at default).
    state.researches.obtainium = Decimal::zero();
    for &slot in RESET_RESEARCHES {
        state.researches.researches[slot] = 0.0;
    }
    if state.singularity.highest_singularity_count < RESET_RESEARCHES_SING_THRESHOLD {
        for &slot in RESET_RESEARCHES_PRE_SING25 {
            state.researches.researches[slot] = 0.0;
        }
    }

    // resetAnts(AntSacrificeTiers.ascension) (Reset.ts:650).
    reset_ants_ascension(&mut state.ants);

    // resetTalismanData('ascension') (Reset.ts:651).
    reset_talismans_ascension(&mut state.talismans);

    // Reincarnation-tier currencies (Reset.ts:652-653).
    state.upgrades.reincarnation_points = Decimal::zero();
    state.reset_counters.reincarnation_shards = Decimal::zero();

    // Upgrade slots the ascension wipes (Reset.ts:655-660).
    reset_upgrade_slots(state, 61..=80);
    reset_upgrade_slots(state, 94..=100);

    // Particle producers fully reset (owned + cascade + cost), unlike the
    // reincarnation layer which only zeroes their cascade (Reset.ts:661-670).
    for (tier, cost) in state
        .particle_producers
        .tiers
        .iter_mut()
        .zip(PARTICLE_BASE_COSTS)
    {
        tier.owned = 0.0;
        tier.generated = Decimal::zero();
        tier.cost = Decimal::from_finite(cost);
    }

    state.automation.offerings = Decimal::zero(); // Reset.ts:671
    state.crystal_upgrades.crystal_upgrades = [0.0; 8]; // Reset.ts:672

    // resetRunes('ascension') (Runes.ts:917-932): the ascension-tier runes
    // reset to level + EXP 0, then regrant level = 3 * cubeUpgrades[26]. That
    // is every classic rune EXCEPT antiquities, which is singularity-tier
    // (`resetTiers.ascension=4 < singularity=5`) and so survives. Blessings /
    // spirits / free-levels are not touched (the legacy loop only zeroes
    // level + EXP). The `setRuneLevel` EXP-sync for a regranted level > 0 is
    // deferred — inert while `cubeUpgrades[26] == 0` at default.
    let rune_regrant = 3.0 * state.cube_upgrade_levels.cube_upgrades[26];
    for rune in 0..RUNE_COUNT {
        if rune == RUNE_ANTIQUITIES {
            continue;
        }
        state.runes.rune_levels[rune] = rune_regrant;
        state.runes.rune_exp[rune] = 0.0;
    }

    // cubeUpgrades[27] regrants one of each particle producer (676-682) —
    // `0` at default, so inert; ported faithfully.
    if state.cube_upgrade_levels.cube_upgrades[27] == 1.0 {
        for tier in &mut state.particle_producers.tiers {
            tier.owned = 1.0;
        }
    }

    // Apply the c10 cube award captured up front (its compute reads pre-reset
    // researches / ant upgrades / finiteDescent that the destroy-lists above
    // zero). The cube-balance writes + ascensionCount still happen here, before
    // the challenge/counter zeroing below (Reset.ts:687-694). Inert at default.
    let wow_cubes_gained = if let Some((count_gain, rewards)) = ascension_award {
        state.reset_counters.ascension_count += count_gain;
        let cap = Decimal::from_finite(1e300);
        let cb = &mut state.cube_balances;
        // `player.wow*.add()` = `min(1e300, balance + gain)`; hepteracts fold into
        // `wowAbyssals` (Reset.ts:693).
        cb.wow_cubes = (cb.wow_cubes + Decimal::from_finite(rewards.wow_cubes)).min(cap);
        cb.wow_tesseracts =
            (cb.wow_tesseracts + Decimal::from_finite(rewards.wow_tesseracts)).min(cap);
        cb.wow_hypercubes =
            (cb.wow_hypercubes + Decimal::from_finite(rewards.wow_hypercubes)).min(cap);
        cb.wow_platonic_cubes =
            (cb.wow_platonic_cubes + Decimal::from_finite(rewards.wow_platonic_cubes)).min(cap);
        cb.wow_abyssals = 1e300_f64.min(cb.wow_abyssals + rewards.wow_hepteracts);
        Decimal::from_finite(rewards.wow_cubes)
    } else {
        Decimal::zero()
    };

    // Challenge completions 1..=10 and their highs (Reset.ts:696-699).
    for completion in &mut state.challenges.challenge_completions[1..=10] {
        *completion = 0.0;
    }
    for highest in &mut state.challenges.highest_challenge_completions[1..=10] {
        *highest = 0.0;
    }

    // roombaResearchIndex / autoResearch reset (701-703): UI cursors with no
    // logic field — skipped.

    // Ascension counters (Reset.ts:733-735).
    state.reset_counters.ascension_counter = 0.0;
    state.reset_counters.ascension_counter_real = 0.0;
    state.reset_counters.ascension_counter_real_real = 0.0;

    // cubeUpgrades 4/5/6 regrant the just-cleared upgrade slots 94..=100
    // (Reset.ts:739-751) — `0` at default, inert; ported faithfully.
    if state.cube_upgrade_levels.cube_upgrades[4] == 1.0 {
        for slot in 94..=98 {
            state.upgrades.upgrades[slot] = 1;
        }
    }
    if state.cube_upgrade_levels.cube_upgrades[5] == 1.0 {
        state.upgrades.upgrades[99] = 1;
    }
    if state.cube_upgrade_levels.cube_upgrades[6] == 1.0 {
        state.upgrades.upgrades[100] = 1;
    }

    // Campaign reset (762-784) and the sing-gated conveniences block
    // (796-877): DEFERRED — `highestSingularityCount`-gated, inert at default.

    // Corruption loadout swap `used ← next` (Reset.ts:785). `CorruptionLoadout`
    // is `Copy`, so this is a plain assignment. The C15 override (788-790) is
    // deferred (needs `c15Corruptions`; inert unless inside ascension
    // challenge 15).
    state.corruptions.used = state.corruptions.next;

    wow_cubes_gained
}

/// `resetAnts(AntSacrificeTiers.ascension)`
/// (`Features/Ants/player/reset.ts`) — the ant sub-reset an ascension runs.
/// Tier ordering is `sacrifice=0 < ascension=1 < singularity=2 < never=3`;
/// each ant feature resets when `ascension >= its minimum reset tier`.
///
/// Deferred (inert at default): the `highestSingularityCount >= 10/15/20`
/// crumb / producer / upgrade regrants, and the `preserveAnthillCount`
/// achievement that would keep `ant_sacrifice_count` (no achievement is held
/// at default, so the count resets — faithful).
fn reset_ants_ascension(ants: &mut AntsState) {
    // Crumbs reset to their default `1`; `crumbs_ever_made` is never-tier and
    // survives (Crumbs/reset.ts).
    ants.crumbs = Decimal::from_finite(1.0);
    ants.crumbs_this_sacrifice = Decimal::from_finite(1.0);

    // Every producer empties; every mastery *level* resets, but
    // `highest_mastery` survives (AntProducers / AntMasteries reset.ts).
    for producer in &mut ants.producers {
        producer.purchased = 0.0;
        producer.generated = Decimal::zero();
    }
    for mastery in &mut ants.masteries {
        mastery.mastery = 0;
    }

    // Ant upgrades: every slot resets at ascension EXCEPT Mortuus2 (index 15,
    // singularity-tier). Salvage(7) / Mortuus(11) / WowCubes(13) /
    // AscensionScore(14) are ascension-tier; the rest sacrifice-tier
    // (AntUpgrades reset.ts + data table).
    for level in &mut ants.upgrades[..ANT_UPGRADE_MORTUUS2] {
        *level = 0.0;
    }

    // Reborn ELO resets; the daily / ever leaderboards and the quark total are
    // singularity-tier and survive. Immortal ELO is ascension-tier
    // (RebornELO / ImmortalELO reset.ts).
    ants.reborn_elo = 0.0;
    ants.immortal_elo = 0.0;

    // The sacrifice ID always advances (it backs the permanent leaderboard);
    // the count resets, since no `preserveAnthillCount` achievement is held at
    // default (AntSacrifice reset.ts).
    ants.current_sacrifice_id += 1;
    ants.ant_sacrifice_count = 0.0;

    // Sacrifice timers (the `resetAnts` wrapper, reset.ts).
    ants.ant_sacrifice_timer = 0.0;
    ants.ant_sacrifice_timer_real = 0.0;
}

/// `resetTalismanData('ascension')` (`Talismans.ts:1067-1086`). All seven Rust
/// talismans are ascension-tier, so each `resetSingleTalisman` runs: level → 0
/// and a rarity recompute (`setTalismanRarity`). The recompute needs the
/// UI-tier `isUnlocked()` predicate, which is `false` at reachable state
/// (talismans have no unlock / level path in the port yet) ⇒ rarity 0 —
/// faithful here; the unlocked-talisman rarity (`compute_talisman_rarity`
/// returns 1 at level 0) is deferred. The shard balance and the six fragment
/// pools zero; the rune-buff assignments (legacy `talismanOne..Seven`) are
/// **not** touched, matching `resetSingleTalisman`.
fn reset_talismans_ascension(talismans: &mut TalismansState) {
    talismans.talisman_levels = [0.0; TALISMAN_COUNT];
    talismans.talisman_rarity = [0.0; TALISMAN_COUNT];
    talismans.talisman_shards = 0.0;
    talismans.common_fragments = 0.0;
    talismans.uncommon_fragments = 0.0;
    talismans.rare_fragments = 0.0;
    talismans.epic_fragments = 0.0;
    talismans.legendary_fragments = 0.0;
    talismans.mythical_fragments = 0.0;
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

    #[test]
    fn reincarnation_reset_credits_obtainium_above_threshold() {
        let mut state = GameState::default();
        // transcendShards ≥ 1e300 ⇒ reincarnationCheck true.
        state.reset_counters.transcend_shards = Decimal::from_finite(1e305);
        assert_eq!(state.researches.obtainium.to_number(), 0.0);

        perform_reincarnation_reset(&mut state, &gains(0.0, 0.0, 9.0));

        // `calculateObtainium` has a constant Base line of 1.0, so even at
        // default the award is strictly positive.
        assert!(
            state.researches.obtainium.to_number() > 0.0,
            "expected an obtainium award, got {}",
            state.researches.obtainium.to_number()
        );
    }

    #[test]
    fn reincarnation_reset_skips_obtainium_below_threshold() {
        let mut state = GameState::default();
        // transcendShards < 1e300 ⇒ reincarnationCheck false ⇒ no award / count.
        state.reset_counters.transcend_shards = Decimal::from_finite(1e200);

        perform_reincarnation_reset(&mut state, &gains(0.0, 0.0, 9.0));

        assert_eq!(state.researches.obtainium.to_number(), 0.0);
        assert_eq!(state.reset_counters.reincarnation_count, 0.0);
    }

    #[test]
    fn reincarnation_obtainium_scales_with_reset_time() {
        // Two reincarnations identical except reset length. With
        // reincarnationCount ≥ 5 the `offeringObtainiumTimeModifiers`
        // TimeMultiplier line rewards the longer run, so it earns strictly
        // more obtainium (whereas the instant reset takes the
        // ThresholdPenalty of 0 and only the base-obtainium floor).
        let make = |counter: f64| {
            let mut state = GameState::default();
            state.reset_counters.transcend_shards = Decimal::from_finite(1e305);
            state.reset_counters.reincarnation_count = 5.0; // ≥ 5 ⇒ TimeMultiplier active
            state.reset_counters.reincarnation_counter = counter;
            perform_reincarnation_reset(&mut state, &gains(0.0, 0.0, 0.0));
            state.researches.obtainium
        };

        let quick = make(0.0); // ratio 0 ⇒ ThresholdPenalty 0 ⇒ base floor only
        let slow = make(1000.0); // well past the 10s threshold ⇒ large TimeMultiplier

        assert!(
            slow > quick,
            "a longer reincarnation should earn more obtainium: slow={} quick={}",
            slow.to_number(),
            quick.to_number()
        );
    }

    // ─── Ascension ───────────────────────────────────────────────────────

    #[test]
    fn ascension_reset_runs_full_cascade_and_resets_particle_economy() {
        let mut state = GameState::default();
        // Base-layer witness (the cascade must run apply_base_reset).
        state.coin_producers.tiers[0].owned = 10.0;
        // Particle producers fully reset by the ascension layer.
        state.particle_producers.tiers[0].owned = 50.0;
        state.particle_producers.tiers[0].cost = Decimal::from_finite(999.0);
        state.particle_producers.tiers[0].generated = Decimal::from_finite(5.0);
        // Reincarnation-tier currencies zeroed.
        state.upgrades.reincarnation_points = Decimal::from_finite(1e50);
        state.reset_counters.reincarnation_shards = Decimal::from_finite(1e40);
        state.automation.offerings = Decimal::from_finite(1e30);
        state.crystal_upgrades.crystal_upgrades = [3.0; 8];
        // Challenge completions 1..=10 wiped; the ascension-challenge slot
        // (index 11) and its high survive the `1..=10` bound.
        state.challenges.challenge_completions[3] = 12.0;
        state.challenges.challenge_completions[10] = 7.0;
        state.challenges.challenge_completions[11] = 3.0;
        state.challenges.highest_challenge_completions[5] = 8.0;
        state.challenges.highest_challenge_completions[11] = 4.0;
        // resetResearches: a destroy-list slot is wiped, a non-listed one survives.
        state.researches.researches[7] = 5.0; // in RESET_RESEARCHES
        state.researches.researches[50] = 9.0; // not listed → survives
        state.researches.obtainium = Decimal::from_finite(99.0);
        // Ascension counters.
        state.reset_counters.ascension_counter = 123.0;
        state.reset_counters.ascension_counter_real = 45.0;
        state.reset_counters.ascension_counter_real_real = 67.0;
        // Corruption loadout swap `used ← next`.
        state.corruptions.used.levels[0] = 5;
        state.corruptions.next.levels[0] = 9;

        let events = perform_ascension_reset(&mut state, &gains(0.0, 0.0, 0.0));

        // Cascade ran the base.
        assert_eq!(state.coin_counters.coins_this_prestige.to_number(), 100.0);
        assert_eq!(state.coin_producers.tiers[0].owned, 0.0);
        // Particle producers reset to base.
        assert_eq!(state.particle_producers.tiers[0].owned, 0.0);
        assert_eq!(state.particle_producers.tiers[0].cost.to_number(), 1.0);
        assert_eq!(state.particle_producers.tiers[4].cost.to_number(), 1e16);
        assert_eq!(state.particle_producers.tiers[0].generated.to_number(), 0.0);
        assert_eq!(state.upgrades.reincarnation_points.to_number(), 0.0);
        assert_eq!(state.reset_counters.reincarnation_shards.to_number(), 0.0);
        assert_eq!(state.automation.offerings.to_number(), 0.0);
        assert_eq!(state.crystal_upgrades.crystal_upgrades, [0.0; 8]);
        assert_eq!(state.challenges.challenge_completions[3], 0.0);
        assert_eq!(state.challenges.challenge_completions[10], 0.0);
        assert_eq!(state.challenges.challenge_completions[11], 3.0); // survives
        assert_eq!(state.challenges.highest_challenge_completions[5], 0.0);
        assert_eq!(state.challenges.highest_challenge_completions[11], 4.0); // survives
        assert_eq!(state.researches.researches[7], 0.0);
        assert_eq!(state.researches.researches[50], 9.0); // survives
        assert_eq!(state.researches.obtainium.to_number(), 0.0);
        assert_eq!(state.reset_counters.ascension_counter, 0.0);
        assert_eq!(state.reset_counters.ascension_counter_real, 0.0);
        assert_eq!(state.reset_counters.ascension_counter_real_real, 0.0);
        assert_eq!(state.corruptions.used.levels[0], 9); // swapped from next

        assert_eq!(events.len(), 1);
        // c10 = 7 > 0, so the award block fired: ascensionCount gained
        // `calculateAscensionCount()` (= 1 at default mults) and `points_gained`
        // reports the credited wow-cube gain (added to a fresh 0 balance).
        assert_eq!(state.reset_counters.ascension_count, 1.0);
        assert!(matches!(
            events[0],
            CoreEvent::ResetPerformed {
                tier: AutoResetTier::Ascension,
                points_gained,
            } if points_gained == state.cube_balances.wow_cubes
        ));
    }

    #[test]
    fn ascension_reset_credits_cube_award_at_c10() {
        let mut state = GameState::default();
        // c10 cleared, with strong challenge history and a long ascension so the
        // AscensionTime cube line saturates to 1 (frac ≫ 1).
        state.challenges.challenge_completions[10] = 10.0;
        for i in 1..=10 {
            state.challenges.highest_challenge_completions[i] = 100.0;
            state.challenges.challenge_completions[i] = 50.0;
        }
        state.reset_counters.ascension_counter = 1e12;

        let events = perform_ascension_reset(&mut state, &gains(0.0, 0.0, 0.0));

        // The c10 award credited wow-cubes (CalcCorruptionStuff) and the count.
        assert!(state.cube_balances.wow_cubes.to_number() > 0.0);
        assert!(state.reset_counters.ascension_count >= 1.0);
        assert!(matches!(
            events[0],
            CoreEvent::ResetPerformed { points_gained, .. }
                if points_gained.to_number() > 0.0
        ));
    }

    #[test]
    fn ascension_cube_award_reads_pre_reset_research_and_ants() {
        // Audit P1.3: the c10 cube award must be computed from PRE-reset state.
        // research[192] and the Mortuus ant upgrade (11) both feed the cube
        // multiplier and are both zeroed by the ascension reset's destroy-lists;
        // computing the award after the wipes (the bug) dropped their boost.
        let base = || {
            let mut s = GameState::default();
            s.challenges.challenge_completions[10] = 10.0;
            for i in 1..=10 {
                s.challenges.highest_challenge_completions[i] = 100.0;
                s.challenges.challenge_completions[i] = 50.0;
            }
            s.reset_counters.ascension_counter = 1e12;
            s
        };

        let mut without = base();
        let _ = perform_ascension_reset(&mut without, &gains(0.0, 0.0, 0.0));

        let mut with = base();
        with.researches.researches[192] = 1000.0;
        with.ants.upgrades[11] = 100.0; // Mortuus
        let _ = perform_ascension_reset(&mut with, &gains(0.0, 0.0, 0.0));

        assert!(
            with.cube_balances.wow_cubes.to_number() > without.cube_balances.wow_cubes.to_number(),
            "pre-reset research[192]/Mortuus must increase the cube award (lost when computed post-reset)"
        );
    }

    #[test]
    fn prestige_count_uses_ecc_multiplier() {
        use crate::mechanics::challenges::ChallengeType;
        // Audit P1.6: count increments are floor(multiplier) where multiplier rises
        // with the gating challenge's CalcECC, not a flat +1. Identity at default.
        assert_eq!(
            reset_count_increment(ChallengeType::Transcend, 0.0, 0.05),
            1.0
        );

        // c5 = 1000 → CalcECC(Transcend) = 145 → floor(1 + 0.05*145) = 8 (not 1).
        let expected_inc = reset_count_increment(ChallengeType::Transcend, 1000.0, 0.05);
        assert_eq!(expected_inc, 8.0);

        // End-to-end: a prestige reset increments by exactly that (was flat +1).
        let mut state = GameState::default();
        state.challenges.challenge_completions[5] = 1000.0;
        let before = state.reset_counters.prestige_count;
        let _ = perform_prestige_reset(&mut state, Decimal::zero());
        assert_eq!(state.reset_counters.prestige_count, before + expected_inc);
    }

    #[test]
    fn ascension_reset_neutral_defaults_rewards_at_c10_zero() {
        let mut state = GameState::default();
        // c10 == 0 (default) ⇒ the reward block is a no-op: neither the cube
        // balances nor the ascension count change.
        state.cube_balances.wow_cubes = Decimal::from_finite(1e20);
        state.reset_counters.ascension_count = 5.0;
        assert_eq!(state.challenges.challenge_completions[10], 0.0);

        perform_ascension_reset(&mut state, &gains(0.0, 0.0, 0.0));

        assert_eq!(state.cube_balances.wow_cubes.to_number(), 1e20);
        assert_eq!(state.reset_counters.ascension_count, 5.0);
    }

    #[test]
    fn ascension_reset_zeroes_obtainium_even_when_reincarnation_layer_awards_it() {
        let mut state = GameState::default();
        // transcendShards ≥ 1e300 ⇒ the cascaded reincarnation layer awards
        // obtainium; the ascension layer's resetResearches must then wipe it
        // (faithful to the legacy award-then-`obtainium = 0` ordering).
        state.reset_counters.transcend_shards = Decimal::from_finite(1e305);

        perform_ascension_reset(&mut state, &gains(0.0, 0.0, 9.0));

        assert_eq!(state.researches.obtainium.to_number(), 0.0);
    }

    #[test]
    fn ascension_reset_returns_challenge_sweep_to_idle() {
        let mut state = GameState::default();
        state.automation.sweep_state = SweepState::InitialWait;
        state.automation.sweep_time_since_last_change = 5.0;

        perform_ascension_reset(&mut state, &gains(0.0, 0.0, 0.0));

        assert_eq!(state.automation.sweep_state, SweepState::Idle);
        assert_eq!(state.automation.sweep_time_since_last_change, 0.0);
    }

    #[test]
    fn ascension_reset_swaps_corruption_loadout_used_from_next() {
        let mut state = GameState::default();
        state.corruptions.used.levels[2] = 1;
        state.corruptions.next.levels[2] = 7;
        state.corruptions.next.total_corruption_ascension_multiplier = 2.5;

        perform_ascension_reset(&mut state, &gains(0.0, 0.0, 0.0));

        assert_eq!(state.corruptions.used.levels[2], 7);
        assert_eq!(
            state.corruptions.used.total_corruption_ascension_multiplier,
            2.5
        );
    }

    #[test]
    fn ascension_reset_applies_cube_upgrade_regrants() {
        let mut state = GameState::default();
        // cubeUpgrades[27] → one of each particle producer after the reset.
        state.cube_upgrade_levels.cube_upgrades[27] = 1.0;
        // cubeUpgrades[4/5/6] → regrant upgrade slots 94..=98 / 99 / 100.
        state.cube_upgrade_levels.cube_upgrades[4] = 1.0;
        state.cube_upgrade_levels.cube_upgrades[5] = 1.0;
        state.cube_upgrade_levels.cube_upgrades[6] = 1.0;

        perform_ascension_reset(&mut state, &gains(0.0, 0.0, 0.0));

        for tier in &state.particle_producers.tiers {
            assert_eq!(tier.owned, 1.0);
        }
        assert_eq!(state.upgrades.upgrades[94], 1);
        assert_eq!(state.upgrades.upgrades[98], 1);
        assert_eq!(state.upgrades.upgrades[99], 1);
        assert_eq!(state.upgrades.upgrades[100], 1);
    }

    #[test]
    fn ascension_skips_reincarnation_only_deflation_quirk() {
        // deflation > 10 && platonicUpgrades[11] > 0 makes the reincarnation
        // layer add the reincarnation gain to prestige points — but only for a
        // reincarnation input. An ascension cascading through the layer must
        // NOT apply it.
        let seed = || {
            let mut state = GameState::default();
            state.corruptions.used.levels[DEFLATION_INDEX] = 11;
            state.cube_upgrade_levels.platonic_upgrades[11] = 1.0;
            state
        };

        let mut asc = seed();
        perform_ascension_reset(&mut asc, &gains(0.0, 0.0, 7.0));
        assert_eq!(asc.upgrades.prestige_points.to_number(), 0.0);

        let mut rei = seed();
        perform_reincarnation_reset(&mut rei, &gains(0.0, 0.0, 7.0));
        assert_eq!(rei.upgrades.prestige_points.to_number(), 7.0);
    }

    #[test]
    fn perform_reset_dispatches_ascension() {
        let mut state = GameState::default();
        let events = perform_reset(&mut state, ResetRequest::Ascension, &gains(0.0, 0.0, 0.0));
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            CoreEvent::ResetPerformed {
                tier: AutoResetTier::Ascension,
                ..
            }
        ));
    }

    #[test]
    fn ascension_reset_wipes_ascension_tier_runes_but_keeps_antiquities() {
        let mut state = GameState::default();
        for i in 0..RUNE_COUNT {
            state.runes.rune_levels[i] = 100.0;
            state.runes.rune_exp[i] = 500.0;
            state.runes.rune_blessing_levels[i] = 7.0;
        }

        perform_ascension_reset(&mut state, &gains(0.0, 0.0, 0.0));

        for i in 0..RUNE_COUNT {
            if i == RUNE_ANTIQUITIES {
                // Singularity-tier ⇒ survives an ascension.
                assert_eq!(state.runes.rune_levels[i], 100.0);
                assert_eq!(state.runes.rune_exp[i], 500.0);
            } else {
                assert_eq!(state.runes.rune_levels[i], 0.0, "rune {i} level");
                assert_eq!(state.runes.rune_exp[i], 0.0, "rune {i} exp");
            }
            // Blessings are outside the rune reset's scope.
            assert_eq!(
                state.runes.rune_blessing_levels[i], 7.0,
                "rune {i} blessing"
            );
        }
    }

    #[test]
    fn ascension_reset_rune_regrant_scales_with_cube_upgrade_26() {
        let mut state = GameState::default();
        state.cube_upgrade_levels.cube_upgrades[26] = 2.0; // regrant = 3 * 2 = 6
        for i in 0..RUNE_COUNT {
            state.runes.rune_levels[i] = 100.0;
        }

        perform_ascension_reset(&mut state, &gains(0.0, 0.0, 0.0));

        // Ascension-tier runes (index 0 = speed) regrant to 6; EXP stays 0.
        assert_eq!(state.runes.rune_levels[0], 6.0);
        assert_eq!(state.runes.rune_exp[0], 0.0);
        // Antiquities (singularity-tier) is untouched by the regrant.
        assert_eq!(state.runes.rune_levels[RUNE_ANTIQUITIES], 100.0);
    }

    #[test]
    fn ascension_reset_wipes_ant_state_keeping_singularity_tier() {
        let mut state = GameState::default();
        for p in &mut state.ants.producers {
            p.purchased = 50.0;
            p.generated = Decimal::from_finite(9.0);
        }
        for m in &mut state.ants.masteries {
            m.mastery = 8;
            m.highest_mastery = 12; // survives (highest mastery never resets here)
        }
        for u in &mut state.ants.upgrades {
            *u = 5.0;
        }
        state.ants.crumbs = Decimal::from_finite(1e30);
        state.ants.crumbs_this_sacrifice = Decimal::from_finite(1e20);
        state.ants.crumbs_ever_made = Decimal::from_finite(999.0); // never-tier, survives
        state.ants.reborn_elo = 7777.0;
        state.ants.immortal_elo = 8888.0;
        state.ants.quarks_gained_from_ants = 42.0; // singularity-tier, survives
        state
            .ants
            .highest_reborn_elo_daily
            .push(crate::state::RebornELOEntry {
                elo: 100.0,
                sacrifice_id: 1,
            }); // singularity-tier, survives
        state.ants.ant_sacrifice_count = 33.0;
        state.ants.current_sacrifice_id = 5;
        state.ants.ant_sacrifice_timer = 4.0;
        state.ants.ant_sacrifice_timer_real = 6.0;

        perform_ascension_reset(&mut state, &gains(0.0, 0.0, 0.0));

        // Crumbs back to default 1; ever-made survives.
        assert_eq!(state.ants.crumbs.to_number(), 1.0);
        assert_eq!(state.ants.crumbs_this_sacrifice.to_number(), 1.0);
        assert_eq!(state.ants.crumbs_ever_made.to_number(), 999.0);
        // Producers empty; mastery levels reset, highest survives.
        for p in &state.ants.producers {
            assert_eq!(p.purchased, 0.0);
            assert_eq!(p.generated.to_number(), 0.0);
        }
        for m in &state.ants.masteries {
            assert_eq!(m.mastery, 0);
            assert_eq!(m.highest_mastery, 12);
        }
        // Upgrades 0..=14 cleared; Mortuus2 (15, singularity) survives.
        for i in 0..ANT_UPGRADE_MORTUUS2 {
            assert_eq!(state.ants.upgrades[i], 0.0, "upgrade {i}");
        }
        assert_eq!(state.ants.upgrades[ANT_UPGRADE_MORTUUS2], 5.0);
        // Reborn + immortal ELO reset; quark total and daily board survive.
        assert_eq!(state.ants.reborn_elo, 0.0);
        assert_eq!(state.ants.immortal_elo, 0.0);
        assert_eq!(state.ants.quarks_gained_from_ants, 42.0);
        assert_eq!(state.ants.highest_reborn_elo_daily.len(), 1);
        // Sacrifice count reset; the ID advances; timers cleared.
        assert_eq!(state.ants.ant_sacrifice_count, 0.0);
        assert_eq!(state.ants.current_sacrifice_id, 6);
        assert_eq!(state.ants.ant_sacrifice_timer, 0.0);
        assert_eq!(state.ants.ant_sacrifice_timer_real, 0.0);
    }

    #[test]
    fn ascension_reset_wipes_talisman_levels_and_fragments() {
        let mut state = GameState::default();
        state.talismans.talisman_levels = [50.0; TALISMAN_COUNT];
        state.talismans.talisman_rarity = [4.0; TALISMAN_COUNT];
        state.talismans.talisman_shards = 1e9;
        state.talismans.common_fragments = 100.0;
        state.talismans.mythical_fragments = 7.0;
        // Rune-buff assignments survive — resetSingleTalisman leaves them.
        state.talismans.rune_assignments[0][0].allocated = true;
        state.talismans.rune_assignments[0][0].rune_id = 3;

        perform_ascension_reset(&mut state, &gains(0.0, 0.0, 0.0));

        assert_eq!(state.talismans.talisman_levels, [0.0; TALISMAN_COUNT]);
        assert_eq!(state.talismans.talisman_rarity, [0.0; TALISMAN_COUNT]);
        assert_eq!(state.talismans.talisman_shards, 0.0);
        assert_eq!(state.talismans.common_fragments, 0.0);
        assert_eq!(state.talismans.mythical_fragments, 0.0);
        // Rune-buff assignments untouched.
        assert!(state.talismans.rune_assignments[0][0].allocated);
        assert_eq!(state.talismans.rune_assignments[0][0].rune_id, 3);
    }
}
