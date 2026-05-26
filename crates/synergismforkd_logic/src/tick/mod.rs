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
    compute_global_multipliers, GlobalMultipliersPreEvaluated,
};
use crate::mechanics::multipliers::{buy_multiplier, BuyMultiplierInput};
use crate::mechanics::particle_buildings::{buy_particle_building, BuyParticleBuildingInput};
use crate::mechanics::producers::{buy_max, buy_producer, BuyMaxInput, BuyProducerInput};
use crate::mechanics::resource_gain::{resource_gain, ResourceGainPre};
use crate::mechanics::tesseract_buildings::{buy_tesseract_building, BuyTesseractBuildingInput};
use crate::mechanics::update_all_multiplier::{update_all_multiplier, UpdateAllMultiplierPre};
use crate::mechanics::update_all_tick::{update_all_tick, UpdateAllTickPre};
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

/// Run one tick.
///
/// Phase ordering is canonical — see module docs. Reordering is a design
/// change requiring a separate commit and an updated CLAUDE.md note.
pub fn tack(state: &mut GameState, input: &TackInput) -> TickOutput {
    let mut output = TickOutput::default();

    let cache = phase_cross_mechanic_precompute(state, input);
    phase_global_state(state, &cache);
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
/// **Today the cache is built by forwarding the four `*Pre` bundles
/// from [`TackInput`].** As each upstream effect ports (rune effects,
/// ant effects, hepteract effects, achievement rewards, challenge-15
/// rewards), its computation moves into this function — reading from
/// `state` — and the corresponding caller-provided `*Pre` field shrinks
/// or disappears. The cache is the migration target; `TackInput` is the
/// temporary input mechanism.
fn phase_cross_mechanic_precompute(_state: &GameState, input: &TackInput) -> CrossMechanicCache {
    // TODO(cross-mechanic-cache): replace each field with a
    // compute-from-state once the upstream mechanic ports. The
    // forwarding shape lets phase consumers read from the cache today
    // even though every field is still caller-provided. All four
    // `*Pre` bundles are `Copy`, so this is a cheap struct-of-copies.
    CrossMechanicCache {
        global_multipliers_pre: input.global_multipliers_pre,
        update_all_multiplier_pre: input.update_all_multiplier_pre,
        update_all_tick_pre: input.update_all_tick_pre,
        resource_gain_pre: input.resource_gain_pre,
    }
}

/// **Phase 2** — Global state aggregators.
///
/// Reads the precomputed bundles out of [`CrossMechanicCache`] and runs
/// the four pure aggregators in dependency order. Aggregator outputs
/// have no home on [`GameState`] until a `g_cache` slice lands (a
/// state-schema change requiring user permission per CLAUDE.md); for
/// now they're dropped, but the calls are kept so any panic /
/// arithmetic-overflow regressions surface in this tick rather than
/// later.
fn phase_global_state(state: &mut GameState, cache: &CrossMechanicCache) {
    let _ = compute_global_multipliers(state, &cache.global_multipliers_pre);
    let mult = update_all_multiplier(state, &cache.update_all_multiplier_pre);
    let _ = update_all_tick(state, &cache.update_all_tick_pre, mult.total_multiplier);
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
