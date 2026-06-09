#![cfg_attr(not(test), deny(clippy::unwrap_used))]

//! Synergism Forkd — headless game logic.
//!
//! Public surface organized by tier:
//! - **Composed state** — [`GameState`] (the `&GameState` boundary the tick
//!   orchestrator threads through every per-tick call). Per-family slices
//!   stay accessible under [`state`].
//! - **Events** — [`CoreEvent`] (closed enum, mirrors the legacy TS
//!   `CoreEvent` union) plus the supporting kind enums.
//! - **Per-tick aggregators** — four `(state, pre)` entry points:
//!   [`compute_global_multipliers`], [`update_all_multiplier`],
//!   [`update_all_tick`], [`resource_gain`]. Each takes a `&GameState`
//!   plus a `*Pre` bundle of cross-mechanic pre-computed values.
//! - **Math** — RNG, sigmoid, summation, integer-step helpers re-exported
//!   from [`math`].
//! - **Mechanics** — leaf formulas and `buy_*` purchase loops; accessed via
//!   [`mechanics`] sub-modules (per-family namespacing).
//!
//! Subdirs mirror the legacy `packages/logic/src/` tree. Boundary rules
//! carry over: no DOM, no UI imports, no i18n, no modal helpers. Public
//! functions follow the `(state, input) -> (state, events)` shape so side
//! effects are routed through the UI tier.

use synergismforkd_common as _;

pub mod currency;
pub mod events;
pub mod math;
pub mod mechanics;
pub mod state;
pub mod tick;

// ─── Composed loop-edge types ────────────────────────────────────────────

pub use state::{GameState, RngPurpose, RngState};

// ─── Events ──────────────────────────────────────────────────────────────

pub use events::{
    AchievementGroup, AutoPotionType, AutoResetMode, AutoResetTier, AutoTool, CoreEvent,
    ProducerType, RevealTrigger, SweepState, UpgradeTier,
};

// ─── Currency newtypes ───────────────────────────────────────────────────

pub use currency::{Coins, Multiplier, PrestigePoints, ReincarnationPoints, TranscendPoints};

// ─── Tick orchestrator ───────────────────────────────────────────────────

pub use tick::{daily_reset, tack, AutomationPre, BuyRequest, PlayerAction, TackInput, TickOutput};

// Host seam: rebuild the achievement-points total from a loaded bitmap (H5).
pub use mechanics::achievement_points::recompute_achievement_points;

// ─── Per-tick aggregator entry points ────────────────────────────────────

pub use mechanics::global_multipliers::{
    compute_global_multipliers, GlobalMultipliersPreEvaluated, GlobalMultipliersResult,
};
pub use mechanics::resource_gain::{
    resource_gain, AscendBuildingProduction, ResourceGainPre, ResourceGainResult,
};
pub use mechanics::update_all_multiplier::{
    update_all_multiplier, UpdateAllMultiplierPre, UpdateAllMultiplierResult,
};
pub use mechanics::update_all_tick::{update_all_tick, UpdateAllTickPre, UpdateAllTickResult};

// ─── Math helpers ────────────────────────────────────────────────────────

pub use math::rng::{next_f64, next_inclusive, pick};
pub use math::sigmoid::{calculate_sigmoid, calculate_sigmoid_exponential};
pub use math::smallest_inc::{smallest_inc, MAX_SAFE_INTEGER};
pub use math::summations::{
    calculate_cubic_sum_data, calculate_summation_cubic, calculate_summation_non_linear,
    solve_quadratic, CalculateCubicSumDataResult, CalculateSummationNonLinearResult,
    SummationError,
};

#[cfg(test)]
mod tests {
    //! Public-API smoke tests — exercise the loop-edge boundary using only
    //! crate-root re-exports, to keep that surface honest.

    use super::*;

    #[test]
    fn loop_edge_aggregators_compose_against_default_state() {
        // Demonstrates the per-tick orchestration shape: thread one
        // `&GameState` through all four aggregators, using identity `Pre`
        // bundles so the result collapses to a baseline-deterministic
        // state with no schema reads exploding.
        let state = GameState::default();

        let mult = update_all_multiplier(&state, &UpdateAllMultiplierPre::default());
        assert_eq!(mult.free_multiplier, 0.0);

        let tick = update_all_tick(&state, &UpdateAllTickPre::default(), mult.total_multiplier);
        assert_eq!(tick.free_accelerator, 0.0);
        assert_eq!(tick.cost_divisor, 1.0);

        let global = compute_global_multipliers(&state, &GlobalMultipliersPreEvaluated::default());
        assert_eq!(global.global_coin_multiplier.to_number(), 1.0);

        let resource = resource_gain(&state, &ResourceGainPre::default(), 0.025);
        assert_eq!(resource.coins.to_number(), 0.0);
        assert!(resource.events.is_empty());
    }

    #[test]
    fn rng_state_round_trips_through_the_public_api() {
        // Verify the per-purpose RNG layer is reachable from the crate
        // root and produces deterministic sequences across runs.
        let mut a = RngState::from_seed(42);
        let mut b = RngState::from_seed(42);
        for _ in 0..10 {
            let v1 = next_f64(a.draw(RngPurpose::Ambrosia));
            let v2 = next_f64(b.draw(RngPurpose::Ambrosia));
            assert_eq!(v1, v2);
        }
    }

    #[test]
    fn core_event_pattern_matches_with_supporting_enums() {
        // Compile-time check that the supporting enums are usable
        // alongside `CoreEvent` from the crate root.
        let event = CoreEvent::AutoResetTriggered {
            tier: AutoResetTier::Ascension,
            mode: AutoResetMode::Time,
        };
        match event {
            CoreEvent::AutoResetTriggered { tier, mode } => {
                assert_eq!(tier, AutoResetTier::Ascension);
                assert_eq!(mode, AutoResetMode::Time);
            }
            _ => panic!("variant mismatch"),
        }
    }
}
