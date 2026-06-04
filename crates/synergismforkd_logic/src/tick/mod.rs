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
/// The remaining `*_pre` bundles are caller-provided for the duration of
/// the MVP port; they collapse into a single in-orchestrator
/// `CrossMechanicCache` once the upstream mechanics (rune/ant/hepteract
/// effects, achievement rewards, challenge-15 rewards) port. The
/// `update_all_multiplier` / `update_all_tick` bundles have already been
/// retired — both aggregators now self-derive from `&GameState`.
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
    /// Hand-packed pre-evaluated bundle for
    /// [`compute_global_multipliers`].
    pub global_multipliers_pre: GlobalMultipliersPreEvaluated,
    /// Hand-packed pre-evaluated bundle for [`resource_gain`].
    pub resource_gain_pre: ResourceGainPre,
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
/// computations across the four aggregators' `*Pre` parameters, which
/// every caller hand-packed — silently dropping a field gave a working
/// tick that produced slightly less, with no compile error.
///
/// Today this struct holds the four `*Pre` bundles directly. Each
/// future commit migrates one upstream effect into compute-from-state
/// inside `phase_cross_mechanic_precompute`, at which point the
/// corresponding `*Pre` field becomes a `From<&CrossMechanicCache>`
/// view and the caller stops providing it. Eventually
/// [`TackInput::global_multipliers_pre`] et al. all disappear and the
/// cache is fully self-derived from `&GameState`.
#[derive(Debug, Clone, Default)]
pub struct CrossMechanicCache {
    /// Pre-evaluated bundle for [`compute_global_multipliers`]. Owned
    /// by the cache so mechanics never read this from
    /// [`TackInput`] directly.
    pub global_multipliers_pre: GlobalMultipliersPreEvaluated,
    /// Pre-evaluated bundle for [`resource_gain`].
    pub resource_gain_pre: ResourceGainPre,
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
/// `update_all_multiplier` and `update_all_tick` fields aren't read
/// by downstream phases yet — only `global_multipliers` feeds
/// [`compute_resource_gain_pre`] today. The other two are captured
/// for symmetry with the design and to make the future migration of
/// the per-tier `produce_*` fields a local change rather than a
/// signature change.
#[derive(Debug, Clone, Copy)]
struct AggregatorOutputs {
    global_multipliers: GlobalMultipliersResult,
    #[expect(
        dead_code,
        reason = "captured for downstream phase migration; the lint will flip on as soon as a later phase reads it"
    )]
    update_all_multiplier: UpdateAllMultiplierResult,
    #[expect(
        dead_code,
        reason = "captured for downstream phase migration; the lint will flip on as soon as a later phase reads it"
    )]
    update_all_tick: UpdateAllTickResult,
}

/// Run one tick.
///
/// Phase ordering is canonical — see module docs. Reordering is a design
/// change requiring a separate commit and an updated CLAUDE.md note.
pub fn tack(state: &mut GameState, input: &TackInput) -> TickOutput {
    let mut output = TickOutput::default();

    let mut cache = phase_cross_mechanic_precompute(state, input);
    let aggregator_outputs = phase_global_state(state, &cache);
    // Phase 2b: coin production + tax. Mirrors the legacy `calculatetax()`
    // call slot — runs after the aggregators (it needs the fresh coin
    // multipliers), writes `g_cache.taxdivisor` for the *next* tick's
    // updateAllMultiplier, and supplies this tick's tax fields.
    let tax_outputs = phase_tax(state, &aggregator_outputs);
    // Phase 2 + tax outputs feed Phase 4's `ResourceGainPre`. Re-compute
    // the pre now that those results are available; fields whose upstream
    // is only state still come through unchanged.
    cache.resource_gain_pre = compute_resource_gain_pre(
        state,
        &input.resource_gain_pre,
        &aggregator_outputs,
        &tax_outputs,
    );
    phase_player_input(state, input, &mut output);
    phase_generation(state, &cache, input.dt, &mut output);
    phase_automation(state, &cache, input, &mut output);

    output
}

/// **Phase 1** — Cross-mechanic precompute.
///
/// Builds the [`CrossMechanicCache`] — the canonical artifact for every
/// downstream phase's cross-mechanic reads. Phases 2 / 4 / 5 take the
/// cache, not [`TackInput`], so the cache becomes the single screen on
/// which a designer can audit how mechanics flow into each other.
///
/// **Migration in progress.** Each `*Pre` field is being moved from
/// caller-provided to compute-from-state as the upstream mechanic
/// ports settle. The `update_all_multiplier` / `update_all_tick` bundles
/// have already been fully retired — they are now derived inside
/// [`phase_global_state`] (their only consumer), alongside the
/// `total_accelerator_boost` they share. The `global_multipliers_pre`
/// bundle still carries a handful of caller-forwarded fields (crystal /
/// building pipeline) plus aggregator-output fields injected later.
///
/// As mechanics port, the override list grows and `TackInput.*_pre`
/// shrinks. The cache is the migration target; `TackInput` is the
/// temporary input mechanism.
fn phase_cross_mechanic_precompute(state: &GameState, input: &TackInput) -> CrossMechanicCache {
    CrossMechanicCache {
        global_multipliers_pre: compute_global_multipliers_pre(
            state,
            &input.global_multipliers_pre,
        ),
        resource_gain_pre: input.resource_gain_pre,
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

/// State-derive the [`GlobalMultipliersPreEvaluated`] fields whose
/// upstream is a pure function of [`GameState`] and existing ported
/// mechanic helpers.
///
/// Migration coverage today:
/// - `prism_production_log10`           ✓ state-derived (Prism rune)
/// - `ant_multiplier`                   ✓ state-derived (Coins ant upgrade)
/// - `total_coin_owned`                 ✓ state-derived (sum of coin tiers)
/// - `recession_power`                  ✓ state-derived (G.recessionPower table)
/// - `crystal_mult`                     forwarded (chained crystal-coin pipeline)
/// - `building_power`                   forwarded (multi-input formula)
/// - `building_power_mult`              forwarded (depends on building_power)
/// - `crystal_upgrade_3_multiplier`     forwarded (depends on crystal_upgrade_3_base)
/// - `crystal_multiplier_achievement`   ✓ state-derived (achievement_rewards)
/// - `const_upgrade_1_buff_achievement` ✓ always 0 (no achievement grants it)
/// - `const_upgrade_2_buff_achievement` ✓ always 0 (no achievement grants it)
/// - `constant_ex_max_percent_increase` forwarded (shop-effect table not ported)
/// - `ascend_building_dr_value`         forwarded (formula not yet ported)
/// - `multiplier_effect`                ✓ injected by phase_global_state (aggregator output)
/// - `accelerator_effect`               ✓ injected by phase_global_state (aggregator output)
/// - `total_multiplier`                 ✓ injected by phase_global_state (aggregator output)
/// - `total_accelerator`                ✓ injected by phase_global_state (aggregator output)
/// - `total_accelerator_boost`          ✓ injected by precompute (calculate_total_accelerator_boost)
/// - `challenge_15_coin_exponent`       ✓ state-derived (challenge_15_rewards)
/// - `challenge_15_exponent_value`      ✓ state-derived (challenge_15_rewards)
/// - `challenge_15_constant_bonus`      ✓ state-derived (challenge_15_rewards)
#[must_use]
fn compute_global_multipliers_pre(
    state: &GameState,
    fallback: &GlobalMultipliersPreEvaluated,
) -> GlobalMultipliersPreEvaluated {
    use crate::mechanics::ant_upgrades::{coins_ant_upgrade_effect, CoinsAntUpgradeInput};
    use crate::mechanics::calculate::{calculate_total_coin_owned, CalculateTotalCoinOwnedInput};
    use crate::mechanics::corruptions::recession_power_at_level;
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

    GlobalMultipliersPreEvaluated {
        prism_production_log10: prism_rune_effects(prism_level, PrismRuneKey::ProductionLog10),
        total_coin_owned,
        ant_multiplier: ant_effect.coin_multiplier,
        recession_power: recession_power_at_level(recession_level),
        // Forwarded — upstream mechanic not yet plumbed.
        crystal_mult: fallback.crystal_mult,
        building_power: fallback.building_power,
        building_power_mult: fallback.building_power_mult,
        crystal_upgrade_3_multiplier: fallback.crystal_upgrade_3_multiplier,
        crystal_multiplier_achievement: achievement_rewards::crystal_multiplier(&ach),
        // No achievement grants `constUpgrade1Buff`/`constUpgrade2Buff` in
        // the legacy table — the additive reward is always 0.
        const_upgrade_1_buff_achievement: 0.0,
        const_upgrade_2_buff_achievement: 0.0,
        constant_ex_max_percent_increase: fallback.constant_ex_max_percent_increase,
        ascend_building_dr_value: fallback.ascend_building_dr_value,
        multiplier_effect: fallback.multiplier_effect,
        accelerator_effect: fallback.accelerator_effect,
        total_multiplier: fallback.total_multiplier,
        total_accelerator: fallback.total_accelerator,
        total_accelerator_boost: fallback.total_accelerator_boost,
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
fn phase_global_state(state: &mut GameState, cache: &CrossMechanicCache) -> AggregatorOutputs {
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

    // Inject the cross-cutting outputs into the global-multipliers bundle
    // (`total_accelerator_boost` from precompute-equivalent state; the rest
    // from the two aggregators above — all forwarded from `TackInput`
    // before).
    let mut global_multipliers_pre = cache.global_multipliers_pre;
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
        building_cost_scale_ant_upgrade_effect, coins_ant_upgrade_effect, taxes_ant_upgrade_effect,
        CoinsAntUpgradeInput,
    };
    use crate::mechanics::calculate::{calculate_total_coin_owned, CalculateTotalCoinOwnedInput};
    use crate::mechanics::challenges::{calc_ecc, ChallengeType};
    use crate::mechanics::coin_production::{
        calculate_coin_production, CalculateCoinProductionInput, PerCoinTierInput,
    };
    use crate::mechanics::crystal_and_building_power::{
        calculate_building_power, calculate_building_power_coin_multiplier,
        CalculateBuildingPowerInput,
    };
    use crate::mechanics::platonic_blessings::calculate_tax_platonic_blessing;
    use crate::mechanics::rune_effects::{
        duplication_rune_effects, thrift_rune_effects, DuplicationRuneKey, ThriftRuneKey,
    };
    use crate::mechanics::talisman_effects::exemption_talisman_effects;
    use crate::mechanics::tax::{calculate_tax, CalculateTaxInput};
    use crate::mechanics::{campaign_token_rewards, challenge_15_rewards};
    use crate::state::{RUNE_DUPLICATION, RUNE_THRIFT};

    /// Ant-upgrade indices (legacy `AntUpgrades` enum): Coins / Taxes /
    /// BuildingCostScale.
    const ANT_UPGRADE_COINS: usize = 1;
    const ANT_UPGRADE_TAXES: usize = 2;
    const ANT_UPGRADE_BUILDING_COST_SCALE: usize = 6;
    /// Exemption talisman — index 0 in the talisman ordering.
    const TALISMAN_EXEMPTION: usize = 0;

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
    let building_power = calculate_building_power(&CalculateBuildingPowerInput {
        c8_reincarnation_ecc: calc_ecc(
            ChallengeType::Reincarnation,
            challenges.challenge_completions[8],
        ),
        reincarnation_shards: state.reset_counters.reincarnation_shards,
        research_36: researches[36],
        research_37: researches[37],
        research_38: researches[38],
        building_cost_scale_ant_upgrade_building_power_mult:
            building_cost_scale_ant_upgrade_effect(
                state.ants.upgrades[ANT_UPGRADE_BUILDING_COST_SCALE],
            )
            .building_power_mult,
        cube_upgrade_12: state.cube_upgrade_levels.cube_upgrades[12],
        cube_upgrade_36: state.cube_upgrade_levels.cube_upgrades[36],
        in_reincarnation_challenge_7: challenges.current_reincarnation_challenge == 7,
    });
    let building_power_coin_multiplier =
        calculate_building_power_coin_multiplier(building_power, total_coin_owned);

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

/// State + aggregator-output-derive the [`ResourceGainPre`] fields.
///
/// Migration coverage today (`✓` = derived from state / aggregator
/// outputs, `forwarded` = caller-provided fallback):
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
/// - everything else                    forwarded (depends on reset-currency
///   gains or per-tier produce_* values not yet captured by the orchestrator)
#[must_use]
fn compute_resource_gain_pre(
    _state: &GameState,
    fallback: &ResourceGainPre,
    agg: &AggregatorOutputs,
    tax: &TaxOutputs,
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
        // Forwarded — depends on reset-currency / per-tier produce_*
        // pipelines not yet captured by the orchestrator.
        prestige_point_gain: fallback.prestige_point_gain,
        transcend_point_gain: fallback.transcend_point_gain,
        reincarnation_point_gain: fallback.reincarnation_point_gain,
        first_produce_diamonds: fallback.first_produce_diamonds,
        second_produce_diamonds: fallback.second_produce_diamonds,
        third_produce_diamonds: fallback.third_produce_diamonds,
        fourth_produce_diamonds: fallback.fourth_produce_diamonds,
        fifth_produce_diamonds: fallback.fifth_produce_diamonds,
        first_produce_mythos: fallback.first_produce_mythos,
        second_produce_mythos: fallback.second_produce_mythos,
        third_produce_mythos: fallback.third_produce_mythos,
        fourth_produce_mythos: fallback.fourth_produce_mythos,
        fifth_produce_mythos: fallback.fifth_produce_mythos,
        first_produce_particles: fallback.first_produce_particles,
        second_produce_particles: fallback.second_produce_particles,
        third_produce_particles: fallback.third_produce_particles,
        fourth_produce_particles: fallback.fourth_produce_particles,
        fifth_produce_particles: fallback.fifth_produce_particles,
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
/// Calls [`resource_gain`] with the cache's `resource_gain_pre` bundle
/// and writes the result back into the corresponding [`GameState`]
/// slices. Events emitted by `resource_gain` (achievement awards,
/// challenge auto-completions) flow into [`TickOutput::events`].
///
/// Per Ledger Finding 1, the currency fields now have a single
/// source-of-truth in `state.upgrades`; `buy_*` mutators read/write them
/// through `&mut Decimal` parameters rather than via per-slice
/// duplicates. No mid-tick sync workaround is needed.
fn phase_generation(
    state: &mut GameState,
    cache: &CrossMechanicCache,
    dt: f64,
    output: &mut TickOutput,
) {
    let result = resource_gain(state, &cache.resource_gain_pre, dt);

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
            automation_pre: AutomationPre {
                global_time_multiplier: 3.0,
                ascension_speed_multi: 5.0,
                singularity_speed_multi: 1.0,
                max_quark_timer: 90_000.0,
                export_gq_per_hour: 1.0,
                ..AutomationPre::default()
            },
            ..TackInput::default()
        };
        let output = tack(&mut state, &input);

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
            automation_pre: AutomationPre {
                ambrosia_generation_speed: 1.0,
                ambrosia_luck: 200.0,
                time_per_ambrosia: 45.0,
                ..AutomationPre::default()
            },
            ..TackInput::default()
        };
        let output = tack(&mut state, &input);

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
    fn cross_mechanic_cache_forwards_remaining_pre_bundles_from_input() {
        // The bundles still threaded through `TackInput`
        // (`global_multipliers_pre`, and `resource_gain_pre` before the tax
        // phase overrides its fields) are forwarded verbatim by precompute.
        // Pins the forwarding so a future compute-from-state migration has
        // an expected baseline.
        let state = GameState::default();
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let cache = phase_cross_mechanic_precompute(&state, &input);

        assert_eq!(
            cache.global_multipliers_pre.crystal_mult,
            input.global_multipliers_pre.crystal_mult
        );
        assert_eq!(
            cache.resource_gain_pre.produce_total,
            input.resource_gain_pre.produce_total
        );
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
}
