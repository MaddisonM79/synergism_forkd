//! Per-tick timer advancement — the Phase 5 "head" (`addTimers`) leaves.
//!
//! Direct port of `legacy/core_split/packages/logic/src/tick/timers.ts`.
//! Each function is a pure leaf: it takes the current accumulator(s) plus
//! the caller pre-evaluated speed multipliers / caps and returns the
//! advanced value(s). The orchestrator ([`super::phase_automation`])
//! threads these into the [`crate::state::GameState`] slices.
//!
//! This chunk covers the five simple counters (reset ×3 / ascension /
//! singularity / quark / golden-quark). The chunked, event-emitting
//! timers (octeract, auto-potion, ambrosia, red-ambrosia) land in
//! follow-on chunks.

use rand::Rng;
use smallvec::SmallVec;

use synergismforkd_bignum::Decimal;

use crate::events::{AutoPotionType, CoreEvent};
use crate::math::rng::next_f64;
use crate::mechanics::ambrosia::{
    calculate_required_blueberry_time, calculate_required_red_ambrosia_time,
    CalculateRequiredBlueberryTimeInput, CalculateRequiredRedAmbrosiaTimeInput,
};
use crate::mechanics::singularity_milestones::{
    calculate_base_golden_quarks, CalculateBaseGoldenQuarksInput,
};

/// Golden-quark export timer cap — 168 hours, in seconds. Mirrors the
/// legacy `GOLDEN_QUARKS_TIMER_CAP_SECONDS` (`3600 * 168`).
pub(crate) const GOLDEN_QUARKS_TIMER_CAP_SECONDS: f64 = 3600.0 * 168.0;

/// Shared counter advancement for prestige / transcension /
/// reincarnation. Each uses the same shape: `counter += dt × mult`, with
/// no caps or conditional speed multipliers.
pub(crate) fn advance_reset_counter(counter: f64, dt: f64, global_time_multiplier: f64) -> f64 {
    counter + dt * global_time_multiplier
}

/// Inputs to [`advance_ascension_timer`].
pub(crate) struct AdvanceAscensionTimerInput {
    /// Tick delta (seconds).
    pub dt: f64,
    /// `player.ascensionCounter` — advances by `dt × ascension_speed_multi`.
    pub ascension_counter: f64,
    /// `player.ascensionCounterReal` — advances by raw `dt`.
    pub ascension_counter_real: f64,
    /// Pre-evaluated ascension speed (`oneMind` → 10, else
    /// `calculateAscensionSpeedMult()`).
    pub ascension_speed_multi: f64,
}

/// Result of [`advance_ascension_timer`].
pub(crate) struct AdvanceAscensionTimerResult {
    /// Advanced `ascension_counter`.
    pub ascension_counter: f64,
    /// Advanced `ascension_counter_real`.
    pub ascension_counter_real: f64,
}

/// Advance the ascension dual-counter. `ascension_counter` scales with
/// the pre-evaluated speed; `ascension_counter_real` is raw wall time.
/// The legacy ascension case passes `timeMultiplier = 1`, so there is no
/// `global_time_multiplier` input.
pub(crate) fn advance_ascension_timer(
    input: &AdvanceAscensionTimerInput,
) -> AdvanceAscensionTimerResult {
    AdvanceAscensionTimerResult {
        ascension_counter: input.ascension_counter + input.dt * input.ascension_speed_multi,
        ascension_counter_real: input.ascension_counter_real + input.dt,
    }
}

/// Inputs to [`advance_singularity_timer`].
pub(crate) struct AdvanceSingularityTimerInput {
    /// Tick delta (seconds).
    pub dt: f64,
    /// `player.ascensionCounterRealReal` — advances by raw `dt`.
    pub ascension_counter_real_real: f64,
    /// `player.singularityCounter` — advances by `dt × singularity_speed_multi`.
    pub singularity_counter: f64,
    /// `player.singChallengeTimer` — advances by `dt × singularity_speed_multi`
    /// while inside a singularity challenge, else resets to `0`.
    pub sing_challenge_timer: f64,
    /// `player.insideSingularityChallenge` — gates the challenge-timer
    /// accumulation vs. reset.
    pub inside_singularity_challenge: bool,
    /// Pre-evaluated singularity speed
    /// (`ambrosiaBrickOfLead.singularitySpeedMult`).
    pub singularity_speed_multi: f64,
}

/// Result of [`advance_singularity_timer`].
pub(crate) struct AdvanceSingularityTimerResult {
    /// Advanced `ascension_counter_real_real`.
    pub ascension_counter_real_real: f64,
    /// Advanced `singularity_counter`.
    pub singularity_counter: f64,
    /// Advanced (or reset) `sing_challenge_timer`.
    pub sing_challenge_timer: f64,
}

/// Advance the singularity tri-counter. The same `dt` feeds all three;
/// `sing_challenge_timer` accumulates only while inside a singularity
/// challenge, otherwise it resets to `0` every tick.
pub(crate) fn advance_singularity_timer(
    input: &AdvanceSingularityTimerInput,
) -> AdvanceSingularityTimerResult {
    let sing_challenge_timer = if input.inside_singularity_challenge {
        input.sing_challenge_timer + input.dt * input.singularity_speed_multi
    } else {
        0.0
    };
    AdvanceSingularityTimerResult {
        ascension_counter_real_real: input.ascension_counter_real_real + input.dt,
        singularity_counter: input.singularity_counter + input.dt * input.singularity_speed_multi,
        sing_challenge_timer,
    }
}

/// Inputs to [`advance_quarks_timer`].
pub(crate) struct AdvanceQuarksTimerInput {
    /// Tick delta (seconds).
    pub dt: f64,
    /// `player.quarkstimer` — advances by raw `dt`, clamped to
    /// `max_quark_timer`.
    pub quarks_timer: f64,
    /// Pre-evaluated `quark_handler().max_time` — upper bound.
    pub max_quark_timer: f64,
}

/// Advance the quark export timer, clamped at `max_quark_timer`
/// (~25 hours, extended by Research 8x20). Legacy uses
/// `timeMultiplier = 1` here.
pub(crate) fn advance_quarks_timer(input: &AdvanceQuarksTimerInput) -> f64 {
    (input.quarks_timer + input.dt).min(input.max_quark_timer)
}

/// Inputs to [`advance_golden_quarks_timer`].
pub(crate) struct AdvanceGoldenQuarksTimerInput {
    /// Tick delta (seconds).
    pub dt: f64,
    /// `player.goldenQuarksTimer` — advances by raw `dt`, clamped to
    /// [`GOLDEN_QUARKS_TIMER_CAP_SECONDS`].
    pub golden_quarks_timer: f64,
    /// Pre-evaluated `goldenQuarks3.exportGQPerHour` — when `0`, the
    /// timer is untouched (the legacy gate).
    pub export_gq_per_hour: f64,
}

/// Advance the golden-quark export timer, gated by the `goldenQuarks3`
/// GQ upgrade. When `export_gq_per_hour == 0`, the timer is untouched;
/// otherwise it accumulates raw `dt` and clamps to the 168-hour cap.
pub(crate) fn advance_golden_quarks_timer(input: &AdvanceGoldenQuarksTimerInput) -> f64 {
    if input.export_gq_per_hour == 0.0 {
        return input.golden_quarks_timer;
    }
    (input.golden_quarks_timer + input.dt).min(GOLDEN_QUARKS_TIMER_CAP_SECONDS)
}

// ─── Octeracts ──────────────────────────────────────────────────────────────

/// Singularity-count thresholds for the GQ-giveaway scaling bonus. Each
/// crossed threshold adds 1 to `actual_level`, scaling the quark fraction
/// siphoned per giveaway-second. Mirrors the legacy
/// `OCTERACT_GIVEAWAY_LEVELS`.
pub(crate) const OCTERACT_GIVEAWAY_LEVELS: &[f64] = &[
    160.0, 173.0, 185.0, 194.0, 204.0, 210.0, 219.0, 229.0, 240.0, 249.0,
];

/// Quark fraction siphoned into golden quarks per giveaway-second, per
/// crossed [`OCTERACT_GIVEAWAY_LEVELS`] threshold. Mirrors the legacy
/// `frac = 1e-6`.
const OCTERACT_GIVEAWAY_QUARK_FRACTION: f64 = 1e-6;

/// Inputs to [`advance_octeract_timer`].
pub(crate) struct AdvanceOcteractTimerInput {
    /// Tick delta (seconds).
    pub dt: f64,
    /// Per-tick time multiplier — `1.0` in legacy (kept for symmetry).
    pub time_multiplier: f64,
    /// Pre-evaluated `octeractUnlock.unlocked` gate.
    pub octeract_unlocked: bool,
    /// `player.octeractTimer` — fractional accumulator (whole seconds are
    /// spent on giveaways; the remainder carries to the next tick).
    pub octeract_timer: f64,
    /// `player.wowOcteracts` balance.
    pub wow_octeracts: Decimal,
    /// `player.totalWowOcteracts` lifetime balance.
    pub total_wow_octeracts: Decimal,
    /// `player.goldenQuarks` — credited by the GQ-giveaway loop.
    pub golden_quarks: Decimal,
    /// `player.quarksThisSingularity` — geometrically decayed inside the
    /// giveaway loop and fed back into `calculate_base_golden_quarks`.
    pub quarks_this_singularity: f64,
    /// Pre-evaluated `calculateOcteractMultiplier()` — per-second reward.
    pub per_second: f64,
    /// `player.highestSingularityCount` — gates the GQ-giveaway block
    /// (≥ 160) and drives `actual_level` + `calculate_base_golden_quarks`.
    pub highest_singularity_count: f64,
    /// `player.singularityCount` — feeds `calculate_base_golden_quarks`.
    pub singularity_count: f64,
    /// Pre-evaluated product of all golden-quark multiplier stats except
    /// the qts-dependent base. Pass `1.0` when the giveaway block is
    /// inactive (it is unused there).
    pub golden_quarks_multiplier_excluding_base: f64,
}

/// Result of [`advance_octeract_timer`].
pub(crate) struct AdvanceOcteractTimerResult {
    /// Advanced `octeract_timer` (fractional remainder).
    pub octeract_timer: f64,
    /// Advanced `wow_octeracts`.
    pub wow_octeracts: Decimal,
    /// Advanced `total_wow_octeracts`.
    pub total_wow_octeracts: Decimal,
    /// Advanced `golden_quarks`.
    pub golden_quarks: Decimal,
    /// Decayed `quarks_this_singularity`.
    pub quarks_this_singularity: f64,
    /// `OcteractTickFired` when at least one giveaway-second elapsed;
    /// empty otherwise.
    pub events: SmallVec<[CoreEvent; 1]>,
}

/// Octeract case of `addTimers`. Accumulates a 1-second-chunked timer;
/// each whole second credits `per_second` octeracts. Above singularity
/// 160, also siphons a geometric fraction of `quarks_this_singularity`
/// into `golden_quarks` per giveaway-second, recomputing
/// `calculate_base_golden_quarks` each iteration (qts decays in the loop).
pub(crate) fn advance_octeract_timer(
    input: &AdvanceOcteractTimerInput,
) -> AdvanceOcteractTimerResult {
    if !input.octeract_unlocked {
        return AdvanceOcteractTimerResult {
            octeract_timer: input.octeract_timer,
            wow_octeracts: input.wow_octeracts,
            total_wow_octeracts: input.total_wow_octeracts,
            golden_quarks: input.golden_quarks,
            quarks_this_singularity: input.quarks_this_singularity,
            events: SmallVec::new(),
        };
    }

    let mut octeract_timer = input.octeract_timer + input.dt * input.time_multiplier;
    if octeract_timer < 1.0 {
        return AdvanceOcteractTimerResult {
            octeract_timer,
            wow_octeracts: input.wow_octeracts,
            total_wow_octeracts: input.total_wow_octeracts,
            golden_quarks: input.golden_quarks,
            quarks_this_singularity: input.quarks_this_singularity,
            events: SmallVec::new(),
        };
    }

    let amount_of_giveaways = octeract_timer - (octeract_timer % 1.0);
    octeract_timer %= 1.0;

    let giveaway_octeracts = Decimal::from_finite(amount_of_giveaways * input.per_second);
    let wow_octeracts = input.wow_octeracts + giveaway_octeracts;
    let total_wow_octeracts = input.total_wow_octeracts + giveaway_octeracts;

    let mut golden_quarks = input.golden_quarks;
    let mut qts = input.quarks_this_singularity;

    if input.highest_singularity_count >= 160.0 {
        let actual_level = OCTERACT_GIVEAWAY_LEVELS
            .iter()
            .filter(|&&level| input.highest_singularity_count >= level)
            .count() as f64;
        let quark_fraction = OCTERACT_GIVEAWAY_QUARK_FRACTION * actual_level;
        let mut gq_delta = 0.0;
        for _ in 0..(amount_of_giveaways as u64) {
            let base = calculate_base_golden_quarks(&CalculateBaseGoldenQuarksInput {
                singularity: input.singularity_count,
                quarks_this_singularity: qts,
                highest_singularity_count: input.highest_singularity_count,
            });
            gq_delta += quark_fraction * base * input.golden_quarks_multiplier_excluding_base;
            qts *= 1.0 - quark_fraction;
        }
        golden_quarks += Decimal::from_finite(gq_delta);
    }

    let mut events = SmallVec::new();
    events.push(CoreEvent::OcteractTickFired {
        amount_of_giveaways: amount_of_giveaways as u32,
    });

    AdvanceOcteractTimerResult {
        octeract_timer,
        wow_octeracts,
        total_wow_octeracts,
        golden_quarks,
        quarks_this_singularity: qts,
        events,
    }
}

// ─── Auto Potion ──────────────────────────────────────────────────────────

/// Minimum singularity count for the auto-potion dispenser. Mirrors the
/// legacy `highestSingularityCount < 6` gate.
const AUTO_POTION_MIN_SINGULARITY: f64 = 6.0;

/// Inputs to [`advance_auto_potion_timer`].
pub(crate) struct AdvanceAutoPotionTimerInput {
    /// Tick delta (seconds).
    pub dt: f64,
    /// Per-tick time multiplier — `1.0` in legacy.
    pub time_multiplier: f64,
    /// `player.highestSingularityCount` — feature gated on `>= 6`.
    pub highest_singularity_count: f64,
    /// `player.autoPotionTimer` — offering-potion accumulator.
    pub auto_potion_timer: f64,
    /// `player.autoPotionTimerObtainium` — obtainium-potion accumulator.
    pub auto_potion_timer_obtainium: f64,
    /// `player.toggles[42]` — fast offering-potion expenditure.
    pub toggle_offering: bool,
    /// `player.toggles[43]` — fast obtainium-potion expenditure.
    pub toggle_obtainium: bool,
    /// `player.shopUpgrades.offeringPotion` — required `> 0` for fast mode.
    pub offering_potion_count: f64,
    /// `player.shopUpgrades.obtainiumPotion` — required `> 0` for fast mode.
    pub obtainium_potion_count: f64,
    /// Pre-evaluated `octeractAutoPotionSpeed.autoPotionSpeedMult`.
    pub auto_potion_speed_mult: f64,
}

/// Result of [`advance_auto_potion_timer`].
pub(crate) struct AdvanceAutoPotionTimerResult {
    /// Advanced offering accumulator.
    pub auto_potion_timer: f64,
    /// Advanced obtainium accumulator.
    pub auto_potion_timer_obtainium: f64,
    /// `AutoPotionFired` events — up to two (offering, obtainium).
    pub events: SmallVec<[CoreEvent; 2]>,
}

/// Auto-potion case of `addTimers`. Advances the two dispense timers,
/// computes the per-side threshold (`180 × 1.03^(-sing) / speedMult`,
/// with a `min(1, threshold) / 20` fast-mode override), and emits an
/// `AutoPotionFired` event carrying the dispense count + fast-mode flag
/// when a timer crosses its threshold. The UI tier turns the event into
/// the `useConsumable(...)` side effect.
pub(crate) fn advance_auto_potion_timer(
    input: &AdvanceAutoPotionTimerInput,
) -> AdvanceAutoPotionTimerResult {
    if input.highest_singularity_count < AUTO_POTION_MIN_SINGULARITY {
        return AdvanceAutoPotionTimerResult {
            auto_potion_timer: input.auto_potion_timer,
            auto_potion_timer_obtainium: input.auto_potion_timer_obtainium,
            events: SmallVec::new(),
        };
    }

    let mut events = SmallVec::new();

    let toggle_offering_on = input.toggle_offering && input.offering_potion_count > 0.0;
    let toggle_obtainium_on = input.toggle_obtainium && input.obtainium_potion_count > 0.0;

    let mut auto_potion_timer = input.auto_potion_timer + input.dt * input.time_multiplier;
    let mut auto_potion_timer_obtainium =
        input.auto_potion_timer_obtainium + input.dt * input.time_multiplier;

    let timer_threshold =
        (180.0 * 1.03_f64.powf(-input.highest_singularity_count)) / input.auto_potion_speed_mult;

    let effective_offering_threshold = if toggle_offering_on {
        timer_threshold.min(1.0) / 20.0
    } else {
        timer_threshold
    };
    let effective_obtainium_threshold = if toggle_obtainium_on {
        timer_threshold.min(1.0) / 20.0
    } else {
        timer_threshold
    };

    if auto_potion_timer >= effective_offering_threshold {
        let amount = ((auto_potion_timer - (auto_potion_timer % effective_offering_threshold))
            / effective_offering_threshold) as u32;
        auto_potion_timer %= effective_offering_threshold;
        events.push(CoreEvent::AutoPotionFired {
            potion_type: AutoPotionType::Offering,
            amount,
            fast_mode: toggle_offering_on,
        });
    }

    if auto_potion_timer_obtainium >= effective_obtainium_threshold {
        let amount = ((auto_potion_timer_obtainium
            - (auto_potion_timer_obtainium % effective_obtainium_threshold))
            / effective_obtainium_threshold) as u32;
        auto_potion_timer_obtainium %= effective_obtainium_threshold;
        events.push(CoreEvent::AutoPotionFired {
            potion_type: AutoPotionType::Obtainium,
            amount,
            fast_mode: toggle_obtainium_on,
        });
    }

    AdvanceAutoPotionTimerResult {
        auto_potion_timer,
        auto_potion_timer_obtainium,
        events,
    }
}

// ─── Ambrosia / Red Ambrosia ────────────────────────────────────────────────

/// Sub-bar granularity for the ambrosia / red-ambrosia accumulators —
/// the timer processes in 1/8 s (`0.125`) chunks. Mirrors the legacy
/// `0.125` constant.
const AMBROSIA_TIMER_GRANULE: f64 = 0.125;

/// Inputs to [`advance_ambrosia_timer`].
pub(crate) struct AdvanceAmbrosiaTimerInput {
    /// Tick delta (seconds) — or, on the red→ambrosia feedback re-entry,
    /// the bonus blueberry time.
    pub dt: f64,
    /// Per-tick time multiplier — `1.0` in legacy.
    pub time_multiplier: f64,
    /// `noSingularityUpgrades.completions` — branch gate (`> 0` to run).
    pub no_singularity_upgrades_completions: f64,
    /// Pre-evaluated `calculateAmbrosiaGenerationSpeed()` — `0` disables.
    pub ambrosia_generation_speed: f64,
    /// `G.ambrosiaTimer` — 1/8 s granule accumulator.
    pub ambrosia_timer_g: f64,
    /// `player.blueberryTime` — generation-bar accumulator.
    pub blueberry_time: f64,
    /// `player.ambrosia` — credited per loop iteration.
    pub ambrosia: f64,
    /// `player.lifetimeAmbrosia` — feeds the next iteration's threshold.
    pub lifetime_ambrosia: f64,
    /// Pre-evaluated `calculateAmbrosiaLuck()`.
    pub ambrosia_luck: f64,
    /// Pre-evaluated `noAmbrosiaUpgrades.bonusAmbrosia`.
    pub bonus_ambrosia: f64,
    /// `G.TIME_PER_AMBROSIA`.
    pub time_per_ambrosia: f64,
    /// Pre-evaluated `shopAmbrosiaAccelerator.ambrosiaPointRequirementMult`.
    pub accelerator_mult: f64,
    /// Pre-evaluated `ambrosiaBrickOfLead.barRequirementMult`.
    pub brick_of_lead_mult: f64,
}

/// Result of [`advance_ambrosia_timer`].
pub(crate) struct AdvanceAmbrosiaTimerResult {
    /// Advanced `ambrosia_timer_g` (mod the granule).
    pub ambrosia_timer_g: f64,
    /// Advanced `blueberry_time`.
    pub blueberry_time: f64,
    /// Advanced `ambrosia`.
    pub ambrosia: f64,
    /// Advanced `lifetime_ambrosia`.
    pub lifetime_ambrosia: f64,
    /// `AmbrosiaGained` when the granule was crossed; empty otherwise.
    /// (Bug-for-bug: the event fires with `amount == 0` if the granule
    /// crossed but no full bar completed.)
    pub events: SmallVec<[CoreEvent; 1]>,
}

/// Ambrosia case of `addTimers`. Accumulates `G.ambrosiaTimer` in 1/8 s
/// ticks, adds `blueberryTime`, then loops crediting ambrosia bars when
/// `blueberryTime` meets the (mutating) `calculate_required_blueberry_time`
/// threshold. Each iteration rolls one RNG value for luck.
pub(crate) fn advance_ambrosia_timer(
    input: &AdvanceAmbrosiaTimerInput,
    rng: &mut impl Rng,
) -> AdvanceAmbrosiaTimerResult {
    if input.no_singularity_upgrades_completions <= 0.0 || input.ambrosia_generation_speed == 0.0 {
        return AdvanceAmbrosiaTimerResult {
            ambrosia_timer_g: input.ambrosia_timer_g,
            blueberry_time: input.blueberry_time,
            ambrosia: input.ambrosia,
            lifetime_ambrosia: input.lifetime_ambrosia,
            events: SmallVec::new(),
        };
    }

    let mut ambrosia_timer_g = input.ambrosia_timer_g + input.dt * input.time_multiplier;
    if ambrosia_timer_g < AMBROSIA_TIMER_GRANULE {
        return AdvanceAmbrosiaTimerResult {
            ambrosia_timer_g,
            blueberry_time: input.blueberry_time,
            ambrosia: input.ambrosia,
            lifetime_ambrosia: input.lifetime_ambrosia,
            events: SmallVec::new(),
        };
    }

    let mut blueberry_time = input.blueberry_time
        + (8.0 * ambrosia_timer_g).floor() / 8.0 * input.ambrosia_generation_speed;
    ambrosia_timer_g %= AMBROSIA_TIMER_GRANULE;

    let mut ambrosia = input.ambrosia;
    let mut lifetime_ambrosia = input.lifetime_ambrosia;
    let mut total_gained = 0.0;

    let mut time_to_ambrosia =
        calculate_required_blueberry_time(&CalculateRequiredBlueberryTimeInput {
            time_per_ambrosia: input.time_per_ambrosia,
            lifetime_ambrosia,
            accelerator_mult: input.accelerator_mult,
            brick_of_lead_mult: input.brick_of_lead_mult,
        });

    while blueberry_time >= time_to_ambrosia {
        let value = next_f64(rng);
        let luck_per_100 = input.ambrosia_luck / 100.0;
        let ambrosia_mult = luck_per_100.floor();
        let luck_mult = if value < luck_per_100 - ambrosia_mult {
            1.0
        } else {
            0.0
        };
        let ambrosia_to_gain = ambrosia_mult + luck_mult + input.bonus_ambrosia;

        ambrosia += ambrosia_to_gain;
        lifetime_ambrosia += ambrosia_to_gain;
        total_gained += ambrosia_to_gain;
        blueberry_time -= time_to_ambrosia;

        time_to_ambrosia =
            calculate_required_blueberry_time(&CalculateRequiredBlueberryTimeInput {
                time_per_ambrosia: input.time_per_ambrosia,
                lifetime_ambrosia,
                accelerator_mult: input.accelerator_mult,
                brick_of_lead_mult: input.brick_of_lead_mult,
            });
    }

    let mut events = SmallVec::new();
    events.push(CoreEvent::AmbrosiaGained {
        amount: total_gained,
    });

    AdvanceAmbrosiaTimerResult {
        ambrosia_timer_g,
        blueberry_time,
        ambrosia,
        lifetime_ambrosia,
        events,
    }
}

/// Inputs to [`advance_red_ambrosia_timer`].
pub(crate) struct AdvanceRedAmbrosiaTimerInput {
    /// Tick delta (seconds).
    pub dt: f64,
    /// Per-tick time multiplier — `1.0` in legacy.
    pub time_multiplier: f64,
    /// `noAmbrosiaUpgrades.completions` — branch gate (`> 0` to run).
    pub no_ambrosia_upgrades_completions: f64,
    /// Pre-evaluated `calculateRedAmbrosiaGenerationSpeed()`.
    pub red_ambrosia_generation_speed: f64,
    /// `G.redAmbrosiaTimer` — 1/8 s granule accumulator.
    pub red_ambrosia_timer_g: f64,
    /// `player.redAmbrosiaTime` — generation-bar accumulator.
    pub red_ambrosia_time: f64,
    /// `player.redAmbrosia` — credited per loop iteration.
    pub red_ambrosia: f64,
    /// `player.lifetimeRedAmbrosia` — feeds the next iteration's threshold.
    pub lifetime_red_ambrosia: f64,
    /// Pre-evaluated `calculateRedAmbrosiaLuck()`.
    pub red_ambrosia_luck: f64,
    /// Pre-evaluated `redAmbrosiaAccelerator.ambrosiaTimePerRedAmbrosia` —
    /// the bonus blueberry time minted per red ambrosia.
    pub ambrosia_time_per_red_ambrosia: f64,
    /// `G.TIME_PER_RED_AMBROSIA`.
    pub time_per_red_ambrosia: f64,
    /// Pre-evaluated `limitedTime.barRequirementMultiplier`.
    pub bar_requirement_multiplier: f64,
}

/// Result of [`advance_red_ambrosia_timer`].
pub(crate) struct AdvanceRedAmbrosiaTimerResult {
    /// Advanced `red_ambrosia_timer_g` (mod the granule).
    pub red_ambrosia_timer_g: f64,
    /// Advanced `red_ambrosia_time`.
    pub red_ambrosia_time: f64,
    /// Advanced `red_ambrosia`.
    pub red_ambrosia: f64,
    /// Advanced `lifetime_red_ambrosia`.
    pub lifetime_red_ambrosia: f64,
    /// Bonus blueberry time minted this tick — the caller feeds it into
    /// a recursive ambrosia advance.
    pub bonus_ambrosia_time: f64,
    /// `RedAmbrosiaGained` when the granule was crossed; empty otherwise.
    pub events: SmallVec<[CoreEvent; 1]>,
}

/// Red-ambrosia case of `addTimers`. Mirrors the ambrosia case shape;
/// additionally accumulates `bonus_ambrosia_time`
/// (`red_to_gain × ambrosia_time_per_red_ambrosia`) that the caller feeds
/// into the ambrosia timer afterward.
pub(crate) fn advance_red_ambrosia_timer(
    input: &AdvanceRedAmbrosiaTimerInput,
    rng: &mut impl Rng,
) -> AdvanceRedAmbrosiaTimerResult {
    if input.no_ambrosia_upgrades_completions <= 0.0 {
        return AdvanceRedAmbrosiaTimerResult {
            red_ambrosia_timer_g: input.red_ambrosia_timer_g,
            red_ambrosia_time: input.red_ambrosia_time,
            red_ambrosia: input.red_ambrosia,
            lifetime_red_ambrosia: input.lifetime_red_ambrosia,
            bonus_ambrosia_time: 0.0,
            events: SmallVec::new(),
        };
    }

    let mut red_ambrosia_timer_g = input.red_ambrosia_timer_g + input.dt * input.time_multiplier;
    if red_ambrosia_timer_g < AMBROSIA_TIMER_GRANULE {
        return AdvanceRedAmbrosiaTimerResult {
            red_ambrosia_timer_g,
            red_ambrosia_time: input.red_ambrosia_time,
            red_ambrosia: input.red_ambrosia,
            lifetime_red_ambrosia: input.lifetime_red_ambrosia,
            bonus_ambrosia_time: 0.0,
            events: SmallVec::new(),
        };
    }

    let mut red_ambrosia_time = input.red_ambrosia_time
        + (8.0 * red_ambrosia_timer_g).floor() / 8.0 * input.red_ambrosia_generation_speed;
    red_ambrosia_timer_g %= AMBROSIA_TIMER_GRANULE;

    let mut red_ambrosia = input.red_ambrosia;
    let mut lifetime_red_ambrosia = input.lifetime_red_ambrosia;
    let mut total_gained = 0.0;
    let mut bonus_ambrosia_time = 0.0;

    let mut time_to_red =
        calculate_required_red_ambrosia_time(&CalculateRequiredRedAmbrosiaTimeInput {
            time_per_red_ambrosia: input.time_per_red_ambrosia,
            lifetime_red_ambrosia,
            bar_requirement_multiplier: input.bar_requirement_multiplier,
        });

    while red_ambrosia_time >= time_to_red {
        let value = next_f64(rng);
        let luck_per_100 = input.red_ambrosia_luck / 100.0;
        let red_mult = luck_per_100.floor();
        let luck_mult = if value < luck_per_100 - red_mult {
            1.0
        } else {
            0.0
        };
        let red_to_gain = red_mult + luck_mult;

        red_ambrosia += red_to_gain;
        lifetime_red_ambrosia += red_to_gain;
        total_gained += red_to_gain;
        bonus_ambrosia_time += red_to_gain * input.ambrosia_time_per_red_ambrosia;
        red_ambrosia_time -= time_to_red;

        time_to_red =
            calculate_required_red_ambrosia_time(&CalculateRequiredRedAmbrosiaTimeInput {
                time_per_red_ambrosia: input.time_per_red_ambrosia,
                lifetime_red_ambrosia,
                bar_requirement_multiplier: input.bar_requirement_multiplier,
            });
    }

    let mut events = SmallVec::new();
    events.push(CoreEvent::RedAmbrosiaGained {
        amount: total_gained,
    });

    AdvanceRedAmbrosiaTimerResult {
        red_ambrosia_timer_g,
        red_ambrosia_time,
        red_ambrosia,
        lifetime_red_ambrosia,
        bonus_ambrosia_time,
        events,
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_xoshiro::Xoshiro256PlusPlus;

    use super::*;

    #[test]
    fn reset_counter_advances_by_dt_times_mult() {
        assert_eq!(advance_reset_counter(5.0, 2.0, 3.0), 11.0);
        // Identity multiplier → linear in dt.
        assert_eq!(advance_reset_counter(0.0, 0.025, 1.0), 0.025);
    }

    #[test]
    fn ascension_counter_scales_but_real_does_not() {
        let r = advance_ascension_timer(&AdvanceAscensionTimerInput {
            dt: 2.0,
            ascension_counter: 10.0,
            ascension_counter_real: 10.0,
            ascension_speed_multi: 3.0,
        });
        assert_eq!(r.ascension_counter, 16.0); // 10 + 2*3
        assert_eq!(r.ascension_counter_real, 12.0); // 10 + 2
    }

    #[test]
    fn singularity_challenge_timer_accumulates_only_when_inside() {
        let inside = advance_singularity_timer(&AdvanceSingularityTimerInput {
            dt: 2.0,
            ascension_counter_real_real: 0.0,
            singularity_counter: 0.0,
            sing_challenge_timer: 5.0,
            inside_singularity_challenge: true,
            singularity_speed_multi: 2.0,
        });
        assert_eq!(inside.ascension_counter_real_real, 2.0);
        assert_eq!(inside.singularity_counter, 4.0); // 0 + 2*2
        assert_eq!(inside.sing_challenge_timer, 9.0); // 5 + 2*2

        let outside = advance_singularity_timer(&AdvanceSingularityTimerInput {
            dt: 2.0,
            ascension_counter_real_real: 0.0,
            singularity_counter: 0.0,
            sing_challenge_timer: 5.0,
            inside_singularity_challenge: false,
            singularity_speed_multi: 2.0,
        });
        assert_eq!(outside.sing_challenge_timer, 0.0); // reset
    }

    #[test]
    fn quarks_timer_clamps_at_max() {
        assert_eq!(
            advance_quarks_timer(&AdvanceQuarksTimerInput {
                dt: 10.0,
                quarks_timer: 5.0,
                max_quark_timer: 100.0,
            }),
            15.0
        );
        // Clamps when it would overshoot.
        assert_eq!(
            advance_quarks_timer(&AdvanceQuarksTimerInput {
                dt: 10.0,
                quarks_timer: 95.0,
                max_quark_timer: 100.0,
            }),
            100.0
        );
    }

    #[test]
    fn golden_quarks_timer_disabled_when_export_zero() {
        // export == 0 → untouched.
        assert_eq!(
            advance_golden_quarks_timer(&AdvanceGoldenQuarksTimerInput {
                dt: 10.0,
                golden_quarks_timer: 42.0,
                export_gq_per_hour: 0.0,
            }),
            42.0
        );
        // export > 0 → accumulates.
        assert_eq!(
            advance_golden_quarks_timer(&AdvanceGoldenQuarksTimerInput {
                dt: 10.0,
                golden_quarks_timer: 42.0,
                export_gq_per_hour: 5.0,
            }),
            52.0
        );
        // Clamps at the 168-hour cap.
        assert_eq!(
            advance_golden_quarks_timer(&AdvanceGoldenQuarksTimerInput {
                dt: 100.0,
                golden_quarks_timer: GOLDEN_QUARKS_TIMER_CAP_SECONDS - 10.0,
                export_gq_per_hour: 5.0,
            }),
            GOLDEN_QUARKS_TIMER_CAP_SECONDS
        );
    }

    fn octeract_input(dt: f64, octeract_timer: f64) -> AdvanceOcteractTimerInput {
        AdvanceOcteractTimerInput {
            dt,
            time_multiplier: 1.0,
            octeract_unlocked: true,
            octeract_timer,
            wow_octeracts: Decimal::zero(),
            total_wow_octeracts: Decimal::zero(),
            golden_quarks: Decimal::zero(),
            quarks_this_singularity: 0.0,
            per_second: 2.0,
            highest_singularity_count: 0.0,
            singularity_count: 0.0,
            golden_quarks_multiplier_excluding_base: 1.0,
        }
    }

    #[test]
    fn octeract_timer_locked_is_inert() {
        let r = advance_octeract_timer(&AdvanceOcteractTimerInput {
            octeract_unlocked: false,
            wow_octeracts: Decimal::from_finite(10.0),
            ..octeract_input(5.0, 0.5)
        });
        assert_eq!(r.octeract_timer, 0.5);
        assert_eq!(r.wow_octeracts.to_number(), 10.0);
        assert!(r.events.is_empty());
    }

    #[test]
    fn octeract_timer_accumulates_below_one_second() {
        let r = advance_octeract_timer(&octeract_input(0.3, 0.5));
        assert!((r.octeract_timer - 0.8).abs() < 1e-9);
        assert_eq!(r.wow_octeracts.to_number(), 0.0);
        assert!(r.events.is_empty());
    }

    #[test]
    fn octeract_timer_credits_giveaways_below_sing_160() {
        // 0.5 + 3.0 = 3.5 → 3 giveaways, 0.5 remainder; wow += 3 × 2 = 6.
        let r = advance_octeract_timer(&AdvanceOcteractTimerInput {
            golden_quarks: Decimal::from_finite(100.0),
            quarks_this_singularity: 1e6,
            highest_singularity_count: 100.0,
            singularity_count: 100.0,
            ..octeract_input(3.0, 0.5)
        });
        assert!((r.octeract_timer - 0.5).abs() < 1e-9);
        assert_eq!(r.wow_octeracts.to_number(), 6.0);
        assert_eq!(r.total_wow_octeracts.to_number(), 6.0);
        // Below singularity 160 → GQ giveaway skipped; balances unchanged.
        assert_eq!(r.golden_quarks.to_number(), 100.0);
        assert_eq!(r.quarks_this_singularity, 1e6);
        assert_eq!(r.events.len(), 1);
        assert!(matches!(
            r.events[0],
            CoreEvent::OcteractTickFired {
                amount_of_giveaways: 3
            }
        ));
    }

    #[test]
    fn octeract_timer_siphons_quarks_above_sing_160() {
        // highest sing 160 → only the 160 threshold met → actual_level 1,
        // quark_fraction 1e-6. One giveaway second.
        let r = advance_octeract_timer(&AdvanceOcteractTimerInput {
            quarks_this_singularity: 1e6,
            per_second: 0.0,
            highest_singularity_count: 160.0,
            ..octeract_input(1.0, 0.0)
        });
        // qts decays by (1 - 1e-6).
        assert!((r.quarks_this_singularity - 1e6 * (1.0 - 1e-6)).abs() < 1e-3);
        // Some golden quarks were siphoned in.
        assert!(r.golden_quarks.to_number() > 0.0);
        assert_eq!(r.events.len(), 1);
    }

    fn auto_potion_input(dt: f64) -> AdvanceAutoPotionTimerInput {
        AdvanceAutoPotionTimerInput {
            dt,
            time_multiplier: 1.0,
            highest_singularity_count: 6.0,
            auto_potion_timer: 0.0,
            auto_potion_timer_obtainium: 0.0,
            toggle_offering: false,
            toggle_obtainium: false,
            offering_potion_count: 0.0,
            obtainium_potion_count: 0.0,
            auto_potion_speed_mult: 1.0,
        }
    }

    #[test]
    fn auto_potion_locked_below_sing_6() {
        let r = advance_auto_potion_timer(&AdvanceAutoPotionTimerInput {
            highest_singularity_count: 5.0,
            ..auto_potion_input(1000.0)
        });
        assert_eq!(r.auto_potion_timer, 0.0);
        assert_eq!(r.auto_potion_timer_obtainium, 0.0);
        assert!(r.events.is_empty());
    }

    #[test]
    fn auto_potion_normal_mode_fires_both_sides() {
        // threshold = 180 × 1.03^-6 ≈ 150.75; dt 200 crosses both timers.
        let r = advance_auto_potion_timer(&auto_potion_input(200.0));
        assert_eq!(r.events.len(), 2);
        // Both fire in normal (non-fast) mode.
        assert!(r.events.iter().all(|e| matches!(
            e,
            CoreEvent::AutoPotionFired {
                fast_mode: false,
                ..
            }
        )));
        // One offering, one obtainium.
        assert!(r.events.iter().any(|e| matches!(
            e,
            CoreEvent::AutoPotionFired {
                potion_type: AutoPotionType::Offering,
                ..
            }
        )));
        assert!(r.events.iter().any(|e| matches!(
            e,
            CoreEvent::AutoPotionFired {
                potion_type: AutoPotionType::Obtainium,
                ..
            }
        )));
        // The fired timer resets below its (large) threshold.
        assert!(r.auto_potion_timer < 200.0);
    }

    #[test]
    fn auto_potion_fast_offering_only() {
        // Offering fast mode → tiny threshold (min(1, thr)/20 = 0.05);
        // obtainium keeps the large normal threshold, uncrossed by dt 0.1.
        let r = advance_auto_potion_timer(&AdvanceAutoPotionTimerInput {
            toggle_offering: true,
            offering_potion_count: 5.0,
            ..auto_potion_input(0.1)
        });
        assert_eq!(r.events.len(), 1);
        assert!(matches!(
            r.events[0],
            CoreEvent::AutoPotionFired {
                potion_type: AutoPotionType::Offering,
                fast_mode: true,
                ..
            }
        ));
    }

    fn ambrosia_input(dt: f64) -> AdvanceAmbrosiaTimerInput {
        AdvanceAmbrosiaTimerInput {
            dt,
            time_multiplier: 1.0,
            no_singularity_upgrades_completions: 1.0,
            ambrosia_generation_speed: 1.0,
            ambrosia_timer_g: 0.0,
            blueberry_time: 0.0,
            ambrosia: 0.0,
            lifetime_ambrosia: 0.0,
            ambrosia_luck: 200.0, // integer mult (2/bar) → RNG-independent gain
            bonus_ambrosia: 0.0,
            time_per_ambrosia: 45.0,
            accelerator_mult: 1.0,
            brick_of_lead_mult: 1.0,
        }
    }

    #[test]
    fn ambrosia_gated_when_feature_locked() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(0);
        let r = advance_ambrosia_timer(
            &AdvanceAmbrosiaTimerInput {
                no_singularity_upgrades_completions: 0.0,
                ..ambrosia_input(1000.0)
            },
            &mut rng,
        );
        assert_eq!(r.ambrosia, 0.0);
        assert_eq!(r.ambrosia_timer_g, 0.0);
        assert!(r.events.is_empty());
    }

    #[test]
    fn ambrosia_gated_when_generation_speed_zero() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(0);
        let r = advance_ambrosia_timer(
            &AdvanceAmbrosiaTimerInput {
                ambrosia_generation_speed: 0.0,
                ..ambrosia_input(1000.0)
            },
            &mut rng,
        );
        assert_eq!(r.ambrosia, 0.0);
        assert!(r.events.is_empty());
    }

    #[test]
    fn ambrosia_sub_granule_accumulates_only() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(0);
        // dt 0.1 < 0.125 granule → accumulate timer, no bar, no event.
        let r = advance_ambrosia_timer(&ambrosia_input(0.1), &mut rng);
        assert!((r.ambrosia_timer_g - 0.1).abs() < 1e-9);
        assert_eq!(r.ambrosia, 0.0);
        assert!(r.events.is_empty());
    }

    #[test]
    fn ambrosia_credits_full_bars() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(0);
        // dt 1000 → blueberry_time 1000; base threshold 45 → ~22 bars × 2.
        let r = advance_ambrosia_timer(&ambrosia_input(1000.0), &mut rng);
        assert!(r.ambrosia > 0.0);
        assert!((r.ambrosia % 2.0).abs() < 1e-9); // luck 200 → multiples of 2
        assert_eq!(r.lifetime_ambrosia, r.ambrosia); // both started at 0
        assert_eq!(r.events.len(), 1);
        match r.events[0] {
            CoreEvent::AmbrosiaGained { amount } => assert_eq!(amount, r.ambrosia),
            _ => panic!("expected AmbrosiaGained"),
        }
    }

    fn red_ambrosia_input(dt: f64) -> AdvanceRedAmbrosiaTimerInput {
        AdvanceRedAmbrosiaTimerInput {
            dt,
            time_multiplier: 1.0,
            no_ambrosia_upgrades_completions: 1.0,
            red_ambrosia_generation_speed: 1.0,
            red_ambrosia_timer_g: 0.0,
            red_ambrosia_time: 0.0,
            red_ambrosia: 0.0,
            lifetime_red_ambrosia: 0.0,
            red_ambrosia_luck: 200.0,
            ambrosia_time_per_red_ambrosia: 10.0,
            time_per_red_ambrosia: 100_000.0,
            bar_requirement_multiplier: 1.0,
        }
    }

    #[test]
    fn red_ambrosia_gated_when_locked() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(0);
        let r = advance_red_ambrosia_timer(
            &AdvanceRedAmbrosiaTimerInput {
                no_ambrosia_upgrades_completions: 0.0,
                ..red_ambrosia_input(500_000.0)
            },
            &mut rng,
        );
        assert_eq!(r.red_ambrosia, 0.0);
        assert_eq!(r.bonus_ambrosia_time, 0.0);
        assert!(r.events.is_empty());
    }

    #[test]
    fn red_ambrosia_credits_and_feeds_bonus_time() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(0);
        // dt 200_000 → red_ambrosia_time 200_000; base threshold 100_000.
        let r = advance_red_ambrosia_timer(&red_ambrosia_input(200_000.0), &mut rng);
        assert!(r.red_ambrosia > 0.0);
        // bonus_ambrosia_time = total_gained × ambrosia_time_per_red_ambrosia (10).
        assert!((r.bonus_ambrosia_time - r.red_ambrosia * 10.0).abs() < 1e-6);
        assert_eq!(r.events.len(), 1);
    }
}
