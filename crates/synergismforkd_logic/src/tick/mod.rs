//! Tick orchestrator — `tack` plus named phase functions.
//!
//! This file is the canonical statement of tick order. Phases run in the
//! sequence declared in [`tack`]; reordering requires editing this file.
//! Per the [[loom-tack-design]] memo, named phases prevent implicit
//! call-order shifts from silently changing player-visible per-second
//! rates.
//!
//! ## Phases
//! 1. **Cross-mechanic precompute** — stubbed; `*Pre` bundles still
//!    caller-provided. Becomes a single `CrossMechanicCache` once the
//!    upstream mechanics (rune effects, ant effects, hepteract effects,
//!    achievement rewards, challenge-15 rewards) finish porting.
//! 2. **Global state aggregators** — the four pure aggregators
//!    ([`compute_global_multipliers`], [`update_all_multiplier`],
//!    [`update_all_tick`], plus the helpers reading their outputs). Their
//!    results currently live as locals; they will move into a
//!    `state.g_cache` slice once that slice is added.
//! 3. **Player input** — drains [`TackInput::player_actions`] into
//!    `buy_*` mutators. Runs after Phase 2 so purchases spend against
//!    fresh costs.
//! 4. **Resource generation** — calls [`resource_gain`] and writes its
//!    result back into the corresponding [`GameState`] slices.
//! 5. **Automation** — stubbed; head/middle/tail content (timers,
//!    auto-research, rune/ant sacrifice, addObtainium/Offerings, challenge
//!    sweep, auto-reset) lands as those mechanics port.
//!
//! Boundary: this module produces a flat [`TickOutput`] event stream.
//! Modal dispatch, audio cues, save serialization, and i18n live in the
//! UI tier and consume `output.events`.

use smallvec::SmallVec;

use synergismforkd_bignum::Decimal;

use crate::events::{CoreEvent, ProducerType};
use crate::mechanics::accelerators::{buy_accelerator, BuyAcceleratorInput};
use crate::mechanics::achievement_rewards;
use crate::mechanics::challenge_15_rewards;
use crate::mechanics::crystal_upgrades::{buy_crystal_upgrades, BuyCrystalUpgradesInput};
use crate::mechanics::global_multipliers::{
    compute_global_multipliers, GlobalMultipliersPreEvaluated, GlobalMultipliersResult,
};
use crate::mechanics::multipliers::{buy_multiplier, BuyMultiplierInput};
use crate::mechanics::particle_buildings::{buy_particle_building, BuyParticleBuildingInput};
use crate::mechanics::producers::{buy_max, buy_producer, BuyMaxInput, BuyProducerInput};
use crate::mechanics::resource_gain::{resource_gain, ResourceGainPre};
use crate::mechanics::tesseract_buildings::{buy_tesseract_building, BuyTesseractBuildingInput};
use crate::mechanics::update_all_multiplier::{
    update_all_multiplier, UpdateAllMultiplierPre, UpdateAllMultiplierResult,
};
use crate::mechanics::update_all_tick::{update_all_tick, UpdateAllTickPre, UpdateAllTickResult};
use crate::mechanics::upgrades::{buy_upgrades, BuyUpgradeInput};
use crate::state::{GameState, RngPurpose};

mod ant_generation;
mod auto_research;
mod auto_reset;
mod automatic_tools;
mod challenge_sweep;
mod timers;

/// Caller-provided pre-evaluated inputs for the Phase 5 automation
/// layer — speed multipliers and unlock gates that have no ported
/// aggregator yet (`calculateGlobalSpeedMult`,
/// `calculateAscensionSpeedMult`, `calculateSingularitySpeedMult`,
/// `quark_handler`, `exportGQPerHour`, …).
///
/// Mirrors the four existing `*_pre` bundles: caller-packed for the
/// duration of the MVP port, migrating to compute-from-state as each
/// upstream aggregator lands. **Grows per chunk** — each automation
/// sub-phase adds the fields it consumes. Today it carries the head-
/// timer multipliers + gates consumed by the simple counter and
/// octeract timers (Chunks 1–2).
#[derive(Debug, Clone, Copy)]
pub struct AutomationPre {
    /// `calculateGlobalSpeedMult()` — scales the prestige / transcend /
    /// reincarnation reset counters.
    pub global_time_multiplier: f64,
    /// `calculateAscensionSpeedMult()` — scales `ascension_counter`.
    pub ascension_speed_multi: f64,
    /// `calculateSingularitySpeedMult()` — scales `singularity_counter`
    /// and `sing_challenge_timer`.
    pub singularity_speed_multi: f64,
    /// `quark_handler(...).max_time` — clamp ceiling for the quark-
    /// export timer. State-derivable; supplied here until Chunk 1 wires
    /// `quark_handler`.
    pub max_quark_timer: f64,
    /// `exportGQPerHour` — golden-quark export rate; `0.0` disables the
    /// golden-quark timer (the legacy `exportGQPerHour === 0` gate).
    pub export_gq_per_hour: f64,
    /// `octeractUnlock.unlocked` — gates the octeract timer.
    pub octeract_unlocked: bool,
    /// `calculateOcteractMultiplier()` — per-second octeract reward.
    pub octeract_per_second: f64,
    /// Product of the golden-quark multiplier stats except the
    /// qts-dependent base — used by the octeract GQ-giveaway loop.
    pub golden_quarks_multiplier_excluding_base: f64,
    /// `octeractAutoPotionSpeed.autoPotionSpeedMult` — auto-potion
    /// threshold speed.
    pub auto_potion_speed_mult: f64,
    /// `player.shopUpgrades.offeringPotion` — fast-mode gate for the
    /// offering auto-potion (caller reads the shop slot).
    pub offering_potion_count: f64,
    /// `player.shopUpgrades.obtainiumPotion` — fast-mode gate for the
    /// obtainium auto-potion.
    pub obtainium_potion_count: f64,
    /// `calculateAmbrosiaGenerationSpeed()` — `0` disables the ambrosia timer.
    pub ambrosia_generation_speed: f64,
    /// `calculateAmbrosiaLuck()`.
    pub ambrosia_luck: f64,
    /// `noAmbrosiaUpgrades.bonusAmbrosia`.
    pub bonus_ambrosia: f64,
    /// `G.TIME_PER_AMBROSIA` base constant.
    pub time_per_ambrosia: f64,
    /// `shopAmbrosiaAccelerator.ambrosiaPointRequirementMult`.
    pub ambrosia_accelerator_mult: f64,
    /// `ambrosiaBrickOfLead.barRequirementMult`.
    pub ambrosia_brick_of_lead_mult: f64,
    /// `calculateRedAmbrosiaGenerationSpeed()` — `0` disables the red timer.
    pub red_ambrosia_generation_speed: f64,
    /// `calculateRedAmbrosiaLuck()`.
    pub red_ambrosia_luck: f64,
    /// `redAmbrosiaAccelerator.ambrosiaTimePerRedAmbrosia` — bonus
    /// blueberry time minted per red ambrosia (fed back into ambrosia).
    pub ambrosia_time_per_red_ambrosia: f64,
    /// `G.TIME_PER_RED_AMBROSIA` base constant.
    pub time_per_red_ambrosia: f64,
    /// `limitedTime.barRequirementMultiplier`.
    pub red_ambrosia_bar_requirement_multiplier: f64,
    /// `offeringAuto.autoRune` shop effect — combined with the persisted
    /// `rune_sacrifice_auto_enabled` toggle to gate rune auto-sacrifice.
    pub offering_auto_rune: bool,
    /// `getAchievementReward('antSacrificeUnlock')` — gates ant sacrifice.
    pub ant_sacrifice_unlocked: bool,
    /// `calculateAvailableRebornELO()` — drives the "maxed reborn ELO"
    /// derivation used by the ant-sacrifice toggles.
    pub available_reborn_elo: f64,
    /// `antSacrificeRewards().immortalELO` — the `ImmortalELOGain` mode's
    /// projected gain.
    pub immortal_elo_gain: f64,
    /// `calculateResearchAutomaticObtainium(dt)` — per-tick auto-obtainium
    /// gain (before the taxman clamp).
    pub obtainium_gain: Decimal,
    /// `roombaResearchEnabled()` — Roomba auto-research unlock.
    pub roomba_unlocked: bool,
    /// `getLevelMilestone('autoPrestige')` — `== 1` unlocks auto-prestige.
    pub auto_prestige_milestone: f64,
    /// `G.prestigePointGain` (from `reset_currency`) — amount-mode candidate.
    pub prestige_point_gain: Decimal,
    /// `G.transcendPointGain`.
    pub transcend_point_gain: Decimal,
    /// `G.reincarnationPointGain`.
    pub reincarnation_point_gain: Decimal,
    /// `calculateActualAntSpeedMult()` — outer ant-generation multiplier.
    pub ant_speed_mult: Decimal,
    /// Challenge-sweep `initial_wait → active` threshold.
    pub sweep_timer_start: f64,
    /// Challenge-sweep `active → next-stage` threshold.
    pub sweep_timer_exit: f64,
    /// Challenge-sweep `enter_wait → active` threshold.
    pub sweep_timer_enter: f64,
    /// `getNextRegularChallenge(initialIndex, {})` — `-1` = all maxed.
    pub sweep_next_regular_challenge_from_initial: i32,
    /// `getNextRegularChallenge(active.index, explored)` — `-1` = exhausted.
    pub sweep_next_regular_challenge_from_active: i32,
    /// Pre-evaluated `challenge15AutoExponentCheck()`.
    pub sweep_challenge_15_auto_exponent_check: bool,
    /// Pre-evaluated `finished` revalidation guard (c1 + c6 still maxed).
    pub sweep_is_finished_still_valid: bool,
}

impl Default for AutomationPre {
    /// Identity values — multipliers are `1`, the GQ-export gate is off,
    /// and the quark timer clamps at the legacy base ceiling.
    fn default() -> Self {
        Self {
            global_time_multiplier: 1.0,
            ascension_speed_multi: 1.0,
            singularity_speed_multi: 1.0,
            max_quark_timer: 90_000.0,
            export_gq_per_hour: 0.0,
            octeract_unlocked: false,
            octeract_per_second: 0.0,
            golden_quarks_multiplier_excluding_base: 1.0,
            auto_potion_speed_mult: 1.0,
            offering_potion_count: 0.0,
            obtainium_potion_count: 0.0,
            ambrosia_generation_speed: 0.0,
            ambrosia_luck: 0.0,
            bonus_ambrosia: 0.0,
            time_per_ambrosia: 45.0,
            ambrosia_accelerator_mult: 1.0,
            ambrosia_brick_of_lead_mult: 1.0,
            red_ambrosia_generation_speed: 0.0,
            red_ambrosia_luck: 0.0,
            ambrosia_time_per_red_ambrosia: 0.0,
            time_per_red_ambrosia: 100_000.0,
            red_ambrosia_bar_requirement_multiplier: 1.0,
            offering_auto_rune: false,
            ant_sacrifice_unlocked: false,
            available_reborn_elo: 0.0,
            immortal_elo_gain: 0.0,
            obtainium_gain: Decimal::zero(),
            roomba_unlocked: false,
            auto_prestige_milestone: 0.0,
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
            ant_speed_mult: Decimal::one(),
            sweep_timer_start: 0.0,
            sweep_timer_exit: 0.0,
            sweep_timer_enter: 0.0,
            sweep_next_regular_challenge_from_initial: -1,
            sweep_next_regular_challenge_from_active: -1,
            sweep_challenge_15_auto_exponent_check: false,
            sweep_is_finished_still_valid: true,
        }
    }
}

/// Inputs to [`tack`]. Owned by the caller — `logic` has no clock, no
/// input device, no RNG seed source of its own.
///
/// `automation_pre` is the last caller-provided `*_pre` bundle; the four
/// global-state bundles (`global_multipliers`, `update_all_multiplier`,
/// `update_all_tick`, `resource_gain`) have all been retired — the tick
/// now self-derives them from `&GameState`. `automation_pre` follows once
/// the Phase-5 speed-mult aggregators port.
#[derive(Debug, Clone, Default)]
pub struct TackInput {
    /// Wall-clock seconds since the previous tick. The caller is the
    /// only source of time; never read `SystemTime` from `logic`.
    pub dt: f64,
    /// `G.timeWarp` equivalent — skip Phase 5 (automation) during
    /// offline-catchup ticks. Phase 4 (generation) still runs so coins
    /// accumulate.
    pub time_warp: bool,
    /// Player inputs queued since the previous tick. Drained FIFO in
    /// Phase 3. Empty in pure background ticks.
    pub player_actions: SmallVec<[PlayerAction; 4]>,
    /// Pre-evaluated inputs for the Phase 5 automation layer.
    pub automation_pre: AutomationPre,
}

/// A single queued player input. Variants will expand as automation
/// toggles and resets port (`ToggleAuto(AutoTool)`, `Reset(ResetRequest)`
/// per the [[loom-tack-design]] memo).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PlayerAction {
    /// A buy click. Routes to one of the eight `buy_*` mutators based on
    /// the [`BuyRequest`] variant.
    Buy(BuyRequest),
}

/// Per-mechanic dispatcher for the eight `buy_*` purchase loops. The
/// variant carries the same `*Input` the underlying buy function takes.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum BuyRequest {
    /// Routes to [`buy_upgrades`].
    Upgrade(BuyUpgradeInput),
    /// Routes to [`buy_multiplier`].
    Multiplier(BuyMultiplierInput),
    /// Routes to [`buy_accelerator`].
    Accelerator(BuyAcceleratorInput),
    /// Routes to [`buy_crystal_upgrades`].
    CrystalUpgrade(BuyCrystalUpgradesInput),
    /// Routes to [`buy_particle_building`].
    ParticleBuilding(BuyParticleBuildingInput),
    /// Routes to [`buy_tesseract_building`].
    TesseractBuilding(BuyTesseractBuildingInput),
    /// Routes to [`buy_max`] — buy-as-many-as-affordable across the
    /// producer family selected by `input.producer_type`.
    ProducerMax(BuyMaxInput),
    /// Routes to [`buy_producer`] — manual-click loop across the producer
    /// family selected by `input.producer_type`.
    Producer(BuyProducerInput),
}

/// Result of [`tack`]. The accumulated event stream is the only output
/// the UI tier reads from a tick today; derived stats and dirty flags
/// land here once Phase 2 acquires a `state.g_cache` slice to read from.
#[derive(Debug, Clone, Default)]
pub struct TickOutput {
    /// CoreEvent stream for the UI tier to dispatch. Inline capacity of
    /// 16 covers the typical worst-case tick (purchases × N + 1
    /// achievement + up to 5 challenge auto-completions).
    pub events: SmallVec<[CoreEvent; 16]>,
}

/// Cross-mechanic precomputed values, computed once per tick at the top
/// of [`tack`] and threaded through every downstream phase. **The
/// canonical artifact for cross-mechanic flow** — when a designer wants
/// to read "where does Corruption affect Cubes affect Ants?", the
/// answer is this struct and the function that populates it
/// (`phase_cross_mechanic_precompute`).
///
/// Per Loom's tack-design memo, the goal of the cache is to make the
/// synergy graph **legible**. The legacy TS scattered these
/// computations across the aggregators' `*Pre` parameters, which every
/// caller hand-packed — silently dropping a field gave a working tick
/// that produced slightly less, with no compile error.
///
/// The four global-state bundles have all migrated to compute-from-state
/// (their derivations live in the Phase-1/2 functions, not the cache).
/// `automation_pre` is the last bundle still forwarded from
/// [`TackInput`]; it follows once the Phase-5 speed-mult aggregators port,
/// after which the cache is fully self-derived from `&GameState`.
#[derive(Debug, Clone, Default)]
pub struct CrossMechanicCache {
    /// Pre-evaluated inputs for the Phase 5 automation layer. Forwarded
    /// from [`TackInput`] today; state-derived field-by-field as the
    /// upstream speed-mult aggregators port.
    pub automation_pre: AutomationPre,
}

/// Captured outputs of the three Phase 2 aggregators, so downstream
/// phases can read aggregator-derived values without rerunning the
/// math. (Replaces what would otherwise be `G.*`-style mutable globals
/// in the legacy TS implementation.)
///
/// `global_multipliers` feeds [`compute_resource_gain_pre`] + [`phase_tax`];
/// `update_all_tick.accelerator_effect` feeds [`compute_reset_currency_gains`]
/// (the upgrade-16 prestige multiplier). `update_all_multiplier` is captured
/// for symmetry / future downstream reads.
#[derive(Debug, Clone, Copy)]
struct AggregatorOutputs {
    global_multipliers: GlobalMultipliersResult,
    #[expect(
        dead_code,
        reason = "captured for downstream phase migration; the lint will flip on as soon as a later phase reads it"
    )]
    update_all_multiplier: UpdateAllMultiplierResult,
    update_all_tick: UpdateAllTickResult,
}

/// Run one tick.
///
/// Phase ordering is canonical — see module docs. Reordering is a design
/// change requiring a separate commit and an updated CLAUDE.md note.
pub fn tack(state: &mut GameState, input: &TackInput) -> TickOutput {
    let mut output = TickOutput::default();

    let mut cache = phase_cross_mechanic_precompute(state, input);
    let aggregator_outputs = phase_global_state(state);
    // Phase 2b: coin production + tax. Mirrors the legacy `calculatetax()`
    // call slot — runs after the aggregators (it needs the fresh coin
    // multipliers), writes `g_cache.taxdivisor` for the *next* tick's
    // updateAllMultiplier, and supplies this tick's tax fields.
    let tax_outputs = phase_tax(state, &aggregator_outputs);
    // Reset-currency point gains (legacy `resetCurrency()`). They feed both
    // `ResourceGainPre` (point conversion) and the auto-reset amount-mode
    // thresholds in `AutomationPre`.
    let reset_gains = compute_reset_currency_gains(state, &aggregator_outputs);
    // Phase 2 + tax + reset outputs feed Phase 4's `ResourceGainPre`. It is
    // fully derived now (no caller bundle), so it's a tick-local value
    // rather than a cache field.
    let resource_gain_pre =
        compute_resource_gain_pre(&aggregator_outputs, &tax_outputs, &reset_gains);
    // Thread the same point gains into the automation bundle so auto-reset
    // amount mode compares against this tick's gain (state-derived now).
    cache.automation_pre.prestige_point_gain = reset_gains.prestige_point_gain;
    cache.automation_pre.transcend_point_gain = reset_gains.transcend_point_gain;
    cache.automation_pre.reincarnation_point_gain = reset_gains.reincarnation_point_gain;
    // Speed multipliers (legacy `G.timeMultiplier` / `ascensionSpeedMult`) —
    // self-derived from state, replacing the caller-provided AutomationPre
    // values.
    cache.automation_pre.global_time_multiplier = compute_global_speed_mult_pre(state);
    cache.automation_pre.ascension_speed_multi = compute_ascension_speed_mult_pre(state);
    cache.automation_pre.singularity_speed_multi = compute_singularity_speed_mult_pre(state);
    // Non-speed timer fields (auto-potion + GQ export), self-derived.
    let (offering_potions, obtainium_potions, auto_potion_speed, export_gq) =
        compute_auto_timer_fields(state);
    cache.automation_pre.offering_potion_count = offering_potions;
    cache.automation_pre.obtainium_potion_count = obtainium_potions;
    cache.automation_pre.auto_potion_speed_mult = auto_potion_speed;
    cache.automation_pre.export_gq_per_hour = export_gq;
    let ambrosia_luck = compute_ambrosia_luck_pre(state);
    cache.automation_pre.ambrosia_luck = ambrosia_luck;
    let ambrosia_generation_speed = compute_ambrosia_generation_speed_pre(state);
    cache.automation_pre.ambrosia_generation_speed = ambrosia_generation_speed;
    // Red-ambrosia luck / generation speed compose on this tick's ambrosia
    // luck (the `LuckConversion` line) and ambrosia generation speed (the
    // `BlueberrySpeed` line) respectively.
    cache.automation_pre.red_ambrosia_luck = compute_red_ambrosia_luck_pre(state, ambrosia_luck);
    cache.automation_pre.red_ambrosia_generation_speed =
        compute_red_ambrosia_generation_speed_pre(state, ambrosia_generation_speed);
    // Ambrosia-timer threshold fields (legacy Helper.ts `addTimers('ambrosia')`).
    let (bonus_ambrosia, time_per_ambrosia, ambrosia_accelerator_mult, ambrosia_brick_of_lead_mult) =
        compute_ambrosia_timer_fields(state);
    cache.automation_pre.bonus_ambrosia = bonus_ambrosia;
    cache.automation_pre.time_per_ambrosia = time_per_ambrosia;
    cache.automation_pre.ambrosia_accelerator_mult = ambrosia_accelerator_mult;
    cache.automation_pre.ambrosia_brick_of_lead_mult = ambrosia_brick_of_lead_mult;
    phase_player_input(state, input, &mut output);
    phase_generation(state, &resource_gain_pre, input.dt, &mut output);
    phase_automation(state, &cache, input, &mut output);

    output
}

/// **Phase 1** — Cross-mechanic precompute.
///
/// Builds the [`CrossMechanicCache`]. All four global-state bundles now
/// self-derive inside their consuming phases ([`phase_global_state`],
/// [`phase_tax`], [`compute_reset_currency_gains`]), so the cache is down
/// to forwarding the last caller bundle, `automation_pre`, until the
/// Phase-5 speed-mult aggregators port.
fn phase_cross_mechanic_precompute(_state: &GameState, input: &TackInput) -> CrossMechanicCache {
    CrossMechanicCache {
        automation_pre: input.automation_pre,
    }
}

/// Build the shared achievement-reward input from `&GameState` — the
/// earned-flag array plus the cross-state values the reward formulas
/// read (coin-producer owned counts, prestige points).
fn achievement_reward_input(state: &GameState) -> achievement_rewards::AchievementRewardInput<'_> {
    let coin = &state.coin_producers.tiers;
    let cc = &state.challenges.challenge_completions;
    achievement_rewards::AchievementRewardInput {
        achievements: &state.achievements.achievements,
        coin_owned: [
            coin[0].owned,
            coin[1].owned,
            coin[2].owned,
            coin[3].owned,
            coin[4].owned,
        ],
        prestige_points: state.upgrades.prestige_points,
        challenge_completions_6_to_10: [cc[6], cc[7], cc[8], cc[9], cc[10]],
    }
}

/// State-derive `G.acceleratorMultiplier` via the legacy
/// `calculateAcceleratorMultiplier` formula — research/upgrade
/// compounding, the `21..=25` upgrade pentad, and the in-challenge
/// `upgrade[50]` bonus. Pure function of `&GameState`.
fn compute_accelerator_multiplier(state: &GameState) -> f64 {
    use crate::mechanics::accelerator_multipliers::{
        calculate_accelerator_multiplier, CalculateAcceleratorMultiplierInput,
    };
    let upgrade = |i: usize| f64::from(state.upgrades.upgrades[i]);
    let research = |i: usize| state.researches.researches[i];
    calculate_accelerator_multiplier(&CalculateAcceleratorMultiplierInput {
        research_1: research(1),
        challenge_completions_14: state.challenges.challenge_completions[14],
        research_6: research(6),
        research_7: research(7),
        research_8: research(8),
        research_9: research(9),
        research_10: research(10),
        research_86: research(86),
        research_126: research(126),
        research_141: research(141),
        research_156: research(156),
        research_171: research(171),
        research_186: research(186),
        research_200: research(200),
        cube_upgrade_50: state.cube_upgrade_levels.cube_upgrades[50],
        upgrade_21: upgrade(21),
        upgrade_22: upgrade(22),
        upgrade_23: upgrade(23),
        upgrade_24: upgrade(24),
        upgrade_25: upgrade(25),
        upgrade_50: upgrade(50),
        in_transcension_or_reincarnation_challenge: state.challenges.current_transcension_challenge
            != 0
            || state.challenges.current_reincarnation_challenge != 0,
    })
}

/// State-derive `G.totalAcceleratorBoost` via the legacy
/// `calculateTotalAcceleratorBoost` (free boost from upgrades /
/// researches / runes / ant + hepteract effects, then `+ bought`).
/// Pure function of `&GameState`. Injected into all three aggregator
/// `*Pre` bundles by [`phase_cross_mechanic_precompute`].
fn compute_total_accelerator_boost(state: &GameState) -> f64 {
    use crate::mechanics::accelerator_multipliers::{
        calculate_total_accelerator_boost, CalculateTotalAcceleratorBoostInput,
    };
    use crate::mechanics::ant_upgrades::accelerator_boosts_ant_upgrade_effect;
    use crate::mechanics::calculate::{calculate_total_coin_owned, CalculateTotalCoinOwnedInput};
    use crate::mechanics::hepteract_values::{hepteract_effective, HepteractEffectiveInput};

    /// Ant-upgrade index for "AcceleratorBoosts" (legacy `AntUpgrades` = 3).
    const ANT_UPGRADE_ACCELERATOR_BOOSTS: usize = 3;

    let upgrade = |i: usize| f64::from(state.upgrades.upgrades[i]);
    let research = |i: usize| state.researches.researches[i];
    let coin = &state.coin_producers.tiers;
    let total_coin_owned = calculate_total_coin_owned(&CalculateTotalCoinOwnedInput {
        first_owned_coin: coin[0].owned,
        second_owned_coin: coin[1].owned,
        third_owned_coin: coin[2].owned,
        fourth_owned_coin: coin[3].owned,
        fifth_owned_coin: coin[4].owned,
    });
    let sum_of_rune_levels: f64 = state.runes.rune_levels.iter().sum();
    // acceleratorBoost hepteract: LIMIT 1000, DR 1/5, DR_INCREASE = 0.
    let hepteract_effective_accelerator_boost = hepteract_effective(&HepteractEffectiveInput {
        raw_amount: state.hepteracts.accelerator_boost.bal,
        limit: 1000.0,
        dr_exponent: 1.0 / 5.0,
        is_quark: false,
    });
    let ach = achievement_reward_input(state);

    calculate_total_accelerator_boost(&CalculateTotalAcceleratorBoostInput {
        upgrade_26: upgrade(26),
        upgrade_31: upgrade(31),
        total_coin_owned,
        achievement_accel_boosts: achievement_rewards::accel_boosts(&ach),
        research_93: research(93),
        sum_of_rune_levels,
        research_3: research(3),
        challenge_completions_14: state.challenges.challenge_completions[14],
        research_16: research(16),
        research_17: research(17),
        research_88: research(88),
        ant_building_accelerator_boost_mult: accelerator_boosts_ant_upgrade_effect(
            state.ants.upgrades[ANT_UPGRADE_ACCELERATOR_BOOSTS],
        ),
        research_127: research(127),
        research_142: research(142),
        research_157: research(157),
        research_172: research(172),
        research_187: research(187),
        research_200: research(200),
        cube_upgrade_50: state.cube_upgrade_levels.cube_upgrades[50],
        hepteract_effective_accelerator_boost,
        upgrade_73: upgrade(73),
        in_reincarnation_challenge: state.challenges.current_reincarnation_challenge != 0,
        accelerator_boost_bought: state.accelerator.accelerator_boost_bought,
    })
    .total_accelerator_boost
}

/// State-derive `G.buildingPower` via the legacy `calculateBuildingPower`.
/// Pure function of `&GameState`, shared by
/// [`compute_global_multipliers_pre`] (the `building_power` /
/// `building_power_mult` bundle fields) and [`phase_tax`] (the flat
/// max-exponent increase).
fn compute_building_power(state: &GameState) -> f64 {
    use crate::mechanics::ant_upgrades::building_cost_scale_ant_upgrade_effect;
    use crate::mechanics::challenges::{calc_ecc, ChallengeType};
    use crate::mechanics::crystal_and_building_power::{
        calculate_building_power, CalculateBuildingPowerInput,
    };

    /// Ant-upgrade index for "BuildingCostScale" (legacy `AntUpgrades` = 6).
    const ANT_UPGRADE_BUILDING_COST_SCALE: usize = 6;

    calculate_building_power(&CalculateBuildingPowerInput {
        c8_reincarnation_ecc: calc_ecc(
            ChallengeType::Reincarnation,
            state.challenges.challenge_completions[8],
        ),
        reincarnation_shards: state.reset_counters.reincarnation_shards,
        research_36: state.researches.researches[36],
        research_37: state.researches.researches[37],
        research_38: state.researches.researches[38],
        building_cost_scale_ant_upgrade_building_power_mult:
            building_cost_scale_ant_upgrade_effect(
                state.ants.upgrades[ANT_UPGRADE_BUILDING_COST_SCALE],
            )
            .building_power_mult,
        cube_upgrade_12: state.cube_upgrade_levels.cube_upgrades[12],
        cube_upgrade_36: state.cube_upgrade_levels.cube_upgrades[36],
        in_reincarnation_challenge_7: state.challenges.current_reincarnation_challenge == 7,
    })
}

/// State-derive the [`GlobalMultipliersPreEvaluated`] fields whose
/// upstream is a pure function of [`GameState`] and existing ported
/// mechanic helpers.
///
/// Migration coverage today:
/// - `prism_production_log10`           ✓ state-derived (Prism rune)
/// - `ant_multiplier`                   ✓ state-derived (Coins ant upgrade)
/// - `total_coin_owned`                 ✓ state-derived (sum of coin tiers)
/// - `recession_power`                  ✓ state-derived (G.recessionPower table)
/// - `crystal_mult`                     ✓ state-derived (crystal-coin pipeline)
/// - `building_power`                   ✓ state-derived (`compute_building_power`)
/// - `building_power_mult`              ✓ state-derived (building_power ^ coin owned)
/// - `crystal_upgrade_3_multiplier`     ✓ state-derived (crystal-upgrade-3 chain)
/// - `crystal_multiplier_achievement`   ✓ state-derived (achievement_rewards)
/// - `const_upgrade_1_buff_achievement` ✓ always 0 (no achievement grants it)
/// - `const_upgrade_2_buff_achievement` ✓ always 0 (no achievement grants it)
/// - `constant_ex_max_percent_increase` ✓ shop subsystem unported → 0 (no logic buy-path)
/// - `ascend_building_dr_value`         ✓ state-derived (`ascend_building_dr`)
/// - `multiplier_effect`                ✓ injected by phase_global_state (aggregator output)
/// - `accelerator_effect`               ✓ injected by phase_global_state (aggregator output)
/// - `total_multiplier`                 ✓ injected by phase_global_state (aggregator output)
/// - `total_accelerator`                ✓ injected by phase_global_state (aggregator output)
/// - `total_accelerator_boost`          ✓ injected by phase_global_state (compute_total_accelerator_boost)
/// - `challenge_15_coin_exponent`       ✓ state-derived (challenge_15_rewards)
/// - `challenge_15_exponent_value`      ✓ state-derived (challenge_15_rewards)
/// - `challenge_15_constant_bonus`      ✓ state-derived (challenge_15_rewards)
#[must_use]
fn compute_global_multipliers_pre(state: &GameState) -> GlobalMultipliersPreEvaluated {
    use crate::mechanics::ant_upgrades::{coins_ant_upgrade_effect, CoinsAntUpgradeInput};
    use crate::mechanics::calculate::{calculate_total_coin_owned, CalculateTotalCoinOwnedInput};
    use crate::mechanics::challenges::{calc_ecc, ChallengeType};
    use crate::mechanics::corruptions::recession_power_at_level;
    use crate::mechanics::crystal_and_building_power::{
        ascend_building_dr, calculate_building_power_coin_multiplier,
        calculate_crystal_coin_multiplier, calculate_crystal_exponent, crystal_upgrade_3_base,
        crystal_upgrade_3_crystal_multiplier, crystal_upgrade_3_max_base,
        crystal_upgrade_4_max_exponent, CalculateCrystalExponentInput, CrystalUpgrade3BaseInput,
        CrystalUpgrade3CrystalMultiplierInput, CrystalUpgrade3MaxBaseInput,
        CrystalUpgrade4MaxExponentInput,
    };
    use crate::mechanics::rune_effects::{prism_rune_effects, PrismRuneKey};
    use crate::state::{RECESSION_INDEX, RUNE_PRISM};

    /// Ant-upgrade index for "Coins". Mirrors the legacy
    /// `AntUpgrades.Coins = 1` enum value.
    const ANT_UPGRADE_COINS: usize = 1;

    let prism_level = state.runes.rune_levels[RUNE_PRISM];
    let coin_tiers = &state.coin_producers.tiers;
    let total_coin_owned = calculate_total_coin_owned(&CalculateTotalCoinOwnedInput {
        first_owned_coin: coin_tiers[0].owned,
        second_owned_coin: coin_tiers[1].owned,
        third_owned_coin: coin_tiers[2].owned,
        fourth_owned_coin: coin_tiers[3].owned,
        fifth_owned_coin: coin_tiers[4].owned,
    });
    let ant_effect = coins_ant_upgrade_effect(&CoinsAntUpgradeInput {
        level: state.ants.upgrades[ANT_UPGRADE_COINS],
        ascension_challenge: state.challenges.current_ascension_challenge,
        crumbs: state.ants.crumbs,
    });
    let recession_level = state.corruptions.used.levels[RECESSION_INDEX];
    let ach = achievement_reward_input(state);
    let c15_exponent = state.challenges.challenge15_exponent;

    // ─── Building power → coin multiplier ────────────────────────────────
    let building_power = compute_building_power(state);
    let building_power_mult =
        calculate_building_power_coin_multiplier(building_power, total_coin_owned);

    // ─── Crystal coin multiplier (prestige-shards production) ─────────────
    // `prism_spirit_crystal_caps` needs rune-spirit power (the unported
    // `spiritMultiplier` chain); prism spirit level is 0 in current play,
    // so the additive cap contribution is 0.
    let crystal_upgrade_4_max_exp =
        crystal_upgrade_4_max_exponent(&CrystalUpgrade4MaxExponentInput {
            research_129: state.researches.researches[129],
            common_fragments: Decimal::from_finite(state.talismans.common_fragments),
            prism_spirit_crystal_caps: 0.0,
        });
    let crystal_exponent = calculate_crystal_exponent(&CalculateCrystalExponentInput {
        crystal_upgrade_3_max_exponent: crystal_upgrade_4_max_exp,
        crystal_upgrade_3: state.crystal_upgrades.crystal_upgrades[3],
        c3_transcend_ecc: calc_ecc(
            ChallengeType::Transcend,
            state.challenges.challenge_completions[3],
        ),
        research_28: state.researches.researches[28],
        research_29: state.researches.researches[29],
        research_30: state.researches.researches[30],
        cube_upgrade_17: state.cube_upgrade_levels.cube_upgrades[17],
    });
    let crystal_mult =
        calculate_crystal_coin_multiplier(state.crystal_upgrades.prestige_shards, crystal_exponent);

    // ─── Crystal-upgrade-3 crystal multiplier (max_base → base → mult) ───
    let crystal_u3_base = crystal_upgrade_3_base(&CrystalUpgrade3BaseInput {
        max_base: crystal_upgrade_3_max_base(&CrystalUpgrade3MaxBaseInput {
            upgrade_122: f64::from(state.upgrades.upgrades[122]),
            research_129: state.researches.researches[129],
            common_fragments: Decimal::from_finite(state.talismans.common_fragments),
        }),
        crystal_upgrade_2: state.crystal_upgrades.crystal_upgrades[2],
    });
    let diamonds = &state.diamond_producers.tiers;
    let crystal_producers_owned = diamonds[0].owned
        + diamonds[1].owned
        + diamonds[2].owned
        + diamonds[3].owned
        + diamonds[4].owned;
    let crystal_upgrade_3_multiplier =
        crystal_upgrade_3_crystal_multiplier(&CrystalUpgrade3CrystalMultiplierInput {
            base: crystal_u3_base,
            crystal_producers_owned,
        });

    // ─── Ascend-building diminishing returns ─────────────────────────────
    let ab = &state.tesseract_buildings;
    let ascend_building_dr_value = ascend_building_dr(
        ab.ascend_building_1.owned
            + ab.ascend_building_2.owned
            + ab.ascend_building_3.owned
            + ab.ascend_building_4.owned
            + ab.ascend_building_5.owned,
    );

    GlobalMultipliersPreEvaluated {
        prism_production_log10: prism_rune_effects(prism_level, PrismRuneKey::ProductionLog10),
        total_coin_owned,
        ant_multiplier: ant_effect.coin_multiplier,
        recession_power: recession_power_at_level(recession_level),
        crystal_mult,
        building_power,
        building_power_mult,
        crystal_upgrade_3_multiplier,
        crystal_multiplier_achievement: achievement_rewards::crystal_multiplier(&ach),
        // No achievement grants `constUpgrade1Buff`/`constUpgrade2Buff` in
        // the legacy table — the additive reward is always 0.
        const_upgrade_1_buff_achievement: 0.0,
        const_upgrade_2_buff_achievement: 0.0,
        // `constantEX` shop upgrade (`getShopUpgradeEffects` = identity):
        // the shop name→index map / buy-path is UI-tier and unported, so
        // the level is 0 in logic-driven play → 0.
        constant_ex_max_percent_increase: 0.0,
        ascend_building_dr_value,
        // Placeholders — phase_global_state overwrites these five with the
        // aggregator outputs + the shared total_accelerator_boost.
        multiplier_effect: Decimal::zero(),
        accelerator_effect: Decimal::zero(),
        total_multiplier: 0.0,
        total_accelerator: 0.0,
        total_accelerator_boost: 0.0,
        challenge_15_coin_exponent: challenge_15_rewards::coin_exponent(c15_exponent),
        challenge_15_exponent_value: challenge_15_rewards::exponent_reward(c15_exponent),
        challenge_15_constant_bonus: challenge_15_rewards::constant_bonus(c15_exponent),
    }
}

/// State-derive the [`UpdateAllMultiplierPre`] fields whose upstream
/// is a pure function of [`GameState`]. Fields whose upstream depends
/// on cross-aggregator outputs (the `G.*` cache) keep their
/// caller-provided value from `fallback`.
///
/// Migration coverage today:
/// - `sum_of_rune_levels`               ✓ state-derived
/// - `multiplicative_multipliers_rune`  ✓ state-derived (Duplication rune)
/// - `multiplier_boosts_rune`           ✓ state-derived (Duplication rune)
/// - `multiplier_boosts_rune_blessing`  ✓ state-derived (Duplication blessing)
/// - `ant_multiplier_mult`              ✓ state-derived (Multipliers ant upgrade)
/// - `hepteract_multiplier`             ✓ state-derived
/// - `hepteract_multiplier_mult`        ✓ state-derived
/// - `viscosity_power`                  ✓ state-derived (G.viscosityPower table)
/// - `multiplier_cube_blessing`         ✓ state-derived (full blessing chain)
/// - `multipliers_achievement`          ✓ state-derived (achievement_rewards)
/// - `total_accelerator_boost`          ✓ caller-passed (computed once in `phase_global_state`)
/// - `taxdivisor`                        ✓ state-derived (prior tick's `g_cache.taxdivisor` — one-tick lag)
/// - `challenge_15_reward_multiplier`   ✓ state-derived (challenge_15_rewards)
#[must_use]
fn compute_update_all_multiplier_pre(
    state: &GameState,
    total_accelerator_boost: f64,
) -> UpdateAllMultiplierPre {
    use crate::mechanics::ant_upgrades::multipliers_ant_upgrade_effect;
    use crate::mechanics::corruptions::viscosity_power_at_level;
    use crate::mechanics::cube_blessings::calculate_multiplier_cube_blessing;
    use crate::mechanics::hepteract_effects::multiplier_hepteract_effects;
    use crate::mechanics::hypercube_blessings::calculate_multiplier_hypercube_blessing;
    use crate::mechanics::platonic_blessings::calculate_hypercube_blessing_multiplier_platonic_blessing;
    use crate::mechanics::rune_blessing_effects::duplication_rune_blessing_effects;
    use crate::mechanics::rune_effects::{duplication_rune_effects, DuplicationRuneKey};
    use crate::mechanics::tesseract_blessings::calculate_multiplier_tesseract_blessing;
    use crate::state::{RUNE_DUPLICATION, VISCOSITY_INDEX};

    /// Ant-upgrade index for "Multipliers". Mirrors the legacy
    /// `AntUpgrades.Multipliers = 4` enum value.
    const ANT_UPGRADE_MULTIPLIERS: usize = 4;
    /// Cube-upgrade index gating the cube-multiplier blessing's
    /// diminishing-return increase. Legacy `player.cubeUpgrades[35]`.
    const CUBE_UPGRADE_MULTIPLIER_BLESSING: usize = 35;

    let sum_of_rune_levels: f64 = state.runes.rune_levels.iter().sum();
    let duplication_level = state.runes.rune_levels[RUNE_DUPLICATION];
    let duplication_blessing_level = state.runes.rune_blessing_levels[RUNE_DUPLICATION];
    let hept_mult = multiplier_hepteract_effects(state.hepteracts.multiplier.bal);
    let viscosity_level = state.corruptions.used.levels[VISCOSITY_INDEX];
    // Cube-blessing chain: platonic → hypercube → tesseract → cube,
    // mirroring the legacy call chain in `Cubes.ts`.
    let platonic_amplifier =
        calculate_hypercube_blessing_multiplier_platonic_blessing(&state.platonic_blessings);
    let hypercube_blessing =
        calculate_multiplier_hypercube_blessing(&state.hypercube_blessings, platonic_amplifier);
    let tesseract_blessing =
        calculate_multiplier_tesseract_blessing(&state.tesseract_blessings, hypercube_blessing);
    let cube_blessing = calculate_multiplier_cube_blessing(
        &state.cube_blessings,
        tesseract_blessing,
        state.cube_upgrade_levels.cube_upgrades[CUBE_UPGRADE_MULTIPLIER_BLESSING],
    );
    let ach = achievement_reward_input(state);

    UpdateAllMultiplierPre {
        sum_of_rune_levels,
        multiplicative_multipliers_rune: duplication_rune_effects(
            duplication_level,
            DuplicationRuneKey::MultiplicativeMultipliers,
        ),
        multiplier_boosts_rune: duplication_rune_effects(
            duplication_level,
            DuplicationRuneKey::MultiplierBoosts,
        ),
        multiplier_boosts_rune_blessing: duplication_rune_blessing_effects(
            duplication_blessing_level,
        )
        .multiplier_boosts,
        ant_multiplier_mult: multipliers_ant_upgrade_effect(
            state.ants.upgrades[ANT_UPGRADE_MULTIPLIERS],
        ),
        hepteract_multiplier: hept_mult.multiplier,
        hepteract_multiplier_mult: hept_mult.multiplier_multiplier,
        viscosity_power: viscosity_power_at_level(viscosity_level),
        multiplier_cube_blessing: cube_blessing,
        multipliers_achievement: achievement_rewards::multipliers(&ach),
        total_accelerator_boost,
        // Prior tick's `G.taxdivisor`; the tax phase recomputes it later
        // this tick, so the Phase-2 consumer (upgrade-68) reads the lagged
        // value — faithful to the legacy mutable-global ordering.
        taxdivisor: state.g_cache.taxdivisor,
        challenge_15_reward_multiplier: challenge_15_rewards::multiplier(
            state.challenges.challenge15_exponent,
        ),
    }
}

/// State-derive the [`UpdateAllTickPre`] fields whose upstream is a
/// pure function of [`GameState`].
///
/// Migration coverage today:
/// - `multiplicative_accelerators_rune` ✓ state-derived (Speed rune)
/// - `accelerator_power_rune`           ✓ state-derived (Speed rune)
/// - `hepteract_accelerators`           ✓ state-derived
/// - `hepteract_accelerator_mult`       ✓ state-derived
/// - `viscosity_power`                  ✓ state-derived (G.viscosityPower table)
/// - `accelerator_cube_blessing`        ✓ state-derived (full blessing chain)
/// - `accelerators_achievement`         ✓ state-derived (achievement_rewards)
/// - `accelerator_power_achievement`    ✓ state-derived (achievement_rewards)
/// - `total_accelerator_boost`          ✓ caller-passed (computed once in `phase_global_state`)
/// - `accelerator_multiplier`           ✓ state-derived (calculate_accelerator_multiplier)
/// - `challenge_15_reward_accelerator`  ✓ state-derived (challenge_15_rewards)
#[must_use]
fn compute_update_all_tick_pre(
    state: &GameState,
    total_accelerator_boost: f64,
) -> UpdateAllTickPre {
    use crate::mechanics::corruptions::viscosity_power_at_level;
    use crate::mechanics::cube_blessings::calculate_accelerator_cube_blessing;
    use crate::mechanics::hepteract_effects::accelerator_hepteract_effects;
    use crate::mechanics::hypercube_blessings::calculate_accelerator_hypercube_blessing;
    use crate::mechanics::platonic_blessings::calculate_hypercube_blessing_multiplier_platonic_blessing;
    use crate::mechanics::rune_effects::{speed_rune_effects, SpeedRuneKey};
    use crate::mechanics::tesseract_blessings::calculate_accelerator_tesseract_blessing;
    use crate::state::{RUNE_SPEED, VISCOSITY_INDEX};

    /// Cube-upgrade index gating the cube-accelerator blessing's
    /// diminishing-return increase. Legacy `player.cubeUpgrades[45]`.
    const CUBE_UPGRADE_ACCELERATOR_BLESSING: usize = 45;

    let speed_level = state.runes.rune_levels[RUNE_SPEED];
    let hept_acc = accelerator_hepteract_effects(state.hepteracts.accelerator.bal);
    let viscosity_level = state.corruptions.used.levels[VISCOSITY_INDEX];
    // Cube-blessing chain (same shape as the multiplier chain in
    // [`compute_update_all_multiplier_pre`]; the platonic amplifier
    // is shared between the two tracks).
    let platonic_amplifier =
        calculate_hypercube_blessing_multiplier_platonic_blessing(&state.platonic_blessings);
    let hypercube_blessing =
        calculate_accelerator_hypercube_blessing(&state.hypercube_blessings, platonic_amplifier);
    let tesseract_blessing =
        calculate_accelerator_tesseract_blessing(&state.tesseract_blessings, hypercube_blessing);
    let cube_blessing = calculate_accelerator_cube_blessing(
        &state.cube_blessings,
        tesseract_blessing,
        state.cube_upgrade_levels.cube_upgrades[CUBE_UPGRADE_ACCELERATOR_BLESSING],
    );
    let ach = achievement_reward_input(state);
    let accelerator_multiplier = compute_accelerator_multiplier(state);

    UpdateAllTickPre {
        multiplicative_accelerators_rune: speed_rune_effects(
            speed_level,
            SpeedRuneKey::MultiplicativeAccelerators,
        ),
        accelerator_power_rune: speed_rune_effects(speed_level, SpeedRuneKey::AcceleratorPower),
        hepteract_accelerators: hept_acc.accelerators,
        hepteract_accelerator_mult: hept_acc.accelerator_multiplier,
        viscosity_power: viscosity_power_at_level(viscosity_level),
        accelerator_cube_blessing: cube_blessing,
        accelerators_achievement: achievement_rewards::accelerators(&ach),
        accelerator_power_achievement: achievement_rewards::accelerator_power(&ach),
        total_accelerator_boost,
        accelerator_multiplier,
        challenge_15_reward_accelerator: challenge_15_rewards::accelerator(
            state.challenges.challenge15_exponent,
        ),
    }
}

/// **Phase 2** — Global state aggregators.
///
/// Derives the (now fully state-driven) `update_all_multiplier` /
/// `update_all_tick` pre-bundles, runs the three pure aggregators in
/// dependency order, and injects their cross-cutting outputs into the
/// global-multipliers bundle. The results flow into the
/// [`AggregatorOutputs`] return value so Phase 4 (`resource_gain`) and
/// the tax phase can read cross-aggregator values directly instead of
/// forwarding them from `TackInput`.
///
/// `total_accelerator_boost` is a pure function of state read by all three
/// aggregators; it is computed once here (this phase is its only consumer)
/// and threaded into each bundle.
fn phase_global_state(state: &mut GameState) -> AggregatorOutputs {
    let total_accelerator_boost = compute_total_accelerator_boost(state);
    let update_all_multiplier_pre =
        compute_update_all_multiplier_pre(state, total_accelerator_boost);
    let update_all_tick_pre = compute_update_all_tick_pre(state, total_accelerator_boost);

    // Legacy dependency order: `updateAllMultiplier`, then `updateAllTick`
    // (which consumes `total_multiplier`), then `globalMultipliers` last —
    // reading the multiplier/tick `G.*` outputs. The aggregators are pure
    // (no production state writes), so the reorder is behaviour-preserving.
    let update_all_multiplier_result = update_all_multiplier(state, &update_all_multiplier_pre);
    let update_all_tick_result = update_all_tick(
        state,
        &update_all_tick_pre,
        update_all_multiplier_result.total_multiplier,
    );

    // Derive the global-multipliers bundle from state, then inject the
    // cross-cutting outputs (`total_accelerator_boost` plus the two
    // aggregators' effects — all forwarded from `TackInput` before).
    let mut global_multipliers_pre = compute_global_multipliers_pre(state);
    global_multipliers_pre.total_accelerator_boost = total_accelerator_boost;
    global_multipliers_pre.multiplier_effect = update_all_multiplier_result.multiplier_effect;
    global_multipliers_pre.total_multiplier = update_all_multiplier_result.total_multiplier;
    global_multipliers_pre.accelerator_effect = update_all_tick_result.accelerator_effect;
    global_multipliers_pre.total_accelerator = update_all_tick_result.total_accelerator;
    let global_multipliers = compute_global_multipliers(state, &global_multipliers_pre);

    AggregatorOutputs {
        global_multipliers,
        update_all_multiplier: update_all_multiplier_result,
        update_all_tick: update_all_tick_result,
    }
}

/// Coin-side `produce_total` plus the three tax outputs, computed after
/// Phase 2 and fed into Phase 4's [`ResourceGainPre`]. Mirrors the four
/// `G.*` values the legacy `calculatetax()` shim wrote
/// (`produceTotal`, `taxdivisor`, `taxdivisorcheck`, `maxexponent`).
#[derive(Debug, Clone, Copy)]
struct TaxOutputs {
    /// `G.produceTotal` — sum of pre-clamp coin-tier outputs.
    produce_total: Decimal,
    /// `G.taxdivisor` — freshly recomputed this tick.
    taxdivisor: Decimal,
    /// `G.taxdivisorcheck`.
    taxdivisorcheck: Decimal,
    /// `G.maxexponent`.
    maxexponent: f64,
}

/// Legacy `player.{first..fifth}ProduceCoin` — the immutable per-tier coin
/// production scalars (×10 per tier). Never reassigned anywhere in the
/// legacy source, so hoisted as a constant rather than stored per-game.
const COIN_PRODUCE_SCALARS: [f64; 5] = [0.25, 2.5, 25.0, 250.0, 2500.0];

/// **Phase 2b** — coin production + tax.
///
/// Runs after [`phase_global_state`] (it needs that phase's freshly-
/// aggregated coin multipliers) and before Phase 4, mirroring the legacy
/// `calculatetax()`: aggregate the five coin tiers into `G.produceTotal`,
/// then run the tax exponent / divisor formula. The fresh `taxdivisor` is
/// written back into [`crate::state::GCacheState`] so the **next** tick's
/// [`update_all_multiplier`] reads it (the deliberate one-tick lag); the
/// four outputs also feed [`compute_resource_gain_pre`] for this tick's
/// coin gain.
///
/// The legacy `shouldAwardOvertaxed` flag is a UI-tier achievement side
/// effect (`awardUngroupedAchievement('overtaxed')`) with no `CoreEvent`
/// variant yet — deferred, not wired here.
fn phase_tax(state: &mut GameState, agg: &AggregatorOutputs) -> TaxOutputs {
    use crate::mechanics::ant_upgrades::{
        coins_ant_upgrade_effect, taxes_ant_upgrade_effect, CoinsAntUpgradeInput,
    };
    use crate::mechanics::calculate::{calculate_total_coin_owned, CalculateTotalCoinOwnedInput};
    use crate::mechanics::coin_production::{
        calculate_coin_production, CalculateCoinProductionInput, PerCoinTierInput,
    };
    use crate::mechanics::crystal_and_building_power::calculate_building_power_coin_multiplier;
    use crate::mechanics::platonic_blessings::calculate_tax_platonic_blessing;
    use crate::mechanics::rune_effects::{
        duplication_rune_effects, thrift_rune_effects, DuplicationRuneKey, ThriftRuneKey,
    };
    use crate::mechanics::talisman_effects::exemption_talisman_effects;
    use crate::mechanics::tax::{calculate_tax, CalculateTaxInput};
    use crate::mechanics::{campaign_token_rewards, challenge_15_rewards};
    use crate::state::{RUNE_DUPLICATION, RUNE_THRIFT, TALISMAN_EXEMPTION};

    /// Ant-upgrade indices (legacy `AntUpgrades` enum): Coins / Taxes.
    const ANT_UPGRADE_COINS: usize = 1;
    const ANT_UPGRADE_TAXES: usize = 2;

    let g = &agg.global_multipliers;
    let coin = &state.coin_producers.tiers;
    let challenges = &state.challenges;
    let researches = &state.researches.researches;

    // ─── G.produceTotal via the five coin tiers ──────────────────────────
    let tier = |i: usize, coin_multi: Decimal| PerCoinTierInput {
        generated: coin[i].generated,
        owned: coin[i].owned,
        coin_multi,
        produce_coin: COIN_PRODUCE_SCALARS[i],
    };
    let produce_total = calculate_coin_production(CalculateCoinProductionInput {
        first: tier(0, g.coin_one_multi),
        second: tier(1, g.coin_two_multi),
        third: tier(2, g.coin_three_multi),
        fourth: tier(3, g.coin_four_multi),
        fifth: tier(4, g.coin_five_multi),
        global_coin_multiplier: g.global_coin_multiplier,
    })
    .total;

    // ─── flat_max_exponent_increase inputs (ant Coins + building power) ───
    let total_coin_owned = calculate_total_coin_owned(&CalculateTotalCoinOwnedInput {
        first_owned_coin: coin[0].owned,
        second_owned_coin: coin[1].owned,
        third_owned_coin: coin[2].owned,
        fourth_owned_coin: coin[3].owned,
        fifth_owned_coin: coin[4].owned,
    });
    let coins_ant = coins_ant_upgrade_effect(&CoinsAntUpgradeInput {
        level: state.ants.upgrades[ANT_UPGRADE_COINS],
        ascension_challenge: challenges.current_ascension_challenge,
        crumbs: state.ants.crumbs,
    });
    let building_power_coin_multiplier =
        calculate_building_power_coin_multiplier(compute_building_power(state), total_coin_owned);

    // ─── tax exponent / divisor ──────────────────────────────────────────
    let ach = achievement_reward_input(state);
    let total_challenge_completions: f64 = challenges.challenge_completions.iter().sum();

    let tax = calculate_tax(&CalculateTaxInput {
        in_reinc_6: challenges.current_reincarnation_challenge == 6,
        in_reinc_9: challenges.current_reincarnation_challenge == 9,
        in_ascension_15: challenges.current_ascension_challenge == 15,
        in_ascension_13: challenges.current_ascension_challenge == 13,
        c6_completions: challenges.challenge_completions[6],
        c13_completions: challenges.challenge_completions[13],
        total_challenge_completions,
        c11_completions: challenges.challenge_completions[11],
        c12_completions: challenges.challenge_completions[12],
        c14_completions: challenges.challenge_completions[14],
        c15_completions: challenges.challenge_completions[15],
        singularity_count: state.singularity.singularity_count,
        research_51: researches[51],
        research_52: researches[52],
        research_53: researches[53],
        research_54: researches[54],
        research_55: researches[55],
        research_159: researches[159],
        research_200: researches[200],
        cube_upgrade_50: state.cube_upgrade_levels.cube_upgrades[50],
        platonic_upgrade_5: state.cube_upgrade_levels.platonic_upgrades[5],
        platonic_upgrade_10: state.cube_upgrade_levels.platonic_upgrades[10],
        tax_platonic_blessing: calculate_tax_platonic_blessing(&state.platonic_blessings),
        upgrade_121: f64::from(state.upgrades.upgrades[121]),
        upgrade_125: f64::from(state.upgrades.upgrades[125]),
        c10_completions: challenges.challenge_completions[10],
        highest_singularity_count: state.singularity.highest_singularity_count,
        taxman_last_stand_enabled: state.singularity.taxman_last_stand.enabled,
        ascensions_unlocked: state.reset_counters.ascension_unlocked,
        highest_c14_completions: challenges.highest_challenge_completions[14],
        tax_reduction_achievement: achievement_rewards::tax_reduction(&ach),
        duplication_rune_tax_reduction: duplication_rune_effects(
            state.runes.rune_levels[RUNE_DUPLICATION],
            DuplicationRuneKey::TaxReduction,
        ),
        thrift_rune_tax_reduction: thrift_rune_effects(
            state.runes.rune_levels[RUNE_THRIFT],
            ThriftRuneKey::TaxReduction,
        ),
        ant_tax_reduction: taxes_ant_upgrade_effect(state.ants.upgrades[ANT_UPGRADE_TAXES]),
        exemption_talisman_tax_reduction: exemption_talisman_effects(
            state.talismans.talisman_rarity[TALISMAN_EXEMPTION] as i32,
        )
        .tax_reduction,
        challenge_15_taxes_reward: challenge_15_rewards::taxes(challenges.challenge15_exponent),
        // Campaign-token subsystem unported → 0 tokens → multiplier 1
        // (`campaignTaxMultiplier` returns 1 below 250 tokens).
        campaign_tax_multiplier: campaign_token_rewards::campaign_tax_multiplier(0.0),
        ascend_shards: state.campaigns.ascend_shards,
        rare_fragments: Decimal::from_finite(state.talismans.rare_fragments),
        fortunae_formicidae_coin_multiplier: coins_ant.coin_multiplier,
        building_power_coin_multiplier,
        produce_total,
    });

    // Recompute G.taxdivisor for the *next* tick's updateAllMultiplier read.
    state.g_cache.taxdivisor = tax.taxdivisor;

    TaxOutputs {
        produce_total,
        taxdivisor: tax.taxdivisor,
        taxdivisorcheck: tax.taxdivisorcheck,
        maxexponent: tax.maxexponent,
    }
}

/// Global-speed multiplier (legacy `G.timeMultiplier` /
/// `calculateGlobalSpeedMult`), self-derived from `&GameState`.
///
/// Reduces the two legacy StatLine arrays — `allGlobalSpeedStats`
/// (DR-enabled "normal" leg) and `allGlobalSpeedIgnoreDRStats` (DR-ignored
/// "immaculate" leg) — with [`product_f64`] and combines them through
/// [`calculate_global_speed_mult`] using the platonic-7 DR power. Replaces
/// the caller-provided `AutomationPre::global_time_multiplier`.
///
/// Three lines are neutral `1.0` pending unported inputs — each is exactly
/// `1.0` at the current play state, so this stays faithful now:
/// - obtainium-log line: `maxObtainium` is not tracked in state (and
///   `upgrades[70] == 0`);
/// - speed-spirit line: effective rune-spirit power needs the unported
///   `spiritMultiplier` chain (cf. the prism-spirit caveat);
/// - singularity-debuff line: the singularity layer is paused
///   (`calculate_singularity_debuff` has no production caller yet).
///
/// The event-buff line is `1.0` (no wall-clock event calendar in logic).
fn compute_global_speed_mult_pre(state: &GameState) -> f64 {
    use crate::mechanics::ant_upgrades::mortuus_ant_upgrade_effect;
    use crate::mechanics::calculate::{
        calculate_global_speed_mult, calculate_platonic_7_upgrade_power, product_f64,
        GlobalSpeedMultInput,
    };
    use crate::mechanics::cube_blessings::calculate_global_speed_cube_blessing;
    use crate::mechanics::golden_quark_upgrades::{intermediate_pack_effect, IntermediatePackKey};
    use crate::mechanics::hypercube_blessings::calculate_global_speed_hypercube_blessing;
    use crate::mechanics::octeracts::octeract_improved_global_speed_effect;
    use crate::mechanics::platonic_blessings::{
        calculate_global_speed_platonic_blessing,
        calculate_hypercube_blessing_multiplier_platonic_blessing,
    };
    use crate::mechanics::rune_blessing_effects::speed_rune_blessing_effects;
    use crate::mechanics::rune_effects::{speed_rune_effects, SpeedRuneKey};
    use crate::mechanics::shop_upgrades::shop_chronometer_s_effect;
    use crate::mechanics::singularity_challenges::{
        limited_time_effect, LimitedTimeKey, SingularityEffectValue,
    };
    use crate::mechanics::talisman_effects::chronos_talisman_effects;
    use crate::mechanics::tesseract_blessings::calculate_global_speed_tesseract_blessing;
    use crate::mechanics::{challenge_15_rewards, corruptions::dilation_multiplier_at_level};
    use crate::state::golden_quarks::GQ_INTERMEDIATE_PACK;
    use crate::state::octeract_upgrades::OCTERACT_IMPROVED_GLOBAL_SPEED;
    use crate::state::shop::SHOP_CHRONOMETER_S;
    use crate::state::{DILATION_INDEX, RUNE_SPEED, TALISMAN_CHRONOS};

    // Ant-upgrade index (legacy `AntUpgrades.Mortuus`) + the cube upgrades
    // touching global speed (legacy `player.cubeUpgrades[18|34|52]`).
    const ANT_UPGRADE_MORTUUS: usize = 11;
    const CUBE_UPGRADE_2X8: usize = 18;
    const CUBE_UPGRADE_GLOBAL_SPEED_BLESSING: usize = 34;
    const CUBE_UPGRADE_CX2: usize = 52;

    let sing = state.singularity.singularity_count;
    let researches = &state.researches.researches;
    let cube_upgrades = &state.cube_upgrade_levels.cube_upgrades;

    // Cube-blessing chain platonic → hypercube → tesseract → cube, mirroring
    // the legacy `calculateGlobalSpeedCubeBlessing` call chain in `Cubes.ts`.
    let platonic_amplifier =
        calculate_hypercube_blessing_multiplier_platonic_blessing(&state.platonic_blessings);
    let hypercube_blessing =
        calculate_global_speed_hypercube_blessing(&state.hypercube_blessings, platonic_amplifier);
    let tesseract_blessing =
        calculate_global_speed_tesseract_blessing(&state.tesseract_blessings, hypercube_blessing);
    let chronos_cube = calculate_global_speed_cube_blessing(
        &state.cube_blessings,
        tesseract_blessing,
        cube_upgrades[CUBE_UPGRADE_GLOBAL_SPEED_BLESSING],
    );

    let limited_time = match limited_time_effect(
        state.singularity.limited_time.completions,
        LimitedTimeKey::AscensionSpeed,
    ) {
        SingularityEffectValue::Scalar(v) => v,
        SingularityEffectValue::Unlock(_) => 1.0,
    };

    // DR-enabled ("normal") leg — legacy `allGlobalSpeedStats`.
    let normal_mult = product_f64(&[
        speed_rune_effects(
            state.runes.rune_levels[RUNE_SPEED],
            SpeedRuneKey::GlobalSpeed,
        ),
        1.0, // obtainium-log: maxObtainium untracked → 1.0 (upgrades[70] == 0)
        1.0 + researches[121] / 50.0,
        1.0 + 0.015 * researches[136],
        1.0 + 0.012 * researches[151],
        1.0 + 0.009 * researches[166],
        1.0 + 0.006 * researches[181],
        1.0 + 0.003 * researches[196],
        speed_rune_blessing_effects(state.runes.rune_blessing_levels[RUNE_SPEED]).global_speed,
        1.0, // speed spirit: effective spirit power unported → 1.0
        chronos_cube,
        1.0 + cube_upgrades[CUBE_UPGRADE_2X8] / 5.0,
        mortuus_ant_upgrade_effect(state.ants.upgrades[ANT_UPGRADE_MORTUUS]).global_speed,
        chronos_talisman_effects(state.talismans.talisman_rarity[TALISMAN_CHRONOS] as i32)
            .global_speed,
        challenge_15_rewards::global_speed(state.challenges.challenge15_exponent),
        1.0 + 0.01 * cube_upgrades[CUBE_UPGRADE_CX2],
        dilation_multiplier_at_level(state.corruptions.used.levels[DILATION_INDEX]),
    ]);

    // DR-ignored ("immaculate") leg — legacy `allGlobalSpeedIgnoreDRStats`.
    let immaculate_mult = product_f64(&[
        calculate_global_speed_platonic_blessing(&state.platonic_blessings),
        1.0, // singularity debuff: singularity layer paused → 1.0 (sing == 0)
        intermediate_pack_effect(
            state.golden_quarks.upgrades[GQ_INTERMEDIATE_PACK].level,
            IntermediatePackKey::GlobalSpeedMult,
        ),
        octeract_improved_global_speed_effect(
            state.octeract_upgrades.upgrades[OCTERACT_IMPROVED_GLOBAL_SPEED].level,
            sing,
        ),
        limited_time,
        shop_chronometer_s_effect(state.shop.upgrades[SHOP_CHRONOMETER_S], sing),
        1.0, // event buff: UI-tier (wall-clock event calendar) → 1.0
    ]);

    calculate_global_speed_mult(&GlobalSpeedMultInput {
        normal_mult,
        immaculate_mult,
        dr_power: calculate_platonic_7_upgrade_power(
            state.cube_upgrade_levels.platonic_upgrades[7],
        ),
    })
}

/// Ascension-speed multiplier (legacy `calculateAscensionSpeedMult`),
/// self-derived from `&GameState`.
///
/// Reduces the legacy `allAscensionSpeedStats` array with [`product_f64`]
/// into a `base`, then applies the exponent spread (the sum of GQ
/// `singAscensionSpeed`, `singAscensionSpeed2`, and shop
/// `chronometerInfinity`) via [`calculate_ascension_speed_mult`]. Replaces
/// the caller-provided `AutomationPre::ascension_speed_multi`. When the
/// `oneMind` GQ upgrade is unlocked the speed is a flat ×10 (legacy
/// `addTimers('ascension')`), bypassing the StatLine reduction.
///
/// Three lines are neutral `1.0` pending unported inputs — each is exactly
/// `1.0` at the current play state, so this stays faithful now: the shop
/// `panthema` line (needs the unported infinite-shop-upgrade bonus levels),
/// the singularity-debuff line (the singularity layer is paused), and the
/// event-buff line (UI-tier).
fn compute_ascension_speed_mult_pre(state: &GameState) -> f64 {
    use crate::mechanics::ant_upgrades::mortuus_2_ant_upgrade_effect;
    use crate::mechanics::calculate::{
        calculate_ascension_speed_exponent_spread, calculate_ascension_speed_mult, product_f64,
        AscensionSpeedMultInput,
    };
    use crate::mechanics::challenge_15_rewards;
    use crate::mechanics::exalt_penalties::{
        calculate_exalt_3_penalty, CalculateExalt3PenaltyInput,
    };
    use crate::mechanics::golden_quark_upgrades::{
        intermediate_pack_effect, one_mind_effect, sing_ascension_speed_2_effect,
        sing_ascension_speed_effect, IntermediatePackKey,
    };
    use crate::mechanics::hepteract_effects::chronos_hepteract_effects;
    use crate::mechanics::octeracts::{
        octeract_improved_ascension_speed_2_effect, octeract_improved_ascension_speed_effect,
    };
    use crate::mechanics::shop_upgrades::{
        chronometer_2_effect, chronometer_3_effect, chronometer_effect,
        chronometer_infinity_effect, chronometer_z_effect, shop_chronometer_s_effect,
        ChronometerInfinityKey,
    };
    use crate::mechanics::singularity_challenges::{
        limited_ascensions_effect, limited_time_effect, LimitedAscensionsKey, LimitedTimeKey,
        SingularityEffectValue,
    };
    use crate::mechanics::talisman_effects::polymath_talisman_effects;
    use crate::state::golden_quarks::{
        GQ_INTERMEDIATE_PACK, GQ_ONE_MIND, GQ_SING_ASCENSION_SPEED, GQ_SING_ASCENSION_SPEED_2,
    };
    use crate::state::octeract_upgrades::{
        OCTERACT_IMPROVED_ASCENSION_SPEED, OCTERACT_IMPROVED_ASCENSION_SPEED_2,
    };
    use crate::state::shop::{
        SHOP_CHRONOMETER, SHOP_CHRONOMETER_2, SHOP_CHRONOMETER_3, SHOP_CHRONOMETER_INFINITY,
        SHOP_CHRONOMETER_S, SHOP_CHRONOMETER_Z,
    };
    use crate::state::TALISMAN_POLYMATH;

    const ANT_UPGRADE_MORTUUS_2: usize = 15;
    const CUBE_UPGRADE_COOKIE_9: usize = 59;
    const PLATONIC_UPGRADE_OMEGA: usize = 15;

    let sing = state.singularity.singularity_count;
    let shop = &state.shop.upgrades;
    let gq = &state.golden_quarks.upgrades;
    let oct = &state.octeract_upgrades.upgrades;

    // `oneMind` locks ascension speed to a flat ×10, bypassing the StatLine
    // reduction entirely (legacy Helper.ts `addTimers('ascension')`).
    if one_mind_effect(gq[GQ_ONE_MIND].level) {
        return 10.0;
    }

    let scalar = |v: SingularityEffectValue| match v {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 1.0,
    };

    // Platonic OMEGA: 0.002 × (Σ used corruption levels) × platonicUpgrades[15].
    let corruption_total_levels: f64 = state
        .corruptions
        .used
        .levels
        .iter()
        .map(|&l| f64::from(l))
        .sum();
    let platonic_omega = 1.0
        + 0.002
            * corruption_total_levels
            * state.cube_upgrade_levels.platonic_upgrades[PLATONIC_UPGRADE_OMEGA];

    // EXALT limitedAscensions buff: effect ^ (1 + max(0, ⌊log10(ascensions)⌋)).
    let ascension_count = state.reset_counters.ascension_count;
    let limited_ascensions_mult = scalar(limited_ascensions_effect(
        state.singularity.limited_ascensions.completions,
        LimitedAscensionsKey::AscensionSpeedMult,
    ));
    let exalt_buff =
        limited_ascensions_mult.powf(1.0 + 0.0_f64.max(ascension_count.log10().floor()));

    // EXALT limitedAscensions debuff: 1 / Exalt-3 penalty (1 outside Exalt 3).
    let exalt_3_debuff = 1.0
        / calculate_exalt_3_penalty(&CalculateExalt3PenaltyInput {
            limited_ascensions_enabled: state.singularity.limited_ascensions.enabled,
            limited_ascensions_completions: state.singularity.limited_ascensions.completions,
            ascension_count,
        });

    // Base StatLine product — legacy `allAscensionSpeedStats`.
    let base = product_f64(&[
        mortuus_2_ant_upgrade_effect(state.ants.upgrades[ANT_UPGRADE_MORTUUS_2]).ascension_speed,
        polymath_talisman_effects(state.talismans.talisman_rarity[TALISMAN_POLYMATH] as i32)
            .ascension_speed_bonus,
        chronometer_effect(shop[SHOP_CHRONOMETER]),
        chronometer_2_effect(shop[SHOP_CHRONOMETER_2]),
        chronometer_3_effect(shop[SHOP_CHRONOMETER_3]),
        chronos_hepteract_effects(state.hepteracts.chronos.bal).ascension_speed,
        platonic_omega,
        challenge_15_rewards::ascension_speed(state.challenges.challenge15_exponent),
        1.0 + (1.0 / 400.0) * state.cube_upgrade_levels.cube_upgrades[CUBE_UPGRADE_COOKIE_9],
        intermediate_pack_effect(
            gq[GQ_INTERMEDIATE_PACK].level,
            IntermediatePackKey::AscensionSpeedMult,
        ),
        chronometer_z_effect(shop[SHOP_CHRONOMETER_Z], sing),
        octeract_improved_ascension_speed_effect(
            oct[OCTERACT_IMPROVED_ASCENSION_SPEED].level,
            sing,
        ),
        octeract_improved_ascension_speed_2_effect(
            oct[OCTERACT_IMPROVED_ASCENSION_SPEED_2].level,
            sing,
        ),
        chronometer_infinity_effect(
            shop[SHOP_CHRONOMETER_INFINITY],
            ChronometerInfinityKey::AscensionSpeedMult,
        ),
        exalt_buff,
        1.0, // shop panthema (Jack): infinite-shop-upgrade bonus levels unported → 1.0
        scalar(limited_time_effect(
            state.singularity.limited_time.completions,
            LimitedTimeKey::AscensionSpeed,
        )),
        shop_chronometer_s_effect(shop[SHOP_CHRONOMETER_S], sing),
        exalt_3_debuff,
        1.0, // singularity debuff: singularity layer paused → 1.0 (sing == 0)
        1.0, // event buff: UI-tier (wall-clock event calendar) → 1.0
    ]);

    let exponent_spread = calculate_ascension_speed_exponent_spread(
        sing_ascension_speed_effect(gq[GQ_SING_ASCENSION_SPEED].level),
        sing_ascension_speed_2_effect(gq[GQ_SING_ASCENSION_SPEED_2].level),
        chronometer_infinity_effect(
            shop[SHOP_CHRONOMETER_INFINITY],
            ChronometerInfinityKey::ExponentSpread,
        ),
    );

    calculate_ascension_speed_mult(&AscensionSpeedMultInput {
        base,
        exponent_spread,
    })
}

/// Singularity-speed multiplier, self-derived from `&GameState`.
///
/// Legacy Helper.ts `addTimers('singularity')` sets this to
/// `getAmbrosiaUpgradeEffects('ambrosiaBrickOfLead', 'singularitySpeedMult')`
/// — NOT a StatLine reduction — i.e. `1 - effectiveLevel / 100` (= 1 at
/// level 0). Replaces the caller-provided
/// `AutomationPre::singularity_speed_multi`.
///
/// Uses the stored effective level (`level + free_level`). The live
/// effective-level refinements — the red-ambrosia free-level chain
/// (`extraLevelCalc`) and the `noAmbrosiaUpgrades` / `sadisticPrequel` EXALT
/// gate that zeroes it — are not applied; both are inert at the current
/// play state, so this stays faithful now.
fn compute_singularity_speed_mult_pre(state: &GameState) -> f64 {
    use crate::mechanics::blueberry_upgrades::{
        ambrosia_brick_of_lead_effect, AmbrosiaBrickOfLeadEffectKey,
    };
    use crate::state::ambrosia::AMBROSIA_BRICK_OF_LEAD;

    let u = &state.ambrosia.upgrades[AMBROSIA_BRICK_OF_LEAD];
    ambrosia_brick_of_lead_effect(
        u.level + u.free_level,
        AmbrosiaBrickOfLeadEffectKey::SingularitySpeedMult,
    )
}

/// Pure-state non-speed `AutomationPre` timer fields, from the legacy
/// Helper.ts `addTimers('autoPotion' | 'goldenQuarks')` cases: the
/// offering/obtainium potion counts (`player.shopUpgrades.{offering,obtainium}Potion`),
/// the auto-potion speed (`octeractAutoPotionSpeed`), and the GQ export
/// rate (`goldenQuarks3`'s `exportGQPerHour`). Returned as
/// `(offering_potion_count, obtainium_potion_count, auto_potion_speed_mult,
/// export_gq_per_hour)`.
fn compute_auto_timer_fields(state: &GameState) -> (f64, f64, f64, f64) {
    use crate::mechanics::golden_quark_upgrades::golden_quarks_3_effect;
    use crate::mechanics::octeracts::octeract_auto_potion_speed_effect;
    use crate::state::golden_quarks::GQ_GOLDEN_QUARKS_3;
    use crate::state::octeract_upgrades::OCTERACT_AUTO_POTION_SPEED;
    use crate::state::shop::{SHOP_OBTAINIUM_POTION, SHOP_OFFERING_POTION};

    (
        state.shop.upgrades[SHOP_OFFERING_POTION],
        state.shop.upgrades[SHOP_OBTAINIUM_POTION],
        octeract_auto_potion_speed_effect(
            state.octeract_upgrades.upgrades[OCTERACT_AUTO_POTION_SPEED].level,
        ),
        golden_quarks_3_effect(state.golden_quarks.upgrades[GQ_GOLDEN_QUARKS_3].level),
    )
}

/// `G.TIME_PER_AMBROSIA` — base seconds per ambrosia bar (Variables.ts; a
/// fixed `G` constant, never reassigned in the legacy).
const TIME_PER_AMBROSIA: f64 = 45.0;

/// Pure-state ambrosia-timer threshold fields, from the legacy Helper.ts
/// `addTimers('ambrosia')` case: the EXALT-5 bonus-ambrosia grant
/// (`noAmbrosiaUpgrades.bonusAmbrosia`), the base `TIME_PER_AMBROSIA`
/// constant, the shop accelerator's point-requirement multiplier (scaled by
/// `noAmbrosiaUpgrades` completions), and the brick-of-lead bar-requirement
/// multiplier. Returned as `(bonus_ambrosia, time_per_ambrosia,
/// ambrosia_accelerator_mult, ambrosia_brick_of_lead_mult)`. The brick uses
/// the effective level (`level + free_level`). All four equal the old
/// `AutomationPre::default()` at the default state, so the sim/tests don't
/// shift.
fn compute_ambrosia_timer_fields(state: &GameState) -> (f64, f64, f64, f64) {
    use crate::mechanics::blueberry_upgrades::{
        ambrosia_brick_of_lead_effect, AmbrosiaBrickOfLeadEffectKey,
    };
    use crate::mechanics::shop_upgrades::shop_ambrosia_accelerator_effect;
    use crate::mechanics::singularity_challenges::{
        no_ambrosia_upgrades_effect, NoAmbrosiaUpgradesKey, SingularityEffectValue,
    };
    use crate::state::ambrosia::AMBROSIA_BRICK_OF_LEAD;
    use crate::state::shop::SHOP_AMBROSIA_ACCELERATOR;

    let no_amb = state.singularity.no_ambrosia_upgrades.completions;
    let bonus_ambrosia =
        match no_ambrosia_upgrades_effect(no_amb, NoAmbrosiaUpgradesKey::BonusAmbrosia) {
            SingularityEffectValue::Scalar(s) => s,
            SingularityEffectValue::Unlock(_) => 0.0,
        };
    let brick = &state.ambrosia.upgrades[AMBROSIA_BRICK_OF_LEAD];
    (
        bonus_ambrosia,
        TIME_PER_AMBROSIA,
        shop_ambrosia_accelerator_effect(state.shop.upgrades[SHOP_AMBROSIA_ACCELERATOR], no_amb),
        ambrosia_brick_of_lead_effect(
            brick.level + brick.free_level,
            AmbrosiaBrickOfLeadEffectKey::BarRequirementMult,
        ),
    )
}

/// Ambrosia-luck multiplier (legacy `calculateAmbrosiaLuck`), self-derived
/// from `&GameState`.
///
/// `raw_luck × multiplier`, where BOTH legs are **sums** (additive luck):
/// `raw_luck = Σ allAmbrosiaLuckStats` (base 100) and `multiplier =
/// Σ allAdditiveLuckMultStats` (base 1). Replaces the caller-provided
/// `AutomationPre::ambrosia_luck` (which the ambrosia timer consumes, gated
/// by `noSingularityUpgrades.completions > 0` — so it is inert at default).
///
/// Ambrosia/GQ/octeract effect inputs use the stored effective level
/// (`level + free_level`). Lines whose extra context is unported or
/// uncertain are neutral `0` (faithful at the current play state, where the
/// owning upgrade is `0` anyway): the planar-coin / campaign-luck / shop
/// `panthema` lines, the cube-/quark-luck synergy modules (need
/// `wow_cube_log_sum` / `worlds`), `ambrosiaLuck3` (needs the blueberry
/// inventory), `ambrosiaUltra` (needs the EXALT-completion sum), the
/// horseshoe rune/talisman lines, and the event buff (UI-tier).
fn compute_ambrosia_luck_pre(state: &GameState) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::blueberry_upgrades::{
        ambrosia_brick_of_lead_effect, ambrosia_luck_1_effect, ambrosia_luck_2_effect,
        ambrosia_luck_4_effect, AmbrosiaBrickOfLeadEffectKey,
    };
    use crate::mechanics::calculate::{calculate_ambrosia_luck, sum_f64};
    use crate::mechanics::golden_quark_upgrades::{
        sing_ambrosia_luck_2_effect, sing_ambrosia_luck_3_effect, sing_ambrosia_luck_4_effect,
        sing_ambrosia_luck_effect,
    };
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::octeracts::{
        octeract_ambrosia_luck_2_effect, octeract_ambrosia_luck_3_effect,
        octeract_ambrosia_luck_4_effect, octeract_ambrosia_luck_effect,
    };
    use crate::mechanics::red_ambrosia_bonuses::{
        calculate_cookie_upgrade_29_luck, CalculateCookieUpgrade29LuckInput,
    };
    use crate::mechanics::red_ambrosia_upgrades::{
        regular_luck_2_effect, regular_luck_effect, viscount_effect, ViscountEffectKey,
        ViscountEffectValue,
    };
    use crate::mechanics::shop_upgrades::{
        shop_ambrosia_luck_1_effect, shop_ambrosia_luck_2_effect, shop_ambrosia_luck_3_effect,
        shop_ambrosia_luck_4_effect, shop_ambrosia_luck_multiplier_4_effect,
        shop_octeract_ambrosia_luck_effect,
    };
    use crate::mechanics::singularity_challenges::{
        no_ambrosia_upgrades_effect, no_singularity_upgrades_effect, NoAmbrosiaUpgradesKey,
        NoSingularityUpgradesKey, SingularityEffectValue,
    };
    use crate::mechanics::singularity_milestones::{
        calculate_dilated_five_leaf_bonus, calculate_singularity_ambrosia_luck_milestone_bonus,
    };
    use crate::state::ambrosia::{
        AMBROSIA_BRICK_OF_LEAD, AMBROSIA_LUCK_1, AMBROSIA_LUCK_2, AMBROSIA_LUCK_4,
    };
    use crate::state::golden_quarks::{
        GQ_SING_AMBROSIA_LUCK, GQ_SING_AMBROSIA_LUCK_2, GQ_SING_AMBROSIA_LUCK_3,
        GQ_SING_AMBROSIA_LUCK_4,
    };
    use crate::state::octeract_upgrades::{
        OCTERACT_AMBROSIA_LUCK, OCTERACT_AMBROSIA_LUCK_2, OCTERACT_AMBROSIA_LUCK_3,
        OCTERACT_AMBROSIA_LUCK_4,
    };
    use crate::state::red_ambrosia::{
        RED_AMBROSIA_REGULAR_LUCK, RED_AMBROSIA_REGULAR_LUCK_2, RED_AMBROSIA_VISCOUNT,
    };
    use crate::state::shop::{
        SHOP_AMBROSIA_LUCK_1, SHOP_AMBROSIA_LUCK_2, SHOP_AMBROSIA_LUCK_3, SHOP_AMBROSIA_LUCK_4,
        SHOP_AMBROSIA_LUCK_MULTIPLIER_4, SHOP_OCTERACT_AMBROSIA_LUCK,
    };

    // Legacy `player.cubeUpgrades[77|79]` (Cookie 5 / Cookie 29 gate).
    const CUBE_UPGRADE_COOKIE_5: usize = 77;
    const CUBE_UPGRADE_COOKIE_29: usize = 79;

    let shop = &state.shop.upgrades;
    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let highest_sing = state.singularity.highest_singularity_count;
    let amb = |i: usize| state.ambrosia.upgrades[i].level + state.ambrosia.upgrades[i].free_level;
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let oct = |i: usize| {
        state.octeract_upgrades.upgrades[i].level + state.octeract_upgrades.upgrades[i].free_level
    };
    let red = |i: usize| state.red_ambrosia.upgrades[i].level;
    // Additive (luck) context → a missing singularity-effect value is 0.
    let sc = |v: SingularityEffectValue| match v {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 0.0,
    };

    let raw_luck = sum_f64(&[
        100.0, // Base
        0.0,   // PseudoCoins — planar-coin AMBROSIA_LUCK_BUFF (unported)
        get_level_reward(
            LevelRewardKey::AmbrosiaLuck,
            achievement_level_from_points(state.achievements.achievement_points),
        ),
        0.0, // Campaign — player.campaigns.ambrosiaLuckBonus (unported)
        calculate_singularity_ambrosia_luck_milestone_bonus(highest_sing),
        shop_ambrosia_luck_1_effect(shop[SHOP_AMBROSIA_LUCK_1]),
        shop_ambrosia_luck_2_effect(shop[SHOP_AMBROSIA_LUCK_2]),
        shop_ambrosia_luck_3_effect(shop[SHOP_AMBROSIA_LUCK_3]),
        shop_ambrosia_luck_4_effect(shop[SHOP_AMBROSIA_LUCK_4]),
        0.0, // Jack — shopPanthema (needs ShopPanthemaBonusLevels)
        sing_ambrosia_luck_effect(gq(GQ_SING_AMBROSIA_LUCK))
            + sing_ambrosia_luck_2_effect(gq(GQ_SING_AMBROSIA_LUCK_2))
            + sing_ambrosia_luck_3_effect(gq(GQ_SING_AMBROSIA_LUCK_3))
            + sing_ambrosia_luck_4_effect(gq(GQ_SING_AMBROSIA_LUCK_4)),
        octeract_ambrosia_luck_effect(oct(OCTERACT_AMBROSIA_LUCK))
            + octeract_ambrosia_luck_2_effect(oct(OCTERACT_AMBROSIA_LUCK_2))
            + octeract_ambrosia_luck_3_effect(oct(OCTERACT_AMBROSIA_LUCK_3))
            + octeract_ambrosia_luck_4_effect(oct(OCTERACT_AMBROSIA_LUCK_4)),
        ambrosia_luck_1_effect(amb(AMBROSIA_LUCK_1)),
        ambrosia_luck_2_effect(amb(AMBROSIA_LUCK_2), amb(AMBROSIA_LUCK_1)),
        0.0, // AmbrosiaLuck3 — needs the blueberry inventory
        0.0, // AmbrosiaCubeLuck1 — needs wow_cube_log_sum
        0.0, // AmbrosiaQuarkLuck1 — needs `worlds`
        if highest_sing >= 131.0 { 131.0 } else { 0.0 },
        if highest_sing >= 269.0 { 269.0 } else { 0.0 },
        shop_octeract_ambrosia_luck_effect(
            shop[SHOP_OCTERACT_AMBROSIA_LUCK],
            state.cube_balances.wow_octeracts.to_number(),
        ),
        sc(no_ambrosia_upgrades_effect(
            state.singularity.no_ambrosia_upgrades.completions,
            NoAmbrosiaUpgradesKey::AmbrosiaLuck,
        )),
        regular_luck_effect(red(RED_AMBROSIA_REGULAR_LUCK)),
        regular_luck_2_effect(red(RED_AMBROSIA_REGULAR_LUCK_2)),
        match viscount_effect(red(RED_AMBROSIA_VISCOUNT), ViscountEffectKey::LuckBonus) {
            ViscountEffectValue::Scalar(s) => s,
            ViscountEffectValue::RoleUnlock(_) => 0.0,
        },
        2.0 * cube[CUBE_UPGRADE_COOKIE_5],
        calculate_cookie_upgrade_29_luck(&CalculateCookieUpgrade29LuckInput {
            cube_upgrade_79: cube[CUBE_UPGRADE_COOKIE_29],
            lifetime_red_ambrosia: state.red_ambrosia.lifetime_red_ambrosia,
        }),
        0.0, // AmbrosiaUltra — shopAmbrosiaUltra (needs the EXALT-completion sum)
        0.0, // HorseShoeRune — horseshoe-rune level source unported
    ]);

    let multiplier = sum_f64(&[
        1.0, // Base
        sc(no_singularity_upgrades_effect(
            state.singularity.no_singularity_upgrades.completions,
            NoSingularityUpgradesKey::AdditiveLuckMult,
        )),
        calculate_dilated_five_leaf_bonus(highest_sing),
        shop_ambrosia_luck_multiplier_4_effect(shop[SHOP_AMBROSIA_LUCK_MULTIPLIER_4]),
        sc(no_ambrosia_upgrades_effect(
            state.singularity.no_ambrosia_upgrades.completions,
            NoAmbrosiaUpgradesKey::AdditiveLuckMult,
        )),
        0.001 * cube[CUBE_UPGRADE_COOKIE_5],
        ambrosia_luck_4_effect(
            amb(AMBROSIA_LUCK_4),
            state.red_ambrosia.lifetime_red_ambrosia,
            state.ambrosia.lifetime_ambrosia,
        ),
        ambrosia_brick_of_lead_effect(
            amb(AMBROSIA_BRICK_OF_LEAD),
            AmbrosiaBrickOfLeadEffectKey::AdditiveLuckMult,
        ),
        0.0, // HorseShoeTalisman — level source unported
        0.0, // Event buff — UI-tier
    ]);

    calculate_ambrosia_luck(raw_luck, multiplier)
}

/// Ambrosia generation speed (legacy `calculateAmbrosiaGenerationSpeed`),
/// self-derived from `&GameState`.
///
/// `raw_speed × blueberries`, where `raw_speed = Π allAmbrosiaGenerationSpeedStats`
/// (a **product** — the `Default` gate `0|1` × multipliers) and `blueberries
/// = Σ allAmbrosiaBlueberryStats` (a **sum** — additive blueberry count).
/// Replaces the caller-provided `AutomationPre::ambrosia_generation_speed`.
/// The `Default` gate is `0` until `noSingularityUpgrades.completions > 0`,
/// so this is exactly `0` at the default state (ambrosia locked) — matching
/// the old default.
///
/// Multiplicative lines whose context is unported are neutral `1.0`
/// (planar-coin, campaign bonus [campaign-token total not tracked], shop
/// `panthema`, patreon [quark-bonus arg], event); the additive blueberry
/// lines neutral `0`. All are inert at the current play state.
fn compute_ambrosia_generation_speed_pre(state: &GameState) -> f64 {
    use crate::mechanics::ambrosia::{
        calculate_number_of_thresholds, calculate_singularity_milestone_blueberries,
    };
    use crate::mechanics::calculate::{calculate_ambrosia_generation_speed, product_f64, sum_f64};
    use crate::mechanics::golden_quark_upgrades::{
        blueberries_effect as gq_blueberries_effect, sing_ambrosia_generation_2_effect,
        sing_ambrosia_generation_3_effect, sing_ambrosia_generation_4_effect,
        sing_ambrosia_generation_effect,
    };
    use crate::mechanics::octeracts::{
        octeract_ambrosia_generation_2_effect, octeract_ambrosia_generation_3_effect,
        octeract_ambrosia_generation_4_effect, octeract_ambrosia_generation_effect,
        octeract_blueberries_effect,
    };
    use crate::mechanics::red_ambrosia_upgrades::{
        blueberries_effect as red_blueberries_effect, blueberry_generation_speed_2_effect,
        blueberry_generation_speed_effect,
    };
    use crate::mechanics::shop_upgrades::{
        shop_ambrosia_generation_1_effect, shop_ambrosia_generation_2_effect,
        shop_ambrosia_generation_3_effect, shop_ambrosia_generation_4_effect,
        shop_cash_grab_ultra_effect, ShopCashGrabUltraKey,
    };
    use crate::mechanics::singularity_challenges::{
        no_ambrosia_upgrades_effect, one_challenge_cap_effect, NoAmbrosiaUpgradesKey,
        OneChallengeCapKey, SingularityEffectValue,
    };
    use crate::state::golden_quarks::{
        GQ_BLUEBERRIES, GQ_SING_AMBROSIA_GENERATION, GQ_SING_AMBROSIA_GENERATION_2,
        GQ_SING_AMBROSIA_GENERATION_3, GQ_SING_AMBROSIA_GENERATION_4,
    };
    use crate::state::octeract_upgrades::{
        OCTERACT_AMBROSIA_GENERATION, OCTERACT_AMBROSIA_GENERATION_2,
        OCTERACT_AMBROSIA_GENERATION_3, OCTERACT_AMBROSIA_GENERATION_4, OCTERACT_BLUEBERRIES,
    };
    use crate::state::red_ambrosia::{
        RED_AMBROSIA_BLUEBERRIES, RED_AMBROSIA_BLUEBERRY_GENERATION_SPEED,
        RED_AMBROSIA_BLUEBERRY_GENERATION_SPEED_2,
    };
    use crate::state::shop::{
        SHOP_AMBROSIA_GENERATION_1, SHOP_AMBROSIA_GENERATION_2, SHOP_AMBROSIA_GENERATION_3,
        SHOP_AMBROSIA_GENERATION_4, SHOP_CASH_GRAB_ULTRA,
    };

    const CUBE_UPGRADE_COOKIE_26: usize = 76; // legacy player.cubeUpgrades[76]

    let shop = &state.shop.upgrades;
    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let highest_sing = state.singularity.highest_singularity_count;
    let lifetime_amb = state.ambrosia.lifetime_ambrosia;
    let no_sing = state.singularity.no_singularity_upgrades.completions;
    let no_amb = state.singularity.no_ambrosia_upgrades.completions;
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let oct = |i: usize| {
        state.octeract_upgrades.upgrades[i].level + state.octeract_upgrades.upgrades[i].free_level
    };
    let red = |i: usize| state.red_ambrosia.upgrades[i].level;
    // Multiplicative context → a missing singularity-effect value is 1.
    let mc = |v: SingularityEffectValue| match v {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 1.0,
    };

    let raw_speed = product_f64(&[
        if no_sing > 0.0 { 1.0 } else { 0.0 }, // Default gate
        1.0,                                   // PseudoCoins (planar, unported)
        1.0,                                   // Campaign (token total not tracked)
        shop_ambrosia_generation_1_effect(shop[SHOP_AMBROSIA_GENERATION_1]),
        shop_ambrosia_generation_2_effect(shop[SHOP_AMBROSIA_GENERATION_2]),
        shop_ambrosia_generation_3_effect(shop[SHOP_AMBROSIA_GENERATION_3]),
        shop_ambrosia_generation_4_effect(shop[SHOP_AMBROSIA_GENERATION_4]),
        1.0, // Jack (panthema)
        sing_ambrosia_generation_effect(gq(GQ_SING_AMBROSIA_GENERATION))
            * sing_ambrosia_generation_2_effect(gq(GQ_SING_AMBROSIA_GENERATION_2))
            * sing_ambrosia_generation_3_effect(gq(GQ_SING_AMBROSIA_GENERATION_3))
            * sing_ambrosia_generation_4_effect(gq(GQ_SING_AMBROSIA_GENERATION_4)),
        octeract_ambrosia_generation_effect(oct(OCTERACT_AMBROSIA_GENERATION))
            * octeract_ambrosia_generation_2_effect(oct(OCTERACT_AMBROSIA_GENERATION_2))
            * octeract_ambrosia_generation_3_effect(oct(OCTERACT_AMBROSIA_GENERATION_3))
            * octeract_ambrosia_generation_4_effect(oct(OCTERACT_AMBROSIA_GENERATION_4)),
        1.0, // PatreonBonus (quark-bonus arg uncertain)
        mc(one_challenge_cap_effect(
            state.singularity.one_challenge_cap.completions,
            OneChallengeCapKey::BlueberrySpeedMult,
        )),
        mc(no_ambrosia_upgrades_effect(
            no_amb,
            NoAmbrosiaUpgradesKey::BlueberrySpeedMult,
        )),
        blueberry_generation_speed_effect(red(RED_AMBROSIA_BLUEBERRY_GENERATION_SPEED)),
        blueberry_generation_speed_2_effect(red(RED_AMBROSIA_BLUEBERRY_GENERATION_SPEED_2)),
        1.0 + 0.01 * cube[CUBE_UPGRADE_COOKIE_26] * calculate_number_of_thresholds(lifetime_amb),
        shop_cash_grab_ultra_effect(
            shop[SHOP_CASH_GRAB_ULTRA],
            ShopCashGrabUltraKey::AmbrosiaGenerationMult,
            lifetime_amb,
        ),
        1.0, // Event (UI-tier)
    ]);

    let blueberries = sum_f64(&[
        if no_sing > 0.0 { 3.0 } else { 0.0 }, // E1x1Clear
        gq_blueberries_effect(gq(GQ_BLUEBERRIES)),
        octeract_blueberries_effect(oct(OCTERACT_BLUEBERRIES)),
        red_blueberries_effect(red(RED_AMBROSIA_BLUEBERRIES)),
        calculate_singularity_milestone_blueberries(highest_sing),
        match no_ambrosia_upgrades_effect(no_amb, NoAmbrosiaUpgradesKey::Blueberries) {
            SingularityEffectValue::Scalar(s) => s,
            SingularityEffectValue::Unlock(_) => 0.0,
        },
    ]);

    calculate_ambrosia_generation_speed(raw_speed, blueberries)
}

/// Red-ambrosia luck (legacy `calculateRedAmbrosiaLuck`), self-derived from
/// `&GameState`.
///
/// `Σ allRedAmbrosiaLuckStats` (13 lines, additive, base 100). The
/// `LuckConversion` line composes on this tick's ambrosia luck:
/// `⌊(ambrosiaLuck − 100) / luckConversion⌋`, where `luckConversion =
/// Σ allLuckConversionStats` (base 20, with subtractive conversion-improvement
/// / shop-red-luck-ratio lines). Replaces the caller-provided
/// `AutomationPre::red_ambrosia_luck`, which the red-ambrosia timer consumes —
/// but that timer is gated by `noAmbrosiaUpgrades.completions > 0`, so the
/// value is inert at default, where this self-derives to the base `100` (vs
/// the old default `0` — the same harmless gap as `ambrosia_luck`).
///
/// Neutral `0` lines (faithful at default; the owning upgrade is `0` anyway):
/// the planar-coin `RED_LUCK_BUFF` (unported), shop `panthema` (Jack — needs
/// `ShopPanthemaBonusLevels`), and the horseshoe rune/talisman lines (level
/// source unported — same precedent as [`compute_ambrosia_luck_pre`]).
fn compute_red_ambrosia_luck_pre(state: &GameState, ambrosia_luck: f64) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::calculate::sum_f64;
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::red_ambrosia_upgrades::{
        conversion_improvement_1_effect, conversion_improvement_2_effect,
        conversion_improvement_3_effect, red_luck_effect, viscount_effect, ViscountEffectKey,
        ViscountEffectValue,
    };
    use crate::mechanics::shop_upgrades::{
        shop_red_luck_1_effect, shop_red_luck_2_effect, shop_red_luck_3_effect, ShopRedLuckKey,
    };
    use crate::mechanics::singularity_challenges::{
        no_ambrosia_upgrades_effect, NoAmbrosiaUpgradesKey, SingularityEffectValue,
    };
    use crate::state::red_ambrosia::{
        RED_AMBROSIA_CONVERSION_IMPROVEMENT_1, RED_AMBROSIA_CONVERSION_IMPROVEMENT_2,
        RED_AMBROSIA_CONVERSION_IMPROVEMENT_3, RED_AMBROSIA_RED_LUCK, RED_AMBROSIA_VISCOUNT,
    };
    use crate::state::shop::{SHOP_RED_LUCK_1, SHOP_RED_LUCK_2, SHOP_RED_LUCK_3};

    let shop = &state.shop.upgrades;
    let red = |i: usize| state.red_ambrosia.upgrades[i].level;
    let no_amb = state.singularity.no_ambrosia_upgrades.completions;
    // Additive (luck) context → a missing singularity-effect value is 0.
    let sc = |v: SingularityEffectValue| match v {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 0.0,
    };

    // calculateLuckConversion() — Σ allLuckConversionStats (base 20). The
    // conversion-improvement (`-n`) and shop-red-luck ratio lines are
    // subtractive; the horseshoe-rune line's level source is unported.
    let luck_conversion = sum_f64(&[
        20.0, // Base
        conversion_improvement_1_effect(red(RED_AMBROSIA_CONVERSION_IMPROVEMENT_1)),
        conversion_improvement_2_effect(red(RED_AMBROSIA_CONVERSION_IMPROVEMENT_2)),
        conversion_improvement_3_effect(red(RED_AMBROSIA_CONVERSION_IMPROVEMENT_3)),
        shop_red_luck_1_effect(shop[SHOP_RED_LUCK_1], ShopRedLuckKey::LuckConversionRatio),
        shop_red_luck_2_effect(shop[SHOP_RED_LUCK_2], ShopRedLuckKey::LuckConversionRatio),
        shop_red_luck_3_effect(shop[SHOP_RED_LUCK_3], ShopRedLuckKey::LuckConversionRatio),
        0.0, // HorseShoeRune — redLuckConversion, level source unported
    ]);

    sum_f64(&[
        100.0, // Base
        0.0,   // PseudoCoins — planar-coin RED_LUCK_BUFF (unported)
        get_level_reward(
            LevelRewardKey::RedAmbrosiaLuck,
            achievement_level_from_points(state.achievements.achievement_points),
        ),
        ((ambrosia_luck - 100.0) / luck_conversion).floor(), // LuckConversion
        red_luck_effect(red(RED_AMBROSIA_RED_LUCK)),
        sc(no_ambrosia_upgrades_effect(
            no_amb,
            NoAmbrosiaUpgradesKey::RedLuck,
        )),
        shop_red_luck_1_effect(shop[SHOP_RED_LUCK_1], ShopRedLuckKey::RedLuck),
        shop_red_luck_2_effect(shop[SHOP_RED_LUCK_2], ShopRedLuckKey::RedLuck),
        shop_red_luck_3_effect(shop[SHOP_RED_LUCK_3], ShopRedLuckKey::RedLuck),
        0.0, // Jack — shopPanthema (needs ShopPanthemaBonusLevels)
        match viscount_effect(red(RED_AMBROSIA_VISCOUNT), ViscountEffectKey::RedLuckBonus) {
            ViscountEffectValue::Scalar(s) => s,
            ViscountEffectValue::RoleUnlock(_) => 0.0,
        },
        0.0, // HorseShoeRune — redLuck, level source unported
        0.0, // HorseShoeTalisman — redLuck, level source unported
    ])
}

/// Red-ambrosia generation speed (legacy `calculateRedAmbrosiaGenerationSpeed`),
/// self-derived from `&GameState`.
///
/// `Π allRedAmbrosiaGenerationSpeedStats` (5 lines, multiplicative). The
/// `Base` line gates on `noAmbrosiaUpgrades.completions > 0` (`0` otherwise),
/// and the `BlueberrySpeed` line wraps this tick's ambrosia generation speed
/// `b` (`b > 1000 ? √(b·1000) : b`). Both are `0` at default (ambrosia
/// locked → `ambrosia_generation_speed` is `0`), so this self-derives to
/// exactly `0`, matching the old `AutomationPre::default()`. Replaces the
/// caller-provided `AutomationPre::red_ambrosia_generation_speed`.
///
/// Neutral `1` line: the planar-coin `RED_GENERATION_BUFF` (unported).
fn compute_red_ambrosia_generation_speed_pre(
    state: &GameState,
    ambrosia_generation_speed: f64,
) -> f64 {
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::red_ambrosia_upgrades::red_generation_speed_effect;
    use crate::mechanics::singularity_challenges::{
        no_ambrosia_upgrades_effect, NoAmbrosiaUpgradesKey, SingularityEffectValue,
    };
    use crate::state::red_ambrosia::RED_AMBROSIA_RED_GENERATION_SPEED;

    let no_amb = state.singularity.no_ambrosia_upgrades.completions;
    // Multiplicative context → a missing singularity-effect value is 1.
    let mc = |v: SingularityEffectValue| match v {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 1.0,
    };

    let blueberry_speed = if ambrosia_generation_speed > 1000.0 {
        (ambrosia_generation_speed * 1000.0).sqrt()
    } else {
        ambrosia_generation_speed
    };

    product_f64(&[
        if no_amb > 0.0 { 1.0 } else { 0.0 }, // Base gate
        1.0,                                  // PseudoCoins — planar RED_GENERATION_BUFF (unported)
        blueberry_speed,                      // BlueberrySpeed
        red_generation_speed_effect(
            state.red_ambrosia.upgrades[RED_AMBROSIA_RED_GENERATION_SPEED].level,
        ),
        mc(no_ambrosia_upgrades_effect(
            no_amb,
            NoAmbrosiaUpgradesKey::RedSpeedMult,
        )),
    ])
}

/// Compute the per-tick reset-currency point gains (prestige / transcend /
/// reincarnation) from `&GameState` plus the Phase-2 accelerator effect.
///
/// Mirrors the legacy `resetCurrency()` shim. Its three outputs feed both
/// [`compute_resource_gain_pre`] (the coin/shard → point conversion that
/// `resource_gain` credits) and `AutomationPre` (the auto-reset
/// amount-mode thresholds in [`auto_reset`]). Pure — no state mutation.
fn compute_reset_currency_gains(
    state: &GameState,
    agg: &AggregatorOutputs,
) -> crate::mechanics::reset_currency::ResetCurrencyResult {
    use crate::mechanics::challenges::{calc_ecc, ChallengeType};
    use crate::mechanics::corruptions::deflation_multiplier_at_level;
    use crate::mechanics::reset_currency::{reset_currency, ResetCurrencyInput};
    use crate::state::DEFLATION_INDEX;

    let ach = achievement_reward_input(state);
    let challenges = &state.challenges;
    reset_currency(&ResetCurrencyInput {
        ecc5: calc_ecc(
            ChallengeType::Transcend,
            challenges.challenge_completions[5],
        ),
        transcension_challenge: challenges.current_transcension_challenge,
        reincarnation_challenge: challenges.current_reincarnation_challenge,
        ascension_challenge: challenges.current_ascension_challenge,
        deflation_multiplier: deflation_multiplier_at_level(
            state.corruptions.used.levels[DEFLATION_INDEX],
        ),
        coins_this_prestige: state.coin_counters.coins_this_prestige,
        coins_this_transcension: state.coin_counters.coins_this_transcension,
        transcend_shards: state.reset_counters.transcend_shards,
        upgrade_16: f64::from(state.upgrades.upgrades[16]),
        upgrade_44: f64::from(state.upgrades.upgrades[44]),
        upgrade_65: f64::from(state.upgrades.upgrades[65]),
        transcend_count: state.reset_counters.transcend_count,
        accelerator_effect: agg.update_all_tick.accelerator_effect,
        particle_gain_reward: achievement_rewards::particle_gain(&ach),
    })
}

/// Legacy `player.{first..fifth}ProduceDiamonds` — immutable per-tier
/// prestige-producer base scalars. Like the coin scalars these are never
/// reassigned in the legacy, so hoisted as a constant. The cascade
/// formula that turns them into `G.produce*Diamonds` lives in
/// [`resource_gain`]; this slice only carries the base rate.
const DIAMOND_PRODUCE_SCALARS: [f64; 5] = [0.05, 0.0005, 0.00005, 0.000005, 0.000005];
/// Legacy `player.{first..fifth}ProduceMythos` — transcend-producer base.
const MYTHOS_PRODUCE_SCALARS: [f64; 5] = [1.0, 0.01, 0.001, 0.0002, 0.00004];
/// Legacy `player.{first..fifth}ProduceParticles` — reincarnation base.
const PARTICLE_PRODUCE_SCALARS: [f64; 5] = [0.25, 0.2, 0.15, 0.1, 0.5];

/// State + aggregator-output-derive the [`ResourceGainPre`] fields.
///
/// Migration coverage today (`✓` = derived from state / aggregator
/// outputs / constants, `forwarded` = caller-provided fallback):
/// - `global_crystal_multiplier`        ✓ from GlobalMultipliersResult
/// - `global_mythos_multiplier`         ✓ from GlobalMultipliersResult
/// - `grandmaster_multiplier`           ✓ from GlobalMultipliersResult
/// - `mythosupgrade_13`                 ✓ from GlobalMultipliersResult
/// - `mythosupgrade_14`                 ✓ from GlobalMultipliersResult
/// - `mythosupgrade_15`                 ✓ from GlobalMultipliersResult
/// - `global_constant_mult`             ✓ from GlobalMultipliersResult
/// - `challenge_base_requirements`      ✓ static legacy constant
/// - `produce_total`                    ✓ from [`phase_tax`] (coin production)
/// - `taxdivisor`                       ✓ from [`phase_tax`] (fresh this tick)
/// - `taxdivisorcheck`                  ✓ from [`phase_tax`]
/// - `maxexponent`                      ✓ from [`phase_tax`]
/// - `{first..fifth}_produce_{diamonds,mythos,particles}` ✓ immutable base scalars
/// - `{prestige,transcend,reincarnation}_point_gain` ✓ from [`compute_reset_currency_gains`]
///
/// Every field is now derived — `ResourceGainPre` no longer reads any
/// caller fallback.
#[must_use]
fn compute_resource_gain_pre(
    agg: &AggregatorOutputs,
    tax: &TaxOutputs,
    reset: &crate::mechanics::reset_currency::ResetCurrencyResult,
) -> ResourceGainPre {
    /// Verbatim port of the legacy `G.challengeBaseRequirements` const.
    /// Static lookup; no state read.
    const CHALLENGE_BASE_REQUIREMENTS: [f64; 5] = [10.0, 100.0, 1_000.0, 10_000.0, 100_000.0];

    let g = &agg.global_multipliers;
    ResourceGainPre {
        // From Phase 2 aggregator outputs.
        global_crystal_multiplier: g.global_crystal_multiplier,
        global_mythos_multiplier: g.global_mythos_multiplier,
        grandmaster_multiplier: g.grandmaster_multiplier,
        mythosupgrade_13: g.mythosupgrade_13,
        mythosupgrade_14: g.mythosupgrade_14,
        mythosupgrade_15: g.mythosupgrade_15,
        global_constant_mult: g.global_constant_mult,
        // Static legacy constant.
        challenge_base_requirements: CHALLENGE_BASE_REQUIREMENTS,
        // From the tax phase (coin production + tax exponent/divisor).
        produce_total: tax.produce_total,
        taxdivisor: tax.taxdivisor,
        taxdivisorcheck: tax.taxdivisorcheck,
        maxexponent: tax.maxexponent,
        // Immutable per-tier producer base scalars (legacy player
        // constants); the cascade math lives in `resource_gain`.
        first_produce_diamonds: DIAMOND_PRODUCE_SCALARS[0],
        second_produce_diamonds: DIAMOND_PRODUCE_SCALARS[1],
        third_produce_diamonds: DIAMOND_PRODUCE_SCALARS[2],
        fourth_produce_diamonds: DIAMOND_PRODUCE_SCALARS[3],
        fifth_produce_diamonds: DIAMOND_PRODUCE_SCALARS[4],
        first_produce_mythos: MYTHOS_PRODUCE_SCALARS[0],
        second_produce_mythos: MYTHOS_PRODUCE_SCALARS[1],
        third_produce_mythos: MYTHOS_PRODUCE_SCALARS[2],
        fourth_produce_mythos: MYTHOS_PRODUCE_SCALARS[3],
        fifth_produce_mythos: MYTHOS_PRODUCE_SCALARS[4],
        first_produce_particles: PARTICLE_PRODUCE_SCALARS[0],
        second_produce_particles: PARTICLE_PRODUCE_SCALARS[1],
        third_produce_particles: PARTICLE_PRODUCE_SCALARS[2],
        fourth_produce_particles: PARTICLE_PRODUCE_SCALARS[3],
        fifth_produce_particles: PARTICLE_PRODUCE_SCALARS[4],
        // From reset_currency (prestige / transcend / reincarnation gains).
        prestige_point_gain: reset.prestige_point_gain,
        transcend_point_gain: reset.transcend_point_gain,
        reincarnation_point_gain: reset.reincarnation_point_gain,
    }
}

/// **Phase 3** — Player input drain.
///
/// Each queued [`PlayerAction`] dispatches into its corresponding `buy_*`
/// mutator. Events flow into [`TickOutput::events`].
fn phase_player_input(state: &mut GameState, input: &TackInput, output: &mut TickOutput) {
    for action in &input.player_actions {
        match action {
            PlayerAction::Buy(req) => {
                let events = dispatch_buy(state, req);
                output.events.extend(events);
            }
        }
    }
}

/// **Phase 4** — Resource generation + challenge auto-completion.
///
/// Calls [`resource_gain`] with the tick-local `resource_gain_pre` bundle
/// (now fully derived by [`compute_resource_gain_pre`]) and writes the
/// result back into the corresponding [`GameState`] slices. Events emitted
/// by `resource_gain` (achievement awards, challenge auto-completions)
/// flow into [`TickOutput::events`].
///
/// Per Ledger Finding 1, the currency fields now have a single
/// source-of-truth in `state.upgrades`; `buy_*` mutators read/write them
/// through `&mut Decimal` parameters rather than via per-slice
/// duplicates. No mid-tick sync workaround is needed.
fn phase_generation(
    state: &mut GameState,
    pre: &ResourceGainPre,
    dt: f64,
    output: &mut TickOutput,
) {
    let result = resource_gain(state, pre, dt);

    // ─── Canonical writeback (state.upgrades, state.coin_counters) ───────
    state.upgrades.coins = result.coins;
    state.upgrades.prestige_points = result.prestige_points;
    state.upgrades.transcend_points = result.transcend_points;
    state.upgrades.reincarnation_points = result.reincarnation_points;

    state.coin_counters.coins_this_prestige = result.coins_this_prestige;
    state.coin_counters.coins_this_transcension = result.coins_this_transcension;
    state.coin_counters.coins_this_reincarnation = result.coins_this_reincarnation;
    state.coin_counters.coins_total = result.coins_total;

    // ─── Shard writeback (per-slice canonical locations) ─────────────────
    state.crystal_upgrades.prestige_shards = result.prestige_shards;
    state.reset_counters.transcend_shards = result.transcend_shards;
    state.reset_counters.reincarnation_shards = result.reincarnation_shards;
    state.campaigns.ascend_shards = result.ascend_shards;

    // ─── Generated counters (tier 1..4; tier 5 is terminal) ──────────────
    state.diamond_producers.tiers[0].generated = result.first_generated_diamonds;
    state.diamond_producers.tiers[1].generated = result.second_generated_diamonds;
    state.diamond_producers.tiers[2].generated = result.third_generated_diamonds;
    state.diamond_producers.tiers[3].generated = result.fourth_generated_diamonds;

    state.mythos_producers.tiers[0].generated = result.first_generated_mythos;
    state.mythos_producers.tiers[1].generated = result.second_generated_mythos;
    state.mythos_producers.tiers[2].generated = result.third_generated_mythos;
    state.mythos_producers.tiers[3].generated = result.fourth_generated_mythos;

    state.particle_producers.tiers[0].generated = result.first_generated_particles;
    state.particle_producers.tiers[1].generated = result.second_generated_particles;
    state.particle_producers.tiers[2].generated = result.third_generated_particles;
    state.particle_producers.tiers[3].generated = result.fourth_generated_particles;

    state.tesseract_buildings.ascend_building_1.generated = result.ascend_building_1_generated;
    state.tesseract_buildings.ascend_building_2.generated = result.ascend_building_2_generated;
    state.tesseract_buildings.ascend_building_3.generated = result.ascend_building_3_generated;
    state.tesseract_buildings.ascend_building_4.generated = result.ascend_building_4_generated;

    // ─── Challenge completions (c1..=c5 advance via auto-completion) ─────
    state.challenges.challenge_completions[1] = result.c1_completions;
    state.challenges.challenge_completions[2] = result.c2_completions;
    state.challenges.challenge_completions[3] = result.c3_completions;
    state.challenges.challenge_completions[4] = result.c4_completions;
    state.challenges.challenge_completions[5] = result.c5_completions;

    // ─── Events ──────────────────────────────────────────────────────────
    output.events.extend(result.events);
}

/// **Phase 5** — Automation (head / middle / tail).
///
/// Mirrors the legacy `tackBody`: the **head** (ant generation + the 11
/// `addTimers` cases) and **middle** (rune / ant sacrifice, addObtainium,
/// auto-research) run only on live ticks — skipped when
/// [`TackInput::time_warp`] is true — while the **tail** (addOfferings,
/// challenge sweep, auto-reset) always runs so offline catch-up still
/// resets and accrues offerings.
///
/// Cross-mechanic multipliers + unlock gates with no ported aggregator
/// yet arrive via [`AutomationPre`] (caller pre-evaluated); each emitted
/// [`CoreEvent`] is an intent the UI tier turns into the matching side
/// effect.
fn phase_automation(
    state: &mut GameState,
    cache: &CrossMechanicCache,
    input: &TackInput,
    output: &mut TickOutput,
) {
    let pre = &cache.automation_pre;
    let dt = input.dt;

    // Head, middle, and ant generation only run on live ticks; the tail
    // runs unconditionally below (mirroring the legacy `tackBody`).
    if !input.time_warp {
        // ── Generation: ant producers + crumbs (no event) ───────────
        let ant =
            ant_generation::generate_ants_and_crumbs(&ant_generation::GenerateAntsAndCrumbsInput {
                dt,
                ant_speed_mult: pre.ant_speed_mult,
                producers: &state.ants.producers,
                masteries: &state.ants.masteries,
                crumbs: state.ants.crumbs,
                crumbs_this_sacrifice: state.ants.crumbs_this_sacrifice,
                crumbs_ever_made: state.ants.crumbs_ever_made,
            });
        for (tier, generated) in state.ants.producers.iter_mut().zip(ant.producers_generated) {
            tier.generated = generated;
        }
        state.ants.crumbs = ant.crumbs;
        state.ants.crumbs_this_sacrifice = ant.crumbs_this_sacrifice;
        state.ants.crumbs_ever_made = ant.crumbs_ever_made;

        // ── Head: simple counters (no events) ────────────────────────────
        state.reset_counters.prestige_counter = timers::advance_reset_counter(
            state.reset_counters.prestige_counter,
            dt,
            pre.global_time_multiplier,
        );
        state.reset_counters.transcend_counter = timers::advance_reset_counter(
            state.reset_counters.transcend_counter,
            dt,
            pre.global_time_multiplier,
        );
        state.reset_counters.reincarnation_counter = timers::advance_reset_counter(
            state.reset_counters.reincarnation_counter,
            dt,
            pre.global_time_multiplier,
        );

        let asc = timers::advance_ascension_timer(&timers::AdvanceAscensionTimerInput {
            dt,
            ascension_counter: state.reset_counters.ascension_counter,
            ascension_counter_real: state.reset_counters.ascension_counter_real,
            ascension_speed_multi: pre.ascension_speed_multi,
        });
        state.reset_counters.ascension_counter = asc.ascension_counter;
        state.reset_counters.ascension_counter_real = asc.ascension_counter_real;

        let sing = timers::advance_singularity_timer(&timers::AdvanceSingularityTimerInput {
            dt,
            ascension_counter_real_real: state.reset_counters.ascension_counter_real_real,
            singularity_counter: state.singularity.singularity_counter,
            sing_challenge_timer: state.singularity.sing_challenge_timer,
            inside_singularity_challenge: inside_singularity_challenge(&state.singularity),
            singularity_speed_multi: pre.singularity_speed_multi,
        });
        state.reset_counters.ascension_counter_real_real = sing.ascension_counter_real_real;
        state.singularity.singularity_counter = sing.singularity_counter;
        state.singularity.sing_challenge_timer = sing.sing_challenge_timer;

        state.quarks.quarks_timer =
            timers::advance_quarks_timer(&timers::AdvanceQuarksTimerInput {
                dt,
                quarks_timer: state.quarks.quarks_timer,
                max_quark_timer: pre.max_quark_timer,
            });

        state.golden_quarks.golden_quarks_timer =
            timers::advance_golden_quarks_timer(&timers::AdvanceGoldenQuarksTimerInput {
                dt,
                golden_quarks_timer: state.golden_quarks.golden_quarks_timer,
                export_gq_per_hour: pre.export_gq_per_hour,
            });

        // ── Head: octeract timer (emits OcteractTickFired) ───────────────
        // `time_multiplier` is 1.0 here (legacy octeract case is in the
        // `timeMultiplier === 1` list). The GQ-giveaway loop above singularity
        // 160 writes `golden_quarks` + decays `quarks_this_singularity`.
        let oct = timers::advance_octeract_timer(&timers::AdvanceOcteractTimerInput {
            dt,
            time_multiplier: 1.0,
            octeract_unlocked: pre.octeract_unlocked,
            octeract_timer: state.octeract_upgrades.octeract_timer,
            wow_octeracts: state.cube_balances.wow_octeracts,
            total_wow_octeracts: state.cube_balances.total_wow_octeracts,
            golden_quarks: state.golden_quarks.golden_quarks,
            quarks_this_singularity: state.golden_quarks.quarks_this_singularity,
            per_second: pre.octeract_per_second,
            highest_singularity_count: state.singularity.highest_singularity_count,
            singularity_count: state.singularity.singularity_count,
            golden_quarks_multiplier_excluding_base: pre.golden_quarks_multiplier_excluding_base,
        });
        state.octeract_upgrades.octeract_timer = oct.octeract_timer;
        state.cube_balances.wow_octeracts = oct.wow_octeracts;
        state.cube_balances.total_wow_octeracts = oct.total_wow_octeracts;
        state.golden_quarks.golden_quarks = oct.golden_quarks;
        state.golden_quarks.quarks_this_singularity = oct.quarks_this_singularity;
        output.events.extend(oct.events);

        // ── Head: auto-potion timers (emit AutoPotionFired) ──────────────
        // Toggles + accumulators live in `state.automation`; the potion
        // counts + speed mult are caller pre-evals (shop-slot reads).
        let pot = timers::advance_auto_potion_timer(&timers::AdvanceAutoPotionTimerInput {
            dt,
            time_multiplier: 1.0,
            highest_singularity_count: state.singularity.highest_singularity_count,
            auto_potion_timer: state.automation.auto_potion_timer,
            auto_potion_timer_obtainium: state.automation.auto_potion_timer_obtainium,
            toggle_offering: state.automation.auto_potion_toggle_offering,
            toggle_obtainium: state.automation.auto_potion_toggle_obtainium,
            offering_potion_count: pre.offering_potion_count,
            obtainium_potion_count: pre.obtainium_potion_count,
            auto_potion_speed_mult: pre.auto_potion_speed_mult,
        });
        state.automation.auto_potion_timer = pot.auto_potion_timer;
        state.automation.auto_potion_timer_obtainium = pot.auto_potion_timer_obtainium;
        output.events.extend(pot.events);

        // ── Head: ambrosia timer (emits AmbrosiaGained) ──────────────────
        let amb = timers::advance_ambrosia_timer(
            &timers::AdvanceAmbrosiaTimerInput {
                dt,
                time_multiplier: 1.0,
                no_singularity_upgrades_completions: state
                    .singularity
                    .no_singularity_upgrades
                    .completions,
                ambrosia_generation_speed: pre.ambrosia_generation_speed,
                ambrosia_timer_g: state.ambrosia.ambrosia_timer_g,
                blueberry_time: state.ambrosia.blueberry_time,
                ambrosia: state.ambrosia.ambrosia,
                lifetime_ambrosia: state.ambrosia.lifetime_ambrosia,
                ambrosia_luck: pre.ambrosia_luck,
                bonus_ambrosia: pre.bonus_ambrosia,
                time_per_ambrosia: pre.time_per_ambrosia,
                accelerator_mult: pre.ambrosia_accelerator_mult,
                brick_of_lead_mult: pre.ambrosia_brick_of_lead_mult,
            },
            state.rng.draw(RngPurpose::Ambrosia),
        );
        state.ambrosia.ambrosia_timer_g = amb.ambrosia_timer_g;
        state.ambrosia.blueberry_time = amb.blueberry_time;
        state.ambrosia.ambrosia = amb.ambrosia;
        state.ambrosia.lifetime_ambrosia = amb.lifetime_ambrosia;
        output.events.extend(amb.events);

        // ── Head: red-ambrosia timer (emits RedAmbrosiaGained) ───────────
        let red = timers::advance_red_ambrosia_timer(
            &timers::AdvanceRedAmbrosiaTimerInput {
                dt,
                time_multiplier: 1.0,
                no_ambrosia_upgrades_completions: state
                    .singularity
                    .no_ambrosia_upgrades
                    .completions,
                red_ambrosia_generation_speed: pre.red_ambrosia_generation_speed,
                red_ambrosia_timer_g: state.red_ambrosia.red_ambrosia_timer_g,
                red_ambrosia_time: state.red_ambrosia.red_ambrosia_time,
                red_ambrosia: state.red_ambrosia.red_ambrosia,
                lifetime_red_ambrosia: state.red_ambrosia.lifetime_red_ambrosia,
                red_ambrosia_luck: pre.red_ambrosia_luck,
                ambrosia_time_per_red_ambrosia: pre.ambrosia_time_per_red_ambrosia,
                time_per_red_ambrosia: pre.time_per_red_ambrosia,
                bar_requirement_multiplier: pre.red_ambrosia_bar_requirement_multiplier,
            },
            state.rng.draw(RngPurpose::RedAmbrosia),
        );
        state.red_ambrosia.red_ambrosia_timer_g = red.red_ambrosia_timer_g;
        state.red_ambrosia.red_ambrosia_time = red.red_ambrosia_time;
        state.red_ambrosia.red_ambrosia = red.red_ambrosia;
        state.red_ambrosia.lifetime_red_ambrosia = red.lifetime_red_ambrosia;
        output.events.extend(red.events);

        // ── Head 11b: red→ambrosia bonus-time feedback ───────────────────
        // Mirrors the legacy `addTimers('ambrosia', bonusAmbrosiaTime)` shim:
        // re-enter the ambrosia timer with the bonus time as `dt`, continuing
        // from the post-case-10 ambrosia state + RNG stream.
        if red.bonus_ambrosia_time > 0.0 {
            let bonus = timers::advance_ambrosia_timer(
                &timers::AdvanceAmbrosiaTimerInput {
                    dt: red.bonus_ambrosia_time,
                    time_multiplier: 1.0,
                    no_singularity_upgrades_completions: state
                        .singularity
                        .no_singularity_upgrades
                        .completions,
                    ambrosia_generation_speed: pre.ambrosia_generation_speed,
                    ambrosia_timer_g: state.ambrosia.ambrosia_timer_g,
                    blueberry_time: state.ambrosia.blueberry_time,
                    ambrosia: state.ambrosia.ambrosia,
                    lifetime_ambrosia: state.ambrosia.lifetime_ambrosia,
                    ambrosia_luck: pre.ambrosia_luck,
                    bonus_ambrosia: pre.bonus_ambrosia,
                    time_per_ambrosia: pre.time_per_ambrosia,
                    accelerator_mult: pre.ambrosia_accelerator_mult,
                    brick_of_lead_mult: pre.ambrosia_brick_of_lead_mult,
                },
                state.rng.draw(RngPurpose::Ambrosia),
            );
            state.ambrosia.ambrosia_timer_g = bonus.ambrosia_timer_g;
            state.ambrosia.blueberry_time = bonus.blueberry_time;
            state.ambrosia.ambrosia = bonus.ambrosia;
            state.ambrosia.lifetime_ambrosia = bonus.lifetime_ambrosia;
            output.events.extend(bonus.events);
        }

        // ─── Middle (tackMiddle) ─────────────────────────────────────────
        // 1. Rune sacrifice — gate = persisted toggle AND the shop effect.
        if state.automation.rune_sacrifice_auto_enabled && pre.offering_auto_rune {
            let r = automatic_tools::advance_rune_sacrifice(
                &automatic_tools::AdvanceRuneSacrificeInput {
                    dt,
                    sacrifice_timer: state.automation.sacrifice_timer,
                    auto_sacrifice_interval: state.automation.auto_sacrifice_interval,
                    offerings: state.automation.offerings,
                },
            );
            state.automation.sacrifice_timer = r.sacrifice_timer;
            output.events.extend(r.events);
        }

        // 2. Ant sacrifice — advance the dual timers, then check readiness.
        if pre.ant_sacrifice_unlocked {
            let t = automatic_tools::advance_ant_sacrifice_timers(
                &automatic_tools::AdvanceAntSacrificeTimersInput {
                    dt,
                    global_delta: pre.global_time_multiplier,
                    ant_sacrifice_timer: state.ants.ant_sacrifice_timer,
                    ant_sacrifice_timer_real: state.ants.ant_sacrifice_timer_real,
                },
            );
            state.ants.ant_sacrifice_timer = t.ant_sacrifice_timer;
            state.ants.ant_sacrifice_timer_real = t.ant_sacrifice_timer_real;

            let events = automatic_tools::check_ant_sacrifice_ready(
                &automatic_tools::CheckAntSacrificeReadyInput {
                    mode: state.ants.toggles.auto_sacrifice_mode,
                    crumbs_this_sacrifice: state.ants.crumbs_this_sacrifice,
                    ant_sacrifice_timer_real: state.ants.ant_sacrifice_timer_real,
                    auto_sacrifice_enabled: state.ants.toggles.auto_sacrifice_enabled,
                    available_reborn_elo: pre.available_reborn_elo,
                    only_sacrifice_max_reborn_elo: state.ants.toggles.only_sacrifice_max_reborn_elo,
                    always_sacrifice_max_reborn_elo: state
                        .ants
                        .toggles
                        .always_sacrifice_max_reborn_elo,
                    ant_sacrifice_timer: state.ants.ant_sacrifice_timer,
                    auto_sacrifice_threshold: state.ants.toggles.auto_sacrifice_threshold,
                    immortal_elo_gain: pre.immortal_elo_gain,
                    immortal_elo: state.ants.immortal_elo,
                    reborn_elo: state.ants.reborn_elo,
                },
            );
            output.events.extend(events);
        }

        // 3. Obtainium — research[61] == 1 credits gain; else (vestigial)
        //    request a multiplier recompute, mirroring the legacy `else` arm.
        if state.researches.researches[61] == 1.0 {
            let r = automatic_tools::add_obtainium(&automatic_tools::AddObtainiumInput {
                obtainium: state.researches.obtainium,
                obtainium_gain: pre.obtainium_gain,
                ascension_challenge: state.challenges.current_ascension_challenge,
                taxman_last_stand_enabled: state.singularity.taxman_last_stand.enabled,
                taxman_last_stand_completions: state.singularity.taxman_last_stand.completions,
            });
            state.researches.obtainium = r.obtainium;
            output.events.extend(r.events);
        } else {
            output
                .events
                .push(CoreEvent::ObtainiumMultiplierRecomputeRequested);
        }

        // 4. Auto-research dispatch (manual vs Roomba).
        let auto_research_events = auto_research::process_auto_research_tick(
            &auto_research::ProcessAutoResearchTickInput {
                auto_research_toggle: state.researches.auto_research_toggle,
                auto_research_selected: state.researches.auto_research_selected,
                auto_research_mode: state.researches.auto_research_mode,
                roomba_unlocked: pre.roomba_unlocked,
                challengecompletions_14: state.challenges.challenge_completions[14],
            },
        );
        output.events.extend(auto_research_events);
    }

    // ─── Tail (tackTail) — runs unconditionally, even under time-warp
    // (mirrors the legacy `tackBody`). ────────────────────────────────
    //
    // 1. addOfferings (dt/2, no event) — gated by highest c3 completions.
    if state.challenges.highest_challenge_completions[3] > 0.0 {
        let r = automatic_tools::add_offerings(&automatic_tools::AddOfferingsInput {
            dt: dt / 2.0,
            auto_offering_counter: state.automation.auto_offering_counter,
            offerings: state.automation.offerings,
        });
        state.automation.auto_offering_counter = r.auto_offering_counter;
        state.automation.offerings = r.offerings;
    }

    // 2. tickChallengeSweep (dt) — the SweepState machine.
    let should_run_sweep =
        state.researches.researches[150] > 0.0 && state.automation.auto_challenge_running;
    let sweep = challenge_sweep::tick_challenge_sweep(
        &state.automation.sweep_state,
        &challenge_sweep::TickChallengeSweepInput {
            dt,
            time_since_last_state_change: state.automation.sweep_time_since_last_change,
            should_run_sweep,
            timer_start: pre.sweep_timer_start,
            timer_exit: pre.sweep_timer_exit,
            timer_enter: pre.sweep_timer_enter,
            next_regular_challenge_from_initial: pre.sweep_next_regular_challenge_from_initial,
            next_regular_challenge_from_active: pre.sweep_next_regular_challenge_from_active,
            challenge_15_auto_exponent_check: pre.sweep_challenge_15_auto_exponent_check,
            is_finished_still_valid: pre.sweep_is_finished_still_valid,
        },
    );
    state.automation.sweep_state = sweep.state;
    state.automation.sweep_time_since_last_change = sweep.time_since_last_state_change;
    output.events.extend(sweep.events);

    // 3. applyAutoResets (dt) — emits AutoResetTriggered per fired tier.
    let resets = auto_reset::apply_auto_resets(&auto_reset::ApplyAutoResetsInput {
        dt,
        prestige_mode: state.automation.prestige_reset_mode,
        auto_prestige_enabled: state.automation.auto_prestige_enabled,
        auto_prestige_milestone: pre.auto_prestige_milestone,
        prestige_points: state.upgrades.prestige_points,
        prestige_point_gain: pre.prestige_point_gain,
        prestige_amount: state.automation.prestige_amount,
        coins_this_prestige: state.coin_counters.coins_this_prestige,
        auto_reset_timer_prestige: state.automation.auto_reset_timer_prestige,
        transcend_mode: state.automation.transcend_reset_mode,
        auto_transcend_enabled: state.automation.auto_transcend_enabled,
        upgrade_89: state.upgrades.upgrades[89],
        transcend_points: state.upgrades.transcend_points,
        transcend_point_gain: pre.transcend_point_gain,
        transcend_amount: state.automation.transcend_amount,
        coins_this_transcension: state.coin_counters.coins_this_transcension,
        auto_reset_timer_transcension: state.automation.auto_reset_timer_transcension,
        reincarnation_mode: state.automation.reincarnation_reset_mode,
        auto_reincarnate_enabled: state.automation.auto_reincarnate_enabled,
        research_46: state.researches.researches[46],
        reincarnation_points: state.upgrades.reincarnation_points,
        reincarnation_point_gain: pre.reincarnation_point_gain,
        reincarnation_amount: state.automation.reincarnation_amount,
        transcend_shards: state.reset_counters.transcend_shards,
        auto_reset_timer_reincarnation: state.automation.auto_reset_timer_reincarnation,
        ascension_challenge: state.challenges.current_ascension_challenge,
        transcension_challenge: state.challenges.current_transcension_challenge,
        reincarnation_challenge: state.challenges.current_reincarnation_challenge,
    });
    state.automation.auto_reset_timer_prestige = resets.auto_reset_timer_prestige;
    state.automation.auto_reset_timer_transcension = resets.auto_reset_timer_transcension;
    state.automation.auto_reset_timer_reincarnation = resets.auto_reset_timer_reincarnation;
    output.events.extend(resets.events);
}

/// `player.insideSingularityChallenge` — true when the player is inside
/// any singularity (Exalt) challenge. Gates `sing_challenge_timer`
/// accumulation in [`timers::advance_singularity_timer`].
fn inside_singularity_challenge(s: &crate::state::SingularityState) -> bool {
    s.no_singularity_upgrades.enabled
        || s.one_challenge_cap.enabled
        || s.no_octeracts.enabled
        || s.limited_ascensions.enabled
        || s.no_ambrosia_upgrades.enabled
        || s.no_quark_upgrades.enabled
        || s.limited_time.enabled
        || s.sadistic_prequel.enabled
        || s.taxman_last_stand.enabled
}

// ─── Dispatch helpers ────────────────────────────────────────────────────

fn dispatch_buy(state: &mut GameState, req: &BuyRequest) -> SmallVec<[CoreEvent; 4]> {
    // Each arm borrows disjoint `GameState` fields explicitly so the
    // borrow checker can verify the per-slice mutator and the canonical
    // `state.upgrades.*` currency don't alias. (A helper returning
    // `&mut ProducerFamilyState` would force a single whole-state borrow
    // and prevent the second `&mut` for the currency.)
    match req {
        BuyRequest::Upgrade(inp) => buy_upgrades(&mut state.upgrades, *inp),
        BuyRequest::Multiplier(inp) => {
            buy_multiplier(&mut state.multiplier, &mut state.upgrades.coins, *inp)
        }
        BuyRequest::Accelerator(inp) => {
            buy_accelerator(&mut state.accelerator, &mut state.upgrades.coins, *inp)
        }
        BuyRequest::CrystalUpgrade(inp) => buy_crystal_upgrades(&mut state.crystal_upgrades, *inp),
        BuyRequest::ParticleBuilding(inp) => buy_particle_building(
            &mut state.particle_buildings,
            &mut state.upgrades.reincarnation_points,
            *inp,
        ),
        BuyRequest::TesseractBuilding(inp) => {
            buy_tesseract_building(&mut state.tesseract_buildings, *inp)
        }
        BuyRequest::ProducerMax(inp) => match inp.producer_type {
            ProducerType::Coin => {
                buy_max(&mut state.coin_producers, &mut state.upgrades.coins, *inp)
            }
            ProducerType::Diamonds => buy_max(
                &mut state.diamond_producers,
                &mut state.upgrades.prestige_points,
                *inp,
            ),
            ProducerType::Mythos => buy_max(
                &mut state.mythos_producers,
                &mut state.upgrades.transcend_points,
                *inp,
            ),
            ProducerType::Particles => buy_max(
                &mut state.particle_producers,
                &mut state.upgrades.reincarnation_points,
                *inp,
            ),
        },
        BuyRequest::Producer(inp) => match inp.producer_type {
            ProducerType::Coin => {
                buy_producer(&mut state.coin_producers, &mut state.upgrades.coins, *inp)
            }
            ProducerType::Diamonds => buy_producer(
                &mut state.diamond_producers,
                &mut state.upgrades.prestige_points,
                *inp,
            ),
            ProducerType::Mythos => buy_producer(
                &mut state.mythos_producers,
                &mut state.upgrades.transcend_points,
                *inp,
            ),
            ProducerType::Particles => buy_producer(
                &mut state.particle_producers,
                &mut state.upgrades.reincarnation_points,
                *inp,
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tack_runs_against_default_state_without_panic() {
        let mut state = GameState::default();
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let output = tack(&mut state, &input);
        // Default state: head timers advance silently, but the middle's
        // obtainium branch emits the vestigial recompute request every
        // non-warp tick (research[61] != 1) — mirroring the legacy `else`.
        assert_eq!(output.events.len(), 1);
        assert!(matches!(
            output.events[0],
            CoreEvent::ObtainiumMultiplierRecomputeRequested
        ));
    }

    #[test]
    fn phase_automation_advances_head_timers() {
        let mut state = GameState::default();
        let input = TackInput {
            dt: 2.0,
            ..TackInput::default()
        };
        // `tack` now self-derives `global_time_multiplier`; drive
        // `phase_automation` directly with a controlled cache so this stays a
        // focused test of timer advancement under known multipliers.
        let cache = CrossMechanicCache {
            automation_pre: AutomationPre {
                global_time_multiplier: 3.0,
                ascension_speed_multi: 5.0,
                singularity_speed_multi: 1.0,
                max_quark_timer: 90_000.0,
                export_gq_per_hour: 1.0,
                ..AutomationPre::default()
            },
        };
        let mut output = TickOutput::default();
        phase_automation(&mut state, &cache, &input, &mut output);

        // Reset counters advance by dt × global_time_multiplier (2 × 3).
        assert_eq!(state.reset_counters.prestige_counter, 6.0);
        assert_eq!(state.reset_counters.transcend_counter, 6.0);
        assert_eq!(state.reset_counters.reincarnation_counter, 6.0);
        // Ascension counter scales by ascension speed (2 × 5); real by dt.
        assert_eq!(state.reset_counters.ascension_counter, 10.0);
        assert_eq!(state.reset_counters.ascension_counter_real, 2.0);
        // Singularity tri-counter; no challenge active → challenge timer 0.
        assert_eq!(state.reset_counters.ascension_counter_real_real, 2.0);
        assert_eq!(state.singularity.singularity_counter, 2.0);
        assert_eq!(state.singularity.sing_challenge_timer, 0.0);
        // Quark + golden-quark export timers advance by raw dt.
        assert_eq!(state.quarks.quarks_timer, 2.0);
        assert_eq!(state.golden_quarks.golden_quarks_timer, 2.0);
        // Simple counters emit no events; the only event is the middle's
        // vestigial obtainium-recompute request (research[61] != 1).
        assert_eq!(output.events.len(), 1);
        assert!(matches!(
            output.events[0],
            CoreEvent::ObtainiumMultiplierRecomputeRequested
        ));
    }

    #[test]
    fn global_speed_mult_pre_is_one_at_default() {
        let state = GameState::default();
        // Every contributing line is the multiplicative identity at the
        // default state, so the self-derived global-speed mult is exactly 1.
        assert_eq!(compute_global_speed_mult_pre(&state), 1.0);
    }

    #[test]
    fn global_speed_mult_pre_scales_with_research() {
        let mut state = GameState::default();
        // Research 5x21 (`researches[121]`) adds `1 + n/50` to the DR-enabled
        // leg. n = 100 → factor 3.0; nothing else is active and 3.0 sits in
        // the (1, 100] no-DR band, so the mult is exactly 3.0.
        state.researches.researches[121] = 100.0;
        assert!((compute_global_speed_mult_pre(&state) - 3.0).abs() < 1e-12);
    }

    #[test]
    fn ascension_speed_mult_pre_is_one_at_default() {
        let state = GameState::default();
        // Every base line is the multiplicative identity and the exponent
        // spread is 0 at the default state → mult is exactly 1.
        assert_eq!(compute_ascension_speed_mult_pre(&state), 1.0);
    }

    #[test]
    fn ascension_speed_mult_pre_applies_exponent_spread() {
        let mut state = GameState::default();
        // Chronometer (shop[18]) adds `1 + 0.012n` to the base; n = 100 → 2.2.
        state.shop.upgrades[18] = 100.0;
        // GQ singAscensionSpeed (gq[55]) contributes 0.03 to the exponent
        // spread once its level > 0, so the result is base^(1 + 0.03).
        state.golden_quarks.upgrades[55].level = 1.0;
        let expected = (1.0 + 0.012 * 100.0_f64).powf(1.0 + 0.03);
        assert!((compute_ascension_speed_mult_pre(&state) - expected).abs() < 1e-9);
    }

    #[test]
    fn ascension_speed_mult_pre_one_mind_locks_to_ten() {
        let mut state = GameState::default();
        // A large chronometer base would otherwise dominate, but oneMind
        // (gq[59]) locks ascension speed to a flat ×10.
        state.shop.upgrades[18] = 1_000.0;
        state.golden_quarks.upgrades[59].level = 1.0;
        assert_eq!(compute_ascension_speed_mult_pre(&state), 10.0);
    }

    #[test]
    fn singularity_speed_mult_pre_is_one_at_default() {
        let state = GameState::default();
        assert_eq!(compute_singularity_speed_mult_pre(&state), 1.0);
    }

    #[test]
    fn singularity_speed_mult_pre_decreases_with_brick_of_lead() {
        let mut state = GameState::default();
        // ambrosiaBrickOfLead (ambrosia[31]) singularitySpeedMult = 1 - n/100.
        state.ambrosia.upgrades[31].level = 25.0;
        assert!((compute_singularity_speed_mult_pre(&state) - 0.75).abs() < 1e-12);
    }

    #[test]
    fn ambrosia_luck_pre_is_one_hundred_at_default() {
        let state = GameState::default();
        // Base raw luck 100 × base multiplier 1 = 100.
        assert_eq!(compute_ambrosia_luck_pre(&state), 100.0);
    }

    #[test]
    fn ambrosia_luck_pre_grows_with_shop_luck() {
        let mut state = GameState::default();
        // shopAmbrosiaLuck1 (shop[65]) adds to the raw-luck sum.
        state.shop.upgrades[65] = 10.0;
        assert!(compute_ambrosia_luck_pre(&state) > 100.0);
    }

    #[test]
    fn ambrosia_generation_speed_pre_is_zero_when_locked() {
        let state = GameState::default();
        // Ambrosia gated (noSingularityUpgrades completions == 0) → 0.
        assert_eq!(compute_ambrosia_generation_speed_pre(&state), 0.0);
    }

    #[test]
    fn ambrosia_generation_speed_pre_unlocks_with_e1x1() {
        let mut state = GameState::default();
        // Gate open → raw_speed 1; E1x1 grants +3 blueberries → 1 × 3 = 3.
        state.singularity.no_singularity_upgrades.completions = 1.0;
        assert!((compute_ambrosia_generation_speed_pre(&state) - 3.0).abs() < 1e-12);
    }

    #[test]
    fn red_ambrosia_luck_pre_is_one_hundred_at_default() {
        let state = GameState::default();
        // Base 100; the LuckConversion line is ⌊(100 − 100) / 20⌋ = 0.
        assert_eq!(compute_red_ambrosia_luck_pre(&state, 100.0), 100.0);
    }

    #[test]
    fn red_ambrosia_luck_pre_adds_luck_conversion() {
        let state = GameState::default();
        // Default luckConversion is 20, so ambrosiaLuck 200 contributes
        // ⌊(200 − 100) / 20⌋ = 5 → 100 + 5 = 105.
        assert_eq!(compute_red_ambrosia_luck_pre(&state, 200.0), 105.0);
    }

    #[test]
    fn red_ambrosia_luck_pre_grows_with_shop_red_luck() {
        let mut state = GameState::default();
        // shopRedLuck1 (shop[77]) redLuck = 0.05 × n adds to the sum.
        state.shop.upgrades[77] = 10.0;
        assert!(compute_red_ambrosia_luck_pre(&state, 100.0) > 100.0);
    }

    #[test]
    fn red_ambrosia_generation_speed_pre_is_zero_when_locked() {
        let state = GameState::default();
        // Base gate (noAmbrosiaUpgrades == 0) and BlueberrySpeed (0) both 0.
        assert_eq!(compute_red_ambrosia_generation_speed_pre(&state, 0.0), 0.0);
    }

    #[test]
    fn red_ambrosia_generation_speed_pre_unlocks_with_exalt5() {
        let mut state = GameState::default();
        // Gate open → 1 × BlueberrySpeed 10 × redGen 1 × redSpeedMult
        // (1 + 2·1/100 = 1.02) = 10.2.
        state.singularity.no_ambrosia_upgrades.completions = 1.0;
        assert!((compute_red_ambrosia_generation_speed_pre(&state, 10.0) - 10.2).abs() < 1e-12);
    }

    #[test]
    fn red_ambrosia_generation_speed_pre_sqrt_softcaps_above_1000() {
        let mut state = GameState::default();
        // BlueberrySpeed softcap: b > 1000 → √(b·1000); √(4000·1000) = 2000,
        // × redSpeedMult 1.02 = 2040.
        state.singularity.no_ambrosia_upgrades.completions = 1.0;
        assert!((compute_red_ambrosia_generation_speed_pre(&state, 4000.0) - 2040.0).abs() < 1e-9);
    }

    #[test]
    fn auto_timer_fields_at_default() {
        let state = GameState::default();
        assert_eq!(compute_auto_timer_fields(&state), (0.0, 0.0, 1.0, 0.0));
    }

    #[test]
    fn auto_timer_fields_track_upgrades() {
        let mut state = GameState::default();
        state.shop.upgrades[0] = 3.0; // offeringPotion
        state.shop.upgrades[1] = 5.0; // obtainiumPotion
        state.octeract_upgrades.upgrades[26].level = 10.0; // 1 + 4*10/100 = 1.4
        state.golden_quarks.upgrades[2].level = 4.0; // 4*5/2 = 10
        let (off, obt, speed, export) = compute_auto_timer_fields(&state);
        assert_eq!((off, obt, export), (3.0, 5.0, 10.0));
        assert!((speed - 1.4).abs() < 1e-12);
    }

    #[test]
    fn ambrosia_timer_fields_at_default() {
        let state = GameState::default();
        // bonus 0, TIME_PER_AMBROSIA 45, accelerator/brick multipliers 1.
        assert_eq!(compute_ambrosia_timer_fields(&state), (0.0, 45.0, 1.0, 1.0));
    }

    #[test]
    fn ambrosia_timer_fields_track_state() {
        let mut state = GameState::default();
        // noAmbrosiaUpgrades completed → bonusAmbrosia 1; scales the accelerator.
        state.singularity.no_ambrosia_upgrades.completions = 5.0;
        state.shop.upgrades[70] = 10.0; // shopAmbrosiaAccelerator: 1 − 0.006·10·5 = 0.7
        state.ambrosia.upgrades[31].level = 25.0; // brickOfLead barReq: 1/(1 − 25/50) = 2
        let (bonus, tpa, accel, brick) = compute_ambrosia_timer_fields(&state);
        assert_eq!(bonus, 1.0);
        assert_eq!(tpa, 45.0);
        assert!((accel - 0.7).abs() < 1e-12);
        assert!((brick - 2.0).abs() < 1e-12);
    }

    #[test]
    fn time_warp_skips_head_timers() {
        let mut state = GameState::default();
        let input = TackInput {
            dt: 2.0,
            time_warp: true,
            automation_pre: AutomationPre {
                global_time_multiplier: 3.0,
                ..AutomationPre::default()
            },
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);

        // Head timers are gated by `!time_warp` → untouched under warp.
        assert_eq!(state.reset_counters.prestige_counter, 0.0);
        assert_eq!(state.reset_counters.ascension_counter, 0.0);
        assert_eq!(state.quarks.quarks_timer, 0.0);
    }

    #[test]
    fn golden_quarks_timer_inert_without_export() {
        // Default automation_pre has export_gq_per_hour = 0 → GQ timer
        // does not advance even on a normal tick.
        let mut state = GameState::default();
        let input = TackInput {
            dt: 5.0,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        assert_eq!(state.golden_quarks.golden_quarks_timer, 0.0);
        // ...but the quark timer (no export gate) still advances.
        assert_eq!(state.quarks.quarks_timer, 5.0);
    }

    #[test]
    fn phase_automation_fires_octeract_giveaway() {
        let mut state = GameState::default();
        state.octeract_upgrades.octeract_timer = 0.5;
        let input = TackInput {
            dt: 1.0,
            automation_pre: AutomationPre {
                octeract_unlocked: true,
                octeract_per_second: 4.0,
                ..AutomationPre::default()
            },
            ..TackInput::default()
        };
        let output = tack(&mut state, &input);

        // 0.5 + 1.0 = 1.5 → 1 giveaway-second; wow_octeracts += 1 × 4.
        assert_eq!(state.cube_balances.wow_octeracts.to_number(), 4.0);
        assert_eq!(state.cube_balances.total_wow_octeracts.to_number(), 4.0);
        assert!((state.octeract_upgrades.octeract_timer - 0.5).abs() < 1e-9);
        assert!(output.events.iter().any(|e| matches!(
            e,
            CoreEvent::OcteractTickFired {
                amount_of_giveaways: 1
            }
        )));
    }

    #[test]
    fn phase_automation_generates_ambrosia() {
        let mut state = GameState::default();
        // Unlock ambrosia generation (noSingularityUpgrades completed once).
        state.singularity.no_singularity_upgrades.completions = 1.0;
        let input = TackInput {
            dt: 1000.0,
            ..TackInput::default()
        };
        // `tack` now self-derives ambrosia_luck (+ the speed mults); drive
        // phase_automation directly with a controlled cache so this stays a
        // focused test of ambrosia generation.
        let cache = CrossMechanicCache {
            automation_pre: AutomationPre {
                ambrosia_generation_speed: 1.0,
                ambrosia_luck: 200.0,
                time_per_ambrosia: 45.0,
                ..AutomationPre::default()
            },
        };
        let mut output = TickOutput::default();
        phase_automation(&mut state, &cache, &input, &mut output);

        assert!(state.ambrosia.ambrosia > 0.0);
        assert!(state.ambrosia.lifetime_ambrosia > 0.0);
        assert!(output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::AmbrosiaGained { .. })));
    }

    #[test]
    fn phase_automation_middle_credits_obtainium() {
        let mut state = GameState::default();
        // research[61] == 1 routes the obtainium branch to addObtainium.
        state.researches.researches[61] = 1.0;
        state.researches.obtainium = Decimal::from_finite(100.0);
        let input = TackInput {
            dt: 1.0,
            automation_pre: AutomationPre {
                obtainium_gain: Decimal::from_finite(25.0),
                ..AutomationPre::default()
            },
            ..TackInput::default()
        };
        let output = tack(&mut state, &input);

        assert_eq!(state.researches.obtainium.to_number(), 125.0);
        // addObtainium path → AutoToolFired, and NOT the recompute request.
        assert!(output.events.iter().any(|e| matches!(
            e,
            CoreEvent::AutoToolFired {
                tool: crate::events::AutoTool::AddObtainium
            }
        )));
        assert!(!output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ObtainiumMultiplierRecomputeRequested)));
    }

    #[test]
    fn tail_runs_under_time_warp() {
        // Chunk 11: the tail runs even under time-warp (head + middle don't).
        let mut state = GameState::default();
        let input = TackInput {
            dt: 5.0,
            time_warp: true,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        // Head skipped under warp.
        assert_eq!(state.reset_counters.prestige_counter, 0.0);
        assert_eq!(state.quarks.quarks_timer, 0.0);
        // Tail ran: the reincarnation auto-reset timer accrued (ascension
        // challenge != 12), proving the tail executed under warp.
        assert_eq!(state.automation.auto_reset_timer_reincarnation, 5.0);
    }

    #[test]
    fn phase_automation_boots_challenge_sweep() {
        let mut state = GameState::default();
        // shouldRunSweep = researches[150] > 0 && auto_challenge_running.
        state.researches.researches[150] = 1.0;
        state.automation.auto_challenge_running = true;
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let output = tack(&mut state, &input);
        assert_eq!(
            state.automation.sweep_state,
            crate::events::SweepState::InitialWait
        );
        assert!(output.events.iter().any(|e| matches!(
            e,
            CoreEvent::ChallengeSweepTransitioned {
                to: crate::events::SweepState::InitialWait,
                ..
            }
        )));
    }

    #[test]
    fn phase_automation_generates_ants() {
        let mut state = GameState::default();
        // Workers (tier 0) purchased → crumbs accrue this tick.
        state.ants.producers[0].purchased = 1000.0;
        let input = TackInput {
            dt: 1.0,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        assert!(state.ants.crumbs.to_number() > 0.0);
    }

    #[test]
    fn aggregator_pre_bundles_are_state_derived() {
        // The update_all_multiplier / update_all_tick pre-bundles are now
        // computed purely from `&GameState` (+ `total_accelerator_boost`) —
        // no caller input remains. A duplication rune at 800 raises
        // `multiplicative_multipliers_rune` to `1 + 800/400 = 3.0`; a speed
        // rune at 400 raises `multiplicative_accelerators_rune` to
        // `1 + 400/400 = 2.0`.
        let mut state = GameState::default();
        state.runes.rune_levels[crate::state::RUNE_DUPLICATION] = 800.0;
        state.hepteracts.multiplier.bal = 5.0; // hept-multiplier
        state.runes.rune_levels[crate::state::RUNE_SPEED] = 400.0;
        state.hepteracts.accelerator.bal = 10.0;

        let mult = compute_update_all_multiplier_pre(&state, 0.0);
        // Duplication rune at 800: 1 + 800/400 = 3.0.
        assert!((mult.multiplicative_multipliers_rune - 3.0).abs() < 1e-9);
        // Hept-multiplier at 5: 1000 * 5 = 5000.
        assert!((mult.hepteract_multiplier - 5_000.0).abs() < 1e-9);

        let tick = compute_update_all_tick_pre(&state, 0.0);
        // Speed rune at 400: 1 + 400/400 = 2.0.
        assert!((tick.multiplicative_accelerators_rune - 2.0).abs() < 1e-9);
        // Hept-accelerator at 10: 2000 * 10 = 20_000.
        assert!((tick.hepteract_accelerators - 20_000.0).abs() < 1e-9);
    }

    #[test]
    fn total_accelerator_boost_threads_into_aggregator_pre_bundles() {
        // The shared `total_accelerator_boost` is passed straight through
        // into both bundles (it is computed once in `phase_global_state`).
        let state = GameState::default();
        let mult = compute_update_all_multiplier_pre(&state, 42.0);
        let tick = compute_update_all_tick_pre(&state, 42.0);
        assert_eq!(mult.total_accelerator_boost, 42.0);
        assert_eq!(tick.total_accelerator_boost, 42.0);
    }

    #[test]
    fn cross_mechanic_cache_forwards_automation_pre_from_input() {
        // `automation_pre` is the last bundle precompute forwards verbatim
        // (the four global-state bundles all self-derive now). Pins that
        // forwarding until the Phase-5 aggregators port.
        let state = GameState::default();
        let input = TackInput {
            dt: 0.025,
            automation_pre: AutomationPre {
                max_quark_timer: 12_345.0,
                ..AutomationPre::default()
            },
            ..TackInput::default()
        };
        let cache = phase_cross_mechanic_precompute(&state, &input);
        assert_eq!(cache.automation_pre.max_quark_timer, 12_345.0);
    }

    #[test]
    fn tack_dispatches_buy_upgrade_action() {
        use synergismforkd_bignum::Decimal;

        use crate::events::UpgradeTier;

        let mut state = GameState::default();
        state.upgrades.coins = Decimal::from_finite(1e10);

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::Upgrade(BuyUpgradeInput {
                tier: UpgradeTier::Coin,
                pos: 5,
                cost_exponent: 2.0,
                requirement_exists: true,
            })));

        let output = tack(&mut state, &input);

        // The buy event should land in the output.
        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::UpgradePurchased { .. })),
            "expected UpgradePurchased in events, got {:?}",
            output.events
        );
        assert_eq!(state.upgrades.upgrades[5], 1);
    }

    #[test]
    fn time_warp_skips_phase_automation() {
        // Phase 5 is a stub today, so this test only proves the gate
        // compiles and `time_warp = true` produces the same output as
        // `time_warp = false` against an empty action queue. When
        // automation lands, replace with a test that observes a
        // side-effect difference.
        let mut state_a = GameState::default();
        let mut state_b = GameState::default();
        let normal = TackInput {
            dt: 0.025,
            time_warp: false,
            ..TackInput::default()
        };
        let warped = TackInput {
            dt: 0.025,
            time_warp: true,
            ..TackInput::default()
        };
        let out_a = tack(&mut state_a, &normal);
        let out_b = tack(&mut state_b, &warped);
        // The warped tick skips head + middle (the tail runs but emits
        // nothing on default state); the normal tick runs the middle and
        // emits the recompute request.
        assert!(out_b.events.is_empty());
        assert!(out_a.events.len() > out_b.events.len());
    }

    #[test]
    fn dispatch_buy_routes_producer_family_by_type() {
        // Sanity-check the per-arm dispatch — each variant pairs the
        // right producer family with the right currency in state.upgrades.
        let mut state = GameState::default();
        state.upgrades.coins = synergismforkd_bignum::Decimal::from_finite(1e6);

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::ProducerMax(BuyMaxInput {
                index: 1,
                producer_type: ProducerType::Coin,
                cost_input: crate::mechanics::producers::GetProducerCostInput {
                    cost_divisor: 1.0,
                    in_transcension_challenge_4: false,
                    in_reincarnation_challenge_8: false,
                    in_reincarnation_challenge_10: false,
                    challengecompletions_4: 0.0,
                    challengecompletions_8: 0.0,
                },
            })));

        let _ = tack(&mut state, &input);
        // Bought at least one of tier-1 Coin producer.
        assert!(state.coin_producers.tiers[0].owned > 0.0);
    }

    #[test]
    fn phase_tax_feeds_coin_gain_and_writes_taxdivisor() {
        // tier-1 coin producer owned → produce_total = 1000 * 0.25 = 250
        // (default coin multipliers are 1), above the 0.001 coin-gain gate,
        // so coins accrue and `G.taxdivisor` recomputes above 1.
        let mut state = GameState::default();
        state.coin_producers.tiers[0].owned = 1000.0;
        assert_eq!(state.g_cache.taxdivisor, Decimal::one()); // fresh default
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        // `produce_total` flowed from the tax phase into resource_gain.
        assert!(state.upgrades.coins > Decimal::zero());
        // The tax phase recomputed and persisted `G.taxdivisor`.
        assert!(state.g_cache.taxdivisor > Decimal::one());
    }

    #[test]
    fn update_all_multiplier_pre_reads_lagged_taxdivisor_from_g_cache() {
        // The Phase-2 consumer (upgrade-68 free-multiplier term) reads the
        // prior tick's `g_cache.taxdivisor`, NOT a value freshly recomputed
        // this tick — the substrate of the legacy one-tick lag. (The
        // `fallback` bundle no longer backs `taxdivisor`.)
        let mut state = GameState::default();
        state.g_cache.taxdivisor = Decimal::from_finite(1e300);
        let pre = compute_update_all_multiplier_pre(&state, 0.0);
        assert_eq!(pre.taxdivisor, Decimal::from_finite(1e300));
    }

    #[test]
    fn resource_gain_pre_carries_producer_base_scalars() {
        // The 15 diamond/mythos/particle base scalars are now wired from
        // the immutable legacy constants (they were 0 when forwarded from a
        // default `ResourceGainPre`, leaving the cascades inert).
        let mut state = GameState::default();
        let agg = phase_global_state(&mut state);
        let tax = phase_tax(&mut state, &agg);
        let reset = compute_reset_currency_gains(&state, &agg);
        let pre = compute_resource_gain_pre(&agg, &tax, &reset);
        assert_eq!(pre.first_produce_diamonds, 0.05);
        assert_eq!(pre.fifth_produce_diamonds, 0.000_005);
        assert_eq!(pre.first_produce_mythos, 1.0);
        assert_eq!(pre.fifth_produce_mythos, 0.000_04);
        assert_eq!(pre.first_produce_particles, 0.25);
        assert_eq!(pre.fifth_produce_particles, 0.5);
    }

    #[test]
    fn diamond_cascade_produces_prestige_shards_through_tack() {
        // End-to-end: with the base scalars now wired, owning tier-1
        // diamond producers yields prestige shards. (Before this chunk the
        // forwarded scalar was 0, so the cascade was inert.)
        // produce = owned(1000) * first_produce_diamonds(0.05) * gcm(1) = 50
        // per `dt/0.025` step → 50 shards at dt = 0.025.
        let mut state = GameState::default();
        state.diamond_producers.tiers[0].owned = 1000.0;
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        assert!((state.crystal_upgrades.prestige_shards.to_number() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn reset_currency_gains_feed_resource_gain_pre() {
        // reset_currency is now self-derived: with coins-this-prestige set,
        // the prestige point gain is `floor((coins/1e12) ^ prestige_pow)`
        // where the default `prestige_pow = 0.5` (ecc5 0, deflation mult 1).
        // (1e24 / 1e12) ^ 0.5 = (1e12) ^ 0.5 = 1e6.
        let mut state = GameState::default();
        state.coin_counters.coins_this_prestige = Decimal::from_finite(1e24);
        let agg = phase_global_state(&mut state);
        let tax = phase_tax(&mut state, &agg);
        let reset = compute_reset_currency_gains(&state, &agg);
        assert!((reset.prestige_point_gain.to_number() - 1e6).abs() / 1e6 < 1e-9);
        // It threads through into ResourceGainPre (was forwarded/0 before).
        let pre = compute_resource_gain_pre(&agg, &tax, &reset);
        assert_eq!(pre.prestige_point_gain, reset.prestige_point_gain);
    }

    #[test]
    fn global_multipliers_pre_derives_crystal_and_building_fields() {
        // crystal_mult = (prestige_shards + 1) ^ crystal_exponent; default
        // exponent is 1/3, so 1e9 shards → ~1000 (was forwarded identity 1).
        let mut state = GameState::default();
        state.crystal_upgrades.prestige_shards = Decimal::from_finite(1e9);
        let pre = compute_global_multipliers_pre(&state);
        assert!((pre.crystal_mult.to_number() - 1000.0).abs() / 1000.0 < 1e-6);

        // ascend_building_dr_value reflects owned ascend buildings (raw sum
        // below the 100k threshold).
        let mut s2 = GameState::default();
        s2.tesseract_buildings.ascend_building_1.owned = 500.0;
        let pre2 = compute_global_multipliers_pre(&s2);
        assert_eq!(pre2.ascend_building_dr_value, 500.0);
    }
}
