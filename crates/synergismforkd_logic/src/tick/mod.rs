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

use crate::events::{CoreEvent, ProducerType};
use crate::mechanics::accelerators::{buy_accelerator, BuyAcceleratorInput};
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
use crate::state::GameState;

/// Inputs to [`tack`]. Owned by the caller — `logic` has no clock, no
/// input device, no RNG seed source of its own.
///
/// The four `*_pre` bundles are caller-provided for the duration of the
/// MVP port; they collapse into a single in-orchestrator
/// `CrossMechanicCache` once the upstream mechanics (rune/ant/hepteract
/// effects, achievement rewards, challenge-15 rewards) port.
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
    /// Hand-packed pre-evaluated bundle for [`update_all_multiplier`].
    pub update_all_multiplier_pre: UpdateAllMultiplierPre,
    /// Hand-packed pre-evaluated bundle for [`update_all_tick`].
    pub update_all_tick_pre: UpdateAllTickPre,
    /// Hand-packed pre-evaluated bundle for [`resource_gain`].
    pub resource_gain_pre: ResourceGainPre,
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
/// ([`phase_cross_mechanic_precompute`]).
///
/// Per Loom's tack-design memo, the goal of the cache is to make the
/// synergy graph **legible**. The legacy TS scattered these
/// computations across the four aggregators' `*Pre` parameters, which
/// every caller hand-packed — silently dropping a field gave a working
/// tick that produced slightly less, with no compile error.
///
/// Today this struct holds the four `*Pre` bundles directly. Each
/// future commit migrates one upstream effect into compute-from-state
/// inside [`phase_cross_mechanic_precompute`], at which point the
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
    /// Pre-evaluated bundle for [`update_all_multiplier`].
    pub update_all_multiplier_pre: UpdateAllMultiplierPre,
    /// Pre-evaluated bundle for [`update_all_tick`].
    pub update_all_tick_pre: UpdateAllTickPre,
    /// Pre-evaluated bundle for [`resource_gain`].
    pub resource_gain_pre: ResourceGainPre,
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
    #[allow(dead_code)]
    update_all_multiplier: UpdateAllMultiplierResult,
    #[allow(dead_code)]
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
    // Phase 2 outputs feed Phase 4's `ResourceGainPre`. Re-compute the
    // pre now that the aggregator results are available; fields whose
    // upstream is only state still come through unchanged.
    cache.resource_gain_pre =
        compute_resource_gain_pre(state, &input.resource_gain_pre, &aggregator_outputs);
    phase_player_input(state, input, &mut output);
    phase_generation(state, &cache, input.dt, &mut output);
    if !input.time_warp {
        phase_automation(state, &cache, &mut output);
    }

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
/// ports settle. State-derivable fields are overridden here even when
/// the caller supplied them — the caller value still backs fields
/// whose upstream isn't trivially state-readable (e.g., G-cache values
/// like `taxdivisor` and `total_accelerator_boost`, which depend on
/// other aggregator outputs and need a multi-pass solution).
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
        update_all_multiplier_pre: compute_update_all_multiplier_pre(
            state,
            &input.update_all_multiplier_pre,
        ),
        update_all_tick_pre: compute_update_all_tick_pre(state, &input.update_all_tick_pre),
        resource_gain_pre: input.resource_gain_pre,
    }
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
/// - `crystal_multiplier_achievement`   forwarded (achievement-reward table)
/// - `const_upgrade_1_buff_achievement` forwarded (achievement-reward table)
/// - `const_upgrade_2_buff_achievement` forwarded (achievement-reward table)
/// - `constant_ex_max_percent_increase` forwarded (shop-effect table not ported)
/// - `ascend_building_dr_value`         forwarded (formula not yet ported)
/// - `multiplier_effect`                forwarded (G.*, cross-aggregator)
/// - `accelerator_effect`               forwarded (G.*, cross-aggregator)
/// - `total_multiplier`                 forwarded (G.*, cross-aggregator)
/// - `total_accelerator`                forwarded (G.*, cross-aggregator)
/// - `total_accelerator_boost`          forwarded (G.*, cross-aggregator)
/// - `challenge_15_coin_exponent`       forwarded (challenge-15 rewards)
/// - `challenge_15_exponent_value`      forwarded (challenge-15 rewards)
/// - `challenge_15_constant_bonus`      forwarded (challenge-15 rewards)
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
        crystal_multiplier_achievement: fallback.crystal_multiplier_achievement,
        const_upgrade_1_buff_achievement: fallback.const_upgrade_1_buff_achievement,
        const_upgrade_2_buff_achievement: fallback.const_upgrade_2_buff_achievement,
        constant_ex_max_percent_increase: fallback.constant_ex_max_percent_increase,
        ascend_building_dr_value: fallback.ascend_building_dr_value,
        multiplier_effect: fallback.multiplier_effect,
        accelerator_effect: fallback.accelerator_effect,
        total_multiplier: fallback.total_multiplier,
        total_accelerator: fallback.total_accelerator,
        total_accelerator_boost: fallback.total_accelerator_boost,
        challenge_15_coin_exponent: fallback.challenge_15_coin_exponent,
        challenge_15_exponent_value: fallback.challenge_15_exponent_value,
        challenge_15_constant_bonus: fallback.challenge_15_constant_bonus,
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
/// - `multipliers_achievement`          forwarded (achievement-reward table not ported)
/// - `total_accelerator_boost`          forwarded (G.*, cross-aggregator)
/// - `taxdivisor`                       forwarded (cross-mechanic tax pipeline)
/// - `challenge_15_reward_multiplier`   forwarded (challenge-15 rewards not ported)
#[must_use]
fn compute_update_all_multiplier_pre(
    state: &GameState,
    fallback: &UpdateAllMultiplierPre,
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
        // Forwarded — upstream mechanic not yet plumbed.
        multipliers_achievement: fallback.multipliers_achievement,
        total_accelerator_boost: fallback.total_accelerator_boost,
        taxdivisor: fallback.taxdivisor,
        challenge_15_reward_multiplier: fallback.challenge_15_reward_multiplier,
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
/// - `accelerators_achievement`         forwarded (achievement-reward table)
/// - `accelerator_power_achievement`    forwarded (achievement-reward table)
/// - `total_accelerator_boost`          forwarded (G.*, cross-aggregator)
/// - `accelerator_multiplier`           forwarded (G.*, cross-aggregator)
/// - `challenge_15_reward_accelerator`  forwarded (challenge-15 rewards)
#[must_use]
fn compute_update_all_tick_pre(state: &GameState, fallback: &UpdateAllTickPre) -> UpdateAllTickPre {
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
        // Forwarded — upstream mechanic not yet plumbed.
        accelerators_achievement: fallback.accelerators_achievement,
        accelerator_power_achievement: fallback.accelerator_power_achievement,
        total_accelerator_boost: fallback.total_accelerator_boost,
        accelerator_multiplier: fallback.accelerator_multiplier,
        challenge_15_reward_accelerator: fallback.challenge_15_reward_accelerator,
    }
}

/// **Phase 2** — Global state aggregators.
///
/// Reads the precomputed bundles out of [`CrossMechanicCache`] and runs
/// the three pure aggregators in dependency order. Their outputs flow
/// into the [`AggregatorOutputs`] return value so Phase 4
/// (`resource_gain`) can read cross-aggregator values like
/// `global_crystal_multiplier` and `mythosupgrade_13` directly,
/// instead of forwarding them from `TackInput`.
fn phase_global_state(state: &mut GameState, cache: &CrossMechanicCache) -> AggregatorOutputs {
    let global_multipliers = compute_global_multipliers(state, &cache.global_multipliers_pre);
    let update_all_multiplier_result =
        update_all_multiplier(state, &cache.update_all_multiplier_pre);
    let update_all_tick_result = update_all_tick(
        state,
        &cache.update_all_tick_pre,
        update_all_multiplier_result.total_multiplier,
    );
    AggregatorOutputs {
        global_multipliers,
        update_all_multiplier: update_all_multiplier_result,
        update_all_tick: update_all_tick_result,
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
/// - everything else                    forwarded (depends on tax
///   computation, reset-currency gains, or per-tier produce_* values
///   set elsewhere in the legacy tick)
#[must_use]
fn compute_resource_gain_pre(
    _state: &GameState,
    fallback: &ResourceGainPre,
    agg: &AggregatorOutputs,
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
        // Forwarded — depends on tax / reset-currency / per-tier
        // produce_* pipelines not yet captured by the orchestrator.
        produce_total: fallback.produce_total,
        taxdivisor: fallback.taxdivisor,
        taxdivisorcheck: fallback.taxdivisorcheck,
        maxexponent: fallback.maxexponent,
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

/// **Phase 5** — Automation (head/middle/tail).
///
/// **Status: stub.** Skipped when [`TackInput::time_warp`] is true.
/// Implementation lands as the underlying mechanics port:
/// - **Head**: quark / golden-quark / ambrosia timers
/// - **Middle**: rune sacrifice, ant sacrifice, addObtainium, auto-research
/// - **Tail**: addOfferings, challenge sweep state machine, auto-reset
///
/// Each sub-phase is gated on its own auto-toggle flag; the orchestrator
/// reads those flags from `state` slices that haven't been wired yet.
fn phase_automation(_state: &mut GameState, _cache: &CrossMechanicCache, _output: &mut TickOutput) {
    // Intentionally empty. See module docs.
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
        // Default state has zero of everything — no events should fire.
        assert!(output.events.is_empty());
    }

    #[test]
    fn cross_mechanic_cache_overrides_state_derived_fields() {
        // State-derived fields ignore the caller's *Pre values. A
        // duplication rune at level 800 raises
        // `multiplicative_multipliers_rune` from the identity 1.0 to
        // `1 + 800/400 = 3.0`, regardless of what the caller supplied.
        let mut state = GameState::default();
        state.runes.rune_levels[crate::state::RUNE_DUPLICATION] = 800.0;
        state.hepteracts.multiplier.bal = 5.0; // hept-multiplier
        state.runes.rune_levels[crate::state::RUNE_SPEED] = 400.0;
        state.hepteracts.accelerator.bal = 10.0;

        // Caller passes garbage values; the state-derived fields must
        // ignore them.
        let input = TackInput {
            dt: 0.025,
            update_all_multiplier_pre: UpdateAllMultiplierPre {
                multiplicative_multipliers_rune: 99.0, // ignored
                hepteract_multiplier: 99.0,            // ignored
                ..UpdateAllMultiplierPre::default()
            },
            update_all_tick_pre: UpdateAllTickPre {
                multiplicative_accelerators_rune: 99.0, // ignored
                hepteract_accelerators: 99.0,           // ignored
                ..UpdateAllTickPre::default()
            },
            ..TackInput::default()
        };

        let cache = phase_cross_mechanic_precompute(&state, &input);

        // Duplication rune at 800: 1 + 800/400 = 3.0.
        assert!(
            (cache
                .update_all_multiplier_pre
                .multiplicative_multipliers_rune
                - 3.0)
                .abs()
                < 1e-9
        );
        // Hept-multiplier at 5: 1000 * 5 = 5000.
        assert!((cache.update_all_multiplier_pre.hepteract_multiplier - 5_000.0).abs() < 1e-9);

        // Speed rune at 400: 1 + 400/400 = 2.0.
        assert!((cache.update_all_tick_pre.multiplicative_accelerators_rune - 2.0).abs() < 1e-9);
        // Hept-accelerator at 10: 2000 * 10 = 20_000.
        assert!((cache.update_all_tick_pre.hepteract_accelerators - 20_000.0).abs() < 1e-9);
    }

    #[test]
    fn cross_mechanic_cache_forwards_pre_bundles_from_input() {
        // Today the cache is built by copying the four *Pre bundles
        // from TackInput. Future commits will replace these with
        // compute-from-state calls; this test pins the forwarding
        // behavior so a future "did I wire the new compute function
        // correctly?" can compare against expected forwarded values.
        let state = GameState::default();
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let cache = phase_cross_mechanic_precompute(&state, &input);

        // Default-equality check on each *Pre field — forwarding
        // preserves bit-equal values from input to cache.
        assert_eq!(
            cache.global_multipliers_pre.crystal_mult,
            input.global_multipliers_pre.crystal_mult
        );
        assert_eq!(
            cache.update_all_multiplier_pre.multipliers_achievement,
            input.update_all_multiplier_pre.multipliers_achievement
        );
        assert_eq!(
            cache.update_all_tick_pre.accelerators_achievement,
            input.update_all_tick_pre.accelerators_achievement
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
        assert_eq!(out_a.events.len(), out_b.events.len());
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
}
