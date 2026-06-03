//! Synergism Forkd — test fixtures, mock state builders, seeded RNG,
//! headless sim runner, and parity-snapshot helpers.
//!
//! Imported by every other crate's tests. The companion
//! `synergismforkd-sim` binary drives the same [`run_sim`] loop from the
//! CLI for ad-hoc balance checks and bug reproduction.

use std::collections::BTreeMap;

// Held for the parity-snapshot helpers + fixtures that build on this
// runner; not yet referenced by name.
use synergismforkd_bignum as _;
use synergismforkd_save as _;

use synergismforkd_logic::{tack, CoreEvent, GameState, TackInput};

/// Configuration for a headless sim run.
#[derive(Debug, Clone)]
pub struct SimConfig {
    /// Number of ticks to run.
    pub ticks: u64,
    /// Seconds per tick (`G.deltaT` equivalent).
    pub dt: f64,
    /// Run ticks as offline catch-up (skips the head + middle automation,
    /// runs only the tail — see [`synergismforkd_logic::tack`]).
    pub time_warp: bool,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            ticks: 1_000,
            dt: 0.025,
            time_warp: false,
        }
    }
}

/// Summary of a headless sim run.
#[derive(Debug, Clone)]
pub struct SimReport {
    /// Ticks executed.
    pub ticks: u64,
    /// Total `CoreEvent`s emitted across the run.
    pub total_events: u64,
    /// Per-variant event tally (variant name → count), sorted by name.
    pub event_counts: BTreeMap<String, u64>,
    /// Final composed game state after the run.
    pub final_state: GameState,
}

/// Run `config.ticks` ticks against a fresh [`GameState`], threading the
/// same default [`TackInput`] each tick, and tally the emitted events.
///
/// This is the driver harness: it exercises the full `tack` loop
/// end-to-end. Fixture loading, per-tick input variation, and TS-parity
/// snapshotting build on top of it.
#[must_use]
pub fn run_sim(config: &SimConfig) -> SimReport {
    let mut state = GameState::default();
    let input = TackInput {
        dt: config.dt,
        time_warp: config.time_warp,
        ..TackInput::default()
    };

    let mut event_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut total_events = 0u64;

    for _ in 0..config.ticks {
        let output = tack(&mut state, &input);
        for event in &output.events {
            total_events += 1;
            *event_counts.entry(event_kind(event)).or_insert(0) += 1;
        }
    }

    SimReport {
        ticks: config.ticks,
        total_events,
        event_counts,
        final_state: state,
    }
}

/// The `CoreEvent` variant name — the leading identifier of its `Debug`
/// rendering (before any ` ` or `{`). Avoids a ~30-arm match against the
/// `#[non_exhaustive]` event enum.
fn event_kind(event: &CoreEvent) -> String {
    let rendered = format!("{event:?}");
    rendered
        .split([' ', '{'])
        .next()
        .unwrap_or("Unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_runs_and_tallies_events() {
        let report = run_sim(&SimConfig {
            ticks: 100,
            dt: 0.025,
            time_warp: false,
        });
        assert_eq!(report.ticks, 100);
        // Default state: the obtainium-recompute intent fires every tick.
        assert!(report.total_events >= 100);
        assert!(report
            .event_counts
            .contains_key("ObtainiumMultiplierRecomputeRequested"));
    }

    #[test]
    fn time_warp_run_skips_head_and_middle() {
        let report = run_sim(&SimConfig {
            ticks: 100,
            dt: 0.025,
            time_warp: true,
        });
        // Head + middle are skipped under warp; the default-state tail
        // emits nothing → a silent run.
        assert_eq!(report.total_events, 0);
    }

    #[test]
    fn head_timers_advance_over_a_run() {
        let report = run_sim(&SimConfig {
            ticks: 40,
            dt: 0.025,
            time_warp: false,
        });
        // 40 × 0.025 = 1.0s accrued on the (identity-mult) reset counters.
        assert!((report.final_state.reset_counters.prestige_counter - 1.0).abs() < 1e-9);
    }
}
