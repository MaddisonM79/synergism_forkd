//! synergismforkd-sim — headless game simulator CLI.
//!
//! Runs N ticks against a fresh game state and prints the per-variant
//! event tally + a few final-state stats. Useful for ad-hoc balance
//! checks, reproducing reported bugs, and as a bench-harness driver.
//!
//! Usage: `synergismforkd-sim [ticks] [dt]` (defaults: 1000 ticks, 0.025s).

// Held for parity/snapshot drivers added later; not yet named here.
use synergismforkd_bignum as _;
use synergismforkd_logic as _;
use synergismforkd_save as _;

use synergismforkd_testkit::{run_sim, SimConfig};

// serde / serde_json are dev-deps used only by the lib's parity tests;
// silence `unused_crate_dependencies` for this bin's test target.
#[cfg(test)]
use serde as _;
#[cfg(test)]
use serde_json as _;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ticks: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1_000);
    let dt: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.025);

    let config = SimConfig {
        ticks,
        dt,
        time_warp: false,
    };
    let report = run_sim(&config);

    println!("synergismforkd-sim — {ticks} ticks @ dt={dt}s");
    println!("total events: {}", report.total_events);
    if report.event_counts.is_empty() {
        println!("  (no events)");
    } else {
        for (kind, count) in &report.event_counts {
            println!("  {kind}: {count}");
        }
    }

    let s = &report.final_state;
    println!("final state:");
    println!(
        "  prestige_counter          = {}",
        s.reset_counters.prestige_counter
    );
    println!(
        "  ascension_counter_real    = {}",
        s.reset_counters.ascension_counter_real
    );
    println!("  quarks_timer              = {}", s.quarks.quarks_timer);
    println!(
        "  reincarnation reset timer = {}",
        s.automation.auto_reset_timer_reincarnation
    );
    println!(
        "  ant crumbs                = {}",
        s.ants.crumbs.to_number()
    );
}
