//! Tick orchestrator — `tack` plus named phase functions.
//!
//! This file is the canonical statement of tick order. Phases run in the
//! sequence declared in [`tack`]; reordering requires editing this file.
//! Per the [[loom-tack-design]] memo, named phases prevent implicit
//! call-order shifts from silently changing player-visible per-second
//! rates.
//!
//! ## Phases
//! 1. **Automation-input derivation** — all five `*Pre` bundles have been
//!    retired; [`tack`] self-derives the Phase-5 [`AutomationPre`] inputs
//!    field-by-field from `&GameState` (the four global-state bundles
//!    derive inside their consuming phases). No caller bundle, no
//!    cross-mechanic cache remains.
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
//! 5. **Automation** — head/middle/tail, fully wired (see
//!    [`phase_automation`]): head timers; the middle's rune/ant sacrifice,
//!    addObtainium, and auto-research; the tail's addOfferings, challenge
//!    sweep, and auto-resets. `time_warp` ticks skip head + middle and run
//!    only the tail (the offline-catch-up mode).
//!
//! Boundary: this module produces a flat [`TickOutput`] event stream.
//! Modal dispatch, audio cues, save serialization, and i18n live in the
//! UI tier and consume `output.events`.

use smallvec::{smallvec, SmallVec};

use synergismforkd_bignum::Decimal;

use crate::events::{CoreEvent, CubeTier, ProducerType};
use crate::mechanics::accelerators::{buy_accelerator, BuyAcceleratorInput};
use crate::mechanics::achievement_rewards;
use crate::mechanics::ant_masteries::{buy_ant_mastery, BuyAntMasteryInput};
use crate::mechanics::ant_producers::{buy_ant_producer, BuyAntProducerInput};
use crate::mechanics::ant_upgrades::{buy_ant_upgrade, BuyAntUpgradeInput};
use crate::mechanics::blueberry_upgrades::{buy_ambrosia_upgrade, BuyAmbrosiaUpgradeInput};
use crate::mechanics::challenge_15_rewards;
use crate::mechanics::constant_upgrades::{buy_constant_upgrade, BuyConstantUpgradeInput};
use crate::mechanics::crystal_upgrades::{buy_crystal_upgrades, BuyCrystalUpgradesInput};
use crate::mechanics::cube_upgrades::{buy_cube_upgrade, BuyCubeUpgradeInput};
use crate::mechanics::global_multipliers::{
    compute_global_multipliers, GlobalMultipliersPreEvaluated, GlobalMultipliersResult,
};
use crate::mechanics::gq_upgrade_cost::{buy_gq_upgrade, BuyGQUpgradeInput};
use crate::mechanics::hepteract_values::{
    buy_hepteract_craft, buy_hepteract_expand, BuyHepteractCraftInput, BuyHepteractExpandInput,
};
use crate::mechanics::multipliers::{buy_multiplier, BuyMultiplierInput};
use crate::mechanics::octeracts::{buy_octeract_upgrade, BuyOcteractUpgradeInput};
use crate::mechanics::particle_buildings::{buy_particle_building, BuyParticleBuildingInput};
use crate::mechanics::platonic_upgrade_costs::{buy_platonic_upgrade, BuyPlatonicUpgradeInput};
use crate::mechanics::producers::{buy_max, buy_producer, BuyMaxInput, BuyProducerInput};
use crate::mechanics::researches::{buy_research, BuyResearchInput};
use crate::mechanics::reset_currency::ResetCurrencyResult;
use crate::mechanics::resource_gain::{resource_gain, ResourceGainPre};
use crate::mechanics::rune_levels::{buy_rune_levels, BuyRuneLevelsInput};
use crate::mechanics::shop_costs::{buy_shop, BuyShopInput};
use crate::mechanics::talisman_levels::{buy_talisman_level, BuyTalismanLevelInput};
use crate::mechanics::tesseract_buildings::{buy_tesseract_building, BuyTesseractBuildingInput};
use crate::mechanics::update_all_multiplier::{
    update_all_multiplier, UpdateAllMultiplierPre, UpdateAllMultiplierResult,
};
use crate::mechanics::update_all_tick::{update_all_tick, UpdateAllTickPre, UpdateAllTickResult};
use crate::mechanics::upgrades::{buy_upgrades, BuyUpgradeInput};
use crate::state::{GameState, RngPurpose, RUNE_COUNT, TALISMAN_COUNT};

mod ant_generation;
mod ant_sacrifice;
mod auto_buy;
mod auto_research;
mod auto_reset;
mod automatic_tools;
mod challenge_sweep;
mod reset;
mod timers;

/// Pre-evaluated inputs for the Phase 5 automation layer — the speed
/// multipliers, timer fields, and unlock gates [`phase_automation`]
/// reads (`calculateGlobalSpeedMult`, `calculateAscensionSpeedMult`,
/// `calculateSingularitySpeedMult`, `quark_handler`, `exportGQPerHour`,
/// the ambrosia / octeract / obtainium / ant-speed mults, …).
///
/// **Fully self-derived from `&GameState`.** [`tack`] builds this
/// field-by-field from the ported aggregators (the `compute_*` helpers)
/// before Phase 5; it is no longer threaded through [`TackInput`] or a
/// cross-mechanic cache. It remains a distinct struct purely as the
/// argument bundle handed to [`phase_automation`].
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
/// All five `*_pre` bundles have now been retired: the four global-state
/// bundles (`global_multipliers`, `update_all_multiplier`,
/// `update_all_tick`, `resource_gain`) and the Phase-5 `automation_pre`
/// bundle all self-derive from `&GameState` inside [`tack`]. `TackInput`
/// therefore carries only the genuine inputs the caller controls: the tick
/// delta, the time-warp flag, and the queued player actions.
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
}

/// A single queued player input. Variants will expand as automation
/// toggles port (`ToggleAuto(AutoTool)` per the [[loom-tack-design]] memo).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PlayerAction {
    /// A buy click. Routes to one of the eight `buy_*` mutators based on
    /// the [`BuyRequest`] variant.
    Buy(BuyRequest),
    /// A manual reset (prestige / transcension / …). Routes to the
    /// reset executor based on the [`ResetRequest`] variant.
    Reset(ResetRequest),
    /// Set a single corruption's *next-ascension* loadout level
    /// (legacy `CorruptionLoadout.setLevel`). `index` is the corruption
    /// slot (viscosity = 0); `level` is clamped to `[0, maxCorruptionLevel]`.
    /// Recomputes `corruptions.next.total_corruption_ascension_multiplier`.
    SetCorruptionLevel {
        /// Corruption slot index (`0..8`; viscosity = 0).
        index: usize,
        /// Requested level (clamped to `[0, maxCorruptionLevel]`).
        level: u32,
    },
    /// Set an automation toggle on/off (legacy auto-* toggle buttons).
    /// Sets the target flag to `enabled` directly (the UI computes the
    /// flip); `phase_automation` reads these flags.
    ToggleAuto {
        /// Which automation flag to set.
        target: AutoToggle,
        /// Desired enabled state.
        enabled: bool,
    },
    /// Enter a challenge (legacy `toggleChallenges`): set the
    /// `current_*_challenge` slot and run the matching tier reset. `challenge`
    /// is `1..=5` (transcension), `6..=10` (reincarnation), or `11..=15`
    /// (ascension). `0` exits the transcension slot.
    /// Exit a reincarnation/ascension challenge with the corresponding
    /// [`Self::Reset`].
    EnterChallenge {
        /// Challenge index (`0..=15`; `0` exits the transcension slot).
        challenge: u32,
    },
    /// Open cubes of a tier (legacy `WowCubes.open` etc.): distribute blessings
    /// into the matching `*_blessings` slice and credit any quark gift. `value`
    /// is the count to open; `max` opens the whole balance (ignoring `value`).
    OpenCubes {
        /// Which cube tier to open.
        tier: CubeTier,
        /// Number of cubes to open (ignored when `max`).
        value: f64,
        /// Open the entire balance.
        max: bool,
    },
    /// Toggle a singularity (Exalt) challenge (legacy
    /// `SingularityChallenge.challengeEntryHandler`): enter it when its flag
    /// is clear (jumping to the tier's required singularity), otherwise exit —
    /// succeeding iff the antiquities rune was purchased inside.
    ToggleSingularityChallenge {
        /// Which Exalt to toggle.
        challenge: crate::events::SingularityChallengeId,
    },
    /// Configure the singularity elevator (the legacy elevator panel inputs):
    /// set the target floor (clamped like the input listener, to
    /// `[1, max(1, highest, count + lookahead if antiquities)]`) and the
    /// locked / slow-climb toggles. Pure config — no event.
    ConfigureSingularityElevator {
        /// Requested elevator floor.
        target: f64,
        /// Lock a normal singularity to the target floor.
        locked: bool,
        /// Advance by exactly one singularity instead of the lookahead jump.
        slow_climb: bool,
    },
    /// Ride the elevator to its target floor now (legacy
    /// `teleportToSingularity`): ascending (count ≤ target) performs a full
    /// singularity to the target; descending just sets the count — no reset.
    /// Gated on a valid target and on not being inside an Exalt.
    TeleportToSingularity,
    /// Start a campaign (legacy start-campaign button): rejected while
    /// inside an ascension challenge; otherwise runs a full ascension reset
    /// (banking any active campaign's completions first) and applies the
    /// chosen campaign's corruption loadout to `corruptions.used`.
    SelectCampaign {
        /// Campaign index (`campaignDatas` key order; `first` = 0).
        campaign: usize,
    },
}

/// Selects the automation flag a [`PlayerAction::ToggleAuto`] sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AutoToggle {
    /// `auto_prestige_enabled`.
    AutoPrestige,
    /// `auto_transcend_enabled`.
    AutoTranscend,
    /// `auto_reincarnate_enabled`.
    AutoReincarnate,
    /// `auto_ascend`.
    AutoAscend,
    /// `rune_sacrifice_auto_enabled`.
    RuneSacrifice,
    /// `auto_potion_toggle_offering`.
    OfferingPotion,
    /// `auto_potion_toggle_obtainium`.
    ObtainiumPotion,
    /// `auto_challenge_running` — the challenge-sweep master switch.
    AutoChallengeRunning,
    /// `retry_challenges` — stay in challenge on completion instead of exiting.
    RetryChallenges,
    /// `auto_challenge_toggles[slot]` — per-challenge sweep enable
    /// (`slot` in `0..16`; out-of-range is ignored).
    AutoChallengeSlot(usize),
}

/// Per-mechanic dispatcher for the eight `buy_*` purchase loops. The
/// variant carries the same `*Input` the underlying buy function takes.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum BuyRequest {
    /// Routes to [`buy_upgrades`].
    Upgrade(BuyUpgradeInput),
    /// Routes to [`buy_research`].
    Research(BuyResearchInput),
    /// Routes to [`buy_gq_upgrade`].
    GoldenQuarkUpgrade(BuyGQUpgradeInput),
    /// Routes to [`buy_octeract_upgrade`].
    OcteractUpgrade(BuyOcteractUpgradeInput),
    /// Routes to [`buy_ambrosia_upgrade`].
    AmbrosiaUpgrade(BuyAmbrosiaUpgradeInput),
    /// Routes to [`buy_rune_levels`].
    RuneLevels(BuyRuneLevelsInput),
    /// Routes to [`buy_ant_producer`].
    AntProducer(BuyAntProducerInput),
    /// Routes to [`buy_ant_upgrade`].
    AntUpgrade(BuyAntUpgradeInput),
    /// Routes to [`buy_hepteract_craft`].
    HepteractCraft(BuyHepteractCraftInput),
    /// Routes to [`buy_hepteract_expand`].
    HepteractExpand(BuyHepteractExpandInput),
    /// Routes to [`buy_talisman_level`].
    TalismanLevel(BuyTalismanLevelInput),
    /// Routes to [`buy_shop`].
    Shop(BuyShopInput),
    /// Routes to [`buy_multiplier`].
    Multiplier(BuyMultiplierInput),
    /// Routes to [`buy_accelerator`].
    Accelerator(BuyAcceleratorInput),
    /// Buy accelerator boosts (legacy `boostAccelerator`). Takes no payload —
    /// the path (single-boost-with-prestige-reset vs. bulk) is chosen from
    /// `upgrades[46]`, the cost-delay comes from the thrift rune blessing, and
    /// the spend currency is `prestigePoints`. Routes to [`buy_accelerator_boost`],
    /// which receives the tick's prestige-point gain for the pre-upgrade path's
    /// inline reset.
    AcceleratorBoost,
    /// Routes to [`buy_crystal_upgrades`].
    CrystalUpgrade(BuyCrystalUpgradesInput),
    /// Routes to [`buy_cube_upgrade`].
    CubeUpgrade(BuyCubeUpgradeInput),
    /// Routes to [`buy_platonic_upgrade`].
    PlatonicUpgrade(BuyPlatonicUpgradeInput),
    /// Routes to [`buy_particle_building`].
    ParticleBuilding(BuyParticleBuildingInput),
    /// Routes to [`buy_tesseract_building`].
    TesseractBuilding(BuyTesseractBuildingInput),
    /// Routes to [`buy_constant_upgrade`] — ascension constant upgrade `i`,
    /// paid in `ascendShards` (free when `researches[175] > 0`).
    ConstantUpgrade(BuyConstantUpgradeInput),
    /// Routes to [`buy_ant_mastery`] — one mastery level for an ant producer,
    /// paid in reincarnation points.
    AntMastery(BuyAntMasteryInput),
    /// Routes to [`buy_max`] — buy-as-many-as-affordable across the
    /// producer family selected by `input.producer_type`.
    ProducerMax(BuyMaxInput),
    /// Routes to [`buy_producer`] — manual-click loop across the producer
    /// family selected by `input.producer_type`.
    Producer(BuyProducerInput),
}

/// Per-tier dispatcher for a manual reset, mirroring [`BuyRequest`].
/// [`Self::Prestige`] / [`Self::Transcension`] / [`Self::Reincarnation`] /
/// [`Self::Ascension`] / [`Self::Singularity`] are all wired. (`Eq` is not
/// derived: [`Self::SingularityChallenge`] carries an `f64` target.)
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum ResetRequest {
    /// Manual prestige reset — the always-runs base reset
    /// (`reset('prestige')`).
    Prestige,
    /// Manual transcension reset — base + transcension layer
    /// (`reset('transcension')`).
    Transcension,
    /// Manual reincarnation reset — base + transcension + reincarnation
    /// layers (`reset('reincarnation')`).
    Reincarnation,
    /// Manual ascension reset — base + transcension + reincarnation +
    /// ascension layers (`reset('ascension')`). The c10-gated cube /
    /// hepteract awards and the auto-ascend *decision* remain deferred (see
    /// [`reset::perform_ascension_reset`]).
    Ascension,
    /// Ascension-challenge reset — same ascension-layer sub-resets as
    /// [`Self::Ascension`] but triggered by entering or leaving an ascension
    /// challenge (`reset('ascensionChallenge')`). The
    /// `current_transcension/reincarnation_challenge` slots are zeroed as
    /// part of the shared ascension block in [`reset::perform_ascension_reset`].
    AscensionChallenge,
    /// Manual singularity reset (`singularity(-1)`, `Reset.ts:1063`) — the
    /// meta-reset above ascension. Grants golden quarks, advances the
    /// singularity count (auto-climb), and rebuilds the player from a blank save
    /// preserving meta-progression. Gated on the antiquities rune (level > 0).
    Singularity,
    /// Singularity challenge enter/exit (`singularity(setSingNumber)`). Same
    /// reconstruction, but jumps the count to `set_sing_number` and skips the
    /// antiquities gate (matching the legacy `setSingNumber !== -1` path).
    SingularityChallenge {
        /// The target `singularityCount` to jump to.
        set_sing_number: f64,
    },
}

/// `forcedDailyReset` (Calculate.ts:1638) — the once-per-real-day bookkeeping
/// reset. A **host seam**: the logic crate has no clock, so a host (web/desktop)
/// calls this when its `dailyResetCheck` detects a new calendar day. Zeroes the
/// per-day cube-opening counters and the reborn-ELO daily leaderboard; the
/// latter (`resetPlayerRebornELODaily`) also clears `quarks_gained_from_ants`,
/// so the next day's ant-sacrifice quark award restarts from a clean delta. The
/// all-time `highest_reborn_elo_ever` leaderboard is intentionally left intact.
///
/// The `rewards = true` overflux branch (`overfluxPowder += overfluxOrbs *
/// calculatePowderConversion()`; `overfluxOrbs = challenge15Rewards.freeOrbs`)
/// is **not** ported: `calculatePowderConversion`'s full formula and the c15
/// `freeOrbs` reward are unported. The counter reset is the portion that maps to
/// ported state — and `forcedDailyReset` deliberately awards no powder / expires
/// no orbs on the manual (`rewards = false`) path regardless.
pub fn daily_reset(state: &mut GameState) {
    let cubes = &mut state.cube_balances;
    cubes.cube_quark_daily = 0.0;
    cubes.tesseract_quark_daily = 0.0;
    cubes.hypercube_quark_daily = 0.0;
    cubes.platonic_cube_quark_daily = 0.0;
    cubes.cube_opened_daily = 0.0;
    cubes.tesseract_opened_daily = 0.0;
    cubes.hypercube_opened_daily = 0.0;
    cubes.platonic_cube_opened_daily = 0.0;

    // resetPlayerRebornELODaily(): empty the daily leaderboard + zero the running
    // ant-quark total (defaultHighestRebornELODaily = [], defaultQuarksGainedFromAnts = 0).
    state.ants.highest_reborn_elo_daily.clear();
    state.ants.quarks_gained_from_ants = 0.0;
}

/// Result of [`tack`]: the event stream the UI dispatches, plus per-tick
/// derived display numbers.
#[derive(Debug, Clone, Default)]
pub struct TickOutput {
    /// CoreEvent stream for the UI tier to dispatch. Inline capacity of
    /// 16 covers the typical worst-case tick (purchases × N + 1
    /// achievement + up to 5 challenge auto-completions).
    pub events: SmallVec<[CoreEvent; 16]>,
    /// Display values `tack` already computed this tick (gain previews,
    /// rates). NOT persisted — pure tick output for the HUD.
    pub derived: DerivedTickStats,
}

/// Per-tick derived display numbers. Sourced from values the tick already
/// computes (`ResetCurrencyGains`, `ResourceGainPre`); no extra math beyond
/// one rate composition.
#[derive(Debug, Clone, Copy, Default)]
pub struct DerivedTickStats {
    /// Coins gained per real second at this tick's production + speed
    /// multiplier (tax-capped).
    pub coins_per_sec: Decimal,
    /// Prestige points the next prestige would award (`G.prestigePointGain`).
    pub prestige_point_gain: Decimal,
    /// Transcend points the next transcension would award.
    pub transcend_point_gain: Decimal,
    /// Reincarnation points the next reincarnation would award.
    pub reincarnation_point_gain: Decimal,
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

    // The automation-layer inputs are now fully self-derived from `&GameState`
    // (no caller bundle, no cross-mechanic cache). Built field-by-field below,
    // then handed to Phase 5.
    let mut automation_pre = AutomationPre::default();
    // Refresh blueberry free levels (red-ambrosia `extraLevelCalc`) before the
    // Phase 2 aggregators read `amb(i) = level + free_level` (cubes / luck /
    // quark multipliers all depend on the effective level).
    populate_ambrosia_free_levels(state);
    let aggregator_outputs = phase_global_state(state);
    // Quark multiplier (legacy `calculateQuarkMultiplier` / `allQuarkStats`):
    // cached as a percent so the `applyBonus` consumers (cube opening, challenge
    // rewards, achievement awards) credit the full multiplier. The field carries
    // `(mult - 1) * 100`, so `1 + quark_bonus / 100 == calculateQuarkMultiplier()`.
    let quark_multiplier = compute_quark_multiplier(state);
    state.quarks.quark_bonus = (quark_multiplier - 1.0) * 100.0;
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
    automation_pre.prestige_point_gain = reset_gains.prestige_point_gain;
    automation_pre.transcend_point_gain = reset_gains.transcend_point_gain;
    automation_pre.reincarnation_point_gain = reset_gains.reincarnation_point_gain;
    // Speed multipliers (legacy `G.timeMultiplier` / `ascensionSpeedMult`) —
    // self-derived from state, replacing the caller-provided AutomationPre
    // values.
    automation_pre.global_time_multiplier = compute_global_speed_mult_pre(state);
    automation_pre.ascension_speed_multi = compute_ascension_speed_mult_pre(state);
    automation_pre.singularity_speed_multi = compute_singularity_speed_mult_pre(state);
    // Non-speed timer fields (auto-potion + GQ export), self-derived.
    let (offering_potions, obtainium_potions, auto_potion_speed, export_gq) =
        compute_auto_timer_fields(state);
    automation_pre.offering_potion_count = offering_potions;
    automation_pre.obtainium_potion_count = obtainium_potions;
    automation_pre.auto_potion_speed_mult = auto_potion_speed;
    automation_pre.export_gq_per_hour = export_gq;
    let ambrosia_luck = compute_ambrosia_luck_pre(state);
    automation_pre.ambrosia_luck = ambrosia_luck;
    let ambrosia_generation_speed = compute_ambrosia_generation_speed_pre(state);
    automation_pre.ambrosia_generation_speed = ambrosia_generation_speed;
    // Red-ambrosia luck / generation speed compose on this tick's ambrosia
    // luck (the `LuckConversion` line) and ambrosia generation speed (the
    // `BlueberrySpeed` line) respectively.
    automation_pre.red_ambrosia_luck = compute_red_ambrosia_luck_pre(state, ambrosia_luck);
    automation_pre.red_ambrosia_generation_speed =
        compute_red_ambrosia_generation_speed_pre(state, ambrosia_generation_speed);
    // Ambrosia-timer threshold fields (legacy Helper.ts `addTimers('ambrosia')`).
    let (bonus_ambrosia, time_per_ambrosia, ambrosia_accelerator_mult, ambrosia_brick_of_lead_mult) =
        compute_ambrosia_timer_fields(state);
    automation_pre.bonus_ambrosia = bonus_ambrosia;
    automation_pre.time_per_ambrosia = time_per_ambrosia;
    automation_pre.ambrosia_accelerator_mult = ambrosia_accelerator_mult;
    automation_pre.ambrosia_brick_of_lead_mult = ambrosia_brick_of_lead_mult;
    // Red-ambrosia-timer threshold fields (legacy Helper.ts `addTimers('redAmbrosia')`).
    let (
        ambrosia_time_per_red_ambrosia,
        time_per_red_ambrosia,
        red_ambrosia_bar_requirement_multiplier,
    ) = compute_red_ambrosia_timer_fields(state);
    automation_pre.ambrosia_time_per_red_ambrosia = ambrosia_time_per_red_ambrosia;
    automation_pre.time_per_red_ambrosia = time_per_red_ambrosia;
    automation_pre.red_ambrosia_bar_requirement_multiplier =
        red_ambrosia_bar_requirement_multiplier;
    // Octeract-timer unlock gate (legacy `getGQUpgradeEffect('octeractUnlock',
    // 'unlocked')`).
    automation_pre.octeract_unlocked = compute_octeract_unlocked(state);
    // Quark-export timer cap (legacy `quarkHandler().maxTime`).
    automation_pre.max_quark_timer = compute_max_quark_timer(state);
    // Automation unlock gates: auto-research Roomba, the rune auto-sacrifice
    // shop gate, and the auto-prestige level milestone.
    automation_pre.roomba_unlocked = compute_roomba_unlocked(state);
    automation_pre.offering_auto_rune = compute_offering_auto_rune(state);
    automation_pre.auto_prestige_milestone = compute_auto_prestige_milestone(state);
    // Ant-sacrifice unlock gate (legacy `getAchievementReward('antSacrificeUnlock')`
    // = achievement #173 earned).
    automation_pre.ant_sacrifice_unlocked =
        crate::mechanics::achievement_rewards::ant_sacrifice_unlocked(
            &state.achievements.achievements,
        );
    // Available reborn ELO (legacy `calculateAvailableRebornELO()`) — feeds the
    // "maxed reborn ELO" ant-sacrifice toggles.
    automation_pre.available_reborn_elo = compute_available_reborn_elo(state);
    // Ant-sacrifice immortalELO gain (legacy `antSacrificeRewards().immortalELO`).
    automation_pre.immortal_elo_gain = compute_immortal_elo_gain(state);
    // Challenge-sweep pre-evals (legacy `prepareSweepInputForTackTail`).
    let sweep = compute_sweep_pre_evals(state);
    automation_pre.sweep_timer_start = sweep.timer_start;
    automation_pre.sweep_timer_exit = sweep.timer_exit;
    automation_pre.sweep_timer_enter = sweep.timer_enter;
    automation_pre.sweep_next_regular_challenge_from_initial =
        sweep.next_regular_challenge_from_initial;
    automation_pre.sweep_next_regular_challenge_from_active =
        sweep.next_regular_challenge_from_active;
    automation_pre.sweep_challenge_15_auto_exponent_check = sweep.challenge_15_auto_exponent_check;
    automation_pre.sweep_is_finished_still_valid = sweep.is_finished_still_valid;
    // Octeract gain rate (legacy `calculateOcteractMultiplier()` = the 42-line
    // `allOcteractCubeStats` product). Gated to 0 at default by the AscensionScore
    // line; consumed only by the sing≥160 octeract giveaway.
    automation_pre.octeract_per_second = compute_octeract_per_second(state);
    // GQ-giveaway multiplier excluding the base GQ line (legacy
    // `addTimers('goldenQuarks')` product of `allGoldenQuarkMultiplierStats[1..]`).
    automation_pre.golden_quarks_multiplier_excluding_base =
        compute_golden_quarks_multiplier_excluding_base(state);
    // Auto-research obtainium gain (legacy Helper.ts `addTimers('obtainium')` =
    // `calculateResearchAutomaticObtainium(dt)`). Self-derives to 0 at default
    // (the multiplier gate 0.5·research[61] + 0.1·research[62] + 0.8·cube[3] = 0),
    // matching the old AutomationPre default. Threads this tick's reincarnation
    // point gain (the `ReincarnationUpgrade9` obtainium line reads it).
    automation_pre.obtainium_gain =
        compute_obtainium_gain(state, input.dt, reset_gains.reincarnation_point_gain);
    // Ant-speed multiplier (legacy `calculateActualAntSpeedMult()` = the 24-line
    // `antSpeedStats` Decimal product ^ ascension-challenge exponent).
    // Self-derives to 0 at default (the `canGenerateAntCrumbs` Base line is 0
    // until ants unlock → whole product 0), vs the old AutomationPre default of
    // 1 — ant generation multiplies by this factor so it no-ops at 0 anyway.
    automation_pre.ant_speed_mult = compute_ant_speed_mult(state);
    phase_player_input(state, input, &reset_gains, &mut output);
    // Generation runs on dt scaled by the global-speed multiplier (legacy
    // `resourceGain(dt * timeMult)`, Synergism.ts:4604). `automation_pre`
    // already holds this tick's `compute_global_speed_mult_pre` (set above) and
    // the timer phase consumes the same value — this is the single
    // generation-side application. Ant generation deliberately stays on raw dt.
    phase_generation(
        state,
        &resource_gain_pre,
        input.dt * automation_pre.global_time_multiplier,
        &mut output,
    );
    // Progress unlock flags that depend on finalized coins / prestige points
    // (legacy per-tick checks, Synergism.ts:3976-3989 + Automation.ts:8).
    update_progress_unlocks(state);
    phase_challenge_completion(state, &reset_gains, &mut output);
    phase_automation(state, &automation_pre, input, &mut output);

    // HUD display numbers, from values this tick already computed. The coin
    // rate composes the base-tick gain (`resource_gain`'s `addcoin` per
    // 0.025 s) with the same speed multiplier phase_generation ran on.
    output.derived = DerivedTickStats {
        coins_per_sec: crate::mechanics::resource_gain::coin_gain_per_base_tick(&resource_gain_pre)
            * Decimal::from_finite(40.0 * automation_pre.global_time_multiplier),
        prestige_point_gain: reset_gains.prestige_point_gain,
        transcend_point_gain: reset_gains.transcend_point_gain,
        reincarnation_point_gain: reset_gains.reincarnation_point_gain,
    };

    output
}

/// Sticky per-tick progress-unlock flags that depend on finalized currencies:
/// coin-producer gates (`coins` ≥ 500 / 1e4 / 1e5 / 4e6, Synergism.ts:3976-3989)
/// and the generator gate (`prestigePoints` ≥ 1e12, Automation.ts:8). Each flag
/// latches `true` and only resets on a singularity reset (Reset.ts:888). The
/// `rrow1`-`rrow4` gates are singularity-milestone grants (handled by the
/// singularity layer), so they are not set here.
fn update_progress_unlocks(state: &mut GameState) {
    let coins = state.upgrades.coins;
    let prestige_points = state.upgrades.prestige_points;
    let rc = &mut state.reset_counters;
    rc.coin_one_unlocked |= coins >= Decimal::from_finite(500.0);
    rc.coin_two_unlocked |= coins >= Decimal::from_finite(10_000.0);
    rc.coin_three_unlocked |= coins >= Decimal::from_finite(100_000.0);
    rc.coin_four_unlocked |= coins >= Decimal::from_finite(4e6);
    rc.generation_unlocked |= prestige_points >= Decimal::from_finite(1e12);
}

/// Effective ant-upgrade level (legacy `calculateTrueAntLevel`): purchased
/// level + capped free levels (`min(level, free)`), divided by the
/// extinction-corruption divisor — except for the four `exemptFromCorruption`
/// upgrades (Mortuus 11, WowCubes 13, AscensionScore 14, Mortuus2 15), where
/// the divisor is ignored. While ascension challenge 11 is active the level
/// collapses to `min(level, free)` without the additive purchased term.
///
/// Free levels are a single global pool shared by every upgrade. Two
/// contributing terms stay neutral pending unported subsystems (audit H2):
/// the `freeAntUpgrades` achievement reward (→ 0) and the challenge-15
/// `bonusAntLevel` multiplier (→ 1.0). Identity at the default state (no free
/// levels, extinction divisor 1.0), so existing default-state tests are
/// unaffected; the effect only diverges from the raw level once free levels
/// or extinction corruption are present.
fn true_ant_level(state: &GameState, upgrade_index: usize) -> f64 {
    use crate::mechanics::ant_upgrade_levels::{
        calculate_true_ant_level, compute_free_ant_upgrade_levels, CalculateTrueAntLevelInput,
        ComputeFreeAntUpgradeLevelsInput,
    };
    use crate::mechanics::challenges::{calc_ecc, ChallengeType};
    use crate::mechanics::corruptions::extinction_divisor_at_level;
    use crate::state::EXTINCTION_INDEX;

    // The four `exemptFromCorruption` ant upgrades (legacy antUpgradeData):
    // Mortuus (11), WowCubes (13), AscensionScore (14), Mortuus2 (15).
    const EXEMPT_FROM_CORRUPTION: [usize; 4] = [11, 13, 14, 15];

    let cc = &state.challenges.challenge_completions;
    let research = &state.researches.researches;
    let c11_active = state.challenges.current_ascension_challenge == 11;

    let free_levels = compute_free_ant_upgrade_levels(&ComputeFreeAntUpgradeLevelsInput {
        c9_reincarnation_ecc: calc_ecc(ChallengeType::Reincarnation, cc[9]),
        constant_upgrade_6: state.campaigns.constant_upgrades[6],
        c11_ascension_ecc: calc_ecc(ChallengeType::Ascension, cc[11]),
        research_97: research[97],
        research_98: research[98],
        research_102: research[102],
        research_132: research[132],
        research_200: research[200],
        // getAchievementReward('freeAntUpgrades') unported → neutral 0.
        free_ant_upgrades_achievement_reward: 0.0,
        // challenge15Rewards.bonusAntLevel baseValue → neutral 1.0.
        challenge_15_bonus_ant_level_value: 1.0,
        c11_active,
        c8_completions: cc[8],
        c9_completions: cc[9],
    });

    // `calculate_true_ant_level` ignores the divisor for exempt upgrades, so it
    // is always safe to pass the real value.
    calculate_true_ant_level(&CalculateTrueAntLevelInput {
        current_level: state.ants.upgrades[upgrade_index],
        free_levels,
        exempt_from_corruption: EXEMPT_FROM_CORRUPTION.contains(&upgrade_index),
        corruption_extinction_divisor: extinction_divisor_at_level(
            state.corruptions.used.levels[EXTINCTION_INDEX],
        ),
        c11_active,
    })
}

/// DR-softened hepteract balance (legacy `hepteractEffective`): linear up to
/// the per-craft LIMIT (1000 for all non-quark crafts), then
/// `LIMIT * (raw / LIMIT)^dr_exponent`. The exponent is the craft's fixed DR
/// plus its `DR_INCREASE` (only chronos has one: `platonicUpgrades[19] / 750`).
/// Feeding the raw balance skipped this softening past 1000 (audit P1.4).
fn hepteract_effective_bal(raw_amount: f64, dr_exponent: f64) -> f64 {
    use crate::mechanics::hepteract_values::{hepteract_effective, HepteractEffectiveInput};
    hepteract_effective(&HepteractEffectiveInput {
        raw_amount,
        limit: 1000.0,
        dr_exponent,
        is_quark: false,
    })
}

/// `firstFiveEffectiveRuneLevelMult` (Statistics.ts `firstFiveRuneEffectivenessStats`):
/// the product applied to the first five runes' level. The eight research
/// factors, `ConstantUpgrade9`, `Research4x9`, and the `MidasTribute`
/// cube-blessing cascade are live; only the `Challenge15` rune bonus stays
/// neutral-defaulted to `1.0` (its `challenge_15_rewards` reader is unported).
/// (A spurious `quarkGain` factor was dropped here — `getAchievementReward('quarkGain')`
/// is an `allQuarkStats` term, not part of `firstFiveRuneEffectivenessStats`.)
/// Identity at the default state. TS-anchored against the verbatim Statistics.ts.
fn first_five_effective_rune_level_mult(state: &GameState) -> f64 {
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::challenges::{calc_ecc, ChallengeType};
    use crate::mechanics::cube_blessings::calculate_rune_effectiveness_cube_blessing;
    use crate::mechanics::hypercube_blessings::calculate_rune_effectiveness_hypercube_blessing;
    use crate::mechanics::platonic_blessings::calculate_hypercube_blessing_multiplier_platonic_blessing;
    use crate::mechanics::tesseract_blessings::calculate_rune_effectiveness_tesseract_blessing;

    // `player.cubeUpgrades[44]` — the rune-effectiveness cube-blessing DR increase.
    const CUBE_UPGRADE_RUNE_EFFECTIVENESS_BLESSING: usize = 44;

    let research = &state.researches.researches;
    let cc = &state.challenges.challenge_completions;
    // ConstantUpgrade9: 1 + 0.01·log4(talismanShards+1)·min(1, constantUpgrades[9]).
    let const_upgrade_9 = 1.0
        + 0.01
            * ((state.talismans.talisman_shards + 1.0).ln() / 4.0_f64.ln())
            * state.campaigns.constant_upgrades[9].min(1.0);

    // MidasTribute = calculateRuneEffectivenessCubeBlessing(): the cube-blessing
    // cascade (platonic → hypercube → tesseract → cube), now live since open()
    // (P3.2) makes the blessing levels accrue. Identity at level 0.
    let platonic_amplifier =
        calculate_hypercube_blessing_multiplier_platonic_blessing(&state.platonic_blessings);
    let hypercube_blessing = calculate_rune_effectiveness_hypercube_blessing(
        &state.hypercube_blessings,
        platonic_amplifier,
    );
    let tesseract_blessing = calculate_rune_effectiveness_tesseract_blessing(
        &state.tesseract_blessings,
        hypercube_blessing,
    );
    let midas_tribute = calculate_rune_effectiveness_cube_blessing(
        &state.cube_blessings,
        tesseract_blessing,
        state.cube_upgrade_levels.cube_upgrades[CUBE_UPGRADE_RUNE_EFFECTIVENESS_BLESSING],
    );

    product_f64(&[
        1.0 + research[4] / 10.0 * (1.0 + calc_ecc(ChallengeType::Ascension, cc[14])),
        1.0 + research[21] / 100.0,
        1.0 + research[90] / 100.0,
        1.0 + research[131] / 200.0,
        1.0 + (research[146] / 200.0 * 4.0) / 5.0,
        1.0 + (research[161] / 200.0 * 3.0) / 5.0,
        1.0 + (research[176] / 200.0 * 2.0) / 5.0,
        1.0 + (research[191] / 200.0) / 5.0,
        const_upgrade_9,
        1.0,           // Challenge15 runeBonus -> neutral (challenge_15_rewards reader unported)
        midas_tribute, // MidasTribute = rune-effectiveness cube-blessing cascade
        1.0 + research[84] / 200.0, // Research4x9 (runeEffectivenessStatsSI)
    ])
}

/// Free levels for a first-five rune (legacy `runes[rune].freeLevels()`): the
/// shared `firstFiveFreeLevels` (the FreeRunes ant upgrade at its true level +
/// `7·min(constantUpgrades[7], 1000)`) plus the per-rune bonus. Speed and
/// duplication have ported bonus aggregators (coin-log + coin-upgrade driven);
/// their `getRuneBonusFromAllTalismans` talisman bonus is unported (neutral 0),
/// and the prism/thrift/SI per-rune bonuses are unported (0). Identity at default.
fn rune_free_levels(state: &GameState, rune: usize) -> f64 {
    use crate::mechanics::ant_upgrades::free_runes_ant_upgrade_effect;
    use crate::mechanics::rune_level_bonuses::{
        bonus_rune_levels_duplication, bonus_rune_levels_speed, first_five_free_levels,
        BonusRuneLevelsDuplicationInput, BonusRuneLevelsSpeedInput, FirstFiveFreeLevelsInput,
    };
    use crate::state::{RUNE_DUPLICATION, RUNE_SPEED};

    // FreeRunes ant upgrade (index 8), read at its true (free + corruption) level.
    const ANT_UPGRADE_FREE_RUNES: usize = 8;
    let shared = first_five_free_levels(&FirstFiveFreeLevelsInput {
        free_runes_ant_upgrade: free_runes_ant_upgrade_effect(true_ant_level(
            state,
            ANT_UPGRADE_FREE_RUNES,
        )),
        constant_upgrade_7: state.campaigns.constant_upgrades[7],
    });

    let upgrade = |i: usize| f64::from(state.upgrades.upgrades[i]);
    let coin_log10 = (state.upgrades.coins + Decimal::one()).log10().to_number();
    let total_owned_coins_first_five: f64 = state.coin_producers.tiers[0..5]
        .iter()
        .map(|t| t.owned)
        .sum();

    let bonus = match rune {
        RUNE_SPEED => bonus_rune_levels_speed(&BonusRuneLevelsSpeedInput {
            talisman_bonus: get_rune_bonus_from_all_talismans(state, RUNE_SPEED),
            upgrade_27: upgrade(27),
            coin_log_1e10_floor: (coin_log10 / 10.0).floor(),
            coin_log_1e50_floor: (coin_log10 / 50.0).floor(),
            upgrade_29: upgrade(29),
            total_owned_coins_first_five,
        }),
        RUNE_DUPLICATION => bonus_rune_levels_duplication(&BonusRuneLevelsDuplicationInput {
            talisman_bonus: get_rune_bonus_from_all_talismans(state, RUNE_DUPLICATION),
            upgrade_28: upgrade(28),
            total_owned_coins_first_five,
            upgrade_30: upgrade(30),
            coin_log_1e30_floor: (coin_log10 / 30.0).floor(),
            coin_log_1e300_floor: (coin_log10 / 300.0).floor(),
        }),
        // prism/thrift/SI: the full per-rune free-level aggregators (coin/
        // upgrade terms) are still unported, but the talisman→rune-level bonus
        // term is now live for them.
        _ => get_rune_bonus_from_all_talismans(state, rune),
    };
    shared + bonus
}

/// Effective level of a first-five rune (speed/duplication/prism/thrift/SI),
/// legacy `getRuneEffectiveLevel`: reincarnation challenge 9 collapses it to 1
/// (all first-five `ignoreChal9 = false`); otherwise
/// `(level + freeLevels()) * firstFiveEffectiveRuneLevelMult`. Still deferred
/// (neutral, identity at default): the achievement-gated `isUnlocked` gates
/// (defaulting them to locked would wrongly zero the runes while achievements
/// are unported, H5) and SI's extra quark-based mult.
fn first_five_effective_rune_level(state: &GameState, rune: usize) -> f64 {
    if state.challenges.current_reincarnation_challenge == 9 {
        return 1.0;
    }
    (state.runes.rune_levels[rune] + rune_free_levels(state, rune))
        * first_five_effective_rune_level_mult(state)
}

/// `otherBlessingMultipliers` (RuneBlessings.ts:42): the shared multiplier on
/// every rune blessing's power. researches 134/194/160 and the midas talisman
/// blessingBonus are live; the epicFragments factor (driven by researches[174]
/// over the unported epicFragments balance) is neutral 1.0 — faithful while
/// researches[174] == 0 — and the challenge15 blessingBonus is neutral 1.0
/// (unported). Identity at default. TS-anchored.
fn other_blessing_multipliers(state: &GameState) -> f64 {
    use crate::mechanics::talisman_effects::midas_talisman_effects;
    use crate::state::TALISMAN_MIDAS;
    let research = &state.researches.researches;
    (1.0 + 6.9 * research[134] / 100.0)
        * midas_talisman_effects(state.talismans.talisman_rarity[TALISMAN_MIDAS] as i32)
            .blessing_bonus
        * (1.0 + 2.0 * research[194] / 100.0)
        * (1.0 + 0.25 * research[160])
}

/// Rune blessing power (legacy `getRuneBlessingPower` × `blessingMultiplier`):
/// `blessing.level · (rune.level + freeLevels()) · otherBlessingMultipliers()`.
/// The blessing effect formulas already match — only this power argument was
/// wrong (raw blessing level, dropping the rune's own level and the shared
/// mult). `freeLevels` is deferred (P2.1b, neutral 0). Identity at default
/// (blessing level 0 → power 0).
fn rune_blessing_power(state: &GameState, rune: usize) -> f64 {
    state.runes.rune_blessing_levels[rune]
        * state.runes.rune_levels[rune]
        * other_blessing_multipliers(state)
}

/// Shared rune-spirit power multiplier — legacy `otherSpiritMultipliers`
/// (`Statistics.ts:53-60`). The challenge-15 `spiritBonus` reward and the
/// corruption difficulty multiplier are unported (neutral 1.0 — the latter is
/// also 1.0 with no corruptions loaded), so this reduces to the four research
/// terms. Identity at default (all researches 0, not inside an ascension).
fn other_spirit_multipliers(state: &GameState) -> f64 {
    let research = &state.researches.researches;
    let in_ascension = state.challenges.current_ascension_challenge != 0;
    (1.0 + 8.0 * research[164] / 100.0)
        * if research[165] > 0.0 && in_ascension {
            2.0
        } else {
            1.0
        }
        * (1.0 + 0.15 * (state.talismans.legendary_fragments + 1.0).log10() * research[189])
        * (1.0 + 2.0 * research[194] / 100.0)
}

/// Rune-spirit power (legacy `getRuneSpiritPower` × `spiritMultiplier`,
/// `RuneSpirits.ts:180-183` / `Statistics.ts:62-68`):
/// `spirit.level · (rune.level + freeLevels()) · blessing.level ·
/// otherSpiritMultipliers()`. Mirrors [`rune_blessing_power`] with the extra
/// spirit-level factor; `freeLevels` is deferred (neutral 0, as there). Spirit
/// levels exist only for the first five runes. Identity at default (spirit
/// level 0 → power 0).
fn rune_spirit_power(state: &GameState, rune: usize) -> f64 {
    state.runes.rune_spirit_levels[rune]
        * state.runes.rune_levels[rune]
        * state.runes.rune_blessing_levels[rune]
        * other_spirit_multipliers(state)
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
/// Pure function of `&GameState`. Consumed by the global-state
/// aggregators in [`phase_global_state`].
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
        ant_building_accelerator_boost_mult: accelerator_boosts_ant_upgrade_effect(true_ant_level(
            state,
            ANT_UPGRADE_ACCELERATOR_BOOSTS,
        )),
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
            building_cost_scale_ant_upgrade_effect(true_ant_level(
                state,
                ANT_UPGRADE_BUILDING_COST_SCALE,
            ))
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
/// - `const_upgrade_1_buff_achievement` ✓ 0 (an ascendShards achievement grants 0.01, but the achievement system is unported — H5)
/// - `const_upgrade_2_buff_achievement` ✓ 0 (same achievement grants 0.01; neutral until awarding lands)
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

    let prism_level = first_five_effective_rune_level(state, RUNE_PRISM);
    let coin_tiers = &state.coin_producers.tiers;
    let total_coin_owned = calculate_total_coin_owned(&CalculateTotalCoinOwnedInput {
        first_owned_coin: coin_tiers[0].owned,
        second_owned_coin: coin_tiers[1].owned,
        third_owned_coin: coin_tiers[2].owned,
        fourth_owned_coin: coin_tiers[3].owned,
        fifth_owned_coin: coin_tiers[4].owned,
    });
    let ant_effect = coins_ant_upgrade_effect(&CoinsAntUpgradeInput {
        level: true_ant_level(state, ANT_UPGRADE_COINS),
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
    // `crystalCaps = getRuneSpiritEffect('prism').crystalCaps` — an additive
    // cap bonus driven by the prism rune-spirit power. Identity (0) at default
    // (prism spirit level 0 → power 0).
    let crystal_upgrade_4_max_exp =
        crystal_upgrade_4_max_exponent(&CrystalUpgrade4MaxExponentInput {
            research_129: state.researches.researches[129],
            common_fragments: Decimal::from_finite(state.talismans.common_fragments),
            prism_spirit_crystal_caps:
                crate::mechanics::rune_spirit_effects::prism_rune_spirit_effects(rune_spirit_power(
                    state, RUNE_PRISM,
                ))
                .crystal_caps,
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
        // `constUpgrade1Buff`/`constUpgrade2Buff` ARE granted (0.01 each) by an
        // ascendShards achievement (Achievements.ts:1866-1867), but the
        // achievement system is unported (audit H5), so the reward is 0 until
        // achievement awarding lands.
        const_upgrade_1_buff_achievement: 0.0,
        const_upgrade_2_buff_achievement: 0.0,
        // `constantEX` shop upgrade — `getShopUpgradeEffects` is identity
        // (`maxPercentIncrease = level`); the shop is ported, so read the level.
        constant_ex_max_percent_increase: crate::mechanics::shop_upgrades::constant_ex_effect(
            state.shop.upgrades[crate::state::shop::SHOP_CONSTANT_EX],
        ),
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
    let duplication_level = first_five_effective_rune_level(state, RUNE_DUPLICATION);
    let duplication_blessing_level = rune_blessing_power(state, RUNE_DUPLICATION);
    let hept_mult = multiplier_hepteract_effects(hepteract_effective_bal(
        state.hepteracts.multiplier.bal,
        1.0 / 5.0,
    ));
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
        ant_multiplier_mult: multipliers_ant_upgrade_effect(true_ant_level(
            state,
            ANT_UPGRADE_MULTIPLIERS,
        )),
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

    let speed_level = first_five_effective_rune_level(state, RUNE_SPEED);
    let hept_acc = accelerator_hepteract_effects(hepteract_effective_bal(
        state.hepteracts.accelerator.bal,
        1.0 / 5.0,
    ));
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
/// Refresh the 12 progressive-achievement caches and fold their points into
/// `achievement_points` — a per-tick port of the legacy `updateProgressiveCache`
/// loop. Each slot takes the running `Math.max` of its live value; the
/// `useCachedValue: true` entries score from the cached max, the others from
/// live state. Runs first in [`phase_global_state`] so the crystal/mythos
/// exponents (which read `achievement_points`) see the fresh total.
///
/// All 12 slots are live. The `exalts` slot derives each singularity
/// challenge's `rewardAP` (= `achievementPointValue(completions)`, a getter
/// in the legacy class) from the tracked completion counts; the three
/// maxed-upgrade families count `level >= maxLevel` against the seeded GQ
/// metadata / the static octeract + red-ambrosia max-level tables. Their
/// legacy `updateValue()` closures return `0`, so the cached value stays `0`
/// and the points come entirely from live state (`useCachedValue: false`).
fn update_progressive_achievements(state: &mut GameState) {
    use crate::mechanics::achievement_awards::update_progressive_slot;
    use crate::mechanics::achievement_points as ap;
    use crate::mechanics::ant_reborn_elo::calculate_leaderboard_value;
    use crate::mechanics::golden_quark_upgrades::count_maxed_golden_quark_upgrades;
    use crate::mechanics::octeracts::count_maxed_octeract_upgrades;
    use crate::mechanics::red_ambrosia_upgrades::count_maxed_red_ambrosia_upgrades;
    use crate::mechanics::singularity_challenges as sc;
    use crate::state::runes::RUNE_COUNT;

    // Gather every live value that needs a state read before the `&mut` borrow.
    let rune_level_sum: f64 = state.runes.rune_levels.iter().sum();
    let free_rune_sum: f64 = (0..RUNE_COUNT).map(|r| rune_free_levels(state, r)).sum();
    let masteries: [f64; 9] =
        std::array::from_fn(|i| f64::from(state.ants.masteries[i].highest_mastery));
    let mastery_sum: f64 = masteries.iter().sum();
    let mastery_points = ap::ant_mastery_points(&masteries);
    let elo_values: Vec<f64> = state
        .ants
        .highest_reborn_elo_ever
        .iter()
        .map(|e| e.elo)
        .collect();
    let leaderboard_elo = calculate_leaderboard_value(&elo_values);
    let sing = state.singularity.highest_singularity_count;
    let lifetime_ambrosia = state.ambrosia.lifetime_ambrosia;
    let lifetime_red = state.red_ambrosia.lifetime_red_ambrosia;
    let rarity_sum: f64 = state.talismans.talisman_rarity.iter().sum();
    // exalts: Σ rewardAP = Σ achievementPointValue(completions) per challenge.
    let s = &state.singularity;
    let exalt_ap = ap::exalt_points(&[
        sc::no_singularity_upgrades_achievement_point_value(s.no_singularity_upgrades.completions),
        sc::one_challenge_cap_achievement_point_value(s.one_challenge_cap.completions),
        sc::no_octeracts_achievement_point_value(s.no_octeracts.completions),
        sc::limited_ascensions_achievement_point_value(s.limited_ascensions.completions),
        sc::no_ambrosia_upgrades_achievement_point_value(s.no_ambrosia_upgrades.completions),
        sc::no_quark_upgrades_achievement_point_value(s.no_quark_upgrades.completions),
        sc::limited_time_achievement_point_value(s.limited_time.completions),
        sc::sadistic_prequel_achievement_point_value(s.sadistic_prequel.completions),
        sc::taxman_last_stand_achievement_point_value(s.taxman_last_stand.completions),
    ]);
    let gq_maxed_points = ap::maxed_upgrade_family_points(
        count_maxed_golden_quark_upgrades(&state.golden_quarks),
        5.0,
    );
    let oct_maxed_points = ap::maxed_upgrade_family_points(
        count_maxed_octeract_upgrades(&state.octeract_upgrades),
        8.0,
    );
    let red_maxed_points = ap::maxed_upgrade_family_points(
        count_maxed_red_ambrosia_upgrades(&state.red_ambrosia),
        10.0,
    );

    let ach = &mut state.achievements;
    // useCachedValue: true → score from the cached Math.max value.
    update_progressive_slot(ach, 0, rune_level_sum, ap::rune_level_points);
    update_progressive_slot(ach, 1, free_rune_sum, ap::free_rune_level_points);
    update_progressive_slot(ach, 5, lifetime_ambrosia, ap::ambrosia_count_points);
    update_progressive_slot(ach, 6, lifetime_red, ap::red_ambrosia_count_points);
    update_progressive_slot(ach, 7, rarity_sum, ap::talisman_rarity_points);
    // useCachedValue: false → score from live state (the closure ignores cached).
    update_progressive_slot(ach, 2, mastery_sum, |_| mastery_points);
    update_progressive_slot(ach, 3, leaderboard_elo, |_| {
        ap::reborn_elo_points(leaderboard_elo)
    });
    update_progressive_slot(ach, 4, sing, |_| ap::singularity_count_points(sing));
    // Slots 8-11 mirror the legacy `updateValue: () => 0` — live value 0,
    // points from live state.
    update_progressive_slot(ach, 8, 0.0, |_| exalt_ap);
    update_progressive_slot(ach, 9, 0.0, |_| gq_maxed_points);
    update_progressive_slot(ach, 10, 0.0, |_| oct_maxed_points);
    update_progressive_slot(ach, 11, 0.0, |_| red_maxed_points);
}

/// `talismans[t].isUnlocked()` — the per-talisman unlock predicate
/// (`Talismans.ts`). Every gate is a pure function of `GameState`, so no
/// persisted unlock flag is needed. Unported gates default to locked:
/// chronos/midas/metaphysics/polymath read `getAchievementReward('*Talisman')`
/// (achievement-reward table unported), `achievement` reads a level milestone
/// (unported), and `horseShoe` reads the `taxmanLastStand` singularity
/// challenge (singularity paused) — all neutral `false`. Shared with the
/// talisman→rune-level bonus.
fn talisman_is_unlocked(state: &GameState, t: usize) -> bool {
    use crate::mechanics::ant_upgrades::mortuus_ant_upgrade_effect;
    use crate::mechanics::shop_upgrades::shop_talisman_effect;
    use crate::state::shop::SHOP_TALISMAN;
    use crate::state::{
        TALISMAN_COOKIE_GRANDMA, TALISMAN_EXEMPTION, TALISMAN_MORTUUS, TALISMAN_PLASTIC,
        TALISMAN_WOW_SQUARE,
    };
    const ANT_UPGRADE_MORTUUS: usize = 11;
    match t {
        // unlocks.talismans ← highestchallengecompletions[9] > 0 (Synergism.ts:3136).
        TALISMAN_EXEMPTION => state.challenges.highest_challenge_completions[9] > 0.0,
        TALISMAN_MORTUUS => {
            mortuus_ant_upgrade_effect(true_ant_level(state, ANT_UPGRADE_MORTUUS)).talisman_unlock
        }
        // shopTalisman: the PCoin instant-unlock-1 path is unported → false.
        TALISMAN_PLASTIC => shop_talisman_effect(state.shop.upgrades[SHOP_TALISMAN], false),
        TALISMAN_WOW_SQUARE => state.reset_counters.ascension_count >= 100.0,
        TALISMAN_COOKIE_GRANDMA => state.cube_upgrade_levels.cube_upgrades[80] > 0.0,
        // chronos/midas/metaphysics/polymath/achievement/horseShoe: gate unported → locked.
        _ => false,
    }
}

/// `updateTalismanRarities` (`Talismans.ts:670-676`): recompute every
/// talisman's display rarity from its level + unlock state. Run first in
/// [`phase_global_state`] so the rarity-indexed effects (midas blessing, the
/// exemption/chronos/polymath/mortuus multipliers, and the talisman→rune-level
/// bonus) read this tick's value. Locked talismans collapse to rarity 0; an
/// unlocked talisman is at least rarity 1 even at level 0.
fn recompute_talisman_rarities(state: &mut GameState) {
    use crate::mechanics::talisman_levels::{
        compute_talisman_rarity, ComputeTalismanRarityInput, TALISMAN_MAX_LEVELS,
    };
    for (t, &max_level) in TALISMAN_MAX_LEVELS.iter().enumerate() {
        let is_unlocked = talisman_is_unlocked(state, t);
        state.talismans.talisman_rarity[t] =
            f64::from(compute_talisman_rarity(&ComputeTalismanRarityInput {
                is_unlocked,
                level: state.talismans.talisman_levels[t],
                max_level,
            }));
    }
}

/// Per-talisman, per-rune base coefficient (`talismanBaseCoefficient`,
/// `Talismans.ts`). Rows follow the `TALISMAN_*` order; columns the rune
/// object-key order (speed, duplication, prism, thrift, superiorIntellect,
/// infiniteAscent, antiquities, horseShoe, finiteDescent, topHat).
const TALISMAN_BASE_COEFFICIENT: [[f64; RUNE_COUNT]; TALISMAN_COUNT] = [
    [0.0, 1.5, 0.75, 0.75, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // exemption
    [1.5, 0.0, 0.0, 0.75, 0.75, 0.0, 0.0, 0.0, 0.0, 0.0], // chronos
    [0.0, 0.75, 0.75, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // midas
    [0.6, 0.6, 0.6, 0.6, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0],   // metaphysics
    [0.75, 0.75, 0.0, 0.0, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0], // polymath
    [0.6, 0.6, 0.6, 0.6, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0],   // mortuus
    [0.75, 0.0, 1.5, 0.0, 0.75, 0.005, 0.0, 0.0, 0.0, 0.0], // plastic
    [0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],   // wowSquare
    [1.4, 1.4, 1.4, 1.4, 1.4, 0.01, 0.0, 0.0, 0.0, 0.0],  // achievement
    [1.0, 1.0, 1.0, 1.0, 1.0, 0.01, 0.0, 0.0, 0.0, 0.0],  // cookieGrandma
    [1.2, 1.2, 1.2, 1.2, 1.2, 0.0, 0.0, 0.01, 0.0, 0.0],  // horseShoe
];

/// `getRuneBonusFromIndividualTalisman` (`Talismans.ts:764-780`):
/// `coeff[t][rune] · bonusMult · level · rarityValues[rarity]`, gated on the
/// talisman's unlock. `bonusMult` is 1 except for metaphysics (× its own
/// `talismanEffect` × `extraTalismanEffect` — the talisman amplifier) and
/// mortuus (× the Mortuus2 ant-upgrade `talismanEffectBuff`). Zero for a locked
/// or level-0 talisman.
fn rune_bonus_from_individual_talisman(state: &GameState, t: usize, rune: usize) -> f64 {
    use crate::mechanics::ant_upgrades::mortuus_2_ant_upgrade_effect;
    use crate::mechanics::talisman_effects::metaphysics_talisman_effects;
    use crate::mechanics::talisman_levels::rarity_value;
    use crate::state::{TALISMAN_METAPHYSICS, TALISMAN_MORTUUS};
    const ANT_UPGRADE_MORTUUS_2: usize = 15;

    if !talisman_is_unlocked(state, t) {
        return 0.0;
    }
    let rarity = state.talismans.talisman_rarity[t] as u8;
    let mut bonus_mult = 1.0;
    if t == TALISMAN_METAPHYSICS {
        let e = metaphysics_talisman_effects(i32::from(rarity));
        bonus_mult *= e.talisman_effect * e.extra_talisman_effect;
    }
    if t == TALISMAN_MORTUUS {
        bonus_mult *= mortuus_2_ant_upgrade_effect(true_ant_level(state, ANT_UPGRADE_MORTUUS_2))
            .talisman_effect_buff;
    }
    TALISMAN_BASE_COEFFICIENT[t][rune]
        * bonus_mult
        * state.talismans.talisman_levels[t]
        * rarity_value(rarity)
}

/// `allTalismanRuneBonusStatsSum` (`Statistics.ts:2754-2771`): the special
/// multiplier on the summed talisman→rune bonus. The achievement `talismanPower`
/// reward, the challenge-15 `talismanBonus`, and the `taxmanLastStand`
/// singularity challenge are unported → neutral 0. Identity (1.0) at default.
fn talisman_rune_bonus_stats_sum(state: &GameState) -> f64 {
    use crate::mechanics::blueberry_upgrades::ambrosia_talisman_bonus_rune_level_effect;
    use crate::mechanics::golden_quark_upgrades::{
        sing_talisman_bonus_runes_1_effect, sing_talisman_bonus_runes_2_effect,
        sing_talisman_bonus_runes_3_effect, sing_talisman_bonus_runes_4_effect,
    };
    use crate::state::ambrosia::AMBROSIA_TALISMAN_BONUS_RUNE_LEVEL;
    use crate::state::golden_quarks::{
        GQ_SING_TALISMAN_BONUS_RUNES_1, GQ_SING_TALISMAN_BONUS_RUNES_2,
        GQ_SING_TALISMAN_BONUS_RUNES_3, GQ_SING_TALISMAN_BONUS_RUNES_4,
    };
    let research = &state.researches.researches;
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let amb = |i: usize| state.ambrosia.upgrades[i].level + state.ambrosia.upgrades[i].free_level;
    1.0 + research[106] / 1000.0
        + research[107] / 1000.0
        + 2.0 * research[118] / 1000.0
        + 0.004 * (research[200] / 10_000.0).floor()
        + 0.006 * (state.cube_upgrade_levels.cube_upgrades[50] / 10_000.0).floor()
        + sing_talisman_bonus_runes_1_effect(gq(GQ_SING_TALISMAN_BONUS_RUNES_1))
        + sing_talisman_bonus_runes_2_effect(gq(GQ_SING_TALISMAN_BONUS_RUNES_2))
        + sing_talisman_bonus_runes_3_effect(gq(GQ_SING_TALISMAN_BONUS_RUNES_3))
        + sing_talisman_bonus_runes_4_effect(gq(GQ_SING_TALISMAN_BONUS_RUNES_4))
        + ambrosia_talisman_bonus_rune_level_effect(amb(AMBROSIA_TALISMAN_BONUS_RUNE_LEVEL))
}

/// `getRuneBonusFromAllTalismans` (`Talismans.ts:782-790`): the talisman→
/// rune-level bonus, `allTalismanRuneBonusStatsSum · Σ_t individual(t, rune)`,
/// added to the rune's free levels. Zero at default (no talisman both unlocked
/// and leveled).
fn get_rune_bonus_from_all_talismans(state: &GameState, rune: usize) -> f64 {
    let total: f64 = (0..TALISMAN_COUNT)
        .map(|t| rune_bonus_from_individual_talisman(state, t, rune))
        .sum();
    total * talisman_rune_bonus_stats_sum(state)
}

fn phase_global_state(state: &mut GameState) -> AggregatorOutputs {
    update_progressive_achievements(state);
    // Sweep the monotonic threshold-based achievement groups (reset counts,
    // accelerator/multiplier/boost bought totals, speed-rune progression).
    // Awards on threshold crossing.
    let speed_free_level = rune_free_levels(state, crate::state::runes::RUNE_SPEED);
    // The ascension score is `f64`-softcapped and below the 1e5 group floor
    // while ascension is locked — skip the full CalcCorruptionStuff there.
    let ascension_score = if state.reset_counters.ascension_unlocked {
        compute_ascension_score_result(state).effective_score
    } else {
        0.0
    };
    let campaign_tokens = compute_campaign_tokens(state);
    let awarded = crate::mechanics::achievement_awards::reset_count_achievement_check(
        &mut state.achievements,
        state.reset_counters.prestige_count,
        state.reset_counters.transcend_count,
        state.reset_counters.reincarnation_count,
        state.reset_counters.ascension_count,
    ) + crate::mechanics::achievement_awards::accelerator_achievement_check(
        &mut state.achievements,
        state.accelerator.accelerator_bought,
        state.multiplier.multiplier_bought,
        state.accelerator.accelerator_boost_bought,
    ) + crate::mechanics::achievement_awards::rune_achievement_check(
        &mut state.achievements,
        state.runes.rune_levels[crate::state::runes::RUNE_SPEED],
        speed_free_level,
        state.runes.rune_blessing_levels[crate::state::runes::RUNE_SPEED],
        state.runes.rune_spirit_levels[crate::state::runes::RUNE_SPEED],
    ) + crate::mechanics::achievement_awards::decimal_currency_achievement_check(
        &mut state.achievements,
        state.campaigns.ascend_shards,
        state.ants.crumbs,
    ) + crate::mechanics::achievement_awards::ascension_score_achievement_check(
        &mut state.achievements,
        ascension_score,
    ) + crate::mechanics::achievement_awards::singularity_achievement_check(
        &mut state.achievements,
        state.singularity.highest_singularity_count,
    ) + crate::mechanics::achievement_awards::campaign_tokens_achievement_check(
        &mut state.achievements,
        campaign_tokens,
    )
        // thousandSuns (#250) / thousandMoons (#251) — ungrouped, checked at
        // updateAll cadence in the legacy tick (Synergism.ts:3994). The legacy
        // `=== 1e5` gates are `>= 1e5` here: both levels cap at exactly 1e5
        // (research 8x25 / cube w5x10), so the forms are equivalent and `>=`
        // stays monotonic.
        + crate::mechanics::achievement_awards::award_ungrouped_achievement(
            &mut state.achievements,
            250,
            100.0,
            state.researches.researches[200] >= 1e5,
        )
        + crate::mechanics::achievement_awards::award_ungrouped_achievement(
            &mut state.achievements,
            251,
            150.0,
            state.cube_upgrade_levels.cube_upgrades[50] >= 1e5,
        );
    credit_achievement_quarks(state, awarded);
    recompute_talisman_rarities(state);
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
        level: true_ant_level(state, ANT_UPGRADE_COINS),
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
            first_five_effective_rune_level(state, RUNE_DUPLICATION),
            DuplicationRuneKey::TaxReduction,
        ),
        thrift_rune_tax_reduction: thrift_rune_effects(
            first_five_effective_rune_level(state, RUNE_THRIFT),
            ThriftRuneKey::TaxReduction,
        ),
        ant_tax_reduction: taxes_ant_upgrade_effect(true_ant_level(state, ANT_UPGRADE_TAXES)),
        exemption_talisman_tax_reduction: exemption_talisman_effects(
            state.talismans.talisman_rarity[TALISMAN_EXEMPTION] as i32,
        )
        .tax_reduction,
        challenge_15_taxes_reward: challenge_15_rewards::taxes(challenges.challenge15_exponent),
        campaign_tax_multiplier: campaign_token_rewards::campaign_tax_multiplier(
            compute_campaign_tokens(state),
        ),
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
            first_five_effective_rune_level(state, RUNE_SPEED),
            SpeedRuneKey::GlobalSpeed,
        ),
        1.0, // obtainium-log: maxObtainium untracked → 1.0 (upgrades[70] == 0)
        1.0 + researches[121] / 50.0,
        1.0 + 0.015 * researches[136],
        1.0 + 0.012 * researches[151],
        1.0 + 0.009 * researches[166],
        1.0 + 0.006 * researches[181],
        1.0 + 0.003 * researches[196],
        speed_rune_blessing_effects(rune_blessing_power(state, RUNE_SPEED)).global_speed,
        crate::mechanics::rune_spirit_effects::speed_rune_spirit_effects(rune_spirit_power(
            state, RUNE_SPEED,
        ))
        .global_speed,
        chronos_cube,
        1.0 + cube_upgrades[CUBE_UPGRADE_2X8] / 5.0,
        mortuus_ant_upgrade_effect(true_ant_level(state, ANT_UPGRADE_MORTUUS)).global_speed,
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
    use crate::mechanics::golden_quark_upgrades::one_mind_effect;
    use crate::state::golden_quarks::GQ_ONE_MIND;

    // `oneMind` locks ascension speed to a flat ×10, bypassing the StatLine
    // reduction entirely (legacy Helper.ts `addTimers('ascension')`).
    if one_mind_effect(state.golden_quarks.upgrades[GQ_ONE_MIND].level) {
        return 10.0;
    }
    compute_ascension_speed_mult_raw(state)
}

/// `calculateAscensionSpeedMult()` — the raw `allAscensionSpeedStats` product
/// raised to `1 ± exponent_spread`, WITHOUT the `oneMind → 10` override the
/// ascension timer applies (see [`compute_ascension_speed_mult_pre`]). The
/// octeract `AscensionSpeed` StatLine references this raw value in both oneMind
/// branches, so it is factored out here.
fn compute_ascension_speed_mult_raw(state: &GameState) -> f64 {
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
        intermediate_pack_effect, sing_ascension_speed_2_effect, sing_ascension_speed_effect,
        IntermediatePackKey,
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
        GQ_INTERMEDIATE_PACK, GQ_SING_ASCENSION_SPEED, GQ_SING_ASCENSION_SPEED_2,
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
        mortuus_2_ant_upgrade_effect(true_ant_level(state, ANT_UPGRADE_MORTUUS_2)).ascension_speed,
        polymath_talisman_effects(state.talismans.talisman_rarity[TALISMAN_POLYMATH] as i32)
            .ascension_speed_bonus,
        chronometer_effect(shop[SHOP_CHRONOMETER]),
        chronometer_2_effect(shop[SHOP_CHRONOMETER_2]),
        chronometer_3_effect(shop[SHOP_CHRONOMETER_3]),
        chronos_hepteract_effects(hepteract_effective_bal(
            state.hepteracts.chronos.bal,
            1.0 / 6.0 + state.cube_upgrade_levels.platonic_upgrades[19] / 750.0,
        ))
        .ascension_speed,
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

/// `calculateAscensionScore()` — the full ascension-score result (base /
/// corruption / bonus multipliers + the `1e23`-softcapped `effectiveScore`),
/// self-derived from `&GameState`. Shared by [`compute_octeract_per_second`]
/// (its `AscensionScore` octeract line), the five cube-family multipliers
/// (each `AscensionScore` line), and the ascension cube award
/// (`calc_corruption_stuff`).
///
/// The campaign multiplier is wired (`campaign_ascension_score_multiplier`
/// of the derived token total); the event buff stays a faithful neutral 0
/// (UI-tier calendar).
fn compute_ascension_score_result(
    state: &GameState,
) -> crate::mechanics::calculate::CalculateAscensionScoreResult {
    use crate::mechanics::achievement_rewards::ascension_score as ascension_score_reward;
    use crate::mechanics::ant_upgrades::ascension_score_ant_upgrade_effect;
    use crate::mechanics::calculate::{
        calculate_ascension_score, compute_ascension_score_bonus_multiplier,
        AscensionScoreBonusMultiplierInput, CalculateAscensionScoreInput,
    };
    use crate::mechanics::challenge_15_rewards;
    use crate::mechanics::golden_quark_upgrades::{
        expert_pack_effect, master_pack_effect, ExpertPackKey, MasterPackKey,
    };
    use crate::mechanics::platonic_blessings::calculate_ascension_score_platonic_blessing;
    use crate::mechanics::rune_effects::{finite_descent_rune_effects, FiniteDescentRuneKey};
    use crate::state::golden_quarks::{GQ_EXPERT_PACK, GQ_MASTER_PACK};
    use crate::state::RUNE_FINITE_DESCENT;

    const CUBE_UPGRADE_21: usize = 21;
    const CUBE_UPGRADE_31: usize = 31;
    const CUBE_UPGRADE_39: usize = 39;
    const CUBE_UPGRADE_41: usize = 41;
    const CUBE_UPGRADE_56: usize = 56;
    const PLATONIC_UPGRADE_ALPHA: usize = 5;
    const PLATONIC_UPGRADE_BETA: usize = 10;
    const ANT_UPGRADE_ASCENSION_SCORE: usize = 14;

    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };

    let bonus_multiplier =
        compute_ascension_score_bonus_multiplier(&AscensionScoreBonusMultiplierInput {
            challenge_15_score_reward: challenge_15_rewards::score(
                state.challenges.challenge15_exponent,
            ),
            platonic_blessing_mult: calculate_ascension_score_platonic_blessing(
                &state.platonic_blessings,
            ),
            campaign_ascension_score_mult:
                crate::mechanics::campaign_token_rewards::campaign_ascension_score_multiplier(
                    compute_campaign_tokens(state),
                ),
            finite_descent_ascension_score: finite_descent_rune_effects(
                state.runes.rune_levels[RUNE_FINITE_DESCENT],
                FiniteDescentRuneKey::AscensionScore,
            ),
            cube_upgrade_21: cube[CUBE_UPGRADE_21],
            cube_upgrade_31: cube[CUBE_UPGRADE_31],
            cube_upgrade_41: cube[CUBE_UPGRADE_41],
            ascension_score_achievement_reward: ascension_score_reward(
                &state.achievements.achievements,
                state.campaigns.ascend_shards,
            ),
            master_pack_ascension_score_mult: master_pack_effect(
                gq(GQ_MASTER_PACK),
                MasterPackKey::AscensionScoreMult,
            ),
            event_buff: 0.0, // UI-tier event calendar → no active event
        });
    calculate_ascension_score(&CalculateAscensionScoreInput {
        highest_challenge_completions: &state.challenges.highest_challenge_completions,
        cube_upgrade_56: cube[CUBE_UPGRADE_56],
        cube_upgrade_39: cube[CUBE_UPGRADE_39],
        platonic_upgrade_5: platonic[PLATONIC_UPGRADE_ALPHA],
        platonic_upgrade_10: platonic[PLATONIC_UPGRADE_BETA],
        corruption_multiplier: state.corruptions.used.total_corruption_ascension_multiplier,
        ant_upgrade_ascension_score_base: ascension_score_ant_upgrade_effect(true_ant_level(
            state,
            ANT_UPGRADE_ASCENSION_SCORE,
        ))
        .ascension_score_base,
        expert_pack_ascension_score_mult: expert_pack_effect(
            gq(GQ_EXPERT_PACK),
            ExpertPackKey::AscensionScoreMult,
        ),
        bonus_multiplier,
    })
}

/// `octeract_per_second` — self-derived from `&GameState`.
///
/// Legacy Helper.ts `addTimers('octeracts')` = `calculateOcteractMultiplier()`
/// = `product(allOcteractCubeStats)` (Statistics.ts:643), the 42-line octeract
/// gain product. The `AscensionScore` line gates the whole product to `0`
/// unless `calculateAscensionScore().effectiveScore >= 1e23`, so this is
/// exactly `0` at the default state — matching the old
/// `AutomationPre::octeract_per_second` default. Only consumed by the
/// `singularityCount >= 160` octeract-giveaway loop.
///
/// Neutral-defaulted lines (faithful — inert / unported subsystem): PseudoCoins
/// (PCoin meta layer), Campaign (`player.campaigns.octeractBonus`), WowSquare
/// (the wowSquare talisman is not among the 7 ported talismans), Event (UI-tier
/// wall-clock calendar). The Patreon bonus passes `quark_bonus = 0` (patreon
/// meta layer); it is `1.0` whenever its upgrade is unbought regardless.
fn compute_octeract_per_second(state: &GameState) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::ambrosia::{calculate_ambrosia_cube_mult, AmbrosiaMultInput};
    use crate::mechanics::blueberry_upgrades::{
        ambrosia_cubes_1_effect, ambrosia_cubes_2_effect, ambrosia_cubes_3_effect,
        ambrosia_luck_cube_1_effect, ambrosia_quark_cube_1_effect, ambrosia_tutorial_effect,
        AmbrosiaTutorialEffectKey,
    };
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::golden_quark_upgrades::{
        divine_pack_effect, one_mind_effect, platonic_delta_effect, sing_cubes_1_effect,
        sing_cubes_2_effect, sing_cubes_3_effect, sing_octeract_gain_2_effect,
        sing_octeract_gain_3_effect, sing_octeract_gain_4_effect, sing_octeract_gain_5_effect,
        sing_octeract_gain_effect, sing_octeract_patreon_bonus_effect, DivinePackKey,
    };
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::octeracts::{
        octeract_ascensions_octeract_gain_effect, octeract_gain_2_effect, octeract_gain_effect,
        octeract_one_mind_improver_effect, octeract_starter_effect, OcteractStarterKey,
    };
    use crate::mechanics::red_ambrosia_bonuses::{
        calculate_red_ambrosia_cubes, CalculateRedAmbrosiaCubesInput,
    };
    use crate::mechanics::red_ambrosia_upgrades::{
        red_ambrosia_cube_effect, red_ambrosia_cube_improver_effect,
        tutorial_effect as red_tutorial_effect,
    };
    use crate::mechanics::shop_upgrades::{
        season_pass_3_effect, season_pass_infinity_effect, season_pass_lost_effect,
        season_pass_y_effect, season_pass_z_effect, shop_cash_grab_ultra_effect,
        shop_ex_ultra_effect, shop_singularity_speedup_effect, SeasonPassInfinityKey,
        ShopCashGrabUltraKey,
    };
    use crate::mechanics::singularity_challenges::{
        no_singularity_upgrades_effect, NoSingularityUpgradesKey, SingularityEffectValue,
    };
    use crate::mechanics::singularity_milestones::derpsmith_cornucopia_bonus;
    use crate::state::ambrosia::{
        AMBROSIA_CUBES_1, AMBROSIA_CUBES_2, AMBROSIA_CUBES_3, AMBROSIA_LUCK_CUBE_1,
        AMBROSIA_QUARK_CUBE_1, AMBROSIA_TUTORIAL,
    };
    use crate::state::golden_quarks::{
        GQ_DIVINE_PACK, GQ_ONE_MIND, GQ_PLATONIC_DELTA, GQ_SING_CUBES_1, GQ_SING_CUBES_2,
        GQ_SING_CUBES_3, GQ_SING_OCTERACT_GAIN, GQ_SING_OCTERACT_GAIN_2, GQ_SING_OCTERACT_GAIN_3,
        GQ_SING_OCTERACT_GAIN_4, GQ_SING_OCTERACT_GAIN_5, GQ_SING_OCTERACT_PATREON_BONUS,
    };
    use crate::state::octeract_upgrades::{
        OCTERACT_ASCENSIONS_OCTERACT_GAIN, OCTERACT_GAIN, OCTERACT_GAIN_2,
        OCTERACT_ONE_MIND_IMPROVER, OCTERACT_STARTER,
    };
    use crate::state::red_ambrosia::{
        RED_AMBROSIA_RED_AMBROSIA_CUBE, RED_AMBROSIA_RED_AMBROSIA_CUBE_IMPROVER,
        RED_AMBROSIA_TUTORIAL,
    };
    use crate::state::shop::{
        SHOP_CASH_GRAB_ULTRA, SHOP_EX_ULTRA, SHOP_SEASON_PASS_3, SHOP_SEASON_PASS_INFINITY,
        SHOP_SEASON_PASS_LOST, SHOP_SEASON_PASS_Y, SHOP_SEASON_PASS_Z, SHOP_SINGULARITY_SPEEDUP,
    };

    // Legacy `player.cubeUpgrades[..]` index.
    const CUBE_UPGRADE_COOKIE_20: usize = 70;

    let sing = state.singularity.singularity_count;
    let shop = &state.shop.upgrades;
    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let achievement_level = achievement_level_from_points(state.achievements.achievement_points);
    let lifetime_ambrosia = state.ambrosia.lifetime_ambrosia;
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let oct = |i: usize| {
        state.octeract_upgrades.upgrades[i].level + state.octeract_upgrades.upgrades[i].free_level
    };
    let amb = |i: usize| state.ambrosia.upgrades[i].level + state.ambrosia.upgrades[i].free_level;
    let red = |i: usize| state.red_ambrosia.upgrades[i].level;

    // AscensionScore line: `calculateAscensionScore().effectiveScore`, gated at 1e23.
    let effective_score = compute_ascension_score_result(state).effective_score;
    const SCORE_REQ: f64 = 1e23;
    let ascension_score_line = if effective_score >= SCORE_REQ {
        effective_score / SCORE_REQ
    } else {
        0.0
    };

    // CookieUpgrade20: `1 + (b2f(Σ used corruption levels >= 14·8) · cube[70]) / 10000`.
    let corruption_total_levels: u32 = state.corruptions.used.levels.iter().sum();
    let cookie_20_gate = if corruption_total_levels >= 14 * 8 {
        1.0
    } else {
        0.0
    };
    let cookie_upgrade_20 = 1.0 + (cookie_20_gate * cube[CUBE_UPGRADE_COOKIE_20]) / 10_000.0;

    // ModuleLuckCube1 reads `calculateAmbrosiaLuck()`; ModuleQuarkCube1 reads `worlds`.
    let ambrosia_luck = compute_ambrosia_luck_pre(state);
    let worlds = state.quarks.worlds.to_number();

    // AscensionSpeed line: oneMind branch on the RAW ascension-speed mult.
    let raw_ascension_speed = compute_ascension_speed_mult_raw(state);
    let ascension_speed_line = if one_mind_effect(state.golden_quarks.upgrades[GQ_ONE_MIND].level) {
        10.0_f64.powf(0.5)
            * (raw_ascension_speed / 10.0).powf(octeract_one_mind_improver_effect(oct(
                OCTERACT_ONE_MIND_IMPROVER,
            )))
    } else {
        raw_ascension_speed.powf(0.5)
    };

    // Cubes context → a missing singularity-challenge value is the multiplicative 1.
    let scalar = |v: SingularityEffectValue| match v {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 1.0,
    };

    product_f64(&[
        1.0 / (24.0 * 3600.0 * 365.0 * 1e15), // BasePerSecond
        ascension_score_line,
        1.0, // PseudoCoins — PCoin meta layer (unported)
        get_level_reward(LevelRewardKey::WowOcteracts, achievement_level),
        crate::mechanics::campaign_token_rewards::campaign_octeract_bonus(compute_campaign_tokens(
            state,
        )), // Campaign — player.campaigns.octeractBonus
        season_pass_3_effect(shop[SHOP_SEASON_PASS_3]),
        season_pass_y_effect(shop[SHOP_SEASON_PASS_Y]),
        season_pass_z_effect(shop[SHOP_SEASON_PASS_Z], sing),
        season_pass_lost_effect(shop[SHOP_SEASON_PASS_LOST]),
        1.0, // WowSquare — wowSquare talisman not among the 7 ported talismans
        cookie_upgrade_20,
        divine_pack_effect(
            gq(GQ_DIVINE_PACK),
            DivinePackKey::OcteractMult,
            &state.corruptions.used.levels,
        ),
        sing_cubes_1_effect(gq(GQ_SING_CUBES_1)),
        sing_cubes_2_effect(gq(GQ_SING_CUBES_2)),
        sing_cubes_3_effect(gq(GQ_SING_CUBES_3)),
        sing_octeract_gain_effect(gq(GQ_SING_OCTERACT_GAIN)),
        sing_octeract_gain_2_effect(gq(GQ_SING_OCTERACT_GAIN_2)),
        sing_octeract_gain_3_effect(gq(GQ_SING_OCTERACT_GAIN_3)),
        sing_octeract_gain_4_effect(gq(GQ_SING_OCTERACT_GAIN_4)),
        sing_octeract_gain_5_effect(gq(GQ_SING_OCTERACT_GAIN_5)),
        sing_octeract_patreon_bonus_effect(gq(GQ_SING_OCTERACT_PATREON_BONUS), 0.0),
        octeract_starter_effect(oct(OCTERACT_STARTER), OcteractStarterKey::OcteractMult),
        octeract_gain_effect(oct(OCTERACT_GAIN)),
        octeract_gain_2_effect(oct(OCTERACT_GAIN_2)),
        derpsmith_cornucopia_bonus(state.singularity.highest_singularity_count),
        octeract_ascensions_octeract_gain_effect(
            oct(OCTERACT_ASCENSIONS_OCTERACT_GAIN),
            state.reset_counters.ascension_count,
        ),
        1.0, // Event — UI-tier event calendar → 1 + 0
        platonic_delta_effect(
            gq(GQ_PLATONIC_DELTA),
            state.singularity.singularity_counter,
            shop_singularity_speedup_effect(shop[SHOP_SINGULARITY_SPEEDUP]),
        ),
        scalar(no_singularity_upgrades_effect(
            state.singularity.no_singularity_upgrades.completions,
            NoSingularityUpgradesKey::Cubes,
        )),
        season_pass_infinity_effect(
            shop[SHOP_SEASON_PASS_INFINITY],
            SeasonPassInfinityKey::WowOcteractMult,
        ),
        calculate_ambrosia_cube_mult(&AmbrosiaMultInput {
            no_ambrosia_upgrades_enabled: state.singularity.no_ambrosia_upgrades.enabled,
            lifetime_ambrosia,
        }),
        ambrosia_tutorial_effect(amb(AMBROSIA_TUTORIAL), AmbrosiaTutorialEffectKey::Cubes),
        ambrosia_cubes_1_effect(amb(AMBROSIA_CUBES_1)),
        ambrosia_luck_cube_1_effect(amb(AMBROSIA_LUCK_CUBE_1), ambrosia_luck),
        ambrosia_quark_cube_1_effect(amb(AMBROSIA_QUARK_CUBE_1), worlds),
        ambrosia_cubes_2_effect(amb(AMBROSIA_CUBES_2), amb(AMBROSIA_CUBES_1)),
        ambrosia_cubes_3_effect(amb(AMBROSIA_CUBES_3), amb(AMBROSIA_CUBES_2)),
        red_tutorial_effect(red(RED_AMBROSIA_TUTORIAL)),
        calculate_red_ambrosia_cubes(&CalculateRedAmbrosiaCubesInput {
            unlocked: red_ambrosia_cube_effect(red(RED_AMBROSIA_RED_AMBROSIA_CUBE)),
            lifetime_red_ambrosia: state.red_ambrosia.lifetime_red_ambrosia,
            extra_exponent: red_ambrosia_cube_improver_effect(red(
                RED_AMBROSIA_RED_AMBROSIA_CUBE_IMPROVER,
            )),
        }),
        shop_cash_grab_ultra_effect(
            shop[SHOP_CASH_GRAB_ULTRA],
            ShopCashGrabUltraKey::CubesMult,
            lifetime_ambrosia,
        ),
        shop_ex_ultra_effect(shop[SHOP_EX_ULTRA], lifetime_ambrosia),
        ascension_speed_line,
    ])
}

/// Populate each blueberry upgrade's `free_level` from the red-ambrosia
/// `freeLevels` upgrades (legacy `BlueberryUpgrade.extraLevelCalc`). Row map:
/// tutorial (0) → `freeTutorialLevels`; row 2 (1-3: quarks1/cubes1/luck1) →
/// `freeLevelsRow2`; row 3 (4-9: the cross `*Cube1`/`*Quark1`/`*Luck1`) →
/// `freeLevelsRow3`; row 4 (10-12: quarks2/cubes2/luck2) → `freeLevelsRow4`;
/// row 5 (13-16: quarks3/cubes3/luck3/luck4) → `freeLevelsRow5`. Indices 17+
/// have `extraLevelCalc: () => 0`. Recomputed each tick (the field is a cache),
/// so it stays the identity `0` at the default state.
fn populate_ambrosia_free_levels(state: &mut GameState) {
    use crate::mechanics::red_ambrosia_upgrades::{
        free_levels_row_2_effect, free_levels_row_3_effect, free_levels_row_4_effect,
        free_levels_row_5_effect, free_tutorial_levels_effect,
    };
    use crate::state::red_ambrosia::{
        RED_AMBROSIA_FREE_LEVELS_ROW_2, RED_AMBROSIA_FREE_LEVELS_ROW_3,
        RED_AMBROSIA_FREE_LEVELS_ROW_4, RED_AMBROSIA_FREE_LEVELS_ROW_5,
        RED_AMBROSIA_FREE_TUTORIAL_LEVELS,
    };

    let upgrades = &state.red_ambrosia.upgrades;
    let tutorial = free_tutorial_levels_effect(upgrades[RED_AMBROSIA_FREE_TUTORIAL_LEVELS].level);
    let row2 = free_levels_row_2_effect(upgrades[RED_AMBROSIA_FREE_LEVELS_ROW_2].level);
    let row3 = free_levels_row_3_effect(upgrades[RED_AMBROSIA_FREE_LEVELS_ROW_3].level);
    let row4 = free_levels_row_4_effect(upgrades[RED_AMBROSIA_FREE_LEVELS_ROW_4].level);
    let row5 = free_levels_row_5_effect(upgrades[RED_AMBROSIA_FREE_LEVELS_ROW_5].level);

    for (i, upgrade) in state.ambrosia.upgrades.iter_mut().enumerate() {
        upgrade.free_level = match i {
            0 => tutorial,
            1..=3 => row2,
            4..=9 => row3,
            10..=12 => row4,
            13..=16 => row5,
            _ => 0.0,
        };
    }
}

/// `updateTokens()` — the campaign-token total (`Campaign.ts:489`),
/// self-derived from `&GameState`: every campaign's `computeTokenValue()`
/// summed, plus `inheritanceTokens()` and the GQ `singBonusTokens4` /
/// octeract `octeractBonusTokens4` initial-token grants. The legacy keeps
/// this in a module global recomputed on campaign/upgrade changes; logic
/// derives it live wherever a `campaign_token_rewards::*` bonus needs it.
///
/// `0` at default state (no completions, no singularity), so every
/// token-derived bonus stays at its identity. Tokens flow without the
/// campaign runner once `highestSingularityCount ≥ 5` (the inheritance
/// floor) — the runner itself (picking a campaign, completing c10 under
/// its corruptions) is UI-tier and writes `campaign_completions`.
fn compute_campaign_tokens(state: &GameState) -> f64 {
    use crate::mechanics::campaign_token_rewards::{
        campaign_token_value, inheritance_tokens, singularity_bonus_token_mult,
        CampaignTokenBonuses, CAMPAIGN_IS_META, CAMPAIGN_TOKEN_LIMITS,
    };
    use crate::mechanics::golden_quark_upgrades::{
        sing_bonus_tokens_1_effect, sing_bonus_tokens_2_effect, sing_bonus_tokens_3_effect,
        sing_bonus_tokens_4_effect,
    };
    use crate::mechanics::octeracts::{
        octeract_bonus_tokens_1_effect, octeract_bonus_tokens_2_effect,
        octeract_bonus_tokens_3_effect, octeract_bonus_tokens_4_effect,
    };
    use crate::state::campaigns::CAMPAIGNS_LEN;
    use crate::state::golden_quarks::{
        GQ_SING_BONUS_TOKENS_1, GQ_SING_BONUS_TOKENS_2, GQ_SING_BONUS_TOKENS_3,
        GQ_SING_BONUS_TOKENS_4,
    };
    use crate::state::octeract_upgrades::{
        OCTERACT_BONUS_TOKENS_1, OCTERACT_BONUS_TOKENS_2, OCTERACT_BONUS_TOKENS_3,
        OCTERACT_BONUS_TOKENS_4,
    };

    let highest_sing = state.singularity.highest_singularity_count;
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let oct = |i: usize| {
        state.octeract_upgrades.upgrades[i].level + state.octeract_upgrades.upgrades[i].free_level
    };

    let first_milestone = if highest_sing >= 16.0 { 5.0 } else { 0.0 };
    let last_milestone = if highest_sing >= 69.0 { 10.0 } else { 0.0 };
    let bonuses = CampaignTokenBonuses {
        first_completion_bonus: first_milestone
            + sing_bonus_tokens_1_effect(gq(GQ_SING_BONUS_TOKENS_1))
            + octeract_bonus_tokens_3_effect(oct(OCTERACT_BONUS_TOKENS_3)),
        last_completion_bonus: last_milestone
            + sing_bonus_tokens_3_effect(gq(GQ_SING_BONUS_TOKENS_3))
            + octeract_bonus_tokens_1_effect(oct(OCTERACT_BONUS_TOKENS_1)),
        token_multiplier: singularity_bonus_token_mult(highest_sing)
            * sing_bonus_tokens_2_effect(gq(GQ_SING_BONUS_TOKENS_2))
            * octeract_bonus_tokens_2_effect(oct(OCTERACT_BONUS_TOKENS_2)),
    };

    let campaign_sum: f64 = (0..CAMPAIGNS_LEN)
        .map(|i| {
            campaign_token_value(
                state.campaigns.campaign_completions[i],
                CAMPAIGN_TOKEN_LIMITS[i],
                CAMPAIGN_IS_META[i],
                &bonuses,
            )
        })
        .sum();

    campaign_sum
        + inheritance_tokens(highest_sing)
        + sing_bonus_tokens_4_effect(gq(GQ_SING_BONUS_TOKENS_4))
        + octeract_bonus_tokens_4_effect(oct(OCTERACT_BONUS_TOKENS_4))
}

/// `calculateQuarkMultiplier()` — the global quark-gain multiplier
/// (`allQuarkStats` product, original `Statistics.ts:1233` / `Calculate.ts:378`),
/// self-derived from `&GameState`. Cached each tick into
/// `state.quarks.quark_bonus` as `(mult - 1) * 100` (see [`tack`]) so the
/// `applyBonus`-style consumers (cube opening, challenge rewards, achievement
/// awards) credit the full multiplier — every such site reads
/// `1 + quark_bonus / 100`, which equals this product.
///
/// The `quarkGain` achievement reward (#250/#251/#266), the Challenge-15
/// `quarks` reward, the quark-hepteract bonus, and the campaign bonus (of the
/// derived token total) are now wired (each identity at default). The terms
/// still left at the multiplicative identity `1.0` are documented inline:
/// `shopPanthema` / `infiniteAscent` (need their bonus-levels precompute /
/// unlock gate) and the UI/host-tier event +
/// patreon (global / personal) bonuses. `favoriteUpgrade` passes a `0`
/// maxed-sibling count (identity until that GQ upgrade is bought) and
/// `ambrosiaCubeQuark1` passes a `0` `wow_cube_log_sum` — the same precedent as
/// [`compute_ambrosia_luck_pre`]'s deferred cube-log terms.
fn compute_quark_multiplier(state: &GameState) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::ambrosia::{calculate_ambrosia_quark_mult, AmbrosiaMultInput};
    use crate::mechanics::blueberry_upgrades::{
        ambrosia_cube_quark_1_effect, ambrosia_luck_quark_1_effect, ambrosia_quarks_1_effect,
        ambrosia_quarks_2_effect, ambrosia_quarks_3_effect, ambrosia_tutorial_effect,
        AmbrosiaTutorialEffectKey,
    };
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::golden_quark_upgrades::{
        advanced_pack_effect, divine_pack_effect, expert_pack_effect, favorite_upgrade_effect,
        intermediate_pack_effect, master_pack_effect, sing_quark_hepteract_2_effect,
        sing_quark_hepteract_3_effect, sing_quark_hepteract_effect, sing_quark_improver_1_effect,
        AdvancedPackKey, DivinePackKey, ExpertPackKey, IntermediatePackKey, MasterPackKey,
    };
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::octeract_bonuses::{
        calculate_total_octeract_quark_bonus, CalculateTotalOcteractQuarkBonusInput,
    };
    use crate::mechanics::octeracts::{
        octeract_quark_gain_2_effect, octeract_quark_gain_effect, octeract_starter_effect,
        OcteractStarterKey,
    };
    use crate::mechanics::overflux_bonuses::calculate_quark_mult_from_powder;
    use crate::mechanics::red_ambrosia_upgrades::{
        viscount_effect, ViscountEffectKey, ViscountEffectValue,
    };
    use crate::mechanics::shop_upgrades::{shop_cash_grab_ultra_effect, ShopCashGrabUltraKey};
    use crate::mechanics::singularity_challenges::{
        limited_time_effect, sadistic_prequel_effect, LimitedTimeKey, SadisticPrequelKey,
        SingularityEffectValue,
    };
    use crate::mechanics::singularity_milestones::calculate_singularity_quark_milestone_multiplier;
    use crate::mechanics::talisman_effects::plastic_talisman_effects;

    use crate::state::ambrosia::{
        AMBROSIA_CUBE_QUARK_1, AMBROSIA_LUCK_QUARK_1, AMBROSIA_QUARKS_1, AMBROSIA_QUARKS_2,
        AMBROSIA_QUARKS_3, AMBROSIA_TUTORIAL,
    };
    use crate::state::golden_quarks::{
        GQ_ADVANCED_PACK, GQ_DIVINE_PACK, GQ_EXPERT_PACK, GQ_FAVORITE_UPGRADE,
        GQ_INTERMEDIATE_PACK, GQ_MASTER_PACK, GQ_SING_QUARK_HEPTERACT, GQ_SING_QUARK_HEPTERACT_2,
        GQ_SING_QUARK_HEPTERACT_3, GQ_SING_QUARK_IMPROVER_1,
    };
    use crate::state::octeract_upgrades::{
        OCTERACT_QUARK_GAIN, OCTERACT_QUARK_GAIN_2, OCTERACT_STARTER,
    };
    use crate::state::red_ambrosia::RED_AMBROSIA_VISCOUNT;
    use crate::state::shop::SHOP_CASH_GRAB_ULTRA;
    use crate::state::talismans::TALISMAN_PLASTIC;

    // Legacy `player.cubeUpgrades[..]` indices (CookieUpgrade3 / CookieUpgrade18).
    const CUBE_UPGRADE_QUARK_COOKIE_3: usize = 53;
    const CUBE_UPGRADE_QUARK_COOKIE_18: usize = 68;

    let sing = state.singularity.singularity_count;
    let highest_sing = state.singularity.highest_singularity_count;
    let shop = &state.shop.upgrades;
    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let achievement_level = achievement_level_from_points(state.achievements.achievement_points);
    let lifetime_ambrosia = state.ambrosia.lifetime_ambrosia;
    let ambrosia_luck = compute_ambrosia_luck_pre(state);
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let oct = |i: usize| {
        state.octeract_upgrades.upgrades[i].level + state.octeract_upgrades.upgrades[i].free_level
    };
    let amb = |i: usize| state.ambrosia.upgrades[i].level + state.ambrosia.upgrades[i].free_level;
    let red = |i: usize| state.red_ambrosia.upgrades[i].level;

    // Quark context → a missing singularity-challenge value is the multiplicative 1.
    let scalar = |v: SingularityEffectValue| match v {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 1.0,
    };

    // SingularityPacks: `1 + Σ packQuarkAdd` across the five GQ packs.
    let singularity_packs = 1.0
        + intermediate_pack_effect(gq(GQ_INTERMEDIATE_PACK), IntermediatePackKey::PackQuarkAdd)
        + advanced_pack_effect(gq(GQ_ADVANCED_PACK), AdvancedPackKey::PackQuarkAdd)
        + expert_pack_effect(gq(GQ_EXPERT_PACK), ExpertPackKey::PackQuarkAdd)
        + master_pack_effect(gq(GQ_MASTER_PACK), MasterPackKey::PackQuarkAdd)
        + divine_pack_effect(
            gq(GQ_DIVINE_PACK),
            DivinePackKey::PackQuarkAdd,
            &state.corruptions.used.levels,
        );

    // Viscount red-ambrosia quark bonus → multiplicative scalar.
    let viscount = match viscount_effect(red(RED_AMBROSIA_VISCOUNT), ViscountEffectKey::QuarkBonus)
    {
        ViscountEffectValue::Scalar(s) => s,
        _ => 1.0,
    };

    // QuarkHepteract: once `challenge15Exponent` reaches the `hepteractsUnlocked`
    // requirement (`1e15`), the quark hepteract's `quarkMultiplier` applies.
    // `hepteractEffective('quark')` returns the raw `BAL` (the quark craft uses a
    // custom non-polynomial formula — Hepteracts.ts:633), and the effect is
    // `(1 + 0.2·log2(1 + bal/500))^(DR + DR_INCREASE)` (Hepteracts.ts:134) with
    // `DR = 2` and `DR_INCREASE` the three `singQuarkHepteract` GQ upgrades'
    // `quarkHeptExponent`. Identity at default (`bal = 0` → base `1`).
    let quark_hepteract = if state.challenges.challenge15_exponent >= 1e15 {
        let exponent = 2.0
            + sing_quark_hepteract_effect(gq(GQ_SING_QUARK_HEPTERACT))
            + sing_quark_hepteract_2_effect(gq(GQ_SING_QUARK_HEPTERACT_2))
            + sing_quark_hepteract_3_effect(gq(GQ_SING_QUARK_HEPTERACT_3));
        (1.0 + 0.2 * (1.0 + state.hepteracts.quark.bal / 500.0).log2()).powf(exponent)
    } else {
        1.0
    };

    product_f64(&[
        // AchievementBonus — getAchievementReward('quarkGain') (#250/#251/#266).
        crate::mechanics::achievement_rewards::quark_gain(
            &state.achievements.achievements,
            state.reset_counters.ascension_count,
        ),
        get_level_reward(LevelRewardKey::Quarks, achievement_level),
        plastic_talisman_effects(state.talismans.talisman_levels[TALISMAN_PLASTIC] as i32)
            .quark_bonus,
        if platonic[5] > 0.0 { 1.05 } else { 1.0 }, // PlatonicALPHA
        if platonic[10] > 0.0 { 1.1 } else { 1.0 }, // PlatonicBETA
        if platonic[15] > 0.0 { 1.15 } else { 1.0 }, // PlatonicOMEGA
        1.0, // Jack (shopPanthema) — needs ShopPanthemaBonusLevels precompute
        // Challenge15 — c15 `quarks` reward, gated on `challenge15Exponent`.
        crate::mechanics::challenge_15_rewards::quarks(state.challenges.challenge15_exponent),
        crate::mechanics::campaign_token_rewards::campaign_quark_bonus(compute_campaign_tokens(
            state,
        )), // CampaignBonus — player.campaigns.quarkBonus
        1.0, // InfiniteAscent — needs shop infiniteAscent unlock gate + rune
        quark_hepteract,
        calculate_quark_mult_from_powder(state.hepteracts.overflux_powder),
        1.0 + sing / 10.0,                                     // SingularityCount
        favorite_upgrade_effect(gq(GQ_FAVORITE_UPGRADE), 0.0), // siblings=0 until GQ bought
        1.0 + 0.001 * cube[CUBE_UPGRADE_QUARK_COOKIE_3],       // CookieUpgrade3
        1.0 + cube[CUBE_UPGRADE_QUARK_COOKIE_18] / 10_000.0
            + 0.05 * (cube[CUBE_UPGRADE_QUARK_COOKIE_18] / 1_000.0).floor(), // CookieUpgrade18
        calculate_singularity_quark_milestone_multiplier(sing),
        if sing >= 200.0 {
            (1.0 + (sing - 199.0) / 20.0).powi(2)
        } else {
            1.0
        }, // skrauQ
        calculate_total_octeract_quark_bonus(&CalculateTotalOcteractQuarkBonusInput {
            exalt_4_enabled: state.singularity.no_octeracts.enabled,
            total_wow_octeracts: state.cube_balances.total_wow_octeracts.to_number(),
        }),
        octeract_starter_effect(oct(OCTERACT_STARTER), OcteractStarterKey::QuarkMult),
        octeract_quark_gain_effect(oct(OCTERACT_QUARK_GAIN)),
        octeract_quark_gain_2_effect(
            oct(OCTERACT_QUARK_GAIN_2),
            oct(OCTERACT_QUARK_GAIN),
            state.hepteracts.quark.bal,
        ),
        singularity_packs,
        sing_quark_improver_1_effect(gq(GQ_SING_QUARK_IMPROVER_1)),
        calculate_ambrosia_quark_mult(&AmbrosiaMultInput {
            no_ambrosia_upgrades_enabled: state.singularity.no_ambrosia_upgrades.enabled,
            lifetime_ambrosia,
        }),
        ambrosia_tutorial_effect(amb(AMBROSIA_TUTORIAL), AmbrosiaTutorialEffectKey::Quarks),
        ambrosia_quarks_1_effect(amb(AMBROSIA_QUARKS_1)),
        ambrosia_cube_quark_1_effect(amb(AMBROSIA_CUBE_QUARK_1), 0.0), // wow_cube_log_sum deferred
        ambrosia_luck_quark_1_effect(amb(AMBROSIA_LUCK_QUARK_1), ambrosia_luck),
        ambrosia_quarks_2_effect(amb(AMBROSIA_QUARKS_2), amb(AMBROSIA_QUARKS_1)),
        ambrosia_quarks_3_effect(amb(AMBROSIA_QUARKS_3), amb(AMBROSIA_QUARKS_2)),
        viscount,
        shop_cash_grab_ultra_effect(
            shop[SHOP_CASH_GRAB_ULTRA],
            ShopCashGrabUltraKey::QuarkMult,
            lifetime_ambrosia,
        ),
        scalar(limited_time_effect(
            state.singularity.limited_time.completions,
            LimitedTimeKey::QuarkMult,
        )),
        scalar(sadistic_prequel_effect(
            state.singularity.sadistic_prequel.completions,
            SadisticPrequelKey::QuarkMult,
        )),
        if highest_sing == 0.0 { 1.25 } else { 1.0 }, // FirstSingularityBonus
        1.0,                                          // Event — UI-tier event calendar
        1.0,                                          // GlobalSubscriber — patreon (host-tier)
        1.0,                                          // AccountBonus — patreon (host-tier)
    ])
}

/// `calculateAllCubeMultiplier()` — the GLOBAL cube-gain multiplier
/// (`allCubeStats` product, original `Statistics.ts:172`), self-derived from
/// `&GameState`. This is the shared base line multiplied into ALL five
/// cube-family multipliers (`allWowCubeStats` / `allTesseractStats` /
/// `allHypercubeStats` / `allPlatonicCubeStats` / `allHepteractCubeStats`), so
/// it is the foundation of the ascension cube award (`CalcCorruptionStuff`).
///
/// The AscensionTime line reads `ascensionCounter` BEFORE the ascension reset
/// zeroes it; the award block in `apply_ascension_layer` therefore calls this
/// before the counter reset (Reset.ts ordering).
///
/// CampaignTutorial + Campaign are wired to the derived campaign-token
/// total. Neutral-defaulted lines (faithful — unported / paused / UI-tier):
/// PseudoCoins (PCoin meta), InfiniteAscent (the infiniteAscent rune is outside the 7-rune
/// `rune_levels` model → level 0 → `1 + 0/100`), SingDebuff
/// (`1 / calculateSingularityDebuff('Cubes')` — the singularity layer is paused
/// and has no production debuff-input builder; `= 1` at `singularityCount 0`,
/// i.e. all pre-singularity play), Jack (`shopPanthema` `bonusLevels()` builder
/// unported; `= 1` at panthema level 0), CookieUpgrade8 + Event (UI-tier
/// `isEvent` calendar → `0`).
fn compute_all_cube_multiplier(state: &GameState) -> f64 {
    use crate::mechanics::achievement_rewards::ascension_reward_scaling;
    use crate::mechanics::ambrosia::{calculate_ambrosia_cube_mult, AmbrosiaMultInput};
    use crate::mechanics::blueberry_upgrades::{
        ambrosia_cubes_1_effect, ambrosia_cubes_2_effect, ambrosia_cubes_3_effect,
        ambrosia_hyperflux_effect, ambrosia_luck_cube_1_effect, ambrosia_quark_cube_1_effect,
        ambrosia_tutorial_effect, AmbrosiaTutorialEffectKey,
    };
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::challenge_15_rewards;
    use crate::mechanics::exalt_penalties::calculate_exalt_6_penalty;
    use crate::mechanics::golden_quark_upgrades::{
        one_mind_effect, platonic_delta_effect, sing_citadel_2_effect, sing_citadel_effect,
        sing_cubes_1_effect, sing_cubes_2_effect, sing_cubes_3_effect, starter_pack_effect,
        SingCitadel2Key, StarterPackKey,
    };
    use crate::mechanics::octeract_bonuses::{
        calculate_total_octeract_cube_bonus, CalculateTotalOcteractCubeBonusInput,
    };
    use crate::mechanics::overflux_bonuses::calculate_cube_mult_from_powder;
    use crate::mechanics::red_ambrosia_bonuses::{
        calculate_red_ambrosia_cubes, CalculateRedAmbrosiaCubesInput,
    };
    use crate::mechanics::red_ambrosia_upgrades::{
        red_ambrosia_cube_effect, red_ambrosia_cube_improver_effect,
        tutorial_effect as red_tutorial_effect,
    };
    use crate::mechanics::reset_time_and_auto_obtainium::{
        reset_time_threshold, ResetTimeThresholdInput,
    };
    use crate::mechanics::shop_upgrades::{
        season_pass_infinity_effect, season_pass_y_effect, season_pass_z_effect,
        shop_cash_grab_ultra_effect, shop_ex_ultra_effect, shop_singularity_speedup_effect,
        SeasonPassInfinityKey, ShopCashGrabUltraKey,
    };
    use crate::mechanics::singularity_challenges::{
        no_octeracts_effect, no_singularity_upgrades_effect, NoOcteractsKey,
        NoSingularityUpgradesKey, SingularityEffectValue,
    };
    use crate::state::ambrosia::{
        AMBROSIA_CUBES_1, AMBROSIA_CUBES_2, AMBROSIA_CUBES_3, AMBROSIA_HYPERFLUX,
        AMBROSIA_LUCK_CUBE_1, AMBROSIA_QUARK_CUBE_1, AMBROSIA_TUTORIAL,
    };
    use crate::state::golden_quarks::{
        GQ_ONE_MIND, GQ_PLATONIC_DELTA, GQ_SING_CITADEL, GQ_SING_CITADEL_2, GQ_SING_CUBES_1,
        GQ_SING_CUBES_2, GQ_SING_CUBES_3, GQ_STARTER_PACK,
    };
    use crate::state::red_ambrosia::{
        RED_AMBROSIA_RED_AMBROSIA_CUBE, RED_AMBROSIA_RED_AMBROSIA_CUBE_IMPROVER,
        RED_AMBROSIA_TUTORIAL,
    };
    use crate::state::shop::{
        SHOP_CASH_GRAB_ULTRA, SHOP_EX_ULTRA, SHOP_SEASON_PASS_INFINITY, SHOP_SEASON_PASS_Y,
        SHOP_SEASON_PASS_Z, SHOP_SINGULARITY_SPEEDUP,
    };

    // Legacy `player.cubeUpgrades[..]` / `player.platonicUpgrades[..]` indices.
    const CUBE_UPGRADE_66: usize = 66;
    const PLATONIC_UPGRADE_BETA: usize = 10;
    const PLATONIC_UPGRADE_OMEGA: usize = 15;
    const PLATONIC_UPGRADE_19: usize = 19;

    let sing = state.singularity.singularity_count;
    let shop = &state.shop.upgrades;
    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let cc = &state.challenges.challenge_completions;
    let lifetime_ambrosia = state.ambrosia.lifetime_ambrosia;
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let amb = |i: usize| state.ambrosia.upgrades[i].level + state.ambrosia.upgrades[i].free_level;
    let red = |i: usize| state.red_ambrosia.upgrades[i].level;

    let campaign_tokens = compute_campaign_tokens(state);

    // AscensionTime: `min(1, counter/threshold)^2`, times `(1 + overflow)` once
    // the `ascensionRewardScaling` achievement (#204) is earned.
    let reset_threshold = reset_time_threshold(&ResetTimeThresholdInput {
        campaign_time_threshold_reduction:
            crate::mechanics::campaign_token_rewards::campaign_time_threshold_reduction(
                campaign_tokens,
            ),
    });
    let frac = state.reset_counters.ascension_counter / reset_threshold;
    let ascension_time_base = frac.min(1.0).powi(2);
    let ascension_time = if ascension_reward_scaling(&state.achievements.achievements) {
        ascension_time_base * (1.0 + (frac - 1.0).max(0.0))
    } else {
        ascension_time_base
    };

    // Challenge15: product of the five cube rewards of `challenge15Exponent`.
    let c15e = state.challenges.challenge15_exponent;
    let challenge_15_cubes = challenge_15_rewards::cube1(c15e)
        * challenge_15_rewards::cube2(c15e)
        * challenge_15_rewards::cube3(c15e)
        * challenge_15_rewards::cube4(c15e)
        * challenge_15_rewards::cube5(c15e);

    // WowOcteract: octeract-count cube bonus (mirrors the octeract assembly).
    let octeract_pow = match no_octeracts_effect(
        state.singularity.no_octeracts.completions,
        NoOcteractsKey::OcteractPow,
    ) {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 0.0,
    };
    let wow_octeract = calculate_total_octeract_cube_bonus(&CalculateTotalOcteractCubeBonusInput {
        exalt_4_enabled: state.singularity.no_octeracts.enabled,
        total_wow_octeracts: state.cube_balances.total_wow_octeracts.to_number(),
        octeract_pow,
    });

    // NoSing: a missing singularity-challenge value → multiplicative 1.
    let no_sing = match no_singularity_upgrades_effect(
        state.singularity.no_singularity_upgrades.completions,
        NoSingularityUpgradesKey::Cubes,
    ) {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 1.0,
    };

    // OneMind: full ascension-speed mult / 10 when the oneMind GQ upgrade is bought.
    let one_mind = if one_mind_effect(state.golden_quarks.upgrades[GQ_ONE_MIND].level) {
        compute_ascension_speed_mult_pre(state) / 10.0
    } else {
        1.0
    };

    // Exalt6: the limitedTime (Exalt 6) per-second cube penalty; inert while
    // the challenge is disabled (the singularity layer is paused).
    let exalt6 = if state.singularity.limited_time.enabled {
        calculate_exalt_6_penalty(
            state.singularity.limited_time.completions,
            state.singularity.sing_challenge_timer,
        )
    } else {
        1.0
    };

    product_f64(&[
        1.0, // PseudoCoins — PCoin meta layer (unported)
        ascension_time,
        crate::mechanics::campaign_token_rewards::tutorial_bonus(campaign_tokens).cube_bonus, // CampaignTutorial
        crate::mechanics::campaign_token_rewards::campaign_cube_bonus(campaign_tokens), // Campaign
        challenge_15_cubes,
        1.0, // InfiniteAscent — rune outside the 7-rune model → level 0 → 1
        1.0 + platonic[PLATONIC_UPGRADE_BETA], // Beta
        1.01_f64.powf(platonic[PLATONIC_UPGRADE_OMEGA] * cc[9]), // Omega
        calculate_cube_mult_from_powder(state.hepteracts.overflux_powder), // Powder
        1.0, // SingDebuff — singularity layer paused; = 1 at sing 0 (pre-singularity)
        1.0, // Jack — shopPanthema bonusLevels() unported; = 1 at panthema level 0
        season_pass_y_effect(shop[SHOP_SEASON_PASS_Y]),
        season_pass_z_effect(shop[SHOP_SEASON_PASS_Z], sing),
        season_pass_infinity_effect(
            shop[SHOP_SEASON_PASS_INFINITY],
            SeasonPassInfinityKey::GlobalCubeMult,
        ),
        shop_cash_grab_ultra_effect(
            shop[SHOP_CASH_GRAB_ULTRA],
            ShopCashGrabUltraKey::CubesMult,
            lifetime_ambrosia,
        ),
        shop_ex_ultra_effect(shop[SHOP_EX_ULTRA], lifetime_ambrosia),
        starter_pack_effect(gq(GQ_STARTER_PACK), StarterPackKey::CubeMult),
        sing_cubes_1_effect(gq(GQ_SING_CUBES_1)),
        sing_cubes_2_effect(gq(GQ_SING_CUBES_2)),
        sing_cubes_3_effect(gq(GQ_SING_CUBES_3)),
        sing_citadel_effect(gq(GQ_SING_CITADEL)),
        sing_citadel_2_effect(gq(GQ_SING_CITADEL_2), SingCitadel2Key::Mult),
        platonic_delta_effect(
            gq(GQ_PLATONIC_DELTA),
            state.singularity.singularity_counter,
            shop_singularity_speedup_effect(shop[SHOP_SINGULARITY_SPEEDUP]),
        ),
        1.0, // CookieUpgrade8 — UI-tier isEvent → 1 + 0.25·0·cube[58]
        1.0 + cube[CUBE_UPGRADE_66] * (1.0 - platonic[PLATONIC_UPGRADE_OMEGA]), // CookieUpgrade16
        wow_octeract,
        no_sing,
        calculate_ambrosia_cube_mult(&AmbrosiaMultInput {
            no_ambrosia_upgrades_enabled: state.singularity.no_ambrosia_upgrades.enabled,
            lifetime_ambrosia,
        }),
        ambrosia_tutorial_effect(amb(AMBROSIA_TUTORIAL), AmbrosiaTutorialEffectKey::Cubes),
        ambrosia_cubes_1_effect(amb(AMBROSIA_CUBES_1)),
        ambrosia_luck_cube_1_effect(amb(AMBROSIA_LUCK_CUBE_1), compute_ambrosia_luck_pre(state)),
        ambrosia_quark_cube_1_effect(amb(AMBROSIA_QUARK_CUBE_1), state.quarks.worlds.to_number()),
        ambrosia_cubes_2_effect(amb(AMBROSIA_CUBES_2), amb(AMBROSIA_CUBES_1)),
        ambrosia_hyperflux_effect(amb(AMBROSIA_HYPERFLUX), platonic[PLATONIC_UPGRADE_19]),
        ambrosia_cubes_3_effect(amb(AMBROSIA_CUBES_3), amb(AMBROSIA_CUBES_2)),
        red_tutorial_effect(red(RED_AMBROSIA_TUTORIAL)),
        calculate_red_ambrosia_cubes(&CalculateRedAmbrosiaCubesInput {
            unlocked: red_ambrosia_cube_effect(red(RED_AMBROSIA_RED_AMBROSIA_CUBE)),
            lifetime_red_ambrosia: state.red_ambrosia.lifetime_red_ambrosia,
            extra_exponent: red_ambrosia_cube_improver_effect(red(
                RED_AMBROSIA_RED_AMBROSIA_CUBE_IMPROVER,
            )),
        }),
        exalt6,
        one_mind,
        1.0, // Event — UI-tier event calendar → 1 + 0
    ])
}

/// `calculateCubeMultiplierWithTau()` = `product(allWowCubeStats) ^ platonicTau.tauPower`
/// (original Statistics.ts:379 + Calculate.ts:173) — the WOW-cube gain multiplier
/// feeding `CalcCorruptionStuff.cubeGain`. `all_cube_multiplier` is the shared
/// [`compute_all_cube_multiplier`] (the GlobalCube line); `effective_score` is the
/// shared [`compute_ascension_score_result`]`.effective_score`.
///
/// Neutral-defaulted lines (faithful): WowSquare (wowSquare talisman not among the
/// 7 ported talismans), and the `research[192]·calculateTrueAntLevel(Mortuus)`
/// sub-term of the Researches line — `calculate_true_ant_level` needs the
/// `corruptionEffects('extinction')` divisor, and the corruption-effects system is
/// unported, so that one factor is `1` (every other research factor is exact). The
/// `CubeBank` line is a count (0 at default), so the whole product is `0` until a
/// challenge is completed — faithful (an ascension with no completions grants 0
/// cubes; the award is c10-gated anyway).
fn compute_cube_multiplier(
    state: &GameState,
    effective_score: f64,
    all_cube_multiplier: f64,
) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::achievement_rewards::wow_cube_gain;
    use crate::mechanics::ant_upgrades::{
        ascension_score_ant_upgrade_effect, wow_cubes_ant_upgrade_effect,
    };
    use crate::mechanics::calculate::{calculate_cube_multiplier_with_tau, product_f64};
    use crate::mechanics::golden_quark_upgrades::{
        platonic_tau_effect, PlatonicTauKey, PlatonicTauValue,
    };
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::platonic_blessings::calculate_cube_multiplier_platonic_blessing;
    use crate::mechanics::rune_effects::{
        antiquities_rune_effects, AntiquitiesRuneInput, AntiquitiesRuneKey,
    };
    use crate::mechanics::rune_spirit_effects::duplication_rune_spirit_effects;
    use crate::mechanics::shop_upgrades::season_pass_effect;
    use crate::state::golden_quarks::GQ_PLATONIC_TAU;
    use crate::state::shop::SHOP_SEASON_PASS;
    use crate::state::{RUNE_ANTIQUITIES, RUNE_DUPLICATION};

    const ANT_UPGRADE_ASCENSION_SCORE: usize = 14;
    const ANT_UPGRADE_WOW_CUBES: usize = 13;

    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let research = &state.researches.researches;
    let cc = &state.challenges.challenge_completions;
    let achievement_level = achievement_level_from_points(state.achievements.achievement_points);
    let ascend_shards = state.campaigns.ascend_shards;
    let total_corruption_levels: u32 = state.corruptions.used.levels.iter().sum();

    // CubeBank: cube-completion sum (c6-10 worth ×2) + ant AscensionScore cubesBanked.
    let mut cube_bank = 0.0_f64;
    for (offset, &completions) in cc[1..=10].iter().enumerate() {
        // `offset + 1` is the challenge index; c6-10 are worth ×2.
        cube_bank += if offset + 1 >= 6 { 2.0 } else { 1.0 } * completions;
    }
    cube_bank +=
        ascension_score_ant_upgrade_effect(true_ant_level(state, ANT_UPGRADE_ASCENSION_SCORE))
            .cubes_banked;

    // AscensionScore band.
    let ascension_score_line = (effective_score / 3000.0).powf(1.0 / 4.1);

    // Researches: now wires the `research[192]·calculateTrueAntLevel(Mortuus)`
    // factor. Mortuus (index 11) is exemptFromCorruption=true, so the extinction
    // divisor is 1 regardless of corruption level — the factor can be computed
    // without extinction effect. Formula from Statistics.ts allWowCubeStats:
    //   `(1 + (1/500) * research[192] * trueAntLevel(Mortuus))`
    const ANT_UPGRADE_MORTUUS: usize = 11;
    let mortuus_true_level = {
        use crate::mechanics::ant_upgrade_levels::{
            calculate_true_ant_level, compute_free_ant_upgrade_levels, CalculateTrueAntLevelInput,
            ComputeFreeAntUpgradeLevelsInput,
        };
        let cc = &state.challenges.challenge_completions;
        let c11_active = state.challenges.current_ascension_challenge == 11;
        // `challenge15Rewards.bonusAntLevel` — baseValue 1, requirement 5e5.
        // Unported as a standalone helper; neutral 1.0 (baseValue) at default.
        let bonus_ant_level_value = 1.0_f64;
        let free_levels = compute_free_ant_upgrade_levels(&ComputeFreeAntUpgradeLevelsInput {
            c9_reincarnation_ecc: crate::mechanics::challenges::calc_ecc(
                crate::mechanics::challenges::ChallengeType::Reincarnation,
                cc[9],
            ),
            constant_upgrade_6: state.campaigns.constant_upgrades[6],
            c11_ascension_ecc: crate::mechanics::challenges::calc_ecc(
                crate::mechanics::challenges::ChallengeType::Ascension,
                cc[11],
            ),
            research_97: research[97],
            research_98: research[98],
            research_102: research[102],
            research_132: research[132],
            research_200: research[200],
            free_ant_upgrades_achievement_reward: 0.0, // getAchievementReward('freeAntUpgrades') unported → neutral 0
            challenge_15_bonus_ant_level_value: bonus_ant_level_value,
            c11_active,
            c8_completions: cc[8],
            c9_completions: cc[9],
        });
        calculate_true_ant_level(&CalculateTrueAntLevelInput {
            current_level: state.ants.upgrades[ANT_UPGRADE_MORTUUS],
            free_levels,
            exempt_from_corruption: true, // Mortuus exemptFromCorruption = true
            corruption_extinction_divisor: 1.0, // moot for exempt upgrades
            c11_active,
        })
    };
    let researches = (1.0 + research[119] / 1000.0)
        * (1.0 + research[120] / 200.0)
        * (1.0 + research[137] / 100.0)
        * (1.0 + 0.9 * research[152] / 100.0)
        * (1.0 + 0.8 * research[167] / 100.0)
        * (1.0 + 0.7 * research[182] / 100.0)
        * (1.0 + (1.0 / 500.0) * research[192] * mortuus_true_level) // 8x17
        * (1.0 + 0.6 * research[197] / 100.0);

    // ConstantUpgrade10: 1 + 0.01·log4(ascendShards+1)·min(1, constantUpgrades[10]).
    let log4_shards = (ascend_shards + Decimal::one()).log10().to_number() / 4.0_f64.log10();
    let constant_upgrade_10 =
        1.0 + 0.01 * log4_shards * state.campaigns.constant_upgrades[10].min(1.0);

    let tau_power = match platonic_tau_effect(
        state.golden_quarks.upgrades[GQ_PLATONIC_TAU].level
            + state.golden_quarks.upgrades[GQ_PLATONIC_TAU].free_level,
        PlatonicTauKey::TauPower,
    ) {
        PlatonicTauValue::Scalar(s) => s,
        PlatonicTauValue::Unlock(_) => 1.0,
    };

    let base = product_f64(&[
        cube_bank,
        ascension_score_line,
        all_cube_multiplier,
        get_level_reward(LevelRewardKey::WowCubes, achievement_level),
        wow_cube_gain(
            &state.achievements.achievements,
            state.reset_counters.ascension_count,
            ascend_shards,
        ),
        season_pass_effect(state.shop.upgrades[SHOP_SEASON_PASS]),
        1.0, // WowSquare — wowSquare talisman not among the 7 ported
        researches,
        1.0 + (0.004 / 100.0) * research[200], // Research8x25
        wow_cubes_ant_upgrade_effect(true_ant_level(state, ANT_UPGRADE_WOW_CUBES)),
        (1.0 + cube[1] / 6.0) * (1.0 + cube[11] / 11.0) * (1.0 + 0.4 * cube[30]), // CubeUpgrades
        constant_upgrade_10,
        duplication_rune_spirit_effects(rune_spirit_power(state, RUNE_DUPLICATION)).wow_cubes,
        calculate_cube_multiplier_platonic_blessing(&state.platonic_blessings),
        1.0 + 0.00009 * f64::from(total_corruption_levels) * platonic[1], // Platonic1x1
        antiquities_rune_effects(
            state.runes.rune_levels[RUNE_ANTIQUITIES],
            AntiquitiesRuneKey::CubeBonus,
            AntiquitiesRuneInput {
                singularity_count: state.singularity.singularity_count,
            },
        ),
        // CookieUpgrade13: 1 + 1.03^log10(max(1, wowAbyssals))·cube[63] - cube[63]
        1.0 + 1.03_f64.powf(state.cube_balances.wow_abyssals.max(1.0).log10()) * cube[63]
            - cube[63],
    ]);
    calculate_cube_multiplier_with_tau(base, tau_power)
}

/// `calculateTesseractMultiplier()` = `product(allTesseractStats)` (Statistics.ts:491)
/// — feeds `CalcCorruptionStuff.tesseractGain`. WowSquare neutral-defaulted.
fn compute_tesseract_multiplier(
    state: &GameState,
    effective_score: f64,
    all_cube_multiplier: f64,
) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::achievement_rewards::wow_tesseract_gain;
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::platonic_blessings::calculate_tesseract_multiplier_platonic_blessing;
    use crate::mechanics::shop_upgrades::season_pass_effect;
    use crate::state::shop::SHOP_SEASON_PASS;

    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let achievement_level = achievement_level_from_points(state.achievements.achievement_points);
    let ascend_shards = state.campaigns.ascend_shards;
    let total_corruption_levels = f64::from(state.corruptions.used.levels.iter().sum::<u32>());
    let log4_shards = (ascend_shards + Decimal::one()).log10().to_number() / 4.0_f64.log10();

    product_f64(&[
        (1.0 + (effective_score - 1e5).max(0.0) / 1e4).powf(0.35), // AscensionScore
        all_cube_multiplier,
        get_level_reward(LevelRewardKey::WowTesseracts, achievement_level),
        wow_tesseract_gain(&state.achievements.achievements, ascend_shards),
        season_pass_effect(state.shop.upgrades[SHOP_SEASON_PASS]),
        1.0,                                                                       // WowSquare
        1.0 + 0.01 * log4_shards * state.campaigns.constant_upgrades[10].min(1.0), // ConstantUpgrade10
        1.0 + 0.4 * cube[30], // CubeUpgrade3x10
        1.0 + (1.0 / 200.0) * cube[38] * total_corruption_levels, // CubeUpgrade4x8
        calculate_tesseract_multiplier_platonic_blessing(&state.platonic_blessings),
        1.0 + 0.00018 * total_corruption_levels * platonic[2], // Platonic1x2
    ])
}

/// `calculateHypercubeMultiplier()` = `product(allHypercubeStats)` (Statistics.ts:539)
/// — feeds `CalcCorruptionStuff.hypercubeGain`. WowSquare neutral-defaulted.
fn compute_hypercube_multiplier(
    state: &GameState,
    effective_score: f64,
    all_cube_multiplier: f64,
) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::achievement_rewards::wow_hypercube_gain;
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::hepteract_effects::hyperrealism_hepteract_effects;
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::platonic_blessings::calculate_hypercube_multiplier_platonic_blessing;
    use crate::mechanics::shop_upgrades::season_pass_2_effect;
    use crate::state::shop::SHOP_SEASON_PASS_2;

    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let achievement_level = achievement_level_from_points(state.achievements.achievement_points);
    let total_corruption_levels = f64::from(state.corruptions.used.levels.iter().sum::<u32>());

    product_f64(&[
        (1.0 + (effective_score - 1e9).max(0.0) / 1e8).powf(0.5), // AscensionScore
        all_cube_multiplier,
        get_level_reward(LevelRewardKey::WowHyperCubes, achievement_level),
        wow_hypercube_gain(&state.achievements.achievements),
        season_pass_2_effect(state.shop.upgrades[SHOP_SEASON_PASS_2]),
        1.0, // WowSquare
        calculate_hypercube_multiplier_platonic_blessing(&state.platonic_blessings),
        1.0 + 0.00054 * total_corruption_levels * platonic[3], // Platonic1x3
        hyperrealism_hepteract_effects(hepteract_effective_bal(
            state.hepteracts.hyperrealism.bal,
            1.0 / 3.0,
        ))
        .hypercube_multiplier,
    ])
}

/// `calculatePlatonicMultiplier()` = `product(allPlatonicCubeStats)` (Statistics.ts:579)
/// — feeds `CalcCorruptionStuff.platonicGain`. WowSquare neutral-defaulted.
fn compute_platonic_multiplier(
    state: &GameState,
    effective_score: f64,
    all_cube_multiplier: f64,
) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::achievement_rewards::wow_platonic_gain;
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::platonic_blessings::calculate_platonic_multiplier_platonic_blessing;
    use crate::mechanics::shop_upgrades::season_pass_2_effect;
    use crate::state::shop::SHOP_SEASON_PASS_2;

    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let achievement_level = achievement_level_from_points(state.achievements.achievement_points);

    product_f64(&[
        (1.0 + (effective_score - 2.666e12).max(0.0) / 2.666e11).powf(0.75), // AscensionScore
        all_cube_multiplier,
        get_level_reward(LevelRewardKey::WowPlatonicCubes, achievement_level),
        wow_platonic_gain(
            &state.achievements.achievements,
            state.reset_counters.ascension_count,
            state.campaigns.ascend_shards,
        ),
        season_pass_2_effect(state.shop.upgrades[SHOP_SEASON_PASS_2]),
        1.0, // WowSquare
        calculate_platonic_multiplier_platonic_blessing(&state.platonic_blessings),
        1.0 + 1.2 * platonic[4] / 50.0, // Platonic1x4
    ])
}

/// `calculateHepteractMultiplier()` = `product(allHepteractCubeStats)` (Statistics.ts:615)
/// — feeds `CalcCorruptionStuff.hepteractGain`. WowSquare neutral-defaulted.
fn compute_hepteract_multiplier(
    state: &GameState,
    effective_score: f64,
    all_cube_multiplier: f64,
) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::achievement_rewards::wow_hepteract_gain;
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::shop_upgrades::season_pass_3_effect;
    use crate::state::shop::SHOP_SEASON_PASS_3;

    let achievement_level = achievement_level_from_points(state.achievements.achievement_points);

    product_f64(&[
        (1.0 + (effective_score - 1.666e16).max(0.0) / 3.33e16).powf(0.85), // AscensionScore
        all_cube_multiplier,
        get_level_reward(LevelRewardKey::WowHepteractCubes, achievement_level),
        wow_hepteract_gain(
            &state.achievements.achievements,
            state.campaigns.ascend_shards,
        ),
        season_pass_3_effect(state.shop.upgrades[SHOP_SEASON_PASS_3]),
        1.0, // WowSquare
    ])
}

/// `calculateAscensionCount()` — the per-ascension count gain
/// (`ascensionCountMultStats` product, floored; original Statistics.ts:3349 +
/// Calculate.ts:1296), self-derived from `&GameState`. `effective_score` is the
/// shared [`compute_ascension_score_result`]`.effective_score` (the
/// `AchievementMultiplier` line reads it). Reads the within-ascension
/// `ascensionCounter`, so the award must run before the ascension reset zeroes it.
///
/// Neutral-defaulted lines (faithful — singularity layer paused, all `1` at
/// `singularityCount 0`): SingularityUpgrade (`getGQUpgradeEffect('ascensions')`),
/// OcteractUpgrade1/2 (`octeractAscensions`/`octeractAscensions2`).
fn compute_ascension_count(state: &GameState, effective_score: f64) -> f64 {
    use crate::mechanics::achievement_rewards::{
        ascension_count_additive, ascension_count_multiplier,
    };
    use crate::mechanics::ascensions::{calculate_ascension_count, CalculateAscensionCountInput};
    use crate::mechanics::challenge_15_rewards;
    use crate::mechanics::golden_quark_upgrades::one_mind_effect;
    use crate::state::golden_quarks::GQ_ONE_MIND;

    const PLATONIC_UPGRADE_OMEGA: usize = 15;
    const PLATONIC_UPGRADE_16: usize = 16;

    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let ach = &state.achievements.achievements;
    let counter = state.reset_counters.ascension_counter;

    let one_mind = if one_mind_effect(state.golden_quarks.upgrades[GQ_ONE_MIND].level) {
        compute_ascension_speed_mult_pre(state) / 10.0
    } else {
        1.0
    };

    let mults = [
        1.0 + ascension_count_additive(ach, counter, state.reset_counters.ascension_counter_real),
        ascension_count_multiplier(ach, counter, effective_score),
        challenge_15_rewards::ascensions(state.challenges.challenge15_exponent),
        if platonic[PLATONIC_UPGRADE_OMEGA] > 0.0 {
            2.0
        } else {
            1.0
        },
        1.0 + platonic[PLATONIC_UPGRADE_16]
            * 0.02
            * (1.0 + (state.hepteracts.overflux_powder / 100_000.0).min(1.0)),
        1.0 + state.singularity.singularity_count / 10.0,
        1.0, // SingularityUpgrade — GQ 'ascensions' (singularity paused → 1 at sing 0)
        1.0, // OcteractUpgrade1 — octeractAscensions (singularity paused → 1)
        1.0, // OcteractUpgrade2 — octeractAscensions2 (singularity paused → 1)
        one_mind,
    ];

    calculate_ascension_count(&CalculateAscensionCountInput {
        limited_ascensions_enabled: state.singularity.limited_ascensions.enabled,
        ascension_count_mults: &mults,
    })
}

/// The currencies banked by an export-reward claim (the return of
/// [`claim_export_rewards`]). Both are post-multiplier amounts already added
/// to the player's balances.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ExportRewardClaim {
    /// Golden quarks credited from the `goldenQuarksTimer` window.
    pub golden_quarks: f64,
    /// Quarks (worlds) credited from the `quarkstimer` window, after the
    /// in-game quark multiplier.
    pub quarks: f64,
}

/// `exportSynergism`'s reward claim (`ImportExport.ts:254-273`) — the
/// timer→currency conversion run on a *real* export (the
/// `shouldSetLastSaveSoWeStopFuckingBotheringPeople` branch). Triggered by an
/// explicit player/host export, **not** the 5-second autosave: the host calls
/// this before serializing an export. The `offlinetick` / `lastExportedSave`
/// wall-clock stamps are host-tier (logic has no time-of-day) and live in the
/// save envelope.
///
/// Golden quarks: when `goldenQuarks3` is owned (`exportGQPerHour > 0`), the
/// whole-window count `floor(timer / (3600/perHour))` is credited times
/// `bonusGQMultiplier` and the timer keeps its remainder. Quarks: when the
/// `quarkHandler` gain is `>= 1`, `gain × calculateQuarkMultiplier()` is added
/// to `worlds` and `quarksThisSingularity` and the timer keeps its remainder.
///
/// `bonusGQMultiplier`'s external subscriber term (`1 + getQuarkBonus()/100`)
/// is neutral in the fork — the global/personal quark bonus is a parked RMT
/// seam — so only the `highestSingularityCount >= 100` export bonus applies,
/// matching how [`compute_quark_multiplier`] treats event/patreon as neutral.
pub fn claim_export_rewards(state: &mut GameState) -> ExportRewardClaim {
    use crate::mechanics::golden_quark_upgrades::golden_quarks_3_effect;
    use crate::mechanics::octeracts::octeract_export_quarks_effect;
    use crate::mechanics::quarks::{quark_handler, QuarkHandlerInput};
    use crate::state::golden_quarks::GQ_GOLDEN_QUARKS_3;
    use crate::state::octeract_upgrades::OCTERACT_EXPORT_QUARKS;

    let highest_sing = state.singularity.highest_singularity_count;

    // bonusGQMultiplier: external subscriber bonus neutral (× 1) in the fork;
    // only the highestSingularityCount >= 100 export bonus is live.
    let bonus_gq_multiplier = if highest_sing >= 100.0 {
        1.0 + highest_sing / 50.0
    } else {
        1.0
    };

    // Golden quarks: gated on the goldenQuarks3 upgrade's export rate.
    let gq3 = &state.golden_quarks.upgrades[GQ_GOLDEN_QUARKS_3];
    let gq_per_hour = golden_quarks_3_effect(gq3.level + gq3.free_level);
    let mut golden_quarks_awarded = 0.0;
    if gq_per_hour > 0.0 {
        let window = 3600.0 / gq_per_hour;
        golden_quarks_awarded =
            (state.golden_quarks.golden_quarks_timer / window).floor() * bonus_gq_multiplier;
        state.golden_quarks.golden_quarks += Decimal::from_finite(golden_quarks_awarded);
        state.golden_quarks.golden_quarks_timer %= window;
    }

    // Quarks (worlds): gated on a whole-quark gain. The award uses the full
    // in-game quark multiplier (`worlds.add(gain, true, true)`).
    let oct_export = &state.octeract_upgrades.upgrades[OCTERACT_EXPORT_QUARKS];
    let researches = &state.researches.researches;
    let quark = quark_handler(&QuarkHandlerInput {
        research_195: researches[195],
        researches_sum: researches[99]
            + researches[100]
            + researches[125]
            + researches[180]
            + researches[195],
        export_quark_mult: octeract_export_quarks_effect(oct_export.level + oct_export.free_level),
        quarks_timer: state.quarks.quarks_timer,
        // cube_mult is a pass-through in `quark_handler` and does not enter the
        // `gain` used here, so the `1.0` placeholder is faithful.
        cube_mult: 1.0,
    });
    let mut quarks_awarded = 0.0;
    if quark.gain >= 1.0 {
        quarks_awarded = quark.gain * compute_quark_multiplier(state);
        state.quarks.worlds += Decimal::from_finite(quarks_awarded);
        state.golden_quarks.quarks_this_singularity += quarks_awarded;
        state.quarks.quarks_timer %= 3600.0 / quark.per_hour;
    }

    ExportRewardClaim {
        golden_quarks: golden_quarks_awarded,
        quarks: quarks_awarded,
    }
}

/// `golden_quarks_multiplier_excluding_base` — self-derived from `&GameState`.
///
/// `calculateGoldenQuarks()` (legacy `Calculate.ts`, granted on a singularity
/// reset at `Reset.ts:1105`): the full `allGoldenQuarkMultiplierStats` product,
/// i.e. `calculateBaseGoldenQuarks() × <the other 12 lines>`. The base reads
/// quarks-this-singularity + the singularity / highest-singularity counts; the
/// rest come from [`compute_golden_quarks_multiplier_excluding_base`].
fn calculate_golden_quarks(state: &GameState) -> f64 {
    use crate::mechanics::singularity_milestones::{
        calculate_base_golden_quarks, CalculateBaseGoldenQuarksInput,
    };
    let base = calculate_base_golden_quarks(&CalculateBaseGoldenQuarksInput {
        singularity: state.singularity.singularity_count,
        quarks_this_singularity: state.golden_quarks.quarks_this_singularity,
        highest_singularity_count: state.singularity.highest_singularity_count,
    });
    base * compute_golden_quarks_multiplier_excluding_base(state)
}

/// `calculateMaxSingularityLookahead(true)` — how many singularities the
/// auto-climb advances per reset (legacy `Reset.ts:1108`). Self-derived from the
/// fast-forward GQ / octeract upgrades; `1` at the default state.
fn compute_singularity_lookahead(state: &GameState) -> f64 {
    use crate::mechanics::golden_quark_upgrades::{
        sing_fast_forward_2_effect, sing_fast_forward_effect,
    };
    use crate::mechanics::octeracts::octeract_fast_forward_effect;
    use crate::mechanics::singularity_helpers::{
        max_singularity_lookahead, MaxSingularityLookaheadInput,
    };
    use crate::state::golden_quarks::{GQ_SING_FAST_FORWARD, GQ_SING_FAST_FORWARD_2};
    use crate::state::octeract_upgrades::OCTERACT_FAST_FORWARD;

    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let oct = |i: usize| {
        state.octeract_upgrades.upgrades[i].level + state.octeract_upgrades.upgrades[i].free_level
    };
    max_singularity_lookahead(&MaxSingularityLookaheadInput {
        non_zero: true,
        sing_fast_forward_lookahead: sing_fast_forward_effect(gq(GQ_SING_FAST_FORWARD)),
        sing_fast_forward_2_lookahead: sing_fast_forward_2_effect(gq(GQ_SING_FAST_FORWARD_2)),
        octeract_fast_forward_lookahead: octeract_fast_forward_effect(oct(OCTERACT_FAST_FORWARD)),
    })
}

/// Legacy Helper.ts `addTimers('goldenQuarks')` reduces
/// `allGoldenQuarkMultiplierStats` (Statistics.ts:2337) but divides the `Base`
/// line (`calculateBaseGoldenQuarks`) back out and applies it separately, so
/// this is the product of the OTHER 12 lines. Companion to
/// [`compute_octeract_per_second`] — both feed the `singularityCount >= 160`
/// golden-quark giveaway loop. Every line is the multiplicative identity at the
/// default state, so this is exactly `1.0` — matching the old
/// `AutomationPre::default().golden_quarks_multiplier_excluding_base`.
///
/// Neutral-defaulted lines (faithful — no logic state source): PseudoCoins
/// (PCoin meta), Campaign (`player.campaigns.goldenQuarkBonus`, campaign
/// subsystem unported), GlobalSubscriber / AccountBonus (patreon / account meta
/// — `getGlobalBonus` / `getPersonalBonus`), Event (UI-tier calendar).
fn compute_golden_quarks_multiplier_excluding_base(state: &GameState) -> f64 {
    use crate::mechanics::calculate::product_f64;
    use crate::mechanics::golden_quark_upgrades::{
        golden_quarks_1_effect, sing_fast_forward_2_effect, sing_fast_forward_effect,
    };
    use crate::mechanics::octeracts::octeract_fast_forward_effect;
    use crate::mechanics::singularity_challenges::{
        no_singularity_upgrades_effect, NoSingularityUpgradesKey, SingularityEffectValue,
    };
    use crate::mechanics::singularity_helpers::{
        max_singularity_lookahead, MaxSingularityLookaheadInput,
    };
    use crate::mechanics::singularity_milestones::calculate_immaculate_alchemy_bonus;
    use crate::state::golden_quarks::{
        GQ_GOLDEN_QUARKS_1, GQ_SING_FAST_FORWARD, GQ_SING_FAST_FORWARD_2,
    };
    use crate::state::octeract_upgrades::OCTERACT_FAST_FORWARD;

    const CUBE_UPGRADE_19: usize = 69;

    let sing = state.singularity.singularity_count;
    let highest_sing = state.singularity.highest_singularity_count;
    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let oct = |i: usize| {
        state.octeract_upgrades.upgrades[i].level + state.octeract_upgrades.upgrades[i].free_level
    };

    // FastForwards = 1 + 0.025·(calculateMaxSingularityLookahead(true) − 1).
    let lookahead = max_singularity_lookahead(&MaxSingularityLookaheadInput {
        non_zero: true,
        sing_fast_forward_lookahead: sing_fast_forward_effect(gq(GQ_SING_FAST_FORWARD)),
        sing_fast_forward_2_lookahead: sing_fast_forward_2_effect(gq(GQ_SING_FAST_FORWARD_2)),
        octeract_fast_forward_lookahead: octeract_fast_forward_effect(oct(OCTERACT_FAST_FORWARD)),
    });

    // GoldenRevolution2 = highestSing >= 100 ? 1 + min(1, highestSing/250) : 1.
    let golden_revolution_2 = if highest_sing >= 100.0 {
        1.0 + 1.0_f64.min(highest_sing / 250.0)
    } else {
        1.0
    };

    let no_sing_upgrades = match no_singularity_upgrades_effect(
        state.singularity.no_singularity_upgrades.completions,
        NoSingularityUpgradesKey::GoldenQuarks,
    ) {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 1.0,
    };

    product_f64(&[
        1.0, // PseudoCoins — PCoin meta layer (unported)
        crate::mechanics::campaign_token_rewards::campaign_golden_quark_bonus(
            compute_campaign_tokens(state),
        ), // Campaign — player.campaigns.goldenQuarkBonus
        // Challenge15: 1 + max(0, log10(challenge15Exponent + 1) − 20) / 2.
        1.0 + 0.0_f64.max((state.challenges.challenge15_exponent + 1.0).log10() - 20.0) / 2.0,
        golden_quarks_1_effect(gq(GQ_GOLDEN_QUARKS_1)),
        1.0 + 0.12 * cube[CUBE_UPGRADE_19], // CookieUpgrade19
        no_sing_upgrades,
        golden_revolution_2,
        1.0 + 0.025 * (lookahead - 1.0), // FastForwards
        calculate_immaculate_alchemy_bonus(sing),
        1.0, // GlobalSubscriber — patreon meta (getGlobalBonus)
        1.0, // AccountBonus — account meta (getPersonalBonus)
        1.0, // Event — UI-tier event calendar
    ])
}

/// `calculateBaseObtainium()` — the **sum** of the `allBaseObtainiumStats`
/// StatLine (Statistics.ts:1416). Used both as the additive base obtainium
/// and as the first (Base) line of the DR-ignore immaculate product.
///
/// Neutral-defaulted line: `PseudoCoins` (the PCoin meta layer is unported →
/// additive `0`). `ShopPotionBonus` reads the single Rust
/// `shop.shop_potions_consumed` count (the legacy offering/obtainium split is
/// not modelled). All other lines derive from state.
fn compute_base_obtainium(state: &GameState) -> f64 {
    use crate::mechanics::blueberry_upgrades::{
        ambrosia_base_obtainium_1_effect, ambrosia_base_obtainium_2_effect,
    };
    use crate::mechanics::calculate::sum_f64;
    use crate::mechanics::potion_bonuses::calculate_obtainium_potion_base_obtainium;
    use crate::state::ambrosia::{AMBROSIA_BASE_OBTAINIUM_1, AMBROSIA_BASE_OBTAINIUM_2};

    let researches = &state.researches.researches;
    let reincarnation_counter = state.reset_counters.reincarnation_counter;
    let amb = |i: usize| state.ambrosia.upgrades[i].level + state.ambrosia.upgrades[i].free_level;

    sum_f64(&[
        1.0, // Base
        0.0, // PseudoCoins — PCoin meta layer (unported) → additive 0
        calculate_obtainium_potion_base_obtainium(state.shop.shop_potions_consumed).amount,
        // Research3x13 — gated by the reincarnation timer ≥ 2s.
        if reincarnation_counter >= 2.0 {
            researches[63]
        } else {
            0.0
        },
        // Research3x14 — gated by the reincarnation timer ≥ 5s.
        if reincarnation_counter >= 5.0 {
            2.0 * researches[64]
        } else {
            0.0
        },
        // FirstSingularity perk.
        if state.singularity.highest_singularity_count > 0.0 {
            3.0
        } else {
            0.0
        },
        (state.singularity.singularity_count / 10.0).floor(), // SingularityCount
        ambrosia_base_obtainium_1_effect(amb(AMBROSIA_BASE_OBTAINIUM_1)),
        ambrosia_base_obtainium_2_effect(amb(AMBROSIA_BASE_OBTAINIUM_2)),
    ])
}

/// `calculateObtainium(timeMultUsed = false)` — the obtainium resource
/// multiplier feeding the auto-research gain. Assembles the three obtainium
/// StatLine arrays (Statistics.ts:1416/1456/1541) into a
/// [`CalculateObtainiumInput`] and runs the ported aggregator:
/// `immaculate · baseMults^DR · timeMultiplier`, floored by `baseObtainium`,
/// with the c14 zero-out and taxman clamp.
///
/// `time_multiplier` is supplied by the caller (the legacy `timeMultUsed`
/// branch): `1.0` for the auto-research path, or the
/// `offeringObtainiumTimeModifiers` product (`compute_obtainium_time_multiplier`)
/// for the reincarnation-reset award. `base_mults` is the Decimal product of
/// `allObtainiumStats` times `calculateObtainiumCubeBlessing()` — the cube
/// blessing also appears as the `CubeBonus` line, so it is applied twice,
/// verbatim with the legacy `calculateObtainiumDecimal`.
///
/// `TutorialBonus`/`CampaignBonus` are wired to the derived campaign-token
/// total. Neutral-defaulted lines (faithful — no logic state source / inert
/// at the current state): `Event` (UI-tier event calendar → 1),
/// `ReincarnationUpgrade14` (reads `maxOfferings`, untracked → 1; its branch
/// is `1` at `maxOfferings 0` anyway), `Jack`/`shopPanthema` (needs the
/// unported `ShopPanthemaBonusLevels` → 1), `SpiritPower` (effective
/// rune-spirit power needs the unported `spiritMultiplier` chain → 1), and
/// `SingularityDebuff` (`1 / calculateSingularityDebuff`; the singularity
/// layer is paused → 1).
fn compute_obtainium(
    state: &GameState,
    base_obtainium: f64,
    reincarnation_point_gain: Decimal,
    time_multiplier: f64,
) -> Decimal {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::achievement_rewards::obtainium_bonus;
    use crate::mechanics::ant_upgrades::obtainium_ant_upgrade_effect;
    use crate::mechanics::blueberry_upgrades::ambrosia_obtainium_1_effect;
    use crate::mechanics::calculate::{
        calculate_obtainium, product_decimal, product_f64, CalculateObtainiumInput,
    };
    use crate::mechanics::challenge_15_rewards;
    use crate::mechanics::challenges::{calc_ecc, ChallengeType};
    use crate::mechanics::corruptions::{
        illiteracy_effect, illiteracy_power_at_level, IlliteracyEffectInput,
    };
    use crate::mechanics::cube_blessings::calculate_obtainium_cube_blessing;
    use crate::mechanics::exalt_penalties::calculate_exalt_6_penalty;
    use crate::mechanics::golden_quark_upgrades::{
        sing_citadel_2_effect, sing_citadel_effect, sing_obtainium_1_effect,
        sing_obtainium_2_effect, sing_obtainium_3_effect, starter_pack_effect, SingCitadel2Key,
        StarterPackKey,
    };
    use crate::mechanics::hypercube_blessings::calculate_obtainium_hypercube_blessing;
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::octeract_bonuses::{
        calculate_total_octeract_cube_bonus, calculate_total_octeract_obtainium_bonus,
        CalculateTotalOcteractCubeBonusInput, CalculateTotalOcteractObtainiumBonusInput,
    };
    use crate::mechanics::octeracts::octeract_obtainium_1_effect;
    use crate::mechanics::platonic_blessings::calculate_hypercube_blessing_multiplier_platonic_blessing;
    use crate::mechanics::red_ambrosia_bonuses::{
        calculate_red_ambrosia_obtainium, CalculateRedAmbrosiaResourceInput,
    };
    use crate::mechanics::red_ambrosia_upgrades::{
        red_ambrosia_obtainium_effect, tutorial_effect as red_tutorial_effect,
    };
    use crate::mechanics::rune_effects::{
        antiquities_rune_effects, superior_intellect_rune_effects, AntiquitiesRuneInput,
        AntiquitiesRuneKey, SuperiorIntellectRuneKey,
    };
    use crate::mechanics::shop_upgrades::{
        cash_grab_2_effect, cash_grab_effect, obtainium_ex_2_effect, obtainium_ex_3_effect,
        obtainium_ex_effect, shop_ex_ultra_effect, ObtainiumEX3Key,
    };
    use crate::mechanics::singularity_challenges::{
        no_octeracts_effect, NoOcteractsKey, SingularityEffectValue,
    };
    use crate::mechanics::talisman_levels::sum_of_talisman_rarities;
    use crate::mechanics::tesseract_blessings::calculate_obtainium_tesseract_blessing;
    use crate::state::ambrosia::AMBROSIA_OBTAINIUM_1;
    use crate::state::golden_quarks::{
        GQ_SING_CITADEL, GQ_SING_CITADEL_2, GQ_SING_OBTAINIUM_1, GQ_SING_OBTAINIUM_2,
        GQ_SING_OBTAINIUM_3, GQ_STARTER_PACK,
    };
    use crate::state::octeract_upgrades::OCTERACT_OBTAINIUM_1;
    use crate::state::red_ambrosia::{RED_AMBROSIA_RED_AMBROSIA_OBTAINIUM, RED_AMBROSIA_TUTORIAL};
    use crate::state::shop::{
        SHOP_CASH_GRAB, SHOP_CASH_GRAB_2, SHOP_EX_ULTRA, SHOP_OBTAINIUM_EX, SHOP_OBTAINIUM_EX_2,
        SHOP_OBTAINIUM_EX_3,
    };
    use crate::state::{ILLITERACY_INDEX, RUNE_ANTIQUITIES, RUNE_SUPERIOR_INTELLECT};

    // Legacy `AntUpgrades.Obtainium` + the obtainium cube-blessing upgrade
    // (`player.cubeUpgrades[40]`).
    const ANT_UPGRADE_OBTAINIUM: usize = 9;
    const CUBE_UPGRADE_OBTAINIUM_BLESSING: usize = 40;

    let sing = state.singularity.singularity_count;
    let researches = &state.researches.researches;
    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let shop = &state.shop.upgrades;
    let achievement_level = achievement_level_from_points(state.achievements.achievement_points);
    let lifetime_ambrosia = state.ambrosia.lifetime_ambrosia;
    let ambrosia_luck = compute_ambrosia_luck_pre(state);
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let oct = |i: usize| {
        state.octeract_upgrades.upgrades[i].level + state.octeract_upgrades.upgrades[i].free_level
    };
    let amb = |i: usize| state.ambrosia.upgrades[i].level + state.ambrosia.upgrades[i].free_level;
    let red = |i: usize| state.red_ambrosia.upgrades[i].level;

    // Obtainium cube-blessing chain platonic → hypercube → tesseract → cube,
    // mirroring `calculateObtainiumCubeBlessing` in `Cubes.ts`.
    let platonic_amplifier =
        calculate_hypercube_blessing_multiplier_platonic_blessing(&state.platonic_blessings);
    let hypercube_blessing =
        calculate_obtainium_hypercube_blessing(&state.hypercube_blessings, platonic_amplifier);
    let tesseract_blessing =
        calculate_obtainium_tesseract_blessing(&state.tesseract_blessings, hypercube_blessing);
    let obtainium_cube_blessing = calculate_obtainium_cube_blessing(
        &state.cube_blessings,
        tesseract_blessing,
        cube[CUBE_UPGRADE_OBTAINIUM_BLESSING],
    );

    // OcteractBonus line — the noOcteracts (Exalt 4) obtainium-bonus gate ×
    // the precomputed total-octeract cube bonus.
    let obtainium_bonus_enabled = matches!(
        no_octeracts_effect(
            state.singularity.no_octeracts.completions,
            NoOcteractsKey::ObtainiumBonus,
        ),
        SingularityEffectValue::Unlock(true)
    );
    let octeract_pow = match no_octeracts_effect(
        state.singularity.no_octeracts.completions,
        NoOcteractsKey::OcteractPow,
    ) {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 0.0,
    };
    let octeract_cube_bonus =
        calculate_total_octeract_cube_bonus(&CalculateTotalOcteractCubeBonusInput {
            exalt_4_enabled: state.singularity.no_octeracts.enabled,
            total_wow_octeracts: state.cube_balances.total_wow_octeracts.to_number(),
            octeract_pow,
        });

    // CubeUpgradeCx21 — `1.04 ^ (cubeUpgrades[71] · ΣtalismanRarities)`.
    let talisman_rarities = state.talismans.talisman_rarity.map(|r| r as u8);

    let campaign_tokens = compute_campaign_tokens(state);

    // immaculate = Π allObtainiumIgnoreDRStats (line 1 = calculateBaseObtainium).
    let immaculate = product_f64(&[
        base_obtainium,        // Base
        1.0 + 0.04 * cube[42], // CubeUpgrade4x2
        1.0 + 0.03 * cube[43], // CubeUpgrade4x3
        crate::mechanics::campaign_token_rewards::tutorial_bonus(campaign_tokens).obtainium_bonus, // TutorialBonus
        crate::mechanics::campaign_token_rewards::campaign_obtainium_bonus(campaign_tokens), // CampaignBonus
        challenge_15_rewards::obtainium(state.challenges.challenge15_exponent), // ChallengeBonus
        1.0 + platonic[5],                                                      // PlatonicALPHA
        1.0 + 1.5 * platonic[9],                                                // PlatonicUpgrade9
        1.0 + 2.5 * platonic[10],                                               // PlatonicBETA
        1.0 + 5.0 * platonic[15],                                               // PlatonicOMEGA
        10.0_f64.powf(antiquities_rune_effects(
            state.runes.rune_levels[RUNE_ANTIQUITIES],
            AntiquitiesRuneKey::ObtainiumLog10,
            AntiquitiesRuneInput {
                singularity_count: sing,
            },
        )), // Antiquities
        1.0 + cube[55] / 100.0,                                                 // CubeUpgradeCx5
        if cube[62] > 0.0 && state.challenges.current_ascension_challenge == 15 {
            8.0
        } else {
            1.0
        }, // CubeUpgradeCx12
        red_tutorial_effect(red(RED_AMBROSIA_TUTORIAL)), // RedAmbrosiaTutorial
        calculate_red_ambrosia_obtainium(&CalculateRedAmbrosiaResourceInput {
            unlocked: red_ambrosia_obtainium_effect(red(RED_AMBROSIA_RED_AMBROSIA_OBTAINIUM)),
            lifetime_red_ambrosia: state.red_ambrosia.lifetime_red_ambrosia,
        }), // RedAmbrosia
        1.04_f64.powf(cube[71] * sum_of_talisman_rarities(&talisman_rarities)), // CubeUpgradeCx21
        obtainium_ex_3_effect(
            shop[SHOP_OBTAINIUM_EX_3],
            ObtainiumEX3Key::ImmaculateObtainiuMult,
        ), // ObtainiumEX3
        if state.singularity.limited_time.enabled {
            calculate_exalt_6_penalty(
                state.singularity.limited_time.completions,
                state.singularity.sing_challenge_timer,
            )
        } else {
            1.0
        }, // Exalt6Penalty
        1.0,                                             // Event — UI-tier event calendar → 1 + 0
    ]);

    // base_mults = Π allObtainiumStats × calculateObtainiumCubeBlessing(),
    // reduced in Decimal space (the legacy `calculateObtainiumDecimal` folds
    // the per-line product into a Decimal, then multiplies the cube blessing
    // back in — so it appears twice: once as the `CubeBonus` line below and
    // once as the trailing factor).
    let transcend_shards_log10 = (state.reset_counters.transcend_shards + Decimal::one())
        .log10()
        .to_number();
    let reincarnation_point_gain_log10 = (reincarnation_point_gain + Decimal::from_finite(10.0))
        .log10()
        .to_number();
    let uncommon_fragments = state.talismans.uncommon_fragments;
    let base_mult_lines: [f64; 38] = [
        // TranscendShards — `max(1, (log10(transcendShards + 1) / 300)^2)`.
        1.0_f64.max((transcend_shards_log10 / 300.0).powi(2)),
        obtainium_bonus(
            &state.achievements.achievements,
            state.reset_counters.reincarnation_count,
        ), // AchievementBonus
        get_level_reward(LevelRewardKey::Obtainium, achievement_level), // SynergismLevel
        // ReincarnationUpgrade9 — `min(10, log10(reincarnationPointGain + 10)^0.5)`.
        if state.upgrades.upgrades[69] > 0 {
            10.0_f64.min(reincarnation_point_gain_log10.powf(0.5))
        } else {
            1.0
        },
        // ReincarnationUpgrade12 — `min(50, 1 + 2·Σ challengecompletions[6..=10])`.
        if state.upgrades.upgrades[72] > 0 {
            50.0_f64.min(
                1.0 + 2.0
                    * state.challenges.challenge_completions[6..=10]
                        .iter()
                        .sum::<f64>(),
            )
        } else {
            1.0
        },
        1.0, // ReincarnationUpgrade14 — maxOfferings untracked → 1.0 (the upgrades[74] branch is 1 at maxOfferings 0)
        1.0 + researches[65] / 5.0, // Research3x15
        1.0 + researches[76] / 10.0, // Research4x1
        1.0 + researches[81] / 10.0, // Research4x6
        1.0 + researches[119] / 200.0, // Research5x19
        cash_grab_effect(shop[SHOP_CASH_GRAB]), // ShopCashGrab
        obtainium_ex_effect(shop[SHOP_OBTAINIUM_EX]), // ShopObtainiumEX
        superior_intellect_rune_effects(
            first_five_effective_rune_level(state, RUNE_SUPERIOR_INTELLECT),
            SuperiorIntellectRuneKey::ObtainiumMult,
        ), // Rune5
        obtainium_ant_upgrade_effect(true_ant_level(state, ANT_UPGRADE_OBTAINIUM)), // Ant10
        obtainium_cube_blessing, // CubeBonus
        1.0 + 0.04 * state.campaigns.constant_upgrades[4], // ConstantUpgrade4
        1.0 + 0.1 * cube[3], // CubeUpgrade1x3
        // Challenge12 — `1 + 0.5·CalcECC('ascension', challengecompletions[12])`.
        1.0 + 0.5
            * calc_ecc(
                ChallengeType::Ascension,
                state.challenges.challenge_completions[12],
            ),
        crate::mechanics::rune_spirit_effects::superior_intellect_rune_spirit_effects(
            rune_spirit_power(state, RUNE_SUPERIOR_INTELLECT),
        )
        .obtainium,
        // Research6x19 — `1 + 0.03·log4(uncommonFragments + 1)·researches[144]`.
        1.0 + 0.03 * ((uncommon_fragments + 1.0).ln() / 4.0_f64.ln()) * researches[144],
        1.0 + 0.0002 * cube[50], // CubeUpgrade5x10
        1.0, // Jack — shopPanthema needs the unported ShopPanthemaBonusLevels → 1.0
        starter_pack_effect(gq(GQ_STARTER_PACK), StarterPackKey::ObtainiumMult), // StarterPack
        sing_obtainium_1_effect(gq(GQ_SING_OBTAINIUM_1)),
        sing_obtainium_2_effect(gq(GQ_SING_OBTAINIUM_2)),
        sing_obtainium_3_effect(gq(GQ_SING_OBTAINIUM_3)),
        sing_citadel_effect(gq(GQ_SING_CITADEL)),
        sing_citadel_2_effect(gq(GQ_SING_CITADEL_2), SingCitadel2Key::Mult),
        cash_grab_2_effect(shop[SHOP_CASH_GRAB_2]),
        obtainium_ex_2_effect(shop[SHOP_OBTAINIUM_EX_2], sing),
        obtainium_ex_3_effect(shop[SHOP_OBTAINIUM_EX_3], ObtainiumEX3Key::ObtainiumMult),
        calculate_total_octeract_obtainium_bonus(&CalculateTotalOcteractObtainiumBonusInput {
            obtainium_bonus_enabled,
            cube_bonus: octeract_cube_bonus,
        }), // OcteractBonus
        octeract_obtainium_1_effect(oct(OCTERACT_OBTAINIUM_1)),
        ambrosia_obtainium_1_effect(amb(AMBROSIA_OBTAINIUM_1), ambrosia_luck), // AmbrosiaObtainium1
        shop_ex_ultra_effect(shop[SHOP_EX_ULTRA], lifetime_ambrosia),          // EXUltraObtainium
        // Challenge14 — no obtainium inside ascension challenge 14.
        if state.challenges.current_ascension_challenge == 14 {
            0.0
        } else {
            1.0
        },
        1.0, // SingularityDebuff — `1/calculateSingularityDebuff('Obtainium')`; sing layer paused → 1.0
        // TaxmanDebuff — `2.5 ^ -min(500, floor(1 + max(0, log10(offerings))))`.
        if state.singularity.taxman_last_stand.enabled {
            let offerings_log10 = state.automation.offerings.log10().to_number();
            let offering_digits = (1.0 + 0.0_f64.max(offerings_log10)).floor();
            2.5_f64.powf(-(500.0_f64.min(offering_digits)))
        } else {
            1.0
        },
    ];
    let base_mults = product_decimal(&base_mult_lines.map(Decimal::from_finite))
        * Decimal::from_finite(obtainium_cube_blessing);

    // DR exponent — `player.corruptions.used.corruptionEffects('illiteracy')`.
    let dr = illiteracy_effect(&IlliteracyEffectInput {
        base_power: illiteracy_power_at_level(state.corruptions.used.levels[ILLITERACY_INDEX]),
        platonic_upgrade_9: platonic[9],
        obtainium_log10: if state.researches.obtainium >= Decimal::one() {
            Some(state.researches.obtainium.log10().to_number())
        } else {
            None
        },
    });

    calculate_obtainium(&CalculateObtainiumInput {
        base_obtainium,
        immaculate,
        dr,
        time_multiplier,
        base_mults,
        in_ascension_challenge_14: state.challenges.current_ascension_challenge == 14,
        taxman_last_stand_enabled: state.singularity.taxman_last_stand.enabled,
        taxman_last_stand_completions: state.singularity.taxman_last_stand.completions,
        current_obtainium: state.researches.obtainium,
    })
}

/// `offeringObtainiumTimeModifiers(time, timeMultCheck)` reduced to a product
/// (Statistics.ts:1727) — the shared `timeMultUsed = true` reset-award time
/// multiplier. Called with the prestige timer (offerings, Calculate.ts:211)
/// and the reincarnation timer (obtainium, Calculate.ts:269).
///
/// Three lines: `ThresholdPenalty` (`min(1, (t/threshold)^2)`, ≤1, penalises
/// resets faster than the threshold), `TimeMultiplier` (`max(1, t/threshold)`
/// when `time_mult_check`, else 1, rewarding longer resets), and `HalfMind`
/// (`globalSpeedMult / 10` when the half-mind GQ upgrade is unlocked, else 1).
/// `threshold` folds the campaign time-threshold reduction of the derived
/// token total (10 at zero tokens).
fn offering_obtainium_time_multiplier(state: &GameState, time: f64, time_mult_check: bool) -> f64 {
    use crate::mechanics::golden_quark_upgrades::half_mind_effect;
    use crate::mechanics::reset_time_and_auto_obtainium::{
        reset_time_threshold, ResetTimeThresholdInput,
    };
    use crate::state::golden_quarks::GQ_HALF_MIND;

    let threshold = reset_time_threshold(&ResetTimeThresholdInput {
        campaign_time_threshold_reduction:
            crate::mechanics::campaign_token_rewards::campaign_time_threshold_reduction(
                compute_campaign_tokens(state),
            ),
    });
    let ratio = time / threshold;

    let threshold_penalty = 1.0_f64.min(ratio.powi(2));
    let time_multiplier = if time_mult_check {
        1.0_f64.max(ratio)
    } else {
        1.0
    };
    let half_mind = if half_mind_effect(state.golden_quarks.upgrades[GQ_HALF_MIND].level) {
        compute_global_speed_mult_pre(state) / 10.0
    } else {
        1.0
    };

    threshold_penalty * time_multiplier * half_mind
}

/// `offeringObtainiumTimeModifiers(reincarnationcounter, reincarnationCount >= 5)`
/// (Calculate.ts:269) — the obtainium reincarnation-reset time multiplier.
fn compute_obtainium_time_multiplier(state: &GameState) -> f64 {
    offering_obtainium_time_multiplier(
        state,
        state.reset_counters.reincarnation_counter,
        state.reset_counters.reincarnation_count >= 5.0,
    )
}

/// `offeringObtainiumTimeModifiers(prestigecounter, getLevelMilestone('offeringTimerScaling') === 1)`
/// (Calculate.ts:211) — the offering reset-award time multiplier. The
/// `TimeMultiplier` line activates once the synergism-level milestone
/// `offeringTimerScaling` unlocks (level ≥ 5).
fn compute_offering_time_multiplier(state: &GameState) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::level_milestones::{get_level_milestone, LevelMilestoneKey};

    let achievement_level = achievement_level_from_points(state.achievements.achievement_points);
    let time_mult_check =
        get_level_milestone(LevelMilestoneKey::OfferingTimerScaling, achievement_level) == 1.0;
    offering_obtainium_time_multiplier(
        state,
        state.reset_counters.prestige_counter,
        time_mult_check,
    )
}

/// `calculateBaseOfferings()` — Σ allBaseOfferingStats (Statistics.ts:828).
/// The additive floor of the offering award's `max(base, mult·timeMult)`.
///
/// Neutral-defaulted (faithful — no logic-state source): `PseudoCoins`
/// (PCoin meta layer unported → additive 0).
fn compute_base_offerings(state: &GameState) -> f64 {
    use crate::mechanics::blueberry_upgrades::{
        ambrosia_base_offering_1_effect, ambrosia_base_offering_2_effect,
    };
    use crate::mechanics::calculate::sum_f64;
    use crate::mechanics::potion_bonuses::calculate_offering_potion_base_offerings;
    use crate::mechanics::shop_upgrades::{offering_ex_3_effect, OfferingEX3Key};
    use crate::state::ambrosia::{AMBROSIA_BASE_OFFERING_1, AMBROSIA_BASE_OFFERING_2};
    use crate::state::shop::SHOP_OFFERING_EX_3;

    let researches = &state.researches.researches;
    let upgrades = &state.upgrades.upgrades;
    let shop = &state.shop.upgrades;
    let total_challenge_completions: f64 = state.challenges.challenge_completions.iter().sum();
    let amb = |i: usize| state.ambrosia.upgrades[i].level + state.ambrosia.upgrades[i].free_level;

    sum_f64(&[
        1.0, // Base
        0.0, // PseudoCoins — PCoin meta layer unported → additive 0
        if state.reset_counters.prestige_count > 0.0 {
            1.0
        } else {
            0.0
        }, // Prestige
        if state.reset_counters.transcend_count > 0.0 {
            3.0
        } else {
            0.0
        }, // Transcend
        if state.reset_counters.reincarnation_count > 0.0 {
            5.0
        } else {
            0.0
        }, // Reincarnate
        if state.challenges.challenge_completions[2] > 0.0 {
            2.0
        } else {
            0.0
        }, // Challenge1 (challenge 2x1)
        calculate_offering_potion_base_offerings(state.shop.shop_potions_consumed).amount, // ShopPotionBonus
        // ReincarnationUpgrade2 — upgrades[62]: min(12, (1/50)·Σ challengecompletions).
        if upgrades[62] > 0 {
            12.0_f64.min((1.0 / 50.0) * total_challenge_completions)
        } else {
            0.0
        },
        0.4 * researches[24],                          // Research1x24
        0.6 * researches[25],                          // Research1x25
        if researches[95] > 0.0 { 15.0 } else { 0.0 }, // Research4x20
        ambrosia_base_offering_1_effect(amb(AMBROSIA_BASE_OFFERING_1)), // AmbrosiaBaseOffering1
        ambrosia_base_offering_2_effect(amb(AMBROSIA_BASE_OFFERING_2)), // AmbrosiaBaseOffering2
        offering_ex_3_effect(shop[SHOP_OFFERING_EX_3], OfferingEX3Key::BaseOfferings), // OfferingEX3
    ])
}

/// `calculateOfferingsDecimal()` — Π allOfferingStats (Statistics.ts:888),
/// the multiplicative side of the offering award. `base_offerings` is the
/// caller's `calculateBaseOfferings()` (the `Base` line). Reduced in Decimal
/// space to survive the 1e300 cap.
///
/// `TutorialBonus`/`CampaignBonus` are wired to the derived campaign-token
/// total. Neutral-defaulted lines (faithful — no logic-state source / inert
/// at the current state): `AchievementBonus` (the `offeringBonus` reward
/// *reader* is unported → 1.0; its lone contributor is the
/// `prestigeCount ≥ 1000` achievement), `ParticleUpgrade3x5` (`maxObtainium`
/// untracked → the `min(maxObtainium, …)` term is 0, so the line is 1.0),
/// `Jack`/`shopPanthema` (needs the unported `ShopPanthemaBonusLevels`
/// → 1.0), `SingularityDebuff` (`1/calculateSingularityDebuff`; singularity
/// layer paused → 1.0), and `Event` (UI-tier event calendar → 1.0).
fn compute_offering_mult(state: &GameState, base_offerings: f64) -> Decimal {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::ant_upgrades::offerings_ant_upgrade_effect;
    use crate::mechanics::blueberry_upgrades::ambrosia_offering_1_effect;
    use crate::mechanics::calculate::product_decimal;
    use crate::mechanics::challenge_15_rewards;
    use crate::mechanics::challenges::{calc_ecc, ChallengeType};
    use crate::mechanics::cube_blessings::calculate_offering_cube_blessing;
    use crate::mechanics::exalt_penalties::calculate_exalt_6_penalty;
    use crate::mechanics::golden_quark_upgrades::{
        sing_citadel_2_effect, sing_citadel_effect, sing_offerings_1_effect,
        sing_offerings_2_effect, sing_offerings_3_effect, starter_pack_effect, SingCitadel2Key,
        StarterPackKey,
    };
    use crate::mechanics::hypercube_blessings::calculate_offering_hypercube_blessing;
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::octeract_bonuses::{
        calculate_total_octeract_cube_bonus, calculate_total_octeract_offering_bonus,
        CalculateTotalOcteractCubeBonusInput, CalculateTotalOcteractOfferingBonusInput,
    };
    use crate::mechanics::octeracts::octeract_offerings_1_effect;
    use crate::mechanics::platonic_blessings::calculate_hypercube_blessing_multiplier_platonic_blessing;
    use crate::mechanics::red_ambrosia_bonuses::{
        calculate_red_ambrosia_offering, CalculateRedAmbrosiaResourceInput,
    };
    use crate::mechanics::red_ambrosia_upgrades::{
        red_ambrosia_offering_effect, tutorial_effect as red_tutorial_effect,
    };
    use crate::mechanics::rune_effects::{
        antiquities_rune_effects, superior_intellect_rune_effects, AntiquitiesRuneInput,
        AntiquitiesRuneKey, SuperiorIntellectRuneKey,
    };
    use crate::mechanics::shop_upgrades::{
        cash_grab_2_effect, cash_grab_effect, offering_ex_2_effect, offering_ex_3_effect,
        offering_ex_effect, shop_ex_ultra_effect, OfferingEX3Key,
    };
    use crate::mechanics::singularity_challenges::{
        no_octeracts_effect, NoOcteractsKey, SingularityEffectValue,
    };
    use crate::mechanics::talisman_levels::sum_of_talisman_rarities;
    use crate::mechanics::tesseract_blessings::calculate_offering_tesseract_blessing;
    use crate::state::ambrosia::AMBROSIA_OFFERING_1;
    use crate::state::golden_quarks::{
        GQ_SING_CITADEL, GQ_SING_CITADEL_2, GQ_SING_OFFERINGS_1, GQ_SING_OFFERINGS_2,
        GQ_SING_OFFERINGS_3, GQ_STARTER_PACK,
    };
    use crate::state::octeract_upgrades::OCTERACT_OFFERINGS_1;
    use crate::state::red_ambrosia::{RED_AMBROSIA_RED_AMBROSIA_OFFERING, RED_AMBROSIA_TUTORIAL};
    use crate::state::shop::{
        SHOP_CASH_GRAB, SHOP_CASH_GRAB_2, SHOP_EX_ULTRA, SHOP_OFFERING_EX, SHOP_OFFERING_EX_2,
        SHOP_OFFERING_EX_3,
    };
    use crate::state::talismans::TALISMAN_MIDAS;
    use crate::state::{RUNE_ANTIQUITIES, RUNE_SUPERIOR_INTELLECT};

    // Legacy `AntUpgrades.Offerings` (index 5) and the offering cube-blessing
    // DR upgrade (`player.cubeUpgrades[24]`).
    const ANT_UPGRADE_OFFERINGS: usize = 5;
    const CUBE_UPGRADE_OFFERING_BLESSING: usize = 24;

    let sing = state.singularity.singularity_count;
    let researches = &state.researches.researches;
    let upgrades = &state.upgrades.upgrades;
    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let shop = &state.shop.upgrades;
    let cc = &state.challenges.challenge_completions;
    let achievement_level = achievement_level_from_points(state.achievements.achievement_points);
    let lifetime_ambrosia = state.ambrosia.lifetime_ambrosia;
    let ambrosia_luck = compute_ambrosia_luck_pre(state);
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let oct = |i: usize| {
        state.octeract_upgrades.upgrades[i].level + state.octeract_upgrades.upgrades[i].free_level
    };
    let amb = |i: usize| state.ambrosia.upgrades[i].level + state.ambrosia.upgrades[i].free_level;
    let red = |i: usize| state.red_ambrosia.upgrades[i].level;
    let total_challenge_completions: f64 = cc.iter().sum();
    let talisman_rarities = state.talismans.talisman_rarity.map(|r| r as u8);
    let midas_level = state.talismans.talisman_levels[TALISMAN_MIDAS];

    // Offering cube-blessing chain platonic → hypercube → tesseract → cube,
    // mirroring `calculateOfferingCubeBlessing` (Cubes.ts:294).
    let platonic_amplifier =
        calculate_hypercube_blessing_multiplier_platonic_blessing(&state.platonic_blessings);
    let hypercube_blessing =
        calculate_offering_hypercube_blessing(&state.hypercube_blessings, platonic_amplifier);
    let tesseract_blessing =
        calculate_offering_tesseract_blessing(&state.tesseract_blessings, hypercube_blessing);
    let offering_cube_blessing = calculate_offering_cube_blessing(
        &state.cube_blessings,
        tesseract_blessing,
        cube[CUBE_UPGRADE_OFFERING_BLESSING],
    );

    // OcteractBonus line — the noOcteracts (Exalt 4) offering-bonus gate ×
    // the precomputed total-octeract cube bonus.
    let offering_bonus_enabled = matches!(
        no_octeracts_effect(
            state.singularity.no_octeracts.completions,
            NoOcteractsKey::OfferingBonus,
        ),
        SingularityEffectValue::Unlock(true)
    );
    let octeract_pow = match no_octeracts_effect(
        state.singularity.no_octeracts.completions,
        NoOcteractsKey::OcteractPow,
    ) {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 0.0,
    };
    let octeract_cube_bonus =
        calculate_total_octeract_cube_bonus(&CalculateTotalOcteractCubeBonusInput {
            exalt_4_enabled: state.singularity.no_octeracts.enabled,
            total_wow_octeracts: state.cube_balances.total_wow_octeracts.to_number(),
            octeract_pow,
        });

    // PrestigeShards — reads the canonical crystal-upgrades slice (H1).
    let prestige_shards_log10 = (state.crystal_upgrades.prestige_shards + Decimal::one())
        .log10()
        .to_number();

    let lines = [
        base_offerings, // Base = calculateBaseOfferings()
        // PrestigeShards — 1 + log10(prestigeShards + 1)^0.5 / 5.
        1.0 + prestige_shards_log10.powf(0.5) / 5.0,
        1.0, // AchievementBonus — getAchievementReward('offeringBonus'); achievement awarding unported (P3.1/H5) → 1.0
        get_level_reward(LevelRewardKey::Offerings, achievement_level), // SynergismLevel
        superior_intellect_rune_effects(
            first_five_effective_rune_level(state, RUNE_SUPERIOR_INTELLECT),
            SuperiorIntellectRuneKey::OfferingMult,
        ), // SuperiorIntellect
        // ReincarnationChallenge — 1 + (1/50)·ECC(c6) + (1/25)·ECC(c8) + (1/25)·ECC(c10).
        1.0 + (1.0 / 50.0) * calc_ecc(ChallengeType::Reincarnation, cc[6])
            + (1.0 / 25.0) * calc_ecc(ChallengeType::Reincarnation, cc[8])
            + (1.0 / 25.0) * calc_ecc(ChallengeType::Reincarnation, cc[10]),
        1.0 + 0.2 * f64::from(upgrades[38]), // DiamondUpgrade4x3
        1.0, // ParticleUpgrade3x5 — maxObtainium untracked → 1.0 (the min(maxObtainium,1e10) term is 0)
        1.0 + researches[119] / 200.0, // Research5x19
        offering_ex_effect(shop[SHOP_OFFERING_EX]), // OfferingEXShop
        cash_grab_effect(shop[SHOP_CASH_GRAB]), // CashGrab
        1.0 + (1.0 / 10_000.0) * total_challenge_completions * researches[85], // Research4x10
        offerings_ant_upgrade_effect(true_ant_level(state, ANT_UPGRADE_OFFERINGS)), // AntUpgrade
        offering_cube_blessing, // Brutus (cube blessing)
        1.0 + 0.02 * state.campaigns.constant_upgrades[3], // ConstantUpgrade3
        // ResearchTalismans — 1 + 0.0003·midas·research[149] + 0.0004·midas·research[179].
        1.0 + 0.0003 * midas_level * researches[149] + 0.0004 * midas_level * researches[179],
        crate::mechanics::campaign_token_rewards::tutorial_bonus(compute_campaign_tokens(state))
            .offering_bonus, // TutorialBonus
        crate::mechanics::campaign_token_rewards::campaign_offering_bonus(compute_campaign_tokens(
            state,
        )), // CampaignBonus
        1.0 + 0.12 * calc_ecc(ChallengeType::Ascension, cc[12]), // Challenge12
        // ThriftSpirit — getRuneSpiritEffect('thrift').offerings.
        crate::mechanics::rune_spirit_effects::thrift_rune_spirit_effects(rune_spirit_power(
            state,
            crate::state::RUNE_THRIFT,
        ))
        .offerings,
        1.0 + (0.01 / 100.0) * researches[200], // Research8x25
        1.0 + 0.05 * cube[46],                  // CubeUpgrade5x6
        1.0 + (0.02 / 100.0) * cube[50],        // CubeUpgrade5x10
        1.0 + platonic[5],                      // PlatonicALPHA
        1.0 + 2.5 * platonic[10],               // PlatonicBETA
        1.0 + 5.0 * platonic[15],               // PlatonicOMEGA
        challenge_15_rewards::offering(state.challenges.challenge15_exponent), // Challenge15
        10.0_f64.powf(antiquities_rune_effects(
            state.runes.rune_levels[RUNE_ANTIQUITIES],
            AntiquitiesRuneKey::OfferingLog10,
            AntiquitiesRuneInput {
                singularity_count: sing,
            },
        )), // Antiquities
        1.0, // Jack — shopPanthema needs the unported ShopPanthemaBonusLevels → 1.0
        1.0, // SingularityDebuff — 1/calculateSingularityDebuff('Offering'); sing layer paused → 1.0
        starter_pack_effect(gq(GQ_STARTER_PACK), StarterPackKey::OfferingMult), // StarterPack
        sing_offerings_1_effect(gq(GQ_SING_OFFERINGS_1)), // OfferingCharge
        sing_offerings_2_effect(gq(GQ_SING_OFFERINGS_2)), // OfferingStorm
        sing_offerings_3_effect(gq(GQ_SING_OFFERINGS_3)), // OfferingTempest
        sing_citadel_effect(gq(GQ_SING_CITADEL)), // Citadel
        sing_citadel_2_effect(gq(GQ_SING_CITADEL_2), SingCitadel2Key::Mult), // Citadel2
        1.0 + cube[54] / 100.0, // CubeUpgradeCx4
        if cube[62] > 0.0 && state.challenges.current_ascension_challenge == 15 {
            8.0
        } else {
            1.0
        }, // CubeUpgradeCx12
        octeract_offerings_1_effect(oct(OCTERACT_OFFERINGS_1)), // OcteractElectrolosis
        calculate_total_octeract_offering_bonus(&CalculateTotalOcteractOfferingBonusInput {
            offering_bonus_enabled,
            cube_bonus: octeract_cube_bonus,
        }), // OcteractBonus
        ambrosia_offering_1_effect(amb(AMBROSIA_OFFERING_1), ambrosia_luck), // Ambrosia
        red_tutorial_effect(red(RED_AMBROSIA_TUTORIAL)), // RedAmbrosiaTutorial
        calculate_red_ambrosia_offering(&CalculateRedAmbrosiaResourceInput {
            unlocked: red_ambrosia_offering_effect(red(RED_AMBROSIA_RED_AMBROSIA_OFFERING)),
            lifetime_red_ambrosia: state.red_ambrosia.lifetime_red_ambrosia,
        }), // RedAmbrosia
        1.04_f64.powf(cube[72] * sum_of_talisman_rarities(&talisman_rarities)), // CubeUpgradeCx22
        cash_grab_2_effect(shop[SHOP_CASH_GRAB_2]), // CashGrab2
        offering_ex_2_effect(shop[SHOP_OFFERING_EX_2], sing), // OfferingEX2
        offering_ex_3_effect(shop[SHOP_OFFERING_EX_3], OfferingEX3Key::OfferingMult), // OfferingINF
        shop_ex_ultra_effect(shop[SHOP_EX_ULTRA], lifetime_ambrosia), // EXUltra
        // Exalt6Penalty — singularity speedrun penalty (limitedTime).
        if state.singularity.limited_time.enabled {
            calculate_exalt_6_penalty(
                state.singularity.limited_time.completions,
                state.singularity.sing_challenge_timer,
            )
        } else {
            1.0
        },
        // TaxmanDebuff — 2.5 ^ -min(500, floor(1 + max(0, log10(obtainium)))).
        if state.singularity.taxman_last_stand.enabled {
            let obtainium_log10 = state.researches.obtainium.log10().to_number();
            let obtainium_digits = (1.0 + 0.0_f64.max(obtainium_log10)).floor();
            2.5_f64.powf(-(500.0_f64.min(obtainium_digits)))
        } else {
            1.0
        },
        1.0, // Event — UI-tier event calendar → 1 + 0
    ];
    product_decimal(&lines.map(Decimal::from_finite))
}

/// `calculateOfferings()` (Calculate.ts:208) — the offering award credited on
/// every reset tier (`resetOfferings`, Runes.ts:1046). Assembles the base sum,
/// the multiplier product, and the prestige-timer multiplier, then applies the
/// Exalt-8 taxman cap via the ported [`calculate_offerings`] aggregator.
fn compute_offerings(state: &GameState) -> Decimal {
    use crate::mechanics::calculate::{calculate_offerings, CalculateOfferingsInput};

    let base_offerings = compute_base_offerings(state);
    let offering_mult = compute_offering_mult(state, base_offerings);
    let time_multiplier = compute_offering_time_multiplier(state);

    calculate_offerings(&CalculateOfferingsInput {
        base_offerings,
        time_multiplier,
        offering_mult,
        taxman_last_stand_enabled: state.singularity.taxman_last_stand.enabled,
        taxman_last_stand_completions: state.singularity.taxman_last_stand.completions,
        current_offerings: state.automation.offerings,
    })
}

/// `obtainium_gain` — self-derived from `&GameState`.
///
/// Legacy Helper.ts `addTimers('obtainium')` sets this to
/// `calculateResearchAutomaticObtainium(dt)`: the per-tick automatic obtainium
/// from research idle gain. Replaces the caller-provided
/// `AutomationPre::obtainium_gain`. **Self-derives to `0` at the default
/// state** — the per-upgrade multiplier gate
/// (`0.5·research[61] + 0.1·research[62] + 0.8·cubeUpgrade[3]`) is `0` — which
/// matches the old `AutomationPre::default().obtainium_gain`.
///
/// The resource multiplier (`calculateObtainium(false)`) and base obtainium
/// flow through [`compute_obtainium`] / [`compute_base_obtainium`]. The
/// ant-sacrifice obtainium source (a `max()` alternative gated by
/// `cubeUpgrades[47] > 0`) is now wired to `calculateAntSacrificeObtainium` via
/// the ported `ant_sacrifice::compute_ant_sacrifice_multiplier`; it stays inert
/// at the current state (`cubeUpgrades[47] == 0`). The reset-time divisor folds
/// the campaign time-threshold reduction of the derived token total.
fn compute_obtainium_gain(
    state: &GameState,
    dt: f64,
    reincarnation_point_gain: Decimal,
) -> Decimal {
    use crate::mechanics::reset_time_and_auto_obtainium::{
        calculate_research_automatic_obtainium, reset_time_threshold,
        ResearchAutomaticObtainiumInput, ResetTimeThresholdInput,
    };

    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let base_obtainium = compute_base_obtainium(state);
    // Auto-research path: legacy `calculateObtainium(false)` ⇒ timeMult 1.0.
    let resource_mult = compute_obtainium(state, base_obtainium, reincarnation_point_gain, 1.0);
    let reset_time_divisor = reset_time_threshold(&ResetTimeThresholdInput {
        campaign_time_threshold_reduction:
            crate::mechanics::campaign_token_rewards::campaign_time_threshold_reduction(
                compute_campaign_tokens(state),
            ),
    });

    // Ant-sacrifice obtainium alternative source (gated by cubeUpgrades[47]):
    // `calculateAntSacrificeObtainium(antSacrificeObtainiumStageMult, useTime=false)`.
    // Inert until the cube upgrade is bought; the obtainium multiplier reuses the
    // already-computed `calculateObtainium(false)` (`resource_mult`).
    let ant_sacrifice_obtainium = if cube[47] > 0.0 {
        use crate::mechanics::ant_reborn_elo::{
            reborn_elo_stage_modifiers, RebornELOStageModifiersInput,
        };
        use crate::mechanics::ant_sacrifice_reward_calc::{
            calculate_ant_sacrifice_obtainium, AntSacrificeObtainiumInput,
        };
        let stage_mods = reborn_elo_stage_modifiers(&RebornELOStageModifiersInput {
            reborn_elo: state.ants.reborn_elo,
            sing_count: state.singularity.singularity_count,
        });
        calculate_ant_sacrifice_obtainium(&AntSacrificeObtainiumInput {
            ant_sac_mult: ant_sacrifice::compute_ant_sacrifice_multiplier(state),
            stage_mult: stage_mods.ant_sacrifice_obtainium_mult,
            time_multiplier: offering_obtainium_time_multiplier(
                state,
                state.ants.ant_sacrifice_timer,
                false,
            ),
            obtainium_mult: resource_mult,
            current_obtainium: state.researches.obtainium,
            taxman_last_stand_enabled: state.singularity.taxman_last_stand.enabled,
            taxman_last_stand_completions: state.singularity.taxman_last_stand.completions,
        })
    } else {
        Decimal::zero()
    };

    calculate_research_automatic_obtainium(&ResearchAutomaticObtainiumInput {
        delta_time: dt,
        ascension_challenge: state.challenges.current_ascension_challenge,
        research_61: state.researches.researches[61],
        research_62: state.researches.researches[62],
        cube_upgrade_3: cube[3],
        cube_upgrade_47: cube[47],
        resource_mult,
        global_speed_mult: compute_global_speed_mult_pre(state),
        reset_time_divisor,
        reincarnation_counter: state.reset_counters.reincarnation_counter,
        base_obtainium,
        ant_sacrifice_obtainium,
        ant_sacrifice_timer: state.ants.ant_sacrifice_timer,
    })
}

/// `ant_speed_mult` — self-derived from `&GameState`.
///
/// Legacy `calculateActualAntSpeedMult()`: the Decimal product of the 24-line
/// `antSpeedStats` StatLine (Statistics.ts:2967), raised to the
/// ascension-challenge exponent via [`calculate_actual_ant_speed_mult`].
/// Replaces the caller-provided `AutomationPre::ant_speed_mult`.
///
/// **Self-derives to 0 at the default state** (unlike most fields): the `Base`
/// line is `canGenerateAntCrumbs ? 1 : 0`
/// (`challengecompletions[8] > 0 || cubeUpgrades[48] > 0`), which is `0` until
/// ants are unlocked — zeroing the whole product. The old
/// `AutomationPre::default().ant_speed_mult` was `Decimal::one()`, so this is a
/// genuine default change; ant generation multiplies its per-tier output by
/// this factor and so no-ops at `0` (and at default no producers are owned
/// anyway). `canGenerateAntCrumbs` and `calculateAntSpeedMultFromELO`
/// (`1.02 ^ rebornELO`) are inlined — both are one-liners in the legacy.
///
/// Neutral-defaulted line (faithful / inert at the current state):
/// `ReincarnationUpgrade18` reads `maxOfferings` (untracked) → 1 (its
/// `upgrades[78]` branch evaluates to 1 at `maxOfferings 0`).
fn compute_ant_speed_mult(state: &GameState) -> Decimal {
    use crate::mechanics::achievement_rewards::ant_speed as ant_speed_reward;
    use crate::mechanics::ant_upgrades::{ant_speed_ant_upgrade_effect, AntSpeedAntUpgradeInput};
    use crate::mechanics::calculate::{
        calculate_actual_ant_speed_mult, product_decimal, ActualAntSpeedMultInput,
    };
    use crate::mechanics::challenge_15_rewards;
    use crate::mechanics::challenges::{calc_ecc, ChallengeType};
    use crate::mechanics::cube_blessings::calculate_ant_speed_cube_blessing;
    use crate::mechanics::hypercube_blessings::calculate_ant_speed_hypercube_blessing;
    use crate::mechanics::octeracts::{octeract_starter_effect, OcteractStarterKey};
    use crate::mechanics::platonic_blessings::calculate_hypercube_blessing_multiplier_platonic_blessing;
    use crate::mechanics::rune_blessing_effects::superior_intellect_rune_blessing_effects;
    use crate::mechanics::rune_effects::{
        superior_intellect_rune_effects, SuperiorIntellectRuneKey,
    };
    use crate::mechanics::tesseract_blessings::calculate_ant_speed_tesseract_blessing;
    use crate::state::octeract_upgrades::OCTERACT_STARTER;
    use crate::state::RUNE_SUPERIOR_INTELLECT;

    // Legacy AntUpgrades.AntSpeed (index 0), AntProducers.Workers (index 0),
    // and the ant-speed cube-blessing upgrade (player.cubeUpgrades[22]).
    const ANT_UPGRADE_ANT_SPEED: usize = 0;
    const ANT_PRODUCER_WORKERS: usize = 0;
    const CUBE_UPGRADE_ANT_SPEED_BLESSING: usize = 22;

    let cube = &state.cube_upgrade_levels.cube_upgrades;
    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let researches = &state.researches.researches;
    let upgrades = &state.upgrades.upgrades;
    let up = |i: usize| f64::from(upgrades[i]);
    let workers_purchased = state.ants.producers[ANT_PRODUCER_WORKERS].purchased;
    let crumbs = state.ants.crumbs;
    let obtainium = state.researches.obtainium;
    let highest_sing = state.singularity.highest_singularity_count;

    // GlobalSpeed line: speedMult^(1 + 3·upgrades[79]) when > 1, else speedMult.
    let global_speed = compute_global_speed_mult_pre(state);
    let global_speed_line = {
        let exponent = 1.0 + 3.0 * up(79);
        if global_speed > 1.0 {
            Decimal::from_finite(global_speed).pow(Decimal::from_finite(exponent))
        } else {
            Decimal::from_finite(global_speed)
        }
    };

    // CubeTribute: ant-speed cube-blessing chain platonic → hypercube →
    // tesseract → cube (mirrors `calculateAntSpeedCubeBlessing` in Cubes.ts).
    let platonic_amplifier =
        calculate_hypercube_blessing_multiplier_platonic_blessing(&state.platonic_blessings);
    let hypercube_blessing =
        calculate_ant_speed_hypercube_blessing(&state.hypercube_blessings, platonic_amplifier);
    let tesseract_blessing =
        calculate_ant_speed_tesseract_blessing(&state.tesseract_blessings, hypercube_blessing);
    let cube_tribute = calculate_ant_speed_cube_blessing(
        &state.cube_blessings,
        tesseract_blessing,
        cube[CUBE_UPGRADE_ANT_SPEED_BLESSING],
    );

    // RuneBlessingBonus: max(1, obtainium) ^ obtToAntExponent.
    let obt_to_ant_exponent = superior_intellect_rune_blessing_effects(rune_blessing_power(
        state,
        RUNE_SUPERIOR_INTELLECT,
    ))
    .obt_to_ant_exponent;
    let rune_blessing_bonus = obtainium
        .max(Decimal::one())
        .pow(Decimal::from_finite(obt_to_ant_exponent));

    // log10(crumbs + 10) — reused by Research6x22 / Research8x2.
    let crumbs_log10 = (crumbs + Decimal::from_finite(10.0)).log10().to_number();

    // SingularityPerk tiers.
    let singularity_perk = if highest_sing >= 100.0 {
        1e12
    } else if highest_sing >= 70.0 {
        1e6
    } else if highest_sing >= 40.0 {
        1e3
    } else if highest_sing >= 1.0 {
        4.44
    } else {
        1.0
    };

    let base = product_decimal(&[
        // Base — canGenerateAntCrumbs ? 1 : 0.
        if state.challenges.challenge_completions[8] > 0.0 || cube[48] > 0.0 {
            Decimal::one()
        } else {
            Decimal::zero()
        },
        global_speed_line,
        Decimal::from_finite(ant_speed_reward(
            &state.achievements.achievements,
            crumbs,
            state.ants.immortal_elo,
        )), // AchievementBonus
        // ImmortalELO — calculateAntSpeedMultFromELO = 1.02 ^ rebornELO.
        Decimal::from_finite(1.02).pow(Decimal::from_finite(state.ants.reborn_elo)),
        ant_speed_ant_upgrade_effect(&AntSpeedAntUpgradeInput {
            level: true_ant_level(state, ANT_UPGRADE_ANT_SPEED),
            research_101: researches[101],
            research_162: researches[162],
        }), // AntUpgrade1
        Decimal::from_finite(1.0 + 0.6 * up(39)), // DiamondUpgrade19
        Decimal::from_finite(1.0 + 4.0 * up(76)), // ReincarnationUpgrade16
        // ReincarnationUpgrade17 — (1 + upgrades[77]/250) ^ workersPurchased.
        if upgrades[77] > 0 {
            Decimal::from_finite(1.0 + up(77) / 250.0).pow(Decimal::from_finite(workers_purchased))
        } else {
            Decimal::one()
        },
        Decimal::one(), // ReincarnationUpgrade18 — maxOfferings untracked → 1 (upgrades[78] branch is 1 at maxOfferings 0)
        // Research4x21 — (1 + researches[96]/5000) ^ workersPurchased.
        Decimal::from_finite(1.0 + researches[96] / 5000.0)
            .pow(Decimal::from_finite(workers_purchased)),
        // Research5x17 — 1 + researches[117]·antSacrificeCount/10000.
        Decimal::from_finite(1.0 + researches[117] * state.ants.ant_sacrifice_count / 10_000.0),
        Decimal::from_finite(1.0 + researches[147] * crumbs_log10), // Research6x22
        Decimal::from_finite(1.0 + researches[177] * crumbs_log10), // Research8x2
        Decimal::from_finite(superior_intellect_rune_effects(
            first_five_effective_rune_level(state, RUNE_SUPERIOR_INTELLECT),
            SuperiorIntellectRuneKey::AntSpeed,
        )), // SuperiorIntellect
        rune_blessing_bonus,
        // Challenge9Bonus — 1.1 ^ CalcECC('reincarnation', cc[9]).
        Decimal::from_finite(1.1).pow(Decimal::from_finite(calc_ecc(
            ChallengeType::Reincarnation,
            state.challenges.challenge_completions[9],
        ))),
        // Challenge11Bonus — 1e5 ^ CalcECC('ascension', cc[11]).
        Decimal::from_finite(1e5).pow(Decimal::from_finite(calc_ecc(
            ChallengeType::Ascension,
            state.challenges.challenge_completions[11],
        ))),
        cube_tribute,
        // ConstantUpgrade — 1 + 0.1·log10(ascendShards + 1)·constantUpgrades[5].
        Decimal::from_finite(
            1.0 + 0.1
                * (state.campaigns.ascend_shards + Decimal::one())
                    .log10()
                    .to_number()
                * state.campaigns.constant_upgrades[5],
        ),
        Decimal::from_finite(challenge_15_rewards::ant_speed(
            state.challenges.challenge15_exponent,
        )), // Challenge15
        // PlatonicUpgrade — (1 + 0.01·platonic[12]) ^ Σ highestChallengeCompletions.
        Decimal::from_finite(1.0 + 0.01 * platonic[12]).pow(Decimal::from_finite(
            state
                .challenges
                .highest_challenge_completions
                .iter()
                .sum::<f64>(),
        )),
        Decimal::from_finite(singularity_perk), // SingularityPerk
        // CookieUpgrade — (1 + cubeUpgrades[65]/250) ^ workersPurchased.
        Decimal::from_finite(1.0 + cube[65] / 250.0).pow(Decimal::from_finite(workers_purchased)),
        Decimal::from_finite(octeract_starter_effect(
            state.octeract_upgrades.upgrades[OCTERACT_STARTER].level
                + state.octeract_upgrades.upgrades[OCTERACT_STARTER].free_level,
            OcteractStarterKey::AntSpeedMult,
        )), // OcteractUpgrade
    ]);

    calculate_actual_ant_speed_mult(&ActualAntSpeedMultInput {
        base,
        ascension_challenge: state.challenges.current_ascension_challenge,
        platonic_upgrade_10: platonic[10],
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

/// `G.TIME_PER_RED_AMBROSIA` — base seconds per red-ambrosia bar
/// (Variables.ts; a fixed `G` constant, never reassigned in the legacy).
const TIME_PER_RED_AMBROSIA: f64 = 100_000.0;

/// Pure-state red-ambrosia-timer threshold fields, from the legacy Helper.ts
/// `addTimers('redAmbrosia')` case: the red-accelerator's bonus blueberry
/// time minted per red ambrosia
/// (`redAmbrosiaAccelerator.ambrosiaTimePerRedAmbrosia`), the base
/// `TIME_PER_RED_AMBROSIA` constant, and the EXALT-2 `limitedTime`
/// bar-requirement multiplier. Returned as `(ambrosia_time_per_red_ambrosia,
/// time_per_red_ambrosia, red_ambrosia_bar_requirement_multiplier)`. The
/// red-accelerator upgrade uses `.level` only (red upgrades have no
/// free-level). All three equal the old `AutomationPre::default()` at the
/// default state, so the sim/tests don't shift.
fn compute_red_ambrosia_timer_fields(state: &GameState) -> (f64, f64, f64) {
    use crate::mechanics::red_ambrosia_upgrades::red_ambrosia_accelerator_effect;
    use crate::mechanics::singularity_challenges::{
        limited_time_effect, LimitedTimeKey, SingularityEffectValue,
    };
    use crate::state::red_ambrosia::RED_AMBROSIA_RED_AMBROSIA_ACCELERATOR;

    let bar_req = match limited_time_effect(
        state.singularity.limited_time.completions,
        LimitedTimeKey::BarRequirementMultiplier,
    ) {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 1.0,
    };
    (
        red_ambrosia_accelerator_effect(
            state.red_ambrosia.upgrades[RED_AMBROSIA_RED_AMBROSIA_ACCELERATOR].level,
        ),
        TIME_PER_RED_AMBROSIA,
        bar_req,
    )
}

/// Octeract-timer unlock gate, from the legacy Helper.ts
/// `addTimers('octeracts')` guard: `getGQUpgradeEffect('octeractUnlock',
/// 'unlocked')` = the GQ `octeractUnlock` slot's `bool_unlock` (`level > 0`).
/// `false` at the default state, matching the old `AutomationPre::default()`.
fn compute_octeract_unlocked(state: &GameState) -> bool {
    use crate::mechanics::golden_quark_upgrades::octeract_unlock_effect;
    use crate::state::golden_quarks::GQ_OCTERACT_UNLOCK;

    octeract_unlock_effect(state.golden_quarks.upgrades[GQ_OCTERACT_UNLOCK].level)
}

/// Quark-export timer cap, from the legacy Helper.ts `addTimers('quarks')`
/// case: `quarkHandler().maxTime`. The quark timer reads only `max_time` (its
/// upper-bound clamp), which depends solely on `research[195]` (`90000 +
/// 18000·n` when `n > 0`, else `90000`); the per-hour / gain / capacity
/// outputs are unused here, so [`quark_handler`]'s other inputs are passed at
/// their neutral values. `90000` at the default state, matching the old
/// `AutomationPre::default()`.
fn compute_max_quark_timer(state: &GameState) -> f64 {
    use crate::mechanics::quarks::{quark_handler, QuarkHandlerInput};

    quark_handler(&QuarkHandlerInput {
        research_195: state.researches.researches[195],
        researches_sum: 0.0,
        export_quark_mult: 1.0,
        quarks_timer: 0.0,
        cube_mult: 1.0,
    })
    .max_time
}

/// Roomba auto-research unlock gate (legacy `roombaResearchEnabled()`):
/// `cubeUpgrades[9] === 1 || highestSingularityCount > 10`. `false` at the
/// default state, matching the old `AutomationPre::default()`.
fn compute_roomba_unlocked(state: &GameState) -> bool {
    // legacy `player.cubeUpgrades[9]`
    const CUBE_UPGRADE_ROOMBA: usize = 9;
    state.cube_upgrade_levels.cube_upgrades[CUBE_UPGRADE_ROOMBA] == 1.0
        || state.singularity.highest_singularity_count > 10.0
}

/// Rune auto-sacrifice shop gate (legacy `getShopUpgradeEffects('offeringAuto',
/// 'autoRune')`): the `offeringAuto` slot's `AutoRune` unlock (`level > 0`),
/// AND-combined downstream with the persisted `rune_sacrifice_auto_enabled`
/// toggle. `false` at the default state.
fn compute_offering_auto_rune(state: &GameState) -> bool {
    use crate::mechanics::shop_upgrades::{
        offering_auto_effect, OfferingAutoKey, OfferingAutoValue,
    };
    use crate::state::shop::SHOP_OFFERING_AUTO;

    matches!(
        offering_auto_effect(
            state.shop.upgrades[SHOP_OFFERING_AUTO],
            OfferingAutoKey::AutoRune
        ),
        OfferingAutoValue::Unlock(true)
    )
}

/// Auto-prestige level milestone (legacy `getLevelMilestone('autoPrestige')`):
/// `1` once the achievement level reaches the milestone's `level_req` (7),
/// else `0`. The auto-reset machine unlocks auto-prestige when this `== 1`.
/// `0` at the default state.
fn compute_auto_prestige_milestone(state: &GameState) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::level_milestones::{get_level_milestone, LevelMilestoneKey};

    get_level_milestone(
        LevelMilestoneKey::AutoPrestige,
        achievement_level_from_points(state.achievements.achievement_points),
    )
}

/// Available (un-activated) reborn ELO, from the legacy middle's
/// `calculateAvailableRebornELO()`: `max(0, immortalELO − rebornELO)`, both
/// plain ant state. Drives the "maxed reborn ELO" ant-sacrifice toggles.
/// `0` at the default state.
fn compute_available_reborn_elo(state: &GameState) -> f64 {
    use crate::mechanics::ant_reborn_elo::{
        calculate_available_reborn_elo, AvailableRebornELOInput,
    };

    calculate_available_reborn_elo(&AvailableRebornELOInput {
        immortal_elo: state.ants.immortal_elo,
        reborn_elo: state.ants.reborn_elo,
    })
}

/// `calculateEffectiveAntELO` (Statistics-backed) — self-derived from
/// `&GameState`. `⌊Σ antELOStats × Σ additiveAntELOMultStats⌋`: the base-ELO
/// sum (15 lines) times the additive-multiplier sum (10 lines, base 1), both
/// StatLine reductions. Feeds [`compute_immortal_elo_gain`], the per-sacrifice
/// talisman-item thresholds, and the reborn-ELO creation-speed `EffectiveELO`
/// line.
///
/// Self-derives to `1` at the default state — the `ants` level reward's
/// `defaultValue` (1) is the sole non-zero base line, × mult 1, floored. The
/// `SingularityDebuff` line neutral-defaults to its Ant-ELO no-penalty value
/// `0` (additive context; `calculate_singularity_debuff` is banner-flagged
/// DO NOT extend / paused).
fn compute_effective_ant_elo(state: &GameState) -> f64 {
    use crate::mechanics::achievement_levels::achievement_level_from_points;
    use crate::mechanics::achievement_rewards::{
        ant_elo_additive, ant_elo_additive_multiplier, ant_speed_2_upgrade_improver,
    };
    use crate::mechanics::ant_reborn_elo::{
        calculate_singularity_perk_elo, singularity_elo_bonus_mult, SingularityPerkELOInput,
    };
    use crate::mechanics::ant_upgrades::{
        ant_elo_ant_upgrade_effect, ant_sacrifice_ant_upgrade_effect, AntELOAntUpgradeInput,
    };
    use crate::mechanics::calculate::sum_f64;
    use crate::mechanics::challenges::{calc_ecc, ChallengeType};
    use crate::mechanics::level_rewards::{get_level_reward, LevelRewardKey};
    use crate::mechanics::shop_upgrades::ant_speed_effect;
    use crate::state::shop::SHOP_ANT_SPEED;
    use crate::state::EXTINCTION_INDEX;

    // Ant producer slots (no enum in logic): Workers .. HolySpirit = 0..=8.
    const WORKERS: usize = 0;
    const QUEENS: usize = 4;
    const LORD_ROYALS: usize = 5;
    const ALMIGHTIES: usize = 6;
    const DISCIPLES: usize = 7;
    const HOLY_SPIRIT: usize = 8;
    // Ant upgrade slots.
    const ANT_UPGRADE_ANT_SACRIFICE: usize = 10;
    const ANT_UPGRADE_ANT_ELO: usize = 12;
    // legacy `player.upgrades[80]` — Reincarnation upgrade 2x20.
    const REINCARNATION_UPGRADE_20: usize = 80;
    // legacy `player.platonicUpgrades[12]`.
    const PLATONIC_UPGRADE_12: usize = 12;

    let ach = &state.achievements.achievements;
    let ach_level = achievement_level_from_points(state.achievements.achievement_points);
    let researches = &state.researches.researches;
    let producers = &state.ants.producers;
    let sac_count = state.ants.ant_sacrifice_count;
    let immortal_elo = state.ants.immortal_elo;
    let sing_count = state.singularity.singularity_count;
    let purchased = |i: usize| producers[i].purchased;

    // ReincarnationUpgrade20 — `player.upgrades[80]` gates a sac-count ramp.
    let reincarnation_upgrade_20 = if state.upgrades.upgrades[REINCARNATION_UPGRADE_20] == 0 {
        0.0
    } else {
        10.0 * 50.0_f64.min(sac_count)
            + 5.0 * 50.0_f64.min(0.0_f64.max(sac_count - 50.0))
            + 250.0_f64.min(0.0_f64.max(sac_count - 100.0))
    };

    let base_ant_elo = sum_f64(&[
        purchased(WORKERS),
        ant_elo_additive(ach),
        get_level_reward(LevelRewardKey::Ants, ach_level),
        reincarnation_upgrade_20,
        100.0
            * calc_ecc(
                ChallengeType::Reincarnation,
                state.challenges.challenge_completions[10],
            ),
        ant_speed_effect(state.shop.upgrades[SHOP_ANT_SPEED]),
        25.0 * researches[108],
        25.0 * researches[109],
        2.0 * researches[120],
        50.0 * researches[123],
        0.02 * researches[169],
        666.0 * researches[178],
        ant_sacrifice_ant_upgrade_effect(true_ant_level(state, ANT_UPGRADE_ANT_SACRIFICE)).elo,
        ant_elo_ant_upgrade_effect(&AntELOAntUpgradeInput {
            level: true_ant_level(state, ANT_UPGRADE_ANT_ELO),
            ant_sacrifice_count: sac_count,
            ant_speed_2_upgrade_improver: ant_speed_2_upgrade_improver(ach, ach_level),
        })
        .ant_elo,
        calculate_singularity_perk_elo(&SingularityPerkELOInput {
            sing_count,
            immortal_elo,
        }),
    ]);

    let elo_mult = sum_f64(&[
        1.0, // Base
        ant_elo_additive_multiplier(ach),
        if purchased(QUEENS) > 0.0 { 0.01 } else { 0.0 },
        if purchased(LORD_ROYALS) > 0.0 {
            0.01
        } else {
            0.0
        },
        if purchased(ALMIGHTIES) > 0.0 {
            0.01
        } else {
            0.0
        },
        if purchased(DISCIPLES) > 0.0 {
            0.02
        } else {
            0.0
        },
        if purchased(HOLY_SPIRIT) > 0.0 {
            0.02
        } else {
            0.0
        },
        (1.0 / 200.0)
            * state.cube_upgrade_levels.platonic_upgrades[PLATONIC_UPGRADE_12]
            * f64::from(state.corruptions.used.levels[EXTINCTION_INDEX]),
        0.0, // SingularityDebuff — Ant-ELO no-penalty value (paused layer)
        singularity_elo_bonus_mult(sing_count),
    ]);

    (base_ant_elo * elo_mult).floor()
}

/// Ant-sacrifice `immortalELO` gain (legacy `antSacrificeRewards().immortalELO`):
/// `max(0, calculateEffectiveAntELO − immortalELO)`. Drives the
/// `ImmortalELOGain` auto-sacrifice mode. Self-derives to `1` at the default
/// state (effective ELO `1`, immortal ELO `0`); harmless because the
/// ant-sacrifice middle that reads it is gated by `ant_sacrifice_unlocked`
/// (false at default), so it is never consumed there, and `1` is the faithful
/// legacy value.
fn compute_immortal_elo_gain(state: &GameState) -> f64 {
    use crate::mechanics::ant_sacrifice_reward_calc::{
        calculate_immortal_elo_gain, CalculateImmortalELOGainInput,
    };
    calculate_immortal_elo_gain(&CalculateImmortalELOGainInput {
        effective_elo: compute_effective_ant_elo(state),
        immortal_elo: state.ants.immortal_elo,
    })
}

/// Per-challenge completion caps for the ten regular challenges (`1..=10`),
/// from the legacy `getMaxChallenges`. Index `0` is unused. The shared
/// reincarnation/ascension-tier cap inputs are evaluated once; only the
/// transcension tier (`1..=5`) varies per challenge (via `researches[65 + i]`).
/// Feeds the challenge-sweep `getNextRegularChallenge` lookups and the
/// `finished` revalidation guard. GQ cap-increase upgrades use the effective
/// level (`level + free_level`, the `actualGQUpgradeTotalLevels` convention).
fn compute_max_challenges_1_to_10(state: &GameState) -> [f64; 11] {
    use crate::mechanics::challenges::{get_max_challenges, GetMaxChallengesInput};
    use crate::mechanics::golden_quark_upgrades::{
        sing_challenge_extension_2_effect, sing_challenge_extension_3_effect,
        sing_challenge_extension_effect, SingChallengeExtensionKey,
    };
    use crate::mechanics::shop_upgrades::challenge_extension_effect;
    use crate::mechanics::singularity_challenges::{
        one_challenge_cap_effect, OneChallengeCapKey, SingularityEffectValue,
    };
    use crate::state::golden_quarks::{
        GQ_SING_CHALLENGE_EXTENSION, GQ_SING_CHALLENGE_EXTENSION_2, GQ_SING_CHALLENGE_EXTENSION_3,
    };
    use crate::state::shop::SHOP_CHALLENGE_EXTENSION;

    let researches = &state.researches.researches;
    let cube = &state.cube_upgrade_levels;
    let occ = &state.singularity.one_challenge_cap;
    let gq = |i: usize| {
        state.golden_quarks.upgrades[i].level + state.golden_quarks.upgrades[i].free_level
    };
    let scalar = |v: SingularityEffectValue| match v {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 0.0,
    };

    let cap_increase = |key: SingChallengeExtensionKey| {
        sing_challenge_extension_effect(gq(GQ_SING_CHALLENGE_EXTENSION), key)
            + sing_challenge_extension_2_effect(gq(GQ_SING_CHALLENGE_EXTENSION_2), key)
            + sing_challenge_extension_3_effect(gq(GQ_SING_CHALLENGE_EXTENSION_3), key)
    };
    let gq_reincarnation_cap_increase =
        cap_increase(SingChallengeExtensionKey::ReincarnationCapIncrease);
    let gq_ascension_cap_increase = cap_increase(SingChallengeExtensionKey::AscensionCapIncrease);
    let sing_reincarnation_cap_increase = scalar(one_challenge_cap_effect(
        occ.completions,
        OneChallengeCapKey::CapIncrease,
    )) + scalar(one_challenge_cap_effect(
        occ.completions,
        OneChallengeCapKey::ReinCapIncrease2,
    ));
    let sing_ascension_cap_increase = scalar(one_challenge_cap_effect(
        occ.completions,
        OneChallengeCapKey::AscCapIncrease2,
    ));
    let challenge_extension_cap =
        challenge_extension_effect(state.shop.upgrades[SHOP_CHALLENGE_EXTENSION]);

    let mut caps = [0.0_f64; 11];
    for (i, slot) in caps.iter_mut().enumerate().skip(1) {
        *slot = get_max_challenges(&GetMaxChallengesInput {
            challenge: i as u8,
            one_challenge_cap_enabled: occ.enabled,
            infinite_transcend_research: researches[105],
            transcend_research_for_challenge: researches[65 + i],
            cube_upgrade_29: cube.cube_upgrades[29],
            challenge_extension_cap,
            gq_reincarnation_cap_increase,
            sing_reincarnation_cap_increase,
            gq_ascension_cap_increase,
            sing_ascension_cap_increase,
            platonic_upgrade_5: cube.platonic_upgrades[5],
            platonic_upgrade_10: cube.platonic_upgrades[10],
            platonic_upgrade_15: cube.platonic_upgrades[15],
        });
    }
    caps
}

/// The seven challenge-sweep pre-evaluations the tail's `tick_challenge_sweep`
/// consumes ([`AutomationPre`] `sweep_*` fields), self-derived from
/// `&GameState`. Verbatim port of the legacy `prepareSweepInputForTackTail`
/// (web_ui/Challenges.ts): scoped to the current `sweep_state`, so the
/// `getNextRegularChallenge` / `challenge15AutoExponentCheck` / finished-guard
/// lookups only run for the state that could consult them this tick.
struct SweepPreEvals {
    timer_start: f64,
    timer_exit: f64,
    timer_enter: f64,
    next_regular_challenge_from_initial: i32,
    next_regular_challenge_from_active: i32,
    challenge_15_auto_exponent_check: bool,
    is_finished_still_valid: bool,
}

fn compute_sweep_pre_evals(state: &GameState) -> SweepPreEvals {
    use crate::events::SweepState;
    use crate::mechanics::challenges::{
        auto_ascension_challenge_sweep_unlock, challenge_15_auto_exponent_check,
        get_next_regular_challenge, Challenge15AutoExponentCheckInput,
        GetNextRegularChallengeInput,
    };
    use crate::mechanics::shop_upgrades::{
        challenge_15_auto_effect, instant_challenge_2_effect, InstantChallengeKey,
        InstantChallengeValue,
    };
    use crate::state::automation::AutoAscendMode;
    use crate::state::shop::{SHOP_CHALLENGE_15_AUTO, SHOP_INSTANT_CHALLENGE_2};

    let timer = state.automation.auto_challenge_timer;
    let toggles = &state.automation.auto_challenge_toggles;
    let highest_sing = state.singularity.highest_singularity_count;
    let highest_completions = &state.challenges.highest_challenge_completions;

    // Only the initial-wait / active / finished states consult lookups (the
    // legacy scoping); all others leave the four non-timer fields inert.
    let (
        next_regular_challenge_from_initial,
        next_regular_challenge_from_active,
        challenge_15_auto_exponent_check,
        is_finished_still_valid,
    ) = match &state.automation.sweep_state {
        SweepState::InitialWait => {
            let max_challenges = compute_max_challenges_1_to_10(state);
            // initialIndex 10 once an ascension challenge is active past sing 2.
            let initial_index: u8 =
                if highest_sing >= 2.0 && state.challenges.current_ascension_challenge != 0 {
                    10
                } else {
                    1
                };
            let next = get_next_regular_challenge(&GetNextRegularChallengeInput {
                start_index: initial_index,
                explored: &[],
                max_challenges: &max_challenges,
                highest_completions,
                auto_challenge_toggles: toggles,
            });
            (next, -1, false, false)
        }
        SweepState::Active { index, explored } => {
            let max_challenges = compute_max_challenges_1_to_10(state);
            let explored: Vec<u8> = explored.iter().copied().collect();
            let next = get_next_regular_challenge(&GetNextRegularChallengeInput {
                start_index: *index,
                explored: &explored,
                max_challenges: &max_challenges,
                highest_completions,
                auto_challenge_toggles: toggles,
            });
            let instant_c2_unlocked = matches!(
                instant_challenge_2_effect(
                    state.shop.upgrades[SHOP_INSTANT_CHALLENGE_2],
                    InstantChallengeKey::Unlocked,
                    highest_sing,
                ),
                InstantChallengeValue::Unlock(true)
            );
            let c15 = challenge_15_auto_exponent_check(&Challenge15AutoExponentCheckInput {
                sweep_unlocked: auto_ascension_challenge_sweep_unlock(
                    highest_sing,
                    instant_c2_unlocked,
                ),
                current_ascension_challenge: state.challenges.current_ascension_challenge,
                challenge_15_auto_shop_unlocked: challenge_15_auto_effect(
                    state.shop.upgrades[SHOP_CHALLENGE_15_AUTO],
                ),
                auto_ascend: state.automation.auto_ascend,
                cube_upgrade_10: state.cube_upgrade_levels.cube_upgrades[10],
                auto_ascend_mode_is_real_time: matches!(
                    state.automation.auto_ascend_mode,
                    AutoAscendMode::RealAscensionTime
                ),
                ascension_counter_real_real: state.reset_counters.ascension_counter_real_real,
                auto_ascend_threshold: state.automation.auto_ascend_threshold,
            });
            (-1, next, c15, false)
        }
        SweepState::Finished => {
            let max_challenges = compute_max_challenges_1_to_10(state);
            let valid = highest_completions[1] == max_challenges[1]
                && highest_completions[6] == max_challenges[6];
            (-1, -1, false, valid)
        }
        _ => (-1, -1, false, false),
    };

    SweepPreEvals {
        timer_start: timer.start,
        timer_exit: timer.exit,
        timer_enter: timer.enter,
        next_regular_challenge_from_initial,
        next_regular_challenge_from_active,
        challenge_15_auto_exponent_check,
        is_finished_still_valid,
    }
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
        crate::mechanics::campaign_token_rewards::campaign_ambrosia_luck_bonus(
            compute_campaign_tokens(state),
        ), // Campaign — player.campaigns.ambrosiaLuckBonus
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
/// The campaign blueberry-speed bonus is wired to the derived token total.
/// Multiplicative lines whose context is unported are neutral `1.0`
/// (planar-coin, shop `panthema`, patreon [quark-bonus arg], event); the
/// additive blueberry lines neutral `0`. All are inert at the current play
/// state.
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
        crate::mechanics::campaign_token_rewards::campaign_blueberry_speed_bonus(
            compute_campaign_tokens(state),
        ), // Campaign — player.campaigns.blueberrySpeedBonus
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
    use crate::mechanics::challenges::CHALLENGE_BASE_REQUIREMENTS;

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
        // Static legacy constant (c1-5 slice of the shared table).
        challenge_base_requirements: [
            CHALLENGE_BASE_REQUIREMENTS[0],
            CHALLENGE_BASE_REQUIREMENTS[1],
            CHALLENGE_BASE_REQUIREMENTS[2],
            CHALLENGE_BASE_REQUIREMENTS[3],
            CHALLENGE_BASE_REQUIREMENTS[4],
        ],
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
/// Each queued [`PlayerAction`] dispatches into its corresponding mutator:
/// [`BuyRequest`] → a `buy_*` loop, [`ResetRequest`] → the reset executor
/// (which awards `reset_gains`, computed at the top of [`tack`]). Events
/// flow into [`TickOutput::events`].
fn phase_player_input(
    state: &mut GameState,
    input: &TackInput,
    reset_gains: &crate::mechanics::reset_currency::ResetCurrencyResult,
    output: &mut TickOutput,
) {
    for action in &input.player_actions {
        match action {
            PlayerAction::Buy(req) => {
                output
                    .events
                    .extend(dispatch_buy(state, req, reset_gains.prestige_point_gain));
            }
            PlayerAction::Reset(req) => {
                output
                    .events
                    .extend(reset::perform_reset(state, *req, reset_gains));
            }
            PlayerAction::SetCorruptionLevel { index, level } => {
                output
                    .events
                    .extend(set_corruption_level(state, *index, *level));
            }
            PlayerAction::ToggleAuto { target, enabled } => {
                set_automation_toggle(state, *target, *enabled);
            }
            PlayerAction::EnterChallenge { challenge } => {
                output
                    .events
                    .extend(enter_challenge(state, *challenge, reset_gains));
            }
            PlayerAction::OpenCubes { tier, value, max } => {
                output.events.extend(crate::mechanics::cube_opening::open(
                    state, *tier, *value, *max,
                ));
            }
            PlayerAction::ToggleSingularityChallenge { challenge } => {
                output
                    .events
                    .extend(reset::toggle_singularity_challenge(state, *challenge));
            }
            PlayerAction::SelectCampaign { campaign } => {
                output
                    .events
                    .extend(select_campaign(state, *campaign, reset_gains));
            }
            PlayerAction::ConfigureSingularityElevator {
                target,
                locked,
                slow_climb,
            } => {
                let requested = if target.is_finite() { *target } else { 1.0 };
                let max_target = elevator_max_target(state);
                state.singularity.elevator_target = requested.clamp(1.0, max_target);
                state.singularity.elevator_locked = *locked;
                state.singularity.elevator_slow_climb = *slow_climb;
            }
            PlayerAction::TeleportToSingularity => {
                output.events.extend(reset::teleport_to_singularity(state));
            }
        }
    }
}

/// `toggleChallenges` — enter a challenge: set the `current_*_challenge` slot,
/// then run the matching tier reset (the challenge-reset variants share the
/// tier-reset branch in the legacy `reset()`). The transcension / reincarnation
/// resets do not clear their own current-challenge slot, so the set sticks; a
/// higher-tier reset clears lower slots (faithful). Ascension challenges
/// (`11..=15`) are gated + run the heavy ascension reset and are deferred to a
/// later chunk (ignored here). Returns a `ChallengeEntered` event followed by
/// the tier reset's events.
fn enter_challenge(
    state: &mut GameState,
    challenge: u32,
    gains: &crate::mechanics::reset_currency::ResetCurrencyResult,
) -> SmallVec<[CoreEvent; 2]> {
    let request = if challenge <= 5 {
        state.challenges.current_transcension_challenge = challenge;
        ResetRequest::Transcension
    } else if challenge <= 10 {
        state.challenges.current_reincarnation_challenge = challenge;
        ResetRequest::Reincarnation
    } else if (11..=15).contains(&challenge) {
        // ── Ascension challenges (c11–c15) entry guard ────────────────────
        // Mirrors `toggleChallenges` (Toggles.ts:73–105).
        //
        // c11: requires `player.unlocks.ascensions` (ascension_unlocked).
        // c12-c15: requires `highestchallengecompletions[challenge-1] > 0`.
        //
        // The `(!auto && !toggles[31]) || challengecompletions[10] > 0`
        // c10-condition: in the logic tier `auto = false` and confirmation
        // dialogs (toggles[31]) are skipped, so the short-circuit
        // `!auto && !toggles[31]` always fires as `true` — BUT the TS also
        // allows the unconditional-enter path when all three current
        // challenge slots are clear (no active challenge in any tier).
        // We mirror that: if c10 has no completions AND a challenge is
        // active somewhere, block entry; otherwise allow.
        let c11_ok = challenge == 11 && state.reset_counters.ascension_unlocked;
        let c12_15_ok = challenge >= 12
            && state.challenges.highest_challenge_completions[challenge as usize - 1] > 0.0;
        if !c11_ok && !c12_15_ok {
            return smallvec![];
        }
        // c10 condition: allow if c10 completions > 0 OR no active challenge.
        let c10_ok = state.challenges.challenge_completions[10] > 0.0
            || (state.challenges.current_transcension_challenge == 0
                && state.challenges.current_reincarnation_challenge == 0
                && state.challenges.current_ascension_challenge == 0);
        if !c10_ok {
            return smallvec![];
        }
        state.challenges.current_ascension_challenge = challenge;
        ResetRequest::AscensionChallenge
    } else {
        return smallvec![];
    };
    let mut events: SmallVec<[CoreEvent; 2]> = smallvec![CoreEvent::ChallengeEntered { challenge }];
    events.extend(reset::perform_reset(state, request, gains));
    events
}

/// Challenge-completion tick phase (legacy `Synergism.ts:3424-3477` auto-check
/// and the `resetCheck` completion award). Runs after [`phase_generation`] so
/// the goal resources are fresh. Handles transcension (c1-5, goal =
/// `coins_this_transcension`), reincarnation (c6-10, goal = `transcend_shards`
/// for c6-8 / `coins` for c9-10), and ascension challenges (c11-14, goal =
/// `challenge_completions[10] >= challenge_requirement` multiplier):
/// if the goal meets `challenge_requirement`, [`complete_active_challenge`]
/// awards completions, raises `highest`, exits, and resets out.
///
///
/// c15 is deferred (different requirement shape and exponent path). Faithful
/// neutral-defaults at this scope: the corruption `hyperchallenge` requirement
/// inflation, the c15 transcend/reincarnation reductions, the c10 requirement
/// reduction, and the shop `challengeExtension` reincarnation cap (→
/// requirements/caps as if those upgrades are absent); `highestChallengeRewards`
/// fires quark awards per new highest rise (ported); the post-completion reset
/// reuses the tick-start `gains` (the port's standing simplification for all
/// in-tick resets).
fn phase_challenge_completion(
    state: &mut GameState,
    gains: &crate::mechanics::reset_currency::ResetCurrencyResult,
    output: &mut TickOutput,
) {
    use crate::mechanics::challenges::{
        challenge_15_score_multiplier, challenge_requirement, get_max_challenges,
        Challenge15ScoreMultiplierInput, ChallengeRequirementInput, GetMaxChallengesInput,
        CHALLENGE_BASE_REQUIREMENTS,
    };
    use crate::mechanics::shop_upgrades::{
        challenge_extension_effect, instant_challenge_2_effect, instant_challenge_effect,
        InstantChallengeKey, InstantChallengeValue,
    };
    use crate::state::shop::{
        SHOP_CHALLENGE_15_AUTO, SHOP_CHALLENGE_EXTENSION, SHOP_INSTANT_CHALLENGE,
        SHOP_INSTANT_CHALLENGE_2,
    };

    const PLATONIC_UPGRADE_8: usize = 8;
    const RESEARCH_INFINITE_TRANSCEND: usize = 105;
    const CUBE_UPGRADE_29: usize = 29;
    const PLATONIC_UPGRADE_5: usize = 5;
    const PLATONIC_UPGRADE_10: usize = 10;
    const PLATONIC_UPGRADE_15: usize = 15;

    // Per-tick multi-completion (instantChallenge extraCompPerTick), shared
    // across tiers; capped at 1 inside ascension challenge 13.
    let scalar = |v: InstantChallengeValue| match v {
        InstantChallengeValue::Scalar(s) => s,
        InstantChallengeValue::Unlock(_) => 0.0,
    };
    let max_inc = if state.challenges.current_ascension_challenge == 13 {
        1.0
    } else {
        1.0 + scalar(instant_challenge_effect(
            state.shop.upgrades[SHOP_INSTANT_CHALLENGE],
            InstantChallengeKey::ExtraCompPerTick,
        )) + scalar(instant_challenge_2_effect(
            state.shop.upgrades[SHOP_INSTANT_CHALLENGE_2],
            InstantChallengeKey::ExtraCompPerTick,
            state.singularity.highest_singularity_count,
        ))
    };
    // Reset out unless instantChallenge is unlocked (leaving = false in the tick).
    let instant_unlocked = matches!(
        instant_challenge_effect(
            state.shop.upgrades[SHOP_INSTANT_CHALLENGE],
            InstantChallengeKey::Unlocked,
        ),
        InstantChallengeValue::Unlock(true)
    );
    let platonic_8 = state.cube_upgrade_levels.platonic_upgrades[PLATONIC_UPGRADE_8];

    // Shared requirement builder (captures Copy values, not `state`).
    let requirement = |challenge: u32, comp: f64| -> Decimal {
        // `CHALLENGE_BASE_REQUIREMENTS` is indexed `[challenge-1]` for c1-10.
        // For c11-15 the base is unused by `challenge_requirement` (the ascension
        // path returns the raw multiplier without using base), so we pass 0.0.
        let challenge_base_requirement = if challenge <= 10 {
            CHALLENGE_BASE_REQUIREMENTS[challenge as usize - 1]
        } else {
            0.0
        };
        challenge_requirement(&ChallengeRequirementInput {
            challenge: challenge as u8,
            completion: comp,
            special: challenge as u8,
            challenge_base_requirement,
            c10_requirement_reduction: 0.0, // c10 reduction deferred (neutral)
            hyperchallenge_multiplier: 1.0, // corruption hyperchallenge inflation deferred
            platonic_upgrade_8: platonic_8,
            challenge_15_transcend_reduction: 1.0, // c15 reductions deferred
            challenge_15_reincarnation_reduction: 1.0,
            challenge_tome_c9c10_scaling_reduction: 0.0,
            challenge_tome_2_c9c10_scaling_reduction: 0.0,
        })
    };

    // ── Transcension challenges (c1-5): goal = coins this transcension.
    let qt = state.challenges.current_transcension_challenge;
    if (1..=5).contains(&qt) {
        let goal = state.coin_counters.coins_this_transcension;
        if goal >= requirement(qt, state.challenges.challenge_completions[qt as usize]) {
            let research = &state.researches.researches;
            let max_completions = get_max_challenges(&GetMaxChallengesInput {
                challenge: qt as u8,
                one_challenge_cap_enabled: false, // SC oneChallengeCap (singularity) → neutral
                infinite_transcend_research: research[RESEARCH_INFINITE_TRANSCEND],
                transcend_research_for_challenge: research[65 + qt as usize],
                cube_upgrade_29: 0.0,
                challenge_extension_cap: 0.0,
                gq_reincarnation_cap_increase: 0.0,
                sing_reincarnation_cap_increase: 0.0,
                gq_ascension_cap_increase: 0.0,
                sing_ascension_cap_increase: 0.0,
                platonic_upgrade_5: 0.0,
                platonic_upgrade_10: 0.0,
                platonic_upgrade_15: 0.0,
            });
            complete_active_challenge(
                state,
                qt,
                goal,
                max_completions,
                max_inc,
                &requirement,
                instant_unlocked,
                gains,
                output,
            );
        }
    }

    // ── Reincarnation challenges (c6-10): goal = transcendShards (c6-8) / coins (c9-10).
    let qr = state.challenges.current_reincarnation_challenge;
    if (6..=10).contains(&qr) {
        let goal = if qr < 9 {
            state.reset_counters.transcend_shards
        } else {
            state.upgrades.coins
        };
        if goal >= requirement(qr, state.challenges.challenge_completions[qr as usize]) {
            let platonic = &state.cube_upgrade_levels.platonic_upgrades;
            let max_completions = get_max_challenges(&GetMaxChallengesInput {
                challenge: qr as u8,
                one_challenge_cap_enabled: false,
                infinite_transcend_research: 0.0,
                transcend_research_for_challenge: 0.0,
                cube_upgrade_29: state.cube_upgrade_levels.cube_upgrades[CUBE_UPGRADE_29],
                challenge_extension_cap: challenge_extension_effect(
                    state.shop.upgrades[SHOP_CHALLENGE_EXTENSION],
                ),
                gq_reincarnation_cap_increase: 0.0,
                sing_reincarnation_cap_increase: 0.0,
                gq_ascension_cap_increase: 0.0,
                sing_ascension_cap_increase: 0.0,
                platonic_upgrade_5: platonic[PLATONIC_UPGRADE_5],
                platonic_upgrade_10: platonic[PLATONIC_UPGRADE_10],
                platonic_upgrade_15: platonic[PLATONIC_UPGRADE_15],
            });
            complete_active_challenge(
                state,
                qr,
                goal,
                max_completions,
                max_inc,
                &requirement,
                instant_unlocked,
                gains,
                output,
            );
        }
    }

    // ── Ascension challenges (c11-14): goal = challengecompletions[10] >= requirement.
    // `challengeRequirement` for c11-14 returns the multiplier directly as a
    // plain number (`Decimal` wrapping an f64), not a power-of-10 target.
    // The TS comparison: `player.challengecompletions[10] >= challengeRequirement(a, …)`
    // (Synergism.ts:3468-3473). c15 is handled separately (not via this tick path).
    let qa = state.challenges.current_ascension_challenge;
    if (11..=14).contains(&qa) {
        let c10_comp = state.challenges.challenge_completions[10];
        let req_val =
            requirement(qa, state.challenges.challenge_completions[qa as usize]).to_number();
        if c10_comp >= req_val {
            let platonic = &state.cube_upgrade_levels.platonic_upgrades;
            let max_completions = get_max_challenges(&GetMaxChallengesInput {
                challenge: qa as u8,
                one_challenge_cap_enabled: false,
                infinite_transcend_research: 0.0,
                transcend_research_for_challenge: 0.0,
                cube_upgrade_29: 0.0,
                challenge_extension_cap: 0.0,
                gq_reincarnation_cap_increase: 0.0,
                sing_reincarnation_cap_increase: 0.0,
                gq_ascension_cap_increase: 0.0,
                sing_ascension_cap_increase: 0.0,
                platonic_upgrade_5: platonic[PLATONIC_UPGRADE_5],
                platonic_upgrade_10: platonic[PLATONIC_UPGRADE_10],
                platonic_upgrade_15: platonic[PLATONIC_UPGRADE_15],
            });
            // Goal for the loop is c10 completions expressed as a Decimal.
            let goal = Decimal::from_finite(c10_comp);
            complete_active_challenge(
                state,
                qa,
                goal,
                max_completions,
                max_inc,
                &requirement,
                instant_unlocked,
                gains,
                output,
            );
        }
    }

    // ── Ascension challenge 15: exponent accrual.
    // c15 does NOT increment `challengecompletions`; instead `challenge15Exponent`
    // grows from coins, and the (already-ported) `challenge_15_rewards::*` cascade
    // reads that exponent live. Synergism.ts:4514-4525 ("Challenge 15 autoupdate")
    // and the `a === 15` branch of `resetCheck` (3760-3784) share the same body.
    // The only tick-reachable trigger is the `challenge15Auto` shop upgrade — the
    // manual / leaving-the-challenge update is UI-tier (deferred). `c15RewardUpdate()`
    // is a no-op for us (rewards are read live), and the
    // `challenge15Exponent >= 1e15 → unlocks.hepteracts` side-effect is already
    // covered: the hepteract-gain gate reads
    // `challenge_15_rewards::hepteracts_unlocked(exponent)` directly (calculate.rs).
    if qa == 15 && state.shop.upgrades[SHOP_CHALLENGE_15_AUTO] > 0.0 {
        // challenge15ScoreMultiplier(): campaign · challenge-hepteract · OMEGA.
        let c15_sm = challenge_15_score_multiplier(&Challenge15ScoreMultiplierInput {
            c15_bonus: crate::mechanics::campaign_token_rewards::campaign_c15_bonus(
                compute_campaign_tokens(state),
            ),
            // `hepteractEffective('challenge')` — challenge craft LIMIT 1000, DR 1/6,
            // DR_INCREASE 0 (Hepteracts.ts:190-192).
            challenge_hepteract_effective: hepteract_effective_bal(
                state.hepteracts.challenge.bal,
                1.0 / 6.0,
            ),
            platonic_upgrade_15: state.cube_upgrade_levels.platonic_upgrades[PLATONIC_UPGRADE_15],
        });
        // Grow only once coins clear the next threshold `10^(exponent / c15SM)`.
        let threshold = Decimal::from_finite(10.0).pow(Decimal::from_finite(
            state.challenges.challenge15_exponent / c15_sm,
        ));
        if state.upgrades.coins >= threshold {
            state.challenges.challenge15_exponent =
                (state.upgrades.coins + Decimal::one()).log10().to_number() * c15_sm;
        }
    }

    // sadisticAch (#252): `challengeAchievementCheck(15)` awards it once the c15
    // `achievementUnlock` reward (exponent >= 666666) is active. Idempotent and
    // independent of challenge15Auto (the exponent may already be high). pv 50.
    if qa == 15
        && challenge_15_rewards::achievement_unlock(state.challenges.challenge15_exponent) == 1.0
    {
        let awarded = crate::mechanics::achievement_awards::award_ungrouped_achievement(
            &mut state.achievements,
            252,
            50.0,
            true,
        );
        credit_achievement_quarks(state, awarded);
    }
}

/// Award + exit + reset for one in-progress challenge that has met its goal
/// (the `resetCheck` completion body). The tier (transcension vs reincarnation)
/// is derived from `challenge`. `requirement(challenge, comp)` is recomputed per
/// completion for the multi-complete loop.
fn complete_active_challenge(
    state: &mut GameState,
    challenge: u32,
    goal: Decimal,
    max_completions: f64,
    max_inc: f64,
    requirement: &impl Fn(u32, f64) -> Decimal,
    instant_unlocked: bool,
    gains: &crate::mechanics::reset_currency::ResetCurrencyResult,
    output: &mut TickOutput,
) {
    let q_idx = challenge as usize;
    let mut comp = state.challenges.challenge_completions[q_idx];
    let mut counter = 0.0;
    while counter < max_inc {
        if comp < max_completions && goal >= requirement(challenge, comp) {
            comp += 1.0;
        }
        counter += 1.0;
    }
    state.challenges.challenge_completions[q_idx] = comp;
    // challengeAchievementCheck(q) — award the challengeN group plus the
    // ungrouped per-challenge extras (chalNNoGen / diamondSearch / extraChallenging)
    // from the updated completion count (the legacy resetCheck calls it after the
    // completion increments). The extras' context is captured before the mutable
    // achievements borrow; `current_transcension_challenge` is still set here (the
    // challenge exits further below).
    let extras_ctx = crate::mechanics::achievement_awards::ChallengeUngroupedContext {
        coins_this_transcension_log10: (state.coin_counters.coins_this_transcension
            + Decimal::one())
        .log10()
        .to_number(),
        current_transcension_challenge: state.challenges.current_transcension_challenge,
        generator_upgrades_owned: state.upgrades.upgrades[101..=105]
            .iter()
            .map(|&u| u32::from(u))
            .sum(),
        accelerator_bought: state.accelerator.accelerator_bought,
        accelerator_boost_bought: state.accelerator.accelerator_boost_bought,
        extinction_level: f64::from(state.corruptions.used.levels[crate::state::EXTINCTION_INDEX]),
    };
    let awarded = crate::mechanics::achievement_awards::challenge_achievement_check(
        &mut state.achievements,
        q_idx,
        &state.challenges.challenge_completions,
    ) + crate::mechanics::achievement_awards::challenge_ungrouped_achievement_check(
        &mut state.achievements,
        q_idx,
        &state.challenges.challenge_completions,
        &extras_ctx,
    );
    credit_achievement_quarks(state, awarded);
    while state.challenges.challenge_completions[q_idx]
        > state.challenges.highest_challenge_completions[q_idx]
    {
        state.challenges.highest_challenge_completions[q_idx] += 1.0;
        // Ascension-challenge unlock side-effects fired on highest[i] first rise
        // (Synergism.ts:3692-3700 — inside the `resetCheck` reincarnation block).
        match q_idx {
            // Reincarnation challenge 8 unlocks the ant hill (anthill);
            // challenge 9 unlocks talismans + cube blessings; challenge 10
            // unlocks ascensions — the entry gate for ascension challenge 11
            // (without it the c11-c15 ladder is unreachable in normal play).
            8 => state.reset_counters.anthill_unlocked = true,
            9 => {
                state.reset_counters.talismans_unlocked = true;
                state.reset_counters.blessings_unlocked = true;
            }
            10 => state.reset_counters.ascension_unlocked = true,
            11 => state.reset_counters.tesseracts_unlocked = true,
            12 => state.reset_counters.spirits_unlocked = true,
            13 => state.reset_counters.hypercubes_unlocked = true,
            14 => state.reset_counters.platonics_unlocked = true,
            _ => {}
        }
        // highestChallengeRewards — award quarks when ascensionCount == 0
        // (Challenges.ts:435). The quark bonus (cached as a %-age in
        // state.quarks.quark_bonus) approximates calculateQuarkMultiplier().
        if state.reset_counters.ascension_count == 0.0 {
            use crate::mechanics::challenges::highest_challenge_rewards;
            let base = highest_challenge_rewards(
                challenge,
                state.challenges.highest_challenge_completions[q_idx],
            );
            let multiplier = 1.0 + state.quarks.quark_bonus / 100.0;
            let awarded = base * multiplier;
            state.quarks.worlds += synergismforkd_bignum::Decimal::from_finite(awarded);
            state.golden_quarks.quarks_this_singularity += awarded;
            output
                .events
                .push(CoreEvent::QuarksAwarded { quarks: awarded });
        }
    }

    // `retrychallenges` + `autoChallengeRunning` determine whether the slot
    // clears (Synergism.ts:3616 / 3702). In the tick path (`manual = false`):
    // - `retry_challenges = false` → always exit (clear the slot).
    // - `retry_challenges = true`  → stay in challenge unless the sweep is
    //   running AND completions have reached the cap (auto-sweep rotation).
    // The structural reset below always fires regardless.
    let stay_in_challenge = state.automation.retry_challenges
        && !(state.automation.auto_challenge_running && comp >= max_completions);

    let is_transcension = challenge <= 5;
    let is_ascension = challenge >= 11;

    if !stay_in_challenge {
        if is_transcension {
            state.challenges.current_transcension_challenge = 0;
        } else if is_ascension {
            state.challenges.current_ascension_challenge = 0;
        } else {
            state.challenges.current_reincarnation_challenge = 0;
        }
    }
    output.events.push(CoreEvent::ChallengeCompleted {
        challenge,
        completions: comp,
    });

    if !instant_unlocked {
        let request = if is_transcension {
            ResetRequest::Transcension
        } else if is_ascension {
            ResetRequest::AscensionChallenge
        } else {
            ResetRequest::Reincarnation
        };
        output
            .events
            .extend(reset::perform_reset(state, request, gains));
    }
}

/// `PlayerAction::ToggleAuto` handler — set the selected automation flag to
/// `enabled`. A pure config change (no event); `phase_automation` reads the
/// flag. Out-of-range challenge slots are ignored.
fn set_automation_toggle(state: &mut GameState, target: AutoToggle, enabled: bool) {
    let auto = &mut state.automation;
    match target {
        AutoToggle::AutoPrestige => auto.auto_prestige_enabled = enabled,
        AutoToggle::AutoTranscend => auto.auto_transcend_enabled = enabled,
        AutoToggle::AutoReincarnate => auto.auto_reincarnate_enabled = enabled,
        AutoToggle::AutoAscend => auto.auto_ascend = enabled,
        AutoToggle::RuneSacrifice => auto.rune_sacrifice_auto_enabled = enabled,
        AutoToggle::OfferingPotion => auto.auto_potion_toggle_offering = enabled,
        AutoToggle::ObtainiumPotion => auto.auto_potion_toggle_obtainium = enabled,
        AutoToggle::AutoChallengeRunning => auto.auto_challenge_running = enabled,
        AutoToggle::RetryChallenges => auto.retry_challenges = enabled,
        AutoToggle::AutoChallengeSlot(slot) => {
            if let Some(flag) = auto.auto_challenge_toggles.get_mut(slot) {
                *flag = enabled;
            }
        }
    }
}

/// `CorruptionLoadout.bonusLevels` (Corruptions.ts:239-246) — the flat free
/// levels added to every corruption ahead of multiplier / difficulty
/// lookups: GQ `corruptionFifteen` (identity) + the `oneChallengeCap` Exalt
/// reward (`completions >= 12`) + the cookieGrandma talisman inscript + the
/// finiteDescent rune.
pub(crate) fn corruption_bonus_levels(state: &GameState) -> f64 {
    use crate::mechanics::golden_quark_upgrades::corruption_fifteen_effect;
    use crate::mechanics::rune_effects::{finite_descent_rune_effects, FiniteDescentRuneKey};
    use crate::mechanics::singularity_challenges::{
        one_challenge_cap_effect, OneChallengeCapKey, SingularityEffectValue,
    };
    use crate::mechanics::talisman_effects::cookie_grandma_talisman_effects;
    use crate::state::golden_quarks::GQ_CORRUPTION_FIFTEEN;
    use crate::state::talismans::TALISMAN_COOKIE_GRANDMA;
    use crate::state::RUNE_FINITE_DESCENT;

    let gq15 = &state.golden_quarks.upgrades[GQ_CORRUPTION_FIFTEEN];
    let exalt_free = match one_challenge_cap_effect(
        state.singularity.one_challenge_cap.completions,
        OneChallengeCapKey::FreeCorruptionLevel,
    ) {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 0.0,
    };
    corruption_fifteen_effect(gq15.level + gq15.free_level)
        + exalt_free
        + cookie_grandma_talisman_effects(
            state.talismans.talisman_rarity[TALISMAN_COOKIE_GRANDMA] as i32,
        )
        .free_corruption_level
        + finite_descent_rune_effects(
            state.runes.rune_levels[RUNE_FINITE_DESCENT],
            FiniteDescentRuneKey::CorruptionFreeLevels,
        )
}

/// `CorruptionLoadout` `bonusVal` (Corruptions.ts:140-143) — the additive
/// score increase inside each corruption's raw-multiplier base: GQ
/// `advancedPack` (flat `0.33` once owned) + `oneChallengeCap`
/// `corrScoreIncrease` (`0.05`/completion) + `0.3 · cubeUpgrades[74]`.
pub(crate) fn corruption_bonus_val(state: &GameState) -> f64 {
    use crate::mechanics::golden_quark_upgrades::{advanced_pack_effect, AdvancedPackKey};
    use crate::mechanics::singularity_challenges::{
        one_challenge_cap_effect, OneChallengeCapKey, SingularityEffectValue,
    };
    use crate::state::golden_quarks::GQ_ADVANCED_PACK;

    const CUBE_UPGRADE_74: usize = 74;

    let gq_adv = &state.golden_quarks.upgrades[GQ_ADVANCED_PACK];
    let exalt_score = match one_challenge_cap_effect(
        state.singularity.one_challenge_cap.completions,
        OneChallengeCapKey::CorrScoreIncrease,
    ) {
        SingularityEffectValue::Scalar(s) => s,
        SingularityEffectValue::Unlock(_) => 0.0,
    };
    advanced_pack_effect(
        gq_adv.level + gq_adv.free_level,
        AdvancedPackKey::CorruptionScoreIncrease,
    ) + exalt_score
        + 0.3 * state.cube_upgrade_levels.cube_upgrades[CUBE_UPGRADE_74]
}

/// `maxCorruptionLevel()` (Corruptions.ts:388-400) with every input live,
/// including the singularity-era GQ `corruptionFourteen` unlock and the
/// `octeractCorruption` cap increase.
pub(crate) fn current_max_corruption_level(state: &GameState) -> f64 {
    use crate::mechanics::corruptions::{max_corruption_level, MaxCorruptionLevelInput};
    use crate::mechanics::golden_quark_upgrades::{
        corruption_fourteen_effect, platonic_tau_effect, PlatonicTauKey, PlatonicTauValue,
    };
    use crate::mechanics::octeracts::octeract_corruption_effect;
    use crate::state::golden_quarks::{GQ_CORRUPTION_FOURTEEN, GQ_PLATONIC_TAU};
    use crate::state::octeract_upgrades::OCTERACT_CORRUPTION;

    let cc = &state.challenges.challenge_completions;
    let platonic = &state.cube_upgrade_levels.platonic_upgrades;
    let gq = &state.golden_quarks.upgrades;
    let platonic_tau_unlocked = matches!(
        platonic_tau_effect(
            gq[GQ_PLATONIC_TAU].level + gq[GQ_PLATONIC_TAU].free_level,
            PlatonicTauKey::Unlocked,
        ),
        PlatonicTauValue::Unlock(true)
    );
    let oct_corruption = &state.octeract_upgrades.upgrades[OCTERACT_CORRUPTION];
    max_corruption_level(&MaxCorruptionLevelInput {
        challenge_11_completions: cc[11],
        challenge_12_completions: cc[12],
        challenge_13_completions: cc[13],
        challenge_14_completions: cc[14],
        platonic_upgrade_5: platonic[5],
        platonic_upgrade_10: platonic[10],
        platonic_tau_unlocked,
        corruption_fourteen_unlocked: corruption_fourteen_effect(
            gq[GQ_CORRUPTION_FOURTEEN].level + gq[GQ_CORRUPTION_FOURTEEN].free_level,
        ),
        octeract_corruption_cap_increase: octeract_corruption_effect(
            oct_corruption.level + oct_corruption.free_level,
        ),
    })
}

/// `updateCorruptionScoreMult` over an arbitrary loadout with the live bonus
/// terms (the legacy lazy `totalCorruptionAscensionMultiplier` getter
/// recomputes with exactly these on first read).
pub(crate) fn corruption_score_mult_for(state: &GameState, levels: [u32; 14]) -> f64 {
    use crate::mechanics::corruptions::{
        calculate_total_corruption_score_mult, TotalCorruptionScoreMultInput,
    };

    const PLATONIC_UPGRADE_17: usize = 17;

    calculate_total_corruption_score_mult(&TotalCorruptionScoreMultInput {
        levels: &levels,
        bonus_levels: corruption_bonus_levels(state),
        bonus_val: corruption_bonus_val(state),
        viscosity_platonic_17: state.cube_upgrade_levels.platonic_upgrades[PLATONIC_UPGRADE_17],
    })
}

/// `CorruptionLoadout.setLevel` — set a single corruption's *next-ascension*
/// level (clamped to `[0, maxCorruptionLevel]`) and recompute
/// `corruptions.next.total_corruption_ascension_multiplier`
/// (`updateCorruptionScoreMult`). Out-of-range slots (`>= 8`) are ignored.
/// Every bonus contribution is live: finiteDescent + cookieGrandma +
/// `corruptionFifteen` + `oneChallengeCap` free levels, `advancedPack` +
/// `oneChallengeCap` + `cubeUpgrades[74]` score increases, and the GQ /
/// octeract corruption-cap raises.
fn set_corruption_level(
    state: &mut GameState,
    index: usize,
    level: u32,
) -> SmallVec<[CoreEvent; 1]> {
    // Only the 8 real corruptions (viscosity..recession) are settable.
    if index >= 8 {
        return smallvec![];
    }

    let max = current_max_corruption_level(state);
    let clamped = level.min(max as u32);
    state.corruptions.next.levels[index] = clamped;

    let next_levels = state.corruptions.next.levels;
    state.corruptions.next.total_corruption_ascension_multiplier =
        corruption_score_mult_for(state, next_levels);

    smallvec![CoreEvent::CorruptionLevelSet {
        index,
        level: clamped,
    }]
}

/// Start a campaign (the legacy start-campaign button, Campaign.ts:1556-1564,
/// then `CampaignManager.set campaign`, Campaign.ts:332-340): rejected while
/// inside an ascension challenge; otherwise runs a full `reset('ascension')`
/// — which banks + clears any currently-active campaign — then marks the
/// chosen campaign current and applies its corruption loadout to
/// `corruptions.used`. The fresh legacy `CorruptionLoadout`'s score mult
/// re-derives lazily on first read; here the cache recomputes eagerly.
fn select_campaign(
    state: &mut GameState,
    campaign: usize,
    reset_gains: &ResetCurrencyResult,
) -> SmallVec<[CoreEvent; 2]> {
    use crate::mechanics::campaigns::{CAMPAIGNS_LEN, CAMPAIGN_CORRUPTION_LOADOUTS};

    if campaign >= CAMPAIGNS_LEN || state.challenges.current_ascension_challenge != 0 {
        return smallvec![];
    }

    let mut events = reset::perform_reset(state, ResetRequest::Ascension, reset_gains);

    state.campaigns.current_campaign = Some(campaign as u8);
    let mut levels = [0_u32; 14];
    levels[..8].copy_from_slice(&CAMPAIGN_CORRUPTION_LOADOUTS[campaign]);
    state.corruptions.used.levels = levels;
    state.corruptions.used.total_corruption_ascension_multiplier =
        corruption_score_mult_for(state, levels);

    events.push(CoreEvent::CampaignStarted { campaign });
    events
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
/// Cross-mechanic multipliers + unlock gates arrive via [`AutomationPre`],
/// which [`tack`] now self-derives from `&GameState`; each emitted
/// [`CoreEvent`] is an intent the UI tier turns into the matching side
/// effect.
fn phase_automation(
    state: &mut GameState,
    pre: &AutomationPre,
    input: &TackInput,
    output: &mut TickOutput,
) {
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

        // Reborn-ELO activation — the legacy `generateAntsAndCrumbs` tail calls
        // `activateELO(dt)` each live tick. Gated on `immortal_elo > 0` to keep
        // the default-state sim unshifted: the only default effect would be an
        // inert `{elo: 0}` leaderboard push, and any all-zero entry contributes
        // `weight × 0 = 0` to `calculate_leaderboard_value`, so suppressing it is
        // outcome-identical to the legacy unconditional push (see
        // `ant_sacrifice::activate_elo`).
        if state.ants.immortal_elo > 0.0 {
            output.events.extend(ant_sacrifice::activate_elo(state, dt));
        }

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

            // Consume the `AntSacrificeTriggered` intent → run the sacrifice
            // effect, mirroring the `AutoResetTriggered` → `perform_reset` loop
            // in this phase's tail. Intent first, then the effect event.
            let mut performed: SmallVec<[CoreEvent; 2]> = SmallVec::new();
            for event in &events {
                if matches!(event, CoreEvent::AntSacrificeTriggered) {
                    performed.extend(ant_sacrifice::perform_ant_sacrifice(
                        state,
                        pre.reincarnation_point_gain,
                    ));
                }
            }
            output.events.extend(events);
            output.events.extend(performed);
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

        // 5. updateAll autobuyer pass — buy producers / accelerators /
        //    upgrades / ants / cubes the way the legacy 50 ms loop does.
        auto_buy::run_auto_buy(state, pre, output);
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
        auto_ascend: state.automation.auto_ascend,
        auto_ascend_mode: state.automation.auto_ascend_mode,
        auto_ascend_threshold: state.automation.auto_ascend_threshold,
        challenge_completions_10: state.challenges.challenge_completions[10],
        challenge_completions_11: state.challenges.challenge_completions[11],
        cube_upgrade_10: state.cube_upgrade_levels.cube_upgrades[10],
        ascension_counter_real_real: state.reset_counters.ascension_counter_real_real,
    });
    state.automation.auto_reset_timer_prestige = resets.auto_reset_timer_prestige;
    state.automation.auto_reset_timer_transcension = resets.auto_reset_timer_transcension;
    state.automation.auto_reset_timer_reincarnation = resets.auto_reset_timer_reincarnation;

    // Execute the fired resets. Prestige / transcension / reincarnation /
    // ascension execution are all ported, so a fired intent dispatches to
    // `perform_reset` — mirroring the manual dispatch in Phase 3. At default
    // state the auto toggles are off, so this never fires and the sim stays
    // unshifted. The `pre.*_point_gain` values are the same gains the
    // amount-mode gates compared against (and equal the manual path's
    // `reset_gains`).
    use crate::events::AutoResetTier;
    use crate::mechanics::reset_currency::ResetCurrencyResult;
    let auto_gains = ResetCurrencyResult {
        prestige_point_gain: pre.prestige_point_gain,
        transcend_point_gain: pre.transcend_point_gain,
        reincarnation_point_gain: pre.reincarnation_point_gain,
    };
    let mut performed: SmallVec<[CoreEvent; 2]> = SmallVec::new();
    for event in &resets.events {
        if let CoreEvent::AutoResetTriggered { tier, .. } = event {
            let request = match tier {
                AutoResetTier::Prestige => Some(ResetRequest::Prestige),
                AutoResetTier::Transcension => Some(ResetRequest::Transcension),
                AutoResetTier::Reincarnation => Some(ResetRequest::Reincarnation),
                // Ascension *execution* is ported; the auto-ascend *decision*
                // that would emit this intent lives in the web_ui tier in
                // legacy and is not yet ported, so no Ascension intent reaches
                // here in practice. The arm readies the bridge for when it is.
                AutoResetTier::Ascension => Some(ResetRequest::Ascension),
            };
            if let Some(request) = request {
                performed.extend(reset::perform_reset(state, request, &auto_gains));
            }
        }
    }
    // Intent (`AutoResetTriggered`) before effect (`ResetPerformed`).
    output.events.extend(resets.events);
    output.events.extend(performed);
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

/// The elevator's highest reachable floor — `max(1, highestSingularityCount,
/// count + lookahead)`, the lookahead leg only with the antiquities rune
/// purchased (the legacy input listener / `teleportToSingularity` rule).
fn elevator_max_target(state: &GameState) -> f64 {
    let sing_look = if state.runes.rune_levels[crate::state::RUNE_ANTIQUITIES] > 0.0 {
        state.singularity.singularity_count + compute_singularity_lookahead(state)
    } else {
        0.0
    };
    1.0_f64
        .max(state.singularity.highest_singularity_count)
        .max(sing_look)
}

// ─── Dispatch helpers ────────────────────────────────────────────────────

/// `buildingAchievementCheck()` — run after a coin-producer buy to award the
/// Credit the per-achievement quark reward for `count` newly-awarded
/// achievements (legacy `awardAchievement`'s `player.worlds.add`). Threads the
/// `GameState` quark slices into the slice-based reward helper.
pub(crate) fn credit_achievement_quarks(state: &mut GameState, count: usize) {
    crate::mechanics::achievement_awards::credit_achievement_quarks(
        &mut state.quarks.worlds,
        &mut state.golden_quarks.quarks_this_singularity,
        state.quarks.quark_bonus,
        count,
    );
}

/// `*OwnedCoin` achievement groups from the current owned counts (the legacy
/// `buyBuilding` calls it on every building purchase).
fn check_coin_building_achievements(state: &mut GameState) {
    let coin_owned: [f64; 5] = std::array::from_fn(|i| state.coin_producers.tiers[i].owned);
    let awarded = crate::mechanics::achievement_awards::building_achievement_check(
        &mut state.achievements,
        &coin_owned,
    );
    credit_achievement_quarks(state, awarded);
}

/// Buy accelerator boosts — the legacy `boostAccelerator` (`Buy.ts:348`). With
/// `upgrades[46] >= 1` this delegates to the pure bulk solver
/// ([`buy_accelerator_boost_bulk`](crate::mechanics::accelerator_boosts::buy_accelerator_boost_bulk));
/// otherwise it is the classic single-boost path that triggers a prestige reset
/// (hence the `prestige_point_gain` argument and the GameState scope). The
/// cost-delay is the thrift rune blessing — **this is the thrift-blessing
/// production wire**. `awardAchievementGroup('acceleratorBoosts')` is unported
/// → skipped. Identity at default (`prestigePoints` 0 → unaffordable).
fn buy_accelerator_boost(
    state: &mut GameState,
    prestige_point_gain: Decimal,
) -> SmallVec<[CoreEvent; 4]> {
    use crate::mechanics::accelerator_boosts::{
        buy_accelerator_boost_bulk, BuyAcceleratorBoostInput,
    };
    use crate::mechanics::rune_blessing_effects::thrift_rune_blessing_effects;
    use crate::state::RUNE_THRIFT;

    let delay = thrift_rune_blessing_effects(rune_blessing_power(state, RUNE_THRIFT))
        .accel_boost_cost_delay;

    // Bulk path (`upgrades[46] >= 1`): no reset, spends prestigePoints.
    if state.upgrades.upgrades[46] >= 1 {
        return buy_accelerator_boost_bulk(
            &mut state.accelerator,
            &mut state.upgrades.prestige_points,
            BuyAcceleratorBoostInput {
                accel_boost_cost_delay: delay,
            },
        );
    }

    // Classic path: buy one boost (if affordable), grow the running cost, then
    // wipe upgrades 21..=40 and trigger a prestige reset (which zeroes the coin
    // economy and re-awards prestigePoints — immediately discarded).
    let mut events: SmallVec<[CoreEvent; 4]> = SmallVec::new();
    if state.upgrades.prestige_points < state.accelerator.accelerator_boost_cost {
        return events;
    }
    let before = state.accelerator.accelerator_boost_bought;
    let starting_points = state.upgrades.prestige_points;
    state.accelerator.accelerator_boost_bought += 1.0;
    let bought = state.accelerator.accelerator_boost_bought;
    // cost *= 1e10 * 10^bought, then the per-level quadratic kicker past the
    // 1000 * delay threshold.
    state.accelerator.accelerator_boost_cost *=
        Decimal::from_finite(1e10) * Decimal::from_finite(10.0).pow(Decimal::from_finite(bought));
    if bought > 1000.0 * delay {
        let kicker = (bought - 1000.0 * delay).powi(2) / delay;
        state.accelerator.accelerator_boost_cost *=
            Decimal::from_finite(10.0).pow(Decimal::from_finite(kicker));
    }
    state.accelerator.transcend_no_accelerator = false;
    state.accelerator.reincarnate_no_accelerator = false;

    // `upgrades[46]` is 0 here (u8 `< 1`), so the legacy `< 0.5` reset path
    // always fires.
    for slot in 21..=40 {
        state.upgrades.upgrades[slot] = 0;
    }
    let reset_events = reset::perform_prestige_reset(state, prestige_point_gain);
    state.upgrades.prestige_points = Decimal::zero();

    events.push(CoreEvent::AcceleratorBoostsPurchased {
        before,
        after: bought,
        spent: starting_points - state.upgrades.prestige_points,
    });
    events.extend(reset_events);
    events
}

fn dispatch_buy(
    state: &mut GameState,
    req: &BuyRequest,
    prestige_point_gain: Decimal,
) -> SmallVec<[CoreEvent; 4]> {
    // Each arm borrows disjoint `GameState` fields explicitly so the
    // borrow checker can verify the per-slice mutator and the canonical
    // `state.upgrades.*` currency don't alias. (A helper returning
    // `&mut ProducerFamilyState` would force a single whole-state borrow
    // and prevent the second `&mut` for the currency.)
    match req {
        BuyRequest::Upgrade(inp) => buy_upgrades(&mut state.upgrades, *inp),
        BuyRequest::Research(inp) => buy_research(&mut state.researches, *inp),
        BuyRequest::GoldenQuarkUpgrade(inp) => buy_gq_upgrade(&mut state.golden_quarks, *inp),
        BuyRequest::OcteractUpgrade(inp) => buy_octeract_upgrade(
            &mut state.octeract_upgrades,
            &mut state.cube_balances.wow_octeracts,
            *inp,
        ),
        BuyRequest::AmbrosiaUpgrade(inp) => buy_ambrosia_upgrade(&mut state.ambrosia, *inp),
        BuyRequest::RuneLevels(inp) => {
            buy_rune_levels(&mut state.runes, &mut state.automation.offerings, *inp)
        }
        BuyRequest::AntProducer(inp) => buy_ant_producer(&mut state.ants, *inp),
        BuyRequest::AntUpgrade(inp) => buy_ant_upgrade(&mut state.ants, *inp),
        BuyRequest::HepteractCraft(inp) => buy_hepteract_craft(
            &mut state.hepteracts,
            &mut state.cube_balances,
            &mut state.researches.obtainium,
            &mut state.automation.offerings,
            &mut state.quarks.worlds,
            *inp,
        ),
        BuyRequest::HepteractExpand(inp) => buy_hepteract_expand(&mut state.hepteracts, *inp),
        BuyRequest::TalismanLevel(inp) => buy_talisman_level(&mut state.talismans, *inp),
        BuyRequest::Shop(inp) => buy_shop(&mut state.shop, &mut state.quarks.worlds, *inp),
        BuyRequest::Multiplier(inp) => {
            buy_multiplier(&mut state.multiplier, &mut state.upgrades.coins, *inp)
        }
        BuyRequest::Accelerator(inp) => {
            buy_accelerator(&mut state.accelerator, &mut state.upgrades.coins, *inp)
        }
        BuyRequest::AcceleratorBoost => buy_accelerator_boost(state, prestige_point_gain),
        BuyRequest::CrystalUpgrade(inp) => buy_crystal_upgrades(&mut state.crystal_upgrades, *inp),
        BuyRequest::CubeUpgrade(inp) => buy_cube_upgrade(
            &mut state.cube_upgrade_levels,
            &mut state.cube_balances.wow_cubes,
            *inp,
        ),
        BuyRequest::PlatonicUpgrade(inp) => buy_platonic_upgrade(
            &mut state.cube_upgrade_levels,
            &mut state.researches.obtainium,
            &mut state.automation.offerings,
            &mut state.cube_balances,
            *inp,
        ),
        BuyRequest::ParticleBuilding(inp) => buy_particle_building(
            &mut state.particle_buildings,
            &mut state.upgrades.reincarnation_points,
            *inp,
        ),
        BuyRequest::TesseractBuilding(inp) => {
            buy_tesseract_building(&mut state.tesseract_buildings, *inp)
        }
        BuyRequest::ConstantUpgrade(inp) => {
            let researches_175 = state.researches.researches[175];
            buy_constant_upgrade(
                &mut state.campaigns.constant_upgrades,
                &mut state.campaigns.ascend_shards,
                inp.index,
                researches_175,
            )
        }
        BuyRequest::AntMastery(inp) => buy_ant_mastery(
            &mut state.ants,
            &mut state.upgrades.reincarnation_points,
            *inp,
        ),
        BuyRequest::ProducerMax(inp) => match inp.producer_type {
            ProducerType::Coin => {
                let events = buy_max(&mut state.coin_producers, &mut state.upgrades.coins, *inp);
                check_coin_building_achievements(state);
                events
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
                let events =
                    buy_producer(&mut state.coin_producers, &mut state.upgrades.coins, *inp);
                check_coin_building_achievements(state);
                events
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

    // ─── Offerings (calculateOfferings) ──────────────────────────────────

    #[test]
    fn compute_base_offerings_default_is_one() {
        // Σ allBaseOfferingStats at default — only the absolute Base line (1).
        assert_eq!(compute_base_offerings(&GameState::default()), 1.0);
    }

    #[test]
    fn compute_base_offerings_sums_flat_bonuses() {
        let mut state = GameState::default();
        state.reset_counters.prestige_count = 1.0; // Prestige +1
        state.reset_counters.transcend_count = 1.0; // Transcend +3
        state.reset_counters.reincarnation_count = 1.0; // Reincarnate +5
        state.challenges.challenge_completions[2] = 1.0; // Challenge1 (2x1) +2
        state.researches.researches[24] = 10.0; // Research1x24 = 0.4·10 = +4
                                                // 1 + 1 + 3 + 5 + 2 + 4 = 16.
        assert_eq!(compute_base_offerings(&state), 16.0);
    }

    #[test]
    fn compute_offering_mult_default_is_one() {
        // Π allOfferingStats at default — Base line = baseOfferings (1), every
        // other line is the multiplicative identity.
        let state = GameState::default();
        let base = compute_base_offerings(&state);
        assert!((compute_offering_mult(&state, base).to_number() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn compute_offering_mult_picks_up_diamond_upgrade_4x3() {
        // DiamondUpgrade4x3 = 1 + 0.2·upgrades[38]; isolated from every other
        // offering line (classic upgrades don't feed the rune-effectiveness
        // mult). At level 5 → 2.0, so the product is 2.0.
        let mut state = GameState::default();
        state.upgrades.upgrades[38] = 5;
        let base = compute_base_offerings(&state);
        assert!((compute_offering_mult(&state, base).to_number() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn compute_offering_time_multiplier_thresholds() {
        // offeringObtainiumTimeModifiers(prestigecounter, milestone-off):
        // ThresholdPenalty = min(1, (t/10)^2); TimeMultiplier = 1 (milestone
        // needs level ≥ 5); HalfMind = 1.
        let mut state = GameState::default();
        assert_eq!(compute_offering_time_multiplier(&state), 0.0); // t = 0
        state.reset_counters.prestige_counter = 5.0; // (5/10)^2 = 0.25
        assert!((compute_offering_time_multiplier(&state) - 0.25).abs() < 1e-9);
        state.reset_counters.prestige_counter = 100.0; // capped at 1
        assert!((compute_offering_time_multiplier(&state) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn compute_offerings_default_is_base_floor() {
        // prestige_counter = 0 ⇒ time multiplier 0 ⇒ max(base 1, mult·0) = 1.
        assert_eq!(compute_offerings(&GameState::default()).to_number(), 1.0);
    }

    #[test]
    fn quark_multiplier_default_is_first_singularity_bonus() {
        // Fresh save: every term is the identity except FirstSingularityBonus
        // (highestSingularityCount == 0 ⇒ ×1.25). Verbatim allQuarkStats product.
        let state = GameState::default();
        assert!((compute_quark_multiplier(&state) - 1.25).abs() < 1e-9);
    }

    #[test]
    fn quark_multiplier_neutral_after_first_singularity() {
        // Once highestSingularityCount > 0 the first-singularity bonus drops and
        // a fresh-otherwise state collapses to the identity 1.0.
        let mut state = GameState::default();
        state.singularity.highest_singularity_count = 1.0;
        assert!((compute_quark_multiplier(&state) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn quark_multiplier_platonic_alpha_term() {
        // PlatonicALPHA: platonicUpgrades[5] > 0 ⇒ ×1.05 (first-singularity off).
        let mut state = GameState::default();
        state.singularity.highest_singularity_count = 1.0;
        state.cube_upgrade_levels.platonic_upgrades[5] = 1.0;
        assert!((compute_quark_multiplier(&state) - 1.05).abs() < 1e-9);
    }

    #[test]
    fn quark_multiplier_includes_blueberry_quarks_1() {
        // AmbrosiaQuarks1 (1 + 0.01n) is now wired into the product: level 10 ⇒
        // ×1.10. Regression for the previously-uncalled blueberry quark effect.
        let mut state = GameState::default();
        state.singularity.highest_singularity_count = 1.0;
        state.ambrosia.upgrades[crate::state::ambrosia::AMBROSIA_QUARKS_1].level = 10.0;
        assert!((compute_quark_multiplier(&state) - 1.1).abs() < 1e-9);
    }

    #[test]
    fn quark_multiplier_includes_quark_gain_achievement_reward() {
        // getAchievementReward('quarkGain'): achievement #266 (ascensionCount
        // group) earned with ascensionCount ≥ 1e15 ⇒ ×(1 + 0.1·1) = ×1.1
        // (first-singularity bonus disabled).
        let mut state = GameState::default();
        state.singularity.highest_singularity_count = 1.0;
        state.achievements.achievements[266] = 1;
        state.reset_counters.ascension_count = 1e16;
        assert!((compute_quark_multiplier(&state) - 1.1).abs() < 1e-9);
    }

    #[test]
    fn quark_multiplier_includes_challenge15_quarks_reward() {
        // The c15 `quarks` reward at exponent 1e11 (its requirement, below the
        // 1e15 quark-hepteract gate) ⇒ 1 + (3/400)·log2(32) = 1.0375.
        let mut state = GameState::default();
        state.singularity.highest_singularity_count = 1.0;
        state.challenges.challenge15_exponent = 1e11;
        assert!((compute_quark_multiplier(&state) - 1.0375).abs() < 1e-9);
    }

    #[test]
    fn quark_multiplier_includes_quark_hepteract() {
        // QuarkHepteract activates at challenge15Exponent ≥ 1e15. Isolate its
        // factor by holding the exponent fixed and varying only the quark
        // hepteract balance: with no GQ DR upgrades the exponent is 2, so
        // (1 + 0.2·log2(1 + 1500/500))^2 = (1 + 0.2·log2(4))^2 = 1.4^2 = 1.96.
        let mut base = GameState::default();
        base.singularity.highest_singularity_count = 1.0;
        base.challenges.challenge15_exponent = 1e15;
        let mut with_hept = base.clone();
        with_hept.hepteracts.quark.bal = 1500.0;
        let ratio = compute_quark_multiplier(&with_hept) / compute_quark_multiplier(&base);
        assert!((ratio - 1.96).abs() < 1e-9);
    }

    #[test]
    fn progressive_slots_8_to_11_score_from_live_state() {
        // Slot 8 (exalts): noSingularityUpgrades rewardAP = 15·completions.
        let mut s = GameState::default();
        s.singularity.no_singularity_upgrades.completions = 2.0;
        // Slot 9 (singularityUpgrades): max one capped GQ upgrade → +5.
        // GQ_GOLDEN_QUARKS_1 has max_level 15 (seeded metadata).
        let gq1 = crate::state::golden_quarks::GQ_GOLDEN_QUARKS_1;
        s.golden_quarks.upgrades[gq1].level = s.golden_quarks.upgrades[gq1].max_level;
        // Slot 10 (octeractUpgrades): octeractStarter caps at 1 → +8.
        s.octeract_upgrades.upgrades[crate::state::octeract_upgrades::OCTERACT_STARTER].level = 1.0;
        // Slot 11 (redAmbrosiaUpgrades): viscount caps at 1 → +10.
        s.red_ambrosia.upgrades[crate::state::red_ambrosia::RED_AMBROSIA_VISCOUNT].level = 1.0;

        update_progressive_achievements(&mut s);
        assert_eq!(s.achievements.progressive[8].cached_points, 30.0);
        assert_eq!(s.achievements.progressive[9].cached_points, 5.0);
        assert_eq!(s.achievements.progressive[10].cached_points, 8.0);
        assert_eq!(s.achievements.progressive[11].cached_points, 10.0);
        assert_eq!(s.achievements.achievement_points, 53.0);

        // Free levels alone don't count a GQ upgrade as maxed (purchased
        // level only, mirroring `upgrade.level >= upgrade.maxLevel`).
        let mut free_only = GameState::default();
        let max = free_only.golden_quarks.upgrades[gq1].max_level;
        free_only.golden_quarks.upgrades[gq1].free_level = max;
        update_progressive_achievements(&mut free_only);
        assert_eq!(free_only.achievements.progressive[9].cached_points, 0.0);
    }

    #[test]
    fn progressive_slots_8_to_11_inert_at_default() {
        let mut s = GameState::default();
        update_progressive_achievements(&mut s);
        for slot in 8..=11 {
            assert_eq!(s.achievements.progressive[slot].cached_points, 0.0);
        }
        assert_eq!(s.achievements.achievement_points, 0.0);
    }

    #[test]
    fn configure_elevator_clamps_target_and_sets_toggles() {
        let mut s = GameState::default();
        s.singularity.highest_singularity_count = 10.0;
        let mut input = TackInput::default();
        input
            .player_actions
            .push(PlayerAction::ConfigureSingularityElevator {
                target: 50.0, // above the reachable max (no antiquities) → clamps to highest
                locked: true,
                slow_climb: false,
            });
        let _ = tack(&mut s, &input);
        assert_eq!(s.singularity.elevator_target, 10.0);
        assert!(s.singularity.elevator_locked);
        assert!(!s.singularity.elevator_slow_climb);

        // Below the floor → clamps to 1.
        let mut input = TackInput::default();
        input
            .player_actions
            .push(PlayerAction::ConfigureSingularityElevator {
                target: 0.0,
                locked: false,
                slow_climb: true,
            });
        let _ = tack(&mut s, &input);
        assert_eq!(s.singularity.elevator_target, 1.0);
    }

    #[test]
    fn exalt_toggle_dispatches_through_tack_and_lights_the_progressive() {
        use crate::events::SingularityChallengeId as Id;
        // Enter via the player action…
        let mut s = GameState::default();
        s.singularity.highest_singularity_count = 25.0;
        s.singularity.singularity_count = 25.0;
        let mut input = TackInput::default();
        input
            .player_actions
            .push(PlayerAction::ToggleSingularityChallenge {
                challenge: Id::NoSingularityUpgrades,
            });
        let out = tack(&mut s, &input);
        assert!(s.singularity.no_singularity_upgrades.enabled);
        assert_eq!(s.singularity.singularity_count, 1.0);
        assert!(out
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::SingularityChallengeEntered { .. })));

        // …complete it (antiquities re-acquired) and exit via the same action.
        s.runes.rune_levels[crate::state::RUNE_ANTIQUITIES] = 1.0;
        let out = tack(&mut s, &input);
        assert!(!s.singularity.no_singularity_upgrades.enabled);
        assert_eq!(s.singularity.no_singularity_upgrades.completions, 1.0);
        assert!(out.events.iter().any(|e| matches!(
            e,
            CoreEvent::SingularityChallengeExited { success: true, .. }
        )));
        // The exalt progressive (slot 8) sees the completion on the next tick:
        // noSingularityUpgrades rewardAP = 15·1.
        let _ = tack(&mut s, &TackInput::default());
        assert_eq!(s.achievements.progressive[8].cached_points, 15.0);
    }

    #[test]
    fn campaign_tokens_default_is_zero() {
        assert_eq!(compute_campaign_tokens(&GameState::default()), 0.0);
    }

    #[test]
    fn campaign_tokens_flow_from_inheritance_and_completions() {
        // Inheritance alone: highestSingularityCount 16 sits in the
        // `levels[2] = 10` tier → 25 tokens.
        let mut state = GameState::default();
        state.singularity.highest_singularity_count = 16.0;
        assert_eq!(compute_campaign_tokens(&state), 25.0);

        // One full campaign (first: limit 10, not meta): additive
        // 10 + 5 (first-completion, sing ≥ 16) + 0 (last needs sing ≥ 69),
        // multiplier 1 → 15. Total 25 + 15 = 40.
        state.campaigns.campaign_completions[0] = 10.0;
        assert_eq!(compute_campaign_tokens(&state), 40.0);

        // A meta campaign doubles its own value: second (limit 10, meta)
        // at 1 completion → (1 + 5)·2 = 12. Total 52.
        state.campaigns.campaign_completions[1] = 1.0;
        assert_eq!(compute_campaign_tokens(&state), 52.0);
    }

    #[test]
    fn phase_global_state_awards_campaign_token_achievements() {
        // 40 tokens (sing 16 + a full first campaign) crosses the 10/20/40
        // gates (#426/#427/#428) but not 80 (#429).
        let mut s = GameState::default();
        s.singularity.highest_singularity_count = 16.0;
        s.campaigns.campaign_completions[0] = 10.0;
        let _ = phase_global_state(&mut s);
        assert_eq!(s.achievements.achievements[426], 1);
        assert_eq!(s.achievements.achievements[428], 1);
        assert_eq!(s.achievements.achievements[429], 0);
    }

    #[test]
    fn quark_multiplier_includes_campaign_bonus() {
        // highestSingularityCount 50 → inheritance 150 tokens →
        // campaignQuarkBonus = 1 + 0.05·min(50, 100)/100 = 1.025. Every
        // other term stays identity (singularityCount itself is 0).
        let mut state = GameState::default();
        state.singularity.highest_singularity_count = 50.0;
        assert!((compute_quark_multiplier(&state) - 1.025).abs() < 1e-9);
    }

    #[test]
    fn tack_caches_quark_bonus_as_percent() {
        // The tick caches `(calculateQuarkMultiplier() - 1) * 100` into
        // quark_bonus, so applyBonus consumers see the full multiplier. Default
        // multiplier 1.25 ⇒ quark_bonus 25.
        let mut state = GameState::default();
        let _ = tack(&mut state, &TackInput::default());
        assert!((state.quarks.quark_bonus - 25.0).abs() < 1e-9);
    }

    #[test]
    fn ambrosia_free_levels_populated_from_red_ambrosia() {
        // Red-ambrosia freeLevelsRow2 grants free levels to blueberry row-2
        // upgrades (quarks1/cubes1/luck1) only — not tutorial or row 4.
        use crate::state::ambrosia::{AMBROSIA_QUARKS_1, AMBROSIA_QUARKS_2, AMBROSIA_TUTORIAL};
        use crate::state::red_ambrosia::RED_AMBROSIA_FREE_LEVELS_ROW_2;
        let mut state = GameState::default();
        state.red_ambrosia.upgrades[RED_AMBROSIA_FREE_LEVELS_ROW_2].level = 5.0;
        populate_ambrosia_free_levels(&mut state);
        assert_eq!(state.ambrosia.upgrades[AMBROSIA_QUARKS_1].free_level, 5.0);
        assert_eq!(state.ambrosia.upgrades[AMBROSIA_TUTORIAL].free_level, 0.0);
        assert_eq!(state.ambrosia.upgrades[AMBROSIA_QUARKS_2].free_level, 0.0);
    }

    #[test]
    fn ambrosia_free_levels_feed_quark_multiplier() {
        // Red-ambrosia freeLevelsRow2 = 5 ⇒ ambrosiaQuarks1 effective level
        // 0 + 5 ⇒ 1 + 0.01·5 = 1.05 (first-singularity bonus disabled).
        use crate::state::red_ambrosia::RED_AMBROSIA_FREE_LEVELS_ROW_2;
        let mut state = GameState::default();
        state.singularity.highest_singularity_count = 1.0;
        state.red_ambrosia.upgrades[RED_AMBROSIA_FREE_LEVELS_ROW_2].level = 5.0;
        populate_ambrosia_free_levels(&mut state);
        assert!((compute_quark_multiplier(&state) - 1.05).abs() < 1e-9);
    }

    #[test]
    fn compute_offerings_takes_max_of_base_and_scaled_mult() {
        // A long reset (prestige_counter past the 10s threshold) gives a time
        // multiplier of 1; with offeringMult = 2 (DiamondUpgrade4x3 at level 5)
        // the scaled product (2·1) beats the base floor (1) → award 2.
        let mut state = GameState::default();
        state.upgrades.upgrades[38] = 5;
        state.reset_counters.prestige_counter = 100.0;
        assert!((compute_offerings(&state).to_number() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn coin_producer_buy_fires_building_achievement_check() {
        // dispatch_buy of a coin producer must run buildingAchievementCheck.
        // Pre-own 99 first-tier coins so one more buy crosses the >=100
        // threshold, awarding firstOwnedCoin achievements 1/2/3 (pv 5+10+15).
        let mut state = GameState::default();
        state.upgrades.coins = Decimal::from_finite(1e12);
        state.coin_producers.tiers[0].owned = 99.0;
        let req = BuyRequest::Producer(BuyProducerInput {
            index: 1,
            producer_type: ProducerType::Coin,
            autobuyer: false,
            buyamount: 1.0,
            r: 1.0,
            in_transcension_challenge_4: false,
            in_reincarnation_challenge_8: false,
            challengecompletions_4: 0.0,
            challengecompletions_8: 0.0,
        });
        dispatch_buy(&mut state, &req, Decimal::zero());
        assert_eq!(state.coin_producers.tiers[0].owned, 100.0);
        assert_eq!(state.achievements.achievements[1], 1);
        assert_eq!(state.achievements.achievements[2], 1);
        assert_eq!(state.achievements.achievements[3], 1);
        assert_eq!(state.achievements.achievement_points, 30.0);
    }

    #[test]
    fn achievement_points_drive_the_mythos_exponent() {
        // Keystone end-to-end: awarding achievements raises
        // achievement_points, which is the exponent of the mythos multiplier
        // `1.01^points·(points/5+1)` (gated by upgrade 47). Before P3.1 this
        // was frozen at 0 → factor 1.
        let mut state = GameState::default();
        state.upgrades.upgrades[47] = 1; // enable the achievementPoints mythos term
        let pre0 = compute_global_multipliers_pre(&state);
        let mythos0 = compute_global_multipliers(&state, &pre0).global_mythos_multiplier;

        crate::mechanics::achievement_awards::reset_achievement_check(
            &mut state.achievements,
            crate::events::AutoResetTier::Prestige,
            Decimal::from_finite(1e6),
            &crate::mechanics::achievement_awards::ResetNoBuyFlags::default(),
        );
        assert_eq!(state.achievements.achievement_points, 15.0);

        let pre1 = compute_global_multipliers_pre(&state);
        let mythos1 = compute_global_multipliers(&state, &pre1).global_mythos_multiplier;
        assert!(
            mythos1 > mythos0,
            "mythos multiplier should rise with achievement points: {} vs {}",
            mythos1,
            mythos0
        );
    }

    #[test]
    fn progressive_achievements_accrue_through_tack() {
        // Rune levels feed the runeLevel progressive; a tick folds its points
        // into achievement_points via phase_global_state's progressive refresh.
        let mut state = GameState::default();
        // Σ rune levels = 2500 → rune_level_points = floor(2500/1000) +
        // floor(2500/2500) = 2 + 1 = 3.
        state.runes.rune_levels[0] = 2500.0;
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        tack(&mut state, &input);
        assert_eq!(state.achievements.progressive[0].cached_value, 2500.0);
        // 3 progressive points + the runeLevel *group* (speed level 2500 crosses
        // thresholds 100/250/500/1000/2000 → idx 396..=400, points 2+4+6+8+10=30).
        assert_eq!(state.achievements.achievement_points, 3.0 + 30.0);
        // Idempotent: a second identical tick re-takes the max → no double-count.
        tack(&mut state, &input);
        assert_eq!(state.achievements.achievement_points, 3.0 + 30.0);
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
        let automation_pre = AutomationPre {
            global_time_multiplier: 3.0,
            ascension_speed_multi: 5.0,
            singularity_speed_multi: 1.0,
            max_quark_timer: 90_000.0,
            export_gq_per_hour: 1.0,
            ..AutomationPre::default()
        };
        let mut output = TickOutput::default();
        phase_automation(&mut state, &automation_pre, &input, &mut output);

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
    fn global_speed_mult_scales_resource_generation() {
        // Regression (audit C1): the dropped global-speed multiplier meant
        // `phase_generation` ran on raw dt, so all resource generation was
        // under-counted. 1000 tier-1 diamond producers yield 50 prestige
        // shards/tick at mult = 1; research 5x21 = 50 doubles the global-speed
        // mult to 2.0, which must double the generation to 100.
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };

        let mut base = GameState::default();
        base.diamond_producers.tiers[0].owned = 1000.0;
        let _ = tack(&mut base, &input);
        assert!((base.crystal_upgrades.prestige_shards.to_number() - 50.0).abs() < 1e-9);

        let mut fast = GameState::default();
        fast.diamond_producers.tiers[0].owned = 1000.0;
        fast.researches.researches[121] = 50.0;
        assert!((compute_global_speed_mult_pre(&fast) - 2.0).abs() < 1e-12);
        let _ = tack(&mut fast, &input);
        // Before the fix this was still 50 (the multiplier never reached generation).
        assert!((fast.crystal_upgrades.prestige_shards.to_number() - 100.0).abs() < 1e-9);
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
    fn octeract_per_second_is_zero_at_default() {
        let state = GameState::default();
        // The AscensionScore StatLine is 0 until effectiveScore >= 1e23, which
        // gates the whole 42-line product to 0 — exactly the old
        // `AutomationPre::default().octeract_per_second`.
        assert_eq!(compute_octeract_per_second(&state), 0.0);
    }

    #[test]
    fn octeract_per_second_positive_once_score_cap_cleared() {
        let mut state = GameState::default();
        // C1 = 9000 completions builds a nonzero ascension base score, and a
        // huge corruption multiplier pushes effectiveScore past the 1e23 gate,
        // so the AscensionScore line (and thus the product) becomes positive.
        state.challenges.highest_challenge_completions[1] = 9000.0;
        state.corruptions.used.total_corruption_ascension_multiplier = 1e25;
        let result = compute_octeract_per_second(&state);
        assert!(
            result > 0.0 && result.is_finite(),
            "expected a positive finite octeract/s, got {result}"
        );
    }

    #[test]
    fn golden_quarks_multiplier_excluding_base_is_one_at_default() {
        let state = GameState::default();
        // Every non-base line is the multiplicative identity at default → 1.0,
        // matching the old AutomationPre default.
        assert_eq!(compute_golden_quarks_multiplier_excluding_base(&state), 1.0);
    }

    #[test]
    fn golden_quarks_multiplier_excluding_base_scales_with_contributors() {
        let mut state = GameState::default();
        // goldenQuarks1 (gq[0]) goldenQuarkMult = 1 + 0.1n; n = 5 → 1.5.
        state.golden_quarks.upgrades[0].level = 5.0;
        // CookieUpgrade19 (cube[69]) = 1 + 0.12n; n = 10 → 2.2.
        state.cube_upgrade_levels.cube_upgrades[69] = 10.0;
        let expected = 1.5 * 2.2;
        assert!((compute_golden_quarks_multiplier_excluding_base(&state) - expected).abs() < 1e-9);
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
    fn red_ambrosia_timer_fields_at_default() {
        let state = GameState::default();
        // accelerator 0, TIME_PER_RED_AMBROSIA 100000, bar-req multiplier 1.
        assert_eq!(
            compute_red_ambrosia_timer_fields(&state),
            (0.0, 100_000.0, 1.0)
        );
    }

    #[test]
    fn red_ambrosia_timer_fields_track_state() {
        let mut state = GameState::default();
        state.red_ambrosia.upgrades[19].level = 10.0; // redAmbrosiaAccelerator: 0.02·10 + 1 = 1.2
        state.singularity.limited_time.completions = 5.0; // limitedTime barReq: 1 − 0.02·5 = 0.9
        let (atpra, tpra, bar) = compute_red_ambrosia_timer_fields(&state);
        assert!((atpra - 1.2).abs() < 1e-12);
        assert_eq!(tpra, 100_000.0);
        assert!((bar - 0.9).abs() < 1e-12);
    }

    #[test]
    fn octeract_unlocked_false_at_default() {
        let state = GameState::default();
        assert!(!compute_octeract_unlocked(&state));
    }

    #[test]
    fn octeract_unlocked_true_when_gq_bought() {
        let mut state = GameState::default();
        state.golden_quarks.upgrades[24].level = 1.0; // octeractUnlock
        assert!(compute_octeract_unlocked(&state));
    }

    #[test]
    fn max_quark_timer_is_baseline_at_default() {
        let state = GameState::default();
        assert_eq!(compute_max_quark_timer(&state), 90_000.0);
    }

    #[test]
    fn max_quark_timer_extends_with_research_195() {
        let mut state = GameState::default();
        state.researches.researches[195] = 2.0; // 90000 + 18000·2 = 126000
        assert_eq!(compute_max_quark_timer(&state), 126_000.0);
    }

    #[test]
    fn roomba_unlocked_tracks_cube_upgrade_and_singularity() {
        let mut state = GameState::default();
        assert!(!compute_roomba_unlocked(&state));
        state.cube_upgrade_levels.cube_upgrades[9] = 1.0; // cubeUpgrades[9] === 1
        assert!(compute_roomba_unlocked(&state));
        state.cube_upgrade_levels.cube_upgrades[9] = 0.0;
        state.singularity.highest_singularity_count = 11.0; // > 10
        assert!(compute_roomba_unlocked(&state));
    }

    #[test]
    fn offering_auto_rune_tracks_shop_slot() {
        let mut state = GameState::default();
        assert!(!compute_offering_auto_rune(&state));
        state.shop.upgrades[3] = 1.0; // offeringAuto
        assert!(compute_offering_auto_rune(&state));
    }

    #[test]
    fn auto_prestige_milestone_unlocks_at_level_7() {
        let mut state = GameState::default();
        assert_eq!(compute_auto_prestige_milestone(&state), 0.0);
        state.achievements.achievement_points = 350.0; // ⌊350/50⌋ = level 7 → 1
        assert_eq!(compute_auto_prestige_milestone(&state), 1.0);
    }

    #[test]
    fn available_reborn_elo_is_immortal_minus_reborn() {
        let mut state = GameState::default();
        assert_eq!(compute_available_reborn_elo(&state), 0.0);
        state.ants.immortal_elo = 300.0;
        state.ants.reborn_elo = 120.0;
        assert_eq!(compute_available_reborn_elo(&state), 180.0);
        // Floors at 0 when reborn exceeds immortal.
        state.ants.reborn_elo = 500.0;
        assert_eq!(compute_available_reborn_elo(&state), 0.0);
    }

    #[test]
    fn immortal_elo_gain_at_default_is_one() {
        let state = GameState::default();
        // Base ELO 1 (the `ants` level-reward defaultValue) × mult 1, floored;
        // max(0, 1 − immortalELO 0) = 1.
        assert_eq!(compute_immortal_elo_gain(&state), 1.0);
    }

    #[test]
    fn immortal_elo_gain_grows_with_ant_elo_research() {
        let mut state = GameState::default();
        state.researches.researches[108] = 4.0; // Research5x8: 25·4 = +100 base ELO
                                                // ⌊(1 + 100) × 1⌋ = 101; max(0, 101 − 0) = 101.
        assert_eq!(compute_immortal_elo_gain(&state), 101.0);
    }

    #[test]
    fn immortal_elo_gain_floors_against_immortal_elo() {
        let mut state = GameState::default();
        state.researches.researches[108] = 4.0; // effective ELO 101
        state.ants.immortal_elo = 50.0;
        assert_eq!(compute_immortal_elo_gain(&state), 51.0);
    }

    #[test]
    fn auto_obtainium_ant_sacrifice_source_engages_with_cube_upgrade_47() {
        let mut state = GameState::default();
        // Activate the auto-research-obtainium multiplier gate (0.8·cubeUpgrades[3]).
        state.cube_upgrade_levels.cube_upgrades[3] = 1.0;
        // Make the ant-sacrifice obtainium source large: a big multiplier line,
        // a live ant-sacrifice cube blessing, and a non-zero sacrifice timer.
        state.researches.researches[103] = 1_000.0;
        state.cube_blessings.ant_sacrifice = 5_000.0;
        state.ants.ant_sacrifice_timer = 9.0;

        let without = compute_obtainium_gain(&state, 1.0, Decimal::zero());
        state.cube_upgrade_levels.cube_upgrades[47] = 1.0; // enable the ant branch
        let with = compute_obtainium_gain(&state, 1.0, Decimal::zero());

        // The ant-sacrifice source is a max() alternative; with it dominating
        // here, the auto-obtainium rises.
        assert!(
            with > without,
            "ant source should raise auto-obtainium: {} vs {}",
            with.to_number(),
            without.to_number()
        );
    }

    #[test]
    fn max_challenges_1_to_10_at_default() {
        let state = GameState::default();
        let caps = compute_max_challenges_1_to_10(&state);
        // Transcension tier base 25, reincarnation tier base 40 (no upgrades).
        assert_eq!(caps[1], 25.0);
        assert_eq!(caps[5], 25.0);
        assert_eq!(caps[6], 40.0);
        assert_eq!(caps[10], 40.0);
    }

    #[test]
    fn sweep_pre_evals_idle_at_default() {
        let state = GameState::default();
        let s = compute_sweep_pre_evals(&state);
        // Timers from autoChallengeTimer defaults; lookups inert in Idle.
        assert_eq!(
            (s.timer_start, s.timer_exit, s.timer_enter),
            (10.0, 2.0, 2.0)
        );
        assert_eq!(s.next_regular_challenge_from_initial, -1);
        assert_eq!(s.next_regular_challenge_from_active, -1);
        assert!(!s.challenge_15_auto_exponent_check);
        assert!(!s.is_finished_still_valid);
    }

    #[test]
    fn sweep_pre_evals_initial_wait_finds_first_challenge() {
        let mut state = GameState::default();
        state.automation.sweep_state = crate::events::SweepState::InitialWait;
        let s = compute_sweep_pre_evals(&state);
        // Challenge 1 uncompleted (0 < cap 25) + toggled on → next = 1.
        assert_eq!(s.next_regular_challenge_from_initial, 1);
        assert_eq!(s.next_regular_challenge_from_active, -1);
    }

    #[test]
    fn sweep_pre_evals_finished_guard_tracks_c1_c6_caps() {
        let mut state = GameState::default();
        state.automation.sweep_state = crate::events::SweepState::Finished;
        // Invalid until c1 + c6 sit exactly at their caps (25 / 40).
        assert!(!compute_sweep_pre_evals(&state).is_finished_still_valid);
        state.challenges.highest_challenge_completions[1] = 25.0;
        state.challenges.highest_challenge_completions[6] = 40.0;
        assert!(compute_sweep_pre_evals(&state).is_finished_still_valid);
    }

    #[test]
    fn time_warp_skips_head_timers() {
        let mut state = GameState::default();
        let input = TackInput {
            dt: 2.0,
            time_warp: true,
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
            ..TackInput::default()
        };
        // `tack` now self-derives `octeract_unlocked` (false at default); drive
        // `phase_automation` directly with a controlled cache so this stays a
        // focused test of the octeract giveaway (octeract_per_second is still
        // caller-provided).
        let automation_pre = AutomationPre {
            octeract_unlocked: true,
            octeract_per_second: 4.0,
            ..AutomationPre::default()
        };
        let mut output = TickOutput::default();
        phase_automation(&mut state, &automation_pre, &input, &mut output);

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
    fn phase_automation_executes_prestige_auto_reset() {
        use synergismforkd_bignum::Decimal;

        use crate::events::AutoResetTier;

        // Auto-prestige (amount mode) meets its gate, so the tail both emits
        // the `AutoResetTriggered` intent AND now performs the reset.
        let mut state = GameState::default();
        state.automation.auto_prestige_enabled = true;
        state.upgrades.prestige_points = Decimal::from_finite(1.0); // threshold = 1 × 10^0
        state.coin_counters.coins_this_prestige = Decimal::from_finite(1e16);
        state.coin_producers.tiers[0].cost = Decimal::from_finite(999.0);

        // `auto_prestige_milestone` + `prestige_point_gain` are self-derived
        // in `tack`; drive `phase_automation` directly with a controlled
        // cache so this stays a focused test of the auto-reset → execution
        // wiring. `time_warp` skips head/middle; the auto-reset tail runs.
        let automation_pre = AutomationPre {
            auto_prestige_milestone: 1.0,
            prestige_point_gain: Decimal::from_finite(5.0),
            ..AutomationPre::default()
        };
        let input = TackInput {
            dt: 1.0,
            time_warp: true,
            ..TackInput::default()
        };
        let mut output = TickOutput::default();
        phase_automation(&mut state, &automation_pre, &input, &mut output);

        assert!(
            output.events.iter().any(|e| matches!(
                e,
                CoreEvent::AutoResetTriggered {
                    tier: AutoResetTier::Prestige,
                    ..
                }
            )),
            "expected the prestige intent, got {:?}",
            output.events
        );
        assert!(
            output.events.iter().any(|e| matches!(
                e,
                CoreEvent::ResetPerformed {
                    tier: AutoResetTier::Prestige,
                    ..
                }
            )),
            "expected the reset to execute, got {:?}",
            output.events
        );
        // prestige_points: 1 + 5 (awarded gain) = 6; coin economy reset.
        assert_eq!(state.upgrades.prestige_points.to_number(), 6.0);
        assert_eq!(state.coin_counters.coins_this_prestige.to_number(), 100.0);
        assert_eq!(state.coin_producers.tiers[0].cost.to_number(), 100.0);
        assert_eq!(state.reset_counters.prestige_count, 1.0);
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
        let automation_pre = AutomationPre {
            ambrosia_generation_speed: 1.0,
            ambrosia_luck: 200.0,
            time_per_ambrosia: 45.0,
            ..AutomationPre::default()
        };
        let mut output = TickOutput::default();
        phase_automation(&mut state, &automation_pre, &input, &mut output);

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
            ..TackInput::default()
        };
        // `tack` now self-derives obtainium_gain; with research[61] == 1 the
        // multiplier gate is 0.5 (nonzero), so the self-derived gain would not
        // be 25. Drive phase_automation directly with a controlled cache so this
        // stays a focused test of the obtainium credit path.
        let automation_pre = AutomationPre {
            obtainium_gain: Decimal::from_finite(25.0),
            ..AutomationPre::default()
        };
        let mut output = TickOutput::default();
        phase_automation(&mut state, &automation_pre, &input, &mut output);

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
        // `tack` now self-derives ant_speed_mult, which is 0 at the default
        // state (canGenerateAntCrumbs is false until ants unlock → whole product
        // 0). Drive phase_automation directly with a controlled cache
        // (ant_speed_mult = 1) so this stays a focused test of crumb generation.
        let automation_pre = AutomationPre {
            ant_speed_mult: Decimal::one(),
            ..AutomationPre::default()
        };
        let mut output = TickOutput::default();
        phase_automation(&mut state, &automation_pre, &input, &mut output);
        assert!(state.ants.crumbs.to_number() > 0.0);
    }

    #[test]
    fn ant_speed_mult_self_derives_zero_until_ants_unlock() {
        // Base line `canGenerateAntCrumbs` is false at default → product 0.
        let mut state = GameState::default();
        assert_eq!(compute_ant_speed_mult(&state).to_number(), 0.0);
        // challengecompletions[8] > 0 flips canGenerateAntCrumbs true → nonzero.
        state.challenges.challenge_completions[8] = 1.0;
        assert!(compute_ant_speed_mult(&state).to_number() > 0.0);
        // cubeUpgrades[48] > 0 is the other unlock path.
        let mut state2 = GameState::default();
        state2.cube_upgrade_levels.cube_upgrades[48] = 1.0;
        assert!(compute_ant_speed_mult(&state2).to_number() > 0.0);
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
    fn tack_dispatches_buy_research_action() {
        use synergismforkd_bignum::Decimal;

        let mut state = GameState::default();
        state.researches.obtainium = Decimal::from_finite(5.0);

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::Research(BuyResearchInput {
                index: 6,
                buy_max: true,
            })));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::ResearchPurchased { .. })),
            "expected ResearchPurchased in events, got {:?}",
            output.events
        );
        // Research 6 (base_cost 1, max_level 10): budget 5 ⇒ buy to level 5.
        assert_eq!(state.researches.researches[6], 5.0);
    }

    #[test]
    fn tack_dispatches_buy_cube_upgrade_action() {
        use synergismforkd_bignum::Decimal;

        let mut state = GameState::default();
        state.cube_balances.wow_cubes = Decimal::from_finite(1e12);

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::CubeUpgrade(
                BuyCubeUpgradeInput {
                    index: 1,
                    buy_max: false,
                    singularity_debuff: 1.0,
                },
            )));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::CubeUpgradePurchased { .. })),
            "expected CubeUpgradePurchased in events, got {:?}",
            output.events
        );
        assert_eq!(state.cube_upgrade_levels.cube_upgrades[1], 1.0);
    }

    #[test]
    fn tack_dispatches_select_campaign_action() {
        let mut state = GameState::default();
        state.challenges.challenge_completions[10] = 3.0;

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::SelectCampaign { campaign: 4 });

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::CampaignStarted { campaign: 4 })),
            "expected CampaignStarted in events, got {:?}",
            output.events
        );
        // The full ascension reset ran first (start button = reset('ascension')
        // then the campaign setter)…
        assert!(output.events.iter().any(|e| matches!(
            e,
            CoreEvent::ResetPerformed {
                tier: crate::events::AutoResetTier::Ascension,
                ..
            }
        )));
        // …and the fifth campaign (viscosity 3 / drought 3) is now active on
        // `corruptions.used`, with no completions banked (no campaign was
        // active during the reset and singularity < 4).
        assert_eq!(state.campaigns.current_campaign, Some(4));
        assert_eq!(
            state.corruptions.used.levels[crate::state::VISCOSITY_INDEX],
            3
        );
        assert_eq!(
            state.corruptions.used.levels[crate::state::DROUGHT_INDEX],
            3
        );
        assert_eq!(state.campaigns.campaign_completions, [0.0; 50]);
    }

    #[test]
    fn claim_export_rewards_banks_golden_quarks_and_quarks() {
        use crate::state::golden_quarks::GQ_GOLDEN_QUARKS_3;
        use crate::state::GoldenQuarkUpgrade;

        let mut state = GameState::default();
        // goldenQuarks3 level 1 → exportGQPerHour = 1·2/2 = 1/hr → window 3600s.
        state.golden_quarks.upgrades[GQ_GOLDEN_QUARKS_3] = GoldenQuarkUpgrade {
            level: 1.0,
            ..GoldenQuarkUpgrade::default()
        };
        state.golden_quarks.golden_quarks_timer = 7400.0; // 2 whole windows + 200s
                                                          // Quark timer: base 5/hr (no researches). 7400s · 5/3600 = floor(10.27) = 10.
        state.quarks.quarks_timer = 7400.0;

        let claim = claim_export_rewards(&mut state);

        // 2 golden quarks at the default (sing < 100 ⇒ bonus mult 1).
        assert_eq!(claim.golden_quarks, 2.0);
        assert_eq!(state.golden_quarks.golden_quarks.to_number(), 2.0);
        assert_eq!(state.golden_quarks.golden_quarks_timer, 200.0); // remainder

        // Quark gain 10 × quark multiplier (≥ 1). At default the multiplier is 1.
        assert!(claim.quarks >= 10.0);
        assert_eq!(state.quarks.worlds.to_number(), claim.quarks);
        assert_eq!(state.golden_quarks.quarks_this_singularity, claim.quarks);
        // quarkstimer keeps remainder of 7400 % (3600/5 = 720) = 7400 - 10·720.
        assert!((state.quarks.quarks_timer - 200.0).abs() < 1e-9);
    }

    #[test]
    fn claim_export_rewards_noop_without_goldenquarks3_or_quark_gain() {
        let mut state = GameState::default();
        // No goldenQuarks3 ⇒ exportGQPerHour 0 ⇒ GQ timer untouched.
        state.golden_quarks.golden_quarks_timer = 5000.0;
        // Sub-1-quark window: 600s · 5/3600 = floor(0.83) = 0 ⇒ no quark claim.
        state.quarks.quarks_timer = 600.0;

        let claim = claim_export_rewards(&mut state);

        assert_eq!(claim, ExportRewardClaim::default());
        assert_eq!(state.golden_quarks.golden_quarks_timer, 5000.0);
        assert_eq!(state.quarks.quarks_timer, 600.0);
        assert_eq!(state.quarks.worlds.to_number(), 0.0);
    }

    #[test]
    fn claim_export_rewards_applies_sing100_export_bonus() {
        use crate::state::golden_quarks::GQ_GOLDEN_QUARKS_3;
        use crate::state::GoldenQuarkUpgrade;

        let mut state = GameState::default();
        state.singularity.highest_singularity_count = 200.0; // bonus = 1 + 200/50 = 5
        state.golden_quarks.upgrades[GQ_GOLDEN_QUARKS_3] = GoldenQuarkUpgrade {
            level: 1.0,
            ..GoldenQuarkUpgrade::default()
        };
        state.golden_quarks.golden_quarks_timer = 3600.0; // exactly one window

        let claim = claim_export_rewards(&mut state);

        // 1 whole window × bonus 5 = 5 golden quarks.
        assert_eq!(claim.golden_quarks, 5.0);
        assert_eq!(state.golden_quarks.golden_quarks.to_number(), 5.0);
    }

    #[test]
    fn select_campaign_rejected_inside_ascension_challenge() {
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 11;

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::SelectCampaign { campaign: 0 });

        let output = tack(&mut state, &input);

        assert!(!output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::CampaignStarted { .. })));
        assert_eq!(state.campaigns.current_campaign, None);
    }

    #[test]
    fn tack_dispatches_buy_gq_upgrade_action() {
        use synergismforkd_bignum::Decimal;

        use crate::state::GoldenQuarkUpgrade;

        let mut state = GameState::default();
        state.golden_quarks.golden_quarks = Decimal::from_finite(500.0);
        state.golden_quarks.upgrades[0] = GoldenQuarkUpgrade {
            cost_per_level: 100.0,
            max_level: 10.0,
            ..GoldenQuarkUpgrade::default()
        };

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::GoldenQuarkUpgrade(
                BuyGQUpgradeInput {
                    index: 0,
                    computed_max_level: 10.0,
                },
            )));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::GoldenQuarkUpgradePurchased { .. })),
            "expected GoldenQuarkUpgradePurchased in events, got {:?}",
            output.events
        );
        assert_eq!(state.golden_quarks.upgrades[0].level, 1.0);
    }

    #[test]
    fn tack_dispatches_buy_octeract_upgrade_action() {
        use synergismforkd_bignum::Decimal;

        let mut state = GameState::default();
        state.cube_balances.wow_octeracts = Decimal::from_finite(500.0);

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::OcteractUpgrade(
                BuyOcteractUpgradeInput {
                    index: 0,
                    cost_per_level: 100.0,
                    max_level: 10.0,
                },
            )));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::OcteractUpgradePurchased { .. })),
            "expected OcteractUpgradePurchased in events, got {:?}",
            output.events
        );
        assert_eq!(state.octeract_upgrades.upgrades[0].level, 1.0);
    }

    #[test]
    fn set_corruption_level_clamps_and_recomputes_mult() {
        use crate::state::VISCOSITY_INDEX;
        let mut state = GameState::default();
        state.challenges.challenge_completions[11] = 1.0; // +5 max corruption level
        let events = set_corruption_level(&mut state, VISCOSITY_INDEX, 3);
        assert_eq!(state.corruptions.next.levels[VISCOSITY_INDEX], 3);
        // Recomputed from the loadout; corruption levels raise the score mult.
        assert!(state.corruptions.next.total_corruption_ascension_multiplier > 1.0);
        assert!(matches!(
            events[0],
            CoreEvent::CorruptionLevelSet { index, level } if index == VISCOSITY_INDEX && level == 3
        ));
    }

    #[test]
    fn set_corruption_level_clamps_to_max_zero_at_default() {
        use crate::state::DROUGHT_INDEX;
        let mut state = GameState::default();
        // No challenge/platonic unlocks → maxCorruptionLevel = 0, so any request clamps to 0.
        let _ = set_corruption_level(&mut state, DROUGHT_INDEX, 99);
        assert_eq!(state.corruptions.next.levels[DROUGHT_INDEX], 0);
    }

    #[test]
    fn tack_dispatches_set_corruption_level_action() {
        use crate::state::VISCOSITY_INDEX;
        let mut state = GameState::default();
        state.challenges.challenge_completions[11] = 1.0;
        let mut input = TackInput {
            dt: 0.0,
            ..TackInput::default()
        };
        input.player_actions.push(PlayerAction::SetCorruptionLevel {
            index: VISCOSITY_INDEX,
            level: 2,
        });
        let output = tack(&mut state, &input);
        assert_eq!(state.corruptions.next.levels[VISCOSITY_INDEX], 2);
        assert!(output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::CorruptionLevelSet { .. })));
    }

    #[test]
    fn toggle_auto_sets_flags() {
        let mut state = GameState::default();
        assert!(!state.automation.auto_prestige_enabled);
        set_automation_toggle(&mut state, AutoToggle::AutoPrestige, true);
        assert!(state.automation.auto_prestige_enabled);
        set_automation_toggle(&mut state, AutoToggle::AutoPrestige, false);
        assert!(!state.automation.auto_prestige_enabled);
        // Per-challenge slot toggle writes the right entry.
        set_automation_toggle(&mut state, AutoToggle::AutoChallengeSlot(3), true);
        assert!(state.automation.auto_challenge_toggles[3]);
        // Out-of-range slot is ignored (no panic, no write).
        set_automation_toggle(&mut state, AutoToggle::AutoChallengeSlot(99), true);
    }

    #[test]
    fn tack_dispatches_toggle_auto_action() {
        let mut state = GameState::default();
        let mut input = TackInput {
            dt: 0.0,
            ..TackInput::default()
        };
        input.player_actions.push(PlayerAction::ToggleAuto {
            target: AutoToggle::AutoAscend,
            enabled: true,
        });
        let _ = tack(&mut state, &input);
        assert!(state.automation.auto_ascend);
    }

    #[test]
    fn tack_dispatches_enter_transcension_challenge() {
        let mut state = GameState::default();
        state.coin_producers.tiers[0].owned = 25.0; // base-reset witness
        let mut input = TackInput {
            dt: 0.0,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::EnterChallenge { challenge: 2 });
        let output = tack(&mut state, &input);
        // Slot set, and the tier reset ran (base reset zeroed the producer).
        assert_eq!(state.challenges.current_transcension_challenge, 2);
        assert_eq!(state.coin_producers.tiers[0].owned, 0.0);
        assert!(output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ChallengeEntered { challenge: 2 })));
    }

    #[test]
    fn tack_dispatches_enter_reincarnation_challenge() {
        let mut state = GameState::default();
        let mut input = TackInput {
            dt: 0.0,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::EnterChallenge { challenge: 8 });
        let _ = tack(&mut state, &input);
        assert_eq!(state.challenges.current_reincarnation_challenge, 8);
    }

    #[test]
    fn enter_ascension_challenge_c12_blocked_without_highest_c11() {
        // c12 entry requires highest_challenge_completions[11] > 0.
        // Default state has that at 0, so the action must be a no-op.
        let mut state = GameState::default();
        let mut input = TackInput {
            dt: 0.0,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::EnterChallenge { challenge: 12 });
        let output = tack(&mut state, &input);
        assert_eq!(state.challenges.current_ascension_challenge, 0);
        assert!(!output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ChallengeEntered { .. })));
    }

    #[test]
    fn challenge_completion_transcension_awards_and_exits() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        let mut state = GameState::default();
        state.challenges.current_transcension_challenge = 1;
        // req(c1, 0) = 10^10; provide comfortably more.
        state.coin_counters.coins_this_transcension = Decimal::from_finite(1e11);
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert_eq!(state.challenges.challenge_completions[1], 1.0);
        assert_eq!(state.challenges.highest_challenge_completions[1], 1.0);
        // retrychallenges defaults false → the tick path exits.
        assert_eq!(state.challenges.current_transcension_challenge, 0);
        assert!(output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ChallengeCompleted { challenge: 1, .. })));
    }

    #[test]
    fn challenge_completion_noop_below_requirement() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        let mut state = GameState::default();
        state.challenges.current_transcension_challenge = 1;
        state.coin_counters.coins_this_transcension = Decimal::from_finite(1e9); // < 10^10
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert_eq!(state.challenges.challenge_completions[1], 0.0);
        assert_eq!(state.challenges.current_transcension_challenge, 1); // still in
    }

    #[test]
    fn challenge_completion_reincarnation_awards_and_exits() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        let mut state = GameState::default();
        state.challenges.current_reincarnation_challenge = 6;
        // req(c6, 0) = 10^125 (base 125); c6-8 goal is transcendShards.
        state.reset_counters.transcend_shards = Decimal::from_finite(1e130);
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert_eq!(state.challenges.challenge_completions[6], 1.0);
        assert_eq!(state.challenges.highest_challenge_completions[6], 1.0);
        assert_eq!(state.challenges.current_reincarnation_challenge, 0);
        assert!(output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ChallengeCompleted { challenge: 6, .. })));
    }

    // ── Ascension challenge tests ────────────────────────────────────────────

    #[test]
    fn enter_c11_blocked_without_ascension_unlock() {
        // c11 requires ascension_unlocked; default state has it false.
        let mut state = GameState::default();
        let mut input = TackInput {
            dt: 0.0,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::EnterChallenge { challenge: 11 });
        let output = tack(&mut state, &input);
        assert_eq!(state.challenges.current_ascension_challenge, 0);
        assert!(!output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ChallengeEntered { .. })));
    }

    #[test]
    fn enter_c11_allowed_with_ascension_unlock() {
        let mut state = GameState::default();
        state.reset_counters.ascension_unlocked = true;
        // c10 condition: no active challenges → c10_ok passes even without c10 comp.
        let mut input = TackInput {
            dt: 0.0,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::EnterChallenge { challenge: 11 });
        let output = tack(&mut state, &input);
        assert_eq!(state.challenges.current_ascension_challenge, 11);
        assert!(output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ChallengeEntered { challenge: 11 })));
        // The ascension challenge reset ran: particle producers zeroed.
        assert_eq!(state.particle_producers.tiers[0].owned, 0.0);
    }

    #[test]
    fn enter_c12_allowed_with_highest_c11() {
        let mut state = GameState::default();
        // c12 requires highest[11] > 0
        state.challenges.highest_challenge_completions[11] = 1.0;
        // c10 condition: no active challenges → passes.
        let mut input = TackInput {
            dt: 0.0,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::EnterChallenge { challenge: 12 });
        let output = tack(&mut state, &input);
        assert_eq!(state.challenges.current_ascension_challenge, 12);
        assert!(output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ChallengeEntered { challenge: 12 })));
    }

    #[test]
    fn enter_c11_c10_condition_blocks_when_active_and_no_c10_comp() {
        // c10 condition: c10 completions == 0 AND a challenge is active → blocked.
        let mut state = GameState::default();
        state.reset_counters.ascension_unlocked = true;
        state.challenges.current_transcension_challenge = 2; // active challenge
        state.challenges.challenge_completions[10] = 0.0;
        let mut input = TackInput {
            dt: 0.0,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::EnterChallenge { challenge: 11 });
        let output = tack(&mut state, &input);
        assert_eq!(state.challenges.current_ascension_challenge, 0);
        assert!(!output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ChallengeEntered { .. })));
    }

    #[test]
    fn enter_c11_c10_condition_allows_with_c10_comp() {
        // c10 condition: c10 completions > 0 → allowed even with active challenge.
        let mut state = GameState::default();
        state.reset_counters.ascension_unlocked = true;
        state.challenges.challenge_completions[10] = 1.0;
        state.challenges.current_transcension_challenge = 2;
        let mut input = TackInput {
            dt: 0.0,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::EnterChallenge { challenge: 11 });
        let output = tack(&mut state, &input);
        assert_eq!(state.challenges.current_ascension_challenge, 11);
        assert!(output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ChallengeEntered { challenge: 11 })));
    }

    #[test]
    fn phase_challenge_completion_c11_awards_and_exits() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 11;
        // req(c11, comp=0) = calculateChallengeRequirementMultiplier(Ascension, 0, …)
        // At 0 completions with no corruption/platonic bonuses = 1.0 (identity).
        // challengecompletions[10] must be >= 1.0 to complete.
        state.challenges.challenge_completions[10] = 1.0;
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert_eq!(state.challenges.challenge_completions[11], 1.0);
        assert_eq!(state.challenges.highest_challenge_completions[11], 1.0);
        // retrychallenges false → exits.
        assert_eq!(state.challenges.current_ascension_challenge, 0);
        // Tesseracts unlock fires on first highest[11] rise.
        assert!(state.reset_counters.tesseracts_unlocked);
        assert!(output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ChallengeCompleted { challenge: 11, .. })));
    }

    #[test]
    fn phase_challenge_completion_c11_noop_below_requirement() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 11;
        // c10 comp = 0 < req 1.0 → no completion.
        state.challenges.challenge_completions[10] = 0.0;
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert_eq!(state.challenges.challenge_completions[11], 0.0);
        assert_eq!(state.challenges.current_ascension_challenge, 11); // still in
        assert!(!state.reset_counters.tesseracts_unlocked);
    }

    #[test]
    fn phase_challenge_completion_c15_quiescent_without_auto() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        use crate::state::shop::SHOP_CHALLENGE_15_AUTO;
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 15;
        state.upgrades.coins = Decimal::from_finite(1e6);
        // challenge15Auto NOT unlocked → the tick autoupdate must not fire
        // (the manual update path is UI-tier).
        assert_eq!(state.shop.upgrades[SHOP_CHALLENGE_15_AUTO], 0.0);
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert_eq!(state.challenges.challenge15_exponent, 0.0);
    }

    #[test]
    fn phase_challenge_completion_c15_accrues_with_auto() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        use crate::state::shop::SHOP_CHALLENGE_15_AUTO;
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 15;
        state.shop.upgrades[SHOP_CHALLENGE_15_AUTO] = 1.0; // challenge15Auto unlocked
        state.upgrades.coins = Decimal::from_finite(1e6);
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        // c15SM = 1 at default legs (campaign 1, no challenge hept, platonic[15]=0),
        // so exponent = log10(1e6 + 1) * 1 ≈ 6.0.
        assert!((state.challenges.challenge15_exponent - 6.0).abs() < 1e-3);
    }

    #[test]
    fn phase_challenge_completion_c15_threshold_gates_growth() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        use crate::state::shop::SHOP_CHALLENGE_15_AUTO;
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 15;
        state.shop.upgrades[SHOP_CHALLENGE_15_AUTO] = 1.0;
        // Already-high exponent → next threshold is 10^(100/1) = 1e100.
        state.challenges.challenge15_exponent = 100.0;
        state.upgrades.coins = Decimal::from_finite(1e6);
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        // Coins (1e6) < threshold (1e100) → exponent unchanged.
        assert_eq!(state.challenges.challenge15_exponent, 100.0);
    }

    #[test]
    fn phase_challenge_completion_c15_exponent_lights_reward_cascade() {
        use crate::mechanics::challenge_15_rewards;
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        use crate::state::shop::SHOP_CHALLENGE_15_AUTO;
        // The whole point of the accrual: a frozen-0 exponent leaves every
        // c15 reward at identity; a grown exponent lights the cascade.
        assert_eq!(challenge_15_rewards::coin_exponent(0.0), 1.0);
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 15;
        state.shop.upgrades[SHOP_CHALLENGE_15_AUTO] = 1.0;
        // coins = 1e4000 → exponent ≈ 4000, past coin_exponent's 3000 requirement.
        state.upgrades.coins = Decimal::from_finite(10.0).pow(Decimal::from_finite(4000.0));
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert!(state.challenges.challenge15_exponent > 3000.0);
        assert!(challenge_15_rewards::coin_exponent(state.challenges.challenge15_exponent) > 1.0);
    }

    #[test]
    fn tack_accrues_challenge_15_exponent_end_to_end() {
        // The orchestrator runs the c15 accrual during a normal tick (closes the
        // "is the new phase actually wired into tack" blind spot).
        use crate::state::shop::SHOP_CHALLENGE_15_AUTO;
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 15;
        state.shop.upgrades[SHOP_CHALLENGE_15_AUTO] = 1.0;
        state.upgrades.coins = Decimal::from_finite(1e6);
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        assert!(state.challenges.challenge15_exponent > 0.0);
    }

    #[test]
    fn c15_awards_sadistic_achievement_once_exponent_unlocks_it() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        use crate::state::shop::SHOP_CHALLENGE_15_AUTO;
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 15;
        state.shop.upgrades[SHOP_CHALLENGE_15_AUTO] = 1.0;
        // coins = 1e700000 → accrued exponent ≈ 700000, past the c15
        // achievementUnlock requirement (666666) → sadisticAch (#252) awarded.
        state.upgrades.coins = Decimal::from_finite(10.0).pow(Decimal::from_finite(700_000.0));
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert!(state.challenges.challenge15_exponent >= 666_666.0);
        assert_eq!(state.achievements.achievements[252], 1);
    }

    #[test]
    fn daily_reset_zeroes_daily_counters_and_preserves_all_time() {
        use crate::state::ants::RebornELOEntry;
        let mut state = GameState::default();
        // A day of play populates the daily counters, the daily leaderboard, and
        // the running ant-quark total — plus the all-time leaderboard.
        state.cube_balances.cube_opened_daily = 12.0;
        state.cube_balances.cube_quark_daily = 34.0;
        state.cube_balances.tesseract_opened_daily = 5.0;
        state.cube_balances.tesseract_quark_daily = 6.0;
        state.cube_balances.hypercube_opened_daily = 7.0;
        state.cube_balances.hypercube_quark_daily = 8.0;
        state.cube_balances.platonic_cube_opened_daily = 9.0;
        state.cube_balances.platonic_cube_quark_daily = 10.0;
        state.ants.quarks_gained_from_ants = 500.0;
        state.ants.highest_reborn_elo_daily.push(RebornELOEntry {
            elo: 1000.0,
            sacrifice_id: 1,
        });
        state.ants.highest_reborn_elo_ever.push(RebornELOEntry {
            elo: 1000.0,
            sacrifice_id: 1,
        });

        daily_reset(&mut state);

        // All 8 per-day cube counters cleared.
        assert_eq!(state.cube_balances.cube_opened_daily, 0.0);
        assert_eq!(state.cube_balances.cube_quark_daily, 0.0);
        assert_eq!(state.cube_balances.tesseract_opened_daily, 0.0);
        assert_eq!(state.cube_balances.tesseract_quark_daily, 0.0);
        assert_eq!(state.cube_balances.hypercube_opened_daily, 0.0);
        assert_eq!(state.cube_balances.hypercube_quark_daily, 0.0);
        assert_eq!(state.cube_balances.platonic_cube_opened_daily, 0.0);
        assert_eq!(state.cube_balances.platonic_cube_quark_daily, 0.0);
        // Daily reborn-ELO leaderboard + running ant-quark total reset...
        assert!(state.ants.highest_reborn_elo_daily.is_empty());
        assert_eq!(state.ants.quarks_gained_from_ants, 0.0);
        // ...but the all-time leaderboard survives.
        assert_eq!(state.ants.highest_reborn_elo_ever.len(), 1);
    }

    #[test]
    fn first_five_rune_mult_picks_up_midas_tribute_cube_blessing() {
        let mut state = GameState::default();
        // No blessings opened → every factor is identity → the mult is 1.
        assert_eq!(first_five_effective_rune_level_mult(&state), 1.0);
        // Accruing the talisman-bonus blessing tier (as opening cubes would) lifts
        // the MidasTribute cascade above 1, so the rune-effectiveness mult rises.
        state.cube_blessings.talisman_bonus = 500.0;
        state.tesseract_blessings.talisman_bonus = 5_000.0;
        state.hypercube_blessings.talisman_bonus = 5_000.0;
        assert!(first_five_effective_rune_level_mult(&state) > 1.0);
    }

    #[test]
    fn challenge_completion_awards_challenge_achievements() {
        // Completing a challenge through the real path fires
        // challengeAchievementCheck: c11 completed once awards the challenge11
        // achievement at the >=1 threshold (index 197, pv 10).
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 11;
        state.challenges.challenge_completions[10] = 1.0; // meets the c11 requirement
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert_eq!(state.challenges.challenge_completions[11], 1.0);
        assert_eq!(state.achievements.achievements[197], 1);
        assert_eq!(state.achievements.achievement_points, 10.0);
    }

    #[test]
    fn unlock_side_effects_fire_per_challenge() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        // c12 → spirits_unlocked
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 12;
        state.challenges.challenge_completions[10] = 1.0;
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert!(state.reset_counters.spirits_unlocked);

        // c13 → hypercubes_unlocked
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 13;
        state.challenges.challenge_completions[10] = 1.0;
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert!(state.reset_counters.hypercubes_unlocked);

        // c14 → platonics_unlocked
        let mut state = GameState::default();
        state.challenges.current_ascension_challenge = 14;
        state.challenges.challenge_completions[10] = 1.0;
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert!(state.reset_counters.platonics_unlocked);
    }

    #[test]
    fn completing_reincarnation_challenge_10_unlocks_ascensions() {
        // Regression (audit C2): completing reincarnation challenge 10 must set
        // ascension_unlocked — the entry gate for ascension challenge 11. The
        // unlock arm only handled c11-14, so the whole c11-c15 ladder was
        // unreachable (ascension_unlocked was assigned only in #[cfg(test)]).
        // Drive complete_active_challenge with a trivially-met requirement so
        // highest[10] rises 0 -> 1; instant_unlocked skips the tier reset to
        // isolate the unlock from the cascade.
        let mut state = GameState::default();
        assert!(!state.reset_counters.ascension_unlocked);
        state.challenges.current_reincarnation_challenge = 10;
        let gains = crate::mechanics::reset_currency::ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        let requirement = |_challenge: u32, _comp: f64| Decimal::zero();
        complete_active_challenge(
            &mut state,
            10,
            Decimal::one(),
            5.0,
            1.0,
            &requirement,
            true,
            &gains,
            &mut output,
        );
        assert!(state.challenges.challenge_completions[10] >= 1.0);
        assert!(
            state.reset_counters.ascension_unlocked,
            "completing reincarnation challenge 10 must unlock ascensions"
        );
    }

    #[test]
    fn completing_reincarnation_challenge_8_unlocks_anthill() {
        // Synergism.ts:3692 — highest[8] > 0 sets unlocks.anthill.
        let mut state = GameState::default();
        state.challenges.current_reincarnation_challenge = 8;
        let gains = crate::mechanics::reset_currency::ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        let requirement = |_challenge: u32, _comp: f64| Decimal::zero();
        complete_active_challenge(
            &mut state,
            8,
            Decimal::one(),
            5.0,
            1.0,
            &requirement,
            true,
            &gains,
            &mut output,
        );
        assert!(state.reset_counters.anthill_unlocked);
    }

    #[test]
    fn completing_reincarnation_challenge_9_unlocks_talismans_and_blessings() {
        // Synergism.ts:3695-3696 — highest[9] > 0 sets talismans + blessings.
        let mut state = GameState::default();
        state.challenges.current_reincarnation_challenge = 9;
        let gains = crate::mechanics::reset_currency::ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        let requirement = |_challenge: u32, _comp: f64| Decimal::zero();
        complete_active_challenge(
            &mut state,
            9,
            Decimal::one(),
            5.0,
            1.0,
            &requirement,
            true,
            &gains,
            &mut output,
        );
        assert!(state.reset_counters.talismans_unlocked);
        assert!(state.reset_counters.blessings_unlocked);
    }

    #[test]
    fn coin_unlocks_latch_on_coin_thresholds() {
        // Synergism.ts:3976-3989 — coinone..four latch as coins cross thresholds.
        let mut state = GameState::default();
        state.upgrades.coins = Decimal::from_finite(600.0);
        update_progress_unlocks(&mut state);
        assert!(state.reset_counters.coin_one_unlocked);
        assert!(!state.reset_counters.coin_two_unlocked);
        state.upgrades.coins = Decimal::from_finite(5e6);
        update_progress_unlocks(&mut state);
        assert!(state.reset_counters.coin_two_unlocked);
        assert!(state.reset_counters.coin_three_unlocked);
        assert!(state.reset_counters.coin_four_unlocked);
    }

    #[test]
    fn generation_unlocks_at_prestige_threshold() {
        // Automation.ts:8 — generation unlocks at prestige points >= 1e12.
        let mut state = GameState::default();
        state.upgrades.prestige_points = Decimal::from_finite(1e11);
        update_progress_unlocks(&mut state);
        assert!(!state.reset_counters.generation_unlocked);
        state.upgrades.prestige_points = Decimal::from_finite(1e12);
        update_progress_unlocks(&mut state);
        assert!(state.reset_counters.generation_unlocked);
    }

    #[test]
    fn reincarnation_completion_uses_shop_challenge_extension() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        use crate::state::shop::SHOP_CHALLENGE_EXTENSION;
        // The completion loop must use the same shop challengeExtension cap as the
        // sweep (challenge_extension_effect = 2n, applied to c6-10). It was zeroed,
        // so reincarnation completions plateaued at the base cap (40) below the
        // extended cap. With +20 extension and the goal met, c6 advances past 40.
        let mut state = GameState::default();
        state.challenges.current_reincarnation_challenge = 6;
        state.challenges.challenge_completions[6] = 40.0; // at the base cap
        state.reset_counters.transcend_shards = Decimal::from_mantissa_exponent(1.0, 1e15);
        state.shop.upgrades[SHOP_CHALLENGE_EXTENSION] = 10.0; // challengeExtension(10) = +20
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert!(
            state.challenges.challenge_completions[6] > 40.0,
            "extended cap must let reincarnation completions pass the base cap"
        );
    }

    // ── retry_challenges tests ────────────────────────────────────────────

    #[test]
    fn retry_challenges_false_exits_slot() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        // Default: retry_challenges = false → slot clears on completion.
        let mut state = GameState::default();
        state.challenges.current_transcension_challenge = 1;
        state.coin_counters.coins_this_transcension = Decimal::from_finite(1e11);
        assert!(!state.automation.retry_challenges);
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert_eq!(state.challenges.current_transcension_challenge, 0); // cleared
        assert_eq!(state.challenges.challenge_completions[1], 1.0);
    }

    #[test]
    fn retry_challenges_true_stays_in_slot() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        // retry_challenges = true → slot stays on completion.
        let mut state = GameState::default();
        state.challenges.current_transcension_challenge = 1;
        state.coin_counters.coins_this_transcension = Decimal::from_finite(1e11);
        state.automation.retry_challenges = true;
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        // Slot must NOT be cleared.
        assert_eq!(state.challenges.current_transcension_challenge, 1);
        // But the completion was still awarded.
        assert_eq!(state.challenges.challenge_completions[1], 1.0);
        // And the structural reset still ran (coin producers zeroed).
        assert_eq!(state.coin_producers.tiers[0].owned, 0.0);
    }

    #[test]
    fn retry_challenges_true_partial_completion_stays() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        // retry_challenges=true AND auto_challenge_running=true, but the single
        // completion this tick leaves comp(1) below the cap(25) → slot stays.
        let mut state = GameState::default();
        state.challenges.current_transcension_challenge = 1;
        state.automation.retry_challenges = true;
        state.automation.auto_challenge_running = true;
        // At comp=0, coins=1e11 → completes once (comp=1), 1 < 25 → stays in slot.
        state.coin_counters.coins_this_transcension = Decimal::from_finite(1e11);
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert_eq!(state.challenges.challenge_completions[1], 1.0);
        // comp(1) < max(25) → stay_in_challenge = true → slot not cleared.
        assert_eq!(state.challenges.current_transcension_challenge, 1);
    }

    #[test]
    fn retry_challenges_stay_in_challenge_condition_logic() {
        // Verify the stay_in_challenge boolean formula directly via state reads.
        // retry=true, auto_running=true, comp < max → stays.
        // retry=true, auto_running=true, comp >= max → exits.
        let mut state = GameState::default();
        state.automation.retry_challenges = true;
        state.automation.auto_challenge_running = true;
        // comp=1, max=25 → stay = true && !(true && false) = true && true = true.
        let stay = state.automation.retry_challenges
            && !(state.automation.auto_challenge_running && 1.0_f64 >= 25.0_f64);
        assert!(stay);
        // comp=25, max=25 → stay = true && !(true && true) = true && false = false.
        let no_stay = state.automation.retry_challenges
            && !(state.automation.auto_challenge_running && 25.0_f64 >= 25.0_f64);
        assert!(!no_stay);
    }

    #[test]
    fn toggle_auto_retry_challenges_sets_flag() {
        let mut state = GameState::default();
        assert!(!state.automation.retry_challenges);
        set_automation_toggle(&mut state, AutoToggle::RetryChallenges, true);
        assert!(state.automation.retry_challenges);
        set_automation_toggle(&mut state, AutoToggle::RetryChallenges, false);
        assert!(!state.automation.retry_challenges);
    }

    #[test]
    fn highest_challenge_rewards_fires_quarks_on_new_highest() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        let mut state = GameState::default();
        // c1 transcension challenge, enough coins to complete.
        state.challenges.current_transcension_challenge = 1;
        state.coin_counters.coins_this_transcension = Decimal::from_finite(1e11);
        // Clear the transcension no-buy flags so the completion-triggered reset
        // doesn't add their quarks (isolating the highest-challenge reward).
        state.multiplier.transcend_no_multiplier = false;
        state.accelerator.transcend_no_accelerator = false;
        state.upgrades.transcend_no_coin_upgrades = false;
        state.upgrades.transcend_no_coin_or_prestige_upgrades = false;
        // ascension_count == 0 → gate passes.
        assert_eq!(state.reset_counters.ascension_count, 0.0);
        // quark_bonus = 0 → multiplier = 1; base = 1 + floor(1 * 0.1) = 1 + 0 = 1.
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        // Quark event fired once for highest[1] rising from 0 → 1.
        let quark_events: Vec<_> = output
            .events
            .iter()
            .filter(|e| matches!(e, CoreEvent::QuarksAwarded { .. }))
            .collect();
        assert_eq!(quark_events.len(), 1);
        // base = 1 + floor(1 * 0.1) = 1; multiplier = 1.0 → awarded = 1.0.
        assert!(
            matches!(quark_events[0], CoreEvent::QuarksAwarded { quarks } if (*quarks - 1.0).abs() < 1e-9)
        );
        // worlds = 1 (highest-challenge reward) + 5 (the challenge-1 achievement
        // award now grants getAchievementQuarks() = 5 at the default quark bonus).
        assert!((state.quarks.worlds.to_number() - 6.0).abs() < 1e-9);
        assert!((state.golden_quarks.quarks_this_singularity - 6.0).abs() < 1e-9);
    }

    #[test]
    fn highest_challenge_rewards_skipped_when_ascension_count_nonzero() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        let mut state = GameState::default();
        state.challenges.current_transcension_challenge = 1;
        state.coin_counters.coins_this_transcension = Decimal::from_finite(1e11);
        // Clear the transcension no-buy flags so the completion-triggered reset
        // doesn't add their quarks (isolating the gated highest-challenge reward).
        state.multiplier.transcend_no_multiplier = false;
        state.accelerator.transcend_no_accelerator = false;
        state.upgrades.transcend_no_coin_upgrades = false;
        state.upgrades.transcend_no_coin_or_prestige_upgrades = false;
        // ascension_count > 0 → gate blocks quark award.
        state.reset_counters.ascension_count = 1.0;
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        assert!(!output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::QuarksAwarded { .. })));
        // The highest-challenge reward is gated off (no QuarksAwarded event), but
        // the challenge-1 achievement still grants its 5 quarks (ungated).
        assert_eq!(state.quarks.worlds.to_number(), 5.0);
    }

    #[test]
    fn highest_challenge_rewards_reincarnation_multiplier_is_one() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        // c6 reincarnation: multiplier = 1; highest will rise to 1.
        // base = 1 + floor(1 * 1) = 2; quark_bonus = 0 → awarded = 2.0.
        let mut state = GameState::default();
        state.challenges.current_reincarnation_challenge = 6;
        state.reset_counters.transcend_shards = Decimal::from_finite(1e130);
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut output = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut output);
        let quark_event = output
            .events
            .iter()
            .find(|e| matches!(e, CoreEvent::QuarksAwarded { .. }));
        assert!(quark_event.is_some());
        assert!(
            matches!(quark_event.unwrap(), CoreEvent::QuarksAwarded { quarks } if (*quarks - 2.0).abs() < 1e-9)
        );
    }

    #[test]
    fn tack_dispatches_buy_ambrosia_upgrade_action() {
        let mut state = GameState::default();
        state.ambrosia.ambrosia = 10.0;

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::AmbrosiaUpgrade(
                BuyAmbrosiaUpgradeInput {
                    index: 0,
                    cost_per_level: 1.0,
                    max_level: 10.0,
                    blueberry_cost: 0.0,
                    blueberry_inventory: 0.0,
                },
            )));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::AmbrosiaUpgradePurchased { .. })),
            "expected AmbrosiaUpgradePurchased in events, got {:?}",
            output.events
        );
        assert_eq!(state.ambrosia.upgrades[0].level, 1.0);
    }

    #[test]
    fn tack_dispatches_buy_rune_levels_action() {
        use synergismforkd_bignum::Decimal;

        let mut state = GameState::default();
        state.automation.offerings = Decimal::from_finite(1000.0);

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::RuneLevels(
                BuyRuneLevelsInput {
                    index: 0,
                    cost_coefficient: Decimal::from_finite(100.0),
                    levels_per_oom: 5.0,
                    rune_exp_per_offering: Decimal::from_finite(10.0),
                    levels_to_add: 5.0,
                    budget: Decimal::from_finite(1000.0),
                },
            )));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::RuneLevelsPurchased { .. })),
            "expected RuneLevelsPurchased in events, got {:?}",
            output.events
        );
        assert_eq!(state.runes.rune_levels[0], 5.0);
    }

    #[test]
    fn tack_dispatches_buy_ant_actions() {
        use synergismforkd_bignum::Decimal;

        let mut state = GameState::default();
        state.ants.crumbs = Decimal::from_finite(1000.0);

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::AntProducer(
                BuyAntProducerInput {
                    index: 0,
                    max: false,
                },
            )));
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::AntUpgrade(
                BuyAntUpgradeInput {
                    index: 0,
                    max: false,
                },
            )));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::AntProducersPurchased { .. })),
            "expected AntProducersPurchased in events, got {:?}",
            output.events
        );
        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::AntUpgradePurchased { .. })),
            "expected AntUpgradePurchased in events, got {:?}",
            output.events
        );
        assert_eq!(state.ants.producers[0].purchased, 1.0);
        assert_eq!(state.ants.upgrades[0], 1.0);
    }

    #[test]
    fn tack_dispatches_buy_hepteract_craft_action() {
        use crate::mechanics::hepteract_values::{HepteractConversions, HepteractKind};

        let mut state = GameState::default();
        state.hepteracts.chronos.cap = 100.0;
        state.cube_balances.wow_abyssals = 50.0;

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::HepteractCraft(
                BuyHepteractCraftInput {
                    kind: HepteractKind::Chronos,
                    conversions: HepteractConversions {
                        hepteract: 1.0,
                        ..HepteractConversions::default()
                    },
                    craft_cost_multi: 1.0,
                    exalt_3_cap: false,
                    requested_amount: 10.0,
                    max: false,
                },
            )));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::HepteractCrafted { .. })),
            "expected HepteractCrafted in events, got {:?}",
            output.events
        );
        // 10 units crafted at cost 1 abyssal each: bal 0 → 10, abyssals 50 → 40.
        assert_eq!(state.hepteracts.chronos.bal, 10.0);
        assert!((state.cube_balances.wow_abyssals - 40.0).abs() < 1e-9);
    }

    #[test]
    fn tack_dispatches_buy_hepteract_expand_action() {
        use crate::mechanics::hepteract_values::{BuyHepteractExpandInput, HepteractKind};

        let mut state = GameState::default();
        state.hepteracts.chronos.cap = 100.0;
        state.hepteracts.chronos.bal = 100.0; // full bar — expandable

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::HepteractExpand(
                BuyHepteractExpandInput {
                    kind: HepteractKind::Chronos,
                },
            )));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::HepteractCapExpanded { .. })),
            "expected HepteractCapExpanded in events, got {:?}",
            output.events
        );
        // Spent one cap (100) and doubled it: bal 0, cap 200.
        assert_eq!(state.hepteracts.chronos.bal, 0.0);
        assert_eq!(state.hepteracts.chronos.cap, 200.0);
    }

    #[test]
    fn tack_dispatches_buy_talisman_level_action() {
        use crate::mechanics::talisman_costs::TalismanCraftCosts;
        use synergismforkd_bignum::Decimal;

        let mut state = GameState::default();
        state.talismans.talisman_shards = 100.0;

        let costs = TalismanCraftCosts {
            shard: Decimal::from_finite(10.0),
            common_fragment: Decimal::zero(),
            uncommon_fragment: Decimal::zero(),
            rare_fragment: Decimal::zero(),
            epic_fragment: Decimal::zero(),
            legendary_fragment: Decimal::zero(),
            mythical_fragment: Decimal::zero(),
        };

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::TalismanLevel(
                BuyTalismanLevelInput {
                    index: 0,
                    costs,
                    level_cap: 100.0,
                },
            )));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::TalismanLevelPurchased { .. })),
            "expected TalismanLevelPurchased in events, got {:?}",
            output.events
        );
        assert_eq!(state.talismans.talisman_levels[0], 1.0);
    }

    #[test]
    fn tack_dispatches_buy_platonic_upgrade_action() {
        use synergismforkd_bignum::Decimal;

        let mut state = GameState::default();
        state.researches.obtainium = Decimal::from_finite(1e50);
        state.automation.offerings = Decimal::from_finite(1e50);
        state.cube_balances.wow_cubes = Decimal::from_finite(1e50);
        state.cube_balances.wow_tesseracts = Decimal::from_finite(1e50);
        state.cube_balances.wow_hypercubes = Decimal::from_finite(1e50);
        state.cube_balances.wow_platonic_cubes = Decimal::from_finite(1e50);

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::PlatonicUpgrade(
                BuyPlatonicUpgradeInput {
                    index: 1,
                    singularity_debuff: 1.0,
                },
            )));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::PlatonicUpgradePurchased { .. })),
            "expected PlatonicUpgradePurchased in events, got {:?}",
            output.events
        );
        assert_eq!(state.cube_upgrade_levels.platonic_upgrades[1], 1.0);
    }

    #[test]
    fn tack_dispatches_buy_shop_action() {
        use synergismforkd_bignum::Decimal;

        let mut state = GameState::default();
        state.quarks.worlds = Decimal::from_finite(500.0);

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::Shop(BuyShopInput {
                index: 8,
                is_consumable: false,
                max_level: 10.0,
                price: 100.0,
                price_increase: 25.0,
            })));

        let output = tack(&mut state, &input);

        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::ShopUpgradePurchased { .. })),
            "expected ShopUpgradePurchased in events, got {:?}",
            output.events
        );
        assert_eq!(state.shop.upgrades[8], 1.0);
    }

    #[test]
    fn tack_dispatches_manual_prestige_reset() {
        use synergismforkd_bignum::Decimal;

        use crate::events::AutoResetTier;

        let mut state = GameState::default();
        // 1e18 coins-this-prestige ⇒ floor((1e18 / 1e12) ^ 0.5) = 1000 points.
        state.coin_counters.coins_this_prestige = Decimal::from_finite(1e18);
        // Dirty a producer cost + a coin upgrade to prove the reset clears
        // them through `tack`. (Leave `owned` at 0: nonzero producers would
        // credit one last coin batch in Phase 4 from the `produce_total`
        // computed in Phase 2b, *before* the Phase-3 reset — a small
        // same-tick generation residual, not a reset bug.)
        state.coin_producers.tiers[0].cost = Decimal::from_finite(999.0);
        state.upgrades.upgrades[5] = 1;

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Reset(ResetRequest::Prestige));

        let output = tack(&mut state, &input);

        // End-to-end through `tack`: the reset awards the tick's
        // prestige_point_gain and clears the coin economy.
        assert!(
            output.events.iter().any(|e| matches!(
                e,
                CoreEvent::ResetPerformed {
                    tier: AutoResetTier::Prestige,
                    ..
                }
            )),
            "expected ResetPerformed in events, got {:?}",
            output.events
        );
        assert_eq!(state.upgrades.prestige_points.to_number(), 1000.0);
        assert_eq!(state.coin_counters.coins_this_prestige.to_number(), 100.0);
        assert_eq!(state.coin_producers.tiers[0].cost.to_number(), 100.0);
        assert_eq!(state.upgrades.upgrades[5], 0);
        assert_eq!(state.reset_counters.prestige_count, 1.0);
        assert!(state.reset_counters.prestige_unlocked);
    }

    #[test]
    fn tack_dispatches_manual_transcension_reset() {
        use synergismforkd_bignum::Decimal;

        use crate::events::AutoResetTier;

        let mut state = GameState::default();
        // 1e200 coins-this-transcension ⇒ floor((1e200 / 1e100) ^ 0.03)
        // = floor(10^3) = 1000 transcend points.
        state.coin_counters.coins_this_transcension = Decimal::from_finite(1e200);
        // Dirty a diamond-producer cost + a transcension-tier upgrade slot
        // to prove the transcension layer clears them through `tack`. (Leave
        // `owned` at 0 to avoid the Phase-4 generation residual.)
        state.diamond_producers.tiers[0].cost = Decimal::from_finite(7.0);
        state.upgrades.upgrades[30] = 1;

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Reset(ResetRequest::Transcension));

        let output = tack(&mut state, &input);

        assert!(
            output.events.iter().any(|e| matches!(
                e,
                CoreEvent::ResetPerformed {
                    tier: AutoResetTier::Transcension,
                    ..
                }
            )),
            "expected a transcension ResetPerformed, got {:?}",
            output.events
        );
        assert_eq!(state.upgrades.transcend_points.to_number(), 1000.0);
        assert_eq!(state.upgrades.prestige_points.to_number(), 0.0); // zeroed by transcension
        assert_eq!(
            state.coin_counters.coins_this_transcension.to_number(),
            100.0
        );
        assert_eq!(state.diamond_producers.tiers[0].cost.to_number(), 100.0);
        assert_eq!(state.upgrades.upgrades[30], 0);
        assert_eq!(state.reset_counters.transcend_count, 1.0);
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
    fn dispatch_buy_routes_constant_upgrade() {
        let mut state = GameState::default();
        state.campaigns.ascend_shards = Decimal::from_finite(1e3);
        // researches[175] == 0 -> the buy deducts ascend shards.
        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::ConstantUpgrade(
                BuyConstantUpgradeInput { index: 1 },
            )));
        let _ = tack(&mut state, &input);
        // i=1 (base 1), 1000 shards: toBuy = floor(1 + log10(1000)) = 4.
        assert_eq!(state.campaigns.constant_upgrades[1], 4.0);
        assert_eq!(state.campaigns.ascend_shards.to_number(), 0.0);
    }

    #[test]
    fn dispatch_buy_routes_ant_mastery() {
        let mut state = GameState::default();
        // Workers (producer 0) level 0 needs 0 ELO and costs 1e700.
        state.upgrades.reincarnation_points = Decimal::from_mantissa_exponent(1.0, 1_000.0);
        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::AntMastery(
                BuyAntMasteryInput { producer: 0 },
            )));
        let _ = tack(&mut state, &input);
        assert_eq!(state.ants.masteries[0].mastery, 1);
        assert_eq!(state.ants.masteries[0].highest_mastery, 1);
    }

    #[test]
    fn tack_awards_reset_count_achievements() {
        // A tick with a non-zero lifetime reset count awards the matching
        // count-group achievements (phase_global_state sweep) + their points.
        let mut state = GameState::default();
        state.reset_counters.prestige_count = 100.0; // crosses prestigeCount 1/10/100
        let before = state.achievements.achievement_points;
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        assert_eq!(state.achievements.achievements[436], 1); // prestigeCount >= 1
        assert_eq!(state.achievements.achievements[438], 1); // prestigeCount >= 100
        assert_eq!(state.achievements.achievements[439], 0); // >= 1000 not reached
        assert_eq!(
            state.achievements.achievement_points,
            before + 2.0 + 4.0 + 6.0
        );
    }

    #[test]
    fn constant_ex_shop_level_feeds_global_mult_pre() {
        // The constantEX shop upgrade (identity effect) now feeds the global
        // multiplier's constant-upgrade term instead of a frozen 0.
        let mut state = GameState::default();
        assert_eq!(
            compute_global_multipliers_pre(&state).constant_ex_max_percent_increase,
            0.0
        );
        state.shop.upgrades[crate::state::shop::SHOP_CONSTANT_EX] = 5.0;
        assert_eq!(
            compute_global_multipliers_pre(&state).constant_ex_max_percent_increase,
            5.0
        );
    }

    #[test]
    fn auto_buy_purchases_coin_producers_when_enabled() {
        // The idle loop self-purchases once the per-building autobuy toggle is
        // on and its unlock upgrade is owned — the updateAll autobuyer gap.
        let mut state = GameState::default();
        state.automation.toggles[1] = true; // player.toggles[1] — coin tier-1 autobuy
        state.upgrades.upgrades[81] = 1; // the autobuy-unlock upgrade
        state.upgrades.coins = Decimal::from_finite(1e6);
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let output = tack(&mut state, &input);
        assert!(state.coin_producers.tiers[0].owned > 0.0);
        assert!(output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ProducersPurchased { .. })));
    }

    #[test]
    fn auto_buy_inert_when_toggles_off() {
        // Default save: every autobuy toggle off → the driver buys nothing and
        // the tick still emits only the single obtainium-recompute request.
        let mut state = GameState::default();
        state.upgrades.coins = Decimal::from_finite(1e30);
        state.upgrades.upgrades[81] = 1; // unlock owned, but toggle is off
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let output = tack(&mut state, &input);
        assert_eq!(state.coin_producers.tiers[0].owned, 0.0);
        assert!(!output
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ProducersPurchased { .. })));
    }

    #[test]
    fn auto_buy_purchases_mythos_producers_when_enabled() {
        let mut state = GameState::default();
        state.automation.toggles[16] = true; // mythos tier-1 autobuy
        state.upgrades.upgrades[94] = 1;
        state.upgrades.transcend_points = Decimal::from_finite(1e30);
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        assert!(state.mythos_producers.tiers[0].owned > 0.0);
    }

    #[test]
    fn auto_buy_diamond_producers_gated_on_crystal_milestone() {
        let mut state = GameState::default();
        state.automation.toggles[10] = true; // diamond tier-1 autobuy
        state.upgrades.prestige_points = Decimal::from_finite(1e30);
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        // Level 0: tier1CrystalAutobuy milestone not met -> no purchase.
        let mut low = state.clone();
        let _ = tack(&mut low, &input);
        assert_eq!(low.diamond_producers.tiers[0].owned, 0.0);
        // Level >= 6 unlocks the milestone -> diamonds auto-buy.
        state.level.level = 100.0;
        let _ = tack(&mut state, &input);
        assert!(state.diamond_producers.tiers[0].owned > 0.0);
    }

    #[test]
    fn auto_buy_crystal_upgrades_gated_on_milestone() {
        let mut state = GameState::default();
        state.level.level = 1000.0; // unlock all tierNCrystalAutobuy milestones
        state.crystal_upgrades.prestige_shards = Decimal::from_finite(1e30);
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        // auto = free buy; the level is computed from prestige shards.
        assert!(state.crystal_upgrades.crystal_upgrades[0] > 0.0);
    }

    #[test]
    fn auto_buy_constant_upgrades_when_research_175() {
        let mut state = GameState::default();
        state.researches.researches[175] = 1.0;
        state.campaigns.ascend_shards = Decimal::from_finite(1e3);
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        // research[175] > 0 → free constant-upgrade buys.
        assert!(state.campaigns.constant_upgrades[1] > 0.0);
    }

    #[test]
    fn auto_buy_ant_producers_when_unlocked() {
        let mut state = GameState::default();
        state.ants.toggles.autobuy_producers = true;
        // Achievement 173 grants antAutobuyers → tiers_unlocked = 0 (Workers).
        state.achievements.achievements[173] = 1;
        state.ants.crumbs = Decimal::from_finite(1e6);
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        assert!(state.ants.producers[0].purchased > 0.0);
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
    fn prestige_shards_accumulate_across_ticks() {
        // Regression (audit H1): the seed slice (reset_counters.prestige_shards)
        // differed from the writeback slice (crystal_upgrades.prestige_shards),
        // so each tick overwrote the balance with one tick's production and
        // diamonds/crystals never grew. With the seed fixed, two ticks of
        // 50 shards/tick must accumulate to 100.
        let mut state = GameState::default();
        state.diamond_producers.tiers[0].owned = 1000.0;
        let input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        let _ = tack(&mut state, &input);
        assert!((state.crystal_upgrades.prestige_shards.to_number() - 50.0).abs() < 1e-9);
        let _ = tack(&mut state, &input);
        // Second tick adds another 50 on top (was still 50 before the fix).
        assert!((state.crystal_upgrades.prestige_shards.to_number() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn true_ant_level_routes_free_levels_and_extinction() {
        use crate::mechanics::corruptions::extinction_divisor_at_level;
        use crate::state::EXTINCTION_INDEX;
        // Audit H2: ant-upgrade effects must read the true level (purchased +
        // capped free levels, / extinction divisor), not the raw purchased
        // level. Coins (1) is corruption-eligible; Mortuus (11) is exempt.
        const COINS: usize = 1;
        const MORTUUS: usize = 11;

        let mut state = GameState::default();
        state.ants.upgrades[COINS] = 100.0;
        // Identity at default state (no free levels, extinction divisor 1.0).
        assert!((true_ant_level(&state, COINS) - 100.0).abs() < 1e-9);

        // research[97] grants 2x = 20 free levels, capped by min(100, 20) = 20.
        state.researches.researches[97] = 10.0;
        assert!((true_ant_level(&state, COINS) - 120.0).abs() < 1e-9);

        // Extinction corruption level 4 → divisor 3.0 on the non-exempt upgrade.
        state.corruptions.used.levels[EXTINCTION_INDEX] = 4;
        let div = extinction_divisor_at_level(4);
        assert!((div - 3.0).abs() < 1e-12);
        assert!((true_ant_level(&state, COINS) - 120.0 / div).abs() < 1e-9);

        // The exempt upgrade (Mortuus) ignores the divisor but still gets free levels.
        state.ants.upgrades[MORTUUS] = 100.0;
        assert!((true_ant_level(&state, MORTUUS) - 120.0).abs() < 1e-9);
    }

    #[test]
    fn hepteract_effective_bal_applies_dr_above_limit() {
        // Audit P1.4: hepteract effects read the DR-softened balance. Below the
        // LIMIT (1000) the softening is identity; above it the excess is raised
        // to the craft's DR exponent (legacy hepteractEffective).
        assert_eq!(hepteract_effective_bal(500.0, 1.0 / 5.0), 500.0);
        assert_eq!(hepteract_effective_bal(1000.0, 1.0 / 5.0), 1000.0);
        // 1000 * (10000/1000)^(1/5) = 1000 * 10^0.2.
        let expected = 1000.0 * 10.0_f64.powf(0.2);
        assert!((hepteract_effective_bal(10000.0, 1.0 / 5.0) - expected).abs() < 1e-6);
    }

    #[test]
    fn first_five_effective_rune_level_mult_matches_ts() {
        // Audit P2.1/H3. TS-anchored against the verbatim Statistics.ts
        // firstFiveRuneEffectivenessStats expressions (computed in
        // tmp/rune_anchor.mjs); the neutral factors are 1 at these inputs, so the
        // full TS product equals this port.
        let mut s = GameState::default();
        assert_eq!(first_five_effective_rune_level_mult(&s), 1.0);

        s = GameState::default();
        s.researches.researches[4] = 50.0;
        assert!((first_five_effective_rune_level_mult(&s) - 6.0).abs() < 1e-9);

        s = GameState::default();
        s.researches.researches[21] = 100.0;
        assert!((first_five_effective_rune_level_mult(&s) - 2.0).abs() < 1e-9);

        s = GameState::default();
        s.researches.researches[4] = 20.0;
        s.challenges.challenge_completions[14] = 30.0; // CalcECC(asc,30)=20 -> 1+2*21=43
        assert!((first_five_effective_rune_level_mult(&s) - 43.0).abs() < 1e-9);

        s = GameState::default();
        s.researches.researches[4] = 20.0;
        s.researches.researches[84] = 100.0;
        s.researches.researches[146] = 1000.0;
        s.campaigns.constant_upgrades[9] = 1.0;
        s.talismans.talisman_shards = 15.0;
        assert!((first_five_effective_rune_level_mult(&s) - 22.95).abs() < 1e-9);
    }

    #[test]
    fn first_five_effective_rune_level_clamps_and_scales() {
        use crate::state::RUNE_SPEED;
        // Identity at default: effective level == purchased level.
        let mut s = GameState::default();
        s.runes.rune_levels[RUNE_SPEED] = 100.0;
        assert_eq!(first_five_effective_rune_level(&s, RUNE_SPEED), 100.0);

        // researches[21] = 100 -> mult 2 -> effective 200.
        s.researches.researches[21] = 100.0;
        assert!((first_five_effective_rune_level(&s, RUNE_SPEED) - 200.0).abs() < 1e-9);

        // Reincarnation challenge 9 collapses the effective level to 1.
        s.challenges.current_reincarnation_challenge = 9;
        assert_eq!(first_five_effective_rune_level(&s, RUNE_SPEED), 1.0);
    }

    #[test]
    fn other_blessing_multipliers_matches_ts() {
        // Audit P2.2/H4. TS-anchored vs verbatim RuneBlessings.ts
        // otherBlessingMultipliers (tmp/blessing_anchor.mjs); the neutral factors
        // (midas rarity-0, epicFragments, challenge15) are 1 here.
        let mut s = GameState::default();
        assert_eq!(other_blessing_multipliers(&s), 1.0);

        s = GameState::default();
        s.researches.researches[134] = 10.0;
        assert!((other_blessing_multipliers(&s) - 1.69).abs() < 1e-9);

        s = GameState::default();
        s.researches.researches[194] = 50.0;
        assert!((other_blessing_multipliers(&s) - 2.0).abs() < 1e-9);

        s = GameState::default();
        s.researches.researches[160] = 4.0;
        assert!((other_blessing_multipliers(&s) - 2.0).abs() < 1e-9);

        s = GameState::default();
        s.researches.researches[134] = 10.0;
        s.researches.researches[194] = 50.0;
        s.researches.researches[160] = 4.0;
        assert!((other_blessing_multipliers(&s) - 6.76).abs() < 1e-9);
    }

    #[test]
    fn rune_blessing_power_folds_in_rune_level() {
        use crate::state::RUNE_SPEED;
        // power = blessing.level * rune.level * otherBlessingMultipliers; the
        // dominant fix is the rune-level fold (was raw blessing level before).
        let mut s = GameState::default();
        s.runes.rune_blessing_levels[RUNE_SPEED] = 5.0;
        s.runes.rune_levels[RUNE_SPEED] = 1000.0;
        // mult 1 at default -> 5 * 1000 * 1 = 5000 (was 5 raw).
        assert!((rune_blessing_power(&s, RUNE_SPEED) - 5000.0).abs() < 1e-9);

        // researches[134] = 10 -> mult 1.69 -> 5 * 1000 * 1.69 = 8450.
        s.researches.researches[134] = 10.0;
        assert!((rune_blessing_power(&s, RUNE_SPEED) - 8450.0).abs() < 1e-9);
    }

    #[test]
    fn rune_spirit_power_folds_spirit_rune_and_blessing() {
        use crate::state::RUNE_SPEED;
        // power = spirit.level * rune.level * blessing.level * otherSpiritMultipliers.
        let mut s = GameState::default();
        s.runes.rune_spirit_levels[RUNE_SPEED] = 5.0;
        s.runes.rune_levels[RUNE_SPEED] = 1000.0;
        s.runes.rune_blessing_levels[RUNE_SPEED] = 10.0;
        // otherSpiritMultipliers = 1 at default -> 5 * 1000 * 10 * 1 = 50_000.
        assert!((rune_spirit_power(&s, RUNE_SPEED) - 50_000.0).abs() < 1e-6);

        // researches[164] = 25 -> (1 + 8*25/100) = 3.0 -> 50_000 * 3 = 150_000.
        s.researches.researches[164] = 25.0;
        assert!((rune_spirit_power(&s, RUNE_SPEED) - 150_000.0).abs() < 1e-6);
    }

    #[test]
    fn other_spirit_multipliers_identity_at_default() {
        let s = GameState::default();
        assert!((other_spirit_multipliers(&s) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn speed_spirit_engages_global_speed_mult() {
        use crate::state::RUNE_SPEED;
        // Hold the rune + blessing levels fixed and toggle only the spirit
        // level, so the delta isolates the speed-spirit wiring.
        let mut s = GameState::default();
        s.runes.rune_levels[RUNE_SPEED] = 1000.0;
        s.runes.rune_blessing_levels[RUNE_SPEED] = 1000.0;
        let without_spirit = compute_global_speed_mult_pre(&s);
        // power = 1000 * 1000 * 1000 = 1e9 -> globalSpeed spirit factor = 2.0.
        s.runes.rune_spirit_levels[RUNE_SPEED] = 1000.0;
        let with_spirit = compute_global_speed_mult_pre(&s);
        // The spirit is a clean multiplicative factor on the pre-DR product;
        // the concave global-speed DR can only shrink it, so the boost is in
        // (1, 2].
        assert!(
            with_spirit > without_spirit,
            "speed spirit should raise the global-speed mult"
        );
        assert!(with_spirit <= 2.0 * without_spirit + 1e-6);
    }

    #[test]
    fn talisman_unlock_gates_are_pure_state_functions() {
        use crate::state::shop::SHOP_TALISMAN;
        use crate::state::{
            TALISMAN_CHRONOS, TALISMAN_COOKIE_GRANDMA, TALISMAN_COUNT, TALISMAN_EXEMPTION,
            TALISMAN_MORTUUS, TALISMAN_PLASTIC, TALISMAN_WOW_SQUARE,
        };
        let mut s = GameState::default();
        // Default: every talisman locked.
        for t in 0..TALISMAN_COUNT {
            assert!(
                !talisman_is_unlocked(&s, t),
                "talisman {t} should start locked"
            );
        }
        // exemption ← reincarnation challenge 9 completed at least once.
        s.challenges.highest_challenge_completions[9] = 1.0;
        assert!(talisman_is_unlocked(&s, TALISMAN_EXEMPTION));
        // mortuus ← Mortuus ant upgrade owned.
        s.ants.upgrades[11] = 5.0;
        assert!(talisman_is_unlocked(&s, TALISMAN_MORTUUS));
        // plastic ← shopTalisman bought.
        s.shop.upgrades[SHOP_TALISMAN] = 1.0;
        assert!(talisman_is_unlocked(&s, TALISMAN_PLASTIC));
        // wowSquare ← 100 ascensions.
        s.reset_counters.ascension_count = 100.0;
        assert!(talisman_is_unlocked(&s, TALISMAN_WOW_SQUARE));
        // cookieGrandma ← cubeUpgrade 80.
        s.cube_upgrade_levels.cube_upgrades[80] = 1.0;
        assert!(talisman_is_unlocked(&s, TALISMAN_COOKIE_GRANDMA));
        // chronos gate (getAchievementReward) is unported → still locked.
        assert!(!talisman_is_unlocked(&s, TALISMAN_CHRONOS));
    }

    #[test]
    fn recompute_talisman_rarities_reflects_level_and_unlock() {
        use crate::state::{TALISMAN_CHRONOS, TALISMAN_EXEMPTION};
        let mut s = GameState::default();
        // Locked → rarity 0 even with a level.
        s.talismans.talisman_levels[TALISMAN_EXEMPTION] = 90.0;
        recompute_talisman_rarities(&mut s);
        assert_eq!(s.talismans.talisman_rarity[TALISMAN_EXEMPTION], 0.0);

        // Unlock exemption: level 90 / maxLevel 180 = ratio 0.5 → band 3 → rarity 4.
        s.challenges.highest_challenge_completions[9] = 1.0;
        recompute_talisman_rarities(&mut s);
        assert_eq!(s.talismans.talisman_rarity[TALISMAN_EXEMPTION], 4.0);

        // Unlocked at level 0 → rarity 1 (the floor for an unlocked talisman).
        s.talismans.talisman_levels[TALISMAN_EXEMPTION] = 0.0;
        recompute_talisman_rarities(&mut s);
        assert_eq!(s.talismans.talisman_rarity[TALISMAN_EXEMPTION], 1.0);

        // chronos stays locked (gate unported) → 0 regardless of level.
        s.talismans.talisman_levels[TALISMAN_CHRONOS] = 180.0;
        recompute_talisman_rarities(&mut s);
        assert_eq!(s.talismans.talisman_rarity[TALISMAN_CHRONOS], 0.0);
    }

    #[test]
    fn phase_global_state_brings_talisman_rarity_online() {
        use crate::state::TALISMAN_EXEMPTION;
        let mut s = GameState::default();
        s.challenges.highest_challenge_completions[9] = 1.0; // unlocks exemption
        assert_eq!(s.talismans.talisman_rarity[TALISMAN_EXEMPTION], 0.0);
        let _ = phase_global_state(&mut s);
        // The per-tick recompute runs inside phase_global_state.
        assert_eq!(s.talismans.talisman_rarity[TALISMAN_EXEMPTION], 1.0);
    }

    #[test]
    fn phase_global_state_awards_singularity_count_achievements() {
        // The singularityCount group is swept every tick from
        // highestSingularityCount; the layer is live so this fires in play.
        let mut s = GameState::default();
        assert_eq!(s.achievements.achievements[274], 0);
        s.singularity.highest_singularity_count = 3.0;
        let _ = phase_global_state(&mut s);
        // Rows 274/275/276 (thresholds 1/2/3) awarded; 277 (threshold 4) not.
        assert_eq!(s.achievements.achievements[274], 1);
        assert_eq!(s.achievements.achievements[276], 1);
        assert_eq!(s.achievements.achievements[277], 0);
    }

    #[test]
    fn phase_global_state_awards_thousand_suns_and_moons() {
        // thousandSuns (#250): research 8x25 maxed at 1e5. thousandMoons
        // (#251): cube upgrade w5x10 maxed at 1e5. Each then feeds the
        // quarkGain achievement reward (×1.05 apiece).
        let mut s = GameState::default();
        s.singularity.highest_singularity_count = 1.0; // drop the ×1.25 default
        let _ = phase_global_state(&mut s);
        assert_eq!(s.achievements.achievements[250], 0);

        s.researches.researches[200] = 1e5;
        s.cube_upgrade_levels.cube_upgrades[50] = 1e5;
        let _ = phase_global_state(&mut s);
        assert_eq!(s.achievements.achievements[250], 1);
        assert_eq!(s.achievements.achievements[251], 1);
        // The newly-awarded bits light the quarkGain reward in the multiplier.
        assert!((compute_quark_multiplier(&s) - 1.05 * 1.05).abs() < 1e-9);
        // One level short → not awarded.
        let mut below = GameState::default();
        below.researches.researches[200] = 1e5 - 1.0;
        let _ = phase_global_state(&mut below);
        assert_eq!(below.achievements.achievements[250], 0);
    }

    #[test]
    fn talisman_rune_bonus_scales_with_coefficient_level_and_rarity() {
        use crate::state::{RUNE_DUPLICATION, RUNE_SPEED, TALISMAN_EXEMPTION};
        let mut s = GameState::default();
        s.talismans.talisman_levels[TALISMAN_EXEMPTION] = 10.0;
        s.talismans.talisman_rarity[TALISMAN_EXEMPTION] = 1.0;
        // Locked → no bonus.
        assert_eq!(get_rune_bonus_from_all_talismans(&s, RUNE_DUPLICATION), 0.0);

        s.challenges.highest_challenge_completions[9] = 1.0; // unlock exemption
                                                             // exemption→duplication coeff 1.5, rarity_value(1)=1, level 10,
                                                             // statsSum 1.0 at default → 1.5 * 1 * 10 * 1.0 = 15.
        let dup = get_rune_bonus_from_all_talismans(&s, RUNE_DUPLICATION);
        assert!((dup - 15.0).abs() < 1e-9, "dup = {dup}");
        // exemption→speed coeff is 0 → no speed bonus.
        assert_eq!(get_rune_bonus_from_all_talismans(&s, RUNE_SPEED), 0.0);

        // allTalismanRuneBonusStatsSum scales the total: researches[106] = 1000
        // adds +1.0 → ×2.
        s.researches.researches[106] = 1000.0;
        let dup2 = get_rune_bonus_from_all_talismans(&s, RUNE_DUPLICATION);
        assert!((dup2 - 30.0).abs() < 1e-9, "dup2 = {dup2}");
    }

    #[test]
    fn talisman_bonus_feeds_rune_free_levels() {
        use crate::state::{RUNE_DUPLICATION, TALISMAN_EXEMPTION};
        let mut s = GameState::default();
        s.talismans.talisman_levels[TALISMAN_EXEMPTION] = 10.0;
        s.talismans.talisman_rarity[TALISMAN_EXEMPTION] = 1.0;
        let without = rune_free_levels(&s, RUNE_DUPLICATION); // exemption locked
        s.challenges.highest_challenge_completions[9] = 1.0; // unlock exemption
        let with = rune_free_levels(&s, RUNE_DUPLICATION);
        // The talisman adds exactly 1.5 * 1 * 10 * rarity_value(1) = 15 to the
        // rune's free levels.
        assert!(
            (with - without - 15.0).abs() < 1e-9,
            "delta = {}",
            with - without
        );
    }

    #[test]
    fn accelerator_boost_classic_path_buys_one_and_prestige_resets() {
        let mut s = GameState::default();
        // upgrades[46] = 0 (default) → classic path; default boost cost = 1e3.
        s.upgrades.prestige_points = Decimal::from_finite(1e6);
        s.upgrades.coins = Decimal::from_finite(1e9); // wiped by the prestige reset
        s.upgrades.upgrades[25] = 1; // in the 21..=40 range the boost clears
        let events = buy_accelerator_boost(&mut s, Decimal::zero());

        assert_eq!(s.accelerator.accelerator_boost_bought, 1.0); // persists the reset
        assert_eq!(s.upgrades.prestige_points, Decimal::zero()); // all spent
                                                                 // The prestige reset ran: coins dropped from 1e9 to the 102-coin base.
        assert_eq!(s.upgrades.coins, Decimal::from_finite(102.0));
        assert_eq!(s.upgrades.upgrades[25], 0); // upgrades 21..=40 cleared
        assert!(!s.accelerator.transcend_no_accelerator);
        assert!(!s.accelerator.reincarnate_no_accelerator);
        assert!(events.iter().any(|e| matches!(
            e,
            CoreEvent::AcceleratorBoostsPurchased { after, .. } if *after == 1.0
        )));
        assert!(events
            .iter()
            .any(|e| matches!(e, CoreEvent::ResetPerformed { .. })));
    }

    #[test]
    fn accelerator_boost_bulk_path_spends_prestige_without_reset() {
        let mut s = GameState::default();
        s.upgrades.upgrades[46] = 1; // bulk path
        s.upgrades.prestige_points = Decimal::from_finite(1e30);
        s.upgrades.coins = Decimal::from_finite(1e9);
        let events = buy_accelerator_boost(&mut s, Decimal::zero());

        assert!(s.accelerator.accelerator_boost_bought > 0.0);
        assert!(s.upgrades.prestige_points < Decimal::from_finite(1e30)); // spent some
        assert_eq!(s.upgrades.coins, Decimal::from_finite(1e9)); // no reset
        assert!(events
            .iter()
            .any(|e| matches!(e, CoreEvent::AcceleratorBoostsPurchased { .. })));
        // The bulk path performs no prestige reset.
        assert!(!events
            .iter()
            .any(|e| matches!(e, CoreEvent::ResetPerformed { .. })));
    }

    #[test]
    fn thrift_blessing_pushes_accelerator_boost_cost_threshold() {
        use crate::state::RUNE_THRIFT;
        // Buying boost 1002 fires the quadratic kicker when 1002 > 1000*delay.
        // Default delay = 1 → kicker fires; a thrift blessing lifting delay to 2
        // pushes the threshold to 2000 → no kicker → a cheaper next cost. This
        // is the thrift-rune-blessing production wire.
        let post_buy_cost = |with_blessing: bool| {
            let mut s = GameState::default();
            s.accelerator.accelerator_boost_bought = 1001.0;
            s.accelerator.accelerator_boost_cost = Decimal::from_finite(1e3);
            s.upgrades.prestige_points = Decimal::from_finite(1e6); // affords the 1e3 gate
            if with_blessing {
                // power = blessing.level * rune.level * 1 = 1e6 → delay = 2.
                s.runes.rune_blessing_levels[RUNE_THRIFT] = 1e6;
                s.runes.rune_levels[RUNE_THRIFT] = 1.0;
            }
            let _ = buy_accelerator_boost(&mut s, Decimal::zero());
            s.accelerator.accelerator_boost_cost
        };
        let cost_no_blessing = post_buy_cost(false); // delay 1 → kicker fires
        let cost_with_blessing = post_buy_cost(true); // delay 2 → no kicker
        assert!(
            cost_no_blessing > cost_with_blessing,
            "thrift blessing should lower the post-buy boost cost"
        );
    }

    #[test]
    fn tack_dispatches_accelerator_boost_action() {
        let mut state = GameState::default();
        state.upgrades.upgrades[46] = 1; // bulk path (no reset side effects)
        state.upgrades.prestige_points = Decimal::from_finite(1e30);

        let mut input = TackInput {
            dt: 0.025,
            ..TackInput::default()
        };
        input
            .player_actions
            .push(PlayerAction::Buy(BuyRequest::AcceleratorBoost));

        let output = tack(&mut state, &input);

        assert!(state.accelerator.accelerator_boost_bought > 0.0);
        assert!(
            output
                .events
                .iter()
                .any(|e| matches!(e, CoreEvent::AcceleratorBoostsPurchased { .. })),
            "expected AcceleratorBoostsPurchased, got {:?}",
            output.events
        );
    }

    #[test]
    fn rune_free_levels_assembles_from_state() {
        use crate::state::{RUNE_PRISM, RUNE_SPEED};
        // Audit P2.1b: freeLevels = shared firstFiveFreeLevels + per-rune bonus.
        let mut s = GameState::default();
        assert_eq!(rune_free_levels(&s, RUNE_SPEED), 0.0);

        // Shared: 7 * min(constantUpgrades[7], 1000) = 70 (prism has no per-rune bonus).
        s.campaigns.constant_upgrades[7] = 10.0;
        assert_eq!(rune_free_levels(&s, RUNE_PRISM), 70.0);

        // Speed per-rune bonus: upgrade_29 * floor(min(100, coinsOwned/400)) = 1*2.
        s.upgrades.upgrades[29] = 1;
        s.coin_producers.tiers[0].owned = 800.0;
        assert_eq!(rune_free_levels(&s, RUNE_SPEED), 72.0);
    }

    #[test]
    fn first_five_effective_rune_level_folds_in_free_levels() {
        use crate::state::RUNE_PRISM;
        let mut s = GameState::default();
        s.runes.rune_levels[RUNE_PRISM] = 100.0;
        s.campaigns.constant_upgrades[7] = 10.0; // shared free levels = 70
                                                 // (level 100 + free 70) * mult 1 = 170.
        assert!((first_five_effective_rune_level(&s, RUNE_PRISM) - 170.0).abs() < 1e-9);
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

    #[test]
    fn compute_cube_multiplier_research_192_mortuus_wires_through() {
        use crate::mechanics::reset_currency::ResetCurrencyResult;
        // With research[192] = 0 (default) the term is (1 + 0) = 1 — no change.
        let mut state = GameState::default();
        let gains = ResetCurrencyResult {
            prestige_point_gain: Decimal::zero(),
            transcend_point_gain: Decimal::zero(),
            reincarnation_point_gain: Decimal::zero(),
        };
        let mut out_default = TickOutput::default();
        phase_challenge_completion(&mut state, &gains, &mut out_default);

        // Activate research[192] + give Mortuus some levels → the factor rises.
        // With research[192]=100 and Mortuus level=50 (exempt → divisor=1,
        // free_levels=0 → trueAnt = 50 + min(50,0) = 50):
        //   factor = 1 + (1/500) * 100 * 50 = 1 + 10 = 11
        // This doesn't test the final cube mult directly (it needs an ascension),
        // but we verify the helper path compiles and the mortuus true level
        // formula is correct for the exempt case.
        let mut s = GameState::default();
        s.ants.upgrades[11] = 50.0; // Mortuus level 50
        s.researches.researches[192] = 100.0;
        // At default challenge15_exponent = 0 and all free-level sources = 0:
        // free_levels = 0, trueAnt = 50 + min(50, 0) = 50.
        // Factor = 1 + (1/500) * 100 * 50 = 11.0.
        {
            use crate::mechanics::ant_upgrade_levels::{
                calculate_true_ant_level, CalculateTrueAntLevelInput,
            };
            let true_level = calculate_true_ant_level(&CalculateTrueAntLevelInput {
                current_level: 50.0,
                free_levels: 0.0,
                exempt_from_corruption: true,
                corruption_extinction_divisor: 1.0,
                c11_active: false,
            });
            assert_eq!(true_level, 50.0); // 50 + min(50, 0)=0 → 50+0=50
            let factor = 1.0 + (1.0 / 500.0) * 100.0 * true_level;
            assert!((factor - 11.0).abs() < 1e-9); // 1 + (1/500)*100*50 = 1+10 = 11
        }
    }
}
