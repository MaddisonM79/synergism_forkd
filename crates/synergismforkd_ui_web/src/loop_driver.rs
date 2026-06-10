//! The 20 Hz game loop: one `tack` per iteration inside exactly one
//! `state.write()`, action/host-command queue drains, autosave via the
//! SaveHost, and catch-up routing for large dt gaps (throttled tabs).

use std::time::Duration;

use dioxus::prelude::*;
use synergismforkd_logic::TackInput;
use synergismforkd_ui::bridge::{DerivedStats, GameBridge, HostCommand};
use synergismforkd_ui::events_map;
use synergismforkd_ui::gating::Route;

use crate::catch_up::{self, CatchUpHandles};
use crate::platform;
use crate::save_host::{web::LocalStorageBackend, BootOutcome, SaveHost};

/// Tick cadence (the legacy 50 ms fast loop).
const TICK_MS: u64 = 50;

/// A dt above this is a throttled/suspended tab, not a frame — route it
/// through the catch-up chunker instead of one giant tick.
const CATCH_UP_DT_S: f64 = 2.0;

/// Run forever. Spawned once from the web root.
pub async fn run(
    bridge: GameBridge,
    mut host: SaveHost<LocalStorageBackend>,
    outcome: BootOutcome,
    handles: CatchUpHandles,
) {
    // Surface the boot outcome before the first tick.
    match outcome {
        BootOutcome::Corrupt => bridge.toast_warn("toasts.save_corrupt"),
        BootOutcome::Loaded {
            saved_at_ms: Some(saved_at_ms),
        } => {
            let elapsed_s = (platform::now_ms().saturating_sub(saved_at_ms)) as f64 / 1000.0;
            catch_up::run(bridge, handles, elapsed_s).await;
            host.persist(&bridge.state.peek(), platform::now_ms());
        }
        _ => {}
    }

    let mut last_s = platform::perf_now_s();
    let mut was_hidden = platform::document_hidden();

    loop {
        gloo_timers::future::sleep(Duration::from_millis(TICK_MS)).await;
        let now_s = platform::perf_now_s();
        let mut dt = (now_s - last_s).max(0.0);
        last_s = now_s;

        // Throttled-tab resume: simulate the gap as offline time, then
        // resume normal ticking.
        if dt > CATCH_UP_DT_S {
            catch_up::run(bridge, handles, dt).await;
            dt = 0.0;
        }

        // Drain queued player actions into this tick.
        let mut input = TackInput {
            dt,
            ..TackInput::default()
        };
        {
            let mut actions = bridge.actions;
            let mut actions = actions.write();
            if !actions.is_empty() {
                input.player_actions.extend(actions.drain(..));
            }
        }

        // THE tick: one state.write() per iteration.
        let output = {
            let mut state = bridge.state;
            let mut state = state.write();
            host.tick(&mut state, &input, platform::now_ms())
        };

        // HUD derived numbers — only notify when they actually changed.
        let derived = DerivedStats {
            coins_per_sec: output.derived.coins_per_sec,
            prestige_gain: output.derived.prestige_point_gain,
        };
        if *bridge.derived.peek() != derived {
            let mut signal = bridge.derived;
            signal.set(derived);
        }

        events_map::apply(&bridge, &output.events);

        handle_host_commands(&bridge, &mut host).await;

        // Force-save on the visible→hidden edge (tab switch / close).
        let hidden = platform::document_hidden();
        if hidden && !was_hidden {
            host.persist(&bridge.state.peek(), platform::now_ms());
        }
        was_hidden = hidden;
    }
}

/// Execute queued host commands (storage / clipboard side effects).
async fn handle_host_commands(bridge: &GameBridge, host: &mut SaveHost<LocalStorageBackend>) {
    let commands: Vec<HostCommand> = {
        let mut queue = bridge.host;
        let mut queue = queue.write();
        if queue.is_empty() {
            return;
        }
        queue.drain(..).collect()
    };

    for command in commands {
        match command {
            HostCommand::ExportSave => {
                let blob = {
                    let mut state = bridge.state;
                    let mut state = state.write();
                    host.export(&mut state, platform::now_ms()).1
                };
                match blob {
                    Some(blob) if platform::clipboard_write(&blob).await => {
                        bridge.toast_success("toasts.export_ok");
                    }
                    _ => bridge.toast_warn("toasts.export_fail"),
                }
            }
            HostCommand::ImportSave(blob) => match host.import(&blob, platform::now_ms()) {
                Some(new_state) => {
                    let mut state = bridge.state;
                    state.set(new_state);
                    // An earlier-progression import may have regressed
                    // visibility — re-clamp wherever the player is.
                    bridge.navigate(*bridge.route.peek());
                    bridge.toast_success("toasts.import_ok");
                }
                None => bridge.toast_warn("toasts.import_bad"),
            },
            HostCommand::ForceSave => {
                host.persist(&bridge.state.peek(), platform::now_ms());
            }
            HostCommand::HardReset => {
                let fresh = host.reset();
                let mut state = bridge.state;
                state.set(fresh);
                bridge.navigate(Route::default());
                bridge.toast_info("toasts.hard_reset_done");
            }
        }
    }
}
